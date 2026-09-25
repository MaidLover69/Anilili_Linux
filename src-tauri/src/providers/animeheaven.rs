use crate::clients::http::build_http_client;
use crate::error::AppError;
use crate::models::{Category, EpisodeItem, ProviderData, SourcesResult, StreamItem};
use crate::providers::AnimeProvider;
use async_trait::async_trait;
use regex::Regex;
use reqwest::Client;
use std::collections::HashMap;

pub struct AnimeHeavenProvider {
    client: Client,
    base: &'static str,
}

impl AnimeHeavenProvider {
    pub fn new() -> Self {
        Self {
            client: build_http_client(),
            base: "https://animeheaven.me",
        }
    }
}

#[async_trait]
impl AnimeProvider for AnimeHeavenProvider {
    fn name(&self) -> &'static str {
        "AnimeHeaven"
    }

    fn supports_dub(&self) -> bool {
        false
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

        let search_url = format!("{}/search.php?s={}", self.base, urlencoding::encode(title));
        let search_res = self
            .client
            .get(&search_url)
            .send()
            .await
            .map_err(|e| AppError::Network(e.to_string()))?;

        let html = search_res
            .text()
            .await
            .map_err(|e| AppError::Parse(e.to_string()))?;

        // Extract first anime link
        let re_link = Regex::new(r#"<a href='(anime\.php\?[^']+)'>"#).unwrap();
        let anime_path = match re_link.captures(&html) {
            Some(caps) => caps[1].to_string(),
            None => {
                return Ok(ProviderData {
                    name: self.name().to_string(),
                    sub: vec![],
                    dub: vec![],
                })
            }
        };

        let anime_url = format!("{}/{}", self.base, anime_path);
        let ep_res = self
            .client
            .get(&anime_url)
            .send()
            .await
            .map_err(|e| AppError::Network(e.to_string()))?;

        let ep_html = ep_res
            .text()
            .await
            .map_err(|e| AppError::Parse(e.to_string()))?;

        // Extract episodes with gatea("hash") and number
        let re_ep = Regex::new(r#"onclick='gatea\("([a-f0-9]+)"\)'[^>]*>(?:[\s\S]*?)<div[^>]*\bwatch2\b[^>]*>\s*(\d+)\s*</div>"#).unwrap();
        let mut sub_eps = Vec::new();

        for cap in re_ep.captures_iter(&ep_html) {
            let gate_key = &cap[1];
            let ep_num: f64 = cap[2].parse().unwrap_or(0.0);
            if ep_num > 0.0 {
                sub_eps.push(EpisodeItem {
                    number: ep_num,
                    pipe_id: format!("{}|{}", gate_key, ep_num),
                    title: Some(format!("Episode {}", ep_num)),
                    image: None,
                    synopsis: None,
                    filler: false,
                });
            }
        }

        sub_eps.sort_by(|a, b| a.number.partial_cmp(&b.number).unwrap_or(std::cmp::Ordering::Equal));

        Ok(ProviderData {
            name: self.name().to_string(),
            sub: sub_eps,
            dub: vec![],
        })
    }

    async fn get_sources(
        &self,
        episode: &EpisodeItem,
        _category: Category,
    ) -> Result<SourcesResult, AppError> {
        let parts: Vec<&str> = episode.pipe_id.split('|').collect();
        let gate_key = if !parts.is_empty() { parts[0] } else { &episode.pipe_id };

        let gate_url = format!("{}/gate.php", self.base);
        let anime_referer = format!("{}/anime.php", self.base);

        let res = self
            .client
            .get(&gate_url)
            .header("Cookie", format!("key={}", gate_key))
            .header("Referer", &anime_referer)
            .send()
            .await
            .map_err(|e| AppError::Network(e.to_string()))?;

        let text = res
            .text()
            .await
            .map_err(|e| AppError::Parse(e.to_string()))?;

        // Extract stream URL
        let re_src = Regex::new(r#"(https?://[^\s"'<>]+\.(?:m3u8|mp4)[^\s"'<>]*)"#).unwrap();
        let stream_url = match re_src.captures(&text) {
            Some(caps) => caps[1].to_string(),
            None => return Err(AppError::NoSourcesFound),
        };

        let is_hls = stream_url.contains(".m3u8");
        let mut headers = HashMap::new();
        headers.insert("Referer".to_string(), anime_referer.clone());

        Ok(SourcesResult {
            streams: vec![StreamItem {
                    subtitle_variant: None,
                url: stream_url,
                stream_type: if is_hls { "hls".to_string() } else { "mp4".to_string() },
                quality: Some("1080p".to_string()),
                audio: Some("sub".to_string()),
                referer: Some(anime_referer),
                origin: Some(self.base.to_string()),
                headers: Some(headers),
                is_active: true,
            }],
            subtitles: vec![],
            skip: None,
        })
    }
}
