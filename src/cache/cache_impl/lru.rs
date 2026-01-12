//! LRU缓存实现

use crate::cache::traits::*;
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::sync::{Arc, Mutex};

/// LRU缓存实现
#[derive(Debug)]
pub struct LruCache<K, V> {
    capacity: usize,
    cache: HashMap<K, V>,
    access_order: VecDeque<K>,
}

impl<K, V> LruCache<K, V>
where
    K: Eq + Hash + Clone,
{
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            cache: HashMap::new(),
            access_order: VecDeque::new(),
        }
    }

    fn move_to_back(&mut self, key: &K) {
        if let Some(pos) = self.access_order.iter().position(|k| k == key) {
            if let Some(found_key) = self.access_order.remove(pos) {
                self.access_order.push_back(found_key);
            }
        }
    }

    fn evict_if_needed(&mut self) {
        if self.cache.len() >= self.capacity {
            if let Some(old_key) = self.access_order.pop_front() {
                self.cache.remove(&old_key);
            }
        }
    }
}

/// 线程安全的LRU缓存
///
/// LruCache 作为内部实现，不直接实现 Cache trait（因为 &self 方法无法
/// 提供内部可变性来更新访问顺序）。仅通过 ConcurrentLruCache 的 Mutex 包装
/// 来提供正确实现。
#[derive(Debug)]
pub struct ConcurrentLruCache<K, V> {
    inner: Arc<Mutex<LruCache<K, V>>>,
}

impl<K, V> ConcurrentLruCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(LruCache::new(capacity))),
        }
    }
}

impl<K, V> Cache<K, V> for ConcurrentLruCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    fn get(&self, key: &K) -> Option<V> {
        let mut cache = self
            .inner
            .lock()
            .expect("LRU cache inner mutex was poisoned");
        let value = cache.cache.get(key).cloned();
        if value.is_some() {
            cache.move_to_back(key);
        }
        value
    }

    fn put(&self, key: K, value: V) {
        let mut cache = self
            .inner
            .lock()
            .expect("LRU cache inner mutex was poisoned");
        if cache.cache.contains_key(&key) {
            cache.move_to_back(&key);
        } else {
            cache.evict_if_needed();
        }
        cache.cache.insert(key.clone(), value);
        cache.access_order.push_back(key);
    }

    fn contains(&self, key: &K) -> bool {
        let cache = self
            .inner
            .lock()
            .expect("LRU cache inner mutex was poisoned");
        cache.cache.contains_key(key)
    }

    fn remove(&self, key: &K) -> Option<V> {
        let mut cache = self
            .inner
            .lock()
            .expect("LRU cache inner mutex was poisoned");
        let result = cache.cache.remove(key);
        if result.is_some() {
            cache.access_order.retain(|k| k != key);
        }
        result
    }

    fn clear(&self) {
        let mut cache = self
            .inner
            .lock()
            .expect("LRU cache inner mutex was poisoned");
        cache.cache.clear();
        cache.access_order.clear();
    }

    fn len(&self) -> usize {
        let cache = self
            .inner
            .lock()
            .expect("LRU cache inner mutex was poisoned");
        cache.cache.len()
    }

    fn is_empty(&self) -> bool {
        let cache = self
            .inner
            .lock()
            .expect("LRU cache inner mutex was poisoned");
        cache.cache.is_empty()
    }
}
