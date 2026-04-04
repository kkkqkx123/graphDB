use lru::LruCache;
use parking_lot::Mutex;
use std::num::NonZeroUsize;
use std::sync::Arc;

use crate::search::engine::SearchEngine;
use crate::search::metadata::IndexKey;

/// Index cache manager with LRU eviction
pub struct IndexCache {
    /// LRU cache
    cache: Mutex<LruCache<IndexKey, Arc<dyn SearchEngine>>>,
    /// Maximum number of cached indexes
    max_indexes: usize,
}

impl IndexCache {
    pub fn new(max_indexes: usize) -> Self {
        let cache_size = NonZeroUsize::new(max_indexes.max(1)).expect("Cache size must be > 0");
        Self {
            cache: Mutex::new(LruCache::new(cache_size)),
            max_indexes,
        }
    }

    /// Get index from cache (with cache hit tracking)
    pub fn get(&self, key: &IndexKey) -> Option<Arc<dyn SearchEngine>> {
        let mut cache = self.cache.lock();
        cache.get(key).cloned()
    }

    /// Insert index into cache
    pub fn put(&self, key: IndexKey, engine: Arc<dyn SearchEngine>) {
        let mut cache = self.cache.lock();

        // If cache is full and key doesn't exist, close the least recently used engine
        if cache.len() >= self.max_indexes && !cache.contains(&key) {
            if let Some((_, lru_engine)) = cache.pop_lru() {
                // Close engine asynchronously
                tokio::spawn(async move {
                    let _ = lru_engine.close().await;
                });
            }
        }

        cache.put(key, engine);
    }

    /// Remove index from cache
    pub fn remove(&self, key: &IndexKey) -> Option<Arc<dyn SearchEngine>> {
        let mut cache = self.cache.lock();
        cache.pop(key)
    }

    /// Check if cache contains key
    pub fn contains(&self, key: &IndexKey) -> bool {
        let cache = self.cache.lock();
        cache.contains(key)
    }

    /// Get cache size
    pub fn len(&self) -> usize {
        let cache = self.cache.lock();
        cache.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        let cache = self.cache.lock();
        cache.is_empty()
    }

    /// Clear all cached indexes
    pub fn clear(&self) {
        let mut cache = self.cache.lock();
        // Close all engines
        while let Some((_, engine)) = cache.pop_lru() {
            tokio::spawn(async move {
                let _ = engine.close().await;
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::adapters::Bm25SearchEngine;
    use tempfile::TempDir;

    #[test]
    fn test_cache_basic_operations() {
        let cache = IndexCache::new(3);
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let key1 = IndexKey::new(1, "Article", "title");
        let key2 = IndexKey::new(1, "Article", "content");

        // Initially empty
        assert!(cache.get(&key1).is_none());
        assert_eq!(cache.len(), 0);

        // Insert engines
        let engine1 = Arc::new(
            Bm25SearchEngine::open_or_create(temp_dir.path()).expect("Failed to create engine"),
        );
        let engine2 = Arc::new(
            Bm25SearchEngine::open_or_create(temp_dir.path()).expect("Failed to create engine"),
        );

        cache.put(key1.clone(), engine1);
        cache.put(key2.clone(), engine2);

        // Check cache
        assert_eq!(cache.len(), 2);
        assert!(cache.contains(&key1));
        assert!(cache.contains(&key2));
        assert!(cache.get(&key1).is_some());
    }

    #[test]
    fn test_cache_lru_eviction() {
        let cache = IndexCache::new(2);
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let key1 = IndexKey::new(1, "Article", "title");
        let key2 = IndexKey::new(1, "Article", "content");
        let key3 = IndexKey::new(1, "Post", "title");

        // Insert 3 engines (max is 2)
        let engine1 = Arc::new(
            Bm25SearchEngine::open_or_create(temp_dir.path()).expect("Failed to create engine"),
        );
        let engine2 = Arc::new(
            Bm25SearchEngine::open_or_create(temp_dir.path()).expect("Failed to create engine"),
        );
        let engine3 = Arc::new(
            Bm25SearchEngine::open_or_create(temp_dir.path()).expect("Failed to create engine"),
        );

        cache.put(key1.clone(), engine1);
        cache.put(key2.clone(), engine2);

        // Access key1 to make it more recent
        let _ = cache.get(&key1);

        // Insert key3, should evict key2 (least recently used)
        cache.put(key3.clone(), engine3);

        assert_eq!(cache.len(), 2);
        assert!(cache.contains(&key1)); // Still exists (recently accessed)
        assert!(!cache.contains(&key2)); // Evicted
        assert!(cache.contains(&key3)); // New entry
    }

    #[test]
    fn test_cache_remove() {
        let cache = IndexCache::new(3);
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let key1 = IndexKey::new(1, "Article", "title");
        let engine = Arc::new(
            Bm25SearchEngine::open_or_create(temp_dir.path()).expect("Failed to create engine"),
        );

        cache.put(key1.clone(), engine);
        assert_eq!(cache.len(), 1);

        let removed = cache.remove(&key1);
        assert!(removed.is_some());
        assert_eq!(cache.len(), 0);
        assert!(!cache.contains(&key1));
    }

    #[test]
    fn test_cache_clear() {
        let cache = IndexCache::new(3);
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let key1 = IndexKey::new(1, "Article", "title");
        let key2 = IndexKey::new(1, "Article", "content");

        let engine1 = Arc::new(
            Bm25SearchEngine::open_or_create(temp_dir.path()).expect("Failed to create engine"),
        );
        let engine2 = Arc::new(
            Bm25SearchEngine::open_or_create(temp_dir.path()).expect("Failed to create engine"),
        );

        cache.put(key1.clone(), engine1);
        cache.put(key2.clone(), engine2);
        assert_eq!(cache.len(), 2);

        cache.clear();
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }
}
