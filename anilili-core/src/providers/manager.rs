use crate::error::AppError;
use crate::models::{Category, ProviderData, SourcesResult, StreamItem};
use crate::providers::anibd::AniBDProvider;
use crate::providers::anidbapp::AniDbAppProvider;
use crate::providers::animegg::AnimeGGProvider;
use crate::providers::animekai::AnimeKaiProvider;
use crate::providers::anikoto::AniKotoProvider;
use crate::providers::animeshqip::AnimeShqipProvider;
use crate::providers::anizone::AniZoneProvider;
use crate::providers::kickassanime::KickAssAnimeProvider;
use crate::providers::rareanimes::RareAnimesProvider;
use crate::providers::senshi::SenshiProvider;
use crate::providers::AnimeProvider;
use once_cell::sync::Lazy;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;

static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime")
});

static COOLDOWNS: Lazy<Mutex<HashMap<String, Instant>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

fn is_cooling_down(name: &str) -> bool {
    if let Ok(guard) = COOLDOWNS.lock() {
        if let Some(until) = guard.get(name) {
            return Instant::now() < *until;
        }
    }
    false
}

fn set_cooldown(name: &str) {
    if let Ok(mut guard) = COOLDOWNS.lock() {
        guard.insert(name.to_string(), Instant::now() + Duration::from_secs(30));
    }
}

fn clear_cooldown(name: &str) {
    if let Ok(mut guard) = COOLDOWNS.lock() {
        guard.remove(name);
    }
}

fn apply_stream_headers(mut req: reqwest::RequestBuilder, stream: &StreamItem) -> reqwest::RequestBuilder {
    if let Some(ref ref_url) = stream.referer {
        req = req.header("Referer", ref_url);
        if let Ok(parsed) = reqwest::Url::parse(ref_url) {
            let origin = format!("{}://{}", parsed.scheme(), parsed.host_str().unwrap_or(""));
            req = req.header("Origin", origin);
        }
    }
    req
}

pub struct ProviderManager {
    providers: Vec<Arc<dyn AnimeProvider>>,
}

pub async fn verify_stream_segment(stream: &StreamItem) -> bool {
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(4))
        .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:146.0) Gecko/20100101 Firefox/146.0")
        .build()
    {
        Ok(c) => c,
        Err(_) => return false,
    };

    if stream.stream_type == "mp4" || stream.url.contains(".mp4") {
        let client_no_redirect = match reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .redirect(reqwest::redirect::Policy::none())
            .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:146.0) Gecko/20100101 Firefox/146.0")
            .build()
        {
            Ok(c) => c,
            Err(_) => return false,
        };

        let mut curr_url = stream.url.clone();
        for _ in 0..5 {
            let req = apply_stream_headers(
                client_no_redirect.get(&curr_url).header("Range", "bytes=0-2048"),
                stream,
            );
            match req.send().await {
                Ok(r) => {
                    let st = r.status().as_u16();
                    if st == 200 || st == 206 {
                        return true;
                    } else if (st == 301 || st == 302 || st == 303 || st == 307 || st == 308) && r.headers().contains_key("location") {
                        if let Some(loc) = r.headers().get("location") {
                            if let Ok(loc_str) = loc.to_str() {
                                curr_url = loc_str.to_string();
                                continue;
                            }
                        }
                    }
                    return false;
                }
                Err(_) => return false,
            }
        }
        return false;
    }

    let req = apply_stream_headers(client.get(&stream.url), stream);

    let resp = match req.send().await {
        Ok(r) if r.status().is_success() => r,
        _ => return false,
    };

    let text = match resp.text().await {
        Ok(t) => t,
        Err(_) => return false,
    };

    if !text.contains("#EXTM3U") {
        return false;
    }

    let lines: Vec<&str> = text
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect();

    if lines.is_empty() {
        return false;
    }

    let first_entry = lines[0];
    let sub_url = if first_entry.starts_with("http") {
        first_entry.to_string()
    } else {
        let base = stream.url.rsplit_once('/').map(|(b, _)| b).unwrap_or(&stream.url);
        format!("{}/{}", base, first_entry)
    };

    let segment_url = if sub_url.contains(".m3u8") {
        let sub_req = apply_stream_headers(client.get(&sub_url), stream);
        let sub_resp = match sub_req.send().await {
            Ok(r) if r.status().is_success() => r,
            _ => return false,
        };
        let sub_text = match sub_resp.text().await {
            Ok(t) => t,
            Err(_) => return false,
        };
        let seg_lines: Vec<&str> = sub_text
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .collect();
        if seg_lines.is_empty() {
            return false;
        }
        let seg_entry = seg_lines[0];
        if seg_entry.starts_with("http") {
            seg_entry.to_string()
        } else {
            let base = sub_url.rsplit_once('/').map(|(b, _)| b).unwrap_or(&sub_url);
            format!("{}/{}", base, seg_entry)
        }
    } else {
        sub_url
    };

    let seg_req = apply_stream_headers(
        client.get(&segment_url).header("Range", "bytes=0-2048"),
        stream,
    );
    match seg_req.send().await {
        Ok(r) => r.status().is_success() || r.status().as_u16() == 206,
        Err(_) => false,
    }
}

