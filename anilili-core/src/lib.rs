pub mod cache;
pub mod clients;
pub mod db;
pub mod downloads;
pub mod error;
pub mod models;
pub mod providers;


use clients::{build_http_client, fetch_konoha_episodes, get_skip_times, AniListClient};
use models::*;
use once_cell::sync::Lazy;
use providers::{anibd_get_episodes, get_all_episodes, get_episode_sources, senshi_get_episodes};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::runtime::Runtime;

static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime")
});

static DB_POOL: Lazy<Arc<tokio::sync::OnceCell<SqlitePool>>> =
    Lazy::new(|| Arc::new(tokio::sync::OnceCell::new()));

async fn get_pool() -> &'static SqlitePool {
    DB_POOL
        .get_or_init(|| async {
            db::init_db()
                .await
                .expect("Failed to initialize SQLite database")
        })
        .await
}

#[pyfunction]
fn fetch_trending(callback: PyObject) -> PyResult<()> {
    RUNTIME.spawn(async move {
        let client = AniListClient::new(build_http_client());
        let res = client.fetch_media_list("TRENDING_DESC", 20).await;
        Python::with_gil(|py| {
            match res {
                Ok(media_list) => {
                    let none_err: Option<String> = None;
                    let _ = callback.call1(py, (true, media_list, none_err));
                }
                Err(err) => {
                    let err_str = Some(err.to_string());
                    let _ = callback.call1(py, (false, Vec::<Media>::new(), err_str));
                }
            }
        });
    });
    Ok(())
}

#[pyfunction]
fn fetch_popular(callback: PyObject) -> PyResult<()> {
    RUNTIME.spawn(async move {
        let client = AniListClient::new(build_http_client());
        let res = client.fetch_media_list("POPULARITY_DESC", 20).await;
        Python::with_gil(|py| {
            match res {
                Ok(media_list) => {
                    let none_err: Option<String> = None;
                    let _ = callback.call1(py, (true, media_list, none_err));
                }
                Err(err) => {
                    let err_str = Some(err.to_string());
                    let _ = callback.call1(py, (false, Vec::<Media>::new(), err_str));
                }
            }
        });
    });
    Ok(())
}

#[pyfunction]
fn fetch_top_rated(callback: PyObject) -> PyResult<()> {
    RUNTIME.spawn(async move {
        let client = AniListClient::new(build_http_client());
        let res = client.fetch_media_list("SCORE_DESC", 20).await;
        Python::with_gil(|py| {
            match res {
                Ok(media_list) => {
                    let none_err: Option<String> = None;
                    let _ = callback.call1(py, (true, media_list, none_err));
                }
                Err(err) => {
                    let err_str = Some(err.to_string());
                    let _ = callback.call1(py, (false, Vec::<Media>::new(), err_str));
                }
            }
        });
    });
    Ok(())
}

#[pyfunction]
fn fetch_home_data(callback: PyObject) -> PyResult<()> {
    RUNTIME.spawn(async move {
        let client = AniListClient::new(build_http_client());
        let pool = get_pool().await;

        let trending_fut = client.fetch_media_list("TRENDING_DESC", 10);
        let popular_fut = client.fetch_media_list("POPULARITY_DESC", 10);
        let top_rated_fut = client.fetch_media_list("SCORE_DESC", 10);
        let newest_fut = client.fetch_media_list("START_DATE_DESC", 10);
        let history_fut = db::get_continue_watching(pool);

        let (trending, popular, top_rated, newest, history) = tokio::join!(
            trending_fut,
            popular_fut,
            top_rated_fut,
            newest_fut,
            history_fut
        );

        let t_list = trending.unwrap_or_default();
        let p_list = popular.unwrap_or_default();
        let tr_list = top_rated.unwrap_or_default();
        let n_list = newest.unwrap_or_default();
        let h_list = history.unwrap_or_default();

        Python::with_gil(|py| {
            let dict = PyDict::new(py);
            let _ = dict.set_item("trending", t_list);
            let _ = dict.set_item("popular", p_list);
            let _ = dict.set_item("top_rated", tr_list);
            let _ = dict.set_item("newest", n_list);
            let _ = dict.set_item("continue_watching", h_list);

            let none_err: Option<String> = None;
            let _ = callback.call1(py, (true, dict, none_err));
        });
    });
    Ok(())
}

