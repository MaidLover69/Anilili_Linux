use crate::db;
use crate::models::{HistoryEntry, MediaListEntry, WatchlistEntry};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn get_watched_episodes(
    state: State<'_, AppState>,
    anilist_id: i32,
) -> Result<Vec<f64>, String> {
    db::get_watched_episodes(&state.pool, anilist_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_episode_progress(
    state: State<'_, AppState>,
    anilist_id: i32,
    episode_number: f64,
) -> Result<Option<(i64, i64)>, String> {
    db::get_episode_progress(&state.pool, anilist_id, episode_number)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_continue_watching(
    state: State<'_, AppState>,
) -> Result<Vec<HistoryEntry>, String> {
    db::get_continue_watching(&state.pool)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_watch_progress(
    state: State<'_, AppState>,
    anilist_id: i32,
    title: String,
    cover: Option<String>,
    episode_number: f64,
    episode_title: Option<String>,
    provider: String,
    category: String,
    position_ms: i64,
    duration_ms: i64,
) -> Result<(), String> {
    let now = db::current_unix_timestamp() * 1000;
    let entry = HistoryEntry {
        anilist_id,
        title,
        cover,
        episode_number,
        episode_title,
        provider,
        category,
        position_ms,
        duration_ms,
        updated_at: now,
        from_remote: false,
    };

    db::update_watch_progress(&state.pool, &entry)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_to_watchlist(
    state: State<'_, AppState>,
    anilist_id: i32,
    title: String,
    cover: Option<String>,
    format: Option<String>,
    average_score: Option<i32>,
) -> Result<(), String> {
    db::add_to_watchlist(
        &state.pool,
        anilist_id,
        &title,
        cover.as_deref(),
        format.as_deref(),
        average_score,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_from_watchlist(
    state: State<'_, AppState>,
    anilist_id: i32,
) -> Result<(), String> {
    db::remove_from_watchlist(&state.pool, anilist_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn is_in_watchlist(
    state: State<'_, AppState>,
    anilist_id: i32,
) -> Result<bool, String> {
    db::is_in_watchlist(&state.pool, anilist_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_watchlist(
    state: State<'_, AppState>,
) -> Result<Vec<WatchlistEntry>, String> {
    db::get_watchlist(&state.pool)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn upsert_library_entry(
    state: State<'_, AppState>,
    entry: MediaListEntry,
) -> Result<(), String> {
    db::upsert_library_entry(&state.pool, &entry)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_library_entries(
    state: State<'_, AppState>,
    status: Option<String>,
) -> Result<Vec<MediaListEntry>, String> {
    db::get_library_entries(&state.pool, status)
        .await
        .map_err(|e| e.to_string())
}
