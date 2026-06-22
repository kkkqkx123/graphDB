//! ID Indexer
//!
//! Maps external IDs (strings or integers) to internal vertex IDs.
//! Provides O(1) concurrent-safe lookup in both directions.
//!
//! Features:
//! - Concurrent-safe: DashMap enables lock-free sharded access
//! - Dynamic expansion: automatically grows when capacity is reached
//! - Zero-copy insertion: supports concurrent inserts from multiple threads

use std::sync::Arc;

use dashmap::DashMap;
use parking_lot::Mutex;

use crate::core::error::{StorageError, StorageResult};

const DEFAULT_INITIAL_CAPACITY: usize = 1024;
const DEFAULT_GROWTH_FACTOR: f64 = 1.5;
const MAX_CAPACITY: usize = u32::MAX as usize;

const ID_KEY_TYPE_INT: u8 = 0;
const ID_KEY_TYPE_TEXT: u8 = 1;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum IdKey {
    Int(i64),
    Text(String),
}

impl IdKey {
    /// Write the key bytes into an existing buffer to avoid extra allocations.
    /// The buffer is cleared before writing.
    pub fn write_to(&self, buf: &mut Vec<u8>) {
        buf.clear();
        match self {
            IdKey::Int(val) => {
                buf.reserve(9);
                buf.push(ID_KEY_TYPE_INT);
                buf.extend_from_slice(&val.to_be_bytes());
            }
            IdKey::Text(val) => {
                buf.reserve(1 + val.len());
                buf.push(ID_KEY_TYPE_TEXT);
                buf.extend_from_slice(val.as_bytes());
            }
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> StorageResult<Self> {
        if bytes.is_empty() {
            return Err(StorageError::deserialize_error(
                "Empty IdKey bytes".to_string(),
            ));
        }

        match bytes[0] {
            ID_KEY_TYPE_INT => {
                if bytes.len() != 9 {
                    return Err(StorageError::deserialize_error(format!(
                        "Invalid Int IdKey length: {}",
                        bytes.len()
                    )));
                }
                let val_bytes: [u8; 8] = bytes[1..9].try_into().map_err(|_| {
                    StorageError::deserialize_error("Invalid Int IdKey bytes".to_string())
                })?;
                Ok(IdKey::Int(i64::from_be_bytes(val_bytes)))
            }
            ID_KEY_TYPE_TEXT => {
                let text = String::from_utf8(bytes[1..].to_vec())
                    .map_err(|e| StorageError::deserialize_error(e.to_string()))?;
                Ok(IdKey::Text(text))
            }
            tag => Err(StorageError::deserialize_error(format!(
                "Unknown IdKey type tag: {}",
                tag
            ))),
        }
    }
}

impl std::fmt::Display for IdKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IdKey::Int(val) => write!(f, "{}", val),
            IdKey::Text(val) => write!(f, "{}", val),
        }
    }
}

#[derive(Debug, Clone)]
pub struct IdIndexerConfig {
    pub initial_capacity: usize,
    pub growth_factor: f64,
    pub max_capacity: usize,
    pub enable_free_list: bool,
}

impl Default for IdIndexerConfig {
    fn default() -> Self {
        Self {
            initial_capacity: DEFAULT_INITIAL_CAPACITY,
            growth_factor: DEFAULT_GROWTH_FACTOR,
            max_capacity: MAX_CAPACITY,
            enable_free_list: true,
        }
    }
}

impl IdIndexerConfig {
    pub fn with_initial_capacity(mut self, capacity: usize) -> Self {
        self.initial_capacity = capacity;
        self
    }
}

/// Concurrent ID indexer using DashMap for lock-free sharded access.
///
/// Provides concurrent-safe O(1) lookup with automatic sharding (typically 16 shards).
/// Suitable for high-concurrency workloads like batch_insert_vertices and scan_vertices.
///
/// # Example
///
/// ```ignore
/// let indexer = IdIndexer::new();
/// let idx = indexer.insert(IdKey::Text("v1".to_string()))?;
/// assert_eq!(indexer.get_index(&IdKey::Text("v1".to_string())), Some(idx));
/// ```
#[derive(Debug, Clone)]
pub struct IdIndexer {
    key_to_index: Arc<DashMap<IdKey, u32>>,
    keys: Arc<Mutex<Vec<Option<IdKey>>>>,
    config: IdIndexerConfig,
}

impl IdIndexer {
    pub fn new() -> Self {
        Self::with_config(IdIndexerConfig::default())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_config(IdIndexerConfig::default().with_initial_capacity(capacity))
    }

    pub fn with_config(config: IdIndexerConfig) -> Self {
        let capacity = config.initial_capacity.min(config.max_capacity);
        Self {
            key_to_index: Arc::new(DashMap::with_capacity(capacity)),
            keys: Arc::new(Mutex::new(Vec::with_capacity(capacity))),
            config,
        }
    }

