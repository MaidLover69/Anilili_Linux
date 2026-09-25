use crate::error::AppError;
use crate::models::{Category, EpisodeItem, ProviderData, SourcesResult, StreamItem};
use crate::providers::AnimeProvider;
use async_trait::async_trait;
use reqwest::Client;
use std::time::Duration;

pub struct AnimeShqipProvider;

impl AnimeShqipProvider {
    pub fn new() -> Self {
        Self
    }

    fn build_client() -> Client {
        Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:146.0) Gecko/20100101 Firefox/146.0")
            .build()
            .unwrap_or_else(|_| Client::new())
    }
}

#[async_trait]
impl AnimeProvider for AnimeShqipProvider {
    fn name(&self) -> &'static str {
        "animeshqip"
    }

    fn supports_dub(&self) -> bool {
        false
    }

    async fn get_episodes(
        &self,
        anilist_id: i64,
        _mal_id: Option<i64>,
        title_romaji: Option<&str>,
    ) -> Result<ProviderData, AppError> {
        let client = Self::build_client();
        let query = title_romaji.unwrap_or("").trim();
        let query_str = if query.is_empty() {
            anilist_id.to_string()
        } else {
            query.to_string()
        };

        // Step 1: GET /search/{query}?format=json
        let search_url = format!("https://www.animeshqip.live/search/{}?format=json&limit=8", query_str.replace(' ', "+"));
        let search_res = client
            .get(&search_url)
            .header("Referer", "https://www.animeshqip.live/")
            .header("X-Requested-With", "XMLHttpRequest")
            .send()
            .await;

        let slug = match search_res {
            Ok(r) if r.status().is_success() => {
                if let Ok(json) = r.json::<serde_json::Value>().await {
                    let items = json.as_array().or_else(|| json["results"].as_array());
                    items.and_then(|arr| arr.first()).and_then(|item| item["slug"].as_str().map(String::from))
                } else {
                    None
                }
            }
            _ => None,
        };

        let slug = match slug {
            Some(s) => s,
            None => query_str.to_lowercase().replace(' ', "-"),
        };

        // Step 2: GET /serie/{slug} to parse episode list
        let series_url = format!("https://www.animeshqip.live/serie/{}", slug);
        let series_res = client
            .get(&series_url)
            .header("Referer", "https://www.animeshqip.live/")
            .send()
            .await;

        let html = match series_res {
            Ok(r) if r.status().is_success() => r.text().await.unwrap_or_default(),
            _ => return Ok(ProviderData {
                name: self.name().to_string(),
                sub: Vec::new(),
                dub: Vec::new(),
            }),
        };

        // Parse episode links: href="/episode/{slug}-episode-{n}" or href="/episode/([^"]+)" or data-id
        let ep_re = regex::Regex::new(r#"href="(/episode/[^"]+)"#).unwrap();
        let num_re = regex::Regex::new(r#"episode-(\d+(?:\.\d+)?)"#).unwrap();

        let mut sub_eps: Vec<EpisodeItem> = Vec::new();
        for cap in ep_re.captures_iter(&html) {
            let ep_path = cap[1].to_string();
            let ep_num = num_re
                .captures(&ep_path)
                .and_then(|c| c[1].parse::<f64>().ok())
                .unwrap_or(sub_eps.len() as f64 + 1.0);

            if !sub_eps.iter().any(|e| (e.number - ep_num).abs() < 0.01) {
                sub_eps.push(EpisodeItem {
                    pipe_id: format!("animeshqip|{}", ep_path),
                    number: ep_num,
                    title: Some(format!("Episode {}", ep_num as i32)),
                    image: None,
                    filler: false,
                });
            }
        }

        sub_eps.sort_by(|a, b| a.number.partial_cmp(&b.number).unwrap_or(std::cmp::Ordering::Equal));

        Ok(ProviderData {
            name: self.name().to_string(),
            sub: sub_eps.clone(),
            dub: Vec::new(),
        })
    }

    async fn get_sources(
        &self,
        episode: &EpisodeItem,
        _category: Category,
    ) -> Result<SourcesResult, AppError> {
        let parts: Vec<&str> = episode.pipe_id.splitn(2, '|').collect();
        if parts.len() < 2 || parts[0] != "animeshqip" {
            return Err(AppError::NoSourcesFound);
        }
        let ep_path = parts[1];

        let client = Self::build_client();
        let ep_url = format!("https://www.animeshqip.live{}", ep_path);

        let res = client
            .get(&ep_url)
            .header("Referer", "https://www.animeshqip.live/")
            .send()
            .await;

        let html = match res {
            Ok(r) if r.status().is_success() => r.text().await.unwrap_or_default(),
            _ => return Err(AppError::NoSourcesFound),
        };

        // Check for adblock verification token & submit status
        let token_re = regex::Regex::new(r#"token=([a-zA-Z0-9_-]+)"#).unwrap();
        if let Some(cap) = token_re.captures(&html) {
            let token = &cap[1];
            let _ = client
                .get(&format!("https://www.animeshqip.live/adsbygoogle.js?token={}", token))
                .header("Referer", &ep_url)
                .send()
                .await;

            let _ = client
                .post("https://www.animeshqip.live/ajax/adblock/status")
                .header("Referer", &ep_url)
                .form(&[("status", "clear")])
                .send()
                .await;
        }

        // Try POST /ajax/embed or extract iframe source
        let embed_post_res = client
            .post("https://www.animeshqip.live/ajax/embed")
            .header("Referer", &ep_url)
            .header("X-Requested-With", "XMLHttpRequest")
            .send()
            .await;

        if let Ok(er) = embed_post_res {
            if let Ok(json) = er.json::<serde_json::Value>().await {
                let stream_url = json["embed"]
                    .as_str()
                    .or_else(|| json["url"].as_str())
                    .or_else(|| json["src"].as_str())
                    .map(String::from);

                if let Some(url) = stream_url {
                    return Ok(SourcesResult {
                        streams: vec![StreamItem {
                            url: url.replace("\\/", "/"),
                            stream_type: "hls".to_string(),
                            quality: Some("auto".to_string()),
                            audio: None,
                            referer: Some("https://www.animeshqip.live/".to_string()),
                            is_active: true,
                        }],
                    });
                }
            }
        }

        // Regex fallback for iframe src or m3u8 in html
        let m3u8_re = regex::Regex::new(r#"https?://[^\s"'\\<>]+?\.(?:m3u8|mp4)[^\s"'\\<>]*"#).unwrap();
        if let Some(mat) = m3u8_re.find(&html) {
            return Ok(SourcesResult {
                streams: vec![StreamItem {
                    url: mat.as_str().to_string(),
                    stream_type: if mat.as_str().contains(".mp4") { "mp4".to_string() } else { "hls".to_string() },
                    quality: Some("auto".to_string()),
                    audio: None,
                    referer: Some("https://www.animeshqip.live/".to_string()),
                    is_active: true,
                }],
            });
        }

        Err(AppError::NoSourcesFound)
    }
}
