use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AiringEntry {
    #[pyo3(get, set)]
    pub id: i64,
    #[pyo3(get, set)]
    pub airing_at: i64,
    #[pyo3(get, set)]
    pub episode: i32,
    #[pyo3(get, set)]
    pub media_id: i64,
    #[pyo3(get, set)]
    pub media_title: String,
    #[pyo3(get, set)]
    pub cover_image: Option<String>,
    #[pyo3(get, set)]
    pub banner_image: Option<String>,
    #[pyo3(get, set)]
    pub format: Option<String>,
    #[pyo3(get, set)]
    pub duration: Option<i32>,
    #[pyo3(get, set)]
    pub average_score: Option<i32>,
}

#[pymethods]
impl AiringEntry {
    #[new]
    #[pyo3(signature = (id, airing_at, episode, media_id, media_title, cover_image=None, banner_image=None, format=None, duration=None, average_score=None))]
    pub fn new(
        id: i64,
        airing_at: i64,
        episode: i32,
        media_id: i64,
        media_title: String,
        cover_image: Option<String>,
        banner_image: Option<String>,
        format: Option<String>,
        duration: Option<i32>,
        average_score: Option<i32>,
    ) -> Self {
        Self {
            id,
            airing_at,
            episode,
            media_id,
            media_title,
            cover_image,
            banner_image,
            format,
            duration,
            average_score,
        }
    }
}

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NotificationPreference {
    #[pyo3(get, set)]
    pub media_id: i64,
    #[pyo3(get, set)]
    pub enabled: bool,
    #[pyo3(get, set)]
    pub media_title: String,
    #[pyo3(get, set)]
    pub cover_image: Option<String>,
}

#[pymethods]
impl NotificationPreference {
    #[new]
    #[pyo3(signature = (media_id, enabled, media_title, cover_image=None))]
    pub fn new(
        media_id: i64,
        enabled: bool,
        media_title: String,
        cover_image: Option<String>,
    ) -> Self {
        Self {
            media_id,
            enabled,
            media_title,
            cover_image,
        }
    }
}
