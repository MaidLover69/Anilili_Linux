use crate::db;
use crate::models::{Media, MediaListEntry, Viewer};
use crate::state::AppState;
use serde_json::json;
use tauri::State;

#[tauri::command]
pub async fn fetch_trending(state: State<'_, AppState>) -> Result<Vec<Media>, String> {
    state
        .anilist_client
        .fetch_media_list("TRENDING_DESC", 20)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn fetch_popular(state: State<'_, AppState>) -> Result<Vec<Media>, String> {
    state
        .anilist_client
        .fetch_media_list("POPULARITY_DESC", 20)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn fetch_top_rated(state: State<'_, AppState>) -> Result<Vec<Media>, String> {
    state
        .anilist_client
        .fetch_media_list("SCORE_DESC", 20)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn fetch_home_data(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let trending_fut = state.anilist_client.fetch_media_list("TRENDING_DESC", 10);
    let popular_fut = state.anilist_client.fetch_media_list("POPULARITY_DESC", 10);
    let top_rated_fut = state.anilist_client.fetch_media_list("SCORE_DESC", 10);
    let newest_fut = state.anilist_client.fetch_media_list("START_DATE_DESC", 10);
    let history_fut = db::get_continue_watching(&state.pool);

    let (trending, popular, top_rated, newest, history) = tokio::join!(
        trending_fut,
        popular_fut,
        top_rated_fut,
        newest_fut,
        history_fut
    );

    Ok(json!({
        "trending": trending.unwrap_or_default(),
        "popular": popular.unwrap_or_default(),
        "top_rated": top_rated.unwrap_or_default(),
        "newest": newest.unwrap_or_default(),
        "continue_watching": history.unwrap_or_default(),
    }))
}

#[tauri::command]
pub async fn search_anime(
    state: State<'_, AppState>,
    query: Option<String>,
    genres: Option<Vec<String>>,
    format: Option<String>,
    status: Option<String>,
    sort: Option<String>,
    page: Option<i32>,
    per_page: Option<i32>,
) -> Result<Vec<Media>, String> {
    state
        .anilist_client
        .search(
            query,
            genres.unwrap_or_default(),
            format,
            status,
            sort,
            page.unwrap_or(1),
            per_page.unwrap_or(20),
        )
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn fetch_anime_details(
    state: State<'_, AppState>,
    id: i64,
) -> Result<serde_json::Value, String> {
    state
        .anilist_client
        .fetch_details(id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_anilist_viewer(
    state: State<'_, AppState>,
    token: String,
) -> Result<Viewer, String> {
    state
        .anilist_client
        .get_viewer(&token)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn fetch_anilist_library(
    state: State<'_, AppState>,
    token: String,
    user_id: i32,
    status: Option<String>,
) -> Result<Vec<MediaListEntry>, String> {
    state
        .anilist_client
        .get_user_library(&token, user_id, status)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_anilist_entry(
    state: State<'_, AppState>,
    token: String,
    media_id: i32,
    status: Option<String>,
    progress: Option<i32>,
    score: Option<f64>,
) -> Result<MediaListEntry, String> {
    state
        .anilist_client
        .save_list_entry(&token, media_id, status, progress, score)
        .await
        .map_err(|e| e.to_string())
}
