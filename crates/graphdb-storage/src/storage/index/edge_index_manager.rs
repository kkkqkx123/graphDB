//! Edge Index Management Module
//!
//! Provide functions for updating, deleting, and querying edge indexes.
//! This implementation uses in-memory storage with BTreeMap for efficient range queries.
//! Supports persistence through flush/load operations.
//! Supports MVCC (Multi-Version Concurrency Control) for snapshot isolation.
//! Supports optional key compression for memory efficiency.

use crate::core::types::{Index, Timestamp, MAX_TIMESTAMP};
use crate::core::{StorageError, StorageResult, Value};
use crate::storage::index::generic_index_manager::GenericIndexManager;
use crate::storage::index::index_data_manager::IndexEntry;
use crate::storage::index::index_types::IndexEstimate;
use crate::storage::index::key_codec::{
    deserialize_value, serialize_value, CompressionConfig, EdgeIndexKeyGen, KeyBuilder, KeyParser,
    SecondaryIndexKey,
};
use std::path::Path;

#[derive(Clone)]
pub struct EdgeIndexManager {
    base: GenericIndexManager<EdgeIndexKeyGen>,
}

impl EdgeIndexManager {
    pub fn new() -> Self {
        Self {
            base: GenericIndexManager::new(),
        }
    }

    pub fn with_compression(config: CompressionConfig) -> Self {
        Self {
            base: GenericIndexManager::with_compression(config),
        }
    }

    pub fn is_compression_enabled(&self) -> bool {
        self.base.is_compression_enabled()
    }

    fn compress_key(&self, key: &[u8]) -> Vec<u8> {
        self.base.compress_key_public(key)
    }

    fn decompress_key(&self, compressed: &[u8]) -> StorageResult<Vec<u8>> {
        self.base.decompress_key_public(compressed)
    }

    pub fn train_compression(&self, keys: &[Vec<u8>]) -> StorageResult<()> {
        self.base.train_compression(keys)
    }

    pub fn compression_ratio(&self) -> Option<f64> {
        self.base.compression_ratio()
    }

    pub fn update_edge_indexes(
        &self,
        space_id: u64,
        src: &Value,
        dst: &Value,
        index_name: &str,
        props: &[(String, Value)],
    ) -> Result<(), StorageError> {
        self.update_edge_indexes_mvcc(space_id, src, dst, index_name, props, MAX_TIMESTAMP)
    }

    pub fn update_edge_indexes_mvcc(
        &self,
        space_id: u64,
        src: &Value,
        dst: &Value,
        index_name: &str,
        props: &[(String, Value)],
        write_ts: Timestamp,
    ) -> Result<(), StorageError> {
        let mut forward_entries: Vec<(SecondaryIndexKey, IndexEntry)> =
            Vec::with_capacity(props.len());
        let mut reverse_entries: Vec<(SecondaryIndexKey, IndexEntry)> =
            Vec::with_capacity(props.len());

        for (_prop_name, prop_value) in props {
            let index_key =
                KeyBuilder::build_edge_index_key(space_id, index_name, prop_value, src, dst)?;

            let reverse_key =
                KeyBuilder::build_edge_reverse_key_v2(space_id, src, dst, index_name)?;

            let entry = IndexEntry::new(write_ts);
            let compressed_forward = self.compress_key(&index_key.0);
            let compressed_reverse = self.compress_key(&reverse_key.0);
            forward_entries.push((compressed_forward, entry.clone()));
            reverse_entries.push((compressed_reverse, entry));
        }

        {
            let mut forward_index = self.base.forward_index().write();
            for (key, entry) in forward_entries {
                forward_index.insert(key, entry);
            }
        }
        {
            let mut reverse_index = self.base.reverse_index().write();
            for (key, entry) in reverse_entries {
                reverse_index.insert(key, entry);
            }
        }

        Ok(())
    }