#[pyfunction]
#[pyo3(signature = (query=None, genres=Vec::new(), format=None, status=None, sort=None, page=1, per_page=20, callback=None))]
fn search_anime(
    query: Option<String>,
    genres: Vec<String>,
    format: Option<String>,
    status: Option<String>,
    sort: Option<String>,
    page: i32,
    per_page: i32,
    callback: Option<PyObject>,
) -> PyResult<()> {
    if let Some(cb) = callback {
        RUNTIME.spawn(async move {
            let client = AniListClient::new(build_http_client());
            let res = client
                .search(query, genres, format, status, sort, page, per_page)
                .await;

            Python::with_gil(|py| {
                match res {
                    Ok(media_list) => {
                        let none_err: Option<String> = None;
                        let _ = cb.call1(py, (true, media_list, none_err));
                    }
                    Err(err) => {
                        let err_str = Some(err.to_string());
                        let _ = cb.call1(py, (false, Vec::<Media>::new(), err_str));
                    }
                }
            });
        });
    }
    Ok(())
}

#[pyfunction]
fn fetch_anime_details(anilist_id: i64, callback: PyObject) -> PyResult<()> {
    RUNTIME.spawn(async move {
        let client = AniListClient::new(build_http_client());
        let res = client.fetch_details(anilist_id).await;

        Python::with_gil(|py| match res {
            Ok(json_val) => {
                let json_str = json_val.to_string();
                let none_err: Option<String> = None;
                let _ = callback.call1(py, (true, json_str, none_err));
            }
            Err(err) => {
                let err_str = Some(err.to_string());
                let _ = callback.call1(py, (false, "", err_str));
            }
        });
    });
    Ok(())
}

#[pyfunction]
fn get_konoha_episodes(anilist_id: i64, callback: PyObject) -> PyResult<()> {
    RUNTIME.spawn(async move {
        let res = fetch_konoha_episodes(anilist_id).await;

        Python::with_gil(|py| match res {
            Ok(episodes) => {
                let none_err: Option<String> = None;
                let _ = callback.call1(py, (true, episodes, none_err));
            }
            Err(err) => {
                let err_str = Some(err.to_string());
                let _ = callback.call1(py, (false, Vec::<EpisodeItem>::new(), err_str));
            }
        });
    });
    Ok(())
}

#[pyfunction]
fn get_watched_episodes(anilist_id: i32, callback: PyObject) -> PyResult<()> {
    RUNTIME.spawn(async move {
        let pool = get_pool().await;
        let res = db::get_watched_episodes(pool, anilist_id).await;

        Python::with_gil(|py| match res {
            Ok(episodes) => {
                let none_err: Option<String> = None;
                let _ = callback.call1(py, (true, episodes, none_err));
            }
            Err(err) => {
                let err_str = Some(err.to_string());
                let _ = callback.call1(py, (false, Vec::<f64>::new(), err_str));
            }
        });
    });
    Ok(())
}

#[pyfunction]
fn get_episode_progress(anilist_id: i32, episode_number: f64, callback: PyObject) -> PyResult<()> {
    RUNTIME.spawn(async move {
        let pool = get_pool().await;
        let res = db::get_episode_progress(pool, anilist_id, episode_number).await;

        Python::with_gil(|py| match res {
            Ok(Some((pos, dur))) => {
                let none_err: Option<String> = None;
                let _ = callback.call1(py, (true, pos, dur, none_err));
            }
            Ok(None) => {
                let none_err: Option<String> = None;
                let _ = callback.call1(py, (true, 0i64, 0i64, none_err));
            }
            Err(err) => {
                let err_str = Some(err.to_string());
                let _ = callback.call1(py, (false, 0i64, 0i64, err_str));
            }
        });
    });
    Ok(())
}

