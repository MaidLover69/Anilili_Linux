use crate::clients::http::build_http_client;
use crate::error::AppError;
use crate::models::{Category, EpisodeItem, ProviderData, SourcesResult, StreamItem};
use crate::providers::AnimeProvider;
use async_trait::async_trait;
use once_cell::sync::Lazy;
use pyo3::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tokio::runtime::Runtime;

static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime")
});

static M3U8_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"https://[^"\s]+\.m3u8"#).expect("Failed to compile m3u8 regex")
});

pub struct AniBDProvider;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AniBDDataItem {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub link: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AniBDServerGroup {
    pub server_name: Option<String>,
    pub server_data: Option<Vec<AniBDDataItem>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AniBDPlayerLink {
    pub server: Option<String>,
    pub link: Option<String>,
}

#[async_trait]
impl AnimeProvider for AniBDProvider {
    fn name(&self) -> &'static str {
        "AniBD"
    }

    async fn get_episodes(
        &self,
        anilist_id: i64,
        _mal_id: Option<i64>,
        _title_romaji: Option<&str>,
    ) -> Result<ProviderData, AppError> {
        let client = build_http_client();
        let url = format!("https://epeng.animeapps.top/api2.php?epid={}", anilist_id);
        let resp = client
            .get(&url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36")
            .header("Referer", "https://epeng.animeapps.top/")
            .send()
            .await;

        let response = match resp {
            Ok(r) if r.status().is_success() => r,
            _ => {
                return Ok(ProviderData {
                    name: "AniBD".to_string(),
                    sub: vec![],
                    dub: vec![],
                });
            }
        };

        let server_groups: Vec<AniBDServerGroup> = match response.json().await {
            Ok(list) => list,
            Err(_) => {
                return Ok(ProviderData {
                    name: "AniBD".to_string(),
                    sub: vec![],
                    dub: vec![],
                });
            }
        };

        let mut sub_episodes = Vec::new();
        let mut dub_episodes = Vec::new();

        for group in server_groups {
            let is_dub = group.server_name.as_deref().unwrap_or("").to_lowercase().contains("dub");
            if let Some(items) = group.server_data {
                for (idx, item) in items.into_iter().enumerate() {
                    let ep_num = item.name.as_deref()
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or((idx + 1) as f64);
                    let link_val = item.link.unwrap_or_default();
                    let ep_item = EpisodeItem {
                        pipe_id: link_val,
                        number: ep_num,
                        title: Some(format!("Episode {}", ep_num as i32)),
                        image: None,
                        filler: false,
                    };

                    if is_dub {
                        dub_episodes.push(ep_item);
                    } else {
                        sub_episodes.push(ep_item);
                    }
                }
            }
        }

        Ok(ProviderData {
            name: "AniBD".to_string(),
            sub: sub_episodes,
            dub: dub_episodes,
        })
    }

    async fn get_sources(
        &self,
        episode: &EpisodeItem,
        _category: Category,
    ) -> Result<SourcesResult, AppError> {
        if !episode.pipe_id.is_empty() {
            let client = build_http_client();
            let url = format!("https://epeng.animeapps.top/apilink.php?data={}", episode.pipe_id);
            let resp = client
                .get(&url)
                .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36")
                .header("Referer", "https://epeng.animeapps.top/")
                .send()
                .await;

            if let Ok(r) = resp {
                if r.status().is_success() {
                    if let Ok(players) = r.json::<Vec<AniBDPlayerLink>>().await {
                        for player in players {
                            if let Some(player_link) = player.link {
                                if let Ok(p_resp) = client.get(&player_link).header("Referer", "https://epeng.animeapps.top/").send().await {
                                    if let Ok(html) = p_resp.text().await {
                                        if let Some(mat) = M3U8_REGEX.find(&html) {
                                            return Ok(SourcesResult {
                                                streams: vec![StreamItem {
                                                    url: mat.as_str().to_string(),
                                                    stream_type: "hls".to_string(),
                                                    quality: Some("auto".to_string()),
                                                    audio: None,
                                                    referer: Some("https://epeng.animeapps.top".to_string()),
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
            }
        }

        Err(AppError::NoSourcesFound)
    }
}

#[pyfunction]
#[pyo3(signature = (anilist_id, callback=None))]
pub fn anibd_get_episodes(anilist_id: i64, callback: Option<PyObject>) -> PyResult<()> {
    if let Some(cb) = callback {
        RUNTIME.spawn(async move {
            let provider = AniBDProvider;
            let res = provider.get_episodes(anilist_id, None, None).await;
            Python::with_gil(|py| match res {
                Ok(pdata) => {
                    let none_err: Option<String> = None;
                    let _ = cb.call1(py, (true, pdata, none_err));
                }
                Err(err) => {
                    let err_str = Some(err.to_string());
                    let empty = ProviderData {
                        name: "AniBD".to_string(),
                        sub: vec![],
                        dub: vec![],
                    };
                    let _ = cb.call1(py, (false, empty, err_str));
                }
            });
        });
    }
    Ok(())
}

