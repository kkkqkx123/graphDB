use std::time::Duration;
use moka::future::Cache;
use crate::search::result::SearchResult;

/// Search result cache
pub struct SearchCache {
    cache: Cache<String, Vec<SearchResult>>,
}

impl SearchCache {
    pub fn new(max_capacity: u64, ttl_secs: u64) -> Self {
        Self {
            cache: Cache::builder()
                .max_capacity(max_capacity)
                .time_to_live(Duration::from_secs(ttl_secs))
                .build(),
        }
    }

    /// Generate cache key
    fn cache_key(space_id: u64, tag: &str, field: &str, query: &str, limit: usize) -> String {
        format!("{}:{}:{}:{}:{}", space_id, tag, field, query, limit)
    }

    /// Get cached results
    pub async fn get(&self, space_id: u64, tag: &str, field: &str, query: &str, limit: usize)
        -> Option<Vec<SearchResult>> {
        let key = Self::cache_key(space_id, tag, field, query, limit);
        self.cache.get(&key).await
    }

    /// Cache results
    pub async fn set(&self, space_id: u64, tag: &str, field: &str, query: &str, limit: usize, results: Vec<SearchResult>) {
        let key = Self::cache_key(space_id, tag, field, query, limit);
        self.cache.insert(key, results).await;
    }

    /// Invalidate cache
    pub async fn invalidate(&self, space_id: u64, tag: &str, field: &str) {
        // Use prefix matching to clear related cache
        let prefix = format!("{}:{}:{}", space_id, tag, field);
        let _ = self.cache.invalidate_entries_if(move |key: &String, _| key.starts_with(&prefix));
    }

    /// Invalidate all cache
    pub async fn invalidate_all(&self) {
        self.cache.invalidate_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;

    fn create_test_result(doc_id: &str, score: f32) -> SearchResult {
        SearchResult {
            doc_id: Value::String(doc_id.to_string()),
            score,
            highlights: None,
            matched_fields: vec!["content".to_string()],
        }
    }

    #[tokio::test]
    async fn test_cache_basic_operations() {
        let cache = SearchCache::new(100, 60);

        // Initially empty
        let result = cache.get(1, "Article", "title", "test", 10).await;
        assert!(result.is_none());

        // Set cache
        let results = vec![
            create_test_result("1", 1.0),
            create_test_result("2", 0.8),
        ];
        cache.set(1, "Article", "title", "test", 10, results.clone()).await;

        // Get cache
        let cached = cache.get(1, "Article", "title", "test", 10).await;
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_cache_invalidation() {
        let cache = SearchCache::new(100, 60);

        // Set multiple cache entries
        cache.set(1, "Article", "title", "test1", 10, vec![create_test_result("1", 1.0)]).await;
        cache.set(1, "Article", "title", "test2", 10, vec![create_test_result("2", 0.9)]).await;
        cache.set(1, "Article", "content", "test3", 10, vec![create_test_result("3", 0.8)]).await;

        // Invalidate specific field
        cache.invalidate(1, "Article", "title").await;

        // title entries should be invalidated
        assert!(cache.get(1, "Article", "title", "test1", 10).await.is_none());
        assert!(cache.get(1, "Article", "title", "test2", 10).await.is_none());

        // content entry should still exist
        assert!(cache.get(1, "Article", "content", "test3", 10).await.is_some());
    }

    #[tokio::test]
    async fn test_cache_different_params() {
        let cache = SearchCache::new(100, 60);

        // Same query with different limits should have different cache keys
        cache.set(1, "Article", "title", "test", 10, vec![create_test_result("1", 1.0)]).await;
        cache.set(1, "Article", "title", "test", 20, vec![
            create_test_result("1", 1.0),
            create_test_result("2", 0.9),
        ]).await;

        let result1 = cache.get(1, "Article", "title", "test", 10).await;
        let result2 = cache.get(1, "Article", "title", "test", 20).await;

        assert_eq!(result1.unwrap().len(), 1);
        assert_eq!(result2.unwrap().len(), 2);
    }
}
