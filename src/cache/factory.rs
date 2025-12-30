//! 缓存工厂
//!
//! 负责创建不同类型的缓存实例，提供统一的缓存创建接口
//!
//! 优化说明：
//! - 移除了 CacheType 和 StatsCacheType 枚举，避免不必要的运行时开销
//! - 保留核心创建方法和配置验证功能
//! - 推荐直接使用具体类型以获得零运行时开销

use super::cache_impl::*;
use super::stats_marker::{StatsDisabled, StatsEnabled};
use super::traits::Cache;
use std::hash::Hash;
use std::sync::Arc;
use std::time::Duration;

/// 缓存工厂
///
/// 负责创建各种类型的缓存实例，提供统一的创建接口
/// 包含配置验证功能，确保缓存参数的有效性
pub struct CacheFactory;

impl CacheFactory {
    /// 创建LRU缓存
    ///
    /// 带配置验证，确保容量参数有效
    pub fn create_lru_cache<K, V>(capacity: usize) -> Arc<ConcurrentLruCache<K, V>>
    where
        K: 'static + Send + Sync + Hash + Eq + Clone,
        V: 'static + Send + Sync + Clone,
    {
        Self::validate_capacity(capacity).expect("Invalid LRU cache capacity");
        Arc::new(ConcurrentLruCache::new(capacity))
    }

    /// 创建LFU缓存
    ///
    /// 带配置验证，确保容量参数有效
    pub fn create_lfu_cache<K, V>(capacity: usize) -> Arc<ConcurrentLfuCache<K, V>>
    where
        K: 'static + Send + Sync + Hash + Eq + Clone,
        V: 'static + Send + Sync + Clone,
    {
        Self::validate_capacity(capacity).expect("Invalid LFU cache capacity");
        Arc::new(ConcurrentLfuCache::new(capacity))
    }

    /// 创建TTL缓存
    ///
    /// 带配置验证，确保容量和TTL参数有效
    pub fn create_ttl_cache<K, V>(
        capacity: usize,
        default_ttl: Duration,
    ) -> Arc<ConcurrentTtlCache<K, V>>
    where
        K: 'static + Send + Sync + Hash + Eq + Clone,
        V: 'static + Send + Sync + Clone,
    {
        Self::validate_capacity(capacity).expect("Invalid TTL cache capacity");
        Self::validate_ttl(default_ttl).expect("Invalid TTL value");
        Arc::new(ConcurrentTtlCache::new(capacity, default_ttl))
    }

    /// 创建FIFO缓存
    ///
    /// 带配置验证，确保容量参数有效
    pub fn create_fifo_cache<K, V>(capacity: usize) -> Arc<ConcurrentFifoCache<K, V>>
    where
        K: 'static + Send + Sync + Hash + Eq + Clone,
        V: 'static + Send + Sync + Clone,
    {
        Self::validate_capacity(capacity).expect("Invalid FIFO cache capacity");
        Arc::new(ConcurrentFifoCache::new(capacity))
    }

    /// 创建自适应缓存
    ///
    /// 带配置验证，确保容量参数有效
    pub fn create_adaptive_cache<K, V>(capacity: usize) -> Arc<AdaptiveCache<K, V>>
    where
        K: 'static + Send + Sync + Hash + Eq + Clone,
        V: 'static + Send + Sync + Clone,
    {
        Self::validate_capacity(capacity).expect("Invalid Adaptive cache capacity");
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

    /// 创建带统计的缓存包装器
    ///
    /// 将基础缓存包装为带统计功能的缓存
    pub fn create_stats_wrapper<K, V, C>(
        cache: Arc<C>,
    ) -> Arc<StatsCacheWrapper<K, V, C, StatsEnabled>>
    where
        K: 'static + Send + Sync + Hash + Eq + Clone,
        V: 'static + Send + Sync + Clone,
        C: Cache<K, V>,
    {
        Arc::new(StatsCacheWrapper::new_with_stats(cache))
    }

    /// 创建无统计的缓存包装器
    ///
    /// 将基础缓存包装为不带统计功能的缓存
    pub fn create_stats_wrapper_no_stats<K, V, C>(
        cache: Arc<C>,
    ) -> Arc<StatsCacheWrapper<K, V, C, StatsDisabled>>
    where
        K: 'static + Send + Sync + Hash + Eq + Clone,
        V: 'static + Send + Sync + Clone,
        C: Cache<K, V>,
    {
        Arc::new(StatsCacheWrapper::new_no_stats(cache))
    }

    /// 验证缓存容量
    ///
    /// 确保容量大于0且不超过最大限制
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
    ///
    /// 确保TTL大于0
    pub fn validate_ttl(ttl: Duration) -> Result<(), String> {
        if ttl.is_zero() {
            return Err("TTL必须大于0".to_string());
        }
        Ok(())
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
    fn test_cache_factory_validation() {
        assert!(CacheFactory::validate_capacity(100).is_ok());
        assert!(CacheFactory::validate_capacity(0).is_err());

        assert!(CacheFactory::validate_ttl(Duration::from_secs(60)).is_ok());
        assert!(CacheFactory::validate_ttl(Duration::from_secs(0)).is_err());
    }
}