#[pyfunction]
fn get_continue_watching(callback: PyObject) -> PyResult<()> {
    RUNTIME.spawn(async move {
        let pool = get_pool().await;
        let res = db::get_continue_watching(pool).await;

        Python::with_gil(|py| match res {
            Ok(entries) => {
                let none_err: Option<String> = None;
                let _ = callback.call1(py, (true, entries, none_err));
            }
            Err(err) => {
                let err_str = Some(err.to_string());
                let _ = callback.call1(py, (false, Vec::<HistoryEntry>::new(), err_str));
            }
        });
    });
    Ok(())
}

#[pyfunction]
#[pyo3(signature = (anilist_id, title, cover, episode_number, episode_title, provider, category, position_ms, duration_ms, callback))]
fn update_watch_progress(
    anilist_id: i32,
    title: String,
    cover: Option<String>,
    episode_number: f64,
    episode_title: Option<String>,
    provider: String,
    category: String,
    position_ms: i64,
    duration_ms: i64,
    callback: PyObject,
) -> PyResult<()> {
    RUNTIME.spawn(async move {
        let pool = get_pool().await;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;

        let entry = HistoryEntry {
            anilist_id,
            title,
            cover,
            episode_number,
            episode_title,
            provider,
            category,
            position_ms,
            duration_ms,
            updated_at: now,
            from_remote: false,
        };

        let res = db::update_watch_progress(pool, &entry).await;

        Python::with_gil(|py| match res {
            Ok(_) => {
                let none_err: Option<String> = None;
                let _ = callback.call1(py, (true, none_err));
            }
            Err(err) => {
                let err_str = Some(err.to_string());
                let _ = callback.call1(py, (false, err_str));
            }
        });
    });
    Ok(())
}

#[pyfunction]
fn get_anilist_viewer(token: String, callback: PyObject) -> PyResult<()> {
    RUNTIME.spawn(async move {
        let client = AniListClient::new(build_http_client());
        let res = client.get_viewer(&token).await;

        Python::with_gil(|py| match res {
            Ok(viewer) => {
                let none_err: Option<String> = None;
                let _ = callback.call1(py, (true, viewer, none_err));
            }
            Err(err) => {
                let err_str = Some(err.to_string());
                let _ = callback.call1(py, (false, Option::<Viewer>::None, err_str));
            }
        });
    });
    Ok(())
}

#[pyfunction]
#[pyo3(signature = (token, user_id, status=None, callback=None))]
fn fetch_anilist_library(
    token: String,
    user_id: i32,
    status: Option<String>,
    callback: Option<PyObject>,
) -> PyResult<()> {
    if let Some(cb) = callback {
        RUNTIME.spawn(async move {
            let client = AniListClient::new(build_http_client());
            let res = client.get_user_library(&token, user_id, status).await;

            Python::with_gil(|py| match res {
                Ok(entries) => {
                    let none_err: Option<String> = None;
                    let _ = cb.call1(py, (true, entries, none_err));
                }
                Err(err) => {
                    let err_str = Some(err.to_string());
                    let _ = cb.call1(py, (false, Vec::<MediaListEntry>::new(), err_str));
                }
            });
        });
    }
    Ok(())
}

#[pyfunction]
#[pyo3(signature = (token, media_id, status=None, progress=None, score=None, callback=None))]
fn save_anilist_entry(
    token: String,
    media_id: i32,
    status: Option<String>,
    progress: Option<i32>,
    score: Option<f64>,
    callback: Option<PyObject>,
) -> PyResult<()> {
    RUNTIME.spawn(async move {
        let client = AniListClient::new(build_http_client());
        let res = client.save_list_entry(&token, media_id, status, progress, score).await;

        if let Some(cb) = callback {
            Python::with_gil(|py| match res {
                Ok(entry) => {
                    let none_err: Option<String> = None;
                    let _ = cb.call1(py, (true, entry, none_err));
                }
                Err(err) => {
                    let err_str = Some(err.to_string());
                    let _ = cb.call1(py, (false, Option::<MediaListEntry>::None, err_str));
                }
            });
        }
    });
    Ok(())
}

