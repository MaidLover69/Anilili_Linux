use crate::clients::{aniskip, jikan, konoha};
use crate::db;
use crate::models::{EpisodeItem, ProviderEpisodes, SkipTimes, SourcesResult};
use crate::state::AppState;
use std::collections::HashMap;
use tauri::State;

#[tauri::command]
pub async fn get_anime_catalog_episodes(
    state: State<'_, AppState>,
    anilist_id: i64,
    mal_id: Option<i64>,
    total_episodes: Option<i32>,
    title_romaji: Option<String>,
    status: Option<String>,
    next_airing_episode: Option<i32>,
) -> Result<Vec<EpisodeItem>, String> {
    let mut catalog_map: HashMap<i32, EpisodeItem> = HashMap::new();

    // 1. Fast fetch from Konoha CDN (thumbnails & titles)
    if let Ok(konoha_eps) = konoha::fetch_konoha_episodes(anilist_id).await {
        for ep in konoha_eps {
            catalog_map.insert(ep.number as i32, ep);
        }
    }

    // 2. Fetch MyAnimeList episode metadata (official titles, filler flags, synopses)
    if let Some(m_id) = mal_id {
        if let Ok(mal_eps) = jikan::fetch_mal_episodes(&state.client, m_id as i32).await {
            for mal_ep in mal_eps {
                let ep_num_i = mal_ep.number as i32;
                let entry = catalog_map.entry(ep_num_i).or_insert_with(|| EpisodeItem {
                    pipe_id: format!("{}-{}", anilist_id, ep_num_i),
                    number: mal_ep.number,
                    title: None,
                    image: None,
                    synopsis: None,
                    filler: false,
                });

                if let Some(ref title) = mal_ep.title {
                    if !title.trim().is_empty() {
                        entry.title = Some(title.clone());
                    }
                }
                entry.filler = mal_ep.filler;
                if entry.synopsis.is_none() {
                    entry.synopsis = mal_ep.synopsis;
                }
            }
        }
    }

    // 3. Fetch IMDb episode synopses
    if let Some(ref title) = title_romaji {
        let imdb_synopses = state.imdb_client.fetch_episode_synopses(title).await;
        for (ep_num, plot) in imdb_synopses {
            if let Some(entry) = catalog_map.get_mut(&ep_num) {
                entry.synopsis = Some(plot);
            }
        }
    }

    // 4. Determine currently released episode count
    let is_releasing = status.as_deref().map(|s| s.eq_ignore_ascii_case("RELEASING")).unwrap_or(false);
    let max_released = if let Some(next_ep) = next_airing_episode {
        (next_ep - 1).max(1)
    } else if is_releasing {
        (catalog_map.len() as i32).max(1)
    } else {
        total_episodes.unwrap_or(0).max(catalog_map.len() as i32)
    };

    // Filter out any unreleased future episodes if releasing
    if is_releasing && max_released > 0 {
        catalog_map.retain(|&num, _| num <= max_released);
    }

    // Fill in any gaps up to max_released
    if max_released > 0 {
        for num in 1..=max_released {
            catalog_map.entry(num).or_insert_with(|| EpisodeItem {
                pipe_id: format!("{}-{}", anilist_id, num),
                number: num as f64,
                title: Some(format!("Episode {}", num)),
                image: None,
                synopsis: None,
                filler: false,
            });
        }
    }

    let mut result: Vec<EpisodeItem> = catalog_map.into_values().collect();
    result.sort_by(|a, b| a.number.partial_cmp(&b.number).unwrap_or(std::cmp::Ordering::Equal));

    Ok(result)
}

