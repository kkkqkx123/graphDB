//! 自适应缓存实现

use super::super::traits::*;
use crate::cache::cache_impl::lfu::ConcurrentLfuCache;
use crate::cache::cache_impl::lru::ConcurrentLruCache;
use std::hash::Hash;
use std::sync::Arc;

#[derive(Debug, Clone)]
enum AdaptiveStrategy {
    LRU,
    LFU,
    Hybrid,
}

/// 自适应缓存
///
/// 包含LRU和LFU两种并发缓存实现，通过strategy字段动态切换
#[derive(Debug)]
pub struct AdaptiveCache<K, V> {
    lru_cache: ConcurrentLruCache<K, V>,
    lfu_cache: ConcurrentLfuCache<K, V>,
    strategy: AdaptiveStrategy,
}

/// 并发自适应缓存
///
/// 使用Arc包装AdaptiveCache，实现线程安全的并发访问
pub type ConcurrentAdaptiveCache<K, V> = Arc<AdaptiveCache<K, V>>;

impl<K, V> AdaptiveCache<K, V>
where
    K: Eq + Hash + Clone + Send + Sync,
    V: Clone + Send + Sync,
{
    pub fn new(capacity: usize) -> Self {
        Self {
            lru_cache: ConcurrentLruCache::new(capacity),
            lfu_cache: ConcurrentLfuCache::new(capacity),
            strategy: AdaptiveStrategy::LRU,
        }
    }
}

impl<K, V> Cache<K, V> for AdaptiveCache<K, V>
where
    K: Eq + Hash + Clone + Send + Sync,
    V: Clone + Send + Sync,
{
    fn get(&self, key: &K) -> Option<V> {
        match self.strategy {
            AdaptiveStrategy::LRU => self.lru_cache.get(key),
            AdaptiveStrategy::LFU => self.lfu_cache.get(key),
            AdaptiveStrategy::Hybrid => {
                // 简单的混合策略：先查LRU，再查LFU
                self.lru_cache.get(key).or_else(|| self.lfu_cache.get(key))
            }
        }
    }

    fn put(&self, key: K, value: V) {
        match self.strategy {
            AdaptiveStrategy::LRU => self.lru_cache.put(key, value),
            AdaptiveStrategy::LFU => self.lfu_cache.put(key, value),
            AdaptiveStrategy::Hybrid => {
                // 同时放入两个缓存
                self.lru_cache.put(key.clone(), value.clone());
                self.lfu_cache.put(key, value);
            }
        }
    }

    fn contains(&self, key: &K) -> bool {
        match self.strategy {
            AdaptiveStrategy::LRU => self.lru_cache.contains(key),
            AdaptiveStrategy::LFU => self.lfu_cache.contains(key),
            AdaptiveStrategy::Hybrid => {
                self.lru_cache.contains(key) || self.lfu_cache.contains(key)
            }
        }
    }

    fn remove(&self, key: &K) -> Option<V> {
        match self.strategy {
            AdaptiveStrategy::LRU => self.lru_cache.remove(key),
            AdaptiveStrategy::LFU => self.lfu_cache.remove(key),
            AdaptiveStrategy::Hybrid => self
                .lru_cache
                .remove(key)
                .or_else(|| self.lfu_cache.remove(key)),
        }
    }

    fn clear(&self) {
        self.lru_cache.clear();
        self.lfu_cache.clear();
    }

    fn len(&self) -> usize {
        match self.strategy {
            AdaptiveStrategy::LRU => self.lru_cache.len(),
            AdaptiveStrategy::LFU => self.lfu_cache.len(),
            AdaptiveStrategy::Hybrid => {
                // 返回两个缓存的最大长度
                self.lru_cache.len().max(self.lfu_cache.len())
            }
        }
    }

    fn is_empty(&self) -> bool {
        match self.strategy {
            AdaptiveStrategy::LRU => self.lru_cache.is_empty(),
            AdaptiveStrategy::LFU => self.lfu_cache.is_empty(),
            AdaptiveStrategy::Hybrid => self.lru_cache.is_empty() && self.lfu_cache.is_empty(),
        }
    }
}
