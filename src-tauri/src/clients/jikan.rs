use crate::error::AppError;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct MalAnimeDetails {
    pub mal_id: i32,
    pub title: String,
    pub title_english: Option<String>,
    pub title_japanese: Option<String>,
    pub synopsis: Option<String>,
    pub score: Option<f64>,
    pub episodes: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct MalEpisodeInfo {
    pub number: f64,
    pub title: Option<String>,
    pub title_japanese: Option<String>,
    pub title_romanji: Option<String>,
    pub filler: bool,
    pub synopsis: Option<String>,
}

pub async fn fetch_mal_details(
    client: &Client,
    mal_id: i32,
) -> Result<MalAnimeDetails, AppError> {
    let url = format!("https://api.jikan.moe/v4/anime/{}", mal_id);
    let res = client
        .get(&url)
        .header("User-Agent", "AnililiLinux/1.0")
        .send()
        .await?;

    if !res.status().is_success() {
        return Err(AppError::Other(format!("Jikan anime details error HTTP {}", res.status())));
    }

    let json: serde_json::Value = res.json().await?;
    let data = &json["data"];

    let title = data["title"].as_str().unwrap_or("").to_string();
    let title_english = data["title_english"].as_str().map(String::from);
    let title_japanese = data["title_japanese"].as_str().map(String::from);
    let synopsis = data["synopsis"].as_str().map(String::from);
    let score = data["score"].as_f64();
    let episodes = data["episodes"].as_i64().map(|v| v as i32);

    Ok(MalAnimeDetails {
        mal_id,
        title,
        title_english,
        title_japanese,
        synopsis,
        score,
        episodes,
    })
}

pub async fn fetch_mal_episodes(
    client: &Client,
    mal_id: i32,
) -> Result<Vec<MalEpisodeInfo>, AppError> {
    let mut ep_list = Vec::new();
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
            return Err(AppError::Other(format!("Jikan episodes error HTTP {}", res.status())));
        }

        let json: serde_json::Value = res.json().await?;
        let data = json["data"].as_array();

        if let Some(episodes) = data {
            for ep in episodes {
                let ep_num = ep["mal_id"].as_f64().unwrap_or(0.0);
                let title = ep["title"].as_str().map(String::from);
                let title_japanese = ep["title_japanese"].as_str().map(String::from);
                let title_romanji = ep["title_romanji"].as_str().map(String::from);
                let filler = ep["filler"].as_bool().unwrap_or(false);
                let synopsis = ep["synopsis"].as_str().map(String::from);

                ep_list.push(MalEpisodeInfo {
                    number: ep_num,
                    title,
                    title_japanese,
                    title_romanji,
                    filler,
                    synopsis,
                });
            }
        }

        let has_next_page = json["pagination"]["has_next_page"].as_bool().unwrap_or(false);
        if !has_next_page || page >= 20 {
            break;
        }

        page += 1;
        sleep(Duration::from_millis(350)).await;
    }

    Ok(ep_list)
}

pub async fn fetch_filler_episodes(
    client: &Client,
    mal_id: i32,
) -> Result<Vec<i32>, AppError> {
    let eps = fetch_mal_episodes(client, mal_id).await?;
    let fillers: Vec<i32> = eps
        .into_iter()
        .filter(|e| e.filler)
        .map(|e| e.number as i32)
        .collect();
    Ok(fillers)
}