impl ProviderManager {
    pub fn new() -> Self {
        Self {
            providers: vec![
                Arc::new(AniBDProvider),
                Arc::new(KickAssAnimeProvider::new()),
                Arc::new(AnimeKaiProvider::new()),
                Arc::new(AniDbAppProvider::new()),
                Arc::new(AnimeGGProvider::new()),
                Arc::new(AnimeShqipProvider::new()),
                Arc::new(RareAnimesProvider::new()),
                Arc::new(AniKotoProvider::new()),
                Arc::new(AniZoneProvider::new()),
                Arc::new(SenshiProvider),
            ],
        }
    }

    pub async fn get_all_episodes(
        &self,
        anilist_id: i64,
        mal_id: Option<i64>,
        title_romaji: Option<String>,
    ) -> Result<Vec<ProviderData>, AppError> {
        let mut futures = Vec::new();
        for p in &self.providers {
            if is_cooling_down(p.name()) {
                continue;
            }
            let provider = p.clone();
            let title = title_romaji.clone();
            futures.push(tokio::spawn(async move {
                (
                    provider.name(),
                    provider.get_episodes(anilist_id, mal_id, title.as_deref()).await,
                )
            }));
        }

        let mut results = Vec::new();
        for fut in futures {
            if let Ok((name, pdata_res)) = fut.await {
                match pdata_res {
                    Ok(pdata) => {
                        clear_cooldown(name);
                        results.push(pdata);
                    }
                    Err(_) => {
                        set_cooldown(name);
                    }
                }
            }
        }

        Ok(results)
    }

    pub async fn get_sources_for_episode(
        &self,
        anilist_id: i64,
        mal_id: Option<i64>,
        episode_number: f64,
        category_str: &str,
        title_romaji: Option<String>,
    ) -> Result<SourcesResult, AppError> {
        let cat = if category_str.to_lowercase() == "dub" {
            Category::Dub
        } else {
            Category::Sub
        };

        let mut tasks = Vec::new();

        for p in &self.providers {
            if is_cooling_down(p.name()) {
                continue;
            }
            let provider = p.clone();
            let title = title_romaji.clone();
            let cat_clone = cat.clone();

            tasks.push(tokio::spawn(async move {
                let name = provider.name();
                let fetch_fut = async {
                    let pdata = provider.get_episodes(anilist_id, mal_id, title.as_deref()).await?;
                    let ep_list = match cat_clone {
                        Category::Sub => &pdata.sub,
                        Category::Dub => &pdata.dub,
                    };

                    if let Some(target_ep) = ep_list.iter().find(|ep| (ep.number - episode_number).abs() < 0.01) {
                        let sources = provider.get_sources(target_ep, cat_clone).await?;
                        let mut verified = Vec::new();
                        for s in sources.streams {
                            if verify_stream_segment(&s).await {
                                verified.push(s);
                            }
                        }
                        if !verified.is_empty() {
                            return Ok(SourcesResult { streams: verified });
                        }
                    }
                    Err(AppError::NoSourcesFound)
                };

                let res = tokio::time::timeout(Duration::from_secs(25), fetch_fut).await;
                (name, res)
            }));
        }

        for task in tasks {
            if let Ok((name, timeout_res)) = task.await {
                match timeout_res {
                    Ok(Ok(sources)) => {
                        clear_cooldown(name);
                        return Ok(sources);
                    }
                    Ok(Err(AppError::NoSourcesFound)) => {
                        // Episode absent in catalog — do not cool down
                    }
                    _ => {
                        // HTTP error or timeout — set cooldown
                        set_cooldown(name);
                    }
                }
            }
        }

        Err(AppError::NoSourcesFound)
    }
}

#[pyfunction]
#[pyo3(signature = (anilist_id, mal_id=None, title_romaji=None, callback=None))]
pub fn get_all_episodes(
    anilist_id: i64,
    mal_id: Option<i64>,
    title_romaji: Option<String>,
    callback: Option<PyObject>,
) -> PyResult<()> {
    if let Some(cb) = callback {
        RUNTIME.spawn(async move {
            let manager = ProviderManager::new();
            let res = manager.get_all_episodes(anilist_id, mal_id, title_romaji).await;
            Python::with_gil(|py| match res {
                Ok(provider_list) => {
                    let dict = PyDict::new(py);
                    for pdata in &provider_list {
                        let p_dict = PyDict::new(py);
                        let _ = p_dict.set_item("sub", pdata.sub.clone());
                        let _ = p_dict.set_item("dub", pdata.dub.clone());
                        let _ = dict.set_item(&pdata.name, p_dict);
                    }
                    let none_err: Option<String> = None;
                    let _ = cb.call1(py, (true, dict, none_err));
                }
                Err(err) => {
                    let err_str = Some(err.to_string());
                    let empty_dict = PyDict::new(py);
                    let _ = cb.call1(py, (false, empty_dict, err_str));
                }
            });
        });
    }
    Ok(())
}

