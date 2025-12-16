//! TTL缓存实现

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use crate::cache::traits::*;

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
            if let Some(key) = self.cache.keys().next() {
                self.cache.remove(key);
            }
        }
    }
}

impl<K, V> Cache<K, V> for TtlCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    fn get(&self, key: &K) -> Option<V> {
        if let Some(entry) = self.cache.get(key) {
            if !entry.is_expired() {
                Some(entry.value().clone())
            } else {
                None
            }
        } else {
            None
        }
    }
    
    fn put(&self, key: K, value: V) {
        unimplemented!("TTL缓存需要内部可变性支持")
    }
    
    fn contains(&self, key: &K) -> bool {
        if let Some(entry) = self.cache.get(key) {
            !entry.is_expired()
        } else {
            false
        }
    }
    
    fn remove(&self, key: &K) -> Option<V> {
        unimplemented!("TTL缓存需要内部可变性支持")
    }
    
    fn clear(&self) {
        unimplemented!("TTL缓存需要内部可变性支持")
    }
    
    fn len(&self) -> usize {
        self.cache.len()
    }
    
    fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

/// 线程安全的TTL缓存
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
        let mut cache = self.inner.lock().unwrap();
        cache.cleanup_expired();
        
        if let Some(entry) = cache.cache.get(key) {
            if !entry.is_expired() {
                Some(entry.value().clone())
            } else {
                None
            }
        } else {
            None
        }
    }
    
    fn put(&self, key: K, value: V) {
        let mut cache = self.inner.lock().unwrap();
        cache.cleanup_expired();
        cache.evict_if_needed();
        
        let entry = TtlEntry::new(value, cache.default_ttl);
        cache.cache.insert(key, entry);
    }
    
    fn contains(&self, key: &K) -> bool {
        let mut cache = self.inner.lock().unwrap();
        cache.cleanup_expired();
        
        if let Some(entry) = cache.cache.get(key) {
            !entry.is_expired()
        } else {
            false
        }
    }
    
    fn remove(&self, key: &K) -> Option<V> {
        let mut cache = self.inner.lock().unwrap();
        cache.cleanup_expired();
        
        if let Some(entry) = cache.cache.remove(key) {
            Some(entry.value().clone())
        } else {
            None
        }
    }
    
    fn clear(&self) {
        let mut cache = self.inner.lock().unwrap();
        cache.cache.clear();
    }
    
    fn len(&self) -> usize {
        let mut cache = self.inner.lock().unwrap();
        cache.cleanup_expired();
        cache.cache.len()
    }
    
    fn is_empty(&self) -> bool {
        let mut cache = self.inner.lock().unwrap();
        cache.cleanup_expired();
        cache.cache.is_empty()
    }
}