    pub fn delete_edge_indexes(
        &self,
        space_id: u64,
        src: &Value,
        dst: &Value,
        index_names: &[String],
    ) -> Result<(), StorageError> {
        self.delete_edge_indexes_mvcc(space_id, src, dst, index_names, MAX_TIMESTAMP)
    }

    pub fn delete_edge_index_single(
        &self,
        space_id: u64,
        src: &Value,
        dst: &Value,
        index_name: &str,
        prop_value: &Value,
        write_ts: Timestamp,
    ) -> Result<(), StorageError> {
        let forward_key =
            KeyBuilder::build_edge_index_key(space_id, index_name, prop_value, src, dst)?;
        let reverse_key = KeyBuilder::build_edge_reverse_key_v2(space_id, src, dst, index_name)?;

        let compressed_forward = self.compress_key(&forward_key.0);
        let compressed_reverse = self.compress_key(&reverse_key.0);

        {
            let mut forward_index = self.base.forward_index().write();
            if let Some(entry) = forward_index.get_mut(&compressed_forward) {
                entry.mark_deleted(write_ts);
            }
        }

        {
            let mut reverse_index = self.base.reverse_index().write();
            if let Some(entry) = reverse_index.get_mut(&compressed_reverse) {
                entry.mark_deleted(write_ts);
            }
        }

        Ok(())
    }

