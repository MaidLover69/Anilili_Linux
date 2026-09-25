use crate::error::AppError;
use crate::models::{Category, EpisodeItem, ProviderData, SourcesResult, StreamItem};
use crate::providers::AnimeProvider;
use async_trait::async_trait;
use reqwest::Client;
use std::time::Duration;

pub struct AnimeKaiProvider;

impl AnimeKaiProvider {
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
impl AnimeProvider for AnimeKaiProvider {
    fn name(&self) -> &'static str {
        "animekai"
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
        let client = Self::build_client();
        let query = title_romaji.unwrap_or_else(|| "").trim();
        let query_str = if query.is_empty() {
            anilist_id.to_string()
        } else {
            query.to_string()
        };

        let mirrors = ["https://anikai.cc", "https://animekai.to", "https://animekai.bz"];

        for mirror in &mirrors {
            // Step 1: Search by title query
            let search_url = format!("{}/browser?keyword={}", mirror, query_str.replace(' ', "+"));
            let search_res = client
                .get(&search_url)
                .header("Referer", format!("{}/", mirror))
                .send()
                .await;

            let search_html = match search_res {
                Ok(r) if r.status().is_success() => r.text().await.unwrap_or_default(),
                _ => continue,
            };

            // Extract show slug: href="/watch/([^"/]+)"
            let slug_re = match regex::Regex::new(r#"href="[^"]*watch/([^"/]+)""#) {
                Ok(r) => r,
                Err(_) => continue,
            };

            let show_slug = match slug_re.captures(&search_html) {
                Some(cap) => cap[1].to_string(),
                None => continue,
            };

            // Step 2: Fetch main show watch page /watch/{slug} to extract ALL episode links & data-num attrs
            let watch_url = format!("{}/watch/{}", mirror, show_slug);
            let watch_res = client
                .get(&watch_url)
                .header("Referer", format!("{}/", mirror))
                .send()
                .await;

            let watch_html = match watch_res {
                Ok(r) if r.status().is_success() => r.text().await.unwrap_or_default(),
                _ => continue,
            };

            let mut sub_eps = Vec::new();

            // Match data-num="(\d+)" or href=".../watch/{slug}/ep-(\d+)" or data-id
            let data_num_re = regex::Regex::new(r#"data-num="(\d+(?:\.\d+)?)"[^>]*data-id="([^"]+)""#).unwrap();
            let ep_href_re = regex::Regex::new(r#"href="[^"]*watch/[^"]*/ep-(\d+(?:\.\d+)?)"#).unwrap();

            for cap in data_num_re.captures_iter(&watch_html) {
                if let Ok(ep_num) = cap[1].parse::<f64>() {
                    let ep_id = cap[2].to_string();
                    if !sub_eps.iter().any(|e: &EpisodeItem| (e.number - ep_num).abs() < 0.01) {
                        sub_eps.push(EpisodeItem {
                            pipe_id: format!("{}|{}|{}|{}", mirror, show_slug, ep_num as i32, ep_id),
                            number: ep_num,
                            title: Some(format!("Episode {}", ep_num as i32)),
                            image: None,
                            filler: false,
                        });
                    }
                }
            }

            if sub_eps.is_empty() {
                for cap in ep_href_re.captures_iter(&watch_html) {
                    if let Ok(ep_num) = cap[1].parse::<f64>() {
                        if !sub_eps.iter().any(|e: &EpisodeItem| (e.number - ep_num).abs() < 0.01) {
                            sub_eps.push(EpisodeItem {
                                pipe_id: format!("{}|{}|{}", mirror, show_slug, ep_num as i32),
                                number: ep_num,
                                title: Some(format!("Episode {}", ep_num as i32)),
                                image: None,
                                filler: false,
                            });
                        }
                    }
                }
            }

            sub_eps.sort_by(|a, b| a.number.partial_cmp(&b.number).unwrap_or(std::cmp::Ordering::Equal));

            if !sub_eps.is_empty() {
                return Ok(ProviderData {
                    name: self.name().to_string(),
                    sub: sub_eps.clone(),
                    dub: sub_eps,
                });
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
        // pipe_id format: "{mirror}|{show_slug}|{ep_num}" or "{mirror}|{show_slug}|{ep_num}|{ep_id}"
        let parts: Vec<&str> = episode.pipe_id.split('|').collect();
        if parts.len() < 3 {
            return Err(AppError::NoSourcesFound);
        }
        let (mirror, show_slug, ep_num) = (parts[0], parts[1], parts[2]);

        let client = Self::build_client();
        let ep_watch_url = format!("{}/watch/{}/ep-{}", mirror, show_slug, ep_num);

        let res = client
            .get(&ep_watch_url)
            .header("Referer", format!("{}/", mirror))
            .send()
            .await;

        let html = match res {
            Ok(r) if r.status().is_success() => r.text().await.unwrap_or_default(),
            _ => return Err(AppError::NoSourcesFound),
        };

        // Extract server embeds from data-video or src attributes
        let video_re = regex::Regex::new(r#"(?:data-video|src)="([^"]+)""#).unwrap();

        let mut streams = Vec::new();
        for cap in video_re.captures_iter(&html) {
            let embed_url = cap[1].replace("\\/", "/");
            if embed_url.contains("http") && !embed_url.contains("google") && !embed_url.contains("analytics") {
                if embed_url.ends_with(".m3u8") || embed_url.contains(".m3u8?") {
                    streams.push(StreamItem {
                        url: embed_url.clone(),
                        stream_type: "hls".to_string(),
                        quality: Some("auto".to_string()),
                        audio: None,
                        referer: Some(format!("{}/", mirror)),
                        is_active: true,
                    });
                } else {
                    // Fetch embed page to resolve .m3u8
                    let embed_res = client
                        .get(&embed_url)
                        .header("Referer", &ep_watch_url)
                        .send()
                        .await;

                    if let Ok(er) = embed_res {
                        if let Ok(embed_html) = er.text().await {
                            let m3u8_re = regex::Regex::new(r#"https?://[^\s"'<>]+?\.m3u8[^\s"'<>]*"#).unwrap();
                            if let Some(mat) = m3u8_re.find(&embed_html) {
                                let m3u8_url = mat.as_str().replace("\\/", "/");
                                let referer_host = if let Ok(parsed) = reqwest::Url::parse(&embed_url) {
                                    format!("{}://{}/", parsed.scheme(), parsed.host_str().unwrap_or(""))
                                } else {
                                    format!("{}/", mirror)
                                };
                                streams.push(StreamItem {
                                    url: m3u8_url,
                                    stream_type: "hls".to_string(),
                                    quality: Some("auto".to_string()),
                                    audio: None,
                                    referer: Some(referer_host),
                                    is_active: true,
                                });
                                break;
                            }
                        }
                    }
                }
            }
        }

        if !streams.is_empty() {
            return Ok(SourcesResult { streams });
        }

        Err(AppError::NoSourcesFound)
    }
}