#[pyfunction]
#[pyo3(signature = (access_token, mal_id, episode, status=None, callback=None))]
fn mal_update_status(
    access_token: String,
    mal_id: i32,
    episode: i32,
    status: Option<String>,
    callback: Option<PyObject>,
) -> PyResult<()> {
    RUNTIME.spawn(async move {
        let client = clients::MalClient::new(build_http_client());
        let res = client.update_anime_status(&access_token, mal_id, episode, status).await;

        if let Some(cb) = callback {
            Python::with_gil(|py| match res {
                Ok(_) => {
                    let none_err: Option<String> = None;
                    let _ = cb.call1(py, (true, none_err));
                }
                Err(err) => {
                    let err_str = Some(err.to_string());
                    let _ = cb.call1(py, (false, err_str));
                }
            });
        }
    });
    Ok(())
}

#[pyfunction]
#[pyo3(signature = (anilist_id, title, cover=None, format=None, average_score=None, callback=None))]
fn add_to_watchlist(
    anilist_id: i32,
    title: String,
    cover: Option<String>,
    format: Option<String>,
    average_score: Option<i32>,
    callback: Option<PyObject>,
) -> PyResult<()> {
    RUNTIME.spawn(async move {
        let pool = get_pool().await;
        let res = db::add_to_watchlist(pool, anilist_id, &title, cover.as_deref(), format.as_deref(), average_score).await;

        if let Some(cb) = callback {
            Python::with_gil(|py| match res {
                Ok(_) => {
                    let none_err: Option<String> = None;
                    let _ = cb.call1(py, (true, none_err));
                }
                Err(err) => {
                    let err_str = Some(err.to_string());
                    let _ = cb.call1(py, (false, err_str));
                }
            });
        }
    });
    Ok(())
}

#[pyfunction]
#[pyo3(signature = (anilist_id, callback=None))]
fn remove_from_watchlist(anilist_id: i32, callback: Option<PyObject>) -> PyResult<()> {
    RUNTIME.spawn(async move {
        let pool = get_pool().await;
        let res = db::remove_from_watchlist(pool, anilist_id).await;

        if let Some(cb) = callback {
            Python::with_gil(|py| match res {
                Ok(_) => {
                    let none_err: Option<String> = None;
                    let _ = cb.call1(py, (true, none_err));
                }
                Err(err) => {
                    let err_str = Some(err.to_string());
                    let _ = cb.call1(py, (false, err_str));
                }
            });
        }
    });
    Ok(())
}

#[pyfunction]
fn is_in_watchlist(anilist_id: i32, callback: PyObject) -> PyResult<()> {
    RUNTIME.spawn(async move {
        let pool = get_pool().await;
        let res = db::is_in_watchlist(pool, anilist_id).await;

        Python::with_gil(|py| match res {
            Ok(exists) => {
                let none_err: Option<String> = None;
                let _ = callback.call1(py, (true, exists, none_err));
            }
            Err(err) => {
                let err_str = Some(err.to_string());
                let _ = callback.call1(py, (false, false, err_str));
            }
        });
    });
    Ok(())
}

#[pyfunction]
fn get_watchlist(callback: PyObject) -> PyResult<()> {
    RUNTIME.spawn(async move {
        let pool = get_pool().await;
        let res = db::get_watchlist(pool).await;

        Python::with_gil(|py| match res {
            Ok(entries) => {
                let none_err: Option<String> = None;
                let _ = callback.call1(py, (true, entries, none_err));
            }
            Err(err) => {
                let err_str = Some(err.to_string());
                let _ = callback.call1(py, (false, Vec::<WatchlistEntry>::new(), err_str));
            }
        });
    });
    Ok(())
}

