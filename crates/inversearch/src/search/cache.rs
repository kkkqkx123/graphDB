//! 缓存模块
//!
//! 提供搜索结果缓存功能，提高查询性能

use crate::error::Result;
use crate::r#type::{SearchOptions, SearchResults};
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// 缓存键生成器
pub struct CacheKeyGenerator;

impl CacheKeyGenerator {
    /// 生成搜索缓存键
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

    /// 生成文档缓存键
    pub fn generate_document_key(doc_id: u64) -> String {
        format!("doc:{}", doc_id)
    }
}

/// 缓存条目
#[derive(Clone)]
struct CacheEntry {
    data: SearchResults,
    created_at: Instant,
    access_count: u64,
}

/// 搜索缓存（异步版本）
#[derive(Clone)]
pub struct SearchCache {
    store: Arc<RwLock<LruCache<String, CacheEntry>>>,
    default_ttl: Option<Duration>,
    max_size: usize,
    hit_count: Arc<AtomicU64>,
    miss_count: Arc<AtomicU64>,
}

impl SearchCache {
    /// 创建新的搜索缓存
    pub fn new(max_size: usize, default_ttl: Option<Duration>) -> Self {
        let cap = NonZeroUsize::new(max_size.max(1))
            .or_else(|| NonZeroUsize::new(1000))
            .expect("Default cache size should be valid");
        Self {
            store: Arc::new(RwLock::new(LruCache::new(cap))),
            default_ttl,
            max_size,
            hit_count: Arc::new(AtomicU64::new(0)),
            miss_count: Arc::new(AtomicU64::new(0)),
        }
    }

    /// 异步获取缓存项
    pub async fn get_async(&self, key: &str) -> Option<SearchResults> {
        let mut store = self.store.write().await;

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

    /// 异步设置缓存项
    pub async fn set_async(&self, key: String, data: SearchResults) {
        let mut store = self.store.write().await;
        let entry = CacheEntry {
            data,
            created_at: Instant::now(),
            access_count: 1,
        };
        store.put(key, entry);
    }

    /// 异步删除缓存项
    pub async fn remove_async(&self, key: &str) -> bool {
        let mut store = self.store.write().await;
        store.pop(key).is_some()
    }

    /// 异步清空缓存
    pub async fn clear_async(&self) {
        let mut store = self.store.write().await;
        store.clear();
        self.hit_count.store(0, Ordering::Relaxed);
        self.miss_count.store(0, Ordering::Relaxed);
    }

    /// 同步获取缓存项（用于向后兼容，可能阻塞）
    pub fn get(&self, key: &str) -> Option<SearchResults> {
        if let Ok(mut store) = self.store.try_write() {
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
        } else {
            None
        }
    }

    /// 同步设置缓存项（用于向后兼容，可能阻塞）
    pub fn set(&self, key: String, data: SearchResults) {
        if let Ok(mut store) = self.store.try_write() {
            let entry = CacheEntry {
                data,
                created_at: Instant::now(),
                access_count: 1,
            };
            store.put(key, entry);
        }
    }

    /// 同步删除缓存项（用于向后兼容，可能阻塞）
    pub fn remove(&self, key: &str) -> bool {
        if let Ok(mut store) = self.store.try_write() {
            store.pop(key).is_some()
        } else {
            false
        }
    }

    /// 同步清空缓存（用于向后兼容，可能阻塞）
    pub fn clear(&self) {
        if let Ok(mut store) = self.store.try_write() {
            store.clear();
        }
        self.hit_count.store(0, Ordering::Relaxed);
        self.miss_count.store(0, Ordering::Relaxed);
    }

    /// 获取缓存统计
    pub fn stats(&self) -> CacheStats {
        let hit_count = self.hit_count.load(Ordering::Relaxed);
        let miss_count = self.miss_count.load(Ordering::Relaxed);
        let total_requests = hit_count + miss_count;
        let hit_rate = if total_requests > 0 {
            hit_count as f64 / total_requests as f64
        } else {
            0.0
        };

        let size = if let Ok(store) = self.store.try_read() {
            store.len()
        } else {
            0
        };

        CacheStats {
            size,
            max_size: self.max_size,
            hit_count,
            miss_count,
            hit_rate,
            total_requests,
        }
    }

    /// 异步获取缓存统计
    pub async fn stats_async(&self) -> CacheStats {
        let hit_count = self.hit_count.load(Ordering::Relaxed);
        let miss_count = self.miss_count.load(Ordering::Relaxed);
        let total_requests = hit_count + miss_count;
        let hit_rate = if total_requests > 0 {
            hit_count as f64 / total_requests as f64
        } else {
            0.0
        };

        let store = self.store.read().await;
        let size = store.len();

        CacheStats {
            size,
            max_size: self.max_size,
            hit_count,
            miss_count,
            hit_rate,
            total_requests,
        }
    }

    /// 检查缓存是否包含键
    pub fn contains(&self, key: &str) -> bool {
        if let Ok(store) = self.store.try_read() {
            store.contains(key)
        } else {
            false
        }
    }

    /// 异步检查缓存是否包含键
    pub async fn contains_async(&self, key: &str) -> bool {
        let store = self.store.read().await;
        store.contains(key)
    }

    /// 获取当前缓存大小
    pub fn len(&self) -> usize {
        if let Ok(store) = self.store.try_read() {
            store.len()
        } else {
            0
        }
    }

    /// 检查缓存是否为空
    pub fn is_empty(&self) -> bool {
        if let Ok(store) = self.store.try_read() {
            store.is_empty()
        } else {
            true
        }
    }
}

/// 缓存统计信息
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub size: usize,
    pub max_size: usize,
    pub hit_count: u64,
    pub miss_count: u64,
    pub hit_rate: f64,
    pub total_requests: u64,
}

/// 带缓存的搜索包装器（异步版本）
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
    /// 创建新的缓存搜索包装器
    pub fn new(search_fn: F, cache_size: usize, ttl: Option<Duration>) -> Self {
        Self {
            cache: SearchCache::new(cache_size, ttl),
            search_fn: Arc::new(search_fn),
        }
    }

    /// 执行带缓存的搜索（同步版本）
    pub fn search(&self, query: &str, options: &SearchOptions) -> Result<SearchResults> {
        let cache_key = CacheKeyGenerator::generate_search_key(query, options);

        if let Some(cached_results) = self.cache.get(&cache_key) {
            return Ok(cached_results);
        }

        let results = (self.search_fn)(query, options)?;

        self.cache.set(cache_key, results.clone());

        Ok(results)
    }

    /// 执行带缓存的搜索（异步版本）
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

    /// 获取缓存统计
    pub fn cache_stats(&self) -> CacheStats {
        self.cache.stats()
    }

    /// 异步获取缓存统计
    pub async fn cache_stats_async(&self) -> CacheStats {
        self.cache.stats_async().await
    }

    /// 清空缓存
    pub fn clear_cache(&self) {
        self.cache.clear();
    }

    /// 异步清空缓存
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

        assert_eq!(cache.len(), 3);
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
}
