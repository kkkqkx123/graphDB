//! Index Data Manager
//!
//! Provide update, delete and query functions for indexed data
//! The management of index metadata is handled by the IndexMetadataManager.
//! All operations identify a space by its space_id, enabling multi-space data segregation.
//! Supports persistence through flush/load operations.
//! Supports MVCC (Multi-Version Concurrency Control) for snapshot isolation.

use crate::core::types::Index;
use crate::core::vertex_edge_path::Tag;
use crate::core::Edge;
use crate::core::{StorageError, StorageResult, Value};
use super::edge_index_manager::EdgeIndexManager;
use super::key_codec::{deserialize_value, serialize_value};
use super::vertex_index_manager::VertexIndexManager;
use std::path::Path;

pub type Timestamp = u32;

pub const INVALID_TIMESTAMP: Timestamp = u32::MAX;
pub const MAX_TIMESTAMP: Timestamp = u32::MAX - 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexEntry {
    pub created_ts: Timestamp,
    pub deleted_ts: Option<Timestamp>,
}

impl IndexEntry {
    pub fn new(created_ts: Timestamp) -> Self {
        Self {
            created_ts,
            deleted_ts: None,
        }
    }

    pub fn with_deleted(mut self, deleted_ts: Timestamp) -> Self {
        self.deleted_ts = Some(deleted_ts);
        self
    }

    pub fn is_visible_at(&self, read_ts: Timestamp) -> bool {
        self.created_ts <= read_ts
            && self
                .deleted_ts
                .is_none_or(|deleted_ts| deleted_ts > read_ts)
    }

    pub fn is_deleted(&self) -> bool {
        self.deleted_ts.is_some()
    }

    pub fn mark_deleted(&mut self, deleted_ts: Timestamp) {
        self.deleted_ts = Some(deleted_ts);
    }
}

impl Default for IndexEntry {
    fn default() -> Self {
        Self::new(MAX_TIMESTAMP)
    }
}

pub trait IndexDataManager {
    fn update_vertex_indexes(
        &self,
        space_id: u64,
        vertex_id: &Value,
        index_name: &str,
        props: &[(String, Value)],
    ) -> Result<(), StorageError> {
        self.update_vertex_indexes_mvcc(space_id, vertex_id, index_name, props, MAX_TIMESTAMP)
    }

    fn update_vertex_indexes_mvcc(
        &self,
        space_id: u64,
        vertex_id: &Value,
        index_name: &str,
        props: &[(String, Value)],
        write_ts: Timestamp,
    ) -> Result<(), StorageError>;

    fn update_edge_indexes(
        &self,
        space_id: u64,
        src: &Value,
        dst: &Value,
        index_name: &str,
        props: &[(String, Value)],
    ) -> Result<(), StorageError> {
        self.update_edge_indexes_mvcc(space_id, src, dst, index_name, props, MAX_TIMESTAMP)
    }

    fn update_edge_indexes_mvcc(
        &self,
        space_id: u64,
        src: &Value,
        dst: &Value,
        index_name: &str,
        props: &[(String, Value)],
        write_ts: Timestamp,
    ) -> Result<(), StorageError>;

    fn delete_vertex_indexes(&self, space_id: u64, vertex_id: &Value) -> Result<(), StorageError> {
        self.delete_vertex_indexes_mvcc(space_id, vertex_id, MAX_TIMESTAMP)
    }

    fn delete_vertex_indexes_mvcc(
        &self,
        space_id: u64,
        vertex_id: &Value,
        write_ts: Timestamp,
    ) -> Result<(), StorageError>;

    /// Delete a single specific vertex index entry
    fn delete_vertex_index_single(
        &self,
        space_id: u64,
        vertex_id: &Value,
        index_name: &str,
        prop_value: &Value,
        write_ts: Timestamp,
    ) -> Result<(), StorageError> {
        self.delete_vertex_index_single_mvcc(space_id, vertex_id, index_name, prop_value, write_ts)
    }

    fn delete_vertex_index_single_mvcc(
        &self,
        space_id: u64,
        vertex_id: &Value,
        index_name: &str,
        prop_value: &Value,
        write_ts: Timestamp,
    ) -> Result<(), StorageError>;

