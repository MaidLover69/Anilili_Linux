use crate::error::AppError;
use crate::models::{Category, SkipTimes, SourcesResult, StreamItem, SubtitleItem};
use base64::engine::general_purpose::{STANDARD, URL_SAFE, URL_SAFE_NO_PAD};
use base64::Engine;
use flate2::read::GzDecoder;
use reqwest::Client;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::Read;
use std::time::Duration;

const PIPE_OBF_KEY: [u8; 16] = [
    0x71, 0x95, 0x10, 0x34, 0xf8, 0xfb, 0xcf, 0x53, 0xd8, 0x9d, 0xb5, 0x2c, 0xeb, 0x3d, 0xc2, 0x2c,
];

const MIRURO_MIRRORS: [&str; 4] = [
    "https://www.miruro.to",
    "https://www.miruro.bz",
    "https://www.miruro.ru",
    "https://www.miruro.tv",
];

pub struct MiruroClient {
    client: Client,
}

impl MiruroClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:146.0) Gecko/20100101 Firefox/146.0")
            .build()
            .unwrap_or_else(|_| Client::new());
        Self { client }
    }

    fn xor_decrypt(data: &[u8]) -> Vec<u8> {
        let mut out = Vec::with_capacity(data.len());
        for (i, &b) in data.iter().enumerate() {
            out.push(b ^ PIPE_OBF_KEY[i % PIPE_OBF_KEY.len()]);
        }
        out
    }

    fn decode_body(body_str: &str, is_obfuscated: bool) -> Result<Value, AppError> {
        if !is_obfuscated {
            return serde_json::from_str(body_str)
                .map_err(|e| AppError::Parse(e.to_string()));
        }

        let trimmed = body_str.trim();
        let raw_bytes = URL_SAFE_NO_PAD
            .decode(trimmed)
            .or_else(|_| URL_SAFE.decode(trimmed))
            .or_else(|_| STANDARD.decode(trimmed))
            .map_err(|e| AppError::Network(format!("Miruro base64 decode failed: {}", e)))?;

        let xor_bytes = Self::xor_decrypt(&raw_bytes);

        let mut gz = GzDecoder::new(&xor_bytes[..]);
        let mut decompressed = String::new();
        gz.read_to_string(&mut decompressed)
            .map_err(|e| AppError::Network(format!("Miruro gzip decompression failed: {}", e)))?;

        serde_json::from_str(&decompressed).map_err(|e| AppError::Parse(e.to_string()))
    }

    async fn pipe_request(
        &self,
        path: &str,
        query: Value,
    ) -> Result<Value, AppError> {
        let envelope = json!({
            "path": path,
            "method": "GET",
            "query": query,
            "body": Value::Null,
            "version": "0.1.0"
        });

        let envelope_str = envelope.to_string();
        let e = URL_SAFE_NO_PAD.encode(envelope_str.as_bytes());

        let mut last_err = String::from("All Miruro mirrors failed");

        for mirror in MIRURO_MIRRORS {
            let url = format!("{}/api/secure/pipe?e={}", mirror, e);
            let res = match self
                .client
                .get(&url)
                .header("Referer", format!("{}/", mirror))
                .header("Origin", mirror)
                .header("Sec-Fetch-Dest", "empty")
                .header("Sec-Fetch-Mode", "cors")
                .header("Sec-Fetch-Site", "same-origin")
                .send()
                .await
            {
                Ok(r) if r.status().is_success() => r,
                Ok(r) => {
                    last_err = format!("Mirror {} returned status {}", mirror, r.status());
                    continue;
                }
                Err(err) => {
                    last_err = format!("Mirror {} error: {}", mirror, err);
                    continue;
                }
            };

            let obf_header = res
                .headers()
                .get("x-obfuscated")
                .and_then(|h| h.to_str().ok())
                .unwrap_or("");

            let is_obf = obf_header == "2" || obf_header == "1";

            let text = match res.text().await {
                Ok(t) => t,
                Err(_) => continue,
            };

            // Check if wrapper json with body/obf properties
            if let Ok(wrapper) = serde_json::from_str::<Value>(&text) {
                if let Some(body_val) = wrapper.get("body").and_then(|b| b.as_str()) {
                    let obf_flag = wrapper
                        .get("obf")
                        .and_then(|o| o.as_str())
                        .map(|s| s == "2" || s == "1")
                        .unwrap_or(is_obf);
                    if let Ok(parsed) = Self::decode_body(body_val, obf_flag) {
                        return Ok(parsed);
                    }
                }
            }

            if let Ok(parsed) = Self::decode_body(&text, is_obf) {
                return Ok(parsed);
            }
        }

        Err(AppError::Network(last_err))
    }

    pub async fn get_sources(
        &self,
        anilist_id: i64,
        episode_number: f64,
        provider: &str,
        category: &Category,
    ) -> Result<SourcesResult, AppError> {
        let ep_str = if episode_number.fract() == 0.0 {
            format!("{}", episode_number as i64)
        } else {
            format!("{}", episode_number)
        };

        let cat_str = category.as_str();

        let query = json!({
            "anilistId": anilist_id,
            "provider": provider,
            "category": cat_str,
            "episodeId": ep_str,
        });

        let data = self.pipe_request("sources", query).await?;

        let mut streams = Vec::new();
        if let Some(streams_arr) = data.get("streams").and_then(|s| s.as_array()) {
            for item in streams_arr {
                let url = item.get("url").and_then(|u| u.as_str()).unwrap_or("");
                if url.is_empty() {
                    continue;
                }

                let stream_type = item
                    .get("type")
                    .and_then(|t| t.as_str())
                    .unwrap_or("hls")
                    .to_string();

                let quality = item
                    .get("quality")
                    .and_then(|q| q.as_str())
                    .map(String::from)
                    .or_else(|| Some("Auto".to_string()));

                let referer = item
                    .get("referer")
                    .and_then(|r| r.as_str())
                    .map(String::from)
                    .or_else(|| Some("https://www.miruro.to/".to_string()));

                let is_active = item
                    .get("isActive")
                    .and_then(|a| a.as_bool())
                    .unwrap_or(true);

                let mut headers = HashMap::new();
                if let Some(ref r) = referer {
                    headers.insert("Referer".to_string(), r.clone());
                }

                streams.push(StreamItem {
                    url: url.to_string(),
                    stream_type,
                    quality,
                    audio: Some(cat_str.to_string()),
                    subtitle_variant: None,
                    referer: referer.clone(),
                    origin: Some("https://www.miruro.to".to_string()),
                    headers: Some(headers),
                    is_active,
                });
            }
        }

        let mut subtitles = Vec::new();
        if let Some(subs_arr) = data.get("subtitles").and_then(|s| s.as_array()) {
            for item in subs_arr {
                if let Some(url) = item.get("url").and_then(|u| u.as_str()) {
                    let language = item
                        .get("language")
                        .or_else(|| item.get("label"))
                        .and_then(|l| l.as_str())
                        .unwrap_or("English")
                        .to_string();

                    subtitles.push(SubtitleItem {
                        url: url.to_string(),
                        language,
                        format: "vtt".to_string(),
                        is_default: Some(false),
                    });
                }
            }
        }

        let skip = data.get("skipTimes").and_then(|s| {
            let intro_start = s.get("intro").and_then(|i| i.get("start")).and_then(|v| v.as_f64());
            let intro_end = s.get("intro").and_then(|i| i.get("end")).and_then(|v| v.as_f64());
            let outro_start = s.get("outro").and_then(|i| i.get("start")).and_then(|v| v.as_f64());
            let outro_end = s.get("outro").and_then(|i| i.get("end")).and_then(|v| v.as_f64());

            Some(SkipTimes {
                intro_start,
                intro_end,
                outro_start,
                outro_end,
            })
        });

        if streams.is_empty() {
            return Err(AppError::NoSourcesFound);
        }

        let variant = if subtitles.is_empty() {
            "h-sub".to_string()
        } else {
            "s-sub".to_string()
        };

        for s in &mut streams {
            s.subtitle_variant = Some(variant.clone());
        }

        Ok(SourcesResult {
            streams,
            subtitles,
            skip,
        })
    }
}
