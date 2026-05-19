//! ID Indexer
//!
//! Maps external IDs (strings or integers) to internal vertex IDs.
//! Provides O(1) lookup in both directions.
//!
//! Features:
//! - Dynamic expansion: automatically grows when capacity is reached
//! - Free list reuse: reuses deleted IDs to reduce fragmentation

use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;

use crate::storage::{StorageError, StorageResult};

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
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            IdKey::Int(val) => {
                let mut bytes = Vec::with_capacity(9);
                bytes.push(ID_KEY_TYPE_INT);
                bytes.extend_from_slice(&val.to_be_bytes());
                bytes
            }
            IdKey::Text(val) => {
                let mut bytes = Vec::with_capacity(1 + val.len());
                bytes.push(ID_KEY_TYPE_TEXT);
                bytes.extend_from_slice(val.as_bytes());
                bytes
            }
        }
    }

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

    pub fn with_growth_factor(mut self, factor: f64) -> Self {
        self.growth_factor = factor;
        self
    }

    pub fn with_max_capacity(mut self, max: usize) -> Self {
        self.max_capacity = max;
        self
    }

    pub fn with_free_list(mut self, enable: bool) -> Self {
        self.enable_free_list = enable;
        self
    }
}

#[derive(Debug, Clone)]
pub struct IdIndexer {
    keys: Vec<Option<IdKey>>,
    key_to_index: HashMap<IdKey, u32>,
    free_list: VecDeque<u32>,
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
            keys: Vec::with_capacity(capacity),
            key_to_index: HashMap::with_capacity(capacity),
            free_list: VecDeque::new(),
            config,
        }
    }

    pub fn insert(&mut self, key: IdKey) -> StorageResult<u32> {
        if self.key_to_index.contains_key(&key) {
            return Err(StorageError::vertex_already_exists(format!("{:?}", key)));
        }

        if self.len() >= self.config.max_capacity {
            return Err(StorageError::capacity_exceeded());
        }

        if let Some(reused_id) = self.free_list.pop_front() {
            self.keys[reused_id as usize] = Some(key.clone());
            self.key_to_index.insert(key, reused_id);
            return Ok(reused_id);
        }

        if self.keys.len() >= self.keys.capacity() {
            self.grow()?;
        }

        let index = self.keys.len() as u32;
        self.keys.push(Some(key.clone()));
        self.key_to_index.insert(key, index);
        Ok(index)
    }

    fn grow(&mut self) -> StorageResult<()> {
        let current_capacity = self.keys.capacity();
        if current_capacity >= self.config.max_capacity {
            return Err(StorageError::capacity_exceeded());
        }

        let new_capacity = ((current_capacity as f64 * self.config.growth_factor) as usize)
            .min(self.config.max_capacity)
            .max(current_capacity + 1);

        self.keys.reserve(new_capacity - current_capacity);
        self.key_to_index
            .reserve(new_capacity - self.key_to_index.len());

        Ok(())
    }

    pub fn remove(&mut self, key: &IdKey) -> Option<u32> {
        if let Some(index) = self.key_to_index.remove(key) {
            self.keys[index as usize] = None;
            if self.config.enable_free_list {
                self.free_list.push_back(index);
            }
            return Some(index);
        }
        None
    }

    pub fn get_index(&self, key: &IdKey) -> Option<u32> {
        self.key_to_index.get(key).copied()
    }

    pub fn get_key(&self, index: u32) -> Option<&IdKey> {
        self.keys.get(index as usize)?.as_ref()
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

    pub fn capacity(&self) -> usize {
        self.keys.capacity()
    }

    pub fn free_count(&self) -> usize {
        self.free_list.len()
    }

    pub fn total_slots(&self) -> usize {
        self.keys.len()
    }

    pub fn reserve(&mut self, additional: usize) {
        let new_capacity = self.keys.len() + additional;
        if new_capacity > self.keys.capacity() {
            self.keys.reserve(new_capacity - self.keys.capacity());
            self.key_to_index.reserve(additional);
        }
    }

    pub fn shrink_to_fit(&mut self) {
        if self.free_list.is_empty() {
            self.keys.shrink_to_fit();
            self.key_to_index.shrink_to_fit();
        }
    }

    pub fn compact(&mut self) -> StorageResult<HashMap<u32, u32>> {
        if self.free_list.is_empty() {
            return Ok(HashMap::new());
        }

        let mut id_mapping = HashMap::new();
        let mut new_keys: Vec<Option<IdKey>> = Vec::with_capacity(self.len());
        let mut new_key_to_index = HashMap::with_capacity(self.len());

        for (old_idx, key_opt) in self.keys.iter().enumerate() {
            if let Some(key) = key_opt {
                let new_idx = new_keys.len() as u32;
                new_keys.push(Some(key.clone()));
                new_key_to_index.insert(key.clone(), new_idx);
                if old_idx as u32 != new_idx {
                    id_mapping.insert(old_idx as u32, new_idx);
                }
            }
        }

        self.keys = new_keys;
        self.key_to_index = new_key_to_index;
        self.free_list.clear();

        Ok(id_mapping)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&IdKey, u32)> {
        self.keys
            .iter()
            .enumerate()
            .filter_map(|(i, k)| k.as_ref().map(|k| (k, i as u32)))
    }

    pub fn keys(&self) -> impl Iterator<Item = &IdKey> {
        self.keys.iter().filter_map(|k| k.as_ref())
    }

    pub fn clear(&mut self) {
        self.keys.clear();
        self.key_to_index.clear();
        self.free_list.clear();
    }

    pub fn set_at(&mut self, index: u32, key: IdKey) {
        if self.key_to_index.contains_key(&key) {
            return;
        }
        while self.keys.len() <= index as usize {
            self.keys.push(None);
        }
        self.keys[index as usize] = Some(key.clone());
        self.key_to_index.insert(key, index);
    }

    pub fn memory_usage(&self) -> usize {
        let keys_size = self.keys.capacity() * std::mem::size_of::<Option<IdKey>>();
        let map_size = self.key_to_index.capacity()
            * (std::mem::size_of::<IdKey>() + std::mem::size_of::<u32>());
        let free_list_size = self.free_list.capacity() * std::mem::size_of::<u32>();
        keys_size + map_size + free_list_size
    }

    pub fn memory_size(&self) -> usize {
        self.memory_usage() + std::mem::size_of::<Self>()
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
        let mut indexer = IdIndexer::new();

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
            Some(&IdKey::Text("vertex1".to_string()))
        );
        assert_eq!(
            indexer.get_key(1),
            Some(&IdKey::Text("vertex2".to_string()))
        );
    }

    #[test]
    fn test_int_id_operations() {
        let mut indexer = IdIndexer::new();

        let idx1 = indexer.insert(IdKey::Int(100)).unwrap();
        assert_eq!(idx1, 0);

        let idx2 = indexer.insert(IdKey::Int(200)).unwrap();
        assert_eq!(idx2, 1);

        assert_eq!(indexer.get_index(&IdKey::Int(100)), Some(0));
        assert_eq!(indexer.get_index(&IdKey::Int(200)), Some(1));
        assert_eq!(indexer.get_index(&IdKey::Int(300)), None);

        assert_eq!(indexer.get_key(0), Some(&IdKey::Int(100)));
        assert_eq!(indexer.get_key(1), Some(&IdKey::Int(200)));
    }

    #[test]
    fn test_mixed_id_operations() {
        let mut indexer = IdIndexer::new();

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
        let mut indexer = IdIndexer::with_config(
            IdIndexerConfig::default()
                .with_initial_capacity(2)
                .with_growth_factor(2.0),
        );

        assert!(indexer.insert(IdKey::Text("v1".to_string())).is_ok());
        assert!(indexer.insert(IdKey::Text("v2".to_string())).is_ok());
        assert!(indexer.insert(IdKey::Text("v3".to_string())).is_ok());
        assert!(indexer.insert(IdKey::Text("v4".to_string())).is_ok());
        assert!(indexer.insert(IdKey::Text("v5".to_string())).is_ok());

        assert_eq!(indexer.len(), 5);
    }

    #[test]
    fn test_free_list_reuse() {
        let mut indexer = IdIndexer::with_config(IdIndexerConfig::default().with_free_list(true));

        let _idx1 = indexer.insert(IdKey::Text("v1".to_string())).unwrap();
        let idx2 = indexer.insert(IdKey::Text("v2".to_string())).unwrap();
        let _idx3 = indexer.insert(IdKey::Text("v3".to_string())).unwrap();

        assert_eq!(indexer.remove(&IdKey::Text("v2".to_string())), Some(idx2));
        assert_eq!(indexer.free_count(), 1);

        let idx4 = indexer.insert(IdKey::Text("v4".to_string())).unwrap();
        assert_eq!(idx4, idx2);
        assert_eq!(indexer.free_count(), 0);
    }

    #[test]
    fn test_compact() {
        let mut indexer = IdIndexer::new();

        indexer.insert(IdKey::Text("v1".to_string())).unwrap();
        indexer.insert(IdKey::Text("v2".to_string())).unwrap();
        indexer.insert(IdKey::Text("v3".to_string())).unwrap();
        indexer.insert(IdKey::Text("v4".to_string())).unwrap();

        indexer.remove(&IdKey::Text("v2".to_string()));
        indexer.remove(&IdKey::Text("v4".to_string()));

        assert_eq!(indexer.free_count(), 2);
        assert_eq!(indexer.total_slots(), 4);
        assert_eq!(indexer.len(), 2);

        let _mapping = indexer.compact().unwrap();
        assert_eq!(indexer.free_count(), 0);
        assert_eq!(indexer.total_slots(), 2);
        assert_eq!(indexer.len(), 2);
    }

    #[test]
    fn test_duplicate_insert() {
        let mut indexer = IdIndexer::new();

        assert!(indexer.insert(IdKey::Text("v1".to_string())).is_ok());
        assert!(indexer.insert(IdKey::Text("v1".to_string())).is_err());
    }

    #[test]
    fn test_max_capacity() {
        let mut indexer = IdIndexer::with_config(
            IdIndexerConfig::default()
                .with_initial_capacity(2)
                .with_max_capacity(3),
        );

        assert!(indexer.insert(IdKey::Text("v1".to_string())).is_ok());
        assert!(indexer.insert(IdKey::Text("v2".to_string())).is_ok());
        assert!(indexer.insert(IdKey::Text("v3".to_string())).is_ok());
        assert!(indexer.insert(IdKey::Text("v4".to_string())).is_err());
    }

    #[test]
    fn test_id_key_serialization() {
        let int_key = IdKey::Int(12345);
        let bytes = int_key.to_bytes();
        let restored = IdKey::from_bytes(&bytes).unwrap();
        assert_eq!(int_key, restored);

        let text_key = IdKey::Text("test_vertex".to_string());
        let bytes = text_key.to_bytes();
        let restored = IdKey::from_bytes(&bytes).unwrap();
        assert_eq!(text_key, restored);
    }

    #[test]
    fn test_id_key_serialization_edge_cases() {
        let zero_key = IdKey::Int(0);
        let bytes = zero_key.to_bytes();
        let restored = IdKey::from_bytes(&bytes).unwrap();
        assert_eq!(zero_key, restored);

        let neg_key = IdKey::Int(-12345);
        let bytes = neg_key.to_bytes();
        let restored = IdKey::from_bytes(&bytes).unwrap();
        assert_eq!(neg_key, restored);

        let empty_text = IdKey::Text("".to_string());
        let bytes = empty_text.to_bytes();
        let restored = IdKey::from_bytes(&bytes).unwrap();
        assert_eq!(empty_text, restored);
    }
}
