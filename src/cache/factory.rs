//! 缓存工厂
//!
//! 负责创建不同类型的缓存实例，提供统一的缓存创建接口

use super::cache_impl::*;
use super::config::CachePolicy;
use super::traits::{Cache, StatsCache};
use std::hash::Hash;
use std::sync::Arc;
use std::time::Duration;

/// 缓存工厂
/// 
/// 负责创建各种类型的缓存实例，提供统一的创建接口
pub struct CacheFactory;

impl CacheFactory {
    /// 创建LRU缓存
    pub fn create_lru_cache<K, V>(capacity: usize) -> Arc<ConcurrentLruCache<K, V>>
    where
        K: 'static + Send + Sync + Hash + Eq + Clone,
        V: 'static + Send + Sync + Clone,
    {
        Arc::new(ConcurrentLruCache::new(capacity))
    }

    /// 创建LFU缓存
    pub fn create_lfu_cache<K, V>(capacity: usize) -> Arc<ConcurrentLfuCache<K, V>>
    where
        K: 'static + Send + Sync + Hash + Eq + Clone,
        V: 'static + Send + Sync + Clone,
    {
        Arc::new(ConcurrentLfuCache::new(capacity))
    }

    /// 创建TTL缓存
    pub fn create_ttl_cache<K, V>(
        capacity: usize,
        default_ttl: Duration,
    ) -> Arc<ConcurrentTtlCache<K, V>>
    where
        K: 'static + Send + Sync + Hash + Eq + Clone,
        V: 'static + Send + Sync + Clone,
    {
        Arc::new(ConcurrentTtlCache::new(capacity, default_ttl))
    }

    /// 创建FIFO缓存
    pub fn create_fifo_cache<K, V>(capacity: usize) -> Arc<ConcurrentFifoCache<K, V>>
    where
        K: 'static + Send + Sync + Hash + Eq + Clone,
        V: 'static + Send + Sync + Clone,
    {
        Arc::new(ConcurrentFifoCache::new(capacity))
    }

    /// 创建自适应缓存
    pub fn create_adaptive_cache<K, V>(capacity: usize) -> Arc<AdaptiveCache<K, V>>
    where
        K: 'static + Send + Sync + Hash + Eq + Clone,
        V: 'static + Send + Sync + Clone,
    {
        Arc::new(AdaptiveCache::new(capacity))
    }

    /// 创建无界缓存
    pub fn create_unbounded_cache<K, V>() -> Arc<ConcurrentUnboundedCache<K, V>>
    where
        K: 'static + Send + Sync + Hash + Eq + Clone,
        V: 'static + Send + Sync + Clone,
    {
        Arc::new(ConcurrentUnboundedCache::new())
    }

    /// 根据策略创建缓存
    pub fn create_cache_by_policy<K, V>(
        policy: &CachePolicy,
        capacity: usize,
    ) -> CacheType<K, V>
    where
        K: 'static + Send + Sync + Hash + Eq + Clone,
        V: 'static + Send + Sync + Clone,
    {
        match policy {
            CachePolicy::LRU => CacheType::Lru(Self::create_lru_cache(capacity)),
            CachePolicy::LFU => CacheType::Lfu(Self::create_lfu_cache(capacity)),
            CachePolicy::TTL(ttl) => CacheType::Ttl(Self::create_ttl_cache(capacity, *ttl)),
            CachePolicy::FIFO => CacheType::Fifo(Self::create_fifo_cache(capacity)),
            CachePolicy::Adaptive => CacheType::Adaptive(Self::create_adaptive_cache(capacity)),
            CachePolicy::None => CacheType::Unbounded(Self::create_unbounded_cache()),
        }
    }

