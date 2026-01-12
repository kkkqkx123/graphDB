//! 无界缓存实现

use crate::cache::traits::*;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

/// 无界缓存
#[derive(Debug)]
pub struct UnboundedCache<K, V> {
    cache: HashMap<K, V>,
}

impl<K, V> UnboundedCache<K, V>
where
    K: Eq + Hash,
{
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }
}

/// 线程安全的无界缓存
///
/// UnboundedCache 作为内部实现，不直接实现 Cache trait（因为 &self 方法无法
/// 提供内部可变性来执行写操作）。仅通过 ConcurrentUnboundedCache 的 Mutex 包装
/// 来提供正确实现。
#[derive(Debug)]
pub struct ConcurrentUnboundedCache<K, V> {
    inner: Arc<Mutex<UnboundedCache<K, V>>>,
}

impl<K, V> ConcurrentUnboundedCache<K, V>
where
    K: Eq + Hash,
    V: Clone,
{
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(UnboundedCache::new())),
        }
    }
}

impl<K, V> Cache<K, V> for ConcurrentUnboundedCache<K, V>
where
    K: Eq + Hash,
    V: Clone,
{
    fn get(&self, key: &K) -> Option<V> {
        let cache = self
            .inner
            .lock()
            .expect("ConcurrentUnboundedCache lock should not be poisoned");
        cache.cache.get(key).cloned()
    }

    fn put(&self, key: K, value: V) {
        let mut cache = self
            .inner
            .lock()
            .expect("ConcurrentUnboundedCache lock should not be poisoned");
        cache.cache.insert(key, value);
    }

    fn contains(&self, key: &K) -> bool {
        let cache = self
            .inner
            .lock()
            .expect("ConcurrentUnboundedCache lock should not be poisoned");
        cache.cache.contains_key(key)
    }

    fn remove(&self, key: &K) -> Option<V> {
        let mut cache = self
            .inner
            .lock()
            .expect("ConcurrentUnboundedCache lock should not be poisoned");
        cache.cache.remove(key)
    }

    fn clear(&self) {
        let mut cache = self
            .inner
            .lock()
            .expect("ConcurrentUnboundedCache lock should not be poisoned");
        cache.cache.clear();
    }

    fn len(&self) -> usize {
        let cache = self
            .inner
            .lock()
            .expect("ConcurrentUnboundedCache lock should not be poisoned");
        cache.cache.len()
    }

    fn is_empty(&self) -> bool {
        let cache = self
            .inner
            .lock()
            .expect("ConcurrentUnboundedCache lock should not be poisoned");
        cache.cache.is_empty()
    }
}
