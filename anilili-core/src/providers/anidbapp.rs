use crate::error::AppError;
use crate::models::{Category, EpisodeItem, ProviderData, SourcesResult, StreamItem};
use crate::providers::AnimeProvider;
use async_trait::async_trait;
use reqwest::Client;
use std::time::Duration;

pub struct AniDbAppProvider {
    client: Client,
}

impl AniDbAppProvider {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("Miruro Android/1.0")
            .build()
            .unwrap_or_else(|_| Client::new());
        Self { client }
    }
}

#[async_trait]
impl AnimeProvider for AniDbAppProvider {
    fn name(&self) -> &'static str {
        "anidbapp"
    }

    fn supports_dub(&self) -> bool {
        true
    }

    async fn get_episodes(
        &self,
        anilist_id: i64,
        _mal_id: Option<i64>,
        title_romaji: Option<&str>,
    ) -> Result<ProviderData, AppError> {
        let query = title_romaji.unwrap_or("").trim();
        let query_str = if query.is_empty() {
            anilist_id.to_string()
        } else {
            query.to_string()
        };

        // Step 1: Search suggestions
        let search_url = format!("https://anidb.app/search/suggestions?q={}", query_str.replace(' ', "+"));
        let res = self.client
            .get(&search_url)
            .header("User-Agent", "Miruro Android/1.0")
            .header("X-Requested-With", "XMLHttpRequest")
            .header("Referer", "https://anidb.app/")
            .send()
            .await;

        let search_html = match res {
            Ok(r) if r.status().is_success() => r.text().await.unwrap_or_default(),
            _ => return Ok(ProviderData {
                name: self.name().to_string(),
                sub: Vec::new(),
                dub: Vec::new(),
            }),
        };

        // Extract show slug and siteId from search HTML:
        // href="https://anidb.app/anime/{slug}" or href="/anime/{slug}"
        let slug_re = regex::Regex::new(r#"href="https?://anidb\.app/anime/([^"]+)""#).unwrap();
        let slug = match slug_re.captures(&search_html).map(|c| c[1].to_string()) {
            Some(s) => s,
            None => {
                let alt_re = regex::Regex::new(r#"href="/anime/([^"]+)""#).unwrap();
                match alt_re.captures(&search_html).map(|c| c[1].to_string()) {
                    Some(s) => s,
                    None => return Ok(ProviderData {
                        name: self.name().to_string(),
                        sub: Vec::new(),
                        dub: Vec::new(),
                    }),
                }
            }
        };

        // Step 2: GET /anime/{slug} to find siteId or fetch episodes
        let anime_url = format!("https://anidb.app/anime/{}", slug);
        let anime_res = self.client
            .get(&anime_url)
            .header("User-Agent", "Miruro Android/1.0")
            .header("Referer", "https://anidb.app/")
            .send()
            .await;

        let anime_html = match anime_res {
            Ok(r) if r.status().is_success() => r.text().await.unwrap_or_default(),
            _ => String::new(),
        };

        let site_id_re = regex::Regex::new(r#"data-site-id="([^"]+)""#).unwrap();
        let site_id = site_id_re
            .captures(&anime_html)
            .map(|c| c[1].to_string())
            .unwrap_or_else(|| slug.clone());

        // Step 3: GET /api/frontend/anime/{siteId}/episodes
        let ep_api_url = format!("https://anidb.app/api/frontend/anime/{}/episodes", site_id);
        let ep_res = self.client
            .get(&ep_api_url)
            .header("User-Agent", "Miruro Android/1.0")
            .header("X-Requested-With", "XMLHttpRequest")
            .header("Referer", &anime_url)
            .send()
            .await;

        let json_arr: serde_json::Value = match ep_res {
            Ok(r) if r.status().is_success() => r.json().await.unwrap_or_default(),
            _ => serde_json::Value::Null,
        };

        let mut sub_eps = Vec::new();
        if let Some(arr) = json_arr.as_array() {
            for item in arr {
                let ep_id = item["id"]
                    .as_str()
                    .map(String::from)
                    .or_else(|| item["id"].as_i64().map(|i| i.to_string()))
                    .unwrap_or_default();

                let ep_num = item["number"]
                    .as_f64()
                    .or_else(|| item["episode_number"].as_f64())
                    .unwrap_or(sub_eps.len() as f64 + 1.0);

                let title = item["title"].as_str().map(String::from);
                let filler = item["filler"].as_bool().unwrap_or(false);

                if !ep_id.is_empty() {
                    sub_eps.push(EpisodeItem {
                        pipe_id: format!("anidbapp|{}|{}", slug, ep_id),
                        number: ep_num,
                        title,
                        image: None,
                        filler,
                    });
                }
            }
        }

        sub_eps.sort_by(|a, b| a.number.partial_cmp(&b.number).unwrap_or(std::cmp::Ordering::Equal));

        Ok(ProviderData {
            name: self.name().to_string(),
            sub: sub_eps.clone(),
            dub: sub_eps,
        })
    }

    async fn get_sources(&self, episode: &EpisodeItem, category: Category) -> Result<SourcesResult, AppError> {
        let parts: Vec<&str> = episode.pipe_id.splitn(3, '|').collect();
        if parts.len() < 3 || parts[0] != "anidbapp" {
            return Err(AppError::NoSourcesFound);
        }
        let (slug, ep_id) = (parts[1], parts[2]);

        // GET /api/frontend/episode/{ep_id}/languages
        let lang_url = format!("https://anidb.app/api/frontend/episode/{}/languages", ep_id);
        let lang_res = self.client
            .get(&lang_url)
            .header("User-Agent", "Miruro Android/1.0")
            .header("X-Requested-With", "XMLHttpRequest")
            .header("Referer", format!("https://anidb.app/anime/{}", slug))
            .send()
            .await;

        let json_val: serde_json::Value = match lang_res {
            Ok(r) if r.status().is_success() => r.json().await.unwrap_or_default(),
            _ => return Err(AppError::NoSourcesFound),
        };

        let target_lang = match category {
            Category::Dub => "en-US",
            Category::Sub => "ja-JP",
        };

        let embed_url = json_val[target_lang]["embedUrl"]
            .as_str()
            .or_else(|| json_val["ja-JP"]["embedUrl"].as_str())
            .or_else(|| json_val[0]["embedUrl"].as_str())
            .map(String::from);

        let embed_url = match embed_url {
            Some(u) => u,
            None => return Err(AppError::NoSourcesFound),
        };

        // GET {embedUrl} to parse .m3u8 URL
        let embed_res = self.client
            .get(&embed_url)
            .header("Referer", format!("https://anidb.app/anime/{}", slug))
            .send()
            .await;

        if let Ok(r) = embed_res {
            if let Ok(text) = r.text().await {
                let re = regex::Regex::new(r#"(https?://[^\s"'<>]+?\.m3u8[^\s"'<>]*)"#).unwrap();
                let mut streams = Vec::new();
                for cap in re.captures_iter(&text) {
                    streams.push(StreamItem {
                        url: cap[1].replace("\\/", "/"),
                        stream_type: "hls".to_string(),
                        quality: Some("auto".to_string()),
                        audio: None,
                        referer: Some("https://anidb.app/".to_string()),
                        is_active: true,
                    });
                }
                if !streams.is_empty() {
                    return Ok(SourcesResult { streams });
                }
            }
        }

        Err(AppError::NoSourcesFound)
    }
}

