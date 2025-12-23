//! TTL缓存实现

use crate::cache::traits::*;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// TTL缓存条目
#[derive(Debug, Clone)]
pub struct TtlEntry<V> {
    value: V,
    created_at: Instant,
    ttl: Duration,
}

impl<V> TtlEntry<V> {
    pub fn new(value: V, ttl: Duration) -> Self {
        Self {
            value,
            created_at: Instant::now(),
            ttl,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }

    pub fn value(&self) -> &V {
        &self.value
    }
}

/// TTL缓存实现
#[derive(Debug)]
pub struct TtlCache<K, V> {
    capacity: usize,
    default_ttl: Duration,
    cache: HashMap<K, TtlEntry<V>>,
}

impl<K, V> TtlCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    pub fn new(capacity: usize, default_ttl: Duration) -> Self {
        Self {
            capacity,
            default_ttl,
            cache: HashMap::new(),
        }
    }

    fn cleanup_expired(&mut self) {
        self.cache.retain(|_, entry| !entry.is_expired());
    }

    fn evict_if_needed(&mut self) {
        if self.cache.len() >= self.capacity {
            // 简单的FIFO驱逐策略
            if let Some(key) = self.cache.keys().next().cloned() {
                self.cache.remove(&key);
            }
        }
    }
}

/// 线程安全的TTL缓存
///
/// TtlCache 作为内部实现，不直接实现 Cache trait（因为 &self 方法无法
/// 提供内部可变性来执行过期清理）。仅通过 ConcurrentTtlCache 的 Mutex 包装
/// 来提供正确实现。
#[derive(Debug)]
pub struct ConcurrentTtlCache<K, V> {
    inner: Arc<Mutex<TtlCache<K, V>>>,
}

impl<K, V> ConcurrentTtlCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    pub fn new(capacity: usize, default_ttl: Duration) -> Self {
        Self {
            inner: Arc::new(Mutex::new(TtlCache::new(capacity, default_ttl))),
        }
    }
}

impl<K, V> Cache<K, V> for ConcurrentTtlCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    fn get(&self, key: &K) -> Option<V> {
        let mut cache = self
            .inner
            .lock()
            .expect("ConcurrentTtlCache lock should not be poisoned");
        cache.cleanup_expired();

        let result = cache.cache.get(key).and_then(|entry| {
            if !entry.is_expired() {
                Some(entry.value().clone())
            } else {
                None
            }
        });
        result
    }

    fn put(&self, key: K, value: V) {
        let mut cache = self
            .inner
            .lock()
            .expect("ConcurrentTtlCache lock should not be poisoned");
        cache.cleanup_expired();
        cache.evict_if_needed();

        let entry = TtlEntry::new(value, cache.default_ttl);
        cache.cache.insert(key, entry);
    }

    fn contains(&self, key: &K) -> bool {
        let mut cache = self
            .inner
            .lock()
            .expect("ConcurrentTtlCache lock should not be poisoned");
        cache.cleanup_expired();

        if let Some(entry) = cache.cache.get(key) {
            !entry.is_expired()
        } else {
            false
        }
    }

    fn remove(&self, key: &K) -> Option<V> {
        let mut cache = self
            .inner
            .lock()
            .expect("ConcurrentTtlCache lock should not be poisoned");
        cache.cleanup_expired();

        if let Some(entry) = cache.cache.remove(key) {
            Some(entry.value().clone())
        } else {
            None
        }
    }

    fn clear(&self) {
        let mut cache = self
            .inner
            .lock()
            .expect("ConcurrentTtlCache lock should not be poisoned");
        cache.cache.clear();
    }

    fn len(&self) -> usize {
        let mut cache = self
            .inner
            .lock()
            .expect("ConcurrentTtlCache lock should not be poisoned");
        cache.cleanup_expired();
        cache.cache.len()
    }

    fn is_empty(&self) -> bool {
        let mut cache = self
            .inner
            .lock()
            .expect("ConcurrentTtlCache lock should not be poisoned");
        cache.cleanup_expired();
        cache.cache.is_empty()
    }
}
