//! 存储客户端实现 - 基于持久化存储的存储操作

use super::super::{
    ExecResponse, NewEdge, NewVertex, StorageOperation,
    StorageResponse, UpdateResponse, UpdatedProp,
};
use crate::core::error::{ManagerError, ManagerResult, StorageError};
use crate::core::{Edge, Value, Vertex, EdgeDirection};
use crate::core::types::{
    SpaceInfo, TagInfo, EdgeTypeSchema, IndexInfo,
    PropertyDef, InsertVertexInfo, InsertEdgeInfo, UpdateInfo,
    PasswordInfo,
};
use crate::expression::storage::Schema;
use crate::storage::MemoryStorage;
use crate::storage::storage_client::StorageClient;
use crate::storage::transaction::TransactionId;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub struct MemoryStorageClient {
    storage: Arc<RwLock<MemoryStorage>>,
    connected: bool,
}

impl MemoryStorageClient {
    pub fn new() -> Self {
        let storage = MemoryStorage::new()
            .expect("Failed to create MemoryStorage");
        Self {
            storage: Arc::new(RwLock::new(storage)),
            connected: true,
        }
    }

    pub fn with_path(_storage_path: PathBuf) -> Self {
        let storage = MemoryStorage::new()
            .expect("Failed to create MemoryStorage");
        Self {
            storage: Arc::new(RwLock::new(storage)),
            connected: true,
        }
    }

    pub fn disconnect(&mut self) {
        self.connected = false;
    }

    pub fn reconnect(&mut self) {
        self.connected = true;
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }

    pub fn load_from_disk(&mut self) -> ManagerResult<()> {
        let mut storage = self
            .storage
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        storage
            .load_from_disk()
            .map_err(|e| ManagerError::StorageError(e.to_string()))
    }

    pub fn save_to_disk(&self) -> ManagerResult<()> {
        let storage = self
            .storage
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        storage
            .save_to_disk()
            .map_err(|e| ManagerError::StorageError(e.to_string()))
    }

    fn parse_value_from_key(key: &str) -> Result<Value, String> {
        if key.starts_with('"') && key.ends_with('"') {
            Ok(Value::String(key[1..key.len()-1].to_string()))
        } else if key == "true" {
            Ok(Value::Bool(true))
        } else if key == "false" {
            Ok(Value::Bool(false))
        } else if let Ok(i) = key.parse::<i64>() {
            Ok(Value::Int(i))
        } else if let Ok(f) = key.parse::<f64>() {
            Ok(Value::Float(f))
        } else {
            Ok(Value::String(key.to_string()))
        }
    }
}

impl Default for MemoryStorageClient {
    fn default() -> Self {
        Self::new()
    }
}

impl StorageClient for MemoryStorageClient {
    fn batch_insert_vertices(
        &mut self,
        space: &str,
        vertices: Vec<Vertex>,
    ) -> Result<Vec<Value>, StorageError> {
        if !self.connected {
            return Err(StorageError::DbError(
                "存储客户端未连接".to_string(),
            ));
        }

        let mut storage = self
            .storage
            .write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let mut results = Vec::new();
        for vertex in vertices {
            let id = storage
                .insert_vertex(space, vertex)
                .map_err(|e| StorageError::DbError(e.to_string()))?;
            results.push(id);
        }

        Ok(results)
    }

    fn get_vertex(&self, space: &str, id: &Value) -> Result<Option<Vertex>, StorageError> {
        if !self.connected {
            return Err(StorageError::DbError(
                "存储客户端未连接".to_string(),
            ));
        }

        let storage = self
            .storage
            .read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .get_vertex(space, id)
            .map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn delete_vertex(&mut self, space: &str, vid: &Value) -> Result<(), StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let mut storage = self
            .storage
            .write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .delete_vertex(space, vid)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        Ok(())
    }

