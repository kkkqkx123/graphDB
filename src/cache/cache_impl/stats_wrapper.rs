//! 统计缓存包装器实现

use crate::cache::stats_collector::CacheStats;
use crate::cache::traits::*;
use std::sync::{Arc, RwLock};

/// 统计缓存包装器 - 泛型版本
#[derive(Debug)]
pub struct StatsCacheWrapper<K, V, C> {
    inner: Arc<C>,
    stats: Arc<RwLock<CacheStats>>,
    _marker: std::marker::PhantomData<(K, V)>,
}

impl<K, V, C> StatsCacheWrapper<K, V, C>
where
    C: Cache<K, V>,
{
    pub fn new(cache: Arc<C>) -> Self {
        Self {
            inner: cache,
            stats: Arc::new(RwLock::new(CacheStats::new())),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn get_cache_stats(&self) -> CacheStats {
        self.stats
            .read()
            .expect("StatsCacheWrapper stats lock should not be poisoned")
            .clone()
    }
}

impl<K, V, C> Cache<K, V> for StatsCacheWrapper<K, V, C>
where
    K: 'static + Send + Sync,
    V: 'static + Send + Sync + Clone,
    C: Cache<K, V>,
{
    fn get(&self, key: &K) -> Option<V> {
        let result = self.inner.get(key);

        let mut stats = self
            .stats
            .write()
            .expect("StatsCacheWrapper stats lock should not be poisoned");
        stats.total_operations += 1;

        if result.is_some() {
            stats.total_hits += 1;
        } else {
            stats.total_misses += 1;
        }

        result
    }

    fn put(&self, key: K, value: V) {
        self.inner.put(key, value);

        let mut stats = self
            .stats
            .write()
            .expect("StatsCacheWrapper stats lock should not be poisoned");
        stats.total_operations += 1;
    }

    fn contains(&self, key: &K) -> bool {
        self.inner.contains(key)
    }

    fn remove(&self, key: &K) -> Option<V> {
        self.inner.remove(key)
    }

    fn clear(&self) {
        self.inner.clear();

        let mut stats = self
            .stats
            .write()
            .expect("StatsCacheWrapper stats lock should not be poisoned");
        stats.reset();
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl<K, V, C> StatsCache<K, V> for StatsCacheWrapper<K, V, C>
where
    K: 'static + Send + Sync,
    V: 'static + Send + Sync + Clone,
    C: Cache<K, V>,
{
    fn hits(&self) -> u64 {
        self.stats
            .read()
            .expect("StatsCacheWrapper stats lock should not be poisoned")
            .total_hits
    }

    fn misses(&self) -> u64 {
        self.stats
            .read()
            .expect("StatsCacheWrapper stats lock should not be poisoned")
            .total_misses
    }

    fn hit_rate(&self) -> f64 {
        let stats = self
            .stats
            .read()
            .expect("StatsCacheWrapper stats lock should not be poisoned");
        stats.hit_rate()
    }

    fn evictions(&self) -> u64 {
        self.stats
            .read()
            .expect("StatsCacheWrapper stats lock should not be poisoned")
            .total_evictions
    }

    fn reset_stats(&self) {
        let mut stats = self
            .stats
            .write()
            .expect("StatsCacheWrapper stats lock should not be poisoned");
        stats.reset();
    }
}