    /// Insert a key and return its internal ID.
    /// Concurrent-safe: multiple threads can insert simultaneously.
    pub fn insert(&self, key: IdKey) -> StorageResult<u32> {
        if self.key_to_index.contains_key(&key) {
            return Err(StorageError::vertex_already_exists(format!("{:?}", key)));
        }

        let mut keys = self.keys.lock();
        if keys.len() >= self.config.max_capacity {
            return Err(StorageError::capacity_exceeded());
        }

        if keys.len() >= keys.capacity() {
            let current_capacity = keys.capacity();
            if current_capacity >= self.config.max_capacity {
                return Err(StorageError::capacity_exceeded());
            }

            let new_capacity = ((current_capacity as f64 * self.config.growth_factor) as usize)
                .min(self.config.max_capacity)
                .max(current_capacity + 1);
            keys.reserve(new_capacity - current_capacity);
        }

        let index = keys.len() as u32;
        keys.push(Some(key.clone()));
        self.key_to_index.insert(key, index);

        Ok(index)
    }

    /// Get the internal ID for a given key (lock-free read).
    pub fn get_index(&self, key: &IdKey) -> Option<u32> {
        self.key_to_index.get(key).map(|ref_multi| *ref_multi)
    }

    /// Get the key for a given internal ID.
    pub fn get_key(&self, index: u32) -> Option<IdKey> {
        let keys = self.keys.lock();
        keys.get(index as usize)?.as_ref().cloned()
    }

    pub fn contains(&self, key: &IdKey) -> bool {
        self.key_to_index.contains_key(key)
    }

    pub fn len(&self) -> usize {
        self.key_to_index.len()
    }

    pub fn is_empty(&self) -> bool {
        self.key_to_index.is_empty()
    }

    /// Remove a key and return its ID.
    pub fn remove(&self, key: &IdKey) -> Option<u32> {
        self.key_to_index.remove(key).map(|(_, idx)| {
            let mut keys = self.keys.lock();
            if (idx as usize) < keys.len() {
                keys[idx as usize] = None;
            }
            idx
        })
    }

    /// Iterate over all active entries (key, index).
    /// Note: returns a snapshot; mutations during iteration are not reflected.
    pub fn iter(&self) -> Vec<(IdKey, u32)> {
        self.key_to_index
            .iter()
            .map(|ref_multi| (ref_multi.key().clone(), *ref_multi.value()))
            .collect()
    }

    pub fn clear(&self) {
        self.key_to_index.clear();
        self.keys.lock().clear();
    }

    pub fn memory_usage(&self) -> usize {
        let keys = self.keys.lock();
        let keys_size = keys.capacity() * std::mem::size_of::<Option<IdKey>>();
        let map_estimate = self.key_to_index.len()
            * (std::mem::size_of::<IdKey>() + std::mem::size_of::<u32>());
        keys_size + map_estimate
    }

    pub fn memory_size(&self) -> usize {
        self.memory_usage() + std::mem::size_of::<Self>()
    }

