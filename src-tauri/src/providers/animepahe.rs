use crate::error::AppError;
use crate::models::{Category, EpisodeItem, ProviderData, SourcesResult, StreamItem};
use crate::providers::AnimeProvider;
use async_trait::async_trait;
use reqwest::Client;
use std::time::Duration;

pub struct AnimePaheProvider;

impl AnimePaheProvider {
    pub fn new() -> Self {
        Self
    }

    fn build_client() -> Client {
        Client::builder()
            .timeout(Duration::from_secs(12))
            .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:146.0) Gecko/20100101 Firefox/146.0")
            .build()
            .unwrap_or_else(|_| Client::new())
    }

    async fn search_session(client: &Client, title: &str) -> Option<String> {
        let clean_title = title.replace(|c: char| !c.is_alphanumeric() && c != ' ', " ");
        let url = format!(
            "https://animepahe.ru/api?m=search&q={}",
            clean_title.trim().replace(' ', "%20")
        );

        let res = client
            .get(&url)
            .header("Referer", "https://animepahe.ru/")
            .send()
            .await
            .ok()?;

        if !res.status().is_success() {
            return None;
        }

        let json: serde_json::Value = res.json().await.ok()?;
        let data = json["data"].as_array()?;

        let first = data.first()?;
        first["session"].as_str().map(String::from)
    }
}

#[async_trait]
impl AnimeProvider for AnimePaheProvider {
    fn name(&self) -> &'static str {
        "AnimePahe"
    }

    async fn get_episodes(
        &self,
        anilist_id: i64,
        _mal_id: Option<i64>,
        title_romaji: Option<&str>,
    ) -> Result<ProviderData, AppError> {
        let query = title_romaji.unwrap_or("").trim();
        let title = if query.is_empty() {
            anilist_id.to_string()
        } else {
            query.to_string()
        };

        let client = Self::build_client();
        let session = match Self::search_session(&client, &title).await {
            Some(s) => s,
            None => {
                return Ok(ProviderData {
                    name: self.name().to_string(),
                    sub: vec![],
                    dub: vec![],
                })
            }
        };

        let mut sub_eps: Vec<EpisodeItem> = Vec::new();
        let mut page = 1;

        loop {
            let url = format!(
                "https://animepahe.ru/api?m=release&id={}&sort=episode_asc&page={}",
                session, page
            );
            let res = match client
                .get(&url)
                .header("Referer", "https://animepahe.ru/")
                .send()
                .await
            {
                Ok(r) if r.status().is_success() => r,
                _ => break,
            };

            let json: serde_json::Value = match res.json().await {
                Ok(j) => j,
                Err(_) => break,
            };

            let data = match json["data"].as_array() {
                Some(d) if !d.is_empty() => d,
                _ => break,
            };

            for item in data {
                let ep_num = item["episode"].as_f64().unwrap_or(sub_eps.len() as f64 + 1.0);
                let ep_session = item["session"].as_str().unwrap_or_default();
                let snapshot = item["snapshot"].as_str().map(String::from);
                let ep_title = item["title"].as_str().map(String::from);
                let filler = item["filler"].as_i64().unwrap_or(0) == 1;

                if !ep_session.is_empty() {
                    sub_eps.push(EpisodeItem {
                        pipe_id: format!("animepahe|{}|{}", session, ep_session),
                        number: ep_num,
                        title: ep_title.or_else(|| Some(format!("Episode {}", ep_num as i32))),
                        image: snapshot,
                        synopsis: None,
                        filler,
                    });
                }
            }

            let last_page = json["last_page"].as_i64().unwrap_or(1);
            if page >= last_page || page >= 20 {
                break;
            }
            page += 1;
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
        if parts.len() < 3 {
            return Err(AppError::NoSourcesFound);
        }
        let (session, ep_sess) = (parts[1], parts[2]);

        let client = Self::build_client();
        let play_url = format!("https://animepahe.ru/play/{}/{}", session, ep_sess);
        let play_res = client
            .get(&play_url)
            .header("Referer", "https://animepahe.ru/")
            .send()
            .await
            .map_err(|e| AppError::Network(e.to_string()))?;

        let play_html = play_res
            .text()
            .await
            .map_err(|e| AppError::Network(e.to_string()))?;

        let link_re = regex::Regex::new(r#"data-src="([^"]+)""#).unwrap();
        let mut streams = Vec::new();

        for cap in link_re.captures_iter(&play_html) {
            let stream_url = cap[1].to_string();
            if stream_url.contains("http") {
                streams.push(StreamItem {
                    subtitle_variant: None,
                    url: stream_url,
                    stream_type: "hls".to_string(),
                    quality: Some("1080p".to_string()),
                    audio: Some("sub".to_string()),
                    referer: Some("https://animepahe.ru/".to_string()),
                    origin: Some("https://animepahe.ru".to_string()),
                    headers: None,
                    is_active: true,
                });
            }
        }

        if streams.is_empty() {
            let kwik_re = regex::Regex::new(r#"https?://kwik\.[a-z]+/[^"'\s]+"#).unwrap();
            for cap in kwik_re.captures_iter(&play_html) {
                streams.push(StreamItem {
                    subtitle_variant: None,
                    url: cap[0].to_string(),
                    stream_type: "hls".to_string(),
                    quality: Some("720p".to_string()),
                    audio: Some("sub".to_string()),
                    referer: Some("https://animepahe.ru/".to_string()),
                    origin: Some("https://animepahe.ru".to_string()),
                    headers: None,
                    is_active: true,
                });
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
