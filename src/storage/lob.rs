//! Large Object Store
//!
//! Provides storage for large objects (BLOBs, large text) that exceed
//! the inline storage threshold. Large objects are stored separately
//! and referenced by ID in the main column store.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::core::StorageResult;

/// Default threshold for large object storage (1KB)
pub const DEFAULT_LOB_THRESHOLD: usize = 1024;

/// Unique identifier for a large object
pub type LobId = u64;

/// Large object store for storing BLOBs and large text
pub struct LargeObjectStore {
    objects: HashMap<LobId, Vec<u8>>,
    next_id: AtomicU64,
    threshold: usize,
    total_size: usize,
}

impl LargeObjectStore {
    /// Create a new large object store with default threshold
    pub fn new() -> Self {
        Self {
            objects: HashMap::new(),
            next_id: AtomicU64::new(1),
            threshold: DEFAULT_LOB_THRESHOLD,
            total_size: 0,
        }
    }

    /// Create a new large object store with custom threshold
    pub fn with_threshold(threshold: usize) -> Self {
        Self {
            objects: HashMap::new(),
            next_id: AtomicU64::new(1),
            threshold,
            total_size: 0,
        }
    }

    /// Store a large object and return its ID
    pub fn store(&mut self, data: Vec<u8>) -> LobId {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        self.total_size += data.len();
        self.objects.insert(id, data);
        id
    }

    /// Load a large object by ID
    pub fn load(&self, id: LobId) -> Option<&[u8]> {
        self.objects.get(&id).map(|v| v.as_slice())
    }

    /// Load a large object by ID as owned data
    pub fn load_owned(&self, id: LobId) -> Option<Vec<u8>> {
        self.objects.get(&id).cloned()
    }

    /// Delete a large object by ID
    pub fn delete(&mut self, id: LobId) -> bool {
        if let Some(data) = self.objects.remove(&id) {
            self.total_size = self.total_size.saturating_sub(data.len());
            true
        } else {
            false
        }
    }

    /// Update a large object
    pub fn update(&mut self, id: LobId, data: Vec<u8>) -> StorageResult<()> {
        if let Some(old_data) = self.objects.get_mut(&id) {
            self.total_size = self.total_size.saturating_sub(old_data.len());
            self.total_size += data.len();
            *old_data = data;
            Ok(())
        } else {
            Err(crate::core::StorageError::invalid_operation(format!(
                "Large object {} not found",
                id
            )))
        }
    }

    /// Check if a large object exists
    pub fn contains(&self, id: LobId) -> bool {
        self.objects.contains_key(&id)
    }

    /// Get the threshold for large object storage
    pub fn threshold(&self) -> usize {
        self.threshold
    }

    /// Check if data should be stored as a large object
    pub fn should_store_large(&self, data_len: usize) -> bool {
        data_len > self.threshold
    }

    /// Get the total number of large objects
    pub fn count(&self) -> usize {
        self.objects.len()
    }

    /// Get the total size of all large objects
    pub fn total_size(&self) -> usize {
        self.total_size
    }

    /// Get statistics about the large object store
    pub fn stats(&self) -> LobStats {
        LobStats {
            object_count: self.objects.len(),
            total_size: self.total_size,
            threshold: self.threshold,
            average_size: if self.objects.is_empty() {
                0.0
            } else {
                self.total_size as f64 / self.objects.len() as f64
            },
        }
    }

    /// Clear all large objects
    pub fn clear(&mut self) {
        self.objects.clear();
        self.total_size = 0;
    }
}

impl Default for LargeObjectStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for the large object store
#[derive(Debug, Clone)]
pub struct LobStats {
    pub object_count: usize,
    pub total_size: usize,
    pub threshold: usize,
    pub average_size: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_and_load() {
        let mut store = LargeObjectStore::new();
        let data = vec![1u8, 2, 3, 4, 5];
        let id = store.store(data.clone());

        assert!(store.contains(id));
        assert_eq!(store.load(id), Some(&[1u8, 2, 3, 4, 5][..]));
        assert_eq!(store.load_owned(id), Some(data));
    }

    #[test]
    fn test_delete() {
        let mut store = LargeObjectStore::new();
        let data = vec![1u8, 2, 3];
        let id = store.store(data);

        assert!(store.delete(id));
        assert!(!store.contains(id));
        assert!(store.load(id).is_none());

        assert!(!store.delete(999));
    }

    #[test]
    fn test_update() {
        let mut store = LargeObjectStore::new();
        let data = vec![1u8, 2, 3];
        let id = store.store(data);

        let new_data = vec![4u8, 5, 6, 7];
        store.update(id, new_data.clone()).unwrap();

        assert_eq!(store.load(id), Some(&[4u8, 5, 6, 7][..]));
    }

    #[test]
    fn test_threshold() {
        let store = LargeObjectStore::with_threshold(100);

        assert!(store.should_store_large(101));
        assert!(!store.should_store_large(100));
        assert!(!store.should_store_large(50));
    }

    #[test]
    fn test_stats() {
        let mut store = LargeObjectStore::new();
        store.store(vec![1u8; 100]);
        store.store(vec![2u8; 200]);
        store.store(vec![3u8; 300]);

        let stats = store.stats();
        assert_eq!(stats.object_count, 3);
        assert_eq!(stats.total_size, 600);
        assert_eq!(stats.average_size, 200.0);
    }

    #[test]
    fn test_clear() {
        let mut store = LargeObjectStore::new();
        store.store(vec![1u8, 2, 3]);
        store.store(vec![4u8, 5, 6]);

        store.clear();
        assert_eq!(store.count(), 0);
        assert_eq!(store.total_size(), 0);
    }

    #[test]
    fn test_total_size_tracking() {
        let mut store = LargeObjectStore::new();
        let id1 = store.store(vec![1u8; 100]);
        let id2 = store.store(vec![2u8; 200]);

        assert_eq!(store.total_size(), 300);

        store.update(id1, vec![3u8; 50]).unwrap();
        assert_eq!(store.total_size(), 250);

        store.delete(id2);
        assert_eq!(store.total_size(), 50);
    }
}
