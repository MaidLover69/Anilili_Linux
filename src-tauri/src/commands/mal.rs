use crate::clients::mal::MalTokenResponse;
use crate::state::AppState;
use tauri::State;

pub const MAL_CLIENT_ID: &str = "4ae5f8056db821737a4f40f8816177cd";

#[tauri::command]
pub async fn mal_refresh_token(
    state: State<'_, AppState>,
    refresh_token: String,
) -> Result<MalTokenResponse, String> {
    state
        .mal_client
        .refresh_token(MAL_CLIENT_ID, &refresh_token)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn mal_update_status(
    state: State<'_, AppState>,
    access_token: String,
    mal_id: i32,
    episode: i32,
    status: Option<String>,
) -> Result<(), String> {
    state
        .mal_client
        .update_anime_status(&access_token, mal_id, episode, status)
        .await
        .map_err(|e| e.to_string())
}
