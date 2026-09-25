use crate::clients::miruro::MiruroClient;
use crate::error::AppError;
use crate::models::{Category, EpisodeItem, ProviderData, SourcesResult};
use crate::providers::AnimeProvider;
use async_trait::async_trait;

pub struct MiruroProvider {
    client: MiruroClient,
    server_name: &'static str,
}

impl MiruroProvider {
    pub fn new(server_name: &'static str) -> Self {
        Self {
            client: MiruroClient::new(),
            server_name,
        }
    }
}

#[async_trait]
impl AnimeProvider for MiruroProvider {
    fn name(&self) -> &'static str {
        self.server_name
    }

    fn supports_dub(&self) -> bool {
        true
    }

    async fn get_episodes(
        &self,
        anilist_id: i64,
        _mal_id: Option<i64>,
        _title_romaji: Option<&str>,
    ) -> Result<ProviderData, AppError> {
        if anilist_id <= 0 {
            return Ok(ProviderData {
                name: self.name().to_string(),
                sub: vec![],
                dub: vec![],
            });
        }
        let mut sub = Vec::with_capacity(1000);
        let mut dub = Vec::with_capacity(1000);
        for n in 1..=2000 {
            let num = n as f64;
            let pipe_id = format!("{}-{}", anilist_id, n);
            sub.push(EpisodeItem::new(pipe_id.clone(), num, None, None, false));
            dub.push(EpisodeItem::new(pipe_id, num, None, None, false));
        }
        Ok(ProviderData {
            name: self.name().to_string(),
            sub,
            dub,
        })
    }

    async fn get_sources(
        &self,
        episode: &EpisodeItem,
        category: Category,
    ) -> Result<SourcesResult, AppError> {
        // Try extracting anilist_id from pipe_id formats: "12345-1", "anilist|12345|1", etc.
        let mut anilist_id: i64 = 0;
        if let Some((first, _)) = episode.pipe_id.split_once('-') {
            anilist_id = first.parse().unwrap_or(0);
        } else if let Some((first, _)) = episode.pipe_id.split_once('|') {
            anilist_id = first.parse().unwrap_or(0);
        }

        if anilist_id == 0 {
            return Err(AppError::NoSourcesFound);
        }

        self.client
            .get_sources(anilist_id, episode.number, self.server_name, &category)
            .await
    }
}
