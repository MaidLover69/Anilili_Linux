use moka::future::Cache;
use std::time::Duration;

pub struct L1Cache<K, V>
where
    K: Send + Sync + 'static + std::hash::Hash + Eq + Clone,
    V: Send + Sync + 'static + Clone,
{
    cache: Cache<K, V>,
}

impl<K, V> L1Cache<K, V>
where
    K: Send + Sync + 'static + std::hash::Hash + Eq + Clone,
    V: Send + Sync + 'static + Clone,
{
    pub fn new(max_capacity: u64, ttl_secs: u64) -> Self {
        let cache = Cache::builder()
            .max_capacity(max_capacity)
            .time_to_live(Duration::from_secs(ttl_secs))
            .build();
        Self { cache }
    }

    pub async fn get(&self, key: &K) -> Option<V> {
        self.cache.get(key).await
    }

    pub async fn insert(&self, key: K, value: V) {
        self.cache.insert(key, value).await;
    }

    pub async fn invalidate(&self, key: &K) {
        self.cache.invalidate(key).await;
    }
}
