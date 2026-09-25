use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HistoryEntry {
    #[pyo3(get, set)]
    pub anilist_id: i32,
    #[pyo3(get, set)]
    pub title: String,
    #[pyo3(get, set)]
    pub cover: Option<String>,
    #[pyo3(get, set)]
    pub episode_number: f64,
    #[pyo3(get, set)]
    pub episode_title: Option<String>,
    #[pyo3(get, set)]
    pub provider: String,
    #[pyo3(get, set)]
    pub category: String, // "sub" or "dub"
    #[pyo3(get, set)]
    pub position_ms: i64,
    #[pyo3(get, set)]
    pub duration_ms: i64,
    #[pyo3(get, set)]
    pub updated_at: i64,
    #[pyo3(get, set)]
    pub from_remote: bool,
}

#[pymethods]
impl HistoryEntry {
    #[new]
    #[pyo3(signature = (anilist_id, title, cover, episode_number, episode_title, provider, category, position_ms, duration_ms, updated_at, from_remote=false))]
    pub fn new(
        anilist_id: i32,
        title: String,
        cover: Option<String>,
        episode_number: f64,
        episode_title: Option<String>,
        provider: String,
        category: String,
        position_ms: i64,
        duration_ms: i64,
        updated_at: i64,
        from_remote: bool,
    ) -> Self {
        Self {
            anilist_id,
            title,
            cover,
            episode_number,
            episode_title,
            provider,
            category,
            position_ms,
            duration_ms,
            updated_at,
            from_remote,
        }
    }
}

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WatchlistEntry {
    #[pyo3(get, set)]
    pub anilist_id: i32,
    #[pyo3(get, set)]
    pub title: String,
    #[pyo3(get, set)]
    pub cover: Option<String>,
    #[pyo3(get, set)]
    pub format: Option<String>,
    #[pyo3(get, set)]
    pub average_score: Option<i32>,
    #[pyo3(get, set)]
    pub added_at: i64,
}

#[pymethods]
impl WatchlistEntry {
    #[new]
    #[pyo3(signature = (anilist_id, title, cover=None, format=None, average_score=None, added_at=0))]
    pub fn new(
        anilist_id: i32,
        title: String,
        cover: Option<String>,
        format: Option<String>,
        average_score: Option<i32>,
        added_at: i64,
    ) -> Self {
        Self {
            anilist_id,
            title,
            cover,
            format,
            average_score,
            added_at,
        }
    }
}

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MediaListEntry {
    #[pyo3(get, set)]
    pub id: i32,
    #[pyo3(get, set)]
    pub progress: i32,
    #[pyo3(get, set)]
    pub score: f64,
    #[pyo3(get, set)]
    pub status: Option<String>,
    #[pyo3(get, set)]
    pub title: Option<String>,
    #[pyo3(get, set)]
    pub cover: Option<String>,
    #[pyo3(get, set)]
    pub format_str: Option<String>,
    #[pyo3(get, set)]
    pub total_episodes: Option<i32>,
    #[pyo3(get, set)]
    pub average_score: Option<i32>,
}

#[pymethods]
impl MediaListEntry {
    #[new]
    #[pyo3(signature = (id, progress, score, status=None, title=None, cover=None, format_str=None, total_episodes=None, average_score=None))]
    pub fn new(
        id: i32,
        progress: i32,
        score: f64,
        status: Option<String>,
        title: Option<String>,
        cover: Option<String>,
        format_str: Option<String>,
        total_episodes: Option<i32>,
        average_score: Option<i32>,
    ) -> Self {
        Self {
            id,
            progress,
            score,
            status,
            title,
            cover,
            format_str,
            total_episodes,
            average_score,
        }
    }
}

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Viewer {
    #[pyo3(get, set)]
    pub id: i32,
    #[pyo3(get, set)]
    pub name: String,
    #[pyo3(get, set)]
    pub avatar_url: Option<String>,
    #[pyo3(get, set)]
    pub anime_count: i32,
    #[pyo3(get, set)]
    pub episodes_watched: i32,
    #[pyo3(get, set)]
    pub minutes_watched: i64,
    #[pyo3(get, set)]
    pub mean_score: f64,
}

#[pymethods]
impl Viewer {
    #[new]
    #[pyo3(signature = (id, name, avatar_url=None, anime_count=0, episodes_watched=0, minutes_watched=0, mean_score=0.0))]
    pub fn new(
        id: i32,
        name: String,
        avatar_url: Option<String>,
        anime_count: i32,
        episodes_watched: i32,
        minutes_watched: i64,
        mean_score: f64,
    ) -> Self {
        Self {
            id,
            name,
            avatar_url,
            anime_count,
            episodes_watched,
            minutes_watched,
            mean_score,
        }
    }
}

