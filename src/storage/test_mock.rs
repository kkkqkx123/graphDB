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
use crate::storage::{StorageEngine, StorageError};

/// 测试用Mock存储引擎
#[cfg(test)]
#[derive(Debug)]
pub struct MockStorage;

#[cfg(test)]
impl StorageEngine for MockStorage {
    fn insert_node(&mut self, _vertex: Vertex) -> Result<Value, StorageError> {
        Ok(Value::Null(NullType::NaN))
    }

    fn get_node(&self, _id: &Value) -> Result<Option<Vertex>, StorageError> {
        Ok(None)
    }

    fn update_node(&mut self, _vertex: Vertex) -> Result<(), StorageError> {
        Ok(())
    }

    fn delete_node(&mut self, _id: &Value) -> Result<(), StorageError> {
        Ok(())
    }

    fn insert_edge(&mut self, _edge: Edge) -> Result<(), StorageError> {
        Ok(())
    }

    fn get_edge(
        &self,
        _src: &Value,
        _dst: &Value,
        _edge_type: &str,
    ) -> Result<Option<Edge>, StorageError> {
        Ok(None)
    }

    fn get_node_edges(
        &self,
        _node_id: &Value,
        _direction: EdgeDirection,
    ) -> Result<Vec<Edge>, StorageError> {
        Ok(Vec::new())
    }

    fn get_node_edges_filtered(
        &self,
        _node_id: &Value,
        _direction: EdgeDirection,
        _filter: Option<Box<dyn Fn(&Edge) -> bool + Send + Sync>>,
    ) -> Result<Vec<Edge>, StorageError> {
        Ok(Vec::new())
    }

    fn delete_edge(
        &mut self,
        _src: &Value,
        _dst: &Value,
        _edge_type: &str,
    ) -> Result<(), StorageError> {
        Ok(())
    }

    fn begin_transaction(&mut self) -> Result<u64, StorageError> {
        Ok(1)
    }

    fn commit_transaction(&mut self, _tx_id: u64) -> Result<(), StorageError> {
        Ok(())
    }

    fn rollback_transaction(&mut self, _tx_id: u64) -> Result<(), StorageError> {
        Ok(())
    }

    fn scan_all_vertices(&self) -> Result<Vec<Vertex>, StorageError> {
        Ok(Vec::new())
    }

    fn scan_vertices_by_tag(&self, _tag: &str) -> Result<Vec<Vertex>, StorageError> {
        Ok(Vec::new())
    }

    fn scan_vertices_by_prop(&self, _tag: &str, _prop: &str, _value: &Value) -> Result<Vec<Vertex>, StorageError> {
        Ok(Vec::new())
    }

    fn scan_edges_by_type(&self, _edge_type: &str) -> Result<Vec<Edge>, StorageError> {
        Ok(Vec::new())
    }

    fn scan_all_edges(&self) -> Result<Vec<Edge>, StorageError> {
        Ok(Vec::new())
    }

    fn batch_insert_nodes(&mut self, _vertices: Vec<Vertex>) -> Result<Vec<Value>, StorageError> {
        Ok(Vec::new())
    }

    fn batch_insert_edges(&mut self, _edges: Vec<Edge>) -> Result<(), StorageError> {
        Ok(())
    }

    fn get_input(&self, _input_var: &str) -> Result<Option<Vec<Value>>, StorageError> {
        Ok(None)
    }
}

#[cfg(test)]
impl Default for MockStorage {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
impl MockStorage {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(test)]
pub mod helpers {
    use super::*;

    /// 创建默认的Mock存储引擎
    pub fn create_mock_storage() -> MockStorage {
        MockStorage::new()
    }

    /// 创建带有预定义数据的Mock存储引擎
    pub fn create_mock_storage_with_data() -> MockStorage {
        MockStorage::new()
    }
}
