use crate::clients::http::build_http_client;
use crate::error::AppError;
use crate::models::{Category, EpisodeItem, ProviderData, SourcesResult, StreamItem};
use crate::providers::AnimeProvider;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;

pub struct GojoWtfProvider {
    client: Client,
    api_base: &'static str,
}

impl GojoWtfProvider {
    pub fn new() -> Self {
        Self {
            client: build_http_client(),
            api_base: "https://api.gojo.wtf/v2/api/anime",
        }
    }
}

#[async_trait]
impl AnimeProvider for GojoWtfProvider {
    fn name(&self) -> &'static str {
        "GojoWtf"
    }

    fn supports_dub(&self) -> bool {
        true
    }

    async fn get_episodes(
        &self,
        _anilist_id: i64,
        _mal_id: Option<i64>,
        title_romaji: Option<&str>,
    ) -> Result<ProviderData, AppError> {
        let title = match title_romaji {
            Some(t) if !t.is_empty() => t,
            _ => return Err(AppError::Provider {
                provider: self.name(),
                message: "No anime title provided".to_string(),
            }),
        };

        let search_url = format!("{}/search?query={}&page=1", self.api_base, urlencoding::encode(title));
        let res = self
            .client
            .get(&search_url)
            .header("Referer", "https://gojo.wtf/")
            .send()
            .await
            .map_err(|e| AppError::Network(e.to_string()))?;

        if !res.status().is_success() {
            return Ok(ProviderData {
                name: self.name().to_string(),
                sub: vec![],
                dub: vec![],
            });
        }

        let json: Value = res
            .json()
            .await
            .map_err(|e| AppError::Parse(e.to_string()))?;

        let first_anime_id = json
            .get("data")
            .and_then(|d| d.as_array())
            .and_then(|arr| arr.first())
            .and_then(|item| item.get("id"))
            .and_then(|id| id.as_str().map(String::from).or_else(|| id.as_i64().map(|v| v.to_string())));

        let anime_id = match first_anime_id {
            Some(id) => id,
            None => {
                return Ok(ProviderData {
                    name: self.name().to_string(),
                    sub: vec![],
                    dub: vec![],
                })
            }
        };

        let ep_url = format!("{}/episodes/{}", self.api_base, anime_id);
        let ep_res = self
            .client
            .get(&ep_url)
            .header("Referer", "https://gojo.wtf/")
            .send()
            .await
            .map_err(|e| AppError::Network(e.to_string()))?;

        let ep_json: Value = ep_res
            .json()
            .await
            .map_err(|e| AppError::Parse(e.to_string()))?;

        let mut sub_eps = Vec::new();
        let mut dub_eps = Vec::new();

        if let Some(ep_arr) = ep_json.get("episodes").and_then(|e| e.as_array()) {
            for ep in ep_arr {
                let num = ep.get("number").and_then(|n| n.as_f64()).unwrap_or(0.0);
                let ep_id = ep.get("id").and_then(|i| i.as_str()).unwrap_or("");
                let is_dub = ep.get("isDub").and_then(|d| d.as_bool()).unwrap_or(false);

                if num > 0.0 && !ep_id.is_empty() {
                    let item = EpisodeItem {
                        number: num,
                        pipe_id: format!("{}|{}", ep_id, if is_dub { "dub" } else { "sub" }),
                        title: ep.get("title").and_then(|t| t.as_str()).map(String::from),
                        image: ep.get("image").and_then(|i| i.as_str()).map(String::from),
                        synopsis: None,
                        filler: false,
                    };
                    if is_dub {
                        dub_eps.push(item);
                    } else {
                        sub_eps.push(item);
                    }
                }
            }
        }

        sub_eps.sort_by(|a, b| a.number.partial_cmp(&b.number).unwrap_or(std::cmp::Ordering::Equal));
        dub_eps.sort_by(|a, b| a.number.partial_cmp(&b.number).unwrap_or(std::cmp::Ordering::Equal));

        Ok(ProviderData {
            name: self.name().to_string(),
            sub: sub_eps,
            dub: dub_eps,
        })
    }

    async fn get_sources(
        &self,
        episode: &EpisodeItem,
        category: Category,
    ) -> Result<SourcesResult, AppError> {
        let parts: Vec<&str> = episode.pipe_id.split('|').collect();
        let ep_id = if !parts.is_empty() { parts[0] } else { &episode.pipe_id };

        let source_url = format!("{}/source/{}?server=pahe", self.api_base, ep_id);
        let res = self
            .client
            .get(&source_url)
            .header("Referer", "https://gojo.wtf/")
            .send()
            .await
            .map_err(|e| AppError::Network(e.to_string()))?;

        let json: Value = res
            .json()
            .await
            .map_err(|e| AppError::Parse(e.to_string()))?;

        let mut streams = Vec::new();
        if let Some(sources) = json.get("sources").and_then(|s| s.as_array()) {
            for src in sources {
                if let Some(url) = src.get("url").and_then(|u| u.as_str()) {
                    let quality = src.get("quality").and_then(|q| q.as_str()).unwrap_or("Auto").to_string();
                    let is_hls = src.get("isM3U8").and_then(|m| m.as_bool()).unwrap_or(url.contains(".m3u8"));

                    let mut headers = HashMap::new();
                    headers.insert("Referer".to_string(), "https://gojo.wtf/".to_string());

                    streams.push(StreamItem {
                    subtitle_variant: None,
                        url: url.to_string(),
                        stream_type: if is_hls { "hls".to_string() } else { "mp4".to_string() },
                        quality: Some(quality),
                        audio: Some(category.as_str().to_string()),
                        referer: Some("https://gojo.wtf/".to_string()),
                        origin: Some("https://gojo.wtf".to_string()),
                        headers: Some(headers),
                        is_active: true,
                    });
                }
            }
        }

        if streams.is_empty() {
            return Err(AppError::NoSourcesFound);
        }

        Ok(SourcesResult {
            streams,
            subtitles: vec![],
            skip: None,
        })
    }
}