    /// 根据策略创建带统计的缓存
    pub fn create_stats_cache_by_policy<K, V>(
        policy: &CachePolicy,
        capacity: usize,
    ) -> StatsCacheType<K, V>
    where
        K: 'static + Send + Sync + Hash + Eq + Clone,
        V: 'static + Send + Sync + Clone,
    {
        match policy {
            CachePolicy::LRU => {
                let cache = Self::create_lru_cache(capacity);
                StatsCacheType::Lru(Arc::new(StatsCacheWrapper::new(cache)))
            }
            CachePolicy::LFU => {
                let cache = Self::create_lfu_cache(capacity);
                StatsCacheType::Lfu(Arc::new(StatsCacheWrapper::new(cache)))
            }
            CachePolicy::TTL(ttl) => {
                let cache = Self::create_ttl_cache(capacity, *ttl);
                StatsCacheType::Ttl(Arc::new(StatsCacheWrapper::new(cache)))
            }
            CachePolicy::FIFO => {
                let cache = Self::create_fifo_cache(capacity);
                StatsCacheType::Fifo(Arc::new(StatsCacheWrapper::new(cache)))
            }
            CachePolicy::Adaptive => {
                let cache = Self::create_adaptive_cache(capacity);
                StatsCacheType::Adaptive(Arc::new(StatsCacheWrapper::new(cache)))
            }
            CachePolicy::None => {
                let cache = Self::create_unbounded_cache();
                StatsCacheType::Unbounded(Arc::new(StatsCacheWrapper::new(cache)))
            }
        }
    }

    /// 创建带统计的缓存包装器
    pub fn create_stats_wrapper<K, V, C>(cache: Arc<C>) -> Arc<StatsCacheWrapper<K, V, C>>
    where
        K: 'static + Send + Sync + Hash + Eq + Clone,
        V: 'static + Send + Sync + Clone,
        C: Cache<K, V>,
    {
        Arc::new(StatsCacheWrapper::new(cache))
    }

    /// 验证缓存容量
    pub fn validate_capacity(capacity: usize) -> Result<(), String> {
        if capacity == 0 {
            return Err("缓存容量必须大于0".to_string());
        }
        if capacity > usize::MAX / 2 {
            return Err("缓存容量过大".to_string());
        }
        Ok(())
    }

    /// 验证TTL
    pub fn validate_ttl(ttl: Duration) -> Result<(), String> {
        if ttl.is_zero() {
            return Err("TTL必须大于0".to_string());
        }
        Ok(())
    }
}

/// 缓存类型枚举 - 避免动态分发
#[derive(Debug)]
pub enum CacheType<K, V> {
    Lru(Arc<ConcurrentLruCache<K, V>>),
    Lfu(Arc<ConcurrentLfuCache<K, V>>),
    Ttl(Arc<ConcurrentTtlCache<K, V>>),
    Fifo(Arc<ConcurrentFifoCache<K, V>>),
    Adaptive(Arc<AdaptiveCache<K, V>>),
    Unbounded(Arc<ConcurrentUnboundedCache<K, V>>),
}

/// 统计缓存类型枚举 - 避免动态分发
#[derive(Debug)]
pub enum StatsCacheType<K, V> {
    Lru(Arc<StatsCacheWrapper<K, V, ConcurrentLruCache<K, V>>>),
    Lfu(Arc<StatsCacheWrapper<K, V, ConcurrentLfuCache<K, V>>>),
    Ttl(Arc<StatsCacheWrapper<K, V, ConcurrentTtlCache<K, V>>>),
    Fifo(Arc<StatsCacheWrapper<K, V, ConcurrentFifoCache<K, V>>>),
    Adaptive(Arc<StatsCacheWrapper<K, V, AdaptiveCache<K, V>>>),
    Unbounded(Arc<StatsCacheWrapper<K, V, ConcurrentUnboundedCache<K, V>>>),
}

