use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

pub const P1080_BYTES: i64 = 576_716_800; // 550 MB
pub const P720_BYTES: i64 = 314_572_800;  // 300 MB
pub const P480_BYTES: i64 = 157_286_400;  // 150 MB
pub const P360_BYTES: i64 = 83_886_080;   // 80 MB
pub const HEADROOM_BYTES: i64 = 1_048_576_000; // 1 GB

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DownloadRecord {
    #[pyo3(get, set)]
    pub id: String,
    #[pyo3(get, set)]
    pub anilist_id: i64,
    #[pyo3(get, set)]
    pub episode_num: f64,
    #[pyo3(get, set)]
    pub episode_title: Option<String>,
    #[pyo3(get, set)]
    pub series_title: String,
    #[pyo3(get, set)]
    pub series_cover: Option<String>,
    #[pyo3(get, set)]
    pub provider: String,
    #[pyo3(get, set)]
    pub category: String,
    #[pyo3(get, set)]
    pub quality: String,
    #[pyo3(get, set)]
    pub status: String,
    #[pyo3(get, set)]
    pub progress: f64,
    #[pyo3(get, set)]
    pub file_path: Option<String>,
    #[pyo3(get, set)]
    pub file_size: Option<i64>,
    #[pyo3(get, set)]
    pub duration_s: Option<f64>,
    #[pyo3(get, set)]
    pub error_msg: Option<String>,
    #[pyo3(get, set)]
    pub created_at: i64,
    #[pyo3(get, set)]
    pub updated_at: i64,
}

#[pymethods]
impl DownloadRecord {
    #[new]
    #[pyo3(signature = (
        id, anilist_id, episode_num, series_title, provider, category, quality,
        episode_title=None, series_cover=None, status="QUEUED".to_string(), progress=0.0,
        file_path=None, file_size=None, duration_s=None, error_msg=None, created_at=0, updated_at=0
    ))]
    pub fn new(
        id: String,
        anilist_id: i64,
        episode_num: f64,
        series_title: String,
        provider: String,
        category: String,
        quality: String,
        episode_title: Option<String>,
        series_cover: Option<String>,
        status: String,
        progress: f64,
        file_path: Option<String>,
        file_size: Option<i64>,
        duration_s: Option<f64>,
        error_msg: Option<String>,
        created_at: i64,
        updated_at: i64,
    ) -> Self {
        Self {
            id,
            anilist_id,
            episode_num,
            episode_title,
            series_title,
            series_cover,
            provider,
            category,
            quality,
            status,
            progress,
            file_path,
            file_size,
            duration_s,
            error_msg,
            created_at,
            updated_at,
        }
    }
}