#[tauri::command]
pub async fn get_all_episodes(
    state: State<'_, AppState>,
    anilist_id: i64,
    mal_id: Option<i64>,
    title_romaji: Option<String>,
) -> Result<HashMap<String, ProviderEpisodes>, String> {
    let pdata_list = state
        .provider_manager
        .get_all_episodes(anilist_id, mal_id, title_romaji.clone())
        .await
        .map_err(|e| e.to_string())?;

    // 1. Fetch MyAnimeList episode metadata (titles, filler tags, synopses)
    let mut mal_ep_map: HashMap<i32, jikan::MalEpisodeInfo> = HashMap::new();
    if let Some(m_id) = mal_id {
        if let Ok(mal_eps) = jikan::fetch_mal_episodes(&state.client, m_id as i32).await {
            for ep in mal_eps {
                mal_ep_map.insert(ep.number as i32, ep);
            }
        }
    }

    // 2. Fetch IMDb episode synopses
    let mut imdb_ep_map: HashMap<i32, String> = HashMap::new();
    if let Some(ref title) = title_romaji {
        let synopses = state.imdb_client.fetch_episode_synopses(title).await;
        for (ep_num, plot) in synopses {
            imdb_ep_map.insert(ep_num, plot);
        }
    }

    let mut map = HashMap::new();
    for pdata in pdata_list {
        let mut sub = pdata.sub;
        let mut dub = pdata.dub;

        // Enrich SUB episodes
        for ep in &mut sub {
            let ep_num_i = ep.number as i32;
            if let Some(mal_info) = mal_ep_map.get(&ep_num_i) {
                if let Some(ref mal_title) = mal_info.title {
                    if !mal_title.trim().is_empty() {
                        ep.title = Some(mal_title.clone());
                    }
                }
                ep.filler = mal_info.filler;
                if ep.synopsis.is_none() {
                    ep.synopsis = mal_info.synopsis.clone();
                }
            }
            if let Some(imdb_plot) = imdb_ep_map.get(&ep_num_i) {
                ep.synopsis = Some(imdb_plot.clone());
            }
        }

        // Enrich DUB episodes
        for ep in &mut dub {
            let ep_num_i = ep.number as i32;
            if let Some(mal_info) = mal_ep_map.get(&ep_num_i) {
                if let Some(ref mal_title) = mal_info.title {
                    if !mal_title.trim().is_empty() {
                        ep.title = Some(mal_title.clone());
                    }
                }
                ep.filler = mal_info.filler;
                if ep.synopsis.is_none() {
                    ep.synopsis = mal_info.synopsis.clone();
                }
            }
            if let Some(imdb_plot) = imdb_ep_map.get(&ep_num_i) {
                ep.synopsis = Some(imdb_plot.clone());
            }
        }

        map.insert(
            pdata.name,
            ProviderEpisodes { sub, dub },
        );
    }

    Ok(map)
}

#[tauri::command]
pub async fn fetch_mal_anime_synopsis(
    state: State<'_, AppState>,
    mal_id: i32,
) -> Result<jikan::MalAnimeDetails, String> {
    jikan::fetch_mal_details(&state.client, mal_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_episode_sources(
    state: State<'_, AppState>,
    anilist_id: i64,
    mal_id: Option<i64>,
    episode_number: f64,
    category: String,
    title_romaji: Option<String>,
) -> Result<SourcesResult, String> {
    state
        .provider_manager
        .get_sources_for_episode(
            anilist_id,
            mal_id,
            episode_number,
            &category,
            title_romaji,
        )
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_skip_times(
    mal_id: i64,
    episode_number: f64,
    duration_s: f64,
) -> Result<SkipTimes, String> {
    aniskip::fetch_skip_times(mal_id, episode_number, duration_s)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_konoha_episodes(anilist_id: i64) -> Result<Vec<EpisodeItem>, String> {
    konoha::fetch_konoha_episodes(anilist_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_filler_episodes(
    state: State<'_, AppState>,
    mal_id: i32,
) -> Result<Vec<i32>, String> {
    if let Ok(Some(cached)) = db::get_cached_filler_episodes(&state.pool, mal_id).await {
        return Ok(cached);
    }

    let episodes = jikan::fetch_filler_episodes(&state.client, mal_id)
        .await
        .map_err(|e| e.to_string())?;

    let _ = db::cache_filler_episodes(&state.pool, mal_id, &episodes).await;
    Ok(episodes)
}
