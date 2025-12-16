//! LRU缓存实现

use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use crate::cache::traits::*;

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
            let key = self.access_order.remove(pos).unwrap();
            self.access_order.push_back(key);
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

impl<K, V> Cache<K, V> for LruCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    fn get(&self, key: &K) -> Option<V> {
        self.cache.get(key).cloned()
    }
    
    fn put(&self, key: K, value: V) {
        // LRU缓存需要可变引用，这里使用内部可变性
        // 实际实现中应该使用Mutex或RwLock包装
        unimplemented!("LRU缓存需要内部可变性支持")
    }
    
    fn contains(&self, key: &K) -> bool {
        self.cache.contains_key(key)
    }
    
    fn remove(&self, key: &K) -> Option<V> {
        self.cache.remove(key).cloned()
    }
    
    fn clear(&self) {
        // 需要可变引用
        unimplemented!("LRU缓存需要内部可变性支持")
    }
    
    fn len(&self) -> usize {
        self.cache.len()
    }
    
    fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

/// 线程安全的LRU缓存
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
        let mut cache = self.inner.lock().unwrap();
        if let Some(value) = cache.cache.get(key) {
            cache.move_to_back(key);
            Some(value.clone())
        } else {
            None
        }
    }
    
    fn put(&self, key: K, value: V) {
        let mut cache = self.inner.lock().unwrap();
        if cache.cache.contains_key(&key) {
            cache.move_to_back(&key);
        } else {
            cache.evict_if_needed();
        }
        cache.cache.insert(key.clone(), value);
        cache.access_order.push_back(key);
    }
    
    fn contains(&self, key: &K) -> bool {
        let cache = self.inner.lock().unwrap();
        cache.cache.contains_key(key)
    }
    
    fn remove(&self, key: &K) -> Option<V> {
        let mut cache = self.inner.lock().unwrap();
        let result = cache.cache.remove(key);
        if result.is_some() {
            cache.access_order.retain(|k| k != key);
        }
        result
    }
    
    fn clear(&self) {
        let mut cache = self.inner.lock().unwrap();
        cache.cache.clear();
        cache.access_order.clear();
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