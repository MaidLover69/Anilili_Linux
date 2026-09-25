use crate::db;
use crate::models::{CustomPlaylist, CustomPlaylistItem, ExternalPlaylistItem};
use crate::state::AppState;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use tauri::State;

#[tauri::command]
pub async fn create_playlist(
    name: String,
    description: Option<String>,
    state: State<'_, AppState>,
) -> Result<CustomPlaylist, String> {
    db::create_custom_playlist(&state.pool, &name, description.as_deref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_playlists(
    state: State<'_, AppState>,
) -> Result<Vec<CustomPlaylist>, String> {
    db::list_custom_playlists(&state.pool)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_playlist_items(
    playlist_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<CustomPlaylistItem>, String> {
    db::get_custom_playlist_items(&state.pool, &playlist_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_to_playlist(
    item: CustomPlaylistItem,
    state: State<'_, AppState>,
) -> Result<(), String> {
    db::add_item_to_custom_playlist(&state.pool, item)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_multiple_to_playlist(
    items: Vec<CustomPlaylistItem>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    for item in items {
        let _ = db::add_item_to_custom_playlist(&state.pool, item).await;
    }
    Ok(())
}

#[tauri::command]
pub async fn remove_from_playlist(
    item_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    db::remove_item_from_custom_playlist(&state.pool, &item_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_playlist(
    playlist_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    db::delete_custom_playlist(&state.pool, &playlist_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn launch_playlist_external_player(
    player: String,
    playlist_name: String,
    items: Vec<ExternalPlaylistItem>,
) -> Result<bool, String> {
    if items.is_empty() {
        return Err("Playlist has no items to play".to_string());
    }

    let sanitized_name: String = playlist_name
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect();

    let temp_dir = std::env::var("XDG_RUNTIME_DIR")
        .unwrap_or_else(|_| "/tmp".to_string());
    let m3u_path = format!("{}/anilili_playlist_{}.m3u8", temp_dir, sanitized_name);

    let mut file = File::create(&m3u_path)
        .map_err(|e| format!("Failed to create playlist file: {}", e))?;

    writeln!(file, "#EXTM3U").map_err(|e| e.to_string())?;

    for item in &items {
        writeln!(file, "#EXTINF:-1,{}", item.title).map_err(|e| e.to_string())?;
        if let Some(ref r) = item.referer {
            writeln!(file, "#EXTVLCOPT:http-referrer={}", r).map_err(|e| e.to_string())?;
            writeln!(file, "#EXT-X-PLAYLIST-TYPE:VOD").map_err(|e| e.to_string())?;
        }
        writeln!(file, "{}", item.url).map_err(|e| e.to_string())?;
    }

    let player_bin = match player.to_lowercase().as_str() {
        "vlc" => "vlc",
        _ => "mpv",
    };

    let mut cmd = Command::new(player_bin);
    if player_bin == "mpv" {
        cmd.arg(format!("--playlist={}", m3u_path));
        cmd.arg(format!("--title={}", playlist_name));
    } else {
        cmd.arg(&m3u_path);
    }

    match cmd.spawn() {
        Ok(_) => Ok(true),
        Err(e) => Err(format!("Failed to spawn {}: {}", player_bin, e)),
    }
}