// 为 CacheType 实现 Cache trait
impl<K, V> Cache<K, V> for CacheType<K, V>
where
    K: 'static + Send + Sync + Hash + Eq + Clone,
    V: 'static + Send + Sync + Clone,
{
    fn get(&self, key: &K) -> Option<V> {
        match self {
            CacheType::Lru(cache) => cache.get(key),
            CacheType::Lfu(cache) => cache.get(key),
            CacheType::Ttl(cache) => cache.get(key),
            CacheType::Fifo(cache) => cache.get(key),
            CacheType::Adaptive(cache) => cache.get(key),
            CacheType::Unbounded(cache) => cache.get(key),
        }
    }

    fn put(&self, key: K, value: V) {
        match self {
            CacheType::Lru(cache) => cache.put(key, value),
            CacheType::Lfu(cache) => cache.put(key, value),
            CacheType::Ttl(cache) => cache.put(key, value),
            CacheType::Fifo(cache) => cache.put(key, value),
            CacheType::Adaptive(cache) => cache.put(key, value),
            CacheType::Unbounded(cache) => cache.put(key, value),
        }
    }

    fn contains(&self, key: &K) -> bool {
        match self {
            CacheType::Lru(cache) => cache.contains(key),
            CacheType::Lfu(cache) => cache.contains(key),
            CacheType::Ttl(cache) => cache.contains(key),
            CacheType::Fifo(cache) => cache.contains(key),
            CacheType::Adaptive(cache) => cache.contains(key),
            CacheType::Unbounded(cache) => cache.contains(key),
        }
    }

    fn remove(&self, key: &K) -> Option<V> {
        match self {
            CacheType::Lru(cache) => cache.remove(key),
            CacheType::Lfu(cache) => cache.remove(key),
            CacheType::Ttl(cache) => cache.remove(key),
            CacheType::Fifo(cache) => cache.remove(key),
            CacheType::Adaptive(cache) => cache.remove(key),
            CacheType::Unbounded(cache) => cache.remove(key),
        }
    }

    fn clear(&self) {
        match self {
            CacheType::Lru(cache) => Cache::clear(cache),
            CacheType::Lfu(cache) => Cache::clear(cache),
            CacheType::Ttl(cache) => Cache::clear(cache),
            CacheType::Fifo(cache) => Cache::clear(cache),
            CacheType::Adaptive(cache) => Cache::clear(cache),
            CacheType::Unbounded(cache) => Cache::clear(cache),
        }
    }

    fn len(&self) -> usize {
        match self {
            CacheType::Lru(cache) => Cache::len(cache),
            CacheType::Lfu(cache) => Cache::len(cache),
            CacheType::Ttl(cache) => Cache::len(cache),
            CacheType::Fifo(cache) => Cache::len(cache),
            CacheType::Adaptive(cache) => Cache::len(cache),
            CacheType::Unbounded(cache) => Cache::len(cache),
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            CacheType::Lru(cache) => cache.is_empty(),
            CacheType::Lfu(cache) => cache.is_empty(),
            CacheType::Ttl(cache) => cache.is_empty(),
            CacheType::Fifo(cache) => cache.is_empty(),
            CacheType::Adaptive(cache) => cache.is_empty(),
            CacheType::Unbounded(cache) => cache.is_empty(),
        }
    }
}

// 为 StatsCacheType 实现 Cache 和 StatsCache trait
impl<K, V> Cache<K, V> for StatsCacheType<K, V>
where
    K: 'static + Send + Sync + Hash + Eq + Clone,
    V: 'static + Send + Sync + Clone,
{
    fn get(&self, key: &K) -> Option<V> {
        match self {
            StatsCacheType::Lru(cache) => cache.get(key),
            StatsCacheType::Lfu(cache) => cache.get(key),
            StatsCacheType::Ttl(cache) => cache.get(key),
            StatsCacheType::Fifo(cache) => cache.get(key),
            StatsCacheType::Adaptive(cache) => cache.get(key),
            StatsCacheType::Unbounded(cache) => cache.get(key),
        }
    }

    fn put(&self, key: K, value: V) {
        match self {
            StatsCacheType::Lru(cache) => cache.put(key, value),
            StatsCacheType::Lfu(cache) => cache.put(key, value),
            StatsCacheType::Ttl(cache) => cache.put(key, value),
            StatsCacheType::Fifo(cache) => cache.put(key, value),
            StatsCacheType::Adaptive(cache) => cache.put(key, value),
            StatsCacheType::Unbounded(cache) => cache.put(key, value),
        }
    }

    fn contains(&self, key: &K) -> bool {
        match self {
            StatsCacheType::Lru(cache) => cache.contains(key),
            StatsCacheType::Lfu(cache) => cache.contains(key),
            StatsCacheType::Ttl(cache) => cache.contains(key),
            StatsCacheType::Fifo(cache) => cache.contains(key),
            StatsCacheType::Adaptive(cache) => cache.contains(key),
            StatsCacheType::Unbounded(cache) => cache.contains(key),
        }
    }

    fn remove(&self, key: &K) -> Option<V> {
        match self {
            StatsCacheType::Lru(cache) => cache.remove(key),
            StatsCacheType::Lfu(cache) => cache.remove(key),
            StatsCacheType::Ttl(cache) => cache.remove(key),
            StatsCacheType::Fifo(cache) => cache.remove(key),
            StatsCacheType::Adaptive(cache) => cache.remove(key),
            StatsCacheType::Unbounded(cache) => cache.remove(key),
        }
    }

    fn clear(&self) {
        match self {
            StatsCacheType::Lru(cache) => cache.clear(),
            StatsCacheType::Lfu(cache) => cache.clear(),
            StatsCacheType::Ttl(cache) => cache.clear(),
            StatsCacheType::Fifo(cache) => cache.clear(),
            StatsCacheType::Adaptive(cache) => cache.clear(),
            StatsCacheType::Unbounded(cache) => cache.clear(),
        }
    }

    fn len(&self) -> usize {
        match self {
            StatsCacheType::Lru(cache) => cache.len(),
            StatsCacheType::Lfu(cache) => cache.len(),
            StatsCacheType::Ttl(cache) => cache.len(),
            StatsCacheType::Fifo(cache) => cache.len(),
            StatsCacheType::Adaptive(cache) => cache.len(),
            StatsCacheType::Unbounded(cache) => cache.len(),
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            StatsCacheType::Lru(cache) => cache.is_empty(),
            StatsCacheType::Lfu(cache) => cache.is_empty(),
            StatsCacheType::Ttl(cache) => cache.is_empty(),
            StatsCacheType::Fifo(cache) => cache.is_empty(),
            StatsCacheType::Adaptive(cache) => cache.is_empty(),
            StatsCacheType::Unbounded(cache) => cache.is_empty(),
        }
    }
}