#[pyfunction]
#[pyo3(signature = (anilist_id, mal_id=None, episode_number=1.0, category="sub", title_romaji=None, callback=None))]
pub fn get_episode_sources(
    anilist_id: i64,
    mal_id: Option<i64>,
    episode_number: f64,
    category: &str,
    title_romaji: Option<String>,
    callback: Option<PyObject>,
) -> PyResult<()> {
    let cat = category.to_string();
    if let Some(cb) = callback {
        RUNTIME.spawn(async move {
            let manager = ProviderManager::new();
            let res = manager
                .get_sources_for_episode(anilist_id, mal_id, episode_number, &cat, title_romaji)
                .await;

            Python::with_gil(|py| match res {
                Ok(sources) => {
                    let none_err: Option<String> = None;
                    let _ = cb.call1(py, (true, sources.streams, none_err));
                }
                Err(err) => {
                    let err_str = Some(err.to_string());
                    let empty: Vec<StreamItem> = Vec::new();
                    let _ = cb.call1(py, (false, empty, err_str));
                }
            });
        });
    }
    Ok(())
}

#[pyfunction]
#[pyo3(signature = (anilist_id, mal_id=None, title_romaji=None, callback=None))]
pub fn kaa_get_episodes(
    anilist_id: i64,
    mal_id: Option<i64>,
    title_romaji: Option<String>,
    callback: Option<PyObject>,
) -> PyResult<()> {
    if let Some(cb) = callback {
        RUNTIME.spawn(async move {
            let provider = KickAssAnimeProvider::new();
            let res = provider.get_episodes(anilist_id, mal_id, title_romaji.as_deref()).await;
            Python::with_gil(|py| match res {
                Ok(pdata) => {
                    let dict = PyDict::new(py);
                    let _ = dict.set_item("sub", pdata.sub);
                    let _ = dict.set_item("dub", pdata.dub);
                    let none_err: Option<String> = None;
                    let _ = cb.call1(py, (true, dict, none_err));
                }
                Err(err) => {
                    let err_str = Some(err.to_string());
                    let empty_dict = PyDict::new(py);
                    let _ = cb.call1(py, (false, empty_dict, err_str));
                }
            });
        });
    }
    Ok(())
}

#[pyfunction]
#[pyo3(signature = (anilist_id, mal_id=None, title_romaji=None, callback=None))]
pub fn animekai_get_episodes(
    anilist_id: i64,
    mal_id: Option<i64>,
    title_romaji: Option<String>,
    callback: Option<PyObject>,
) -> PyResult<()> {
    if let Some(cb) = callback {
        RUNTIME.spawn(async move {
            let provider = AnimeKaiProvider::new();
            let res = provider.get_episodes(anilist_id, mal_id, title_romaji.as_deref()).await;
            Python::with_gil(|py| match res {
                Ok(pdata) => {
                    let dict = PyDict::new(py);
                    let _ = dict.set_item("sub", pdata.sub);
                    let _ = dict.set_item("dub", pdata.dub);
                    let none_err: Option<String> = None;
                    let _ = cb.call1(py, (true, dict, none_err));
                }
                Err(err) => {
                    let err_str = Some(err.to_string());
                    let empty_dict = PyDict::new(py);
                    let _ = cb.call1(py, (false, empty_dict, err_str));
                }
            });
        });
    }
    Ok(())
}

#[pyfunction]
#[pyo3(signature = (anilist_id, mal_id=None, title_romaji=None, callback=None))]
pub fn anikoto_get_episodes(
    anilist_id: i64,
    mal_id: Option<i64>,
    title_romaji: Option<String>,
    callback: Option<PyObject>,
) -> PyResult<()> {
    if let Some(cb) = callback {
        RUNTIME.spawn(async move {
            let provider = AniKotoProvider::new();
            let res = provider.get_episodes(anilist_id, mal_id, title_romaji.as_deref()).await;
            Python::with_gil(|py| match res {
                Ok(pdata) => {
                    let dict = PyDict::new(py);
                    let _ = dict.set_item("sub", pdata.sub);
                    let _ = dict.set_item("dub", pdata.dub);
                    let none_err: Option<String> = None;
                    let _ = cb.call1(py, (true, dict, none_err));
                }
                Err(err) => {
                    let err_str = Some(err.to_string());
                    let empty_dict = PyDict::new(py);
                    let _ = cb.call1(py, (false, empty_dict, err_str));
                }
            });
        });
    }
    Ok(())
}

