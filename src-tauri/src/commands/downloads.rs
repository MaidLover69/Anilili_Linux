use crate::db;
use crate::downloads::{self, StorageCheck};
use crate::models::DownloadRecord;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn create_download(
    state: State<'_, AppState>,
    record: DownloadRecord,
) -> Result<String, String> {
    db::create_download(&state.pool, &record)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_download_status(
    state: State<'_, AppState>,
    id: String,
    status: String,
    progress: Option<f64>,
    file_path: Option<String>,
    file_size: Option<i64>,
    duration_s: Option<f64>,
    error_msg: Option<String>,
) -> Result<(), String> {
    db::update_download_status(
        &state.pool,
        &id,
        &status,
        progress,
        file_path.as_deref(),
        file_size,
        duration_s,
        error_msg.as_deref(),
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_downloads(
    state: State<'_, AppState>,
    anilist_id: i64,
) -> Result<Vec<DownloadRecord>, String> {
    db::get_downloads(&state.pool, anilist_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_all_downloads(
    state: State<'_, AppState>,
) -> Result<Vec<DownloadRecord>, String> {
    db::get_all_downloads(&state.pool)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_download(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    db::delete_download(&state.pool, &id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_download_by_id(
    state: State<'_, AppState>,
    id: String,
) -> Result<Option<DownloadRecord>, String> {
    db::get_download_by_id(&state.pool, &id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn check_storage_for_download(quality: String) -> Result<StorageCheck, String> {
    downloads::check_storage(&quality).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_download_directory() -> Result<String, String> {
    let dir = downloads::get_download_dir().map_err(|e| e.to_string())?;
    Ok(dir.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn start_episode_download(
    state: State<'_, AppState>,
    record: DownloadRecord,
    stream_url: String,
    referer: Option<String>,
    origin: Option<String>,
    subtitles: Vec<crate::models::SubtitleItem>,
) -> Result<String, String> {
    let download_id = db::create_download(&state.pool, &record)
        .await
        .map_err(|e| e.to_string())?;

    let pool = state.pool.clone();
    tokio::spawn(async move {
        downloads::run_download_task(pool, record, stream_url, referer, origin, subtitles).await;
    });

    Ok(download_id)
}
