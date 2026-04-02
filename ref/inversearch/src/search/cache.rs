//! 缓存模块
//! 
//! 提供搜索结果缓存功能，提高查询性能

use lru::LruCache;
use std::num::NonZeroUsize;
use std::time::{Duration, Instant};
use crate::r#type::{SearchOptions, SearchResults};
use crate::error::Result;

/// 缓存键生成器
pub struct CacheKeyGenerator;

impl CacheKeyGenerator {
    /// 生成搜索缓存键
    pub fn generate_search_key(query: &str, options: &SearchOptions) -> String {
        let mut key_parts = vec![query.to_lowercase()];
        
        // 添加基本参数
        key_parts.push(format!("limit:{}", options.limit.unwrap_or(100)));
        key_parts.push(format!("offset:{}", options.offset.unwrap_or(0)));
        key_parts.push(format!("context:{}", options.context.is_some()));
        key_parts.push(format!("resolve:{}", options.resolve.unwrap_or(true)));
        key_parts.push(format!("suggest:{}", options.suggest.unwrap_or(false)));
        
        // 添加其他选项
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

/// 搜索缓存
#[derive(Clone)]
pub struct SearchCache {
    store: std::sync::Arc<std::sync::Mutex<LruCache<String, CacheEntry>>>,
    default_ttl: Option<Duration>,
    max_size: usize,
    hit_count: std::sync::Arc<std::sync::atomic::AtomicU64>,
    miss_count: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

impl SearchCache {
    /// 创建新的搜索缓存
    pub fn new(max_size: usize, default_ttl: Option<Duration>) -> Self {
        Self {
            store: std::sync::Arc::new(std::sync::Mutex::new(LruCache::new(
                NonZeroUsize::new(max_size).unwrap_or_else(|| NonZeroUsize::new(1000).unwrap())
            ))),
            default_ttl,
            max_size,
            hit_count: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0)),
            miss_count: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }
    
    /// 获取缓存项
    pub fn get(&mut self, key: &str) -> Option<SearchResults> {
        if let Ok(mut store) = self.store.lock() {
            if let Some(entry) = store.get_mut(key) {
                // 检查是否过期
                if let Some(ttl) = self.default_ttl {
                    if entry.created_at.elapsed() > ttl {
                        // 过期，移除并返回None
                        store.pop(key);
                        self.miss_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        return None;
                    }
                }
                
                entry.access_count += 1;
                self.hit_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                Some(entry.data.clone())
            } else {
                self.miss_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                None
            }
        } else {
            None
        }
    }
    
    /// 设置缓存项
    pub fn set(&mut self, key: String, data: SearchResults) {
        if let Ok(mut store) = self.store.lock() {
            let entry = CacheEntry {
                data,
                created_at: Instant::now(),
                access_count: 1,
            };
            store.put(key, entry);
        }
    }
    
    /// 删除缓存项
    pub fn remove(&mut self, key: &str) -> bool {
        if let Ok(mut store) = self.store.lock() {
            store.pop(key).is_some()
        } else {
            false
        }
    }
    
    /// 清空缓存
    pub fn clear(&mut self) {
        if let Ok(mut store) = self.store.lock() {
            store.clear();
        }
        self.hit_count.store(0, std::sync::atomic::Ordering::Relaxed);
        self.miss_count.store(0, std::sync::atomic::Ordering::Relaxed);
    }
    
    /// 获取缓存统计
    pub fn stats(&self) -> CacheStats {
        let hit_count = self.hit_count.load(std::sync::atomic::Ordering::Relaxed);
        let miss_count = self.miss_count.load(std::sync::atomic::Ordering::Relaxed);
        let total_requests = hit_count + miss_count;
        let hit_rate = if total_requests > 0 {
            hit_count as f64 / total_requests as f64
        } else {
            0.0
        };
        
        CacheStats {
            size: self.store.lock().map(|s| s.len()).unwrap_or(0),
            max_size: self.max_size,
            hit_count,
            miss_count,
            hit_rate,
            total_requests,
        }
    }
    
    /// 检查缓存是否包含键
    pub fn contains(&self, key: &str) -> bool {
        self.store.lock().map(|s| s.contains(key)).unwrap_or(false)
    }
    
    /// 获取当前缓存大小
    pub fn len(&self) -> usize {
        self.store.lock().map(|s| s.len()).unwrap_or(0)
    }
    
    /// 检查缓存是否为空
    pub fn is_empty(&self) -> bool {
        self.store.lock().map(|s| s.is_empty()).unwrap_or(true)
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

/// 带缓存的搜索包装器
pub struct CachedSearch<F>
where
    F: Fn(&str, &SearchOptions) -> Result<SearchResults>,
{
    cache: SearchCache,
    search_fn: F,
}

impl<F> CachedSearch<F>
where
    F: Fn(&str, &SearchOptions) -> Result<SearchResults>,
{
    /// 创建新的缓存搜索包装器
    pub fn new(search_fn: F, cache_size: usize, ttl: Option<Duration>) -> Self {
        Self {
            cache: SearchCache::new(cache_size, ttl),
            search_fn,
        }
    }
    
    /// 执行带缓存的搜索
    pub fn search(&mut self, query: &str, options: &SearchOptions) -> Result<SearchResults> {
        let cache_key = CacheKeyGenerator::generate_search_key(query, options);
        
        // 尝试从缓存获取
        if let Some(cached_results) = self.cache.get(&cache_key) {
            return Ok(cached_results);
        }
        
        // 执行实际搜索
        let results = (self.search_fn)(query, options)?;
        
        // 缓存结果
        self.cache.set(cache_key, results.clone());
        
        Ok(results)
    }
    
    /// 获取缓存统计
    pub fn cache_stats(&self) -> CacheStats {
        self.cache.stats()
    }
    
    /// 清空缓存
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cache_key_generation() {
        let mut options = SearchOptions::default();
        options.query = Some("hello world".to_string());
        options.limit = Some(50);
        options.offset = Some(10);
        
        let key = CacheKeyGenerator::generate_search_key("Hello World", &options);
        assert!(key.contains("hello world"));
        assert!(key.contains("limit:50"));
        assert!(key.contains("offset:10"));
    }
    
    #[test]
    fn test_search_cache_basic() {
        let mut cache = SearchCache::new(100, None);
        let results = vec![1, 2, 3, 4, 5];
        
        // 设置缓存
        cache.set("test_key".to_string(), results.clone());
        
        // 获取缓存
        let cached = cache.get("test_key").unwrap();
        assert_eq!(cached, results);
        
        // 检查统计
        let stats = cache.stats();
        assert_eq!(stats.size, 1);
        assert_eq!(stats.hit_count, 1);
        assert_eq!(stats.miss_count, 0);
        assert_eq!(stats.hit_rate, 1.0);
    }
    
    #[test]
    fn test_search_cache_miss() {
        let mut cache = SearchCache::new(100, None);
        
        // 尝试获取不存在的键
        let result = cache.get("nonexistent");
        assert!(result.is_none());
        
        let stats = cache.stats();
        assert_eq!(stats.hit_count, 0);
        assert_eq!(stats.miss_count, 1);
        assert_eq!(stats.hit_rate, 0.0);
    }
    
    #[test]
    fn test_cached_search() {
        let search_fn = |_query: &str, _options: &SearchOptions| -> Result<SearchResults> {
            Ok(vec![1, 2, 3])
        };
        
        let mut cached_search = CachedSearch::new(search_fn, 100, None);
        let options = SearchOptions::default();
        
        // 第一次搜索（缓存未命中）
        let results1 = cached_search.search("test", &options).unwrap();
        assert_eq!(results1, vec![1, 2, 3]);
        
        // 第二次搜索（缓存命中）
        let results2 = cached_search.search("test", &options).unwrap();
        assert_eq!(results2, vec![1, 2, 3]);
        
        // 检查统计
        let stats = cached_search.cache_stats();
        assert_eq!(stats.hit_count, 1);
        assert_eq!(stats.miss_count, 1);
        assert_eq!(stats.total_requests, 2);
    }
    
    #[test]
    fn test_cache_ttl() {
        use std::thread;
        
        let mut cache = SearchCache::new(100, Some(Duration::from_millis(100)));
        let results = vec![1, 2, 3];
        
        // 设置缓存
        cache.set("ttl_test".to_string(), results.clone());
        
        // 立即获取（应该命中）
        assert!(cache.get("ttl_test").is_some());
        
        // 等待过期
        thread::sleep(Duration::from_millis(150));
        
        // 再次获取（应该未命中，因为已过期）
        assert!(cache.get("ttl_test").is_none());
    }
}