    fn delete_edge_indexes(
        &self,
        space_id: u64,
        src: &Value,
        dst: &Value,
        index_names: &[String],
    ) -> Result<(), StorageError> {
        self.delete_edge_indexes_mvcc(space_id, src, dst, index_names, MAX_TIMESTAMP)
    }

    fn delete_edge_indexes_mvcc(
        &self,
        space_id: u64,
        src: &Value,
        dst: &Value,
        index_names: &[String],
        write_ts: Timestamp,
    ) -> Result<(), StorageError>;

    /// Delete a single specific edge index entry
    fn delete_edge_index_single(
        &self,
        space_id: u64,
        src: &Value,
        dst: &Value,
        index_name: &str,
        prop_value: &Value,
        write_ts: Timestamp,
    ) -> Result<(), StorageError> {
        self.delete_edge_index_single_mvcc(space_id, src, dst, index_name, prop_value, write_ts)
    }

    fn delete_edge_index_single_mvcc(
        &self,
        space_id: u64,
        src: &Value,
        dst: &Value,
        index_name: &str,
        prop_value: &Value,
        write_ts: Timestamp,
    ) -> Result<(), StorageError>;

    fn lookup_tag_index(
        &self,
        space_id: u64,
        index: &Index,
        value: &Value,
    ) -> Result<Vec<Value>, StorageError> {
        self.lookup_tag_index_mvcc(space_id, index, value, MAX_TIMESTAMP)
    }

    fn lookup_tag_index_mvcc(
        &self,
        space_id: u64,
        index: &Index,
        value: &Value,
        read_ts: Timestamp,
    ) -> Result<Vec<Value>, StorageError>;

    fn lookup_edge_index(
        &self,
        space_id: u64,
        index: &Index,
        value: &Value,
    ) -> Result<Vec<Value>, StorageError> {
        self.lookup_edge_index_mvcc(space_id, index, value, MAX_TIMESTAMP)
    }

    fn lookup_edge_index_mvcc(
        &self,
        space_id: u64,
        index: &Index,
        value: &Value,
        read_ts: Timestamp,
    ) -> Result<Vec<Value>, StorageError>;

    fn clear_edge_index(&self, space_id: u64, index_name: &str) -> Result<(), StorageError>;
    fn build_edge_index_entry(
        &self,
        space_id: u64,
        index: &Index,
        edge: &Edge,
    ) -> Result<(), StorageError>;
    fn delete_tag_indexes(
        &self,
        space_id: u64,
        vertex_id: &Value,
        tag_name: &str,
    ) -> Result<(), StorageError>;
    fn clear_tag_index(&self, space_id: u64, index_name: &str) -> Result<(), StorageError>;
    fn build_vertex_index_entry(
        &self,
        space_id: u64,
        index: &Index,
        vertex_id: &Value,
        tag: &Tag,
    ) -> Result<(), StorageError>;

    // ========================================================================
    // Native ID Type Support (CSR-compatible)
    // ========================================================================

