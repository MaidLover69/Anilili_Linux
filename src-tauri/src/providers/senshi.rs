use crate::clients::http::build_http_client;
use crate::error::AppError;
use crate::models::{Category, EpisodeItem, ProviderData, SourcesResult, StreamItem};
use crate::providers::AnimeProvider;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub struct SenshiProvider;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SenshiEpisodeRaw {
    pub ep_id: Option<f64>,
    pub ep_number: Option<f64>,
    pub number: Option<f64>,
    pub episode: Option<f64>,
    pub ep_title: Option<String>,
    pub title: Option<String>,
    pub ep_filler: Option<bool>,
    pub filler: Option<bool>,
    pub intro_start: Option<f64>,
    pub intro_end: Option<f64>,
    pub outro_start: Option<f64>,
    pub outro_end: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SenshiEmbedRaw {
    pub url: Option<String>,
    pub status: Option<String>,
    pub server2: Option<String>,
    #[serde(rename = "serverFM")]
    pub server_fm: Option<String>,
    pub download: Option<String>,
    pub masked_base_url: Option<String>,
}

#[async_trait]
impl AnimeProvider for SenshiProvider {
    fn name(&self) -> &'static str {
        "Senshi"
    }

    async fn get_episodes(
        &self,
        _anilist_id: i64,
        mal_id: Option<i64>,
        _title_romaji: Option<&str>,
    ) -> Result<ProviderData, AppError> {
        let mal = match mal_id {
            Some(m) => m,
            None => {
                return Ok(ProviderData {
                    name: "Senshi".to_string(),
                    sub: vec![],
                    dub: vec![],
                });
            }
        };

        let client = build_http_client();
        let url = format!("https://senshi.live/episodes/{}", mal);
        let resp = client
            .get(&url)
            .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64; rv:146.0) Gecko/20100101 Firefox/146.0")
            .header("Referer", "https://senshi.live/")
            .header("Accept", "application/json, */*")
            .send()
            .await;

        let response = match resp {
            Ok(r) if r.status().is_success() => r,
            _ => {
                return Ok(ProviderData {
                    name: "Senshi".to_string(),
                    sub: vec![],
                    dub: vec![],
                });
            }
        };

        let raw_list: Vec<SenshiEpisodeRaw> = match response.json().await {
            Ok(list) => list,
            Err(_) => {
                return Ok(ProviderData {
                    name: "Senshi".to_string(),
                    sub: vec![],
                    dub: vec![],
                });
            }
        };

        let mut sub_episodes = Vec::new();
        for (idx, raw) in raw_list.into_iter().enumerate() {
            let ep_num = raw.ep_id.or(raw.ep_number).or(raw.number).or(raw.episode).unwrap_or((idx + 1) as f64);
            let title = raw.ep_title.or(raw.title);
            let filler = raw.ep_filler.or(raw.filler).unwrap_or(false);
            sub_episodes.push(EpisodeItem {
                pipe_id: format!("{}-{}", mal, ep_num as i32),
                number: ep_num,
                title,
                image: None,
                synopsis: None,
                filler,
            });
        }

        Ok(ProviderData {
            name: "Senshi".to_string(),
            sub: sub_episodes,
            dub: vec![],
        })
    }

    async fn get_sources(
        &self,
        episode: &EpisodeItem,
        _category: Category,
    ) -> Result<SourcesResult, AppError> {
        let parts: Vec<&str> = episode.pipe_id.split('-').collect();
        if parts.len() < 2 {
            return Err(AppError::Other("Invalid pipe_id for Senshi".to_string()));
        }
        let mal_id = parts[0];
        let ep_num = episode.number as i32;

        let client = build_http_client();
        let url = format!("https://senshi.live/episode-embeds/{}/{}", mal_id, ep_num);
        let resp = client
            .get(&url)
            .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64; rv:146.0) Gecko/20100101 Firefox/146.0")
            .header("Referer", "https://senshi.live/")
            .header("Accept", "application/json, */*")
            .send()
            .await;

        if let Ok(r) = resp {
            if r.status().is_success() {
                if let Ok(embeds) = r.json::<Vec<SenshiEmbedRaw>>().await {
                    let mut streams = Vec::new();
                    for embed in embeds {
                        if let Some(stream_url) = embed.url.or(embed.server2).or(embed.server_fm) {
                            streams.push(StreamItem {
                    subtitle_variant: None,
                                url: stream_url,
                                stream_type: "hls".to_string(),
                                quality: Some("auto".to_string()),
                                audio: None,
                                referer: Some("https://senshi.live/".to_string()),
                                origin: Some("https://senshi.live".to_string()),
                                headers: None,
                                is_active: true,
                            });
                        }
                    }
                    if !streams.is_empty() {
                        return Ok(SourcesResult { streams, subtitles: Vec::new(), skip: None });
                    }
                }
            }
        }

        Err(AppError::NoSourcesFound)
    }
}
