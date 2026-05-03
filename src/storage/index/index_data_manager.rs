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

pub use crate::storage::index::index_key_codec::{
    KEY_TYPE_EDGE_FORWARD, KEY_TYPE_EDGE_REVERSE, KEY_TYPE_VERTEX_FORWARD, KEY_TYPE_VERTEX_REVERSE,
};

pub trait IndexDataManager {
    fn update_vertex_indexes(
        &self,
        space_id: u64,
        vertex_id: &Value,
        index_name: &str,
        props: &[(String, Value)],
    ) -> Result<(), StorageError>;
    fn update_edge_indexes(
        &self,
        space_id: u64,
        src: &Value,
        dst: &Value,
        index_name: &str,
        props: &[(String, Value)],
    ) -> Result<(), StorageError>;
    fn delete_vertex_indexes(&self, space_id: u64, vertex_id: &Value) -> Result<(), StorageError>;
    fn delete_edge_indexes(
        &self,
        space_id: u64,
        src: &Value,
        dst: &Value,
        index_names: &[String],
    ) -> Result<(), StorageError>;
    fn lookup_tag_index(
        &self,
        space_id: u64,
        index: &Index,
        value: &Value,
    ) -> Result<Vec<Value>, StorageError>;
    fn lookup_edge_index(
        &self,
        space_id: u64,
        index: &Index,
        value: &Value,
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
}

#[derive(Clone)]
pub struct RedbIndexDataManager {
    vertex_manager: VertexIndexManager,
    edge_manager: EdgeIndexManager,
}

impl RedbIndexDataManager {
    pub fn new() -> Self {
        Self {
            vertex_manager: VertexIndexManager::new(),
            edge_manager: EdgeIndexManager::new(),
        }
    }

    pub fn serialize_value(value: &Value) -> Result<Vec<u8>, StorageError> {
        IndexKeyCodec::serialize_value(value)
    }

    pub fn deserialize_value(data: &[u8]) -> Result<Value, StorageError> {
        IndexKeyCodec::deserialize_value(data)
    }
}

impl Default for RedbIndexDataManager {
    fn default() -> Self {
        Self::new()
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
        for field in &index.fields {
            if let Some(prop_value) = edge.props.get(&field.name) {
                self.edge_manager.update_edge_indexes(
                    space_id,
                    &edge.src,
                    &edge.dst,
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
        })
    }

    #[test]
    fn test_serialize_deserialize_value() {
        let value = Value::String("test".to_string());
        let bytes = RedbIndexDataManager::serialize_value(&value).expect("serialize should succeed");
        let decoded = RedbIndexDataManager::deserialize_value(&bytes).expect("deserialize should succeed");
        assert_eq!(value, decoded);
    }

    #[test]
    fn test_update_and_lookup_vertex_index() {
        let manager = RedbIndexDataManager::new();

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
