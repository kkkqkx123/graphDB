//! 统计缓存包装器实现
//!
//! 使用泛型参数在编译时决定是否启用统计功能
//! 编译器会特化此实现，消除所有条件分支

use crate::cache::stats_collector::CacheStats;
use crate::cache::stats_marker::{StatsDisabled, StatsEnabled, StatsMode};
use crate::cache::traits::*;
use std::sync::{Arc, RwLock};

/// 自适应统计缓存包装器
///
/// 使用泛型参数 `S: StatsMode` 在编译时决定是否启用统计。
/// 编译器会为 `WithStats` 和 `NoStats` 版本分别生成优化的代码。
#[derive(Debug)]
pub struct StatsCacheWrapper<K, V, C, S: StatsMode = StatsDisabled> {
    inner: Arc<C>,
    /// 统计信息 - 编译时对 StatsDisabled 版本消除
    stats: Option<Arc<RwLock<CacheStats>>>,
    _marker: std::marker::PhantomData<(K, V, S)>,
}

/// 禁用统计的包装器实现
impl<K, V, C> StatsCacheWrapper<K, V, C, StatsDisabled>
where
    C: Cache<K, V>,
{
    /// 创建无统计开销的包装器
    pub fn new_no_stats(cache: Arc<C>) -> Self {
        Self {
            inner: cache,
            stats: None, // 编译时消除
            _marker: std::marker::PhantomData,
        }
    }
}

/// 启用统计的包装器实现
impl<K, V, C> StatsCacheWrapper<K, V, C, StatsEnabled>
where
    C: Cache<K, V>,
{
    /// 创建带统计功能的包装器
    pub fn new_with_stats(cache: Arc<C>) -> Self {
        Self {
            inner: cache,
            stats: Some(Arc::new(RwLock::new(CacheStats::new()))),
            _marker: std::marker::PhantomData,
        }
    }

    /// 获取缓存统计信息
    pub fn get_cache_stats(&self) -> CacheStats {
        self.stats
            .as_ref()
            .expect("Stats should exist for StatsEnabled variant")
            .read()
            .expect("StatsCacheWrapper stats lock should not be poisoned")
            .clone()
    }
}

/// 通用构造函数 - 用于向后兼容
impl<K, V, C> StatsCacheWrapper<K, V, C>
where
    C: Cache<K, V>,
{
    /// 创建带统计的包装器（默认）
    pub fn new(cache: Arc<C>) -> StatsCacheWrapper<K, V, C, StatsEnabled> {
        StatsCacheWrapper::new_with_stats(cache)
    }
}

