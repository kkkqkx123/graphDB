use std::collections::HashMap;
use std::hash::Hash;
use std::time::{SystemTime, UNIX_EPOCH};

/// A simple object pool for reusing objects to reduce allocation overhead
pub struct ObjectPool<T> {
    pool: Vec<T>,
    max_size: usize,
}

impl<T: Default + Clone> ObjectPool<T> {
    pub fn new(max_size: usize) -> Self {
        Self {
            pool: Vec::new(),
            max_size,
        }
    }

    pub fn get(&mut self) -> T {
        self.pool.pop().unwrap_or_default()
    }

    pub fn put(&mut self, obj: T) {
        if self.pool.len() < self.max_size {
            self.pool.push(obj);
        }
    }
}

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
                let k = self.access_order.remove(pos).unwrap();
                self.access_order.push_back(k);
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

/// Utility function to generate unique IDs
pub fn generate_id() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_nanos() as u64
}

/// Utility function for validating node/edge IDs
pub fn is_valid_id(id: u64) -> bool {
    id != 0
}

/// A simple logger wrapper for consistent logging format
pub struct Logger;

impl Logger {
    pub fn info(message: &str) {
        println!("[INFO] {}", message);
    }

    pub fn warn(message: &str) {
        eprintln!("[WARN] {}", message);
    }

    pub fn error(message: &str) {
        eprintln!("[ERROR] {}", message);
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
    
    #[test]
    fn test_object_pool() {
        let mut pool: ObjectPool<Vec<i32>> = ObjectPool::new(10);
        
        let mut obj = pool.get();
        obj.push(42);
        
        assert_eq!(obj, vec![42]);
        
        pool.put(obj);
        
        let obj2 = pool.get();
        assert_eq!(obj2, vec![42]); // Should reuse the same object
    }
}