    /// Compact the indexer by removing gaps and returning ID remapping.
    ///
    /// Reorganizes IDs to be contiguous starting from 0.
    /// Returns a mapping of old_id → new_id for all IDs that moved.
    ///
    /// # Responsibility in Compaction
    ///
    /// This is the AUTHORITATIVE mapping source during compaction.
    /// All other structures (VertexTimestamp, ColumnStore) must follow this mapping.
    ///
    /// **Why this matters:**
    /// - id_indexer owns the Key↔ID mapping (primary responsibility)
    /// - Other structures index by ID, so they must be updated when IDs change
    /// - This method returns the mapping so VertexTable can propagate it
    ///
    /// # Example
    ///
    /// If we have IDs {0, 2, 5} with keys ["a", "c", "f"],
    /// compaction returns {2→1, 5→2}, so:
    /// - Key "a" stays at ID 0
    /// - Key "c" moves from ID 2 to ID 1
    /// - Key "f" moves from ID 5 to ID 2
    ///
    /// # Algorithm
    ///
    /// 1. Collect all live (ID, Key) pairs from key_to_index
    /// 2. Sort by old ID to maintain stable ordering
    /// 3. Assign new sequential IDs (0, 1, 2, ...)
    /// 4. Compute mapping only for IDs that moved (optimization)
    /// 5. Rebuild internal structures if any mapping exists
    /// 6. Return mapping for propagation to VertexTimestamp and ColumnStore
    ///
    /// # Correctness Invariants
    ///
    /// After compact():
    /// - All IDs are in range [0, live_count)
    /// - No gaps in ID sequence
    /// - All keys are preserved (no data loss)
    /// - External callers can use returned mapping to update dependent structures
    ///
    /// # Concurrency Note
    ///
    /// This method holds Mutex while rebuilding.
    /// Safe for concurrent reads (lock is brief), but not safe with concurrent inserts.
    /// Use outside of transaction for safety.
    ///
    /// # Performance
    ///
    /// - O(n) where n = number of live IDs
    /// - O(n log n) for sorting
    /// - Practical: allocates new vectors, old memory is reclaimed
    ///
    /// # Why Return Mapping?
    ///
    /// This design allows VertexTable to:
    /// 1. Get the mapping: `let mapping = id_indexer.compact()?;`
    /// 2. Apply it to timestamps: `self.remap_timestamps(&mapping);`
    /// 3. Apply it to columns: `self.remap_columns(&mapping);`
    /// 4. Verify consistency: `self.verify_invariants()?;`
    ///
    /// If id_indexer did everything, it would need to know about VertexTimestamp
    /// and ColumnStore, violating module boundaries. This design keeps concerns separate.
    pub fn compact(&self) -> StorageResult<std::collections::HashMap<u32, u32>> {
        // Get snapshot of all live IDs and keys
        let entries: Vec<(u32, IdKey)> = self
            .key_to_index
            .iter()
            .map(|ref_multi| (*ref_multi.value(), ref_multi.key().clone()))
            .collect();

        if entries.is_empty() {
            // No entries, nothing to compact
            return Ok(std::collections::HashMap::new());
        }

        // Sort by old ID to maintain stable ordering
        let mut entries = entries;
        entries.sort_by_key(|(old_id, _)| *old_id);

        // Compute mapping: old_id → new_id
        let mut mapping = std::collections::HashMap::new();
        for (new_id, (old_id, _)) in entries.iter().enumerate() {
            let new_id_u32 = new_id as u32;
            if *old_id != new_id_u32 {
                mapping.insert(*old_id, new_id_u32);
            }
        }

        // If no IDs moved, no need to rebuild
        if mapping.is_empty() {
            return Ok(std::collections::HashMap::new());
        }

        // Rebuild internal structures with new IDs
        self.rebuild_with_mapping(&entries)?;

        Ok(mapping)
    }

    /// Internal: Rebuild id_indexer with remapped IDs
    /// Called only from compact()
    ///
    /// # Arguments
    /// * `entries` - Entries sorted by old ID, with new IDs assigned sequentially
    fn rebuild_with_mapping(&self, entries: &[(u32, IdKey)]) -> StorageResult<()> {
        // Rebuild keys array
        let mut new_keys = vec![None; entries.len()];
        for (new_id, (_, key)) in entries.iter().enumerate() {
            new_keys[new_id] = Some(key.clone());
        }

        // Update the keys Mutex
        {
            let mut keys = self.keys.lock();
            *keys = new_keys;
        }

        // Clear and rebuild key_to_index with new IDs
        self.key_to_index.clear();
        for (new_id, (_, key)) in entries.iter().enumerate() {
            self.key_to_index.insert(key.clone(), new_id as u32);
        }

        Ok(())
    }

    /// Set the key at a specific index (compatibility method for loader).
    pub fn set_at(&self, index: u32, key: IdKey) {
        if self.key_to_index.contains_key(&key) {
            return;
        }
        {
            let mut keys = self.keys.lock();
            while keys.len() <= index as usize {
                keys.push(None);
            }
            keys[index as usize] = Some(key.clone());
        }
        self.key_to_index.insert(key, index);
    }
}