#[pyfunction]
#[pyo3(signature = (anilist_id, status=None, progress=0, score=0.0, title=None, cover=None, format_str=None, total_episodes=None, average_score=None, callback=None))]
fn upsert_library_entry(
    anilist_id: i32,
    status: Option<String>,
    progress: i32,
    score: f64,
    title: Option<String>,
    cover: Option<String>,
    format_str: Option<String>,
    total_episodes: Option<i32>,
    average_score: Option<i32>,
    callback: Option<PyObject>,
) -> PyResult<()> {
    RUNTIME.spawn(async move {
        let pool = get_pool().await;
        let entry = MediaListEntry {
            id: anilist_id,
            status,
            progress,
            score,
            title,
            cover,
            format_str,
            total_episodes,
            average_score,
        };

        let res = db::upsert_library_entry(pool, &entry).await;

        if let Some(cb) = callback {
            Python::with_gil(|py| match res {
                Ok(_) => {
                    let none_err: Option<String> = None;
                    let _ = cb.call1(py, (true, none_err));
                }
                Err(err) => {
                    let err_str = Some(err.to_string());
                    let _ = cb.call1(py, (false, err_str));
                }
            });
        }
    });
    Ok(())
}

#[pyfunction]
#[pyo3(signature = (status=None, callback=None))]
fn get_library_entries(status: Option<String>, callback: Option<PyObject>) -> PyResult<()> {
    if let Some(cb) = callback {
        RUNTIME.spawn(async move {
            let pool = get_pool().await;
            let res = db::get_library_entries(pool, status).await;

            Python::with_gil(|py| match res {
                Ok(entries) => {
                    let none_err: Option<String> = None;
                    let _ = cb.call1(py, (true, entries, none_err));
                }
                Err(err) => {
                    let err_str = Some(err.to_string());
                    let _ = cb.call1(py, (false, Vec::<MediaListEntry>::new(), err_str));
                }
            });
        });
    }
    Ok(())
}

#[pyfunction]
#[pyo3(signature = (record, callback=None))]
fn create_download(
    record: DownloadRecord,
    callback: Option<PyObject>,
) -> PyResult<()> {
    RUNTIME.spawn(async move {
        let pool = get_pool().await;
        let res = db::create_download(pool, &record).await;

        if let Some(cb) = callback {
            Python::with_gil(|py| match res {
                Ok(id) => {
                    let none_err: Option<String> = None;
                    let _ = cb.call1(py, (true, id, none_err));
                }
                Err(err) => {
                    let err_str = Some(err.to_string());
                    let _ = cb.call1(py, (false, "", err_str));
                }
            });
        }
    });
    Ok(())
}

#[pyfunction]
#[pyo3(signature = (id, status, progress=None, file_path=None, file_size=None, duration_s=None, error_msg=None, callback=None))]
fn update_download_status(
    id: String,
    status: String,
    progress: Option<f64>,
    file_path: Option<String>,
    file_size: Option<i64>,
    duration_s: Option<f64>,
    error_msg: Option<String>,
    callback: Option<PyObject>,
) -> PyResult<()> {
    RUNTIME.spawn(async move {
        let pool = get_pool().await;
        let res = db::update_download_status(
            pool,
            &id,
            &status,
            progress,
            file_path.as_deref(),
            file_size,
            duration_s,
            error_msg.as_deref(),
        )
        .await;

        if let Some(cb) = callback {
            Python::with_gil(|py| match res {
                Ok(_) => {
                    let none_err: Option<String> = None;
                    let _ = cb.call1(py, (true, none_err));
                }
                Err(err) => {
                    let err_str = Some(err.to_string());
                    let _ = cb.call1(py, (false, err_str));
                }
            });
        }
    });
    Ok(())
}

#[pyfunction]
#[pyo3(signature = (anilist_id, callback=None))]
fn get_downloads(anilist_id: i64, callback: Option<PyObject>) -> PyResult<()> {
    if let Some(cb) = callback {
        RUNTIME.spawn(async move {
            let pool = get_pool().await;
            let res = db::get_downloads(pool, anilist_id).await;

            Python::with_gil(|py| match res {
                Ok(records) => {
                    let none_err: Option<String> = None;
                    let _ = cb.call1(py, (true, records, none_err));
                }
                Err(err) => {
                    let err_str = Some(err.to_string());
                    let _ = cb.call1(py, (false, Vec::<DownloadRecord>::new(), err_str));
                }
            });
        });
    }
    Ok(())
}