#[pyfunction]
#[pyo3(signature = (anilist_id, mal_id=None, title_romaji=None, callback=None))]
pub fn animegg_get_episodes(
    anilist_id: i64,
    mal_id: Option<i64>,
    title_romaji: Option<String>,
    callback: Option<PyObject>,
) -> PyResult<()> {
    if let Some(cb) = callback {
        RUNTIME.spawn(async move {
            let provider = AnimeGGProvider::new();
            let res = provider.get_episodes(anilist_id, mal_id, title_romaji.as_deref()).await;
            Python::with_gil(|py| match res {
                Ok(pdata) => {
                    let dict = PyDict::new(py);
                    let _ = dict.set_item("sub", pdata.sub);
                    let _ = dict.set_item("dub", pdata.dub);
                    let none_err: Option<String> = None;
                    let _ = cb.call1(py, (true, dict, none_err));
                }
                Err(err) => {
                    let err_str = Some(err.to_string());
                    let empty_dict = PyDict::new(py);
                    let _ = cb.call1(py, (false, empty_dict, err_str));
                }
            });
        });
    }
    Ok(())
}

#[pyfunction]
#[pyo3(signature = (anilist_id, mal_id=None, title_romaji=None, callback=None))]
pub fn anizone_get_episodes(
    anilist_id: i64,
    mal_id: Option<i64>,
    title_romaji: Option<String>,
    callback: Option<PyObject>,
) -> PyResult<()> {
    if let Some(cb) = callback {
        RUNTIME.spawn(async move {
            let provider = AniZoneProvider::new();
            let res = provider.get_episodes(anilist_id, mal_id, title_romaji.as_deref()).await;
            Python::with_gil(|py| match res {
                Ok(pdata) => {
                    let dict = PyDict::new(py);
                    let _ = dict.set_item("sub", pdata.sub);
                    let _ = dict.set_item("dub", pdata.dub);
                    let none_err: Option<String> = None;
                    let _ = cb.call1(py, (true, dict, none_err));
                }
                Err(err) => {
                    let err_str = Some(err.to_string());
                    let empty_dict = PyDict::new(py);
                    let _ = cb.call1(py, (false, empty_dict, err_str));
                }
            });
        });
    }
    Ok(())
}

#[pyfunction]
#[pyo3(signature = (anilist_id, mal_id=None, title_romaji=None, callback=None))]
pub fn rareanimes_get_episodes(
    anilist_id: i64,
    mal_id: Option<i64>,
    title_romaji: Option<String>,
    callback: Option<PyObject>,
) -> PyResult<()> {
    if let Some(cb) = callback {
        RUNTIME.spawn(async move {
            let provider = RareAnimesProvider::new();
            let res = provider.get_episodes(anilist_id, mal_id, title_romaji.as_deref()).await;
            Python::with_gil(|py| match res {
                Ok(pdata) => {
                    let dict = PyDict::new(py);
                    let _ = dict.set_item("sub", pdata.sub);
                    let _ = dict.set_item("dub", pdata.dub);
                    let none_err: Option<String> = None;
                    let _ = cb.call1(py, (true, dict, none_err));
                }
                Err(err) => {
                    let err_str = Some(err.to_string());
                    let empty_dict = PyDict::new(py);
                    let _ = cb.call1(py, (false, empty_dict, err_str));
                }
            });
        });
    }
    Ok(())
}

#[pyfunction]
#[pyo3(signature = (anilist_id, mal_id=None, title_romaji=None, callback=None))]
pub fn animeshqip_get_episodes(
    anilist_id: i64,
    mal_id: Option<i64>,
    title_romaji: Option<String>,
    callback: Option<PyObject>,
) -> PyResult<()> {
    if let Some(cb) = callback {
        RUNTIME.spawn(async move {
            let provider = AnimeShqipProvider::new();
            let res = provider.get_episodes(anilist_id, mal_id, title_romaji.as_deref()).await;
            Python::with_gil(|py| match res {
                Ok(pdata) => {
                    let dict = PyDict::new(py);
                    let _ = dict.set_item("sub", pdata.sub);
                    let _ = dict.set_item("dub", pdata.dub);
                    let none_err: Option<String> = None;
                    let _ = cb.call1(py, (true, dict, none_err));
                }
                Err(err) => {
                    let err_str = Some(err.to_string());
                    let empty_dict = PyDict::new(py);
                    let _ = cb.call1(py, (false, empty_dict, err_str));
                }
            });
        });
    }
    Ok(())
}
