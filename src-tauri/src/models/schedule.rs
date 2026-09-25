use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AiringEntry {
    pub id: i64,
    pub airing_at: i64,
    pub episode: i32,
    pub media_id: i64,
    pub media_title: String,
    pub cover_image: Option<String>,
    pub banner_image: Option<String>,
    pub format: Option<String>,
    pub duration: Option<i32>,
    pub average_score: Option<i32>,
}

impl AiringEntry {
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct NotificationPreference {
    pub media_id: i64,
    pub enabled: bool,
    pub media_title: String,
    pub cover_image: Option<String>,
}

impl NotificationPreference {
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
