//! LFU缓存实现

use crate::cache::traits::*;
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::sync::{Arc, Mutex};

/// LFU缓存实现
#[derive(Debug)]
pub struct LfuCache<K, V> {
    capacity: usize,
    cache: HashMap<K, (V, usize)>,
    frequency_order: HashMap<usize, VecDeque<K>>,
    min_frequency: usize,
}

impl<K, V> LfuCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            cache: HashMap::new(),
            frequency_order: HashMap::new(),
            min_frequency: 0,
        }
    }

    fn update_frequency(&mut self, key: &K) {
        if let Some((_, freq)) = self.cache.get_mut(key) {
            // 从旧频率列表中移除
            self.frequency_order
                .get_mut(freq)
                .expect("Frequency list should exist for this frequency")
                .retain(|k| k != key);

            // 更新频率
            *freq += 1;
            let new_freq = *freq;

            // 添加到新频率列表
            self.frequency_order
                .entry(new_freq)
                .or_insert_with(VecDeque::new)
                .push_back(key.clone());

            // 更新最小频率
            if self
                .frequency_order
                .get(&self.min_frequency)
                .expect("Frequency list should exist for min frequency")
                .is_empty()
            {
                self.min_frequency = new_freq;
            }
        }
    }

    fn evict_if_needed(&mut self) {
        if self.cache.len() >= self.capacity {
            if let Some(keys) = self.frequency_order.get_mut(&self.min_frequency) {
                if let Some(old_key) = keys.pop_front() {
                    self.cache.remove(&old_key);
                }
            }
        }
    }
}

/// 线程安全的LFU缓存
///
/// LfuCache 作为内部实现，不直接实现 Cache trait（因为 &self 方法无法
/// 提供内部可变性来更新频率计数）。仅通过 ConcurrentLfuCache 的 Mutex 包装
/// 来提供正确实现。
#[derive(Debug)]
pub struct ConcurrentLfuCache<K, V> {
    inner: Arc<Mutex<LfuCache<K, V>>>,
}

impl<K, V> ConcurrentLfuCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(LfuCache::new(capacity))),
        }
    }
}

impl<K, V> Cache<K, V> for ConcurrentLfuCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    fn get(&self, key: &K) -> Option<V> {
        let mut cache = self
            .inner
            .lock()
            .expect("ConcurrentLfuCache lock should not be poisoned");
        if let Some((value, _)) = cache.cache.get(key).cloned() {
            cache.update_frequency(key);
            Some(value)
        } else {
            None
        }
    }

    fn put(&self, key: K, value: V) {
        let mut cache = self
            .inner
            .lock()
            .expect("ConcurrentLfuCache lock should not be poisoned");

        if cache.cache.contains_key(&key) {
            // 更新现有条目
            cache.cache.insert(key.clone(), (value, 1));
            cache.update_frequency(&key);
        } else {
            // 添加新条目
            cache.evict_if_needed();
            cache.cache.insert(key.clone(), (value, 1));

            // 添加到频率列表
            cache
                .frequency_order
                .entry(1)
                .or_insert_with(VecDeque::new)
                .push_back(key.clone());
            cache.min_frequency = 1;
        }
    }

    fn contains(&self, key: &K) -> bool {
        let cache = self
            .inner
            .lock()
            .expect("ConcurrentLfuCache lock should not be poisoned");
        cache.cache.contains_key(key)
    }

    fn remove(&self, key: &K) -> Option<V> {
        let mut cache = self
            .inner
            .lock()
            .expect("ConcurrentLfuCache lock should not be poisoned");
        if let Some((value, freq)) = cache.cache.remove(key) {
            // 从频率列表中移除
            if let Some(keys) = cache.frequency_order.get_mut(&freq) {
                keys.retain(|k| k != key);
            }
            Some(value)
        } else {
            None
        }
    }

    fn clear(&self) {
        let mut cache = self
            .inner
            .lock()
            .expect("ConcurrentLfuCache lock should not be poisoned");
        cache.cache.clear();
        cache.frequency_order.clear();
        cache.min_frequency = 0;
    }

    fn len(&self) -> usize {
        let cache = self
            .inner
            .lock()
            .expect("ConcurrentLfuCache lock should not be poisoned");
        cache.cache.len()
    }

    fn is_empty(&self) -> bool {
        let cache = self
            .inner
            .lock()
            .expect("ConcurrentLfuCache lock should not be poisoned");
        cache.cache.is_empty()
    }
}
