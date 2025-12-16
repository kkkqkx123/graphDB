//! 统计缓存包装器实现

use std::sync::{Arc, RwLock};
use crate::cache::traits::*;
use crate::cache::manager::CacheStats;

/// 统计缓存包装器
#[derive(Debug)]
pub struct StatsCacheWrapper<K, V> {
    inner: Arc<dyn Cache<K, V>>,
    stats: Arc<RwLock<CacheStats>>,
}

impl<K, V> StatsCacheWrapper<K, V> {
    pub fn new(cache: Arc<dyn Cache<K, V>>) -> Self {
        Self {
            inner: cache,
            stats: Arc::new(RwLock::new(CacheStats::new())),
        }
    }
    
    pub fn get_cache_stats(&self) -> CacheStats {
        self.stats.read().unwrap().clone()
    }
}

impl<K, V> Cache<K, V> for StatsCacheWrapper<K, V>
where
    K: 'static + Send + Sync,
    V: 'static + Send + Sync + Clone,
{
    fn get(&self, key: &K) -> Option<V> {
        let result = self.inner.get(key);
        
        let mut stats = self.stats.write().unwrap();
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
        
        let mut stats = self.stats.write().unwrap();
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
        
        let mut stats = self.stats.write().unwrap();
        stats.reset();
    }
    
    fn len(&self) -> usize {
        self.inner.len()
    }
    
    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl<K, V> StatsCache<K, V> for StatsCacheWrapper<K, V>
where
    K: 'static + Send + Sync,
    V: 'static + Send + Sync + Clone,
{
    fn hits(&self) -> u64 {
        self.stats.read().unwrap().total_hits
    }
    
    fn misses(&self) -> u64 {
        self.stats.read().unwrap().total_misses
    }
    
    fn hit_rate(&self) -> f64 {
        let stats = self.stats.read().unwrap();
        stats.hit_rate()
    }
    
    fn evictions(&self) -> u64 {
        self.stats.read().unwrap().total_evictions
    }
    
    fn reset_stats(&self) {
        let mut stats = self.stats.write().unwrap();
        stats.reset();
    }
}