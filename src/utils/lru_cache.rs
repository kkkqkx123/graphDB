use std::collections::HashMap;
use std::hash::Hash;

/// A simple LRU cache implementation
pub struct LruCache<K, V> {
    capacity: usize,
    cache: HashMap<K, V>,
    access_order: std::collections::VecDeque<K>,
}

impl<K, V> LruCache<K, V>
where
    K: Eq + Hash + Clone,
{
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            cache: HashMap::new(),
            access_order: std::collections::VecDeque::new(),
        }
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        if self.cache.contains_key(key) {
            // Move the key to the back of the access order (most recent)
            let pos = self.access_order.iter().position(|k| k == key);
            if let Some(pos) = pos {
                if let Some(k) = self.access_order.remove(pos) {
                    self.access_order.push_back(k);
                }
            }
            self.cache.get(key)
        } else {
            None
        }
    }

    pub fn put(&mut self, key: K, value: V) {
        if self.cache.contains_key(&key) {
            // Key exists, update the access order
            let pos = self.access_order.iter().position(|k| k == &key);
            if let Some(pos) = pos {
                self.access_order.remove(pos);
            }
        } else if self.cache.len() >= self.capacity {
            // Cache is full, remove the least recently used item
            if let Some(old_key) = self.access_order.pop_front() {
                self.cache.remove(&old_key);
            }
        }

        self.cache.insert(key.clone(), value);
        self.access_order.push_back(key);
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let result = self.cache.remove(key);
        if result.is_some() {
            // Remove from access order as well
            let pos = self.access_order.iter().position(|k| k == key);
            if let Some(pos) = pos {
                self.access_order.remove(pos);
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lru_cache() {
        let mut cache = LruCache::new(2);

        cache.put(1, "one");
        cache.put(2, "two");

        assert_eq!(cache.get(&1), Some(&"one")); // Access 1
        cache.put(3, "three"); // This should evict 2

        assert_eq!(cache.get(&2), None);
        assert_eq!(cache.get(&1), Some(&"one"));
        assert_eq!(cache.get(&3), Some(&"three"));
    }
}
