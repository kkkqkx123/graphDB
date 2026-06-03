//! Generic Index Manager
//!
//! This module provides a generic implementation of index management
//! that can be used for both vertex and edge indexes.

use crate::core::types::Timestamp;
use crate::core::{StorageError, StorageResult};
use crate::storage::index::index_data_manager::IndexEntry;
use crate::storage::index::key_codec::{IndexKeyGenerator, SecondaryIndexKey};
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::path::Path;
use std::sync::Arc;

/// Generic index manager
///
/// Provides common functionality for index management including:
/// - In-memory storage with BTreeMap
/// - Persistence (flush/load)
/// - GC for tombstones
pub struct GenericIndexManager<K: IndexKeyGenerator> {
    forward_index: Arc<RwLock<BTreeMap<SecondaryIndexKey, IndexEntry>>>,
    reverse_index: Arc<RwLock<BTreeMap<SecondaryIndexKey, IndexEntry>>>,
    _marker: PhantomData<K>,
}

impl<K: IndexKeyGenerator> Clone for GenericIndexManager<K> {
    fn clone(&self) -> Self {
        Self {
            forward_index: Arc::clone(&self.forward_index),
            reverse_index: Arc::clone(&self.reverse_index),
            _marker: PhantomData,
        }
    }
}

impl<K: IndexKeyGenerator> GenericIndexManager<K> {
    pub fn new() -> Self {
        Self {
            forward_index: Arc::new(RwLock::new(BTreeMap::new())),
            reverse_index: Arc::new(RwLock::new(BTreeMap::new())),
            _marker: PhantomData,
        }
    }

    #[cfg(test)]
    pub(crate) fn entry_count(&self) -> (usize, usize) {
        (
            self.forward_index.read().len(),
            self.reverse_index.read().len(),
        )
    }

    pub fn gc_tombstones(&self, safe_ts: Timestamp) -> Result<usize, StorageError> {
        let mut removed_count = 0usize;

        {
            let mut forward_index = self.forward_index.write();
            let keys_to_remove: Vec<SecondaryIndexKey> = forward_index
                .iter()
                .filter(|(_, entry)| {
                    entry
                        .deleted_ts
                        .is_some_and(|deleted_ts| deleted_ts < safe_ts)
                })
                .map(|(key, _)| key.clone())
                .collect();

            removed_count += keys_to_remove.len();
            for key in &keys_to_remove {
                forward_index.remove(key);
            }
        }

        {
            let mut reverse_index = self.reverse_index.write();
            let keys_to_remove: Vec<SecondaryIndexKey> = reverse_index
                .iter()
                .filter(|(_, entry)| {
                    entry
                        .deleted_ts
                        .is_some_and(|deleted_ts| deleted_ts < safe_ts)
                })
                .map(|(key, _)| key.clone())
                .collect();

            removed_count += keys_to_remove.len();
            for key in &keys_to_remove {
                reverse_index.remove(key);
            }
        }

        Ok(removed_count)
    }

    pub fn gc_tombstones_incremental(
        &self,
        safe_ts: Timestamp,
        batch_size: usize,
    ) -> Result<usize, StorageError> {
        let mut total_removed = 0usize;

        {
            let mut forward_index = self.forward_index.write();
            let mut keys_to_remove = Vec::with_capacity(batch_size.min(1000));

            for (key, entry) in forward_index.iter() {
                if keys_to_remove.len() >= batch_size {
                    break;
                }
                if entry
                    .deleted_ts
                    .is_some_and(|deleted_ts| deleted_ts < safe_ts)
                {
                    keys_to_remove.push(key.clone());
                }
            }

            total_removed += keys_to_remove.len();
            for key in &keys_to_remove {
                forward_index.remove(key);
            }
        }

        if total_removed >= batch_size {
            return Ok(total_removed);
        }

        {
            let mut reverse_index = self.reverse_index.write();
            let remaining = batch_size - total_removed;
            let mut keys_to_remove = Vec::with_capacity(remaining.min(1000));

            for (key, entry) in reverse_index.iter() {
                if keys_to_remove.len() >= remaining {
                    break;
                }
                if entry
                    .deleted_ts
                    .is_some_and(|deleted_ts| deleted_ts < safe_ts)
                {
                    keys_to_remove.push(key.clone());
                }
            }

            total_removed += keys_to_remove.len();
            for key in &keys_to_remove {
                reverse_index.remove(key);
            }
        }

        Ok(total_removed)
    }

    pub fn tombstone_count(&self) -> usize {
        let forward_count = self
            .forward_index
            .read()
            .iter()
            .filter(|(_, entry)| entry.deleted_ts.is_some())
            .count();

        let reverse_count = self
            .reverse_index
            .read()
            .iter()
            .filter(|(_, entry)| entry.deleted_ts.is_some())
            .count();

        forward_count + reverse_count
    }

    pub fn flush<P: AsRef<Path>>(&self, path: P) -> StorageResult<()> {
        use std::fs;

        let path = path.as_ref();
        fs::create_dir_all(path)?;

        self.flush_forward_index(&path.join("forward_index.bin"))?;
        self.flush_reverse_index(&path.join("reverse_index.bin"))?;

        Ok(())
    }

