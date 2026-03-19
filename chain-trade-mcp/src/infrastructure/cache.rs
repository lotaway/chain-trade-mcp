use moka::future::Cache;
use std::time::Duration;

#[derive(Clone)]
pub struct CacheService {
    cache: Cache<String, String>,
}

impl CacheService {
    pub fn new(ttl_seconds: u64) -> Self {
        let cache = Cache::builder()
            .time_to_live(Duration::from_secs(ttl_seconds))
            .build();
        
        Self { cache }
    }

    pub async fn get(&self, key: &str) -> Option<String> {
        self.cache.get(key).await
    }

    pub async fn insert(&self, key: String, value: String) {
        self.cache.insert(key, value).await;
    }
}