    fn update_vertex(&mut self, space: &str, vertex: Vertex) -> Result<(), StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let mut storage = self
            .storage
            .write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .update_vertex(space, vertex)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        Ok(())
    }

    fn insert_edge(&mut self, space: &str, edge: Edge) -> Result<(), StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let mut storage = self
            .storage
            .write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .insert_edge(space, edge)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        Ok(())
    }

    fn batch_insert_edges(&mut self, space: &str, edges: Vec<Edge>) -> Result<(), StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let mut storage = self
            .storage
            .write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        for edge in edges {
            storage
                .insert_edge(space, edge)
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }

        Ok(())
    }

    fn get_edge(&self, space: &str, src: &Value, dst: &Value, edge_type: &str) -> Result<Option<Edge>, StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let storage = self
            .storage
            .read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .get_edge(space, src, dst, edge_type)
            .map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn delete_edge(&mut self, space: &str, src: &Value, dst: &Value, edge_type: &str) -> Result<(), StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let mut storage = self
            .storage
            .write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .delete_edge(space, src, dst, edge_type)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        Ok(())
    }

    fn scan_vertices(&self, space: &str) -> Result<Vec<Vertex>, StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let storage = self
            .storage
            .read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .scan_vertices(space)
            .map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn scan_vertices_by_tag(&self, space: &str, tag: &str) -> Result<Vec<Vertex>, StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let storage = self
            .storage
            .read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .scan_vertices_by_tag(space, tag)
            .map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn scan_vertices_by_prop(
        &self,
        space: &str,
        tag: &str,
        prop: &str,
        value: &Value,
    ) -> Result<Vec<Vertex>, StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let storage = self
            .storage
            .read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .scan_vertices_by_prop(space, tag, prop, value)
            .map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn get_node_edges(
        &self,
        space: &str,
        node_id: &Value,
        direction: EdgeDirection,
    ) -> Result<Vec<Edge>, StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let storage = self
            .storage
            .read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .get_node_edges(space, node_id, direction)
            .map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn get_node_edges_filtered(
        &self,
        space: &str,
        node_id: &Value,
        direction: EdgeDirection,
        filter: Option<Box<dyn Fn(&Edge) -> bool + Send + Sync + 'static>>,
    ) -> Result<Vec<Edge>, StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let storage = self
            .storage
            .read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .get_node_edges_filtered(space, node_id, direction, filter)
            .map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn scan_edges_by_type(&self, space: &str, edge_type: &str) -> Result<Vec<Edge>, StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let storage = self
            .storage
            .read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .scan_edges_by_type(space, edge_type)
            .map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn scan_all_edges(&self, space: &str) -> Result<Vec<Edge>, StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let storage = self
            .storage
            .read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .scan_all_edges(space)
            .map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn insert_vertex(&mut self, space: &str, vertex: Vertex) -> Result<Value, StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let mut storage = self
            .storage
            .write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .insert_vertex(space, vertex)
            .map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn begin_transaction(&mut self, space: &str) -> Result<TransactionId, StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let mut storage = self
            .storage
            .write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .begin_transaction(space)
            .map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn commit_transaction(&mut self, space: &str, tx_id: TransactionId) -> Result<(), StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let mut storage = self
            .storage
            .write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .commit_transaction(space, tx_id)
            .map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn rollback_transaction(&mut self, space: &str, tx_id: TransactionId) -> Result<(), StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let mut storage = self
            .storage
            .write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .rollback_transaction(space, tx_id)
            .map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn create_space(&mut self, space: &SpaceInfo) -> Result<bool, StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let mut storage = self
            .storage
            .write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .create_space(space)
            .map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn drop_space(&mut self, space: &str) -> Result<bool, StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let mut storage = self
            .storage
            .write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .drop_space(space)
            .map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn get_space(&self, space: &str) -> Result<Option<SpaceInfo>, StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let storage = self
            .storage
            .read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .get_space(space)
            .map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn list_spaces(&self) -> Result<Vec<SpaceInfo>, StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let storage = self
            .storage
            .read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .list_spaces()
            .map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn create_tag(&mut self, space: &str, info: &TagInfo) -> Result<bool, StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let mut storage = self
            .storage
            .write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .create_tag(space, info)
            .map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn alter_tag(&mut self, space: &str, tag: &str, additions: Vec<PropertyDef>, deletions: Vec<String>) -> Result<bool, StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let mut storage = self
            .storage
            .write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .alter_tag(space, tag, additions, deletions)
            .map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn get_tag(&self, space: &str, tag: &str) -> Result<Option<TagInfo>, StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let storage = self
            .storage
            .read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .get_tag(space, tag)
            .map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn drop_tag(&mut self, space: &str, tag: &str) -> Result<bool, StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let mut storage = self
            .storage
            .write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .drop_tag(space, tag)
            .map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn list_tags(&self, space: &str) -> Result<Vec<TagInfo>, StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let storage = self
            .storage
            .read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .list_tags(space)
            .map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn create_edge_type(&mut self, space: &str, info: &EdgeTypeSchema) -> Result<bool, StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let mut storage = self
            .storage
            .write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .create_edge_type(space, info)
            .map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn alter_edge_type(&mut self, space: &str, edge_type: &str, additions: Vec<PropertyDef>, deletions: Vec<String>) -> Result<bool, StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let mut storage = self
            .storage
            .write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .alter_edge_type(space, edge_type, additions, deletions)
            .map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn get_edge_type(&self, space: &str, edge_type: &str) -> Result<Option<EdgeTypeSchema>, StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let storage = self
            .storage
            .read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .get_edge_type(space, edge_type)
            .map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn drop_edge_type(&mut self, space: &str, edge_type: &str) -> Result<bool, StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let mut storage = self
            .storage
            .write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .drop_edge_type(space, edge_type)
            .map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn list_edge_types(&self, space: &str) -> Result<Vec<EdgeTypeSchema>, StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let storage = self
            .storage
            .read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .list_edge_types(space)
            .map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn create_tag_index(&mut self, space: &str, info: &IndexInfo) -> Result<bool, StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let mut storage = self
            .storage
            .write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .create_tag_index(space, info)
            .map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn drop_tag_index(&mut self, space: &str, index: &str) -> Result<bool, StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let mut storage = self
            .storage
            .write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .drop_tag_index(space, index)
            .map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn get_tag_index(&self, space: &str, index: &str) -> Result<Option<IndexInfo>, StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let storage = self
            .storage
            .read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .get_tag_index(space, index)
            .map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn list_tag_indexes(&self, space: &str) -> Result<Vec<IndexInfo>, StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let storage = self
            .storage
            .read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .list_tag_indexes(space)
            .map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn rebuild_tag_index(&mut self, space: &str, index: &str) -> Result<bool, StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let mut storage = self
            .storage
            .write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .rebuild_tag_index(space, index)
            .map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn create_edge_index(&mut self, space: &str, info: &IndexInfo) -> Result<bool, StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let mut storage = self
            .storage
            .write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .create_edge_index(space, info)
            .map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn drop_edge_index(&mut self, space: &str, index: &str) -> Result<bool, StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let mut storage = self
            .storage
            .write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .drop_edge_index(space, index)
            .map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn get_edge_index(&self, space: &str, index: &str) -> Result<Option<IndexInfo>, StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let storage = self
            .storage
            .read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .get_edge_index(space, index)
            .map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn list_edge_indexes(&self, space: &str) -> Result<Vec<IndexInfo>, StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let storage = self
            .storage
            .read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .list_edge_indexes(space)
            .map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn rebuild_edge_index(&mut self, space: &str, index: &str) -> Result<bool, StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let mut storage = self
            .storage
            .write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .rebuild_edge_index(space, index)
            .map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn insert_vertex_data(&mut self, space: &str, info: &InsertVertexInfo) -> Result<bool, StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let mut storage = self
            .storage
            .write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .insert_vertex_data(space, info)
            .map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn insert_edge_data(&mut self, space: &str, info: &InsertEdgeInfo) -> Result<bool, StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let mut storage = self
            .storage
            .write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .insert_edge_data(space, info)
            .map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn delete_vertex_data(&mut self, space: &str, vertex_id: &str) -> Result<bool, StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let mut storage = self
            .storage
            .write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .delete_vertex_data(space, vertex_id)
            .map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn delete_edge_data(&mut self, space: &str, src: &str, dst: &str, rank: i64) -> Result<bool, StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let mut storage = self
            .storage
            .write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .delete_edge_data(space, src, dst, rank)
            .map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn update_data(&mut self, space: &str, info: &UpdateInfo) -> Result<bool, StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let mut storage = self
            .storage
            .write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .update_data(space, info)
            .map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn change_password(&mut self, info: &PasswordInfo) -> Result<bool, StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let mut storage = self
            .storage
            .write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .change_password(info)
            .map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn lookup_index(&self, space: &str, index: &str, value: &Value) -> Result<Vec<Value>, StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let storage = self
            .storage
            .read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .lookup_index(space, index, value)
            .map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn get_vertex_with_schema(&self, space: &str, tag: &str, id: &Value) -> Result<Option<(Schema, Vec<u8>)>, StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let storage = self
            .storage
            .read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .get_vertex_with_schema(space, tag, id)
            .map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn get_edge_with_schema(&self, space: &str, edge_type: &str, src: &Value, dst: &Value) -> Result<Option<(Schema, Vec<u8>)>, StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let storage = self
            .storage
            .read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .get_edge_with_schema(space, edge_type, src, dst)
            .map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn scan_vertices_with_schema(&self, space: &str, tag: &str) -> Result<Vec<(Schema, Vec<u8>)>, StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let storage = self
            .storage
            .read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .scan_vertices_with_schema(space, tag)
            .map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn scan_edges_with_schema(&self, space: &str, edge_type: &str) -> Result<Vec<(Schema, Vec<u8>)>, StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let storage = self
            .storage
            .read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .scan_edges_with_schema(space, edge_type)
            .map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn load_from_disk(&mut self) -> Result<(), StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let mut storage = self
            .storage
            .write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .load_from_disk()
            .map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn save_to_disk(&self) -> Result<(), StorageError> {
        if !self.connected {
            return Err(StorageError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let storage = self
            .storage
            .read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        storage
            .save_to_disk()
            .map_err(|e| StorageError::DbError(e.to_string()))
    }
}
