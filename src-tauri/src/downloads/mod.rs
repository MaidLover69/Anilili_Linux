use crate::clients::http::build_http_client;
use crate::db;
use crate::error::AppError;
use crate::models::downloads::{
    HEADROOM_BYTES, P1080_BYTES, P360_BYTES, P480_BYTES, P720_BYTES,
};
use crate::models::{DownloadRecord, SubtitleItem};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StorageCheck {
    pub ok: bool,
    pub free_bytes: i64,
    pub needed_bytes: i64,
}

impl StorageCheck {
    pub fn new(ok: bool, free_bytes: i64, needed_bytes: i64) -> Self {
        Self {
            ok,
            free_bytes,
            needed_bytes,
        }
    }
}

pub fn get_download_dir() -> Result<PathBuf, AppError> {
    let app_dir = crate::db::get_app_dir()?;
    let downloads_dir = app_dir.join("downloads");
    fs::create_dir_all(&downloads_dir).map_err(|e| AppError::Other(e.to_string()))?;
    Ok(downloads_dir)
}

pub fn check_storage(quality: &str) -> Result<StorageCheck, AppError> {
    let dir = get_download_dir()?;
    let needed_bytes = match quality {
        "1080p" => P1080_BYTES,
        "720p" => P720_BYTES,
        "480p" => P480_BYTES,
        "360p" => P360_BYTES,
        _ => P720_BYTES,
    };

    #[cfg(target_family = "unix")]
    let free_bytes = {
        unsafe {
            let mut stat: libc::statvfs = std::mem::zeroed();
            let c_path = std::ffi::CString::new(dir.to_str().unwrap_or("/")).unwrap();
            if libc::statvfs(c_path.as_ptr(), &mut stat) == 0 {
                (stat.f_bavail as i64) * (stat.f_frsize as i64)
            } else {
                10_000_000_000i64
            }
        }
    };

    #[cfg(not(target_family = "unix"))]
    let free_bytes = 10_000_000_000i64;

    let ok = free_bytes >= (needed_bytes + HEADROOM_BYTES);
    Ok(StorageCheck {
        ok,
        free_bytes,
        needed_bytes,
    })
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect()
}

