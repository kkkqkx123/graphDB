//! Vertex Index Management Module
//!
//! Provide functions for updating, deleting, and querying vertex indices.
//! This implementation uses in-memory storage with BTreeMap for efficient range queries.
//! Supports persistence through flush/load operations.
//! Supports MVCC (Multi-Version Concurrency Control) for snapshot isolation.

use crate::core::types::{Index, Timestamp, MAX_TIMESTAMP};
use crate::core::{StorageError, StorageResult, Value};
use crate::storage::index::generic_index_manager::GenericIndexManager;
use crate::storage::index::index_data_manager::IndexEntry;
use crate::storage::index::key_codec::{
    serialize_value, KeyBuilder, KeyParser, SecondaryIndexKey, VertexIndexKeyGen,
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

    fn compress_key(&self, key: &[u8]) -> Vec<u8> {
        key.to_vec()
    }

    fn decompress_key(&self, compressed: &[u8]) -> StorageResult<Vec<u8>> {
        Ok(compressed.to_vec())
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

    pub fn flush<P: AsRef<Path>>(&self, path: P) -> StorageResult<()> {
        self.base.flush(path)
    }

    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> StorageResult<()> {
        self.base.load(path)
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
}