impl<K, V> StatsCache<K, V> for StatsCacheType<K, V>
where
    K: 'static + Send + Sync + Hash + Eq + Clone,
    V: 'static + Send + Sync + Clone,
{
    fn hits(&self) -> u64 {
        match self {
            StatsCacheType::Lru(cache) => cache.hits(),
            StatsCacheType::Lfu(cache) => cache.hits(),
            StatsCacheType::Ttl(cache) => cache.hits(),
            StatsCacheType::Fifo(cache) => cache.hits(),
            StatsCacheType::Adaptive(cache) => cache.hits(),
            StatsCacheType::Unbounded(cache) => cache.hits(),
        }
    }

    fn misses(&self) -> u64 {
        match self {
            StatsCacheType::Lru(cache) => cache.misses(),
            StatsCacheType::Lfu(cache) => cache.misses(),
            StatsCacheType::Ttl(cache) => cache.misses(),
            StatsCacheType::Fifo(cache) => cache.misses(),
            StatsCacheType::Adaptive(cache) => cache.misses(),
            StatsCacheType::Unbounded(cache) => cache.misses(),
        }
    }

    fn hit_rate(&self) -> f64 {
        match self {
            StatsCacheType::Lru(cache) => cache.hit_rate(),
            StatsCacheType::Lfu(cache) => cache.hit_rate(),
            StatsCacheType::Ttl(cache) => cache.hit_rate(),
            StatsCacheType::Fifo(cache) => cache.hit_rate(),
            StatsCacheType::Adaptive(cache) => cache.hit_rate(),
            StatsCacheType::Unbounded(cache) => cache.hit_rate(),
        }
    }

    fn evictions(&self) -> u64 {
        match self {
            StatsCacheType::Lru(cache) => cache.evictions(),
            StatsCacheType::Lfu(cache) => cache.evictions(),
            StatsCacheType::Ttl(cache) => cache.evictions(),
            StatsCacheType::Fifo(cache) => cache.evictions(),
            StatsCacheType::Adaptive(cache) => cache.evictions(),
            StatsCacheType::Unbounded(cache) => cache.evictions(),
        }
    }

    fn reset_stats(&self) {
        match self {
            StatsCacheType::Lru(cache) => cache.reset_stats(),
            StatsCacheType::Lfu(cache) => cache.reset_stats(),
            StatsCacheType::Ttl(cache) => cache.reset_stats(),
            StatsCacheType::Fifo(cache) => cache.reset_stats(),
            StatsCacheType::Adaptive(cache) => cache.reset_stats(),
            StatsCacheType::Unbounded(cache) => cache.reset_stats(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_cache_factory_create_lru() {
        let cache = CacheFactory::create_lru_cache::<String, String>(100);
        cache.put("key".to_string(), "value".to_string());
        assert_eq!(cache.get(&"key".to_string()), Some("value".to_string()));
    }

    #[test]
    fn test_cache_factory_create_lfu() {
        let cache = CacheFactory::create_lfu_cache::<i32, String>(100);
        cache.put(1, "value".to_string());
        assert_eq!(cache.get(&1), Some("value".to_string()));
    }

    #[test]
    fn test_cache_factory_create_ttl() {
        let cache = CacheFactory::create_ttl_cache::<String, i32>(100, Duration::from_secs(60));
        cache.put("key".to_string(), 42);
        assert_eq!(cache.get(&"key".to_string()), Some(42));
    }

    #[test]
    fn test_cache_factory_create_fifo() {
        let cache = CacheFactory::create_fifo_cache::<char, f64>(100);
        cache.put('a', 3.14);
        assert_eq!(cache.get(&'a'), Some(3.14));
    }

    #[test]
    fn test_cache_factory_create_adaptive() {
        let cache = CacheFactory::create_adaptive_cache::<bool, String>(100);
        cache.put(true, "value".to_string());
        assert_eq!(cache.get(&true), Some("value".to_string()));
    }

    #[test]
    fn test_cache_factory_create_unbounded() {
        let cache = CacheFactory::create_unbounded_cache::<String, Vec<i32>>();
        cache.put("key".to_string(), vec![1, 2, 3]);
        assert_eq!(cache.get(&"key".to_string()), Some(vec![1, 2, 3]));
    }

    #[test]
    fn test_cache_factory_create_by_policy() {
        let lru_cache = CacheFactory::create_cache_by_policy::<String, String>(
            &CachePolicy::LRU, 
            100
        );
        lru_cache.put("key".to_string(), "value".to_string());
        assert_eq!(lru_cache.get(&"key".to_string()), Some("value".to_string()));

        let ttl_cache = CacheFactory::create_cache_by_policy::<String, String>(
            &CachePolicy::TTL(Duration::from_secs(60)), 
            100
        );
        ttl_cache.put("key".to_string(), "value".to_string());
        assert_eq!(ttl_cache.get(&"key".to_string()), Some("value".to_string()));
    }

    #[test]
    fn test_cache_factory_create_stats_by_policy() {
        let lru_cache = CacheFactory::create_stats_cache_by_policy::<String, String>(
            &CachePolicy::LRU, 
            100
        );
        lru_cache.put("key".to_string(), "value".to_string());
        assert_eq!(lru_cache.get(&"key".to_string()), Some("value".to_string()));
        assert_eq!(lru_cache.hits(), 1);
        assert_eq!(lru_cache.misses(), 0);
    }

    #[test]
    fn test_cache_factory_validation() {
        assert!(CacheFactory::validate_capacity(100).is_ok());
        assert!(CacheFactory::validate_capacity(0).is_err());
        
        assert!(CacheFactory::validate_ttl(Duration::from_secs(60)).is_ok());
        assert!(CacheFactory::validate_ttl(Duration::from_secs(0)).is_err());
    }

    #[test]
    fn test_cache_type_enum() {
        let cache = CacheType::Lru(CacheFactory::create_lru_cache::<String, String>(100));
        cache.put("key".to_string(), "value".to_string());
        assert_eq!(cache.get(&"key".to_string()), Some("value".to_string()));
        assert!(cache.contains(&"key".to_string()));
        assert_eq!(cache.len(), 1);
        assert!(!cache.is_empty());
        
        cache.remove(&"key".to_string());
        assert!(!cache.contains(&"key".to_string()));
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_stats_cache_type_enum() {
        let cache = StatsCacheType::Lru(Arc::new(StatsCacheWrapper::new(
            CacheFactory::create_lru_cache::<String, String>(100)
        )));
        
        cache.put("key".to_string(), "value".to_string());
        assert_eq!(cache.get(&"key".to_string()), Some("value".to_string()));
        assert_eq!(cache.hits(), 1);
        assert_eq!(cache.misses(), 0);
        assert_eq!(cache.hit_rate(), 1.0);
        
        cache.reset_stats();
        assert_eq!(cache.hits(), 0);
        assert_eq!(cache.misses(), 0);
    }
}