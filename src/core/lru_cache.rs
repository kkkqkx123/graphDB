//! Concurrent LRU Cache implementation
//!
//! This module provides a thread-safe LRU cache similar to NebulaGraph's ConcurrentLRUCache

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use std::ptr;

// Node structure for the doubly linked list
struct Node<K, V> {
    key: K,
    value: V,
    prev: *mut Node<K, V>,
    next: *mut Node<K, V>,
}

impl<K, V> Node<K, V> {
    fn new(key: K, value: V) -> Self {
        Node {
            key,
            value,
            prev: ptr::null_mut(),
            next: ptr::null_mut(),
        }
    }
}

/// A single threaded LRU cache implementation
struct LRUCache<K, V> {
    capacity: usize,
    map: HashMap<K, *mut Node<K, V>>,
    head: *mut Node<K, V>,
    tail: *mut Node<K, V>,
    stats_total: u64,
    stats_hits: u64,
    stats_evicts: u64,
}

impl<K, V> LRUCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    fn new(capacity: usize) -> Self {
        LRUCache {
            capacity,
            map: HashMap::new(),
            head: ptr::null_mut(),
            tail: ptr::null_mut(),
            stats_total: 0,
            stats_hits: 0,
            stats_evicts: 0,
        }
    }

    fn get(&mut self, key: &K) -> Option<V> {
        self.stats_total += 1;
        if let Some(node_ptr) = self.map.get(key).copied() {
            // Clone the value before potentially moving the node
            let value = unsafe { (*node_ptr).value.clone() };

            // Remove node from current position
            self.remove_node(node_ptr);

            // Add node to head
            self.add_to_head(node_ptr);

            self.stats_hits += 1;
            Some(value)
        } else {
            None
        }
    }

    fn put(&mut self, key: K, value: V) {
        if let Some(&node_ptr) = self.map.get(&key) {
            // Update existing node
            unsafe {
                (*node_ptr).value = value;
            }

            // Remove from current position
            self.remove_node(node_ptr);

            // Add to head
            self.add_to_head(node_ptr);
        } else {
            // New entry
            if self.map.len() >= self.capacity {
                // Remove tail if at capacity
                if let Some(tail_ptr) = self.pop_tail() {
                    unsafe {
                        let tail_key = (*tail_ptr).key.clone();
                        self.map.remove(&tail_key);
                        // Deallocate the node
                        let _ = Box::from_raw(tail_ptr);
                    }
                    self.stats_evicts += 1;
                }
            }

            // Create new node
            let mut new_node = Box::new(Node::new(key.clone(), value));
            let new_node_ptr = Box::into_raw(new_node);

            // Add to map
            self.map.insert(key, new_node_ptr);

            // Add to head
            self.add_to_head(new_node_ptr);
        }
    }

    fn contains(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }

    fn remove_node(&mut self, node_ptr: *mut Node<K, V>) {
        unsafe {
            let node = &mut *node_ptr;

            if node.prev.is_null() {
                self.head = node.next;
            } else {
                (*node.prev).next = node.next;
            }

            if node.next.is_null() {
                self.tail = node.prev;
            } else {
                (*node.next).prev = node.prev;
            }
        }
    }

    fn add_to_head(&mut self, node_ptr: *mut Node<K, V>) {
        unsafe {
            let node = &mut *node_ptr;

            node.next = self.head;
            node.prev = ptr::null_mut();

            if !self.head.is_null() {
                (*self.head).prev = node_ptr;
            } else {
                self.tail = node_ptr;  // If list was empty, this node is also the tail
            }

            self.head = node_ptr;
        }
    }

    fn pop_tail(&mut self) -> Option<*mut Node<K, V>> {
        if self.tail.is_null() {
            None
        } else {
            let old_tail = self.tail;
            unsafe {
                let tail_node = &mut *old_tail;
                self.tail = tail_node.prev;

                if !self.tail.is_null() {
                    (*self.tail).next = ptr::null_mut();
                } else {
                    self.head = ptr::null_mut(); // List is now empty
                }

                tail_node.prev = ptr::null_mut();
                tail_node.next = ptr::null_mut();
            }
            Some(old_tail)
        }
    }

    fn stats_total(&self) -> u64 {
        self.stats_total
    }

    fn stats_hits(&self) -> u64 {
        self.stats_hits
    }

    fn stats_evicts(&self) -> u64 {
        self.stats_evicts
    }
}

impl<K, V> Drop for LRUCache<K, V> {
    fn drop(&mut self) {
        // Clean up all nodes to prevent memory leaks
        while !self.head.is_null() {
            let node_ptr = self.head;
            unsafe {
                self.head = (*node_ptr).next;
                // Deallocate the node
                let _ = Box::from_raw(node_ptr);
            }
        }
    }
}

/// A thread-safe, concurrent LRU cache implementation
pub struct ConcurrentLRUCache<K, V> {
    buckets: Vec<Arc<Mutex<LRUCache<K, V>>>>,
    bucket_mask: usize,
    stats_total: AtomicU64,
    stats_hits: AtomicU64,
    stats_evicts: AtomicU64,
}

