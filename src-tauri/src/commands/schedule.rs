use crate::db;
use crate::models::{AiringEntry, NotificationPreference};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn fetch_airing_schedule(
    state: State<'_, AppState>,
    from_ts: i64,
    to_ts: i64,
) -> Result<Vec<AiringEntry>, String> {
    if let Ok(Some(cached)) = db::get_cached_schedule(&state.pool, from_ts, to_ts).await {
        return Ok(cached);
    }

    let entries = state
        .anilist_client
        .fetch_airing_schedule(from_ts, to_ts)
        .await
        .map_err(|e| e.to_string())?;

    let _ = db::cache_schedule(&state.pool, from_ts, to_ts, &entries).await;
    Ok(entries)
}

#[tauri::command]
pub async fn save_notification_preference(
    state: State<'_, AppState>,
    pref: NotificationPreference,
) -> Result<(), String> {
    db::save_notification_preference(&state.pool, &pref)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_notification_preference(
    state: State<'_, AppState>,
    media_id: i64,
) -> Result<Option<NotificationPreference>, String> {
    db::get_notification_preference(&state.pool, media_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_notification_preferences(
    state: State<'_, AppState>,
) -> Result<Vec<NotificationPreference>, String> {
    db::list_notification_preferences(&state.pool)
        .await
        .map_err(|e| e.to_string())
}
