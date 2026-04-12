//! Index Data Manager
//!
//! Provide update, delete and query functions for indexed data
//! The management of index metadata is handled by the IndexMetadataManager.
//! All operations identify a space by its space_id, enabling multi-space data segregation.

use crate::core::types::Index;
use crate::core::vertex_edge_path::Tag;
use crate::core::Edge;
use crate::core::{StorageError, Value};
use crate::storage::index::edge_index_manager::EdgeIndexManager;
use crate::storage::index::index_key_codec::IndexKeyCodec;
use crate::storage::index::vertex_index_manager::VertexIndexManager;
use crate::storage::engine::{ByteKey, INDEX_DATA_TABLE};

use redb::Database;
use std::sync::Arc;

/// Index key type identifier
pub use crate::storage::index::index_key_codec::{
    KEY_TYPE_EDGE_FORWARD, KEY_TYPE_EDGE_REVERSE, KEY_TYPE_VERTEX_FORWARD, KEY_TYPE_VERTEX_REVERSE,
};

/// Indexed Data Manager trait
///
/// Provide functions for adding, deleting, modifying, and querying index data.
/// All operations identify a space by its space_id, enabling multi-space data segregation.
pub trait IndexDataManager {
    /// Update the vertex index
    fn update_vertex_indexes(
        &self,
        space_id: u64,
        vertex_id: &Value,
        index_name: &str,
        props: &[(String, Value)],
    ) -> Result<(), StorageError>;
    /// Updating the side index
    fn update_edge_indexes(
        &self,
        space_id: u64,
        src: &Value,
        dst: &Value,
        index_name: &str,
        props: &[(String, Value)],
    ) -> Result<(), StorageError>;
    /// Remove all indexes from the vertex.
    fn delete_vertex_indexes(&self, space_id: u64, vertex_id: &Value) -> Result<(), StorageError>;
    /// Delete all indexes on side
    fn delete_edge_indexes(
        &self,
        space_id: u64,
        src: &Value,
        dst: &Value,
        index_names: &[String],
    ) -> Result<(), StorageError>;
    /// Search for the tag index.
    fn lookup_tag_index(
        &self,
        space_id: u64,
        index: &Index,
        value: &Value,
    ) -> Result<Vec<Value>, StorageError>;
    /// Find Side Index
    fn lookup_edge_index(
        &self,
        space_id: u64,
        index: &Index,
        value: &Value,
    ) -> Result<Vec<Value>, StorageError>;
    /// Clear the edge index.
    fn clear_edge_index(&self, space_id: u64, index_name: &str) -> Result<(), StorageError>;
    /// Constructing side index entries
    fn build_edge_index_entry(
        &self,
        space_id: u64,
        index: &Index,
        edge: &Edge,
    ) -> Result<(), StorageError>;
    /// Delete the index containing the specified tags.
    fn delete_tag_indexes(
        &self,
        space_id: u64,
        vertex_id: &Value,
        tag_name: &str,
    ) -> Result<(), StorageError>;
    /// Clearing the tag index
    fn clear_tag_index(&self, space_id: u64, index_name: &str) -> Result<(), StorageError>;
    /// Constructing vertex index entries
    fn build_vertex_index_entry(
        &self,
        space_id: u64,
        index: &Index,
        vertex_id: &Value,
        tag: &Tag,
    ) -> Result<(), StorageError>;
}

/// Redb-based Indexed Data Manager Implementation
#[derive(Clone)]
pub struct RedbIndexDataManager {
    db: Arc<Database>,
    vertex_manager: VertexIndexManager,
    edge_manager: EdgeIndexManager,
}

impl RedbIndexDataManager {
    pub fn new(db: Arc<Database>) -> Self {
        let vertex_manager = VertexIndexManager::new(db.clone());
        let edge_manager = EdgeIndexManager::new(db.clone());
        Self {
            db,
            vertex_manager,
            edge_manager,
        }
    }

    /// Serialized values (for backward compatibility)
    pub fn serialize_value(value: &Value) -> Result<Vec<u8>, StorageError> {
        IndexKeyCodec::serialize_value(value)
    }