impl<K, V> ConcurrentLRUCache<K, V>
where
    K: Eq + Hash + Clone + std::hash::Hash,
    V: Clone,
{
    /// Create a new concurrent LRU cache with the specified capacity
    /// buckets_exp specifies the number of buckets as 2^buckets_exp
    pub fn new(capacity: usize, buckets_exp: u32) -> Self {
        let buckets_num = 1 << buckets_exp;
        let cap_per_bucket = capacity >> buckets_exp;
        let mut buckets = Vec::with_capacity(buckets_num);
        
        for _ in 0..buckets_num {
            buckets.push(Arc::new(Mutex::new(LRUCache::new(cap_per_bucket))));
        }
        
        ConcurrentLRUCache {
            buckets,
            bucket_mask: buckets_num - 1,
            stats_total: AtomicU64::new(0),
            stats_hits: AtomicU64::new(0),
            stats_evicts: AtomicU64::new(0),
        }
    }

    /// Get a value from the cache
    pub fn get(&self, key: &K) -> Option<V> {
        let bucket_idx = self.bucket_index(key, None);
        let mut bucket = self.buckets[bucket_idx].lock().unwrap();
        
        let result = bucket.get(key);
        if result.is_some() {
            self.stats_hits.fetch_add(1, Ordering::Relaxed);
        }
        self.stats_total.fetch_add(1, Ordering::Relaxed);
        
        result
    }

    /// Put a key-value pair in the cache
    pub fn put(&self, key: K, value: V) {
        let bucket_idx = self.bucket_index(&key, None);
        let mut bucket = self.buckets[bucket_idx].lock().unwrap();
        
        // If replacing, we need to remove first to get the old value out of the map
        bucket.put(key, value);
    }

    /// Check if the cache contains a key
    pub fn contains(&self, key: &K) -> bool {
        let bucket_idx = self.bucket_index(key, None);
        let bucket = self.buckets[bucket_idx].lock().unwrap();
        
        bucket.contains(key)
    }

    /// Insert a key-value pair in the cache
    pub fn insert(&self, key: K, value: V) {
        self.put(key, value);
    }

    /// Get a value if it exists, otherwise insert and return the new value
    /// Returns the existing value if it was already in the cache
    pub fn get_or_insert(&self, key: K, value: V) -> V {
        // First try to get the value
        if let Some(existing) = self.get(&key) {
            return existing;
        }
        
        // If not found, insert the new value
        self.insert(key, value.clone());
        value
    }

    /// Calculate which bucket a key belongs to
    fn bucket_index(&self, key: &K, _hint: Option<i32>) -> usize {
        // Calculate hash of the key
        use std::hash::Hasher;
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut hasher);
        let hash = hasher.finish();
        
        (hash as usize) & self.bucket_mask
    }

    /// Get total number of operations
    pub fn total(&self) -> u64 {
        self.stats_total.load(Ordering::Relaxed)
    }

    /// Get number of hits
    pub fn hits(&self) -> u64 {
        self.stats_hits.load(Ordering::Relaxed)
    }

    /// Get number of evictions
    pub fn evicts(&self) -> u64 {
        self.stats_evicts.load(Ordering::Relaxed)
    }

    /// Get cache hit rate
    pub fn hit_rate(&self) -> f64 {
        let total = self.total();
        if total == 0 {
            0.0
        } else {
            self.hits() as f64 / total as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lru_cache_basic() {
        let mut cache = LRUCache::new(2);
        
        cache.put("a", 1);
        cache.put("b", 2);
        
        assert_eq!(cache.get(&"a"), Some(1)); // Access "a" to make it recently used
        cache.put("c", 3); // This should evict "b" as it's least recently used
        
        assert_eq!(cache.get(&"b"), None);
        assert_eq!(cache.get(&"a"), Some(1));
        assert_eq!(cache.get(&"c"), Some(3));
    }

    #[test]
    fn test_concurrent_lru_cache() {
        let cache = ConcurrentLRUCache::new(100, 2); // 4 buckets
        
        cache.put("a", 1);
        cache.put("b", 2);
        
        assert_eq!(cache.get(&"a"), Some(1));
        assert_eq!(cache.get(&"b"), Some(2));
        
        assert!(cache.contains(&"a"));
        assert!(!cache.contains(&"c"));
        
        let value = cache.get_or_insert("c", 3);
        assert_eq!(value, 3);
        
        let value = cache.get_or_insert("c", 4); // Should return existing value
        assert_eq!(value, 3);
    }

    #[test]
    fn test_cache_statistics() {
        let cache = ConcurrentLRUCache::new(10, 2);
        
        cache.put("a", 1);
        
        assert_eq!(cache.get(&"a"), Some(1)); // Hit
        assert_eq!(cache.get(&"b"), None);   // Miss
        
        assert_eq!(cache.total(), 2);
        assert_eq!(cache.hits(), 1);
        assert!(cache.hit_rate() > 0.0);
    }
}