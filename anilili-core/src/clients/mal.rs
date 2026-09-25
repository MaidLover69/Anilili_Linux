use crate::error::AppError;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub const MAL_API_URL: &str = "https://api.myanimelist.net/v2";
pub const MAL_TOKEN_URL: &str = "https://myanimelist.net/v1/oauth2/token";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MalTokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

pub struct MalClient {
    client: Client,
}

impl MalClient {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn refresh_token(
        &self,
        client_id: &str,
        refresh_token: &str,
    ) -> Result<MalTokenResponse, AppError> {
        let params = [
            ("client_id", client_id),
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
        ];

        let res = self
            .client
            .post(MAL_TOKEN_URL)
            .form(&params)
            .send()
            .await?;

        if !res.status().is_success() {
            return Err(AppError::AuthRequired);
        }

        let tokens: MalTokenResponse = res.json().await?;
        Ok(tokens)
    }

    pub async fn update_anime_status(
        &self,
        access_token: &str,
        mal_id: i32,
        episode: i32,
        status: Option<String>,
    ) -> Result<(), AppError> {
        let url = format!("{}/anime/{}/my_list_status", MAL_API_URL, mal_id);
        
        let mut form_params = vec![("num_watched_episodes", episode.to_string())];
        
        // Status mapping
        let mal_status = match status.as_deref() {
            Some("CURRENT") | Some("watching") => Some("watching"),
            Some("COMPLETED") | Some("completed") => Some("completed"),
            Some("PLANNING") | Some("plan_to_watch") => Some("plan_to_watch"),
            Some("PAUSED") | Some("on_hold") => Some("on_hold"),
            Some("DROPPED") | Some("dropped") => Some("dropped"),
            Some("REPEATING") | Some("repeating") => Some("watching"),
            _ => None,
        };

        if let Some(st) = mal_status {
            form_params.push(("status", st.to_string()));
        }

        let res = self
            .client
            .patch(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .form(&form_params)
            .send()
            .await?;

        if res.status().as_u16() == 401 {
            return Err(AppError::AuthRequired);
        }

        if !res.status().is_success() {
            let status_code = res.status();
            let text = res.text().await.unwrap_or_default();
            return Err(AppError::Other(format!(
                "MAL API status update failed ({}): {}",
                status_code, text
            )));
        }

        Ok(())
    }
}