    /// Deserialized values (backward compatible)
    pub fn deserialize_value(data: &[u8]) -> Result<Value, StorageError> {
        IndexKeyCodec::deserialize_value(data)
    }

    /// Constructing a forward-index key for vertices (backward compatible)
    pub fn build_vertex_index_key(
        space_id: u64,
        index_name: &str,
        prop_value: &Value,
        vertex_id: &Value,
    ) -> Result<ByteKey, StorageError> {
        IndexKeyCodec::build_vertex_index_key(space_id, index_name, prop_value, vertex_id)
    }

    /// Construct vertex forward index key prefixes (backward compatible)
    pub fn build_vertex_index_prefix(space_id: u64, index_name: &str) -> ByteKey {
        IndexKeyCodec::build_vertex_index_prefix(space_id, index_name)
    }

    /// Parse the `vertex_id` from the positive index keys at the vertex (backward compatible).
    pub fn parse_vertex_id_from_key(key_bytes: &[u8]) -> Result<Value, StorageError> {
        IndexKeyCodec::parse_vertex_id_from_key(key_bytes)
    }

    /// Construct vertex reverse index keys (backward compatible)
    pub fn build_vertex_reverse_key(
        space_id: u64,
        index_name: &str,
        vertex_id: &Value,
    ) -> Result<ByteKey, StorageError> {
        IndexKeyCodec::build_vertex_reverse_key(space_id, index_name, vertex_id)
    }

    /// Constructing a prefix for the vertex reverse index key (backward compatibility)
    pub fn build_vertex_reverse_prefix(space_id: u64) -> ByteKey {
        IndexKeyCodec::build_vertex_reverse_prefix(space_id)
    }

    /// Resolve vertex reverse index keys (backward compatible)
    pub fn parse_vertex_reverse_key(key_bytes: &[u8]) -> Result<(String, Vec<u8>), StorageError> {
        IndexKeyCodec::parse_vertex_reverse_key(key_bytes)
    }

    /// Constructing edge forward index keys (backward compatible)
    pub fn build_edge_index_key(
        space_id: u64,
        index_name: &str,
        prop_value: &Value,
        src: &Value,
        dst: &Value,
    ) -> Result<ByteKey, StorageError> {
        IndexKeyCodec::build_edge_index_key(space_id, index_name, prop_value, src, dst)
    }

    /// Constructed edge forward index key prefix (backward compatible)
    pub fn build_edge_index_prefix(space_id: u64, index_name: &str) -> ByteKey {
        IndexKeyCodec::build_edge_index_prefix(space_id, index_name)
    }

    /// Constructing edge reverse index keys (backward compatible)
    pub fn build_edge_reverse_key(
        space_id: u64,
        index_name: &str,
        src: &Value,
    ) -> Result<ByteKey, StorageError> {
        IndexKeyCodec::build_edge_reverse_key(space_id, index_name, src)
    }

    /// Constructed Edge Reverse Index Key Prefixes (Backwards Compatible)
    pub fn build_edge_reverse_prefix(space_id: u64) -> ByteKey {
        IndexKeyCodec::build_edge_reverse_prefix(space_id)
    }

    /// End key for constructing range queries (backward compatibility)
    pub fn build_range_end(prefix: &ByteKey) -> ByteKey {
        IndexKeyCodec::build_range_end(prefix)
    }

    /// Parse side reverse index key (backward compatible)
    pub fn parse_edge_reverse_key(key_bytes: &[u8]) -> Result<(String, Vec<u8>), StorageError> {
        IndexKeyCodec::parse_edge_reverse_key(key_bytes)
    }
}

impl IndexDataManager for RedbIndexDataManager {
    fn update_vertex_indexes(
        &self,
        space_id: u64,
        vertex_id: &Value,
        index_name: &str,
        props: &[(String, Value)],
    ) -> Result<(), StorageError> {
        self.vertex_manager
            .update_vertex_indexes(space_id, vertex_id, index_name, props)
    }

    fn update_edge_indexes(
        &self,
        space_id: u64,
        src: &Value,
        dst: &Value,
        index_name: &str,
        props: &[(String, Value)],
    ) -> Result<(), StorageError> {
        self.edge_manager
            .update_edge_indexes(space_id, src, dst, index_name, props)
    }