    fn update_vertex_indexes_native(
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

    fn update_vertex_indexes_native_mvcc(
        &self,
        space_id: u64,
        vertex_id: u64,
        index_name: &str,
        props: &[(String, Value)],
        write_ts: Timestamp,
    ) -> Result<(), StorageError>;

    fn delete_vertex_indexes_native(
        &self,
        space_id: u64,
        vertex_id: u64,
    ) -> Result<(), StorageError> {
        self.delete_vertex_indexes_native_mvcc(space_id, vertex_id, MAX_TIMESTAMP)
    }

    fn delete_vertex_indexes_native_mvcc(
        &self,
        space_id: u64,
        vertex_id: u64,
        write_ts: Timestamp,
    ) -> Result<(), StorageError>;

    fn lookup_tag_index_native(
        &self,
        space_id: u64,
        index: &Index,
        value: &Value,
    ) -> Result<Vec<u64>, StorageError> {
        self.lookup_tag_index_native_mvcc(space_id, index, value, MAX_TIMESTAMP)
    }

    fn lookup_tag_index_native_mvcc(
        &self,
        space_id: u64,
        index: &Index,
        value: &Value,
        read_ts: Timestamp,
    ) -> Result<Vec<u64>, StorageError>;

    fn update_edge_indexes_native(
        &self,
        space_id: u64,
        src: u64,
        dst: u64,
        index_name: &str,
        props: &[(String, Value)],
    ) -> Result<(), StorageError> {
        self.update_edge_indexes_native_mvcc(space_id, src, dst, index_name, props, MAX_TIMESTAMP)
    }

    fn update_edge_indexes_native_mvcc(
        &self,
        space_id: u64,
        src: u64,
        dst: u64,
        index_name: &str,
        props: &[(String, Value)],
        write_ts: Timestamp,
    ) -> Result<(), StorageError>;

    fn delete_edge_indexes_native(
        &self,
        space_id: u64,
        src: u64,
        dst: u64,
        index_names: &[String],
    ) -> Result<(), StorageError> {
        self.delete_edge_indexes_native_mvcc(space_id, src, dst, index_names, MAX_TIMESTAMP)
    }

    fn delete_edge_indexes_native_mvcc(
        &self,
        space_id: u64,
        src: u64,
        dst: u64,
        index_names: &[String],
        write_ts: Timestamp,
    ) -> Result<(), StorageError>;

    fn lookup_edge_index_native(
        &self,
        space_id: u64,
        index: &Index,
        value: &Value,
    ) -> Result<Vec<(u64, u64)>, StorageError> {
        self.lookup_edge_index_native_mvcc(space_id, index, value, MAX_TIMESTAMP)
    }

    fn lookup_edge_index_native_mvcc(
        &self,
        space_id: u64,
        index: &Index,
        value: &Value,
        read_ts: Timestamp,
    ) -> Result<Vec<(u64, u64)>, StorageError>;
}

#[derive(Clone)]
pub struct InMemoryIndexDataManager {
    vertex_manager: VertexIndexManager,
    edge_manager: EdgeIndexManager,
}

impl InMemoryIndexDataManager {
    pub fn new() -> Self {
        Self {
            vertex_manager: VertexIndexManager::new(),
            edge_manager: EdgeIndexManager::new(),
        }
    }

    pub fn serialize_value(value: &Value) -> Result<Vec<u8>, StorageError> {
        serialize_value(value)
    }

    pub fn deserialize_value(data: &[u8]) -> Result<Value, StorageError> {
        deserialize_value(data)
    }

