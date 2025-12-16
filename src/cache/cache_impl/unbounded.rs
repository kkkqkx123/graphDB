//! 无界缓存实现

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use crate::cache::traits::*;

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

impl<K, V> Cache<K, V> for UnboundedCache<K, V>
where
    K: Eq + Hash,
    V: Clone,
{
    fn get(&self, key: &K) -> Option<V> {
        self.cache.get(key).cloned()
    }
    
    fn put(&self, key: K, value: V) {
        unimplemented!("无界缓存需要内部可变性支持")
    }
    
    fn contains(&self, key: &K) -> bool {
        self.cache.contains_key(key)
    }
    
    fn remove(&self, key: &K) -> Option<V> {
        unimplemented!("无界缓存需要内部可变性支持")
    }
    
    fn clear(&self) {
        unimplemented!("无界缓存需要内部可变性支持")
    }
    
    fn len(&self) -> usize {
        self.cache.len()
    }
    
    fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

/// 线程安全的无界缓存
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
        let cache = self.inner.lock().unwrap();
        cache.cache.get(key).cloned()
    }
    
    fn put(&self, key: K, value: V) {
        let mut cache = self.inner.lock().unwrap();
        cache.cache.insert(key, value);
    }
    
    fn contains(&self, key: &K) -> bool {
        let cache = self.inner.lock().unwrap();
        cache.cache.contains_key(key)
    }
    
    fn remove(&self, key: &K) -> Option<V> {
        let mut cache = self.inner.lock().unwrap();
        cache.cache.remove(key)
    }
    
    fn clear(&self) {
        let mut cache = self.inner.lock().unwrap();
        cache.cache.clear();
    }
    
    fn len(&self) -> usize {
        let cache = self.inner.lock().unwrap();
        cache.cache.len()
    }
    
    fn is_empty(&self) -> bool {
        let cache = self.inner.lock().unwrap();
        cache.cache.is_empty()
    }
}