    pub fn delete_edge_indexes_mvcc(
        &self,
        space_id: u64,
        src: &Value,
        dst: &Value,
        index_names: &[String],
        write_ts: Timestamp,
    ) -> Result<(), StorageError> {
        let reverse_prefix = KeyBuilder::build_edge_reverse_prefix_v2_with_dst(space_id, src, dst)?;
        let reverse_end = KeyBuilder::build_range_end(&reverse_prefix);

        let mut forward_keys_to_delete: Vec<SecondaryIndexKey> = Vec::new();
        let mut reverse_keys_to_delete: Vec<SecondaryIndexKey> = Vec::new();

        {
            let reverse_index = self.base.reverse_index().read();
            for (compressed_key, entry) in
                reverse_index.range(reverse_prefix.0.clone()..reverse_end.0)
            {
                if !entry.is_visible_at(write_ts) {
                    continue;
                }

                let key_bytes = self.decompress_key(compressed_key)?;
                if let Ok((_src_bytes, _dst_bytes, index_name)) =
                    KeyParser::parse_edge_reverse_key_v2(&key_bytes)
                {
                    if index_names.contains(&index_name) {
                        reverse_keys_to_delete.push(compressed_key.clone());

                        let forward_key_start =
                            KeyBuilder::build_edge_index_prefix(space_id, &index_name);
                        let forward_key_end = KeyBuilder::build_range_end(&forward_key_start);

                        let src_bytes = serialize_value(src)?;
                        let dst_bytes = serialize_value(dst)?;
                        let forward_index = self.base.forward_index().read();
                        for (fwd_compressed_key, fwd_entry) in
                            forward_index.range(forward_key_start.0.clone()..forward_key_end.0)
                        {
                            if !fwd_entry.is_visible_at(write_ts) {
                                continue;
                            }

                            let fwd_key_bytes = self.decompress_key(fwd_compressed_key)?;
                            if fwd_key_bytes.len() >= forward_key_start.0.len() + 4 {
                                let prop_len_start = forward_key_start.0.len();
                                let prop_value_len = u32::from_le_bytes(
                                    fwd_key_bytes[prop_len_start..prop_len_start + 4]
                                        .try_into()
                                        .unwrap_or([0; 4]),
                                ) as usize;

                                let src_start = forward_key_start.0.len() + 4 + prop_value_len + 4;
                                if fwd_key_bytes.len() >= src_start {
                                    let src_len = u32::from_le_bytes(
                                        fwd_key_bytes[src_start - 4..src_start]
                                            .try_into()
                                            .unwrap_or([0; 4]),
                                    ) as usize;
                                    if fwd_key_bytes.len() >= src_start + src_len + 4 {
                                        let dst_len_start = src_start + src_len;
                                        let dst_len = u32::from_le_bytes(
                                            fwd_key_bytes[dst_len_start..dst_len_start + 4]
                                                .try_into()
                                                .unwrap_or([0; 4]),
                                        )
                                            as usize;
                                        let dst_start = dst_len_start + 4;
                                        if fwd_key_bytes.len() >= dst_start + dst_len {
                                            let stored_src =
                                                &fwd_key_bytes[src_start..src_start + src_len];
                                            let stored_dst =
                                                &fwd_key_bytes[dst_start..dst_start + dst_len];
                                            if stored_src == src_bytes && stored_dst == dst_bytes {
                                                forward_keys_to_delete
                                                    .push(fwd_compressed_key.clone());
                                            }
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
            let mut reverse_index = self.base.reverse_index().write();
            for key in &reverse_keys_to_delete {
                if let Some(entry) = reverse_index.get_mut(key) {
                    entry.mark_deleted(write_ts);
                }
            }
        }

        {
            let mut forward_index = self.base.forward_index().write();
            for key in &forward_keys_to_delete {
                if let Some(entry) = forward_index.get_mut(key) {
                    entry.mark_deleted(write_ts);
                }
            }
        }

        Ok(())
    }

    pub fn lookup_edge_index(
        &self,
        space_id: u64,
        index: &Index,
        value: &Value,
    ) -> Result<Vec<Value>, StorageError> {
        self.lookup_edge_index_mvcc(space_id, index, value, MAX_TIMESTAMP)
    }

    pub fn lookup_edge_index_mvcc(
        &self,
        space_id: u64,
        index: &Index,
        value: &Value,
        read_ts: Timestamp,
    ) -> Result<Vec<Value>, StorageError> {
        let prefix = KeyBuilder::build_edge_index_prefix(space_id, &index.name);
        let end = KeyBuilder::build_range_end(&prefix);

        let mut results = Vec::new();
        let value_bytes = serialize_value(value)?;

        let forward_index = self.base.forward_index().read();
        for (compressed_key, entry) in forward_index.range(prefix.0.clone()..end.0) {
            if !entry.is_visible_at(read_ts) {
                continue;
            }

            let key_bytes = self.decompress_key(compressed_key)?;
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
                        let src_len_start = prop_value_start + prop_value_len;
                        if key_bytes.len() >= src_len_start + 4 {
                            let src_len = u32::from_le_bytes(
                                key_bytes[src_len_start..src_len_start + 4]
                                    .try_into()
                                    .unwrap_or([0; 4]),
                            ) as usize;
                            let src_start = src_len_start + 4;
                            if key_bytes.len() >= src_start + src_len {
                                let src_bytes = &key_bytes[src_start..src_start + src_len];
                                if let Ok(src) = deserialize_value(src_bytes) {
                                    results.push(src);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    pub fn clear_edge_index(&self, space_id: u64, index_name: &str) -> Result<(), StorageError> {
        let prefix = KeyBuilder::build_edge_index_prefix(space_id, index_name);
        let end = KeyBuilder::build_range_end(&prefix);

        let mut keys_to_delete: Vec<SecondaryIndexKey> = Vec::new();

        {
            let forward_index = self.base.forward_index().read();
            for (key_bytes, _) in forward_index.range(prefix.0.clone()..end.0) {
                keys_to_delete.push(key_bytes.clone());
            }
        }

        {
            let mut forward_index = self.base.forward_index().write();
            for key in &keys_to_delete {
                forward_index.remove(key);
            }
        }

        Ok(())
    }

    pub fn clear_all(&self) -> Result<(), StorageError> {
        self.base.clear_all()
    }

    pub fn lookup_edge_index_range(
        &self,
        space_id: u64,
        index: &Index,
        start_value: &Value,
        end_value: &Value,
    ) -> Result<Vec<Value>, StorageError> {
        self.lookup_edge_index_range_mvcc(space_id, index, start_value, end_value, MAX_TIMESTAMP)
    }

    pub fn lookup_edge_index_range_mvcc(
        &self,
        space_id: u64,
        index: &Index,
        start_value: &Value,
        end_value: &Value,
        read_ts: Timestamp,
    ) -> Result<Vec<Value>, StorageError> {
        let prefix = KeyBuilder::build_edge_index_prefix(space_id, &index.name);
        let start_bytes = serialize_value(start_value)?;
        let end_bytes = serialize_value(end_value)?;

        let range_start = KeyBuilder::build_edge_index_key(
            space_id,
            &index.name,
            start_value,
            &Value::Int(i32::MIN),
            &Value::Int(i32::MIN),
        )?;
        let range_end = KeyBuilder::build_edge_index_key(
            space_id,
            &index.name,
            end_value,
            &Value::Int(i32::MAX),
            &Value::Int(i32::MAX),
        )?;

        let mut results = Vec::new();
        let forward_index = self.base.forward_index().read();

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
                        let src_len_start = prop_value_start + prop_value_len;
                        if key_bytes.len() >= src_len_start + 4 {
                            let src_len = u32::from_le_bytes(
                                key_bytes[src_len_start..src_len_start + 4]
                                    .try_into()
                                    .unwrap_or([0; 4]),
                            ) as usize;
                            let src_start = src_len_start + 4;
                            if key_bytes.len() >= src_start + src_len {
                                let src_bytes = &key_bytes[src_start..src_start + src_len];
                                if let Ok(src) = deserialize_value(src_bytes) {
                                    results.push(src);
                                }
                            }
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
        let prefix = KeyBuilder::build_edge_index_prefix(space_id, index_name);
        let end = KeyBuilder::build_range_end(&prefix);

        let forward_index = self.base.forward_index().read();
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
    ) -> Result<Vec<(Value, Value, Value)>, StorageError> {
        self.scan_index_entries_mvcc(space_id, index, limit, MAX_TIMESTAMP)
    }

    pub fn scan_index_entries_mvcc(
        &self,
        space_id: u64,
        index: &Index,
        limit: usize,
        read_ts: Timestamp,
    ) -> Result<Vec<(Value, Value, Value)>, StorageError> {
        let prefix = KeyBuilder::build_edge_index_prefix(space_id, &index.name);
        let end = KeyBuilder::build_range_end(&prefix);

        let mut results = Vec::with_capacity(limit.min(1000));
        let forward_index = self.base.forward_index().read();

        for (compressed_key, entry) in forward_index.range(prefix.0.clone()..end.0) {
            if results.len() >= limit {
                break;
            }
            if !entry.is_visible_at(read_ts) {
                continue;
            }

            let key_bytes = self.decompress_key(compressed_key)?;
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

                    if let Ok(prop_value) = deserialize_value(stored_prop_value) {
                        let src_len_start = prop_value_start + prop_value_len;
                        if key_bytes.len() >= src_len_start + 8 {
                            let src_len = u32::from_le_bytes(
                                key_bytes[src_len_start..src_len_start + 4]
                                    .try_into()
                                    .unwrap_or([0; 4]),
                            ) as usize;
                            let src_start = src_len_start + 4;

                            if key_bytes.len() >= src_start + src_len + 4 {
                                let src_bytes = &key_bytes[src_start..src_start + src_len];
                                let dst_len_start = src_start + src_len;
                                let dst_len = u32::from_le_bytes(
                                    key_bytes[dst_len_start..dst_len_start + 4]
                                        .try_into()
                                        .unwrap_or([0; 4]),
                                ) as usize;
                                let dst_start = dst_len_start + 4;

                                if key_bytes.len() >= dst_start + dst_len {
                                    let dst_bytes = &key_bytes[dst_start..dst_start + dst_len];
                                    if let (Ok(src), Ok(dst)) =
                                        (deserialize_value(src_bytes), deserialize_value(dst_bytes))
                                    {
                                        results.push((prop_value, src, dst));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    pub fn flush<P: AsRef<Path>>(&self, path: P) -> StorageResult<()> {
        self.base.flush(path)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> StorageResult<()> {
        self.base.save(path)
    }

    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> StorageResult<()> {
        self.base.load(path)
    }

    pub fn entry_count(&self) -> (usize, usize) {
        self.base.entry_count()
    }

    pub fn gc_tombstones(&self, safe_ts: Timestamp) -> Result<usize, StorageError> {
        self.base.gc_tombstones(safe_ts)
    }

    pub fn gc_tombstones_incremental(
        &self,
        safe_ts: Timestamp,
        batch_size: usize,
    ) -> Result<usize, StorageError> {
        self.base.gc_tombstones_incremental(safe_ts, batch_size)
    }

    pub fn tombstone_count(&self) -> usize {
        self.base.tombstone_count()
    }

    pub fn update_edge_indexes_native(
        &self,
        space_id: u64,
        src: u64,
        dst: u64,
        index_name: &str,
        props: &[(String, Value)],
    ) -> Result<(), StorageError> {
        self.update_edge_indexes_native_mvcc(space_id, src, dst, index_name, props, MAX_TIMESTAMP)
    }

    pub fn update_edge_indexes_native_mvcc(
        &self,
        space_id: u64,
        src: u64,
        dst: u64,
        index_name: &str,
        props: &[(String, Value)],
        write_ts: Timestamp,
    ) -> Result<(), StorageError> {
        let mut forward_entries: Vec<(SecondaryIndexKey, IndexEntry)> =
            Vec::with_capacity(props.len());
        let mut reverse_entries: Vec<(SecondaryIndexKey, IndexEntry)> =
            Vec::with_capacity(props.len());

        for (_prop_name, prop_value) in props {
            let index_key = KeyBuilder::build_edge_index_key_native(
                space_id, index_name, prop_value, src, dst,
            )?;

            let reverse_key =
                KeyBuilder::build_edge_reverse_key_native(space_id, src, dst, index_name);

            let entry = IndexEntry::new(write_ts);
            forward_entries.push((index_key.0, entry.clone()));
            reverse_entries.push((reverse_key.0, entry));
        }

        {
            let mut forward_index = self.base.forward_index().write();
            for (key, entry) in forward_entries {
                forward_index.insert(key, entry);
            }
        }
        {
            let mut reverse_index = self.base.reverse_index().write();
            for (key, entry) in reverse_entries {
                reverse_index.insert(key, entry);
            }
        }

        Ok(())
    }

    pub fn delete_edge_indexes_native(
        &self,
        space_id: u64,
        src: u64,
        dst: u64,
        index_names: &[String],
    ) -> Result<(), StorageError> {
        self.delete_edge_indexes_native_mvcc(space_id, src, dst, index_names, MAX_TIMESTAMP)
    }

    pub fn delete_edge_indexes_native_mvcc(
        &self,
        space_id: u64,
        src: u64,
        dst: u64,
        index_names: &[String],
        write_ts: Timestamp,
    ) -> Result<(), StorageError> {
        let reverse_prefix =
            KeyBuilder::build_edge_reverse_prefix_native_with_dst(space_id, src, dst);
        let reverse_end = KeyBuilder::build_range_end(&reverse_prefix);

        let mut forward_keys_to_delete: Vec<SecondaryIndexKey> = Vec::new();
        let mut reverse_keys_to_delete: Vec<SecondaryIndexKey> = Vec::new();

        {
            let reverse_index = self.base.reverse_index().read();
            for (key_bytes, entry) in reverse_index.range(reverse_prefix.0.clone()..reverse_end.0) {
                if !entry.is_visible_at(write_ts) {
                    continue;
                }

                if let Ok((_src, _dst, index_name)) =
                    KeyParser::parse_edge_reverse_key_native(key_bytes)
                {
                    if index_names.contains(&index_name) {
                        reverse_keys_to_delete.push(key_bytes.clone());

                        let forward_key_start =
                            KeyBuilder::build_edge_index_prefix(space_id, &index_name);
                        let forward_key_end = KeyBuilder::build_range_end(&forward_key_start);

                        let forward_index = self.base.forward_index().read();
                        for (fwd_key_bytes, fwd_entry) in
                            forward_index.range(forward_key_start.0.clone()..forward_key_end.0)
                        {
                            if !fwd_entry.is_visible_at(write_ts) {
                                continue;
                            }

                            if let Ok((stored_src, stored_dst)) =
                                KeyParser::parse_edge_ids_from_key_native(fwd_key_bytes)
                            {
                                if stored_src == src && stored_dst == dst {
                                    forward_keys_to_delete.push(fwd_key_bytes.clone());
                                }
                            }
                        }
                    }
                }
            }
        }

        {
            let mut reverse_index = self.base.reverse_index().write();
            for key in &reverse_keys_to_delete {
                if let Some(entry) = reverse_index.get_mut(key) {
                    entry.mark_deleted(write_ts);
                }
            }
        }

        {
            let mut forward_index = self.base.forward_index().write();
            for key in &forward_keys_to_delete {
                if let Some(entry) = forward_index.get_mut(key) {
                    entry.mark_deleted(write_ts);
                }
            }
        }

        Ok(())
    }

    pub fn lookup_edge_index_native(
        &self,
        space_id: u64,
        index: &Index,
        value: &Value,
    ) -> Result<Vec<(u64, u64)>, StorageError> {
        self.lookup_edge_index_native_mvcc(space_id, index, value, MAX_TIMESTAMP)
    }

    pub fn lookup_edge_index_native_mvcc(
        &self,
        space_id: u64,
        index: &Index,
        value: &Value,
        read_ts: Timestamp,
    ) -> Result<Vec<(u64, u64)>, StorageError> {
        let prefix = KeyBuilder::build_edge_index_prefix(space_id, &index.name);
        let end = KeyBuilder::build_range_end(&prefix);

        let mut results = Vec::new();
        let value_bytes = serialize_value(value)?;

        let forward_index = self.base.forward_index().read();
        for (key_bytes, entry) in forward_index.range(prefix.0.clone()..end.0) {
            if !entry.is_visible_at(read_ts) {
                continue;
            }

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
                        if let Ok((src, dst)) = KeyParser::parse_edge_ids_from_key_native(key_bytes)
                        {
                            results.push((src, dst));
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    pub fn lookup_edge_index_range_native(
        &self,
        space_id: u64,
        index: &Index,
        start_value: &Value,
        end_value: &Value,
    ) -> Result<Vec<(u64, u64)>, StorageError> {
        self.lookup_edge_index_range_native_mvcc(
            space_id,
            index,
            start_value,
            end_value,
            MAX_TIMESTAMP,
        )
    }

    pub fn lookup_edge_index_range_native_mvcc(
        &self,
        space_id: u64,
        index: &Index,
        start_value: &Value,
        end_value: &Value,
        read_ts: Timestamp,
    ) -> Result<Vec<(u64, u64)>, StorageError> {
        let start_bytes = serialize_value(start_value)?;
        let end_bytes = serialize_value(end_value)?;

        let range_start =
            KeyBuilder::build_edge_index_key_native(space_id, &index.name, start_value, 0, 0)?;
        let range_end = KeyBuilder::build_edge_index_key_native(
            space_id,
            &index.name,
            end_value,
            u64::MAX,
            u64::MAX,
        )?;

        let mut results = Vec::new();
        let forward_index = self.base.forward_index().read();

        for (key_bytes, entry) in forward_index.range(range_start.0.clone()..range_end.0.clone()) {
            if !entry.is_visible_at(read_ts) {
                continue;
            }

            let prefix = KeyBuilder::build_edge_index_prefix(space_id, &index.name);
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
                        if let Ok((src, dst)) = KeyParser::parse_edge_ids_from_key_native(key_bytes)
                        {
                            results.push((src, dst));
                        }
                    }
                }
            }
        }

        Ok(results)
    }
}

impl Default for EdgeIndexManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::EdgeIndexManager;
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
            index_type: IndexType::EdgeIndex,
            is_unique: false,
            partial_condition: None,
        })
    }

    #[test]
    fn test_update_and_lookup_edge_index() {
        let manager = EdgeIndexManager::new();

        let space_id = 1u64;
        let src = Value::Int(1);
        let dst = Value::Int(2);
        let index_name = "idx_edge_weight";
        let props = vec![("weight".to_string(), Value::Float(10.5))];

        manager
            .update_edge_indexes(space_id, &src, &dst, index_name, &props)
            .expect("Failed to update edge indexes");

        let index = create_test_index(index_name, "knows");

        let results = manager
            .lookup_edge_index(space_id, &index, &Value::Float(10.5))
            .expect("Failed to lookup edge index");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], src);

        let empty_results = manager
            .lookup_edge_index(space_id, &index, &Value::Float(99.9))
            .expect("Failed to lookup edge index");
        assert!(empty_results.is_empty());
    }

    #[test]
    fn test_delete_edge_indexes() {
        let manager = EdgeIndexManager::new();

        let space_id = 1u64;
        let src1 = Value::Int(1);
        let dst1 = Value::Int(2);
        let src2 = Value::Int(3);
        let dst2 = Value::Int(4);
        let index_name = "idx_edge_weight";

        let props1 = vec![("weight".to_string(), Value::Float(10.5))];
        let props2 = vec![("weight".to_string(), Value::Float(20.5))];

        manager
            .update_edge_indexes(space_id, &src1, &dst1, index_name, &props1)
            .expect("Failed to update edge indexes");
        manager
            .update_edge_indexes(space_id, &src2, &dst2, index_name, &props2)
            .expect("Failed to update edge indexes");

        let index = create_test_index(index_name, "knows");

        let results1 = manager
            .lookup_edge_index(space_id, &index, &Value::Float(10.5))
            .expect("Failed to lookup edge index");
        assert_eq!(results1.len(), 1);

        let results2 = manager
            .lookup_edge_index(space_id, &index, &Value::Float(20.5))
            .expect("Failed to lookup edge index");
        assert_eq!(results2.len(), 1);

        manager
            .delete_edge_indexes(space_id, &src1, &dst1, &[index_name.to_string()])
            .expect("Failed to delete edge indexes");

        let results1_after = manager
            .lookup_edge_index(space_id, &index, &Value::Float(10.5))
            .expect("Failed to lookup edge index");
        assert!(results1_after.is_empty());

        let results2_after = manager
            .lookup_edge_index(space_id, &index, &Value::Float(20.5))
            .expect("Failed to lookup edge index");
        assert_eq!(results2_after.len(), 1);
    }

    #[test]
    fn test_clear_edge_index() {
        let manager = EdgeIndexManager::new();

        let space_id = 1u64;
        let src = Value::Int(1);
        let dst = Value::Int(2);
        let index_name = "idx_edge_weight";

        let props = vec![("weight".to_string(), Value::Float(10.5))];
        manager
            .update_edge_indexes(space_id, &src, &dst, index_name, &props)
            .expect("Failed to update edge indexes");

        let index = create_test_index(index_name, "knows");
        let results = manager
            .lookup_edge_index(space_id, &index, &Value::Float(10.5))
            .expect("Failed to lookup edge index");
        assert_eq!(results.len(), 1);

        manager
            .clear_edge_index(space_id, index_name)
            .expect("Failed to clear edge index");

        let results_after = manager
            .lookup_edge_index(space_id, &index, &Value::Float(10.5))
            .expect("Failed to lookup edge index");
        assert!(results_after.is_empty());
    }
}
