use crate::error::AppError;
use reqwest::Client;
use std::time::Duration;
use tokio::time::sleep;

pub async fn fetch_filler_episodes(
    client: &Client,
    mal_id: i32,
) -> Result<Vec<i32>, AppError> {
    let mut filler_episodes = Vec::new();
    let mut page = 1;

    loop {
        let url = format!("https://api.jikan.moe/v4/anime/{}/episodes?page={}", mal_id, page);
        let res = client
            .get(&url)
            .header("User-Agent", "AnililiLinux/1.0")
            .send()
            .await?;

        if !res.status().is_success() {
            if res.status().as_u16() == 404 {
                break;
            }
            return Err(AppError::Other(format!("Jikan API error HTTP {}", res.status())));

        }

        let json: serde_json::Value = res.json().await?;
        let data = json["data"].as_array();

        if let Some(episodes) = data {
            for ep in episodes {
                let is_filler = ep["filler"].as_bool().unwrap_or(false);
                if is_filler {
                    if let Some(ep_num) = ep["mal_id"].as_i64() {
                        filler_episodes.push(ep_num as i32);
                    }
                }
            }
        }

        let has_next_page = json["pagination"]["has_next_page"].as_bool().unwrap_or(false);
        if !has_next_page || page >= 20 {
            break;
        }

        page += 1;
        sleep(Duration::from_millis(400)).await;
    }

    Ok(filler_episodes)
}
