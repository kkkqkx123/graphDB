//! 测试用Mock存储引擎实现
//!
//! 提供统一的Mock存储引擎实现，避免在各个测试模块中重复实现

#[cfg(test)]
use crate::core::value::NullType;
#[cfg(test)]
use crate::core::{
    vertex_edge_path::Edge,
    EdgeDirection, Value,
};
#[cfg(test)]
use crate::core::vertex_edge_path::Vertex;
#[cfg(test)]
use crate::core::error::StorageError;
#[cfg(test)]
use crate::storage::StorageClient;
#[cfg(test)]
use crate::storage::transaction::TransactionId;
#[cfg(test)]
use crate::core::types::{
    EdgeTypeSchema, IndexInfo, InsertEdgeInfo, InsertVertexInfo, PasswordInfo,
    PropertyDef, SpaceInfo, TagInfo, UpdateInfo,
};
#[cfg(test)]
use crate::storage::Schema;

/// 测试用Mock存储引擎
#[cfg(test)]
#[derive(Debug)]
pub struct MockStorage;

#[cfg(test)]
impl StorageClient for MockStorage {
    fn get_vertex(&self, _space: &str, _id: &Value) -> Result<Option<Vertex>, StorageError> {
        Ok(None)
    }

    fn scan_vertices(&self, _space: &str) -> Result<Vec<Vertex>, StorageError> {
        Ok(Vec::new())
    }

    fn scan_vertices_by_tag(&self, _space: &str, _tag: &str) -> Result<Vec<Vertex>, StorageError> {
        Ok(Vec::new())
    }

    fn scan_vertices_by_prop(
        &self,
        _space: &str,
        _tag: &str,
        _prop: &str,
        _value: &Value,
    ) -> Result<Vec<Vertex>, StorageError> {
        Ok(Vec::new())
    }

    fn get_edge(
        &self,
        _space: &str,
        _src: &Value,
        _dst: &Value,
        _edge_type: &str,
    ) -> Result<Option<Edge>, StorageError> {
        Ok(None)
    }

    fn get_node_edges(
        &self,
        _space: &str,
        _node_id: &Value,
        _direction: EdgeDirection,
    ) -> Result<Vec<Edge>, StorageError> {
        Ok(Vec::new())
    }

    fn get_node_edges_filtered(
        &self,
        _space: &str,
        _node_id: &Value,
        _direction: EdgeDirection,
        _filter: Option<Box<dyn Fn(&Edge) -> bool + Send + Sync>>,
    ) -> Result<Vec<Edge>, StorageError> {
        Ok(Vec::new())
    }

    fn scan_edges_by_type(&self, _space: &str, _edge_type: &str) -> Result<Vec<Edge>, StorageError> {
        Ok(Vec::new())
    }

    fn scan_all_edges(&self, _space: &str) -> Result<Vec<Edge>, StorageError> {
        Ok(Vec::new())
    }

    fn insert_vertex(&mut self, _space: &str, _vertex: Vertex) -> Result<Value, StorageError> {
        Ok(Value::Null(NullType::NaN))
    }

    fn update_vertex(&mut self, _space: &str, _vertex: Vertex) -> Result<(), StorageError> {
        Ok(())
    }

    fn delete_vertex(&mut self, _space: &str, _id: &Value) -> Result<(), StorageError> {
        Ok(())
    }

    fn batch_insert_vertices(
        &mut self,
        _space: &str,
        _vertices: Vec<Vertex>,
    ) -> Result<Vec<Value>, StorageError> {
        Ok(Vec::new())
    }

    fn insert_edge(&mut self, _space: &str, _edge: Edge) -> Result<(), StorageError> {
        Ok(())
    }

    fn delete_edge(
        &mut self,
        _space: &str,
        _src: &Value,
        _dst: &Value,
        _edge_type: &str,
    ) -> Result<(), StorageError> {
        Ok(())
    }

    fn batch_insert_edges(&mut self, _space: &str, _edges: Vec<Edge>) -> Result<(), StorageError> {
        Ok(())
    }

    fn begin_transaction(&mut self, _space: &str) -> Result<TransactionId, StorageError> {
        Ok(TransactionId::new(1))
    }

    fn commit_transaction(&mut self, _space: &str, _tx_id: TransactionId) -> Result<(), StorageError> {
        Ok(())
    }

    fn rollback_transaction(&mut self, _space: &str, _tx_id: TransactionId) -> Result<(), StorageError> {
        Ok(())
    }

    fn create_space(&mut self, _space: &SpaceInfo) -> Result<bool, StorageError> {
        Ok(true)
    }

    fn drop_space(&mut self, _space: &str) -> Result<bool, StorageError> {
        Ok(true)
    }

    fn get_space(&self, _space: &str) -> Result<Option<SpaceInfo>, StorageError> {
        Ok(None)
    }

    fn list_spaces(&self) -> Result<Vec<SpaceInfo>, StorageError> {
        Ok(Vec::new())
    }

    fn create_tag(&mut self, _space: &str, _info: &TagInfo) -> Result<bool, StorageError> {
        Ok(true)
    }

    fn alter_tag(
        &mut self,
        _space: &str,
        _tag: &str,
        _additions: Vec<PropertyDef>,
        _deletions: Vec<String>,
    ) -> Result<bool, StorageError> {
        Ok(true)
    }

