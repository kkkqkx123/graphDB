//! Cache Module
//!
//! Provide search result caching function to improve query performance

use crate::error::Result;
use crate::r#type::{SearchOptions, SearchResults};
use lru::LruCache;
use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::RwLock;

const DEFAULT_SHARD_COUNT: usize = 16;

/// Cache Key Generator
pub struct CacheKeyGenerator;

impl CacheKeyGenerator {
    /// Generate search cache keys
    pub fn generate_search_key(query: &str, options: &SearchOptions) -> String {
        let mut key_parts = vec![query.to_lowercase()];

        key_parts.push(format!("limit:{}", options.limit.unwrap_or(100)));
        key_parts.push(format!("offset:{}", options.offset.unwrap_or(0)));
        key_parts.push(format!("context:{}", options.context.is_some()));
        key_parts.push(format!("resolve:{}", options.resolve.unwrap_or(true)));
        key_parts.push(format!("suggest:{}", options.suggest.unwrap_or(false)));

        if let Some(resolution) = options.resolution {
            key_parts.push(format!("resolution:{}", resolution));
        }
        if let Some(boost) = options.boost {
            key_parts.push(format!("boost:{}", boost));
        }

        key_parts.join("|")
    }

    /// Generate Document Cache Keys
    pub fn generate_document_key(doc_id: u64) -> String {
        format!("doc:{}", doc_id)
    }
}

/// cache entry
#[derive(Clone)]
struct CacheEntry {
    data: SearchResults,
    created_at: Instant,
    access_count: u64,
}

struct Shard {
    store: RwLock<LruCache<String, CacheEntry>>,
}

/// Search cache with shard-based lock optimization
#[derive(Clone)]
pub struct SearchCache {
    shards: Arc<Vec<Shard>>,
    shard_mask: usize,
    default_ttl: Option<Duration>,
    max_size: usize,
    total_size: Arc<AtomicUsize>,
    hit_count: Arc<AtomicU64>,
    miss_count: Arc<AtomicU64>,
}

impl SearchCache {
    /// Creating a new search cache with sharding
    pub fn new(max_size: usize, default_ttl: Option<Duration>) -> Self {
        let shard_count = DEFAULT_SHARD_COUNT;
        let cap = NonZeroUsize::new(max_size.max(1)).expect("Shard capacity should be valid");

        let mut shards = Vec::with_capacity(shard_count);
        for _ in 0..shard_count {
            shards.push(Shard {
                store: RwLock::new(LruCache::new(cap)),
            });
        }

        Self {
            shards: Arc::new(shards),
            shard_mask: shard_count - 1,
            default_ttl,
            max_size,
            total_size: Arc::new(AtomicUsize::new(0)),
            hit_count: Arc::new(AtomicU64::new(0)),
            miss_count: Arc::new(AtomicU64::new(0)),
        }
    }

    fn shard_index(&self, key: &str) -> usize {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish() as usize & self.shard_mask
    }

    /// Asynchronous fetching of cache entries
    pub async fn get_async(&self, key: &str) -> Option<SearchResults> {
        let idx = self.shard_index(key);
        let mut store = self.shards[idx].store.write();

        if let Some(entry) = store.get_mut(key) {
            if let Some(ttl) = self.default_ttl {
                if entry.created_at.elapsed() > ttl {
                    store.pop(key);
                    self.miss_count.fetch_add(1, Ordering::Relaxed);
                    return None;
                }
            }

            entry.access_count += 1;
            self.hit_count.fetch_add(1, Ordering::Relaxed);
            Some(entry.data.clone())
        } else {
            self.miss_count.fetch_add(1, Ordering::Relaxed);
            None
        }
    }

    /// Setting cache items asynchronously
    pub async fn set_async(&self, key: String, data: SearchResults) {
        let idx = self.shard_index(&key);
        let mut store = self.shards[idx].store.write();
        let is_new = !store.contains(&key);
        let entry = CacheEntry {
            data,
            created_at: Instant::now(),
            access_count: 1,
        };
        store.put(key, entry);
        if is_new {
            let prev = self.total_size.fetch_add(1, Ordering::Relaxed);
            if prev + 1 > self.max_size {
                drop(store);
                self.evict_one_async().await;
            }
        }
    }

    async fn evict_one_async(&self) {
        let mut max_shard_idx = 0;
        let mut max_size = 0;
        for (i, shard) in self.shards.iter().enumerate() {
            let store = shard.store.read();
            let len = store.len();
            if len > max_size {
                max_size = len;
                max_shard_idx = i;
            }
        }
        if max_size > 0 {
            let mut store = self.shards[max_shard_idx].store.write().unwrap();
            if store.pop_lru().is_some() {
                self.total_size.fetch_sub(1, Ordering::Relaxed);
            }
        }
    }

    /// Deleting cache entries asynchronously
    pub async fn remove_async(&self, key: &str) -> bool {
        let idx = self.shard_index(key);
        let mut store = self.shards[idx].store.write().unwrap();
        let removed = store.pop(key).is_some();
        if removed {
            self.total_size.fetch_sub(1, Ordering::Relaxed);
        }
        removed
    }

