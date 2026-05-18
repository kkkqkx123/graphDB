//! Index Scan Iterator - provides efficient iteration using secondary indexes
//!
//! This module provides iterators that leverage secondary indexes for efficient
//! data retrieval, avoiding full table scans when appropriate indexes exist.

use crate::core::{StorageResult, Value};
use crate::storage::index::secondary::{
    Timestamp, VertexIndexManager,
};
use crate::storage::vertex::{VertexRecord, VertexTable};

pub struct IndexScanIterator<'a> {
    vertex_table: &'a VertexTable,
    ts: Timestamp,
    vertex_ids: Vec<Value>,
    current_idx: usize,
}

impl<'a> IndexScanIterator<'a> {
    pub fn new(
        _index_manager: &'a VertexIndexManager,
        vertex_table: &'a VertexTable,
        _space_id: u64,
        _index_name: String,
        ts: Timestamp,
    ) -> Self {
        Self {
            vertex_table,
            ts,
            vertex_ids: Vec::new(),
            current_idx: 0,
        }
    }

    pub fn with_range(
        index_manager: &'a VertexIndexManager,
        vertex_table: &'a VertexTable,
        space_id: u64,
        index: &crate::core::types::Index,
        start_value: &Value,
        end_value: &Value,
        ts: Timestamp,
    ) -> StorageResult<Self> {
        let vertex_ids = index_manager.lookup_tag_index_range_mvcc(
            space_id,
            index,
            start_value,
            end_value,
            ts,
        )?;

        Ok(Self {
            vertex_table,
            ts,
            vertex_ids,
            current_idx: 0,
        })
    }

    pub fn with_exact_match(
        index_manager: &'a VertexIndexManager,
        vertex_table: &'a VertexTable,
        space_id: u64,
        index: &crate::core::types::Index,
        value: &Value,
        ts: Timestamp,
    ) -> StorageResult<Self> {
        let vertex_ids = index_manager.lookup_tag_index_mvcc(
            space_id,
            index,
            value,
            ts,
        )?;

        Ok(Self {
            vertex_table,
            ts,
            vertex_ids,
            current_idx: 0,
        })
    }

    pub fn with_vertex_ids(
        vertex_table: &'a VertexTable,
        ts: Timestamp,
        vertex_ids: Vec<Value>,
    ) -> Self {
        Self {
            vertex_table,
            ts,
            vertex_ids,
            current_idx: 0,
        }
    }

    pub fn vertex_ids(&self) -> &[Value] {
        &self.vertex_ids
    }

    pub fn len(&self) -> usize {
        self.vertex_ids.len()
    }

    pub fn is_empty(&self) -> bool {
        self.vertex_ids.is_empty()
    }
}

impl<'a> Iterator for IndexScanIterator<'a> {
    type Item = VertexRecord;

    fn next(&mut self) -> Option<Self::Item> {
        while self.current_idx < self.vertex_ids.len() {
            let vertex_id = &self.vertex_ids[self.current_idx];
            self.current_idx += 1;

            let id_str = match vertex_id {
                Value::String(s) => s.clone(),
                Value::Int(i) => i.to_string(),
                _ => vertex_id.to_string().unwrap_or_default(),
            };

            if let Some(internal_id) = self.vertex_table.get_internal_id(&id_str, self.ts) {
                if let Some(record) = self.vertex_table.get_by_internal_id(internal_id, self.ts) {
                    return Some(record);
                }
            }
        }
        None
    }
}

pub struct IndexScanConfig {
    pub batch_size: usize,
}