impl Default for IdIndexer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        let indexer = IdIndexer::new();

        let idx1 = indexer.insert(IdKey::Text("vertex1".to_string())).unwrap();
        assert_eq!(idx1, 0);

        let idx2 = indexer.insert(IdKey::Text("vertex2".to_string())).unwrap();
        assert_eq!(idx2, 1);

        assert_eq!(
            indexer.get_index(&IdKey::Text("vertex1".to_string())),
            Some(0)
        );
        assert_eq!(
            indexer.get_index(&IdKey::Text("vertex2".to_string())),
            Some(1)
        );
        assert_eq!(indexer.get_index(&IdKey::Text("vertex3".to_string())), None);

        assert_eq!(
            indexer.get_key(0),
            Some(IdKey::Text("vertex1".to_string()))
        );
        assert_eq!(
            indexer.get_key(1),
            Some(IdKey::Text("vertex2".to_string()))
        );
    }

    #[test]
    fn test_int_id_operations() {
        let indexer = IdIndexer::new();

        let idx1 = indexer.insert(IdKey::Int(100)).unwrap();
        assert_eq!(idx1, 0);

        let idx2 = indexer.insert(IdKey::Int(200)).unwrap();
        assert_eq!(idx2, 1);

        assert_eq!(indexer.get_index(&IdKey::Int(100)), Some(0));
        assert_eq!(indexer.get_index(&IdKey::Int(200)), Some(1));
        assert_eq!(indexer.get_index(&IdKey::Int(300)), None);

        assert_eq!(indexer.get_key(0), Some(IdKey::Int(100)));
        assert_eq!(indexer.get_key(1), Some(IdKey::Int(200)));
    }

    #[test]
    fn test_mixed_id_operations() {
        let indexer = IdIndexer::new();

        let idx1 = indexer.insert(IdKey::Int(100)).unwrap();
        let idx2 = indexer.insert(IdKey::Text("vertex1".to_string())).unwrap();
        let idx3 = indexer.insert(IdKey::Int(200)).unwrap();

        assert_eq!(idx1, 0);
        assert_eq!(idx2, 1);
        assert_eq!(idx3, 2);

        assert_eq!(indexer.len(), 3);
    }

    #[test]
    fn test_dynamic_expansion() {
        let indexer = IdIndexer::with_config(IdIndexerConfig {
            initial_capacity: 2,
            growth_factor: 2.0,
            max_capacity: MAX_CAPACITY,
            enable_free_list: true,
        });

        assert!(indexer.insert(IdKey::Text("v1".to_string())).is_ok());
        assert!(indexer.insert(IdKey::Text("v2".to_string())).is_ok());
        assert!(indexer.insert(IdKey::Text("v3".to_string())).is_ok());
        assert!(indexer.insert(IdKey::Text("v4".to_string())).is_ok());
        assert!(indexer.insert(IdKey::Text("v5".to_string())).is_ok());

        assert_eq!(indexer.len(), 5);
    }

    #[test]
    fn test_duplicate_insert() {
        let indexer = IdIndexer::new();

        assert!(indexer.insert(IdKey::Text("v1".to_string())).is_ok());
        assert!(indexer.insert(IdKey::Text("v1".to_string())).is_err());
    }

    #[test]
    fn test_max_capacity() {
        let indexer = IdIndexer::with_config(IdIndexerConfig {
            initial_capacity: 2,
            growth_factor: DEFAULT_GROWTH_FACTOR,
            max_capacity: 3,
            enable_free_list: true,
        });

        assert!(indexer.insert(IdKey::Text("v1".to_string())).is_ok());
        assert!(indexer.insert(IdKey::Text("v2".to_string())).is_ok());
        assert!(indexer.insert(IdKey::Text("v3".to_string())).is_ok());
        assert!(indexer.insert(IdKey::Text("v4".to_string())).is_err());
    }

    #[test]
    fn test_concurrent_parallel_inserts() {
        use std::sync::Arc as StdArc;
        use std::thread;

        let indexer = StdArc::new(IdIndexer::new());
        let mut handles = vec![];

        for thread_id in 0..4 {
            let indexer_clone = StdArc::clone(&indexer);
            let handle = thread::spawn(move || {
                for i in 0..25 {
                    let key = IdKey::Text(format!("v_{}_{}", thread_id, i));
                    let _ = indexer_clone.insert(key);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().expect("thread panicked");
        }

        assert_eq!(indexer.len(), 100);
    }

    #[test]
    fn test_concurrent_mixed_operations() {
        use std::sync::Arc as StdArc;
        use std::thread;

        let indexer = StdArc::new(IdIndexer::new());

        for i in 0..10 {
            let key = IdKey::Text(format!("v{}", i));
            let _ = indexer.insert(key);
        }

        let mut handles = vec![];

        for _ in 0..2 {
            let indexer_clone = StdArc::clone(&indexer);
            let handle = thread::spawn(move || {
                for i in 0..10 {
                    let key = IdKey::Text(format!("v{}", i));
                    let _ = indexer_clone.get_index(&key);
                }
            });
            handles.push(handle);
        }

        let indexer_clone = StdArc::clone(&indexer);
        let handle = thread::spawn(move || {
            for i in 10..20 {
                let key = IdKey::Text(format!("v{}", i));
                let _ = indexer_clone.insert(key);
            }
        });
        handles.push(handle);

        for handle in handles {
            handle.join().expect("thread panicked");
        }

        assert_eq!(indexer.len(), 20);
    }

    #[test]
    fn test_remove() {
        let indexer = IdIndexer::new();

        indexer.insert(IdKey::Text("v1".to_string())).unwrap();
        indexer.insert(IdKey::Text("v2".to_string())).unwrap();
        indexer.insert(IdKey::Text("v3".to_string())).unwrap();

        assert_eq!(indexer.len(), 3);

        indexer.remove(&IdKey::Text("v2".to_string()));
        assert_eq!(indexer.len(), 2);

        assert_eq!(
            indexer.get_index(&IdKey::Text("v2".to_string())),
            None
        );
    }
}
