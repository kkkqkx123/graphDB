//! Vertex Index Management Module
//!
//! Provide functions for updating, deleting, and querying vertex indices.
//! This implementation uses in-memory storage with BTreeMap for efficient range queries.
//! Supports persistence through flush/load operations.
//! Supports MVCC (Multi-Version Concurrency Control) for snapshot isolation.
//! Supports optional key compression for memory efficiency.

use crate::core::types::Index;
use crate::core::{StorageError, StorageResult, Value};
use super::index_data_manager::{IndexEntry, Timestamp, MAX_TIMESTAMP};
use super::key_codec::{
    deserialize_value, serialize_value, CompressionConfig, IndexCompressor, KeyBuilder,
    KeyParser,
};
use crate::storage::index::index_types::IndexEstimate;
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

type IndexKey = Vec<u8>;

#[derive(Clone)]
pub struct VertexIndexManager {
    forward_index: Arc<RwLock<BTreeMap<IndexKey, IndexEntry>>>,
    reverse_index: Arc<RwLock<BTreeMap<IndexKey, IndexEntry>>>,
    compressor: Option<Arc<RwLock<IndexCompressor>>>,
}

impl VertexIndexManager {
    pub fn new() -> Self {
        Self {
            forward_index: Arc::new(RwLock::new(BTreeMap::new())),
            reverse_index: Arc::new(RwLock::new(BTreeMap::new())),
            compressor: None,
        }
    }

    pub fn with_compression(config: CompressionConfig) -> Self {
        Self {
            forward_index: Arc::new(RwLock::new(BTreeMap::new())),
            reverse_index: Arc::new(RwLock::new(BTreeMap::new())),
            compressor: Some(Arc::new(RwLock::new(IndexCompressor::new(config)))),
        }
    }

    pub fn is_compression_enabled(&self) -> bool {
        self.compressor
            .as_ref()
            .map(|c| c.read().is_enabled())
            .unwrap_or(false)
    }

    fn compress_key(&self, key: &[u8]) -> Vec<u8> {
        if let Some(ref compressor) = self.compressor {
            compressor.read().compress_key(key)
        } else {
            key.to_vec()
        }
    }

    fn decompress_key(&self, compressed: &[u8]) -> StorageResult<Vec<u8>> {
        if let Some(ref compressor) = self.compressor {
            compressor.read().decompress_key(compressed)
        } else {
            Ok(compressed.to_vec())
        }
    }

    pub fn train_compression(&self, keys: &[Vec<u8>]) -> StorageResult<()> {
        if let Some(ref compressor) = self.compressor {
            compressor.write().train_keys(keys)?;
        }
        Ok(())
    }

    pub fn compression_ratio(&self) -> Option<f64> {
        self.compressor.as_ref().and_then(|c| {
            let c = c.read();
            if c.is_enabled() {
                let forward = self.forward_index.read();
                let original: Vec<Vec<u8>> = forward.keys().map(|k| {
                    c.decompress_key(k).unwrap_or_else(|_| k.clone())
                }).collect();
                let compressed: Vec<Vec<u8>> = forward.keys().cloned().collect();
                Some(c.compression_ratio(&original, &compressed))
            } else {
                None
            }
        })
    }

    pub fn update_vertex_indexes(
        &self,
        space_id: u64,
        vertex_id: &Value,
        index_name: &str,
        props: &[(String, Value)],
    ) -> Result<(), StorageError> {
        self.update_vertex_indexes_mvcc(space_id, vertex_id, index_name, props, MAX_TIMESTAMP)
    }

    pub fn update_vertex_indexes_mvcc(
        &self,
        space_id: u64,
        vertex_id: &Value,
        index_name: &str,
        props: &[(String, Value)],
        write_ts: Timestamp,
    ) -> Result<(), StorageError> {
        let mut forward_entries: Vec<(IndexKey, IndexEntry)> = Vec::with_capacity(props.len());
        let mut reverse_entries: Vec<(IndexKey, IndexEntry)> = Vec::with_capacity(props.len());

        for (_prop_name, prop_value) in props {
            let index_key =
                KeyBuilder::build_vertex_index_key(space_id, index_name, prop_value, vertex_id)?;

            let reverse_key =
                KeyBuilder::build_vertex_reverse_key_v2(space_id, vertex_id, index_name)?;

            let entry = IndexEntry::new(write_ts);
            let compressed_forward = self.compress_key(&index_key.0);
            let compressed_reverse = self.compress_key(&reverse_key.0);
            forward_entries.push((compressed_forward, entry.clone()));
            reverse_entries.push((compressed_reverse, entry));
        }

        {
            let mut forward_index = self.forward_index.write();
            for (key, entry) in forward_entries {
                forward_index.insert(key, entry);
            }
        }
        {
            let mut reverse_index = self.reverse_index.write();
            for (key, entry) in reverse_entries {
                reverse_index.insert(key, entry);
            }
        }

        Ok(())
    }