    fn delete_vertex_indexes(&self, space_id: u64, vertex_id: &Value) -> Result<(), StorageError> {
        self.vertex_manager
            .delete_vertex_indexes(space_id, vertex_id)
    }

    fn delete_edge_indexes(
        &self,
        space_id: u64,
        src: &Value,
        dst: &Value,
        index_names: &[String],
    ) -> Result<(), StorageError> {
        self.edge_manager
            .delete_edge_indexes(space_id, src, dst, index_names)
    }

    fn lookup_tag_index(
        &self,
        space_id: u64,
        index: &Index,
        value: &Value,
    ) -> Result<Vec<Value>, StorageError> {
        self.vertex_manager.lookup_tag_index(space_id, index, value)
    }

    fn lookup_edge_index(
        &self,
        space_id: u64,
        index: &Index,
        value: &Value,
    ) -> Result<Vec<Value>, StorageError> {
        self.edge_manager.lookup_edge_index(space_id, index, value)
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
        let txn = self.db.begin_write().map_err(|e| {
            StorageError::DbError(format!("Failed to start write transaction: {}", e))
        })?;

        {
            let mut table = txn.open_table(INDEX_DATA_TABLE).map_err(|e| {
                StorageError::DbError(format!("Failed to open INDEX_DATA_TABLE: {}", e))
            })?;

            for field in &index.fields {
                if let Some(prop_value) = edge.props.get(&field.name) {
                    let index_key = IndexKeyCodec::build_edge_index_key(
                        space_id,
                        &index.name,
                        prop_value,
                        &edge.src,
                        &edge.dst,
                    )?;

                    table
                        .insert(index_key, ByteKey(field.name.as_bytes().to_vec()))
                        .map_err(|e| {
                            StorageError::DbError(format!(
                                "Failed to insert edge index data: {}",
                                e
                            ))
                        })?;

                    let reverse_key =
                        IndexKeyCodec::build_edge_reverse_key(space_id, &index.name, &edge.src)?;
                    let prop_value_bytes = IndexKeyCodec::serialize_value(prop_value)?;
                    let value_key = format!("{}:{}", field.name, prop_value_bytes.len());
                    table
                        .insert(reverse_key, ByteKey(value_key.into_bytes()))
                        .map_err(|e| {
                            StorageError::DbError(format!(
                                "Failed to insert edge reverse index: {}",
                                e
                            ))
                        })?;
                }
            }
        }

        txn.commit()
            .map_err(|e| StorageError::DbError(format!("Failed to commit transaction: {}", e)))?;

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
        let txn = self.db.begin_write().map_err(|e| {
            StorageError::DbError(format!("Failed to start write transaction: {}", e))
        })?;

        {
            let mut table = txn.open_table(INDEX_DATA_TABLE).map_err(|e| {
                StorageError::DbError(format!("Failed to open INDEX_DATA_TABLE: {}", e))
            })?;

            for field in &index.fields {
                if let Some(prop_value) = tag.properties.get(&field.name) {
                    let index_key = IndexKeyCodec::build_vertex_index_key(
                        space_id,
                        &index.name,
                        prop_value,
                        vertex_id,
                    )?;

                    table
                        .insert(&index_key, ByteKey(field.name.as_bytes().to_vec()))
                        .map_err(|e| {
                            StorageError::DbError(format!(
                                "Failed to insert vertex index data: {}",
                                e
                            ))
                        })?;

                    let reverse_key =
                        IndexKeyCodec::build_vertex_reverse_key(space_id, &index.name, vertex_id)?;
                    let prop_value_bytes = IndexKeyCodec::serialize_value(prop_value)?;
                    let value_key = format!("{}:{}", field.name, prop_value_bytes.len());
                    table
                        .insert(&reverse_key, ByteKey(value_key.into_bytes()))
                        .map_err(|e| {
                            StorageError::DbError(format!(
                                "Failed to insert vertex reverse index: {}",
                                e
                            ))
                        })?;
                }
            }
        }

        txn.commit()
            .map_err(|e| StorageError::DbError(format!("Failed to commit transaction: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{Index, IndexConfig, IndexField, IndexType};
    use crate::core::Value;
    use std::sync::Arc;
    use tempfile::TempDir;

    fn create_test_db() -> (Arc<Database>, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
        let db_path = temp_dir.path().join("test.db");
        let db = Arc::new(Database::create(&db_path).expect("Failed to create test database"));

        let txn = db.begin_write().expect("Failed to begin write transaction");
        {
            let _ = txn
                .open_table(INDEX_DATA_TABLE)
                .expect("Failed to open table");
        }
        txn.commit().expect("Failed to commit transaction");

        (db, temp_dir)
    }

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
        })
    }

    #[test]
    fn test_build_vertex_index_key() {
        let (db, _temp_dir) = create_test_db();
        let _manager = RedbIndexDataManager::new(db);

        let space_id = 1u64;
        let index_name = "idx_test";
        let prop_value = Value::String("test_value".to_string());
        let vertex_id = Value::Int(123);

        let key = RedbIndexDataManager::build_vertex_index_key(
            space_id,
            index_name,
            &prop_value,
            &vertex_id,
        )
        .expect("Failed to build vertex index key");

        assert!(key.0.len() > 9);
        assert_eq!(key.0[8], KEY_TYPE_VERTEX_FORWARD);

        let parsed_vid = RedbIndexDataManager::parse_vertex_id_from_key(&key.0)
            .expect("Failed to parse vertex id from key");
        assert_eq!(parsed_vid, vertex_id);
    }

    #[test]
    fn test_build_vertex_reverse_key() {
        let (db, _temp_dir) = create_test_db();
        let _manager = RedbIndexDataManager::new(db);

        let space_id = 1u64;
        let index_name = "idx_test";
        let vertex_id = Value::Int(456);

        let key = RedbIndexDataManager::build_vertex_reverse_key(space_id, index_name, &vertex_id)
            .expect("Failed to build vertex reverse key");

        assert!(key.0.len() > 9);
        assert_eq!(key.0[8], KEY_TYPE_VERTEX_REVERSE);

        let (parsed_name, parsed_vid_bytes) =
            RedbIndexDataManager::parse_vertex_reverse_key(&key.0)
                .expect("Failed to parse vertex reverse key");
        assert_eq!(parsed_name, index_name);
        let parsed_vid = RedbIndexDataManager::deserialize_value(&parsed_vid_bytes)
            .expect("Failed to deserialize value");
        assert_eq!(parsed_vid, vertex_id);
    }

    #[test]
    fn test_build_edge_index_key() {
        let (db, _temp_dir) = create_test_db();
        let _manager = RedbIndexDataManager::new(db);

        let space_id = 1u64;
        let index_name = "idx_edge_test";
        let prop_value = Value::String("edge_prop".to_string());
        let src = Value::Int(100);
        let dst = Value::Int(200);

        let key = RedbIndexDataManager::build_edge_index_key(
            space_id,
            index_name,
            &prop_value,
            &src,
            &dst,
        )
        .expect("Failed to build edge index key");

        assert!(key.0.len() > 9);
        assert_eq!(key.0[8], KEY_TYPE_EDGE_FORWARD);
    }

    #[test]
    fn test_build_edge_reverse_key() {
        let (db, _temp_dir) = create_test_db();
        let _manager = RedbIndexDataManager::new(db);

        let space_id = 1u64;
        let index_name = "idx_edge_test";
        let src = Value::Int(300);

        let key = RedbIndexDataManager::build_edge_reverse_key(space_id, index_name, &src)
            .expect("Failed to build edge reverse key");

        assert!(key.0.len() > 9);
        assert_eq!(key.0[8], KEY_TYPE_EDGE_REVERSE);

        let (parsed_name, parsed_src_bytes) = RedbIndexDataManager::parse_edge_reverse_key(&key.0)
            .expect("Failed to parse edge reverse key");
        assert_eq!(parsed_name, index_name);
        let parsed_src = RedbIndexDataManager::deserialize_value(&parsed_src_bytes)
            .expect("Failed to deserialize value");
        assert_eq!(parsed_src, src);
    }

    #[test]
    fn test_update_and_lookup_vertex_index() {
        let (db, _temp_dir) = create_test_db();
        let manager = RedbIndexDataManager::new(db);

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
    fn test_update_and_lookup_edge_index() {
        let (db, _temp_dir) = create_test_db();
        let manager = RedbIndexDataManager::new(db);

        let space_id = 1u64;
        let src = Value::Int(1);
        let dst = Value::Int(2);
        let index_name = "idx_edge_weight";
        let props = vec![("weight".to_string(), Value::Float(10.5))];

        manager
            .update_edge_indexes(space_id, &src, &dst, index_name, &props)
            .expect("Failed to update edge indexes");

        let mut index = create_test_index(index_name, "knows");
        index.index_type = IndexType::EdgeIndex;

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
    fn test_delete_vertex_indexes() {
        let (db, _temp_dir) = create_test_db();
        let manager = RedbIndexDataManager::new(db);

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
    fn test_delete_edge_indexes() {
        let (db, _temp_dir) = create_test_db();
        let manager = RedbIndexDataManager::new(db);

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

        let mut index = create_test_index(index_name, "knows");
        index.index_type = IndexType::EdgeIndex;

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
        let (db, _temp_dir) = create_test_db();
        let manager = RedbIndexDataManager::new(db);

        let space_id = 1u64;
        let src = Value::Int(1);
        let dst = Value::Int(2);
        let index_name = "idx_edge_weight";

        let props = vec![("weight".to_string(), Value::Float(10.5))];
        manager
            .update_edge_indexes(space_id, &src, &dst, index_name, &props)
            .expect("Failed to update edge indexes");

        let mut index = create_test_index(index_name, "knows");
        index.index_type = IndexType::EdgeIndex;
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

    #[test]
    fn test_multiple_properties_index() {
        let (db, _temp_dir) = create_test_db();
        let manager = RedbIndexDataManager::new(db);

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

    #[test]
    fn test_binary_value_in_key() {
        let (db, _temp_dir) = create_test_db();
        let manager = RedbIndexDataManager::new(db);

        let space_id = 1u64;
        let vertex_id = Value::Int(1);
        let index_name = "idx_test";

        let props = vec![(
            "data".to_string(),
            Value::String("hello:world:test".to_string()),
        )];

        manager
            .update_vertex_indexes(space_id, &vertex_id, index_name, &props)
            .expect("Failed to update vertex indexes");

        let index = create_test_index(index_name, "test");

        let results = manager
            .lookup_tag_index(
                space_id,
                &index,
                &Value::String("hello:world:test".to_string()),
            )
            .expect("Failed to lookup tag index");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], vertex_id);
    }

    #[test]
    fn test_different_space_isolation() {
        let (db, _temp_dir) = create_test_db();
        let manager = RedbIndexDataManager::new(db);

        let space_id1 = 1u64;
        let space_id2 = 2u64;
        let vertex_id = Value::Int(1);
        let index_name = "idx_person_name";
        let props = vec![("name".to_string(), Value::String("Alice".to_string()))];

        manager
            .update_vertex_indexes(space_id1, &vertex_id, index_name, &props.clone())
            .expect("Failed to update vertex indexes");
        manager
            .update_vertex_indexes(space_id2, &vertex_id, index_name, &props)
            .expect("Failed to update vertex indexes");

        let index = create_test_index(index_name, "person");

        let results1 = manager
            .lookup_tag_index(space_id1, &index, &Value::String("Alice".to_string()))
            .expect("Failed to lookup tag index");
        assert_eq!(results1.len(), 1);

        let results2 = manager
            .lookup_tag_index(space_id2, &index, &Value::String("Alice".to_string()))
            .expect("Failed to lookup tag index");
        assert_eq!(results2.len(), 1);

        manager
            .delete_vertex_indexes(space_id1, &vertex_id)
            .expect("Failed to delete vertex indexes");

        let results1_after = manager
            .lookup_tag_index(space_id1, &index, &Value::String("Alice".to_string()))
            .expect("Failed to lookup tag index");
        assert!(results1_after.is_empty());

        let results2_after = manager
            .lookup_tag_index(space_id2, &index, &Value::String("Alice".to_string()))
            .expect("Failed to lookup tag index");
        assert_eq!(results2_after.len(), 1);
    }
}
