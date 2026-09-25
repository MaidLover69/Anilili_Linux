use crate::clients::mal::{MalTokenResponse, MAL_TOKEN_URL};
use crate::commands::mal::MAL_CLIENT_ID;
use crate::state::AppState;
use tauri::State;

pub const ANILIST_CLIENT_ID: &str = "45552";

#[tauri::command]
pub async fn get_auth_urls() -> Result<serde_json::Value, String> {
    let anilist_url = format!(
        "https://anilist.co/api/v2/oauth/authorize?client_id={}&response_type=token",
        ANILIST_CLIENT_ID
    );
    Ok(serde_json::json!({
        "anilist_client_id": ANILIST_CLIENT_ID,
        "anilist_auth_url": anilist_url,
        "mal_client_id": MAL_CLIENT_ID,
    }))
}

#[tauri::command]
pub async fn exchange_mal_code(
    state: State<'_, AppState>,
    code: String,
    code_verifier: String,
) -> Result<MalTokenResponse, String> {
    let params = [
        ("client_id", MAL_CLIENT_ID),
        ("grant_type", "authorization_code"),
        ("code", &code),
        ("code_verifier", &code_verifier),
    ];

    let res = state
        .client
        .post(MAL_TOKEN_URL)
        .form(&params)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        return Err(format!("MAL code exchange failed ({}): {}", status, body));
    }

    let tokens: MalTokenResponse = res.json().await.map_err(|e| e.to_string())?;
    Ok(tokens)
}
