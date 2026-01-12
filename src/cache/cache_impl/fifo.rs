//! FIFO缓存实现

use crate::cache::traits::*;
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::sync::{Arc, Mutex};

/// FIFO缓存实现
#[derive(Debug)]
pub struct FifoCache<K, V> {
    capacity: usize,
    cache: HashMap<K, V>,
    order: VecDeque<K>,
}

impl<K, V> FifoCache<K, V>
where
    K: Eq + Hash + Clone,
{
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            cache: HashMap::new(),
            order: VecDeque::new(),
        }
    }

    fn evict_if_needed(&mut self) {
        if self.cache.len() >= self.capacity {
            if let Some(old_key) = self.order.pop_front() {
                self.cache.remove(&old_key);
            }
        }
    }
}

/// 线程安全的FIFO缓存
///
/// FifoCache 作为内部实现，不直接实现 Cache trait（因为 &self 方法无法
/// 提供内部可变性）。仅通过 ConcurrentFifoCache 的 Mutex 包装来提供正确实现。
#[derive(Debug)]
pub struct ConcurrentFifoCache<K, V> {
    inner: Arc<Mutex<FifoCache<K, V>>>,
}

impl<K, V> ConcurrentFifoCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(FifoCache::new(capacity))),
        }
    }
}

impl<K, V> Cache<K, V> for ConcurrentFifoCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    fn get(&self, key: &K) -> Option<V> {
        let cache = self
            .inner
            .lock()
            .expect("ConcurrentFifoCache lock should not be poisoned");
        cache.cache.get(key).cloned()
    }

    fn put(&self, key: K, value: V) {
        let mut cache = self
            .inner
            .lock()
            .expect("ConcurrentFifoCache lock should not be poisoned");

        if !cache.cache.contains_key(&key) {
            cache.evict_if_needed();
            cache.order.push_back(key.clone());
        }

        cache.cache.insert(key, value);
    }

    fn contains(&self, key: &K) -> bool {
        let cache = self
            .inner
            .lock()
            .expect("ConcurrentFifoCache lock should not be poisoned");
        cache.cache.contains_key(key)
    }

    fn remove(&self, key: &K) -> Option<V> {
        let mut cache = self
            .inner
            .lock()
            .expect("ConcurrentFifoCache lock should not be poisoned");
        let result = cache.cache.remove(key);
        if result.is_some() {
            cache.order.retain(|k| k != key);
        }
        result
    }

    fn clear(&self) {
        let mut cache = self
            .inner
            .lock()
            .expect("ConcurrentFifoCache lock should not be poisoned");
        cache.cache.clear();
        cache.order.clear();
    }

    fn len(&self) -> usize {
        let cache = self
            .inner
            .lock()
            .expect("ConcurrentFifoCache lock should not be poisoned");
        cache.cache.len()
    }

    fn is_empty(&self) -> bool {
        let cache = self
            .inner
            .lock()
            .expect("ConcurrentFifoCache lock should not be poisoned");
        cache.cache.is_empty()
    }
}
