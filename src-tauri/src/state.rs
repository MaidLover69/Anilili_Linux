use crate::cache::L1Cache;
use crate::clients::anilist::AniListClient;
use crate::clients::http::build_http_client;
use crate::clients::imdb::ImdbClient;
use crate::clients::mal::MalClient;
use crate::providers::manager::ProviderManager;
use sqlx::SqlitePool;
use std::sync::Arc;

pub struct AppState {
    pub pool: SqlitePool,
    pub client: reqwest::Client,
    pub anilist_client: AniListClient,
    pub mal_client: MalClient,
    pub imdb_client: ImdbClient,
    pub provider_manager: ProviderManager,
    pub l1_cache: Arc<L1Cache<String, serde_json::Value>>,
}

impl AppState {
    pub fn new(pool: SqlitePool) -> Self {
        let client = build_http_client();
        let anilist_client = AniListClient::new(client.clone());
        let mal_client = MalClient::new(client.clone());
        let imdb_client = ImdbClient::new(client.clone());
        let provider_manager = ProviderManager::new();
        let l1_cache = Arc::new(L1Cache::new(500, 3600));

        Self {
            pool,
            client,
            anilist_client,
            mal_client,
            imdb_client,
            provider_manager,
            l1_cache,
        }
    }
}
