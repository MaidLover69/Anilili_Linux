use crate::error::AppError;
use crate::models::{Category, ProviderData, SourcesResult, StreamItem};
use crate::providers::anibd::AniBDProvider;
use crate::providers::anidbapp::AniDbAppProvider;
use crate::providers::animegg::AnimeGGProvider;
use crate::providers::animeheaven::AnimeHeavenProvider;
use crate::providers::animekai::AnimeKaiProvider;
use crate::providers::animepahe::AnimePaheProvider;
use crate::providers::anikoto::AniKotoProvider;
use crate::providers::animeshqip::AnimeShqipProvider;
use crate::providers::anizone::AniZoneProvider;
use crate::providers::gojo::GojoWtfProvider;
use crate::providers::kickassanime::KickAssAnimeProvider;
use crate::providers::miruro::MiruroProvider;
use crate::providers::rareanimes::RareAnimesProvider;
use crate::providers::senshi::SenshiProvider;
use crate::providers::AnimeProvider;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

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

pub fn apply_stream_headers(mut req: reqwest::RequestBuilder, stream: &StreamItem) -> reqwest::RequestBuilder {
    if let Some(ref ref_url) = stream.referer {
        req = req.header("Referer", ref_url);
        if let Ok(parsed) = reqwest::Url::parse(ref_url) {
            let origin = format!("{}://{}", parsed.scheme(), parsed.host_str().unwrap_or(""));
            req = req.header("Origin", origin);
        }
    } else if let Some(ref origin_url) = stream.origin {
        req = req.header("Origin", origin_url);
    }
    if let Some(ref headers) = stream.headers {
        for (k, v) in headers {
            req = req.header(k, v);
        }
    }
    req
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
                    }
                    if (300..400).contains(&st) {
                        if let Some(loc) = r.headers().get("Location").and_then(|h| h.to_str().ok()) {
                            curr_url = loc.to_string();
                            continue;
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

pub struct ProviderManager {
    providers: Vec<Arc<dyn AnimeProvider>>,
}

impl Default for ProviderManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ProviderManager {
    pub fn new() -> Self {
        Self {
            providers: vec![
                Arc::new(MiruroProvider::new("bonk")),
                Arc::new(MiruroProvider::new("kiwi")),
                Arc::new(MiruroProvider::new("pewe")),
                Arc::new(MiruroProvider::new("bee")),
                Arc::new(MiruroProvider::new("ally")),
                Arc::new(MiruroProvider::new("moo")),
                Arc::new(MiruroProvider::new("hop")),
                Arc::new(MiruroProvider::new("nun")),
                Arc::new(MiruroProvider::new("bun")),
                Arc::new(MiruroProvider::new("twin")),
                Arc::new(MiruroProvider::new("cog")),
                Arc::new(MiruroProvider::new("telli")),
                Arc::new(AniBDProvider),
                Arc::new(KickAssAnimeProvider::new()),
                Arc::new(AnimeKaiProvider::new()),
                Arc::new(AniDbAppProvider::new()),
                Arc::new(AnimeGGProvider::new()),
                Arc::new(AnimeShqipProvider::new()),
                Arc::new(RareAnimesProvider::new()),
                Arc::new(AniKotoProvider::new()),
                Arc::new(AniZoneProvider::new()),
                Arc::new(AnimePaheProvider::new()),
                Arc::new(AnimeHeavenProvider::new()),
                Arc::new(GojoWtfProvider::new()),
                Arc::new(SenshiProvider),
            ],
        }
    }

    pub fn provider_names(&self) -> Vec<&'static str> {
        self.providers.iter().map(|p| p.name()).collect()
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
                            return Ok(SourcesResult {
                                streams: verified,
                                subtitles: sources.subtitles,
                                skip: sources.skip,
                            });
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