#[pyfunction]
#[pyo3(signature = (callback=None))]
fn get_all_downloads(callback: Option<PyObject>) -> PyResult<()> {
    if let Some(cb) = callback {
        RUNTIME.spawn(async move {
            let pool = get_pool().await;
            let res = db::get_all_downloads(pool).await;

            Python::with_gil(|py| match res {
                Ok(records) => {
                    let none_err: Option<String> = None;
                    let _ = cb.call1(py, (true, records, none_err));
                }
                Err(err) => {
                    let err_str = Some(err.to_string());
                    let _ = cb.call1(py, (false, Vec::<DownloadRecord>::new(), err_str));
                }
            });
        });
    }
    Ok(())
}

#[pyfunction]
#[pyo3(signature = (id, callback=None))]
fn delete_download(id: String, callback: Option<PyObject>) -> PyResult<()> {
    RUNTIME.spawn(async move {
        let pool = get_pool().await;
        let res = db::delete_download(pool, &id).await;

        if let Some(cb) = callback {
            Python::with_gil(|py| match res {
                Ok(_) => {
                    let none_err: Option<String> = None;
                    let _ = cb.call1(py, (true, none_err));
                }
                Err(err) => {
                    let err_str = Some(err.to_string());
                    let _ = cb.call1(py, (false, err_str));
                }
            });
        }
    });
    Ok(())
}

#[pyfunction]
#[pyo3(signature = (id, callback=None))]
fn get_download_by_id(id: String, callback: Option<PyObject>) -> PyResult<()> {
    if let Some(cb) = callback {
        RUNTIME.spawn(async move {
            let pool = get_pool().await;
            let res = db::get_download_by_id(pool, &id).await;

            Python::with_gil(|py| match res {
                Ok(rec) => {
                    let none_err: Option<String> = None;
                    let _ = cb.call1(py, (true, rec, none_err));
                }
                Err(err) => {
                    let err_str = Some(err.to_string());
                    let _ = cb.call1(py, (false, Option::<DownloadRecord>::None, err_str));
                }
            });
        });
    }
    Ok(())
}

#[pyfunction]
fn fetch_airing_schedule(from_ts: i64, to_ts: i64, callback: PyObject) -> PyResult<()> {
    RUNTIME.spawn(async move {
        let pool = get_pool().await;
        // Check cache first
        if let Ok(Some(cached_entries)) = db::get_cached_schedule(pool, from_ts, to_ts).await {
            Python::with_gil(|py| {
                let none_err: Option<String> = None;
                let _ = callback.call1(py, (true, cached_entries, none_err));
            });
            return;
        }

        let client = AniListClient::new(build_http_client());
        let res = client.fetch_airing_schedule(from_ts, to_ts).await;

        match res {
            Ok(entries) => {
                let _ = db::cache_schedule(pool, from_ts, to_ts, &entries).await;
                Python::with_gil(|py| {
                    let none_err: Option<String> = None;
                    let _ = callback.call1(py, (true, entries, none_err));
                });
            }
            Err(err) => {
                Python::with_gil(|py| {
                    let err_str = Some(err.to_string());
                    let _ = callback.call1(py, (false, Vec::<AiringEntry>::new(), err_str));
                });
            }
        }
    });
    Ok(())
}

#[pyfunction]
fn get_filler_episodes(mal_id: i32, callback: PyObject) -> PyResult<()> {
    RUNTIME.spawn(async move {
        let pool = get_pool().await;
        if let Ok(Some(cached_episodes)) = db::get_cached_filler_episodes(pool, mal_id).await {
            Python::with_gil(|py| {
                let none_err: Option<String> = None;
                let _ = callback.call1(py, (true, cached_episodes, none_err));
            });
            return;
        }

        let client = build_http_client();
        let res = clients::fetch_filler_episodes(&client, mal_id).await;

        match res {
            Ok(episodes) => {
                let _ = db::cache_filler_episodes(pool, mal_id, &episodes).await;
                Python::with_gil(|py| {
                    let none_err: Option<String> = None;
                    let _ = callback.call1(py, (true, episodes, none_err));
                });
            }
            Err(err) => {
                Python::with_gil(|py| {
                    let err_str = Some(err.to_string());
                    let _ = callback.call1(py, (false, Vec::<i32>::new(), err_str));
                });
            }
        }
    });
    Ok(())
}