    /// Asynchronous Cache Emptying
    pub async fn clear_async(&self) {
        for shard in self.shards.iter() {
            let mut store = shard.store.write().unwrap();
            store.clear();
        }
        self.total_size.store(0, Ordering::Relaxed);
        self.hit_count.store(0, Ordering::Relaxed);
        self.miss_count.store(0, Ordering::Relaxed);
    }

    /// Synchronized fetch of cache entries (for backward compatibility, may block)
    pub fn get(&self, key: &str) -> Option<SearchResults> {
        let idx = self.shard_index(key);
        let mut store = self.shards[idx].store.write().unwrap();
        if let Some(entry) = store.get_mut(key) {
            if let Some(ttl) = self.default_ttl {
                if entry.created_at.elapsed() > ttl {
                    store.pop(key);
                    self.miss_count.fetch_add(1, Ordering::Relaxed);
                    return None;
                }
            }

            entry.access_count += 1;
            self.hit_count.fetch_add(1, Ordering::Relaxed);
            Some(entry.data.clone())
        } else {
            self.miss_count.fetch_add(1, Ordering::Relaxed);
            None
        }
    }

    /// Synchronized setup of cache entries (for backward compatibility, possible blocking)
    pub fn set(&self, key: String, data: SearchResults) {
        let idx = self.shard_index(&key);
        let mut store = self.shards[idx].store.write().unwrap();
        let is_new = !store.contains(&key);
        let entry = CacheEntry {
            data,
            created_at: Instant::now(),
            access_count: 1,
        };
        store.put(key, entry);
        if is_new {
            let prev = self.total_size.fetch_add(1, Ordering::Relaxed);
            if prev + 1 > self.max_size {
                drop(store);
                self.evict_one();
            }
        }
    }

    fn evict_one(&self) {
        let mut max_shard_idx = 0;
        let mut max_size = 0;
        for (i, shard) in self.shards.iter().enumerate() {
            let store = shard.store.read().unwrap();
            let len = store.len();
            if len > max_size {
                max_size = len;
                max_shard_idx = i;
            }
        }
        if max_size > 0 {
            let mut store = self.shards[max_shard_idx].store.write().unwrap();
            if store.pop_lru().is_some() {
                self.total_size.fetch_sub(1, Ordering::Relaxed);
            }
        }
    }

    /// Synchronized deletion of cache entries (for backward compatibility, may block)
    pub fn remove(&self, key: &str) -> bool {
        let idx = self.shard_index(key);
        let mut store = self.shards[idx].store.write().unwrap();
        let removed = store.pop(key).is_some();
        if removed {
            self.total_size.fetch_sub(1, Ordering::Relaxed);
        }
        removed
    }

    /// Synchronized cache clearing (for backward compatibility, may block)
    pub fn clear(&self) {
        for shard in self.shards.iter() {
            let mut store = shard.store.write().unwrap();
            store.clear();
        }
        self.total_size.store(0, Ordering::Relaxed);
        self.hit_count.store(0, Ordering::Relaxed);
        self.miss_count.store(0, Ordering::Relaxed);
    }

    /// Getting Cache Statistics
    pub fn stats(&self) -> CacheStats {
        let hit_count = self.hit_count.load(Ordering::Relaxed);
        let miss_count = self.miss_count.load(Ordering::Relaxed);
        let total_requests = hit_count + miss_count;
        let hit_rate = if total_requests > 0 {
            hit_count as f64 / total_requests as f64
        } else {
            0.0
        };

        let mut size = 0;
        for shard in self.shards.iter() {
            if let Ok(store) = shard.store.try_read() {
                size += store.len();
            }
        }

        CacheStats {
            size,
            max_size: self.max_size,
            hit_count,
            miss_count,
            hit_rate,
            total_requests,
        }
    }

    /// Asynchronous fetching of cached statistics
    pub async fn stats_async(&self) -> CacheStats {
        let hit_count = self.hit_count.load(Ordering::Relaxed);
        let miss_count = self.miss_count.load(Ordering::Relaxed);
        let total_requests = hit_count + miss_count;
        let hit_rate = if total_requests > 0 {
            hit_count as f64 / total_requests as f64
        } else {
            0.0
        };

        let mut size = 0;
        for shard in self.shards.iter() {
            let store = shard.store.read().unwrap();
            size += store.len();
        }

        CacheStats {
            size,
            max_size: self.max_size,
            hit_count,
            miss_count,
            hit_rate,
            total_requests,
        }
    }

    /// Check if the cache contains keys
    pub fn contains(&self, key: &str) -> bool {
        let idx = self.shard_index(key);
        if let Ok(store) = self.shards[idx].store.try_read() {
            store.contains(key)
        } else {
            false
        }
    }

    /// Asynchronously check if the cache contains keys
    pub async fn contains_async(&self, key: &str) -> bool {
        let idx = self.shard_index(key);
        let store = self.shards[idx].store.read().unwrap();
        store.contains(key)
    }

