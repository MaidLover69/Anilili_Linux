use crate::clients::http::build_http_client;
use crate::error::AppError;
use crate::models::EpisodeItem;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KonohaEpisode {
    pub number: f64,
    pub title: Option<String>,
    pub image: Option<String>,
    pub filler: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KonohaResponse {
    pub episodes: Vec<KonohaEpisode>,
}

pub async fn fetch_konoha_episodes(anilist_id: i64) -> Result<Vec<EpisodeItem>, AppError> {
    let client = build_http_client();
    let cdn_url = format!(
        "https://cdn.jsdelivr.net/gh/Konoha-orgs/Konoha/anime/{}.json",
        anilist_id
    );
    let fallback_url = format!(
        "https://raw.githubusercontent.com/Konoha-orgs/Konoha/main/anime/{}.json",
        anilist_id
    );

    let res = match client.get(&cdn_url).send().await {
        Ok(r) if r.status().is_success() => Ok(r),
        _ => client.get(&fallback_url).send().await,
    };

    let response = match res {
        Ok(r) if r.status().is_success() => r,
        _ => return Ok(Vec::new()), // Return empty if not present in Konoha
    };

    let data: KonohaResponse = match response.json().await {
        Ok(d) => d,
        Err(_) => return Ok(Vec::new()),
    };

    let items = data
        .episodes
        .into_iter()
        .map(|ep| EpisodeItem {
            pipe_id: format!("{}-{}", anilist_id, ep.number),
            number: ep.number,
            title: ep.title,
            image: ep.image,
            synopsis: None,
            filler: ep.filler.unwrap_or(false),
        })
        .collect();

    Ok(items)
}