    pub fn flush<P: AsRef<Path>>(&self, path: P) -> StorageResult<()> {
        let path = path.as_ref();
        self.vertex_manager.flush(path.join("vertex_index"))?;
        self.edge_manager.flush(path.join("edge_index"))?;
        Ok(())
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> StorageResult<()> {
        let path = path.as_ref();
        std::fs::create_dir_all(path)?;
        self.vertex_manager.save(path.join("vertex_index"))?;
        self.edge_manager.save(path.join("edge_index"))?;
        Ok(())
    }

    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> StorageResult<()> {
        let path = path.as_ref();
        self.vertex_manager.load(path.join("vertex_index"))?;
        self.edge_manager.load(path.join("edge_index"))?;
        Ok(())
    }

    pub fn entry_count(&self) -> IndexEntryCount {
        let (forward, reverse) = self.vertex_manager.entry_count();
        let (edge_forward, edge_reverse) = self.edge_manager.entry_count();
        IndexEntryCount {
            vertex_forward: forward,
            vertex_reverse: reverse,
            edge_forward,
            edge_reverse,
        }
    }

    pub fn gc_tombstones(&self, safe_ts: Timestamp) -> Result<GcStats, StorageError> {
        let vertex_removed = self.vertex_manager.gc_tombstones(safe_ts)?;
        let edge_removed = self.edge_manager.gc_tombstones(safe_ts)?;

        Ok(GcStats {
            vertex_entries_removed: vertex_removed,
            edge_entries_removed: edge_removed,
        })
    }

    pub fn gc_tombstones_incremental(
        &self,
        safe_ts: Timestamp,
        batch_size: usize,
    ) -> Result<GcStats, StorageError> {
        let vertex_removed = self
            .vertex_manager
            .gc_tombstones_incremental(safe_ts, batch_size)?;
        let remaining = batch_size.saturating_sub(vertex_removed);
        let edge_removed = if remaining > 0 {
            self.edge_manager
                .gc_tombstones_incremental(safe_ts, remaining)?
        } else {
            0
        };

        Ok(GcStats {
            vertex_entries_removed: vertex_removed,
            edge_entries_removed: edge_removed,
        })
    }

    pub fn tombstone_count(&self) -> usize {
        self.vertex_manager.tombstone_count() + self.edge_manager.tombstone_count()
    }

    pub fn clear_all_indexes(&self) -> Result<(), StorageError> {
        self.vertex_manager.clear_all()?;
        self.edge_manager.clear_all()?;
        Ok(())
    }

    pub fn rebuild_stats(&self) -> RebuildStats {
        let vertex_estimate = self.vertex_manager.entry_count();
        let edge_estimate = self.edge_manager.entry_count();
        RebuildStats {
            vertex_entries: vertex_estimate.0 + vertex_estimate.1,
            edge_entries: edge_estimate.0 + edge_estimate.1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RebuildStats {
    pub vertex_entries: usize,
    pub edge_entries: usize,
}

impl RebuildStats {
    pub fn total_entries(&self) -> usize {
        self.vertex_entries + self.edge_entries
    }

    pub fn is_empty(&self) -> bool {
        self.vertex_entries == 0 && self.edge_entries == 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IndexEntryCount {
    pub vertex_forward: usize,
    pub vertex_reverse: usize,
    pub edge_forward: usize,
    pub edge_reverse: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GcStats {
    pub vertex_entries_removed: usize,
    pub edge_entries_removed: usize,
}

impl GcStats {
    pub fn total_removed(&self) -> usize {
        self.vertex_entries_removed + self.edge_entries_removed
    }

    pub fn is_empty(&self) -> bool {
        self.vertex_entries_removed == 0 && self.edge_entries_removed == 0
    }
}

impl Default for InMemoryIndexDataManager {
    fn default() -> Self {
        Self::new()
    }
}

impl IndexDataManager for InMemoryIndexDataManager {
    fn update_vertex_indexes_mvcc(
        &self,
        space_id: u64,
        vertex_id: &Value,
        index_name: &str,
        props: &[(String, Value)],
        write_ts: Timestamp,
    ) -> Result<(), StorageError> {
        self.vertex_manager
            .update_vertex_indexes_mvcc(space_id, vertex_id, index_name, props, write_ts)
    }

    fn update_edge_indexes_mvcc(
        &self,
        space_id: u64,
        src: &Value,
        dst: &Value,
        index_name: &str,
        props: &[(String, Value)],
        write_ts: Timestamp,
    ) -> Result<(), StorageError> {
        self.edge_manager
            .update_edge_indexes_mvcc(space_id, src, dst, index_name, props, write_ts)
    }

    fn delete_vertex_indexes_mvcc(
        &self,
        space_id: u64,
        vertex_id: &Value,
        write_ts: Timestamp,
    ) -> Result<(), StorageError> {
        self.vertex_manager
            .delete_vertex_indexes_mvcc(space_id, vertex_id, write_ts)
    }

    fn delete_vertex_index_single_mvcc(
        &self,
        space_id: u64,
        vertex_id: &Value,
        index_name: &str,
        prop_value: &Value,
        write_ts: Timestamp,
    ) -> Result<(), StorageError> {
        self.vertex_manager
            .delete_vertex_index_single(space_id, vertex_id, index_name, prop_value, write_ts)
    }

    fn delete_edge_indexes_mvcc(
        &self,
        space_id: u64,
        src: &Value,
        dst: &Value,
        index_names: &[String],
        write_ts: Timestamp,
    ) -> Result<(), StorageError> {
        self.edge_manager
            .delete_edge_indexes_mvcc(space_id, src, dst, index_names, write_ts)
    }

    fn delete_edge_index_single_mvcc(
        &self,
        space_id: u64,
        src: &Value,
        dst: &Value,
        index_name: &str,
        prop_value: &Value,
        write_ts: Timestamp,
    ) -> Result<(), StorageError> {
        self.edge_manager
            .delete_edge_index_single(space_id, src, dst, index_name, prop_value, write_ts)
    }

    fn lookup_tag_index_mvcc(
        &self,
        space_id: u64,
        index: &Index,
        value: &Value,
        read_ts: Timestamp,
    ) -> Result<Vec<Value>, StorageError> {
        self.vertex_manager
            .lookup_tag_index_mvcc(space_id, index, value, read_ts)
    }

    fn lookup_edge_index_mvcc(
        &self,
        space_id: u64,
        index: &Index,
        value: &Value,
        read_ts: Timestamp,
    ) -> Result<Vec<Value>, StorageError> {
        self.edge_manager
            .lookup_edge_index_mvcc(space_id, index, value, read_ts)
    }

    fn clear_edge_index(&self, space_id: u64, index_name: &str) -> Result<(), StorageError> {
        self.edge_manager.clear_edge_index(space_id, index_name)
    }

    fn build_edge_index_entry(
        &self,
        space_id: u64,
        index: &Index,
        edge: &Edge,
    ) -> Result<(), StorageError> {
        let src_value = Value::from(edge.src.clone());
        let dst_value = Value::from(edge.dst.clone());
        for field in &index.fields {
            if let Some(prop_value) = edge.props.get(&field.name) {
                self.edge_manager.update_edge_indexes(
                    space_id,
                    &src_value,
                    &dst_value,
                    &index.name,
                    &[(field.name.clone(), prop_value.clone())],
                )?;
            }
        }
        Ok(())
    }

    fn delete_tag_indexes(
        &self,
        space_id: u64,
        vertex_id: &Value,
        tag_name: &str,
    ) -> Result<(), StorageError> {
        self.vertex_manager
            .delete_tag_indexes(space_id, vertex_id, tag_name)
    }

    fn clear_tag_index(&self, space_id: u64, index_name: &str) -> Result<(), StorageError> {
        self.vertex_manager.clear_tag_index(space_id, index_name)
    }

    fn build_vertex_index_entry(
        &self,
        space_id: u64,
        index: &Index,
        vertex_id: &Value,
        tag: &Tag,
    ) -> Result<(), StorageError> {
        for field in &index.fields {
            if let Some(prop_value) = tag.properties.get(&field.name) {
                self.vertex_manager.update_vertex_indexes(
                    space_id,
                    vertex_id,
                    &index.name,
                    &[(field.name.clone(), prop_value.clone())],
                )?;
            }
        }
        Ok(())
    }

    fn update_vertex_indexes_native_mvcc(
        &self,
        space_id: u64,
        vertex_id: u64,
        index_name: &str,
        props: &[(String, Value)],
        write_ts: Timestamp,
    ) -> Result<(), StorageError> {
        self.vertex_manager
            .update_vertex_indexes_native_mvcc(space_id, vertex_id, index_name, props, write_ts)
    }

    fn delete_vertex_indexes_native_mvcc(
        &self,
        space_id: u64,
        vertex_id: u64,
        write_ts: Timestamp,
    ) -> Result<(), StorageError> {
        self.vertex_manager
            .delete_vertex_indexes_native_mvcc(space_id, vertex_id, write_ts)
    }

    fn lookup_tag_index_native_mvcc(
        &self,
        space_id: u64,
        index: &Index,
        value: &Value,
        read_ts: Timestamp,
    ) -> Result<Vec<u64>, StorageError> {
        self.vertex_manager
            .lookup_tag_index_native_mvcc(space_id, index, value, read_ts)
    }

    fn update_edge_indexes_native_mvcc(
        &self,
        space_id: u64,
        src: u64,
        dst: u64,
        index_name: &str,
        props: &[(String, Value)],
        write_ts: Timestamp,
    ) -> Result<(), StorageError> {
        self.edge_manager
            .update_edge_indexes_native_mvcc(space_id, src, dst, index_name, props, write_ts)
    }

    fn delete_edge_indexes_native_mvcc(
        &self,
        space_id: u64,
        src: u64,
        dst: u64,
        index_names: &[String],
        write_ts: Timestamp,
    ) -> Result<(), StorageError> {
        self.edge_manager
            .delete_edge_indexes_native_mvcc(space_id, src, dst, index_names, write_ts)
    }

    fn lookup_edge_index_native_mvcc(
        &self,
        space_id: u64,
        index: &Index,
        value: &Value,
        read_ts: Timestamp,
    ) -> Result<Vec<(u64, u64)>, StorageError> {
        self.edge_manager
            .lookup_edge_index_native_mvcc(space_id, index, value, read_ts)
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
    fn test_serialize_deserialize_value() {
        let value = Value::String("test".to_string());
        let bytes =
            InMemoryIndexDataManager::serialize_value(&value).expect("serialize should succeed");
        let decoded = InMemoryIndexDataManager::deserialize_value(&bytes)
            .expect("deserialize should succeed");
        assert_eq!(value, decoded);
    }

    #[test]
    fn test_update_and_lookup_vertex_index() {
        let manager = InMemoryIndexDataManager::new();

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
    }
}
