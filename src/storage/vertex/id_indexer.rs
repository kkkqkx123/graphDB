//! ID Indexer
//!
//! Maps external IDs (strings or integers) to internal vertex IDs.
//! Provides O(1) lookup in both directions.

use std::collections::HashMap;
use std::hash::Hash;

use super::{StorageError, StorageResult};

#[derive(Debug, Clone)]
pub struct IdIndexer<K>
where
    K: Eq + Hash + Clone,
{
    keys: Vec<K>,
    key_to_index: HashMap<K, u32>,
    capacity: usize,
}

impl<K> IdIndexer<K>
where
    K: Eq + Hash + Clone,
{
    pub fn new() -> Self {
        Self::with_capacity(1024)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            keys: Vec::with_capacity(capacity),
            key_to_index: HashMap::with_capacity(capacity),
            capacity,
        }
    }

    pub fn insert(&mut self, key: K) -> StorageResult<u32> {
        if self.keys.len() >= self.capacity {
            return Err(StorageError::CapacityExceeded);
        }

        let index = self.keys.len() as u32;
        self.keys.push(key.clone());
        self.key_to_index.insert(key, index);
        Ok(index)
    }

    pub fn get_index(&self, key: &K) -> Option<u32> {
        self.key_to_index.get(key).copied()
    }

    pub fn get_key(&self, index: u32) -> Option<&K> {
        self.keys.get(index as usize)
    }

    pub fn contains(&self, key: &K) -> bool {
        self.key_to_index.contains_key(key)
    }

    pub fn size(&self) -> usize {
        self.keys.len()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn reserve(&mut self, new_capacity: usize) {
        if new_capacity > self.capacity {
            self.capacity = new_capacity;
            self.keys.reserve(new_capacity);
            self.key_to_index.reserve(new_capacity);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, u32)> {
        self.keys.iter().enumerate().map(|(i, k)| (k, i as u32))
    }

    pub fn keys(&self) -> &[K] {
        &self.keys
    }

    pub fn clear(&mut self) {
        self.keys.clear();
        self.key_to_index.clear();
    }
}

impl<K> Default for IdIndexer<K>
where
    K: Eq + Hash + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

pub type StringIdIndexer = IdIndexer<String>;
pub type Int64IdIndexer = IdIndexer<i64>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        let mut indexer: StringIdIndexer = IdIndexer::new();

        let idx1 = indexer.insert("vertex1".to_string()).unwrap();
        assert_eq!(idx1, 0);

        let idx2 = indexer.insert("vertex2".to_string()).unwrap();
        assert_eq!(idx2, 1);

        assert_eq!(indexer.get_index(&"vertex1".to_string()), Some(0));
        assert_eq!(indexer.get_index(&"vertex2".to_string()), Some(1));
        assert_eq!(indexer.get_index(&"vertex3".to_string()), None);

        assert_eq!(indexer.get_key(0), Some(&"vertex1".to_string()));
        assert_eq!(indexer.get_key(1), Some(&"vertex2".to_string()));
    }

    #[test]
    fn test_capacity() {
        let mut indexer: StringIdIndexer = IdIndexer::with_capacity(2);

        assert!(indexer.insert("v1".to_string()).is_ok());
        assert!(indexer.insert("v2".to_string()).is_ok());
        assert!(indexer.insert("v3".to_string()).is_err());

        indexer.reserve(10);
        assert!(indexer.insert("v3".to_string()).is_ok());
    }
}
