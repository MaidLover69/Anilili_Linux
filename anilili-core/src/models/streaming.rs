use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

#[pyclass(eq, eq_int)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Category {
    Sub,
    Dub,
}

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EpisodeItem {
    #[pyo3(get, set)]
    pub pipe_id: String,
    #[pyo3(get, set)]
    pub number: f64,
    #[pyo3(get, set)]
    pub title: Option<String>,
    #[pyo3(get, set)]
    pub image: Option<String>,
    #[pyo3(get, set)]
    pub filler: bool,
}

#[pymethods]
impl EpisodeItem {
    #[new]
    #[pyo3(signature = (pipe_id, number, title=None, image=None, filler=false))]
    pub fn new(
        pipe_id: String,
        number: f64,
        title: Option<String>,
        image: Option<String>,
        filler: bool,
    ) -> Self {
        Self {
            pipe_id,
            number,
            title,
            image,
            filler,
        }
    }
}

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProviderData {
    #[pyo3(get, set)]
    pub name: String,
    #[pyo3(get, set)]
    pub sub: Vec<EpisodeItem>,
    #[pyo3(get, set)]
    pub dub: Vec<EpisodeItem>,
}

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StreamItem {
    #[pyo3(get, set)]
    pub url: String,
    #[pyo3(get, set)]
    pub stream_type: String, // "hls", "mp4", "dash"
    #[pyo3(get, set)]
    pub quality: Option<String>,
    #[pyo3(get, set)]
    pub audio: Option<String>,
    #[pyo3(get, set)]
    pub referer: Option<String>,
    #[pyo3(get, set)]
    pub is_active: bool,
}

#[pymethods]
impl StreamItem {
    #[new]
    #[pyo3(signature = (url, stream_type="hls".to_string(), quality=None, audio=None, referer=None, is_active=true))]
    pub fn new(
        url: String,
        stream_type: String,
        quality: Option<String>,
        audio: Option<String>,
        referer: Option<String>,
        is_active: bool,
    ) -> Self {
        Self {
            url,
            stream_type,
            quality,
            audio,
            referer,
            is_active,
        }
    }
}

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SourcesResult {
    #[pyo3(get, set)]
    pub streams: Vec<StreamItem>,
}

#[pymethods]
impl SourcesResult {
    #[new]
    pub fn new(streams: Vec<StreamItem>) -> Self {
        Self { streams }
    }
}
