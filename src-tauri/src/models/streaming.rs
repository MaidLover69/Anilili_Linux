use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Category {
    #[serde(rename = "sub")]
    Sub,
    #[serde(rename = "dub")]
    Dub,
}

impl Category {
    pub fn as_str(&self) -> &'static str {
        match self {
            Category::Sub => "sub",
            Category::Dub => "dub",
        }
    }
}

impl From<&str> for Category {
    fn from(s: &str) -> Self {
        if s.eq_ignore_ascii_case("dub") {
            Category::Dub
        } else {
            Category::Sub
        }
    }
}

impl From<String> for Category {
    fn from(s: String) -> Self {
        Category::from(s.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EpisodeItem {
    pub pipe_id: String,
    pub number: f64,
    pub title: Option<String>,
    pub image: Option<String>,
    pub synopsis: Option<String>,
    pub filler: bool,
}

impl EpisodeItem {
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
            synopsis: None,
            filler,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ProviderData {
    pub name: String,
    pub sub: Vec<EpisodeItem>,
    pub dub: Vec<EpisodeItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct StreamItem {
    pub url: String,
    pub stream_type: String, // "hls", "mp4", "dash"
    pub quality: Option<String>,
    pub audio: Option<String>,
    #[serde(default)]
    pub subtitle_variant: Option<String>,
    pub referer: Option<String>,
    pub origin: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub is_active: bool,
}

impl StreamItem {
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
            subtitle_variant: None,
            referer,
            origin: None,
            headers: None,
            is_active,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SubtitleItem {
    pub url: String,
    pub language: String,
    pub format: String,
    pub is_default: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SkipTimes {
    pub intro_start: Option<f64>,
    pub intro_end: Option<f64>,
    pub outro_start: Option<f64>,
    pub outro_end: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SourcesResult {
    pub streams: Vec<StreamItem>,
    #[serde(default)]
    pub subtitles: Vec<SubtitleItem>,
    #[serde(default)]
    pub skip: Option<SkipTimes>,
}

impl SourcesResult {
    pub fn new(streams: Vec<StreamItem>) -> Self {
        Self {
            streams,
            subtitles: Vec::new(),
            skip: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ProviderEpisodes {
    pub sub: Vec<EpisodeItem>,
    pub dub: Vec<EpisodeItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct EpisodesResult {
    pub providers: HashMap<String, ProviderEpisodes>,
}