impl Default for IndexScanConfig {
    fn default() -> Self {
        Self {
            batch_size: 1000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::DataType;
    use crate::core::Value;
    use crate::storage::index::secondary::VertexIndexManager;
    use crate::storage::vertex::{PropertyDef, VertexSchema, VertexTable};

    fn create_test_table() -> VertexTable {
        let schema = VertexSchema {
            label_id: 0,
            label_name: "person".to_string(),
            properties: vec![
                PropertyDef::new("name".to_string(), DataType::String),
                PropertyDef::new("age".to_string(), DataType::Int),
            ],
            primary_key_index: 0,
        };

        let mut table = VertexTable::new(0, "person".to_string(), schema);
        let ts = 100u32;

        table
            .insert(
                "1",
                &[
                    ("name".to_string(), Value::String("Alice".to_string())),
                    ("age".to_string(), Value::Int(25)),
                ],
                ts,
            )
            .expect("insert should succeed");

        table
            .insert(
                "2",
                &[
                    ("name".to_string(), Value::String("Bob".to_string())),
                    ("age".to_string(), Value::Int(30)),
                ],
                ts,
            )
            .expect("insert should succeed");

        table
            .insert(
                "3",
                &[
                    ("name".to_string(), Value::String("Charlie".to_string())),
                    ("age".to_string(), Value::Int(35)),
                ],
                ts,
            )
            .expect("insert should succeed");

        table
    }

    #[test]
    fn test_index_scan_iterator_basic() {
        let table = create_test_table();
        let index_manager = VertexIndexManager::new();

        let iter = IndexScanIterator::new(
            &index_manager,
            &table,
            1,
            "person_age_idx".to_string(),
            100,
        );

        let results: Vec<_> = iter.collect();
        assert!(results.is_empty());
    }

    #[test]
    fn test_index_scan_iterator_with_vertex_ids() {
        let table = create_test_table();

        let iter = IndexScanIterator::with_vertex_ids(
            &table,
            100,
            vec![Value::String("1".to_string()), Value::String("2".to_string())],
        );

        let results: Vec<_> = iter.collect();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_vertex_table_and_index_integration() {
        let schema = VertexSchema {
            label_id: 0,
            label_name: "person".to_string(),
            properties: vec![
                PropertyDef::new("name".to_string(), DataType::String),
                PropertyDef::new("age".to_string(), DataType::Int),
            ],
            primary_key_index: 0,
        };

        let mut table = VertexTable::new(0, "person".to_string(), schema);
        let index_manager = VertexIndexManager::new();
        let space_id = 1u64;
        let index_name = "idx_person_name";
        let ts_create: Timestamp = 100;
        let ts_delete: Timestamp = 200;
        let ts_read_before: Timestamp = 150;
        let ts_read_after: Timestamp = 250;

        table
            .insert(
                "v1",
                &[
                    ("name".to_string(), Value::String("Alice".to_string())),
                    ("age".to_string(), Value::Int(30)),
                ],
                ts_create,
            )
            .expect("insert v1");

        index_manager
            .update_vertex_indexes_mvcc(
                space_id,
                &Value::String("v1".to_string()),
                index_name,
                &[("name".to_string(), Value::String("Alice".to_string()))],
                ts_create,
            )
            .expect("index v1");

        table
            .insert(
                "v2",
                &[
                    ("name".to_string(), Value::String("Bob".to_string())),
                    ("age".to_string(), Value::Int(25)),
                ],
                ts_create,
            )
            .expect("insert v2");

        index_manager
            .update_vertex_indexes_mvcc(
                space_id,
                &Value::String("v2".to_string()),
                index_name,
                &[("name".to_string(), Value::String("Bob".to_string()))],
                ts_create,
            )
            .expect("index v2");

        let alice_records = index_manager
            .lookup_tag_index_mvcc(
                space_id,
                &crate::core::types::Index::new(crate::core::types::IndexConfig {
                    id: 1,
                    name: index_name.to_string(),
                    space_id,
                    schema_name: "person".to_string(),
                    fields: vec![],
                    properties: vec![],
                    index_type: crate::core::types::IndexType::TagIndex,
                    is_unique: false,
                    partial_condition: None,
                }),
                &Value::String("Alice".to_string()),
                ts_read_before,
            )
            .expect("lookup Alice");
        assert_eq!(alice_records.len(), 1);
        assert_eq!(alice_records[0], Value::String("v1".to_string()));

        table.delete("v1", ts_delete).expect("delete v1");
        index_manager
            .delete_vertex_indexes_mvcc(space_id, &Value::String("v1".to_string()), ts_delete)
            .expect("delete index v1");

        let alice_records_after = index_manager
            .lookup_tag_index_mvcc(
                space_id,
                &crate::core::types::Index::new(crate::core::types::IndexConfig {
                    id: 1,
                    name: index_name.to_string(),
                    space_id,
                    schema_name: "person".to_string(),
                    fields: vec![],
                    properties: vec![],
                    index_type: crate::core::types::IndexType::TagIndex,
                    is_unique: false,
                    partial_condition: None,
                }),
                &Value::String("Alice".to_string()),
                ts_read_after,
            )
            .expect("lookup Alice after delete");
        assert!(
            alice_records_after.is_empty(),
            "Alice should not be found after deletion"
        );

        let bob_records = index_manager
            .lookup_tag_index_mvcc(
                space_id,
                &crate::core::types::Index::new(crate::core::types::IndexConfig {
                    id: 1,
                    name: index_name.to_string(),
                    space_id,
                    schema_name: "person".to_string(),
                    fields: vec![],
                    properties: vec![],
                    index_type: crate::core::types::IndexType::TagIndex,
                    is_unique: false,
                    partial_condition: None,
                }),
                &Value::String("Bob".to_string()),
                ts_read_after,
            )
            .expect("lookup Bob");
        assert_eq!(bob_records.len(), 1);
        assert_eq!(bob_records[0], Value::String("v2".to_string()));

        let removed = table.compact_with_ts_collect(ts_delete);
        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0], crate::storage::vertex::IdKey::Text("v1".to_string()));

        let gc_removed = index_manager
            .gc_tombstones(ts_delete + 1)
            .expect("gc tombstones");
        assert!(gc_removed > 0, "should clean up tombstoned index entries");

        let tombstone_count = index_manager.tombstone_count();
        assert_eq!(
            tombstone_count, 0,
            "all tombstones should be cleaned after gc"
        );
    }
}