#[pyfunction]
#[pyo3(signature = (pref, callback=None))]
fn save_notification_preference(pref: NotificationPreference, callback: Option<PyObject>) -> PyResult<()> {
    RUNTIME.spawn(async move {
        let pool = get_pool().await;
        let res = db::save_notification_preference(pool, &pref).await;

        if let Some(cb) = callback {
            Python::with_gil(|py| match res {
                Ok(_) => {
                    let none_err: Option<String> = None;
                    let _ = cb.call1(py, (true, none_err));
                }
                Err(err) => {
                    let err_str = Some(err.to_string());
                    let _ = cb.call1(py, (false, err_str));
                }
            });
        }
    });
    Ok(())
}

#[pyfunction]
fn get_notification_preference(media_id: i64, callback: PyObject) -> PyResult<()> {
    RUNTIME.spawn(async move {
        let pool = get_pool().await;
        let res = db::get_notification_preference(pool, media_id).await;

        Python::with_gil(|py| match res {
            Ok(pref) => {
                let none_err: Option<String> = None;
                let _ = callback.call1(py, (true, pref, none_err));
            }
            Err(err) => {
                let err_str = Some(err.to_string());
                let _ = callback.call1(py, (false, Option::<NotificationPreference>::None, err_str));
            }
        });
    });
    Ok(())
}

#[pyfunction]
fn list_notification_preferences(callback: PyObject) -> PyResult<()> {
    RUNTIME.spawn(async move {
        let pool = get_pool().await;
        let res = db::list_notification_preferences(pool).await;

        Python::with_gil(|py| match res {
            Ok(prefs) => {
                let none_err: Option<String> = None;
                let _ = callback.call1(py, (true, prefs, none_err));
            }
            Err(err) => {
                let err_str = Some(err.to_string());
                let _ = callback.call1(py, (false, Vec::<NotificationPreference>::new(), err_str));
            }
        });
    });
    Ok(())
}