    fn get_tag(&self, _space: &str, _tag: &str) -> Result<Option<TagInfo>, StorageError> {
        Ok(None)
    }

    fn drop_tag(&mut self, _space: &str, _tag: &str) -> Result<bool, StorageError> {
        Ok(true)
    }

    fn list_tags(&self, _space: &str) -> Result<Vec<TagInfo>, StorageError> {
        Ok(Vec::new())
    }

    fn create_edge_type(
        &mut self,
        _space: &str,
        _info: &EdgeTypeSchema,
    ) -> Result<bool, StorageError> {
        Ok(true)
    }

    fn alter_edge_type(
        &mut self,
        _space: &str,
        _edge_type: &str,
        _additions: Vec<PropertyDef>,
        _deletions: Vec<String>,
    ) -> Result<bool, StorageError> {
        Ok(true)
    }

    fn get_edge_type(
        &self,
        _space: &str,
        _edge_type: &str,
    ) -> Result<Option<EdgeTypeSchema>, StorageError> {
        Ok(None)
    }

    fn drop_edge_type(&mut self, _space: &str, _edge_type: &str) -> Result<bool, StorageError> {
        Ok(true)
    }

    fn list_edge_types(&self, _space: &str) -> Result<Vec<EdgeTypeSchema>, StorageError> {
        Ok(Vec::new())
    }

    fn create_tag_index(&mut self, _space: &str, _info: &IndexInfo) -> Result<bool, StorageError> {
        Ok(true)
    }

    fn drop_tag_index(&mut self, _space: &str, _index: &str) -> Result<bool, StorageError> {
        Ok(true)
    }

    fn get_tag_index(
        &self,
        _space: &str,
        _index: &str,
    ) -> Result<Option<IndexInfo>, StorageError> {
        Ok(None)
    }

    fn list_tag_indexes(&self, _space: &str) -> Result<Vec<IndexInfo>, StorageError> {
        Ok(Vec::new())
    }

    fn rebuild_tag_index(&mut self, _space: &str, _index: &str) -> Result<bool, StorageError> {
        Ok(true)
    }

    fn create_edge_index(&mut self, _space: &str, _info: &IndexInfo) -> Result<bool, StorageError> {
        Ok(true)
    }

    fn drop_edge_index(&mut self, _space: &str, _index: &str) -> Result<bool, StorageError> {
        Ok(true)
    }

    fn get_edge_index(
        &self,
        _space: &str,
        _index: &str,
    ) -> Result<Option<IndexInfo>, StorageError> {
        Ok(None)
    }

    fn list_edge_indexes(&self, _space: &str) -> Result<Vec<IndexInfo>, StorageError> {
        Ok(Vec::new())
    }

    fn rebuild_edge_index(&mut self, _space: &str, _index: &str) -> Result<bool, StorageError> {
        Ok(true)
    }

    fn lookup_index(
        &self,
        _space: &str,
        _index: &str,
        _value: &Value,
    ) -> Result<Vec<Value>, StorageError> {
        Ok(Vec::new())
    }

    fn insert_vertex_data(
        &mut self,
        _space: &str,
        _info: &InsertVertexInfo,
    ) -> Result<bool, StorageError> {
        Ok(true)
    }

    fn insert_edge_data(&mut self, _space: &str, _info: &InsertEdgeInfo) -> Result<bool, StorageError> {
        Ok(true)
    }

    fn delete_vertex_data(&mut self, _space: &str, _vertex_id: &str) -> Result<bool, StorageError> {
        Ok(true)
    }

    fn delete_edge_data(
        &mut self,
        _space: &str,
        _src: &str,
        _dst: &str,
        _rank: i64,
    ) -> Result<bool, StorageError> {
        Ok(true)
    }

    fn update_data(&mut self, _space: &str, _info: &UpdateInfo) -> Result<bool, StorageError> {
        Ok(true)
    }

    fn change_password(&mut self, _info: &PasswordInfo) -> Result<bool, StorageError> {
        Ok(true)
    }

    fn get_vertex_with_schema(
        &self,
        _space: &str,
        _tag: &str,
        _id: &Value,
    ) -> Result<Option<(Schema, Vec<u8>)>, StorageError> {
        Ok(None)
    }

    fn get_edge_with_schema(
        &self,
        _space: &str,
        _edge_type: &str,
        _src: &Value,
        _dst: &Value,
    ) -> Result<Option<(Schema, Vec<u8>)>, StorageError> {
        Ok(None)
    }

    fn scan_vertices_with_schema(
        &self,
        _space: &str,
        _tag: &str,
    ) -> Result<Vec<(Schema, Vec<u8>)>, StorageError> {
        Ok(Vec::new())
    }

    fn scan_edges_with_schema(
        &self,
        _space: &str,
        _edge_type: &str,
    ) -> Result<Vec<(Schema, Vec<u8>)>, StorageError> {
        Ok(Vec::new())
    }

    fn load_from_disk(&mut self) -> Result<(), StorageError> {
        Ok(())
    }

    fn save_to_disk(&self) -> Result<(), StorageError> {
        Ok(())
    }
}
