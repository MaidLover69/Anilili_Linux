use crate::error::AppError;
use crate::models::{Category, EpisodeItem, ProviderData, SourcesResult, StreamItem};
use crate::providers::AnimeProvider;
use async_trait::async_trait;
use reqwest::Client;
use std::time::Duration;

pub struct AniKotoProvider;

impl AniKotoProvider {
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
impl AnimeProvider for AniKotoProvider {
    fn name(&self) -> &'static str {
        "anikoto"
    }

    fn supports_dub(&self) -> bool {
        true
    }

    async fn get_episodes(
        &self,
        anilist_id: i64,
        _mal_id: Option<i64>,
        _title_romaji: Option<&str>,
    ) -> Result<ProviderData, AppError> {
        let client = Self::build_client();

        // MegaPlay API: GET /api/anime/{anilist_id}
        let url = format!("https://megaplay.buzz/api/anime/{}", anilist_id);
        let res = client
            .get(&url)
            .header("Referer", "https://megaplay.buzz/")
            .header("Origin", "https://megaplay.buzz")
            .send()
            .await;

        if let Ok(r) = res {
            if r.status().is_success() {
                if let Ok(json) = r.json::<serde_json::Value>().await {
                    let mut sub_eps = Vec::new();
                    let mut dub_eps = Vec::new();

                    // Response format: { sub: [...], dub: [...] } or flat array
                    let parse_eps = |arr: &serde_json::Value, audio: &str| -> Vec<EpisodeItem> {
                        arr.as_array()
                            .unwrap_or(&vec![])
                            .iter()
                            .enumerate()
                            .map(|(idx, ep)| {
                                let ep_num = ep["number"]
                                    .as_f64()
                                    .or_else(|| ep["ep"].as_f64())
                                    .unwrap_or((idx + 1) as f64);
                                let ep_id = ep["id"]
                                    .as_str()
                                    .or_else(|| ep["episodeId"].as_str())
                                    .unwrap_or("")
                                    .to_string();
                                EpisodeItem {
                                    pipe_id: format!("megaplay|{}|{}", audio, ep_id),
                                    number: ep_num,
                                    title: ep["title"].as_str().map(String::from),
                                    image: ep["thumbnail"]
                                        .as_str()
                                        .or_else(|| ep["image"].as_str())
                                        .map(String::from),
                                    synopsis: None,
                                    filler: ep["filler"].as_bool().unwrap_or(false),
                                }
                            })
                            .collect()
                    };

                    if let Some(sub_arr) = json.get("sub") {
                        sub_eps = parse_eps(sub_arr, "sub");
                    } else if json.is_array() {
                        sub_eps = parse_eps(&json, "sub");
                    }

                    if let Some(dub_arr) = json.get("dub") {
                        dub_eps = parse_eps(dub_arr, "dub");
                    }

                    if !sub_eps.is_empty() || !dub_eps.is_empty() {
                        return Ok(ProviderData {
                            name: self.name().to_string(),
                            sub: sub_eps,
                            dub: dub_eps,
                        });
                    }
                }
            }
        }

        Ok(ProviderData {
            name: self.name().to_string(),
            sub: Vec::new(),
            dub: Vec::new(),
        })
    }

    async fn get_sources(
        &self,
        episode: &EpisodeItem,
        _category: Category,
    ) -> Result<SourcesResult, AppError> {
        // pipe_id format: "megaplay|{audio}|{ep_id}"
        let parts: Vec<&str> = episode.pipe_id.splitn(3, '|').collect();
        if parts.len() < 3 || parts[0] != "megaplay" {
            return Err(AppError::NoSourcesFound);
        }
        let ep_id = parts[2];
        if ep_id.is_empty() {
            return Err(AppError::NoSourcesFound);
        }

        let client = Self::build_client();

        // MegaPlay API: GET /api/episode/{ep_id}
        let url = format!("https://megaplay.buzz/api/episode/{}", ep_id);
        let res = client
            .get(&url)
            .header("Referer", "https://megaplay.buzz/")
            .send()
            .await;

        if let Ok(r) = res {
            if r.status().is_success() {
                if let Ok(json) = r.json::<serde_json::Value>().await {
                    // Try direct hls field first
                    let hls_url = json["hls"]
                        .as_str()
                        .or_else(|| json["stream"].as_str())
                        .or_else(|| json["url"].as_str())
                        .map(String::from);

                    if let Some(url) = hls_url {
                        if url.contains(".m3u8") {
                            return Ok(SourcesResult {
                                streams: vec![StreamItem {
                    subtitle_variant: None,
                                    url,
                                    stream_type: "hls".to_string(),
                                    quality: Some("auto".to_string()),
                                    audio: None,
                                    referer: Some("https://megaplay.buzz/".to_string()),
                                    is_active: true,
                                    origin: None,
                                    headers: None,
                                }],
                                subtitles: vec![],
                                skip: None,
                            });
                        }
                    }

                    // Fallback: regex extract any .m3u8 from the JSON string
                    let text = json.to_string();
                    let m3u8_re = match regex::Regex::new(
                        r#"https?://[^\s"'<>]+?\.m3u8[^\s"'<>]*"#,
                    ) {
                        Ok(r) => r,
                        Err(_) => return Err(AppError::NoSourcesFound),
                    };
                    if let Some(mat) = m3u8_re.find(&text) {
                        return Ok(SourcesResult {
                            streams: vec![StreamItem {
                    subtitle_variant: None,
                                url: mat.as_str().to_string(),
                                stream_type: "hls".to_string(),
                                quality: Some("auto".to_string()),
                                audio: None,
                                referer: Some("https://megaplay.buzz/".to_string()),
                                is_active: true,
                                origin: None,
                                headers: None,
                            }],
                            subtitles: vec![],
                            skip: None,
                        });
                    }
                }
            }
        }

        Err(AppError::NoSourcesFound)
    }
}
