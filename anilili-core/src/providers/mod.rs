pub mod anibd;
pub mod anidbapp;
pub mod animegg;
pub mod animekai;
pub mod anikoto;
pub mod animeshqip;
pub mod anizone;
pub mod kickassanime;
pub mod manager;
pub mod rareanimes;
pub mod senshi;

pub use anibd::*;
pub use anidbapp::*;
pub use animegg::*;
pub use animekai::*;
pub use anikoto::*;
pub use animeshqip::*;
pub use anizone::*;
pub use kickassanime::*;
pub use manager::*;
pub use rareanimes::*;
pub use senshi::*;


use crate::error::AppError;
use crate::models::{Category, EpisodeItem, ProviderData, SourcesResult};
use async_trait::async_trait;

#[async_trait]
pub trait AnimeProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn is_fast(&self) -> bool {
        true
    }
    fn supports_sub(&self) -> bool {
        true
    }
    fn supports_dub(&self) -> bool {
        false
    }
    fn supports_download(&self) -> bool {
        false
    }
    fn is_adult_only(&self) -> bool {
        false
    }

    async fn get_episodes(
        &self,
        anilist_id: i64,
        mal_id: Option<i64>,
        title_romaji: Option<&str>,
    ) -> Result<ProviderData, AppError>;

    async fn get_sources(
        &self,
        episode: &EpisodeItem,
        category: Category,
    ) -> Result<SourcesResult, AppError>;
}