#[pymodule]
fn anilili_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<MediaTitle>()?;
    m.add_class::<CoverImage>()?;
    m.add_class::<FuzzyDate>()?;
    m.add_class::<MediaTag>()?;
    m.add_class::<Studio>()?;
    m.add_class::<NextAiringEpisode>()?;
    m.add_class::<Media>()?;

    m.add_class::<Category>()?;
    m.add_class::<EpisodeItem>()?;
    m.add_class::<ProviderData>()?;
    m.add_class::<StreamItem>()?;
    m.add_class::<SourcesResult>()?;

    m.add_class::<HistoryEntry>()?;
    m.add_class::<WatchlistEntry>()?;
    m.add_class::<MediaListEntry>()?;
    m.add_class::<Viewer>()?;
    m.add_class::<DownloadRecord>()?;
    m.add_class::<downloads::StorageCheck>()?;
    m.add_class::<AiringEntry>()?;
    m.add_class::<NotificationPreference>()?;

    m.add_function(wrap_pyfunction!(fetch_trending, m)?)?;
    m.add_function(wrap_pyfunction!(fetch_popular, m)?)?;
    m.add_function(wrap_pyfunction!(fetch_top_rated, m)?)?;
    m.add_function(wrap_pyfunction!(fetch_home_data, m)?)?;
    m.add_function(wrap_pyfunction!(search_anime, m)?)?;
    m.add_function(wrap_pyfunction!(fetch_anime_details, m)?)?;
    m.add_function(wrap_pyfunction!(get_konoha_episodes, m)?)?;
    m.add_function(wrap_pyfunction!(senshi_get_episodes, m)?)?;
    m.add_function(wrap_pyfunction!(anibd_get_episodes, m)?)?;
    m.add_function(wrap_pyfunction!(get_all_episodes, m)?)?;
    m.add_function(wrap_pyfunction!(get_episode_sources, m)?)?;
    m.add_function(wrap_pyfunction!(get_skip_times, m)?)?;
    m.add_function(wrap_pyfunction!(get_watched_episodes, m)?)?;
    m.add_function(wrap_pyfunction!(get_episode_progress, m)?)?;
    m.add_function(wrap_pyfunction!(get_continue_watching, m)?)?;
    m.add_function(wrap_pyfunction!(update_watch_progress, m)?)?;

    m.add_function(wrap_pyfunction!(get_anilist_viewer, m)?)?;
    m.add_function(wrap_pyfunction!(fetch_anilist_library, m)?)?;
    m.add_function(wrap_pyfunction!(save_anilist_entry, m)?)?;
    m.add_function(wrap_pyfunction!(mal_update_status, m)?)?;
    m.add_function(wrap_pyfunction!(add_to_watchlist, m)?)?;
    m.add_function(wrap_pyfunction!(remove_from_watchlist, m)?)?;
    m.add_function(wrap_pyfunction!(is_in_watchlist, m)?)?;
    m.add_function(wrap_pyfunction!(get_watchlist, m)?)?;
    m.add_function(wrap_pyfunction!(upsert_library_entry, m)?)?;
    m.add_function(wrap_pyfunction!(get_library_entries, m)?)?;

    m.add_function(wrap_pyfunction!(create_download, m)?)?;
    m.add_function(wrap_pyfunction!(update_download_status, m)?)?;
    m.add_function(wrap_pyfunction!(get_downloads, m)?)?;
    m.add_function(wrap_pyfunction!(get_all_downloads, m)?)?;
    m.add_function(wrap_pyfunction!(delete_download, m)?)?;
    m.add_function(wrap_pyfunction!(get_download_by_id, m)?)?;
    m.add_function(wrap_pyfunction!(downloads::check_storage_sync, m)?)?;
    m.add_function(wrap_pyfunction!(downloads::check_storage_for_download, m)?)?;
    m.add_function(wrap_pyfunction!(downloads::get_download_directory, m)?)?;

    m.add_function(wrap_pyfunction!(fetch_airing_schedule, m)?)?;
    m.add_function(wrap_pyfunction!(get_filler_episodes, m)?)?;
    m.add_function(wrap_pyfunction!(save_notification_preference, m)?)?;
    m.add_function(wrap_pyfunction!(get_notification_preference, m)?)?;
    m.add_function(wrap_pyfunction!(list_notification_preferences, m)?)?;

    m.add_class::<clients::github::UpdateInfo>()?;
    m.add_function(wrap_pyfunction!(check_for_update, m)?)?;
    m.add_function(wrap_pyfunction!(providers::kaa_get_episodes, m)?)?;
    m.add_function(wrap_pyfunction!(providers::animekai_get_episodes, m)?)?;
    m.add_function(wrap_pyfunction!(providers::anikoto_get_episodes, m)?)?;
    m.add_function(wrap_pyfunction!(providers::animegg_get_episodes, m)?)?;
    m.add_function(wrap_pyfunction!(providers::anizone_get_episodes, m)?)?;
    m.add_function(wrap_pyfunction!(providers::rareanimes_get_episodes, m)?)?;
    m.add_function(wrap_pyfunction!(providers::animeshqip_get_episodes, m)?)?;

    Ok(())
}

#[pyfunction]
#[pyo3(signature = (callback=None))]
fn check_for_update(callback: Option<PyObject>) -> PyResult<()> {
    RUNTIME.spawn(async move {
        let client = reqwest::Client::new();
        let opt_info = clients::github::fetch_latest_release(&client).await;

        if let Some(cb) = callback {
            Python::with_gil(|py| match opt_info {
                Some(info) => {
                    let none_err: Option<String> = None;
                    let _ = cb.call1(py, (true, info, none_err));
                }
                None => {
                    let err_str = Some("No release info available".to_string());
                    let _ = cb.call1(py, (false, py.None(), err_str));
                }
            });
        }
    });
    Ok(())
}