pub async fn run_download_task(
    pool: SqlitePool,
    record: DownloadRecord,
    stream_url: String,
    referer: Option<String>,
    origin: Option<String>,
    subtitles: Vec<SubtitleItem>,
) {
    let download_id = record.id.clone();
    let download_dir = match get_download_dir() {
        Ok(d) => d,
        Err(e) => {
            let _ = db::update_download_status(
                &pool,
                &download_id,
                "FAILED",
                Some(0.0),
                None,
                None,
                None,
                Some(&format!("Download dir error: {}", e)),
            )
            .await;
            return;
        }
    };

    let client = build_http_client();
    let clean_title = sanitize_filename(&record.series_title);
    let base_name = format!("{} - EP {:02}", clean_title, record.episode_num);
    let temp_video_path = download_dir.join(format!("{}.tmp.mp4", base_name));
    let final_mkv_path = download_dir.join(format!("{}.mkv", base_name));

    // Update status to DOWNLOADING
    let _ = db::update_download_status(
        &pool,
        &download_id,
        "DOWNLOADING",
        Some(0.05),
        None,
        None,
        None,
        None,
    )
    .await;

    // 1. Download Subtitle file(s)
    let mut downloaded_sub_path: Option<PathBuf> = None;
    for (idx, sub) in subtitles.iter().enumerate() {
        let sub_ext = if sub.format.contains("ass") { "ass" } else { "vtt" };
        let sub_filename = if idx == 0 {
            format!("{}.{}", base_name, sub_ext)
        } else {
            format!("{}.{}.{}", base_name, sanitize_filename(&sub.language), sub_ext)
        };
        let sub_path = download_dir.join(&sub_filename);

        if let Ok(res) = client.get(&sub.url).send().await {
            if let Ok(bytes) = res.bytes().await {
                if fs::write(&sub_path, &bytes).is_ok() && downloaded_sub_path.is_none() {
                    downloaded_sub_path = Some(sub_path);
                }
            }
        }
    }

    // 2. Download Video Stream
    let mut req = client.get(&stream_url);
    if let Some(ref r) = referer {
        req = req.header("Referer", r);
    }
    if let Some(ref o) = origin {
        req = req.header("Origin", o);
    }

    let is_hls = stream_url.contains(".m3u8");

    if is_hls {
        // If stream is HLS and ffmpeg is installed, invoke ffmpeg to download and mux directly into MKV with subtitles
        let has_ffmpeg = Command::new("ffmpeg").arg("-version").output().is_ok();
        if has_ffmpeg {
            let mut cmd = Command::new("ffmpeg");
            cmd.arg("-y");
            if let Some(ref r) = referer {
                cmd.arg("-headers").arg(format!("Referer: {}\r\n", r));
            }
            cmd.arg("-i").arg(&stream_url);

            if let Some(ref sub_path) = downloaded_sub_path {
                cmd.arg("-i").arg(sub_path);
                cmd.arg("-c").arg("copy");
                cmd.arg("-c:s").arg("srt");
            } else {
                cmd.arg("-c").arg("copy");
            }

            cmd.arg(&final_mkv_path);

            match cmd.output() {
                Ok(output) if output.status.success() => {
                    let file_size = fs::metadata(&final_mkv_path).map(|m| m.len() as i64).unwrap_or(0);
                    let _ = db::update_download_status(
                        &pool,
                        &download_id,
                        "COMPLETED",
                        Some(1.0),
                        Some(&final_mkv_path.to_string_lossy()),
                        Some(file_size),
                        None,
                        None,
                    )
                    .await;
                    return;
                }
                Ok(output) => {
                    info!("FFmpeg HLS muxing had status error: {:?}", String::from_utf8_lossy(&output.stderr));
                }
                Err(e) => {
                    info!("FFmpeg failed to launch: {:?}", e);
                }
            }
        }
    }

    // Direct HTTP chunked download fallback
    match req.send().await {
        Ok(mut res) => {
            if !res.status().is_success() {
                let _ = db::update_download_status(
                    &pool,
                    &download_id,
                    "FAILED",
                    Some(0.0),
                    None,
                    None,
                    None,
                    Some(&format!("HTTP status {}", res.status())),
                )
                .await;
                return;
            }

            let total_size = res.content_length().unwrap_or(0);
            let mut downloaded: u64 = 0;
            let mut file = match File::create(&temp_video_path) {
                Ok(f) => f,
                Err(e) => {
                    let _ = db::update_download_status(
                        &pool,
                        &download_id,
                        "FAILED",
                        Some(0.0),
                        None,
                        None,
                        None,
                        Some(&format!("File create error: {}", e)),
                    )
                    .await;
                    return;
                }
            };

            while let Ok(Some(chunk)) = res.chunk().await {
                if file.write_all(&chunk).is_err() {
                    break;
                }
                downloaded += chunk.len() as u64;
                if total_size > 0 {
                    let progress = (downloaded as f64 / total_size as f64).min(0.95);
                    let _ = db::update_download_status(
                        &pool,
                        &download_id,
                        "DOWNLOADING",
                        Some(progress),
                        None,
                        Some(downloaded as i64),
                        None,
                        None,
                    )
                    .await;
                }
            }

            let _ = file.flush();
            drop(file);

            // 3. Remux to MKV with embedded subtitle if ffmpeg is available
            let has_ffmpeg = Command::new("ffmpeg").arg("-version").output().is_ok();
            let mut converted = false;

            if has_ffmpeg {
                let mut cmd = Command::new("ffmpeg");
                cmd.arg("-y").arg("-i").arg(&temp_video_path);

                if let Some(ref sub_path) = downloaded_sub_path {
                    cmd.arg("-i").arg(sub_path);
                    cmd.arg("-c").arg("copy");
                    cmd.arg("-c:s").arg("srt");
                } else {
                    cmd.arg("-c").arg("copy");
                }

                cmd.arg(&final_mkv_path);

                if let Ok(out) = cmd.output() {
                    if out.status.success() {
                        let _ = fs::remove_file(&temp_video_path);
                        converted = true;
                    }
                }
            }

            let actual_path = if converted {
                final_mkv_path
            } else {
                // Rename temp to .mkv
                let _ = fs::rename(&temp_video_path, &final_mkv_path);
                final_mkv_path
            };

            let file_size = fs::metadata(&actual_path).map(|m| m.len() as i64).unwrap_or(downloaded as i64);
            let _ = db::update_download_status(
                &pool,
                &download_id,
                "COMPLETED",
                Some(1.0),
                Some(&actual_path.to_string_lossy()),
                Some(file_size),
                None,
                None,
            )
            .await;
        }
        Err(e) => {
            let _ = db::update_download_status(
                &pool,
                &download_id,
                "FAILED",
                Some(0.0),
                None,
                None,
                None,
                Some(&format!("Download error: {}", e)),
            )
            .await;
        }
    }
}