    fn flush_forward_index(&self, path: &Path) -> StorageResult<()> {
        use std::fs::File;
        use std::io::Write;

        let mut file = File::create(path)?;

        let forward_index = self.forward_index.read();
        let count = forward_index.len() as u64;
        file.write_all(&count.to_le_bytes())?;

        for (key, entry) in forward_index.iter() {
            file.write_all(&(key.len() as u32).to_le_bytes())?;
            file.write_all(key)?;
            file.write_all(&entry.created_ts.to_le_bytes())?;
            if let Some(deleted_ts) = entry.deleted_ts {
                file.write_all(&[1u8])?;
                file.write_all(&deleted_ts.to_le_bytes())?;
            } else {
                file.write_all(&[0u8])?;
            }
        }

        Ok(())
    }

    fn flush_reverse_index(&self, path: &Path) -> StorageResult<()> {
        use std::fs::File;
        use std::io::Write;

        let mut file = File::create(path)?;

        let reverse_index = self.reverse_index.read();
        let count = reverse_index.len() as u64;
        file.write_all(&count.to_le_bytes())?;

        for (key, entry) in reverse_index.iter() {
            file.write_all(&(key.len() as u32).to_le_bytes())?;
            file.write_all(key)?;
            file.write_all(&entry.created_ts.to_le_bytes())?;
            if let Some(deleted_ts) = entry.deleted_ts {
                file.write_all(&[1u8])?;
                file.write_all(&deleted_ts.to_le_bytes())?;
            } else {
                file.write_all(&[0u8])?;
            }
        }

        Ok(())
    }

    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> StorageResult<()> {
        let path = path.as_ref();

        self.load_forward_index(&path.join("forward_index.bin"))?;
        self.load_reverse_index(&path.join("reverse_index.bin"))?;

        Ok(())
    }

    fn load_forward_index(&mut self, path: &Path) -> StorageResult<()> {
        use std::fs::File;
        use std::io::Read;

        if !path.exists() {
            return Ok(());
        }

        let mut file = File::open(path)?;

        let mut count_bytes = [0u8; 8];
        file.read_exact(&mut count_bytes)?;
        let count = u64::from_le_bytes(count_bytes);

        let mut forward_index = self.forward_index.write();
        forward_index.clear();

        for _ in 0..count {
            let mut key_len_bytes = [0u8; 4];
            file.read_exact(&mut key_len_bytes)?;
            let key_len = u32::from_le_bytes(key_len_bytes) as usize;

            let mut key = vec![0u8; key_len];
            file.read_exact(&mut key)?;

            let mut created_ts_bytes = [0u8; 4];
            file.read_exact(&mut created_ts_bytes)?;
            let created_ts = u32::from_le_bytes(created_ts_bytes);

            let mut has_deleted = [0u8; 1];
            file.read_exact(&mut has_deleted)?;
            let deleted_ts = if has_deleted[0] == 1 {
                let mut deleted_ts_bytes = [0u8; 4];
                file.read_exact(&mut deleted_ts_bytes)?;
                Some(u32::from_le_bytes(deleted_ts_bytes))
            } else {
                None
            };

            let entry = IndexEntry {
                created_ts,
                deleted_ts,
            };
            forward_index.insert(key, entry);
        }

        Ok(())
    }

    fn load_reverse_index(&mut self, path: &Path) -> StorageResult<()> {
        use std::fs::File;
        use std::io::Read;

        if !path.exists() {
            return Ok(());
        }

        let mut file = File::open(path)?;

        let mut count_bytes = [0u8; 8];
        file.read_exact(&mut count_bytes)?;
        let count = u64::from_le_bytes(count_bytes);

        let mut reverse_index = self.reverse_index.write();
        reverse_index.clear();

        for _ in 0..count {
            let mut key_len_bytes = [0u8; 4];
            file.read_exact(&mut key_len_bytes)?;
            let key_len = u32::from_le_bytes(key_len_bytes) as usize;

            let mut key = vec![0u8; key_len];
            file.read_exact(&mut key)?;

            let mut created_ts_bytes = [0u8; 4];
            file.read_exact(&mut created_ts_bytes)?;
            let created_ts = u32::from_le_bytes(created_ts_bytes);

            let mut has_deleted = [0u8; 1];
            file.read_exact(&mut has_deleted)?;
            let deleted_ts = if has_deleted[0] == 1 {
                let mut deleted_ts_bytes = [0u8; 4];
                file.read_exact(&mut deleted_ts_bytes)?;
                Some(u32::from_le_bytes(deleted_ts_bytes))
            } else {
                None
            };

            let entry = IndexEntry {
                created_ts,
                deleted_ts,
            };
            reverse_index.insert(key, entry);
        }

        Ok(())
    }

    pub fn forward_index(&self) -> &Arc<RwLock<BTreeMap<SecondaryIndexKey, IndexEntry>>> {
        &self.forward_index
    }

    pub fn reverse_index(&self) -> &Arc<RwLock<BTreeMap<SecondaryIndexKey, IndexEntry>>> {
        &self.reverse_index
    }
}

impl<K: IndexKeyGenerator> Default for GenericIndexManager<K> {
    fn default() -> Self {
        Self::new()
    }
}