    /// Get current cache size
    pub fn len(&self) -> usize {
        self.total_size.load(Ordering::Relaxed)
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Cache Statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub size: usize,
    pub max_size: usize,
    pub hit_count: u64,
    pub miss_count: u64,
    pub hit_rate: f64,
    pub total_requests: u64,
}

/// Search wrapper with cache (asynchronous version)
pub struct CachedSearch<F>
where
    F: Fn(&str, &SearchOptions) -> Result<SearchResults> + Send + Sync,
{
    cache: SearchCache,
    search_fn: Arc<F>,
}

impl<F> CachedSearch<F>
where
    F: Fn(&str, &SearchOptions) -> Result<SearchResults> + Send + Sync,
{
    /// Creating a new cache search wrapper
    pub fn new(search_fn: F, cache_size: usize, ttl: Option<Duration>) -> Self {
        Self {
            cache: SearchCache::new(cache_size, ttl),
            search_fn: Arc::new(search_fn),
        }
    }

    /// Perform search with cache (synchronized version)
    pub fn search(&self, query: &str, options: &SearchOptions) -> Result<SearchResults> {
        let cache_key = CacheKeyGenerator::generate_search_key(query, options);

        if let Some(cached_results) = self.cache.get(&cache_key) {
            return Ok(cached_results);
        }

        let results = (self.search_fn)(query, options)?;

        self.cache.set(cache_key, results.clone());

        Ok(results)
    }

    /// Perform search with cache (asynchronous version)
    pub async fn search_async(
        &self,
        query: &str,
        options: &SearchOptions,
    ) -> Result<SearchResults> {
        let cache_key = CacheKeyGenerator::generate_search_key(query, options);

        if let Some(cached_results) = self.cache.get_async(&cache_key).await {
            return Ok(cached_results);
        }

        let results = (self.search_fn)(query, options)?;

        self.cache.set_async(cache_key, results.clone()).await;

        Ok(results)
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        self.cache.stats()
    }

    /// Asynchronous fetching of cached statistics
    pub async fn cache_stats_async(&self) -> CacheStats {
        self.cache.stats_async().await
    }

    /// Clear cache
    pub fn clear_cache(&self) {
        self.cache.clear();
    }

    /// Asynchronous Cache Emptying
    pub async fn clear_cache_async(&self) {
        self.cache.clear_async().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_generation() {
        let options = SearchOptions {
            query: Some("hello world".to_string()),
            limit: Some(50),
            offset: Some(10),
            ..Default::default()
        };

        let key = CacheKeyGenerator::generate_search_key("Hello World", &options);
        assert!(key.contains("hello world"));
        assert!(key.contains("limit:50"));
        assert!(key.contains("offset:10"));
    }

    #[test]
    fn test_search_cache_basic() {
        let cache = SearchCache::new(100, None);
        let results = vec![1, 2, 3, 4, 5];

        cache.set("test_key".to_string(), results.clone());

        let cached = cache.get("test_key");
        assert_eq!(cached, Some(results));
    }

    #[test]
    fn test_search_cache_miss() {
        let cache = SearchCache::new(100, None);

        let cached = cache.get("nonexistent_key");
        assert!(cached.is_none());

        let stats = cache.stats();
        assert_eq!(stats.miss_count, 1);
    }

    #[test]
    fn test_search_cache_ttl() {
        let cache = SearchCache::new(100, Some(Duration::from_millis(10)));
        let results = vec![1, 2, 3];

        cache.set("test_key".to_string(), results.clone());

        std::thread::sleep(Duration::from_millis(20));

        let cached = cache.get("test_key");
        assert!(cached.is_none());
    }

    #[test]
    fn test_search_cache_eviction() {
        let cache = SearchCache::new(3, None);

        cache.set("key1".to_string(), vec![1]);
        cache.set("key2".to_string(), vec![2]);
        cache.set("key3".to_string(), vec![3]);
        cache.set("key4".to_string(), vec![4]);

        let len = cache.len();
        assert!(len <= 3, "Cache should not exceed max_size, got {}", len);
    }

    #[tokio::test]
    async fn test_search_cache_async() {
        let cache = SearchCache::new(100, None);
        let results = vec![1, 2, 3, 4, 5];

        cache
            .set_async("test_key".to_string(), results.clone())
            .await;

        let cached = cache.get_async("test_key").await;
        assert_eq!(cached, Some(results));
    }

    #[tokio::test]
    async fn test_search_cache_stats_async() {
        let cache = SearchCache::new(100, None);

        cache.set_async("key1".to_string(), vec![1]).await;
        cache.get_async("key1").await;
        cache.get_async("nonexistent").await;

        let stats = cache.stats_async().await;
        assert_eq!(stats.hit_count, 1);
        assert_eq!(stats.miss_count, 1);
    }

    #[test]
    fn test_shard_distribution() {
        let cache = SearchCache::new(100, None);
        let keys: Vec<String> = (0..100).map(|i| format!("key_{}", i)).collect();

        for key in &keys {
            cache.set(key.clone(), vec![1]);
        }

        assert_eq!(cache.len(), 100);

        for key in &keys {
            let cached = cache.get(key);
            assert!(cached.is_some(), "Key {} should be in cache", key);
        }
    }
}