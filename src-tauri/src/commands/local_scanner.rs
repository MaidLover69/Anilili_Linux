use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocalAnimeFile {
    pub file_path: String,
    pub file_name: String,
    pub file_size: u64,
    pub parsed_title: String,
    pub parsed_episode: Option<f64>,
}

fn parse_episode_and_title(file_name: &str) -> (String, Option<f64>) {
    // Remove extension
    let name = file_name.rsplit_once('.').map(|(n, _)| n).unwrap_or(file_name);

    // Common episode patterns: " - 01 ", " - 01.", "E01", "EP01", "Episode 01"
    let re_ep = Regex::new(r#"(?i)(?:[-_]\s*|ep(?:isode)?\.?\s*|e)(\d{1,4}(?:\.\d)?)"#).unwrap();

    let episode = re_ep.captures(name).and_then(|caps| {
        caps.get(1).and_then(|m| m.as_str().parse::<f64>().ok())
    });

    // Clean title by stripping release groups like [SubsPlease], [Erai-raws], etc.
    let re_clean = Regex::new(r#"\[[^\]]+\]|\([^\)]+\)"#).unwrap();
    let cleaned = re_clean.replace_all(name, "").trim().to_string();

    let title = if let Some((t, _)) = cleaned.rsplit_once('-') {
        t.trim().to_string()
    } else {
        cleaned
    };

    (title, episode)
}

fn scan_dir_recursive(path: &Path, depth: usize, max_depth: usize, results: &mut Vec<LocalAnimeFile>) {
    if depth > max_depth {
        return;
    }

    let entries = match fs::read_dir(path) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            scan_dir_recursive(&p, depth + 1, max_depth, results);
        } else if p.is_file() {
            if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                let ext_lower = ext.to_lowercase();
                if matches!(ext_lower.as_str(), "mkv" | "mp4" | "webm" | "avi") {
                    let file_name = p.file_name().and_then(|f| f.to_str()).unwrap_or("").to_string();
                    let metadata = fs::metadata(&p).ok();
                    let file_size = metadata.map(|m| m.len()).unwrap_or(0);
                    let (parsed_title, parsed_episode) = parse_episode_and_title(&file_name);

                    results.push(LocalAnimeFile {
                        file_path: p.to_string_lossy().to_string(),
                        file_name,
                        file_size,
                        parsed_title,
                        parsed_episode,
                    });
                }
            }
        }
    }
}

#[tauri::command]
pub async fn scan_local_anime_directory(dir_path: String) -> Result<Vec<LocalAnimeFile>, String> {
    let path = PathBuf::from(&dir_path);
    if !path.exists() || !path.is_dir() {
        return Err(format!("Directory does not exist: {}", dir_path));
    }

    let mut files = Vec::new();
    scan_dir_recursive(&path, 0, 4, &mut files);
    files.sort_by(|a, b| a.file_name.cmp(&b.file_name));
    Ok(files)
}

#[tauri::command]
pub async fn open_file_in_player(file_path: String, player: Option<String>) -> Result<(), String> {
    let p = PathBuf::from(&file_path);
    if !p.exists() {
        return Err(format!("File does not exist: {}", file_path));
    }

    let player_bin = player.unwrap_or_else(|| "mpv".to_string());
    let mut cmd = std::process::Command::new(&player_bin);
    cmd.arg(&file_path);

    match cmd.spawn() {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to launch {}: {}", player_bin, e)),
    }
}