/// 禁用统计模式的 Cache 实现
///
/// 编译器会优化掉所有统计逻辑，完全不产生开销
impl<K, V, C> Cache<K, V> for StatsCacheWrapper<K, V, C, StatsDisabled>
where
    K: 'static + Send + Sync,
    V: 'static + Send + Sync + Clone,
    C: Cache<K, V>,
{
    #[inline]
    fn get(&self, key: &K) -> Option<V> {
        // 直接委托，无统计
        self.inner.get(key)
    }

    #[inline]
    fn put(&self, key: K, value: V) {
        // 直接委托，无统计
        self.inner.put(key, value)
    }

    #[inline]
    fn contains(&self, key: &K) -> bool {
        self.inner.contains(key)
    }

    #[inline]
    fn remove(&self, key: &K) -> Option<V> {
        self.inner.remove(key)
    }

    #[inline]
    fn clear(&self) {
        self.inner.clear()
    }

    #[inline]
    fn len(&self) -> usize {
        self.inner.len()
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

/// 启用统计模式的 Cache 实现
///
/// 记录命中、未命中和操作计数
impl<K, V, C> Cache<K, V> for StatsCacheWrapper<K, V, C, StatsEnabled>
where
    K: 'static + Send + Sync,
    V: 'static + Send + Sync + Clone,
    C: Cache<K, V>,
{
    fn get(&self, key: &K) -> Option<V> {
        let result = self.inner.get(key);

        if let Some(ref stats) = self.stats {
            let mut s = stats
                .write()
                .expect("StatsCacheWrapper stats lock should not be poisoned");
            s.total_operations += 1;

            if result.is_some() {
                s.total_hits += 1;
            } else {
                s.total_misses += 1;
            }
        }

        result
    }

    fn put(&self, key: K, value: V) {
        self.inner.put(key, value);

        if let Some(ref stats) = self.stats {
            let mut s = stats
                .write()
                .expect("StatsCacheWrapper stats lock should not be poisoned");
            s.total_operations += 1;
        }
    }

    fn contains(&self, key: &K) -> bool {
        self.inner.contains(key)
    }

    fn remove(&self, key: &K) -> Option<V> {
        self.inner.remove(key)
    }

    fn clear(&self) {
        self.inner.clear();

        if let Some(ref stats) = self.stats {
            let mut s = stats
                .write()
                .expect("StatsCacheWrapper stats lock should not be poisoned");
            s.reset();
        }
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

/// 仅为启用统计模式实现 StatsCache trait
impl<K, V, C> StatsCache<K, V> for StatsCacheWrapper<K, V, C, StatsEnabled>
where
    K: 'static + Send + Sync,
    V: 'static + Send + Sync + Clone,
    C: Cache<K, V>,
{
    fn hits(&self) -> u64 {
        self.stats
            .as_ref()
            .expect("Stats should exist for StatsEnabled variant")
            .read()
            .expect("StatsCacheWrapper stats lock should not be poisoned")
            .total_hits
    }

    fn misses(&self) -> u64 {
        self.stats
            .as_ref()
            .expect("Stats should exist for StatsEnabled variant")
            .read()
            .expect("StatsCacheWrapper stats lock should not be poisoned")
            .total_misses
    }

    fn hit_rate(&self) -> f64 {
        let stats = self
            .stats
            .as_ref()
            .expect("Stats should exist for StatsEnabled variant")
            .read()
            .expect("StatsCacheWrapper stats lock should not be poisoned");
        stats.hit_rate()
    }

    fn evictions(&self) -> u64 {
        self.stats
            .as_ref()
            .expect("Stats should exist for StatsEnabled variant")
            .read()
            .expect("StatsCacheWrapper stats lock should not be poisoned")
            .total_evictions
    }

    fn reset_stats(&self) {
        let mut stats = self
            .stats
            .as_ref()
            .expect("Stats should exist for StatsEnabled variant")
            .write()
            .expect("StatsCacheWrapper stats lock should not be poisoned");
        stats.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // 简单的测试缓存实现
    struct TestCache<K, V> {
        data: Arc<Mutex<std::collections::HashMap<K, V>>>,
    }

    impl<K, V> TestCache<K, V>
    where
        K: Clone + Eq + std::hash::Hash,
    {
        fn new() -> Self {
            Self {
                data: Arc::new(Mutex::new(std::collections::HashMap::new())),
            }
        }
    }

    impl<K, V> Cache<K, V> for TestCache<K, V>
    where
        K: Clone + Eq + std::hash::Hash,
        V: Clone,
    {
        fn get(&self, key: &K) -> Option<V> {
            self.data
                .lock()
                .expect("Data lock was poisoned")
                .get(key)
                .cloned()
        }

        fn put(&self, key: K, value: V) {
            self.data
                .lock()
                .expect("Data lock was poisoned")
                .insert(key, value);
        }

        fn contains(&self, key: &K) -> bool {
            self.data
                .lock()
                .expect("Data lock was poisoned")
                .contains_key(key)
        }

        fn remove(&self, key: &K) -> Option<V> {
            self.data
                .lock()
                .expect("Data lock was poisoned")
                .remove(key)
        }

        fn clear(&self) {
            self.data.lock().expect("Data lock was poisoned").clear();
        }

        fn len(&self) -> usize {
            self.data.lock().expect("Data lock was poisoned").len()
        }

        fn is_empty(&self) -> bool {
            self.data.lock().expect("Data lock was poisoned").is_empty()
        }
    }

    #[test]
    fn test_stats_enabled_wrapper() {
        let test_cache = Arc::new(TestCache::new());
        let wrapper = StatsCacheWrapper::new_with_stats(test_cache);

        assert_eq!(wrapper.hits(), 0);
        assert_eq!(wrapper.misses(), 0);

        wrapper.put("key1".to_string(), "value1".to_string());
        assert_eq!(wrapper.len(), 1);

        let _ = wrapper.get(&"key1".to_string());
        assert_eq!(wrapper.hits(), 1);

        let _ = wrapper.get(&"key2".to_string());
        assert_eq!(wrapper.misses(), 1);
    }

    #[test]
    fn test_stats_disabled_wrapper() {
        let test_cache = Arc::new(TestCache::new());
        let wrapper = StatsCacheWrapper::new_no_stats(test_cache);

        wrapper.put("key1".to_string(), "value1".to_string());
        assert_eq!(wrapper.len(), 1);

        let value = wrapper.get(&"key1".to_string());
        assert_eq!(value, Some("value1".to_string()));

        wrapper.remove(&"key1".to_string());
        assert!(wrapper.is_empty());
    }

    #[test]
    fn test_stats_wrapper_hit_rate() {
        let test_cache = Arc::new(TestCache::new());
        let wrapper = StatsCacheWrapper::new_with_stats(test_cache);

        wrapper.put("key1".to_string(), "value1".to_string());
        wrapper.put("key2".to_string(), "value2".to_string());

        let _ = wrapper.get(&"key1".to_string()); // hit
        let _ = wrapper.get(&"key2".to_string()); // hit
        let _ = wrapper.get(&"key3".to_string()); // miss

        assert_eq!(wrapper.hits(), 2);
        assert_eq!(wrapper.misses(), 1);
        assert_eq!(wrapper.hit_rate(), 2.0 / 3.0);
    }

    #[test]
    fn test_stats_wrapper_clear() {
        let test_cache = Arc::new(TestCache::new());
        let wrapper = StatsCacheWrapper::new_with_stats(test_cache);

        wrapper.put("key1".to_string(), "value1".to_string());
        assert_eq!(wrapper.len(), 1);

        wrapper.clear();
        assert!(wrapper.is_empty());
    }
}
