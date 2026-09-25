use crate::error::AppError;
use crate::models::{Category, EpisodeItem, ProviderData, SourcesResult, StreamItem};
use crate::providers::AnimeProvider;
use async_trait::async_trait;
use reqwest::Client;
use std::time::Duration;

pub struct AnimeGGProvider;

impl AnimeGGProvider {
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
impl AnimeProvider for AnimeGGProvider {
    fn name(&self) -> &'static str {
        "animegg"
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

        // Step 1: Search for series slug
        let query_slug = query_str.to_lowercase().replace(' ', "-");
        let exact_series_path = format!("/series/{}", query_slug);

        let search_url = format!("https://www.animegg.org/search?q={}", query_str.replace(' ', "+"));
        let search_res = client
            .get(&search_url)
            .header("Referer", "https://www.animegg.org/")
            .send()
            .await;

        let search_html = match search_res {
            Ok(r) if r.status().is_success() => r.text().await.unwrap_or_default(),
            _ => String::new(),
        };

        let series_re = regex::Regex::new(r#"href="(/series/[^"]+)""#).unwrap();
        let mut series_path = exact_series_path.clone();

        let mut candidate_slugs = Vec::new();
        for cap in series_re.captures_iter(&search_html) {
            candidate_slugs.push(cap[1].to_string());
        }

        if candidate_slugs.contains(&exact_series_path) {
            series_path = exact_series_path;
        } else if let Some(first) = candidate_slugs.first() {
            series_path = first.clone();
        }

        // Step 2: GET /series/{slug} to parse all episode links
        let series_url = format!("https://www.animegg.org{}", series_path);
        let series_res = client
            .get(&series_url)
            .header("Referer", "https://www.animegg.org/")
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

        // Parse episode links: href="(/[^"]+?-episode-(\d+(?:\.\d+)?))"
        let ep_re = regex::Regex::new(r#"href="(/[^"]+?-episode-(\d+(?:\.\d+)?)[^"]*)""#).unwrap();

        let mut sub_eps: Vec<EpisodeItem> = Vec::new();
        for cap in ep_re.captures_iter(&html) {
            let watch_path = cap[1].split('#').next().unwrap_or(&cap[1]).to_string();
            let ep_num = cap[2].parse::<f64>().unwrap_or(sub_eps.len() as f64 + 1.0);
            if !sub_eps.iter().any(|e| (e.number - ep_num).abs() < 0.01) {
                sub_eps.push(EpisodeItem {
                    pipe_id: format!("animegg|{}", watch_path),
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
            dub: sub_eps,
        })
    }

    async fn get_sources(
        &self,
        episode: &EpisodeItem,
        _category: Category,
    ) -> Result<SourcesResult, AppError> {
        let parts: Vec<&str> = episode.pipe_id.splitn(2, '|').collect();
        if parts.len() < 2 || parts[0] != "animegg" {
            return Err(AppError::NoSourcesFound);
        }
        let watch_path = parts[1];

        let client = Self::build_client();
        let watch_url = format!("https://www.animegg.org{}", watch_path);

        let res = client
            .get(&watch_url)
            .header("Referer", "https://www.animegg.org/")
            .send()
            .await;

        let html = match res {
            Ok(r) if r.status().is_success() => r.text().await.unwrap_or_default(),
            _ => return Err(AppError::NoSourcesFound),
        };

        // Check for direct embed iframe: src="(/embed/[^"]+)" or src="(https?://[^"]+)"
        let embed_re = regex::Regex::new(r#"src="(/embed/[^"]+|https?://[^"]*animegg[^"]*)"#).unwrap();
        let embed_src = embed_re.captures(&html).map(|c| c[1].to_string());

        let embed_url = match embed_src {
            Some(src) if src.starts_with('/') => format!("https://www.animegg.org{}", src),
            Some(src) => src,
            None => watch_url.clone(),
        };

        let embed_res = client
            .get(&embed_url)
            .header("Referer", &watch_url)
            .send()
            .await;

        if let Ok(er) = embed_res {
            if let Ok(embed_html) = er.text().await {
                let video_re = regex::Regex::new(r#"(?:file:\s*["']|src=["'])([^"']*(?:/play/|\.mp4|\.m3u8)[^"']*)"#).unwrap();
                let mut streams = Vec::new();
                for cap in video_re.captures_iter(&embed_html) {
                    let raw_path = cap[1].replace("\\/", "/");
                    let full_url = if raw_path.starts_with('/') {
                        format!("https://www.animegg.org{}", raw_path)
                    } else if raw_path.starts_with("http") {
                        raw_path
                    } else {
                        format!("https://www.animegg.org/{}", raw_path)
                    };
                    let is_hls = full_url.contains(".m3u8");
                    streams.push(StreamItem {
                        url: full_url,
                        stream_type: if is_hls { "hls".to_string() } else { "mp4".to_string() },
                        quality: Some("auto".to_string()),
                        audio: None,
                        referer: Some(embed_url.clone()),
                        is_active: true,
                    });
                }
                if !streams.is_empty() {
                    return Ok(SourcesResult { streams });
                }
            }
        }

        // Direct m3u8 fallback from watch page HTML
        let m3u8_re = regex::Regex::new(r#"https?://[^\s"'\\<>]+?\.m3u8[^\s"'\\<>]*"#).unwrap();
        if let Some(mat) = m3u8_re.find(&html) {
            return Ok(SourcesResult {
                streams: vec![StreamItem {
                    url: mat.as_str().to_string(),
                    stream_type: "hls".to_string(),
                    quality: Some("auto".to_string()),
                    audio: None,
                    referer: Some("https://www.animegg.org/".to_string()),
                    is_active: true,
                }],
            });
        }

        Err(AppError::NoSourcesFound)
    }
}

