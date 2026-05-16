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

use tokio::sync::Mutex;
use tokio::sync::RwLock;

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
    eviction_lock: Arc<Mutex<()>>,
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
            eviction_lock: Arc::new(Mutex::new(())),
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
        let mut store = self.shards[idx].store.write().await;

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
        let mut store = self.shards[idx].store.write().await;
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
                let _lock = self.eviction_lock.lock().await;
                if self.total_size.load(Ordering::Relaxed) > self.max_size {
                    self.evict_one_async().await;
                }
            }
        }
    }

    async fn evict_one_async(&self) {
        let mut shard_sizes: Vec<(usize, usize)> = Vec::with_capacity(self.shards.len());
        for (i, shard) in self.shards.iter().enumerate() {
            let store = shard.store.read().await;
            shard_sizes.push((i, store.len()));
        }

        let target = shard_sizes.into_iter().max_by_key(|&(_, size)| size);
        if let Some((idx, max_size)) = target {
            if max_size > 0 {
                let mut store = self.shards[idx].store.write().await;
                if store.pop_lru().is_some() {
                    self.total_size.fetch_sub(1, Ordering::Relaxed);
                }
            }
        }
    }

    /// Deleting cache entries asynchronously
    pub async fn remove_async(&self, key: &str) -> bool {
        let idx = self.shard_index(key);
        let mut store = self.shards[idx].store.write().await;
        let removed = store.pop(key).is_some();
        if removed {
            self.total_size.fetch_sub(1, Ordering::Relaxed);
        }
        removed
    }

    /// Asynchronous Cache Emptying
    pub async fn clear_async(&self) {
        for shard in self.shards.iter() {
            let mut store = shard.store.write().await;
            store.clear();
        }
        self.total_size.store(0, Ordering::Relaxed);
        self.hit_count.store(0, Ordering::Relaxed);
        self.miss_count.store(0, Ordering::Relaxed);
    }

    /// Synchronized fetch of cache entries (for backward compatibility, may block)
    pub fn get(&self, key: &str) -> Option<SearchResults> {
        let idx = self.shard_index(key);
        let mut store = self.shards[idx].store.blocking_write();
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
        let mut store = self.shards[idx].store.blocking_write();
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
                if let Ok(_lock) = self.eviction_lock.try_lock() {
                    if self.total_size.load(Ordering::Relaxed) > self.max_size {
                        self.evict_one();
                    }
                }
            }
        }
    }

    fn evict_one(&self) {
        let mut shard_sizes: Vec<(usize, usize)> = Vec::with_capacity(self.shards.len());
        for (i, shard) in self.shards.iter().enumerate() {
            let store = shard.store.blocking_read();
            shard_sizes.push((i, store.len()));
        }

        let target = shard_sizes.into_iter().max_by_key(|&(_, size)| size);
        if let Some((idx, max_size)) = target {
            if max_size > 0 {
                let mut store = self.shards[idx].store.blocking_write();
                if store.pop_lru().is_some() {
                    self.total_size.fetch_sub(1, Ordering::Relaxed);
                }
            }
        }
    }

    /// Synchronized deletion of cache entries (for backward compatibility, may block)
    pub fn remove(&self, key: &str) -> bool {
        let idx = self.shard_index(key);
        let mut store = self.shards[idx].store.blocking_write();
        let removed = store.pop(key).is_some();
        if removed {
            self.total_size.fetch_sub(1, Ordering::Relaxed);
        }
        removed
    }

    /// Synchronized cache clearing (for backward compatibility, may block)
    pub fn clear(&self) {
        for shard in self.shards.iter() {
            let mut store = shard.store.blocking_write();
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
            let store = shard.store.read().await;
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
        let store = self.shards[idx].store.read().await;
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
    pub fn stats(&self) -> CacheStats {
        self.cache.stats()
    }

    /// Clear cache
    pub fn clear(&self) {
        self.cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_cache_basic() {
        let cache = SearchCache::new(100, None);

        cache.set("key1".to_string(), vec![1, 2, 3]);
        cache.set("key2".to_string(), vec![4, 5, 6]);

        assert_eq!(cache.get("key1"), Some(vec![1, 2, 3]));
        assert_eq!(cache.get("key2"), Some(vec![4, 5, 6]));
        assert_eq!(cache.get("nonexistent"), None);
    }

    #[test]
    fn test_search_cache_overwrite() {
        let cache = SearchCache::new(100, None);

        cache.set("key1".to_string(), vec![1]);
        cache.set("key1".to_string(), vec![2]);

        assert_eq!(cache.get("key1"), Some(vec![2]));
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
    fn test_cache_ttl() {
        let ttl = Some(Duration::from_millis(50));
        let cache = SearchCache::new(100, ttl);

        cache.set("key1".to_string(), vec![1]);
        assert_eq!(cache.get("key1"), Some(vec![1]));

        std::thread::sleep(Duration::from_millis(100));
        assert_eq!(cache.get("key1"), None);
    }

    #[test]
    fn test_cache_clear() {
        let cache = SearchCache::new(100, None);

        cache.set("key1".to_string(), vec![1]);
        cache.set("key2".to_string(), vec![2]);
        cache.clear();

        assert_eq!(cache.get("key1"), None);
        assert_eq!(cache.get("key2"), None);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_remove() {
        let cache = SearchCache::new(100, None);

        cache.set("key1".to_string(), vec![1]);
        assert!(cache.remove("key1"));
        assert_eq!(cache.get("key1"), None);
        assert!(!cache.remove("nonexistent"));
    }

    #[test]
    fn test_cache_stats() {
        let cache = SearchCache::new(100, None);

        cache.set("key1".to_string(), vec![1]);
        cache.get("key1");
        cache.get("nonexistent");

        let stats = cache.stats();
        assert_eq!(stats.hit_count, 1);
        assert_eq!(stats.miss_count, 1);
        assert!(stats.hit_rate > 0.0);
    }

    #[test]
    fn test_cache_contains() {
        let cache = SearchCache::new(100, None);

        cache.set("key1".to_string(), vec![1]);
        assert!(cache.contains("key1"));
        assert!(!cache.contains("nonexistent"));
    }

    #[test]
    fn test_shard_distribution() {
        let cache = SearchCache::new(100, None);

        for i in 0..100 {
            cache.set(format!("key_{}", i), vec![i]);
        }

        assert_eq!(cache.len(), 100);
    }

    #[test]
    fn test_cache_is_empty() {
        let cache = SearchCache::new(100, None);
        assert!(cache.is_empty());

        cache.set("key1".to_string(), vec![1]);
        assert!(!cache.is_empty());
    }

    #[test]
    fn test_cache_len() {
        let cache = SearchCache::new(100, None);
        assert_eq!(cache.len(), 0);

        cache.set("key1".to_string(), vec![1]);
        assert_eq!(cache.len(), 1);

        cache.set("key2".to_string(), vec![2]);
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn test_cache_key_generator() {
        let options = SearchOptions {
            limit: Some(10),
            offset: Some(0),
            context: None,
            resolve: Some(true),
            suggest: Some(false),
            ..Default::default()
        };

        let key = CacheKeyGenerator::generate_search_key("test", &options);
        assert!(key.contains("test"));
        assert!(key.contains("limit:10"));
    }

    #[test]
    fn test_cached_search() {
        let search_fn = |query: &str, _options: &SearchOptions| -> Result<SearchResults> {
            Ok(vec![query.len() as u64])
        };

        let cached_search = CachedSearch::new(search_fn, 100, None);

        let options = SearchOptions::default();
        let results = cached_search.search("hello", &options).unwrap();
        assert_eq!(results, vec![5]);

        let cached_results = cached_search.search("hello", &options).unwrap();
        assert_eq!(cached_results, vec![5]);
    }

    #[tokio::test]
    async fn test_cached_search_async() {
        let search_fn = |query: &str, _options: &SearchOptions| -> Result<SearchResults> {
            Ok(vec![query.len() as u64])
        };

        let cached_search = CachedSearch::new(search_fn, 100, None);

        let options = SearchOptions::default();
        let results = cached_search
            .search_async("hello", &options)
            .await
            .unwrap();
        assert_eq!(results, vec![5]);
    }

    #[test]
    fn test_cache_remove_nonexistent() {
        let cache = SearchCache::new(100, None);
        assert!(!cache.remove("nonexistent"));
    }

    #[test]
    fn test_cache_clear_empty() {
        let cache = SearchCache::new(100, None);
        cache.clear();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_stats_empty() {
        let cache = SearchCache::new(100, None);
        let stats = cache.stats();
        assert_eq!(stats.hit_count, 0);
        assert_eq!(stats.miss_count, 0);
        assert_eq!(stats.hit_rate, 0.0);
    }

    #[test]
    fn test_cache_ttl_expiry() {
        let ttl = Some(Duration::from_millis(10));
        let cache = SearchCache::new(100, ttl);

        cache.set("key1".to_string(), vec![1]);
        assert_eq!(cache.get("key1"), Some(vec![1]));

        std::thread::sleep(Duration::from_millis(20));
        assert_eq!(cache.get("key1"), None);
    }

    #[test]
    fn test_cache_contains_after_remove() {
        let cache = SearchCache::new(100, None);

        cache.set("key1".to_string(), vec![1]);
        assert!(cache.contains("key1"));

        cache.remove("key1");
        assert!(!cache.contains("key1"));
    }

    #[test]
    fn test_cache_len_after_operations() {
        let cache = SearchCache::new(100, None);

        cache.set("key1".to_string(), vec![1]);
        cache.set("key2".to_string(), vec![2]);
        assert_eq!(cache.len(), 2);

        cache.remove("key1");
        assert_eq!(cache.len(), 1);

        cache.clear();
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_cache_overwrite_same_key() {
        let cache = SearchCache::new(100, None);

        cache.set("key1".to_string(), vec![1]);
        assert_eq!(cache.len(), 1);

        cache.set("key1".to_string(), vec![2]);
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_cache_eviction_order() {
        let cache = SearchCache::new(3, None);

        cache.set("key1".to_string(), vec![1]);
        cache.set("key2".to_string(), vec![2]);
        cache.set("key3".to_string(), vec![3]);

        cache.get("key1");

        cache.set("key4".to_string(), vec![4]);

        let len = cache.len();
        assert!(len <= 3, "Cache should not exceed max_size, got {}", len);
    }

    #[test]
    fn test_cache_zero_size() {
        let cache = SearchCache::new(0, None);

        cache.set("key1".to_string(), vec![1]);
        let len = cache.len();
        assert!(len <= 0, "Zero-size cache should be empty, got {}", len);
    }

    #[test]
    fn test_cache_large_scale() {
        let cache = SearchCache::new(1000, None);

        for i in 0..1000 {
            cache.set(format!("key_{}", i), vec![i as u64]);
        }

        assert_eq!(cache.len(), 1000);

        for i in 0..1000 {
            let result = cache.get(&format!("key_{}", i));
            assert_eq!(result, Some(vec![i as u64]));
        }
    }

    #[test]
    fn test_cache_eviction_lru() {
        let cache = SearchCache::new(3, None);

        cache.set("a".to_string(), vec![1]);
        cache.set("b".to_string(), vec![2]);
        cache.set("c".to_string(), vec![3]);

        cache.get("a");
        cache.get("b");

        cache.set("d".to_string(), vec![4]);

        let len = cache.len();
        assert!(len <= 3, "Cache should not exceed max_size, got {}", len);
    }

    #[test]
    fn test_cache_remove_updates_len() {
        let cache = SearchCache::new(100, None);

        cache.set("key1".to_string(), vec![1]);
        cache.set("key2".to_string(), vec![2]);
        assert_eq!(cache.len(), 2);

        assert!(cache.remove("key1"));
        assert_eq!(cache.len(), 1);

        assert!(cache.remove("key2"));
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_cache_clear_resets_stats() {
        let cache = SearchCache::new(100, None);

        cache.set("key1".to_string(), vec![1]);
        cache.get("key1");
        cache.get("nonexistent");

        cache.clear();

        let stats = cache.stats();
        assert_eq!(stats.hit_count, 0);
        assert_eq!(stats.miss_count, 0);
        assert_eq!(stats.size, 0);
    }

    #[test]
    fn test_cache_contains_empty() {
        let cache = SearchCache::new(100, None);
        assert!(!cache.contains("anything"));
    }

    #[test]
    fn test_cache_contains_after_clear() {
        let cache = SearchCache::new(100, None);

        cache.set("key1".to_string(), vec![1]);
        cache.clear();

        assert!(!cache.contains("key1"));
    }

    #[test]
    fn test_cache_eviction_with_ttl() {
        let ttl = Some(Duration::from_millis(10));
        let cache = SearchCache::new(3, ttl);

        cache.set("key1".to_string(), vec![1]);
        cache.set("key2".to_string(), vec![2]);
        cache.set("key3".to_string(), vec![3]);

        std::thread::sleep(Duration::from_millis(20));

        cache.set("key4".to_string(), vec![4]);

        let len = cache.len();
        assert!(len <= 3, "Cache should not exceed max_size, got {}", len);
    }

    #[test]
    fn test_cache_eviction_boundary() {
        let cache = SearchCache::new(1, None);

        cache.set("key1".to_string(), vec![1]);
        assert_eq!(cache.len(), 1);

        cache.set("key2".to_string(), vec![2]);
        let len = cache.len();
        assert!(len <= 1, "Cache with max_size=1 should have at most 1 entry, got {}", len);
    }

    #[test]
    fn test_cache_eviction_all() {
        let cache = SearchCache::new(3, None);

        cache.set("key1".to_string(), vec![1]);
        cache.set("key2".to_string(), vec![2]);
        cache.set("key3".to_string(), vec![3]);
        cache.set("key4".to_string(), vec![4]);
        cache.set("key5".to_string(), vec![5]);
        cache.set("key6".to_string(), vec![6]);

        let len = cache.len();
        assert!(len <= 3, "Cache should not exceed max_size, got {}", len);
    }

    #[test]
    fn test_cache_eviction_after_access() {
        let cache = SearchCache::new(3, None);

        cache.set("a".to_string(), vec![1]);
        cache.set("b".to_string(), vec![2]);
        cache.set("c".to_string(), vec![3]);

        cache.get("a");
        cache.get("b");
        cache.get("c");

        cache.set("d".to_string(), vec![4]);
        cache.set("e".to_string(), vec![5]);
        cache.set("f".to_string(), vec![6]);

        let len = cache.len();
        assert!(len <= 3, "Cache should not exceed max_size, got {}", len);
    }

    #[test]
    fn test_cache_eviction_remove_then_add() {
        let cache = SearchCache::new(3, None);

        cache.set("key1".to_string(), vec![1]);
        cache.set("key2".to_string(), vec![2]);
        cache.set("key3".to_string(), vec![3]);

        cache.remove("key1");

        cache.set("key4".to_string(), vec![4]);
        cache.set("key5".to_string(), vec![5]);

        let len = cache.len();
        assert!(len <= 3, "Cache should not exceed max_size, got {}", len);
    }

    #[test]
    fn test_cache_eviction_clear_then_add() {
        let cache = SearchCache::new(3, None);

        cache.set("key1".to_string(), vec![1]);
        cache.set("key2".to_string(), vec![2]);
        cache.set("key3".to_string(), vec![3]);

        cache.clear();

        cache.set("key4".to_string(), vec![4]);
        cache.set("key5".to_string(), vec![5]);
        cache.set("key6".to_string(), vec![6]);

        assert_eq!(cache.len(), 3);
    }

    #[test]
    fn test_cache_eviction_async() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let cache = SearchCache::new(3, None);

            cache.set_async("key1".to_string(), vec![1]).await;
            cache.set_async("key2".to_string(), vec![2]).await;
            cache.set_async("key3".to_string(), vec![3]).await;
            cache.set_async("key4".to_string(), vec![4]).await;

            let len = cache.len();
            assert!(len <= 3, "Cache should not exceed max_size, got {}", len);
        });
    }

    #[test]
    fn test_cache_eviction_async_parallel() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let cache = std::sync::Arc::new(SearchCache::new(10, None));
            let mut handles = Vec::new();

            for i in 0..20 {
                let cache_clone = cache.clone();
                handles.push(tokio::spawn(async move {
                    cache_clone
                        .set_async(format!("key_{}", i), vec![i])
                        .await;
                }));
            }

            for handle in handles {
                handle.await.unwrap();
            }

            let len = cache.len();
            assert!(len <= 10, "Cache should not exceed max_size, got {}", len);
        });
    }
}