    pub fn delete_vertex_indexes(
        &self,
        space_id: u64,
        vertex_id: &Value,
    ) -> Result<(), StorageError> {
        self.delete_vertex_indexes_mvcc(space_id, vertex_id, MAX_TIMESTAMP)
    }

    /// Delete a single specific index entry (not all indexes for the vertex)
    /// Used for undo operations to revert a specific index insertion
    pub fn delete_vertex_index_single(
        &self,
        space_id: u64,
        vertex_id: &Value,
        index_name: &str,
        prop_value: &Value,
        write_ts: Timestamp,
    ) -> Result<(), StorageError> {
        let forward_key =
            KeyBuilder::build_vertex_index_key(space_id, index_name, prop_value, vertex_id)?;
        let reverse_key =
            KeyBuilder::build_vertex_reverse_key_v2(space_id, vertex_id, index_name)?;

        let compressed_forward = self.compress_key(&forward_key.0);
        let compressed_reverse = self.compress_key(&reverse_key.0);

        {
            let mut forward_index = self.forward_index.write();
            if let Some(entry) = forward_index.get_mut(&compressed_forward) {
                entry.mark_deleted(write_ts);
            }
        }

        {
            let mut reverse_index = self.reverse_index.write();
            if let Some(entry) = reverse_index.get_mut(&compressed_reverse) {
                entry.mark_deleted(write_ts);
            }
        }

        Ok(())
    }

