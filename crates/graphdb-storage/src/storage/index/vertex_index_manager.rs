//! Vertex Index Management Module
//!
//! Provide functions for updating, deleting, and querying vertex indices.
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
    deserialize_value, serialize_value, CompressionConfig, KeyBuilder, KeyParser,
    SecondaryIndexKey, VertexIndexKeyGen,
};
use std::path::Path;

#[derive(Clone)]
pub struct VertexIndexManager {
    base: GenericIndexManager<VertexIndexKeyGen>,
}

impl VertexIndexManager {
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

    pub fn train_compression(&self, keys: &[Vec<u8>]) -> StorageResult<()> {
        self.base.train_compression(keys)
    }

    pub fn compression_ratio(&self) -> Option<f64> {
        self.base.compression_ratio()
    }

    fn compress_key(&self, key: &[u8]) -> Vec<u8> {
        self.base.compress_key_public(key)
    }

    fn decompress_key(&self, compressed: &[u8]) -> StorageResult<Vec<u8>> {
        self.base.decompress_key_public(compressed)
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
        let mut forward_entries: Vec<(SecondaryIndexKey, IndexEntry)> =
            Vec::with_capacity(props.len());
        let mut reverse_entries: Vec<(SecondaryIndexKey, IndexEntry)> =
            Vec::with_capacity(props.len());

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

    pub fn delete_vertex_indexes(
        &self,
        space_id: u64,
        vertex_id: &Value,
    ) -> Result<(), StorageError> {
        self.delete_vertex_indexes_mvcc(space_id, vertex_id, MAX_TIMESTAMP)
    }

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
        let reverse_key = KeyBuilder::build_vertex_reverse_key_v2(space_id, vertex_id, index_name)?;

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

    pub fn delete_vertex_indexes_mvcc(
        &self,
        space_id: u64,
        vertex_id: &Value,
        write_ts: Timestamp,
    ) -> Result<(), StorageError> {
        let reverse_prefix = KeyBuilder::build_vertex_reverse_prefix_v2(space_id, vertex_id)?;
        let reverse_end = KeyBuilder::build_range_end(&reverse_prefix);

        let mut forward_keys_to_delete: Vec<SecondaryIndexKey> = Vec::new();
        let mut reverse_keys_to_delete: Vec<SecondaryIndexKey> = Vec::new();

        {
            let reverse_index = self.base.reverse_index().read();
            for (compressed_key, entry) in
                reverse_index.range(reverse_prefix.0.clone()..reverse_end.0)
            {
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
                        let forward_index = self.base.forward_index().read();
                        for (fwd_compressed_key, fwd_entry) in
                            forward_index.range(forward_key_start.0.clone()..forward_key_end.0)
                        {
                            if fwd_entry.is_visible_at(write_ts) {
                                let fwd_key_bytes = self.decompress_key(fwd_compressed_key)?;
                                if let Ok(vid) = KeyParser::parse_vertex_id_from_key(&fwd_key_bytes)
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

    pub fn delete_tag_indexes(
        &self,
        space_id: u64,
        vertex_id: &Value,
        tag_name: &str,
    ) -> Result<(), StorageError> {
        let reverse_prefix = KeyBuilder::build_vertex_reverse_prefix_v2(space_id, vertex_id)?;
        let reverse_end = KeyBuilder::build_range_end(&reverse_prefix);

        let mut keys_to_delete: Vec<SecondaryIndexKey> = Vec::new();

        {
            let reverse_index = self.base.reverse_index().read();
            for (compressed_key, _) in reverse_index.range(reverse_prefix.0.clone()..reverse_end.0)
            {
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
            let mut reverse_index = self.base.reverse_index().write();
            for key in &keys_to_delete {
                reverse_index.remove(key);
            }
        }

        Ok(())
    }

    pub fn clear_tag_index(&self, space_id: u64, index_name: &str) -> Result<(), StorageError> {
        let prefix = KeyBuilder::build_vertex_index_prefix(space_id, index_name);
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

        let forward_index = self.base.forward_index().read();
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
        let forward_index = self.base.forward_index().read();

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
                        if let Ok(prop_value) = deserialize_value(stored_prop_value) {
                            results.push((prop_value, vertex_id));
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

        let reverse_key = KeyBuilder::build_vertex_reverse_key_v2(space_id, vertex_id, index_name)?;

        let entry = IndexEntry::new(write_ts);

        {
            let mut forward_index = self.base.forward_index().write();
            forward_index.insert(index_key.0, entry.clone());
        }
        {
            let mut reverse_index = self.base.reverse_index().write();
            reverse_index.insert(reverse_key.0, entry);
        }

        Ok(())
    }

    pub fn lookup_composite_tag_index(
        &self,
        space_id: u64,
        index_name: &str,
        field_values: &[Value],
    ) -> Result<Vec<Value>, StorageError> {
        self.lookup_composite_tag_index_mvcc(space_id, index_name, field_values, MAX_TIMESTAMP)
    }

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

        let forward_index = self.base.forward_index().read();
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

        let forward_index = self.base.forward_index().read();
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

    pub fn update_vertex_indexes_native_mvcc(
        &self,
        space_id: u64,
        vertex_id: u64,
        index_name: &str,
        props: &[(String, Value)],
        write_ts: Timestamp,
    ) -> Result<(), StorageError> {
        let mut forward_entries: Vec<(SecondaryIndexKey, IndexEntry)> =
            Vec::with_capacity(props.len());
        let mut reverse_entries: Vec<(SecondaryIndexKey, IndexEntry)> =
            Vec::with_capacity(props.len());

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

    pub fn delete_vertex_indexes_native(
        &self,
        space_id: u64,
        vertex_id: u64,
    ) -> Result<(), StorageError> {
        self.delete_vertex_indexes_native_mvcc(space_id, vertex_id, MAX_TIMESTAMP)
    }

    pub fn delete_vertex_indexes_native_mvcc(
        &self,
        space_id: u64,
        vertex_id: u64,
        write_ts: Timestamp,
    ) -> Result<(), StorageError> {
        let reverse_prefix = KeyBuilder::build_vertex_reverse_prefix_native(space_id, vertex_id);
        let reverse_end = KeyBuilder::build_range_end(&reverse_prefix);

        let mut forward_keys_to_delete: Vec<SecondaryIndexKey> = Vec::new();
        let mut reverse_keys_to_delete: Vec<SecondaryIndexKey> = Vec::new();

        {
            let reverse_index = self.base.reverse_index().read();
            for (key_bytes, entry) in reverse_index.range(reverse_prefix.0.clone()..reverse_end.0) {
                if entry.is_visible_at(write_ts) {
                    reverse_keys_to_delete.push(key_bytes.clone());

                    if let Ok((_vertex_id, index_name)) =
                        KeyParser::parse_vertex_reverse_key_native(key_bytes)
                    {
                        let forward_key_start =
                            KeyBuilder::build_vertex_index_prefix(space_id, &index_name);
                        let forward_key_end = KeyBuilder::build_range_end(&forward_key_start);

                        let forward_index = self.base.forward_index().read();
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

    pub fn lookup_tag_index_native(
        &self,
        space_id: u64,
        index: &Index,
        value: &Value,
    ) -> Result<Vec<u64>, StorageError> {
        self.lookup_tag_index_native_mvcc(space_id, index, value, MAX_TIMESTAMP)
    }

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
                        if let Ok(vid) = KeyParser::parse_vertex_id_from_key_native(key_bytes) {
                            results.push(vid);
                        }
                    }
                }
            }
        }

        Ok(results)
    }

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
        let range_end =
            KeyBuilder::build_vertex_index_key_native(space_id, &index.name, end_value, u64::MAX)?;

        let mut results = Vec::new();
        let forward_index = self.base.forward_index().read();

        for (key_bytes, entry) in forward_index.range(range_start.0.clone()..range_end.0.clone()) {
            if !entry.is_visible_at(read_ts) {
                continue;
            }

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
                        if let Ok(vid) = KeyParser::parse_vertex_id_from_key_native(key_bytes) {
                            results.push(vid);
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
    use crate::core::types::{Index, IndexConfig, IndexField, IndexType};
    use crate::core::Value;
    use crate::storage::index::key_codec::CompressionConfig;

    use super::VertexIndexManager;

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
        let vertex_id = Value::Int(123);
        let index_name = "idx_name";
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
        let vertex_id = Value::Int(123);
        let index_name = "idx_name";
        let props = vec![("name".to_string(), Value::String("Alice".to_string()))];

        manager
            .update_vertex_indexes(space_id, &vertex_id, index_name, &props)
            .expect("Failed to update vertex indexes");

        let index = create_test_index(index_name, "person");

        let results = manager
            .lookup_tag_index(space_id, &index, &Value::String("Alice".to_string()))
            .expect("Failed to lookup tag index");
        assert_eq!(results.len(), 1);

        manager
            .delete_vertex_indexes(space_id, &vertex_id)
            .expect("Failed to delete vertex indexes");

        let results_after = manager
            .lookup_tag_index(space_id, &index, &Value::String("Alice".to_string()))
            .expect("Failed to lookup tag index");
        assert!(results_after.is_empty());
    }

    #[test]
    fn test_clear_tag_index() {
        let manager = VertexIndexManager::new();

        let space_id = 1u64;
        let vertex_id = Value::Int(123);
        let index_name = "idx_name";
        let props = vec![("name".to_string(), Value::String("Alice".to_string()))];

        manager
            .update_vertex_indexes(space_id, &vertex_id, index_name, &props)
            .expect("Failed to update vertex indexes");

        let index = create_test_index(index_name, "person");
        let results = manager
            .lookup_tag_index(space_id, &index, &Value::String("Alice".to_string()))
            .expect("Failed to lookup tag index");
        assert_eq!(results.len(), 1);

        manager
            .clear_tag_index(space_id, index_name)
            .expect("Failed to clear tag index");

        let results_after = manager
            .lookup_tag_index(space_id, &index, &Value::String("Alice".to_string()))
            .expect("Failed to lookup tag index");
        assert!(results_after.is_empty());
    }

    #[test]
    fn test_compression_enabled() {
        let config = CompressionConfig::default();
        let manager = VertexIndexManager::with_compression(config);

        assert!(manager.is_compression_enabled());
    }
}
