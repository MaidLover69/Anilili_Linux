use crate::error::AppError;
use crate::models::{Category, EpisodeItem, ProviderData, SourcesResult, StreamItem};
use crate::providers::AnimeProvider;
use async_trait::async_trait;
use reqwest::Client;
use std::time::Duration;

pub struct AniZoneProvider;

impl AniZoneProvider {
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
impl AnimeProvider for AniZoneProvider {
    fn name(&self) -> &'static str {
        "anizone"
    }

    async fn get_episodes(
        &self,
        anilist_id: i64,
        _mal_id: Option<i64>,
        _title_romaji: Option<&str>,
    ) -> Result<ProviderData, AppError> {
        let client = Self::build_client();

        // AniZone links anime pages directly by anilist ID
        let url = format!("https://anizone.to/anime/{}", anilist_id);
        let res = client
            .get(&url)
            .header("Referer", "https://anizone.to/")
            .send()
            .await;

        let html = match res {
            Ok(r) if r.status().is_success() => r.text().await.unwrap_or_default(),
            _ => {
                return Ok(ProviderData {
                    name: self.name().to_string(),
                    sub: Vec::new(),
                    dub: Vec::new(),
                })
            }
        };

        // Parse episode items: data-ep and data-id from .ep-item elements
        let ep_re =
            match regex::Regex::new(r#"data-ep="([^"]+)"[^>]*data-id="([^"]+)""#) {
                Ok(r) => r,
                Err(_) => {
                    return Ok(ProviderData {
                        name: self.name().to_string(),
                        sub: Vec::new(),
                        dub: Vec::new(),
                    })
                }
            };

        // Also try the reversed attribute order
        let ep_re_rev =
            match regex::Regex::new(r#"data-id="([^"]+)"[^>]*data-ep="([^"]+)""#) {
                Ok(r) => r,
                Err(_) => {
                    return Ok(ProviderData {
                        name: self.name().to_string(),
                        sub: Vec::new(),
                        dub: Vec::new(),
                    })
                }
            };

        let mut sub_eps: Vec<EpisodeItem> = Vec::new();

        // data-ep first, data-id second
        for cap in ep_re.captures_iter(&html) {
            let ep_num = cap[1].parse::<f64>().unwrap_or(sub_eps.len() as f64 + 1.0);
            let ep_id = cap[2].to_string();
            if !sub_eps.iter().any(|e| (e.number - ep_num).abs() < 0.01) {
                sub_eps.push(EpisodeItem {
                    pipe_id: format!("anizone|{}", ep_id),
                    number: ep_num,
                    title: Some(format!("Episode {}", ep_num as i32)),
                    image: None,
                    filler: false,
                });
            }
        }

        // data-id first, data-ep second (alternate HTML ordering)
        if sub_eps.is_empty() {
            for cap in ep_re_rev.captures_iter(&html) {
                let ep_id = cap[1].to_string();
                let ep_num = cap[2].parse::<f64>().unwrap_or(sub_eps.len() as f64 + 1.0);
                if !sub_eps.iter().any(|e| (e.number - ep_num).abs() < 0.01) {
                    sub_eps.push(EpisodeItem {
                        pipe_id: format!("anizone|{}", ep_id),
                        number: ep_num,
                        title: Some(format!("Episode {}", ep_num as i32)),
                        image: None,
                        filler: false,
                    });
                }
            }
        }

        sub_eps.sort_by(|a, b| a.number.partial_cmp(&b.number).unwrap_or(std::cmp::Ordering::Equal));

        Ok(ProviderData {
            name: self.name().to_string(),
            sub: sub_eps,
            dub: Vec::new(),
        })
    }

    async fn get_sources(
        &self,
        episode: &EpisodeItem,
        _category: Category,
    ) -> Result<SourcesResult, AppError> {
        // pipe_id format: "anizone|{ep_id}"
        let parts: Vec<&str> = episode.pipe_id.splitn(2, '|').collect();
        if parts.len() < 2 || parts[0] != "anizone" {
            return Err(AppError::NoSourcesFound);
        }
        let ep_id = parts[1];

        let client = Self::build_client();

        // AniZone AJAX endpoint for episode source
        let ajax_url = format!("https://anizone.to/ajax/episode/source?id={}", ep_id);
        let res = client
            .get(&ajax_url)
            .header("Referer", "https://anizone.to/")
            .header("X-Requested-With", "XMLHttpRequest")
            .send()
            .await;

        if let Ok(r) = res {
            if r.status().is_success() {
                if let Ok(json) = r.json::<serde_json::Value>().await {
                    // Extract direct HLS URL
                    if let Some(link) = json["link"].as_str().or_else(|| json["url"].as_str()) {
                        if link.contains(".m3u8") {
                            return Ok(SourcesResult {
                                streams: vec![StreamItem {
                                    url: link.to_string(),
                                    stream_type: "hls".to_string(),
                                    quality: Some("auto".to_string()),
                                    audio: None,
                                    referer: Some("https://anizone.to/".to_string()),
                                    is_active: true,
                                }],
                            });
                        }

                        // Follow embed link → regex extract .m3u8
                        if let Ok(embed_res) = client
                            .get(link)
                            .header("Referer", "https://anizone.to/")
                            .send()
                            .await
                        {
                            if let Ok(embed_html) = embed_res.text().await {
                                let m3u8_re = match regex::Regex::new(
                                    r#"https?://[^\s"'\\<>]+?\.m3u8[^\s"'\\<>]*"#,
                                ) {
                                    Ok(r) => r,
                                    Err(_) => return Err(AppError::NoSourcesFound),
                                };
                                if let Some(mat) = m3u8_re.find(&embed_html) {
                                    return Ok(SourcesResult {
                                        streams: vec![StreamItem {
                                            url: mat.as_str().to_string(),
                                            stream_type: "hls".to_string(),
                                            quality: Some("auto".to_string()),
                                            audio: None,
                                            referer: Some(link.to_string()),
                                            is_active: true,
                                        }],
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        Err(AppError::NoSourcesFound)
    }
}
