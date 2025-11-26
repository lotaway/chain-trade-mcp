#[cfg(test)]
mod cache_tests {
    use chain_trade_mcp::infrastructure::cache::CacheService;
    use tokio;

    #[tokio::test]
    async fn test_cache_insert_and_get() {
        let cache = CacheService::new(60);

        cache.insert("key1".to_string(), "value1".to_string()).await;

        let result = cache.get("key1").await;
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "value1");
    }

    #[tokio::test]
    async fn test_cache_get_nonexistent() {
        let cache = CacheService::new(60);

        let result = cache.get("nonexistent").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_cache_multiple_entries() {
        let cache = CacheService::new(60);

        cache.insert("key1".to_string(), "value1".to_string()).await;
        cache.insert("key2".to_string(), "value2".to_string()).await;
        cache.insert("key3".to_string(), "value3".to_string()).await;

        assert_eq!(cache.get("key1").await, Some("value1".to_string()));
        assert_eq!(cache.get("key2").await, Some("value2".to_string()));
        assert_eq!(cache.get("key3").await, Some("value3".to_string()));
    }

    #[tokio::test]
    async fn test_cache_overwrite() {
        let cache = CacheService::new(60);

        cache.insert("key1".to_string(), "value1".to_string()).await;
        assert_eq!(cache.get("key1").await, Some("value1".to_string()));

        cache.insert("key1".to_string(), "value2".to_string()).await;
        assert_eq!(cache.get("key1").await, Some("value2".to_string()));
    }

    #[tokio::test]
    async fn test_cache_ttl_expiration() {
        // Create cache with 1 second TTL
        let cache = CacheService::new(1);

        cache.insert("key1".to_string(), "value1".to_string()).await;

        // Should exist immediately
        assert!(cache.get("key1").await.is_some());

        // Wait for expiration
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Should be expired
        assert!(cache.get("key1").await.is_none());
    }

    #[tokio::test]
    async fn test_cache_clone() {
        let cache1 = CacheService::new(60);
        cache1
            .insert("key1".to_string(), "value1".to_string())
            .await;

        let cache2 = cache1.clone();

        // Both should access the same cache
        assert_eq!(cache2.get("key1").await, Some("value1".to_string()));

        cache2
            .insert("key2".to_string(), "value2".to_string())
            .await;
        assert_eq!(cache1.get("key2").await, Some("value2".to_string()));
    }
}
