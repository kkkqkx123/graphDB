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
use crate::core::error::{DBError, StorageError};
#[cfg(test)]
use crate::storage::StorageClient;

/// 测试用Mock存储引擎
#[cfg(test)]
#[derive(Debug)]
pub struct MockStorage;

#[cfg(test)]
impl StorageClient for MockStorage {
    fn insert_node(&mut self, _vertex: Vertex) -> Result<Value, DBError> {
        Ok(Value::Null(NullType::NaN))
    }

    fn get_node(&self, _id: &Value) -> Result<Option<Vertex>, DBError> {
        Ok(None)
    }

    fn update_node(&mut self, _vertex: Vertex) -> Result<(), DBError> {
        Ok(())
    }

    fn delete_node(&mut self, _id: &Value) -> Result<(), DBError> {
        Ok(())
    }

    fn insert_edge(&mut self, _edge: Edge) -> Result<(), DBError> {
        Ok(())
    }

    fn get_edge(
        &self,
        _src: &Value,
        _dst: &Value,
        _edge_type: &str,
    ) -> Result<Option<Edge>, DBError> {
        Ok(None)
    }

    fn get_node_edges(
        &self,
        _node_id: &Value,
        _direction: EdgeDirection,
    ) -> Result<Vec<Edge>, DBError> {
        Ok(Vec::new())
    }

    fn get_node_edges_filtered(
        &self,
        _node_id: &Value,
        _direction: EdgeDirection,
        _filter: Option<Box<dyn Fn(&Edge) -> bool + Send + Sync>>,
    ) -> Result<Vec<Edge>, DBError> {
        Ok(Vec::new())
    }

    fn delete_edge(
        &mut self,
        _src: &Value,
        _dst: &Value,
        _edge_type: &str,
    ) -> Result<(), DBError> {
        Ok(())
    }

    fn begin_transaction(&mut self) -> Result<u64, DBError> {
        Ok(1)
    }

    fn commit_transaction(&mut self, _tx_id: u64) -> Result<(), DBError> {
        Ok(())
    }

    fn rollback_transaction(&mut self, _tx_id: u64) -> Result<(), DBError> {
        Ok(())
    }

    fn scan_all_vertices(&self) -> Result<Vec<Vertex>, DBError> {
        Ok(Vec::new())
    }

    fn scan_vertices_by_tag(&self, _tag: &str) -> Result<Vec<Vertex>, DBError> {
        Ok(Vec::new())
    }

    fn scan_vertices_by_prop(&self, _tag: &str, _prop: &str, _value: &Value) -> Result<Vec<Vertex>, DBError> {
        Ok(Vec::new())
    }

    fn scan_edges_by_type(&self, _edge_type: &str) -> Result<Vec<Edge>, DBError> {
        Ok(Vec::new())
    }

    fn scan_all_edges(&self) -> Result<Vec<Edge>, DBError> {
        Ok(Vec::new())
    }

    fn batch_insert_nodes(&mut self, _vertices: Vec<Vertex>) -> Result<Vec<Value>, DBError> {
        Ok(Vec::new())
    }

    fn batch_insert_edges(&mut self, _edges: Vec<Edge>) -> Result<(), DBError> {
        Ok(())
    }

    fn get_input(&self, _input_var: &str) -> Result<Option<Vec<Value>>, DBError> {
        Ok(None)
    }

    // ========== 空间管理 ==========
    fn create_space(&mut self, _space: &crate::core::types::SpaceInfo) -> Result<bool, DBError> {
        Ok(true)
    }

    fn drop_space(&mut self, _space_name: &str) -> Result<bool, DBError> {
        Ok(true)
    }

    fn get_space(&self, _space_name: &str) -> Result<Option<crate::core::types::SpaceInfo>, DBError> {
        Ok(None)
    }

    fn list_spaces(&self) -> Result<Vec<crate::core::types::SpaceInfo>, DBError> {
        Ok(Vec::new())
    }

    // ========== 标签管理 ==========
    fn create_tag(&mut self, _info: &crate::core::types::TagInfo) -> Result<bool, DBError> {
        Ok(true)
    }

    fn alter_tag(&mut self, _space_name: &str, _tag_name: &str, _additions: Vec<crate::core::types::PropertyDef>, _deletions: Vec<String>) -> Result<bool, DBError> {
        Ok(true)
    }

    fn get_tag(&self, _space_name: &str, _tag_name: &str) -> Result<Option<crate::core::types::TagInfo>, DBError> {
        Ok(None)
    }

    fn drop_tag(&mut self, _space_name: &str, _tag_name: &str) -> Result<bool, DBError> {
        Ok(true)
    }

    fn list_tags(&self, _space_name: &str) -> Result<Vec<crate::core::types::TagInfo>, DBError> {
        Ok(Vec::new())
    }

    // ========== 边类型管理 ==========
    fn create_edge_type(&mut self, _info: &crate::core::types::EdgeTypeSchema) -> Result<bool, DBError> {
        Ok(true)
    }

    fn alter_edge_type(&mut self, _space_name: &str, _edge_type_name: &str, _additions: Vec<crate::core::types::PropertyDef>, _deletions: Vec<String>) -> Result<bool, DBError> {
        Ok(true)
    }

    fn get_edge_type(&self, _space_name: &str, _edge_type_name: &str) -> Result<Option<crate::core::types::EdgeTypeSchema>, DBError> {
        Ok(None)
    }

    fn drop_edge_type(&mut self, _space_name: &str, _edge_type_name: &str) -> Result<bool, DBError> {
        Ok(true)
    }

    fn list_edge_types(&self, _space_name: &str) -> Result<Vec<crate::core::types::EdgeTypeSchema>, DBError> {
        Ok(Vec::new())
    }

    // ========== 索引管理 ==========
    fn create_index(&mut self, _space_name: &str, _index_info: &crate::core::types::IndexInfo) -> Result<bool, DBError> {
        Ok(true)
    }

    fn drop_index(&mut self, _space_name: &str, _index_name: &str, _is_edge: bool) -> Result<bool, DBError> {
        Ok(true)
    }

    fn rebuild_index(&mut self, _space_name: &str, _index_name: &str) -> Result<bool, DBError> {
        Ok(true)
    }

    fn get_index(&self, _space_name: &str, _index_name: &str) -> Result<Option<crate::core::types::IndexInfo>, DBError> {
        Ok(None)
    }

    fn list_indexes(&self, _space_name: &str) -> Result<Vec<crate::core::types::IndexInfo>, DBError> {
        Ok(Vec::new())
    }

    fn lookup_index(&self, _space_name: &str, _index_name: &str, _value: &Value) -> Result<Vec<Value>, DBError> {
        Ok(Vec::new())
    }
}