    pub fn delete_vertex_indexes_mvcc(
        &self,
        space_id: u64,
        vertex_id: &Value,
        write_ts: Timestamp,
    ) -> Result<(), StorageError> {
        let reverse_prefix = KeyBuilder::build_vertex_reverse_prefix_v2(space_id, vertex_id)?;
        let reverse_end = KeyBuilder::build_range_end(&reverse_prefix);

        let mut forward_keys_to_delete: Vec<IndexKey> = Vec::new();
        let mut reverse_keys_to_delete: Vec<IndexKey> = Vec::new();

        {
            let reverse_index = self.reverse_index.read();
            for (compressed_key, entry) in reverse_index.range(reverse_prefix.0.clone()..reverse_end.0) {
                if entry.is_visible_at(write_ts) {
                    let key_bytes = self.decompress_key(compressed_key)?;
                    reverse_keys_to_delete.push(compressed_key.clone());

                    if let Ok((_vertex_id_bytes, index_name)) =
                        KeyParser::parse_vertex_reverse_key_v2(&key_bytes)
                    {
                        let forward_key_start =
                            KeyBuilder::build_vertex_index_prefix(space_id, &index_name);
                        let forward_key_end = KeyBuilder::build_range_end(&forward_key_start);

                        let vertex_bytes = serialize_value(vertex_id)?;
                        let forward_index = self.forward_index.read();
                        for (fwd_compressed_key, fwd_entry) in
                            forward_index.range(forward_key_start.0.clone()..forward_key_end.0)
                        {
                            if fwd_entry.is_visible_at(write_ts) {
                                let fwd_key_bytes = self.decompress_key(fwd_compressed_key)?;
                                if let Ok(vid) =
                                    KeyParser::parse_vertex_id_from_key(&fwd_key_bytes)
                                {
                                    if vid == *vertex_id {
                                        let vid_start = fwd_key_bytes.len() - vertex_bytes.len();
                                        if fwd_key_bytes[vid_start..] == vertex_bytes {
                                            forward_keys_to_delete.push(fwd_compressed_key.clone());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        {
            let mut reverse_index = self.reverse_index.write();
            for key in &reverse_keys_to_delete {
                if let Some(entry) = reverse_index.get_mut(key) {
                    entry.mark_deleted(write_ts);
                }
            }
        }

        {
            let mut forward_index = self.forward_index.write();
            for key in &forward_keys_to_delete {
                if let Some(entry) = forward_index.get_mut(key) {
                    entry.mark_deleted(write_ts);
                }
            }
        }

        Ok(())
    }

    pub fn delete_tag_indexes(
        &self,
        space_id: u64,
        vertex_id: &Value,
        tag_name: &str,
    ) -> Result<(), StorageError> {
        let reverse_prefix = KeyBuilder::build_vertex_reverse_prefix_v2(space_id, vertex_id)?;
        let reverse_end = KeyBuilder::build_range_end(&reverse_prefix);

        let mut keys_to_delete: Vec<IndexKey> = Vec::new();

        {
            let reverse_index = self.reverse_index.read();
            for (compressed_key, _) in reverse_index.range(reverse_prefix.0.clone()..reverse_end.0) {
                let key_bytes = self.decompress_key(compressed_key)?;
                if let Ok((_vertex_id_bytes, index_name)) =
                    KeyParser::parse_vertex_reverse_key_v2(&key_bytes)
                {
                    if index_name.starts_with(tag_name) {
                        keys_to_delete.push(compressed_key.clone());
                    }
                }
            }
        }

        {
            let mut reverse_index = self.reverse_index.write();
            for key in &keys_to_delete {
                reverse_index.remove(key);
            }
        }

        Ok(())
    }

    pub fn clear_tag_index(&self, space_id: u64, index_name: &str) -> Result<(), StorageError> {
        let prefix = KeyBuilder::build_vertex_index_prefix(space_id, index_name);
        let end = KeyBuilder::build_range_end(&prefix);

        let mut keys_to_delete: Vec<IndexKey> = Vec::new();

        {
            let forward_index = self.forward_index.read();
            for (key_bytes, _) in forward_index.range(prefix.0.clone()..end.0) {
                keys_to_delete.push(key_bytes.clone());
            }
        }

        {
            let mut forward_index = self.forward_index.write();
            for key in &keys_to_delete {
                forward_index.remove(key);
            }
        }

        Ok(())
    }

    pub fn clear_all(&self) -> Result<(), StorageError> {
        {
            let mut forward_index = self.forward_index.write();
            forward_index.clear();
        }
        {
            let mut reverse_index = self.reverse_index.write();
            reverse_index.clear();
        }
        Ok(())
    }

    pub fn lookup_tag_index(
        &self,
        space_id: u64,
        index: &Index,
        value: &Value,
    ) -> Result<Vec<Value>, StorageError> {
        self.lookup_tag_index_mvcc(space_id, index, value, MAX_TIMESTAMP)
    }

    pub fn lookup_tag_index_mvcc(
        &self,
        space_id: u64,
        index: &Index,
        value: &Value,
        read_ts: Timestamp,
    ) -> Result<Vec<Value>, StorageError> {
        let prefix = KeyBuilder::build_vertex_index_prefix(space_id, &index.name);
        let end = KeyBuilder::build_range_end(&prefix);

        let mut results = Vec::new();
        let value_bytes = serialize_value(value)?;

        let forward_index = self.forward_index.read();
        for (compressed_key, entry) in forward_index.range(prefix.0.clone()..end.0) {
            if !entry.is_visible_at(read_ts) {
                continue;
            }

            let key_bytes = self.decompress_key(compressed_key)?;
            if let Ok(vertex_id) = KeyParser::parse_vertex_id_from_key(&key_bytes) {
                if key_bytes.len() > prefix.0.len() + 4 {
                    let prop_len_start = prefix.0.len();
                    let prop_value_len = u32::from_le_bytes(
                        key_bytes[prop_len_start..prop_len_start + 4]
                            .try_into()
                            .unwrap_or([0; 4]),
                    ) as usize;

                    if prop_value_len == value_bytes.len() {
                        let prop_value_start = prop_len_start + 4;
                        let stored_prop_value =
                            &key_bytes[prop_value_start..prop_value_start + prop_value_len];

                        if stored_prop_value == value_bytes.as_slice() {
                            results.push(vertex_id);
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    pub fn lookup_tag_index_range(
        &self,
        space_id: u64,
        index: &Index,
        start_value: &Value,
        end_value: &Value,
    ) -> Result<Vec<Value>, StorageError> {
        self.lookup_tag_index_range_mvcc(space_id, index, start_value, end_value, MAX_TIMESTAMP)
    }

    pub fn lookup_tag_index_range_mvcc(
        &self,
        space_id: u64,
        index: &Index,
        start_value: &Value,
        end_value: &Value,
        read_ts: Timestamp,
    ) -> Result<Vec<Value>, StorageError> {
        let prefix = KeyBuilder::build_vertex_index_prefix(space_id, &index.name);
        let start_bytes = serialize_value(start_value)?;
        let end_bytes = serialize_value(end_value)?;

        let range_start = KeyBuilder::build_vertex_index_key(
            space_id,
            &index.name,
            start_value,
            &Value::Int(i32::MIN),
        )?;
        let range_end = KeyBuilder::build_vertex_index_key(
            space_id,
            &index.name,
            end_value,
            &Value::Int(i32::MAX),
        )?;

        let mut results = Vec::new();
        let forward_index = self.forward_index.read();

        let range_bounds = range_start.0.clone()..range_end.0.clone();

        let mut estimated_capacity = 0;
        for (_, entry) in forward_index.range(range_bounds.clone()) {
            if entry.is_visible_at(read_ts) {
                estimated_capacity += 1;
            }
        }
        results.reserve(estimated_capacity.min(10000));

        for (compressed_key, entry) in forward_index.range(range_bounds) {
            if !entry.is_visible_at(read_ts) {
                continue;
            }

            let key_bytes = self.decompress_key(compressed_key)?;
            if let Ok(vertex_id) = KeyParser::parse_vertex_id_from_key(&key_bytes) {
                if key_bytes.len() > prefix.0.len() + 4 {
                    let prop_len_start = prefix.0.len();
                    let prop_value_len = u32::from_le_bytes(
                        key_bytes[prop_len_start..prop_len_start + 4]
                            .try_into()
                            .unwrap_or([0; 4]),
                    ) as usize;

                    let prop_value_start = prop_len_start + 4;
                    if prop_value_start + prop_value_len <= key_bytes.len() {
                        let stored_prop_value =
                            &key_bytes[prop_value_start..prop_value_start + prop_value_len];

                        if stored_prop_value >= start_bytes.as_slice()
                            && stored_prop_value < end_bytes.as_slice()
                        {
                            results.push(vertex_id);
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    pub fn estimate_index_entries(
        &self,
        space_id: u64,
        index_name: &str,
    ) -> Result<IndexEstimate, StorageError> {
        let prefix = KeyBuilder::build_vertex_index_prefix(space_id, index_name);
        let end = KeyBuilder::build_range_end(&prefix);

        let forward_index = self.forward_index.read();
        let mut total_entries = 0usize;
        let mut visible_entries = 0usize;
        let mut tombstone_entries = 0usize;

        for (_, entry) in forward_index.range(prefix.0.clone()..end.0) {
            total_entries += 1;
            if entry.deleted_ts.is_some() {
                tombstone_entries += 1;
            } else {
                visible_entries += 1;
            }
        }

        Ok(IndexEstimate {
            total_entries,
            visible_entries,
            tombstone_entries,
        })
    }

    pub fn scan_index_entries(
        &self,
        space_id: u64,
        index: &Index,
        limit: usize,
    ) -> Result<Vec<(Value, Value)>, StorageError> {
        self.scan_index_entries_mvcc(space_id, index, limit, MAX_TIMESTAMP)
    }

    pub fn scan_index_entries_mvcc(
        &self,
        space_id: u64,
        index: &Index,
        limit: usize,
        read_ts: Timestamp,
    ) -> Result<Vec<(Value, Value)>, StorageError> {
        let prefix = KeyBuilder::build_vertex_index_prefix(space_id, &index.name);
        let end = KeyBuilder::build_range_end(&prefix);

        let mut results = Vec::with_capacity(limit.min(1000));
        let forward_index = self.forward_index.read();

        for (compressed_key, entry) in forward_index.range(prefix.0.clone()..end.0) {
            if results.len() >= limit {
                break;
            }
            if !entry.is_visible_at(read_ts) {
                continue;
            }

            let key_bytes = self.decompress_key(compressed_key)?;
            if let Ok(vertex_id) = KeyParser::parse_vertex_id_from_key(&key_bytes) {
                if key_bytes.len() > prefix.0.len() + 4 {
                    let prop_len_start = prefix.0.len();
                    let prop_value_len = u32::from_le_bytes(
                        key_bytes[prop_len_start..prop_len_start + 4]
                            .try_into()
                            .unwrap_or([0; 4]),
                    ) as usize;

                    let prop_value_start = prop_len_start + 4;
                    if prop_value_start + prop_value_len <= key_bytes.len() {
                        let stored_prop_value =
                            &key_bytes[prop_value_start..prop_value_start + prop_value_len];
                        if let Ok(prop_value) = deserialize_value(stored_prop_value)
                        {
                            results.push((prop_value, vertex_id));
                        }
                    }
                }
            }
        }

        Ok(results)
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

    pub fn entry_count(&self) -> (usize, usize) {
        let forward_count = self.forward_index.read().len();
        let reverse_count = self.reverse_index.read().len();
        (forward_count, reverse_count)
    }

    pub fn gc_tombstones(&self, safe_ts: Timestamp) -> Result<usize, StorageError> {
        let mut removed_count = 0usize;

        {
            let mut forward_index = self.forward_index.write();
            let keys_to_remove: Vec<IndexKey> = forward_index
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
            let keys_to_remove: Vec<IndexKey> = reverse_index
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

    // ========================================================================
    // Composite Index Support
    // ========================================================================

    /// Update composite vertex indexes for multi-field indexes
    ///
    /// This method creates index entries for composite (multi-field) indexes.
    /// Each field value is encoded into the key for efficient range queries.
    pub fn update_composite_vertex_indexes(
        &self,
        space_id: u64,
        vertex_id: &Value,
        index_name: &str,
        field_values: &[Value],
    ) -> Result<(), StorageError> {
        self.update_composite_vertex_indexes_mvcc(
            space_id,
            vertex_id,
            index_name,
            field_values,
            MAX_TIMESTAMP,
        )
    }

    /// Update composite vertex indexes with MVCC timestamp
    pub fn update_composite_vertex_indexes_mvcc(
        &self,
        space_id: u64,
        vertex_id: &Value,
        index_name: &str,
        field_values: &[Value],
        write_ts: Timestamp,
    ) -> Result<(), StorageError> {
        let index_key = KeyBuilder::build_composite_vertex_index_key(
            space_id,
            index_name,
            field_values,
            vertex_id,
        )?;

        let reverse_key =
            KeyBuilder::build_vertex_reverse_key_v2(space_id, vertex_id, index_name)?;

        let entry = IndexEntry::new(write_ts);

        {
            let mut forward_index = self.forward_index.write();
            forward_index.insert(index_key.0, entry.clone());
        }
        {
            let mut reverse_index = self.reverse_index.write();
            reverse_index.insert(reverse_key.0, entry);
        }

        Ok(())
    }

    /// Lookup composite vertex index by field values
    ///
    /// Returns vertex IDs that match all specified field values.
    pub fn lookup_composite_tag_index(
        &self,
        space_id: u64,
        index_name: &str,
        field_values: &[Value],
    ) -> Result<Vec<Value>, StorageError> {
        self.lookup_composite_tag_index_mvcc(space_id, index_name, field_values, MAX_TIMESTAMP)
    }

    /// Lookup composite vertex index with MVCC timestamp
    pub fn lookup_composite_tag_index_mvcc(
        &self,
        space_id: u64,
        index_name: &str,
        field_values: &[Value],
        read_ts: Timestamp,
    ) -> Result<Vec<Value>, StorageError> {
        let prefix = KeyBuilder::build_vertex_index_prefix(space_id, index_name);
        let end = KeyBuilder::build_range_end(&prefix);

        let mut results = Vec::new();

        let forward_index = self.forward_index.read();
        for (key_bytes, entry) in forward_index.range(prefix.0.clone()..end.0) {
            if !entry.is_visible_at(read_ts) {
                continue;
            }

            if let Ok((stored_values, vertex_id)) =
                KeyParser::parse_composite_vertex_index_key(key_bytes)
            {
                if stored_values.len() == field_values.len() {
                    let matches = stored_values
                        .iter()
                        .zip(field_values.iter())
                        .all(|(stored, query)| stored == query);

                    if matches {
                        results.push(vertex_id);
                    }
                }
            }
        }

        Ok(results)
    }

    /// Lookup composite vertex index by prefix (partial field match)
    ///
    /// Returns vertex IDs where the first N fields match the provided values.
    pub fn lookup_composite_tag_index_prefix(
        &self,
        space_id: u64,
        index_name: &str,
        prefix_values: &[Value],
        read_ts: Timestamp,
    ) -> Result<Vec<Value>, StorageError> {
        let prefix = KeyBuilder::build_vertex_index_prefix(space_id, index_name);
        let end = KeyBuilder::build_range_end(&prefix);

        let mut results = Vec::new();

        let forward_index = self.forward_index.read();
        for (key_bytes, entry) in forward_index.range(prefix.0.clone()..end.0) {
            if !entry.is_visible_at(read_ts) {
                continue;
            }

            if let Ok((stored_values, vertex_id)) =
                KeyParser::parse_composite_vertex_index_key(key_bytes)
            {
                if stored_values.len() >= prefix_values.len() {
                    let matches = stored_values[..prefix_values.len()]
                        .iter()
                        .zip(prefix_values.iter())
                        .all(|(stored, query)| stored == query);

                    if matches {
                        results.push(vertex_id);
                    }
                }
            }
        }

        Ok(results)
    }

    // ========================================================================
    // Native ID Type Support (CSR-compatible)
    // ========================================================================

    /// Update vertex indexes with native VertexId
    pub fn update_vertex_indexes_native(
        &self,
        space_id: u64,
        vertex_id: u64,
        index_name: &str,
        props: &[(String, Value)],
    ) -> Result<(), StorageError> {
        self.update_vertex_indexes_native_mvcc(
            space_id,
            vertex_id,
            index_name,
            props,
            MAX_TIMESTAMP,
        )
    }

    /// Update vertex indexes with native VertexId and MVCC timestamp
    pub fn update_vertex_indexes_native_mvcc(
        &self,
        space_id: u64,
        vertex_id: u64,
        index_name: &str,
        props: &[(String, Value)],
        write_ts: Timestamp,
    ) -> Result<(), StorageError> {
        let mut forward_entries: Vec<(IndexKey, IndexEntry)> = Vec::with_capacity(props.len());
        let mut reverse_entries: Vec<(IndexKey, IndexEntry)> = Vec::with_capacity(props.len());

        for (_prop_name, prop_value) in props {
            let index_key = KeyBuilder::build_vertex_index_key_native(
                space_id, index_name, prop_value, vertex_id,
            )?;

            let reverse_key =
                KeyBuilder::build_vertex_reverse_key_native(space_id, vertex_id, index_name);

            let entry = IndexEntry::new(write_ts);
            forward_entries.push((index_key.0, entry.clone()));
            reverse_entries.push((reverse_key.0, entry));
        }

        {
            let mut forward_index = self.forward_index.write();
            for (key, entry) in forward_entries {
                forward_index.insert(key, entry);
            }
        }
        {
            let mut reverse_index = self.reverse_index.write();
            for (key, entry) in reverse_entries {
                reverse_index.insert(key, entry);
            }
        }

        Ok(())
    }

    /// Delete vertex indexes with native VertexId
    pub fn delete_vertex_indexes_native(
        &self,
        space_id: u64,
        vertex_id: u64,
    ) -> Result<(), StorageError> {
        self.delete_vertex_indexes_native_mvcc(space_id, vertex_id, MAX_TIMESTAMP)
    }

    /// Delete vertex indexes with native VertexId and MVCC timestamp
    pub fn delete_vertex_indexes_native_mvcc(
        &self,
        space_id: u64,
        vertex_id: u64,
        write_ts: Timestamp,
    ) -> Result<(), StorageError> {
        let reverse_prefix = KeyBuilder::build_vertex_reverse_prefix_native(space_id, vertex_id);
        let reverse_end = KeyBuilder::build_range_end(&reverse_prefix);

        let mut forward_keys_to_delete: Vec<IndexKey> = Vec::new();
        let mut reverse_keys_to_delete: Vec<IndexKey> = Vec::new();

        {
            let reverse_index = self.reverse_index.read();
            for (key_bytes, entry) in reverse_index.range(reverse_prefix.0.clone()..reverse_end.0) {
                if entry.is_visible_at(write_ts) {
                    reverse_keys_to_delete.push(key_bytes.clone());

                    if let Ok((_vertex_id, index_name)) =
                        KeyParser::parse_vertex_reverse_key_native(key_bytes)
                    {
                        let forward_key_start =
                            KeyBuilder::build_vertex_index_prefix(space_id, &index_name);
                        let forward_key_end = KeyBuilder::build_range_end(&forward_key_start);

                        let forward_index = self.forward_index.read();
                        for (fwd_key_bytes, fwd_entry) in
                            forward_index.range(forward_key_start.0.clone()..forward_key_end.0)
                        {
                            if fwd_entry.is_visible_at(write_ts) {
                                if let Ok(vid) =
                                    KeyParser::parse_vertex_id_from_key_native(fwd_key_bytes)
                                {
                                    if vid == vertex_id {
                                        forward_keys_to_delete.push(fwd_key_bytes.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        {
            let mut reverse_index = self.reverse_index.write();
            for key in &reverse_keys_to_delete {
                if let Some(entry) = reverse_index.get_mut(key) {
                    entry.mark_deleted(write_ts);
                }
            }
        }

        {
            let mut forward_index = self.forward_index.write();
            for key in &forward_keys_to_delete {
                if let Some(entry) = forward_index.get_mut(key) {
                    entry.mark_deleted(write_ts);
                }
            }
        }

        Ok(())
    }

    /// Lookup tag index with native VertexId return type
    pub fn lookup_tag_index_native(
        &self,
        space_id: u64,
        index: &Index,
        value: &Value,
    ) -> Result<Vec<u64>, StorageError> {
        self.lookup_tag_index_native_mvcc(space_id, index, value, MAX_TIMESTAMP)
    }

    /// Lookup tag index with native VertexId return type and MVCC timestamp
    pub fn lookup_tag_index_native_mvcc(
        &self,
        space_id: u64,
        index: &Index,
        value: &Value,
        read_ts: Timestamp,
    ) -> Result<Vec<u64>, StorageError> {
        let prefix = KeyBuilder::build_vertex_index_prefix(space_id, &index.name);
        let end = KeyBuilder::build_range_end(&prefix);

        let mut results = Vec::new();
        let value_bytes = serialize_value(value)?;

        let forward_index = self.forward_index.read();
        for (key_bytes, entry) in forward_index.range(prefix.0.clone()..end.0) {
            if !entry.is_visible_at(read_ts) {
                continue;
            }

            if let Ok(vertex_id) = KeyParser::parse_vertex_id_from_key_native(key_bytes) {
                if key_bytes.len() > prefix.0.len() + 4 {
                    let prop_len_start = prefix.0.len();
                    let prop_value_len = u32::from_le_bytes(
                        key_bytes[prop_len_start..prop_len_start + 4]
                            .try_into()
                            .unwrap_or([0; 4]),
                    ) as usize;

                    if prop_value_len == value_bytes.len() {
                        let prop_value_start = prop_len_start + 4;
                        let stored_prop_value =
                            &key_bytes[prop_value_start..prop_value_start + prop_value_len];

                        if stored_prop_value == value_bytes.as_slice() {
                            results.push(vertex_id);
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    /// Lookup tag index range with native VertexId return type
    pub fn lookup_tag_index_range_native(
        &self,
        space_id: u64,
        index: &Index,
        start_value: &Value,
        end_value: &Value,
    ) -> Result<Vec<u64>, StorageError> {
        self.lookup_tag_index_range_native_mvcc(
            space_id,
            index,
            start_value,
            end_value,
            MAX_TIMESTAMP,
        )
    }

    /// Lookup tag index range with native VertexId return type and MVCC timestamp
    pub fn lookup_tag_index_range_native_mvcc(
        &self,
        space_id: u64,
        index: &Index,
        start_value: &Value,
        end_value: &Value,
        read_ts: Timestamp,
    ) -> Result<Vec<u64>, StorageError> {
        let start_bytes = serialize_value(start_value)?;
        let end_bytes = serialize_value(end_value)?;

        let range_start =
            KeyBuilder::build_vertex_index_key_native(space_id, &index.name, start_value, 0)?;
        let range_end = KeyBuilder::build_vertex_index_key_native(
            space_id,
            &index.name,
            end_value,
            u64::MAX,
        )?;

        let mut results = Vec::new();
        let forward_index = self.forward_index.read();

        for (key_bytes, entry) in forward_index.range(range_start.0.clone()..range_end.0.clone()) {
            if !entry.is_visible_at(read_ts) {
                continue;
            }

            if let Ok(vertex_id) = KeyParser::parse_vertex_id_from_key_native(key_bytes) {
                let prefix = KeyBuilder::build_vertex_index_prefix(space_id, &index.name);
                if key_bytes.len() > prefix.0.len() + 4 {
                    let prop_len_start = prefix.0.len();
                    let prop_value_len = u32::from_le_bytes(
                        key_bytes[prop_len_start..prop_len_start + 4]
                            .try_into()
                            .unwrap_or([0; 4]),
                    ) as usize;

                    let prop_value_start = prop_len_start + 4;
                    if prop_value_start + prop_value_len <= key_bytes.len() {
                        let stored_prop_value =
                            &key_bytes[prop_value_start..prop_value_start + prop_value_len];

                        if stored_prop_value >= start_bytes.as_slice()
                            && stored_prop_value < end_bytes.as_slice()
                        {
                            results.push(vertex_id);
                        }
                    }
                }
            }
        }

        Ok(results)
    }
}

impl Default for VertexIndexManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{Index, IndexConfig, IndexField, IndexType};
    use crate::core::Value;

    fn create_test_index(name: &str, schema_name: &str) -> Index {
        Index::new(IndexConfig {
            id: 1,
            name: name.to_string(),
            space_id: 1,
            schema_name: schema_name.to_string(),
            fields: vec![IndexField::new(
                "name".to_string(),
                Value::String("".to_string()),
                false,
            )],
            properties: vec![],
            index_type: IndexType::TagIndex,
            is_unique: false,
            partial_condition: None,
        })
    }

    #[test]
    fn test_update_and_lookup_vertex_index() {
        let manager = VertexIndexManager::new();

        let space_id = 1u64;
        let vertex_id = Value::Int(1);
        let index_name = "idx_person_name";
        let props = vec![("name".to_string(), Value::String("Alice".to_string()))];

        manager
            .update_vertex_indexes(space_id, &vertex_id, index_name, &props)
            .expect("Failed to update vertex indexes");

        let index = create_test_index(index_name, "person");

        let results = manager
            .lookup_tag_index(space_id, &index, &Value::String("Alice".to_string()))
            .expect("Failed to lookup tag index");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], vertex_id);

        let empty_results = manager
            .lookup_tag_index(space_id, &index, &Value::String("Bob".to_string()))
            .expect("Failed to lookup tag index");
        assert!(empty_results.is_empty());
    }

    #[test]
    fn test_delete_vertex_indexes() {
        let manager = VertexIndexManager::new();

        let space_id = 1u64;
        let vertex_id1 = Value::Int(1);
        let vertex_id2 = Value::Int(2);
        let index_name = "idx_person_name";

        let props1 = vec![("name".to_string(), Value::String("Alice".to_string()))];
        let props2 = vec![("name".to_string(), Value::String("Bob".to_string()))];

        manager
            .update_vertex_indexes(space_id, &vertex_id1, index_name, &props1)
            .expect("Failed to update vertex indexes");
        manager
            .update_vertex_indexes(space_id, &vertex_id2, index_name, &props2)
            .expect("Failed to update vertex indexes");

        let index = create_test_index(index_name, "person");

        let results1 = manager
            .lookup_tag_index(space_id, &index, &Value::String("Alice".to_string()))
            .expect("Failed to lookup tag index");
        assert_eq!(results1.len(), 1);

        let results2 = manager
            .lookup_tag_index(space_id, &index, &Value::String("Bob".to_string()))
            .expect("Failed to lookup tag index");
        assert_eq!(results2.len(), 1);

        manager
            .delete_vertex_indexes(space_id, &vertex_id1)
            .expect("Failed to delete vertex indexes");

        let results1_after = manager
            .lookup_tag_index(space_id, &index, &Value::String("Alice".to_string()))
            .expect("Failed to lookup tag index");
        assert!(results1_after.is_empty());

        let results2_after = manager
            .lookup_tag_index(space_id, &index, &Value::String("Bob".to_string()))
            .expect("Failed to lookup tag index");
        assert_eq!(results2_after.len(), 1);
    }

    #[test]
    fn test_multiple_properties_index() {
        let manager = VertexIndexManager::new();

        let space_id = 1u64;
        let vertex_id = Value::Int(1);
        let index_name = "idx_person";

        let props = vec![
            ("name".to_string(), Value::String("Alice".to_string())),
            ("age".to_string(), Value::Int(30)),
        ];

        manager
            .update_vertex_indexes(space_id, &vertex_id, index_name, &props)
            .expect("Failed to update vertex indexes");

        let index = create_test_index(index_name, "person");

        let results_name = manager
            .lookup_tag_index(space_id, &index, &Value::String("Alice".to_string()))
            .expect("Failed to lookup tag index");
        assert_eq!(results_name.len(), 1);
        assert_eq!(results_name[0], vertex_id);

        let results_age = manager
            .lookup_tag_index(space_id, &index, &Value::Int(30))
            .expect("Failed to lookup tag index");
        assert_eq!(results_age.len(), 1);
        assert_eq!(results_age[0], vertex_id);
    }
}
