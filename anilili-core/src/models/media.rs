use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MediaTitle {
    #[pyo3(get, set)]
    pub romaji: Option<String>,
    #[pyo3(get, set)]
    pub english: Option<String>,
    #[pyo3(get, set)]
    pub native: Option<String>,
    #[pyo3(get, set)]
    pub user_preferred: Option<String>,
}

#[pymethods]
impl MediaTitle {
    #[new]
    #[pyo3(signature = (romaji=None, english=None, native=None, user_preferred=None))]
    pub fn new(
        romaji: Option<String>,
        english: Option<String>,
        native: Option<String>,
        user_preferred: Option<String>,
    ) -> Self {
        Self {
            romaji,
            english,
            native,
            user_preferred,
        }
    }

    pub fn preferred(&self) -> String {
        if let Some(ref e) = self.english {
            if !e.is_empty() {
                return e.clone();
            }
        }
        if let Some(ref u) = self.user_preferred {
            if !u.is_empty() {
                return u.clone();
            }
        }
        if let Some(ref r) = self.romaji {
            if !r.is_empty() {
                return r.clone();
            }
        }
        self.native.clone().unwrap_or_default()
    }
}

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CoverImage {
    #[pyo3(get, set)]
    pub large: Option<String>,
    #[pyo3(get, set)]
    pub extra_large: Option<String>,
    #[pyo3(get, set)]
    pub color: Option<String>,
}

#[pymethods]
impl CoverImage {
    #[new]
    #[pyo3(signature = (large=None, extra_large=None, color=None))]
    pub fn new(large: Option<String>, extra_large: Option<String>, color: Option<String>) -> Self {
        Self {
            large,
            extra_large,
            color,
        }
    }
}

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FuzzyDate {
    #[pyo3(get, set)]
    pub year: Option<i32>,
    #[pyo3(get, set)]
    pub month: Option<i32>,
    #[pyo3(get, set)]
    pub day: Option<i32>,
}

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MediaTag {
    #[pyo3(get, set)]
    pub id: i32,
    #[pyo3(get, set)]
    pub name: String,
    #[pyo3(get, set)]
    pub is_general_spoiler: bool,
    #[pyo3(get, set)]
    pub is_media_spoiler: bool,
}

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Studio {
    #[pyo3(get, set)]
    pub id: i32,
    #[pyo3(get, set)]
    pub name: String,
    #[pyo3(get, set)]
    pub is_animation_studio: bool,
}

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NextAiringEpisode {
    #[pyo3(get, set)]
    pub id: i32,
    #[pyo3(get, set)]
    pub airing_at: i64,
    #[pyo3(get, set)]
    pub time_until_airing: i64,
    #[pyo3(get, set)]
    pub episode: i32,
}

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Media {
    #[pyo3(get, set)]
    pub id: i32,
    #[pyo3(get, set)]
    pub id_mal: Option<i32>,
    #[pyo3(get, set)]
    pub title: MediaTitle,
    #[pyo3(get, set)]
    pub cover_image: CoverImage,
    #[pyo3(get, set)]
    pub banner_image: Option<String>,
    #[pyo3(get, set)]
    pub description: Option<String>,
    #[pyo3(get, set)]
    pub format: Option<String>,
    #[pyo3(get, set)]
    pub season: Option<String>,
    #[pyo3(get, set)]
    pub season_year: Option<i32>,
    #[pyo3(get, set)]
    pub episodes: Option<i32>,
    #[pyo3(get, set)]
    pub duration: Option<i32>,
    #[pyo3(get, set)]
    pub status: Option<String>,
    #[pyo3(get, set)]
    pub average_score: Option<i32>,
    #[pyo3(get, set)]
    pub mean_score: Option<i32>,
    #[pyo3(get, set)]
    pub popularity: Option<i32>,
    #[pyo3(get, set)]
    pub favourites: Option<i32>,
    #[pyo3(get, set)]
    pub is_adult: bool,
    #[pyo3(get, set)]
    pub genres: Vec<String>,
}

#[pymethods]
impl Media {
    pub fn display_title(&self) -> String {
        self.title.preferred()
    }
}
