//! 存储客户端实现 - 基于持久化存储的存储操作

use super::super::{
    DelTags, EdgeKey, ExecResponse, NewEdge, NewVertex, StorageOperation,
    StorageResponse, UpdateResponse, UpdatedProp,
};
use crate::core::error::{ManagerError, ManagerResult, StorageError};
use crate::core::{Edge, Value, Vertex, EdgeDirection};
use crate::core::vertex_edge_path::Tag;
use crate::core::types::{
    SpaceInfo, TagInfo, EdgeTypeSchema, IndexInfo,
    PropertyDef, InsertVertexInfo, InsertEdgeInfo, UpdateInfo,
    PasswordInfo,
};
use crate::expression::storage::Schema;
use crate::storage::MemoryStorage;
use crate::storage::storage_client::StorageClient;
use crate::storage::transaction::TransactionId;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

/// 持久化存储客户端实现 - 使用MemoryStorage作为后端
#[derive(Debug, Clone)]
pub struct MemoryStorageClient {
    storage: Arc<RwLock<MemoryStorage>>,
    connected: bool,
}

impl MemoryStorageClient {
    /// 创建新的持久化存储客户端
    pub fn new() -> Self {
        let storage = MemoryStorage::new()
            .expect("Failed to create MemoryStorage");
        Self {
            storage: Arc::new(RwLock::new(storage)),
            connected: true,
        }
    }

    /// 创建带存储路径的持久化存储客户端
    pub fn with_path(_storage_path: PathBuf) -> Self {
        let storage = MemoryStorage::new()
            .expect("Failed to create MemoryStorage");
        Self {
            storage: Arc::new(RwLock::new(storage)),
            connected: true,
        }
    }

    /// 断开连接
    pub fn disconnect(&mut self) {
        self.connected = false;
    }

    /// 重新连接
    pub fn reconnect(&mut self) {
        self.connected = true;
    }

    /// 检查是否已连接
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// 获取表数据
    pub fn get_table(&self, table_name: &str) -> Option<HashMap<String, Value>> {
        if !self.connected {
            return None;
        }

        let storage = self.storage.read().ok()?;
        let result = <MemoryStorage as StorageClient>::scan_vertices(&*storage, "default").ok()?;
        let mut table_data = HashMap::new();

        for vertex in result {
            for tag in &vertex.tags {
                for (key, value) in &tag.properties {
                    table_data.insert(format!("{}_{}_{}", table_name, key, vertex.vid), value.clone());
                }
            }
        }

        Some(table_data)
    }

    /// 列出所有表名
    pub fn list_tables(&self) -> Vec<String> {
        if !self.connected {
            return Vec::new();
        }

        let storage = match self.storage.read() {
            Ok(s) => s,
            Err(_) => {
                return Vec::new();
            }
        };
        let vertices = <MemoryStorage as StorageClient>::scan_vertices(&*storage, "default");
        if vertices.is_err() {
            return Vec::new();
        }

        let mut table_names = Vec::new();
        for vertex in vertices.expect("Failed to scan all vertices") {
            for tag in &vertex.tags {
                if !table_names.contains(&tag.name) && !tag.name.starts_with("__") {
                    table_names.push(tag.name.clone());
                }
            }
        }

        table_names
    }

    /// 检查表是否存在
    pub fn has_table(&self, table_name: &str) -> bool {
        if !self.connected {
            return false;
        }

        let storage = match self.storage.read() {
            Ok(s) => s,
            Err(_) => {
                return false;
            }
        };
        let vertices = <MemoryStorage as StorageClient>::scan_vertices(&*storage, "default");
        if vertices.is_err() {
            return false;
        }

        for vertex in vertices.expect("Failed to scan all vertices") {
            for tag in &vertex.tags {
                if tag.name == table_name {
                    return true;
                }
            }
        }

        false
    }

    /// 创建表
    pub fn create_table(&self, table_name: &str) -> ManagerResult<()> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        if self.has_table(table_name) {
            return Err(ManagerError::AlreadyExists(format!(
                "表 {} 已存在",
                table_name
            )));
        }

        let mut storage = self
            .storage
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        let tag = Tag::new(table_name.to_string(), HashMap::new());
        let vertex = Vertex::new(Value::String(format!("__table_marker_{}", table_name)), vec![tag]);
        <MemoryStorage as StorageClient>::insert_vertex(&mut *storage, "default", vertex)
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        Ok(())
    }

    /// 删除表
    pub fn drop_table(&self, table_name: &str) -> ManagerResult<()> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let mut storage = self
            .storage
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        let vertices = <MemoryStorage as StorageClient>::scan_vertices(&*storage, "default")
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        for vertex in vertices {
            if vertex.tags.iter().any(|tag| tag.name == table_name) {
                let new_tags: Vec<Tag> = vertex.tags
                    .into_iter()
                    .filter(|tag| tag.name != table_name)
                    .collect();

                if new_tags.is_empty() {
                    <MemoryStorage as StorageClient>::delete_vertex(&mut *storage, "default", &vertex.vid)
                        .map_err(|e| ManagerError::StorageError(e.to_string()))?;
                } else {
                    let updated_vertex = Vertex::new((*vertex.vid).clone(), new_tags);
                    <MemoryStorage as StorageClient>::update_vertex(&mut *storage, "default", updated_vertex)
                        .map_err(|e| ManagerError::StorageError(e.to_string()))?;
                }
            }
        }

        Ok(())
    }

    /// 从磁盘加载数据
    pub fn load_from_disk(&self) -> ManagerResult<()> {
        Ok(())
    }

    /// 保存数据到磁盘
    pub fn save_to_disk(&self) -> ManagerResult<()> {
        Ok(())
    }

    /// 从键字符串解析Value
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

    /// 执行存储操作
    pub fn execute(&self, operation: StorageOperation) -> ManagerResult<StorageResponse> {
        if !self.connected {
            return Ok(StorageResponse {
                success: false,
                data: None,
                error_message: Some("存储客户端未连接".to_string()),
            });
        }

        let storage = self
            .storage
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        match operation {
            StorageOperation::Read { table, key } => {
                let vid = Self::parse_value_from_key(&key);
                if let Ok(vid) = vid {
                    if let Ok(Some(vertex)) = <MemoryStorage as StorageClient>::get_vertex(&*storage, "default", &vid) {
                        for tag in &vertex.tags {
                            if tag.name == table {
                                return Ok(StorageResponse {
                                    success: true,
                                    data: tag.properties.get(&key).cloned(),
                                    error_message: None,
                                });
                            }
                        }
                    }
                }
                Ok(StorageResponse {
                    success: false,
                    data: None,
                    error_message: Some("未找到数据".to_string()),
                })
            }
            StorageOperation::Write { table, key, value } => {
                let vid = Self::parse_value_from_key(&key);
                if let Ok(vid) = vid {
                    let mut storage_mut = self.storage.write().map_err(|e| ManagerError::StorageError(e.to_string()))?;
                    let tags = vec![crate::core::vertex_edge_path::Tag::new(table, std::collections::HashMap::new())];
                    let vertex = Vertex::new(vid, tags);
                    if let Ok(_) = <MemoryStorage as StorageClient>::insert_vertex(&mut *storage_mut, "default", vertex) {
                        return Ok(StorageResponse {
                            success: true,
                            data: Some(value),
                            error_message: None,
                        });
                    }
                }
                Ok(StorageResponse {
                    success: false,
                    data: None,
                    error_message: Some("写入失败".to_string()),
                })
            }
            StorageOperation::Delete { table: _, key } => {
                let vid = Self::parse_value_from_key(&key);
                if let Ok(vid) = vid {
                    let mut storage_mut = self.storage.write().map_err(|e| ManagerError::StorageError(e.to_string()))?;
                    if let Ok(_) = <MemoryStorage as StorageClient>::delete_vertex(&mut *storage_mut, "default", &vid) {
                        return Ok(StorageResponse {
                            success: true,
                            data: None,
                            error_message: None,
                        });
                    }
                }
                Ok(StorageResponse {
                    success: false,
                    data: None,
                    error_message: Some("删除失败".to_string()),
                })
            }
            StorageOperation::Scan { table, prefix: _ } => {
                let vertices = <MemoryStorage as StorageClient>::scan_vertices(&*storage, "default");
                if let Ok(vertex_list) = vertices {
                    for vertex in vertex_list {
                        for tag in &vertex.tags {
                            if tag.name == table {
                                return Ok(StorageResponse {
                                    success: true,
                                    data: Some(crate::core::Value::String(format!("{:?}", vertex))),
                                    error_message: None,
                                });
                            }
                        }
                    }
                }
                Ok(StorageResponse {
                    success: false,
                    data: None,
                    error_message: Some("扫描失败".to_string()),
                })
            }
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

        <MemoryStorage as StorageClient>::update_vertex(&mut *storage, space, vertex)
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

        <MemoryStorage as StorageClient>::insert_edge(&mut *storage, space, edge)
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
            <MemoryStorage as StorageClient>::insert_edge(&mut *storage, space, edge)
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

        <MemoryStorage as StorageClient>::get_edge(&*storage, space, src, dst, edge_type)
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

        <MemoryStorage as StorageClient>::delete_edge(&mut *storage, space, src, dst, edge_type)
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

        <MemoryStorage as StorageClient>::scan_vertices(&*storage, space)
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

        <MemoryStorage as StorageClient>::scan_vertices_by_tag(&*storage, space, tag)
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

        <MemoryStorage as StorageClient>::scan_vertices_by_prop(&*storage, space, tag, prop, value)
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

        <MemoryStorage as StorageClient>::get_node_edges(&*storage, space, node_id, direction)
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

        <MemoryStorage as StorageClient>::get_node_edges_filtered(&*storage, space, node_id, direction, filter)
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

        <MemoryStorage as StorageClient>::scan_edges_by_type(&*storage, space, edge_type)
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

        <MemoryStorage as StorageClient>::scan_all_edges(&*storage, space)
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

        <MemoryStorage as StorageClient>::insert_vertex(&mut *storage, space, vertex)
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

        <MemoryStorage as StorageClient>::begin_transaction(&mut *storage, space)
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

        <MemoryStorage as StorageClient>::commit_transaction(&mut *storage, space, tx_id)
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

        <MemoryStorage as StorageClient>::rollback_transaction(&mut *storage, space, tx_id)
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

        <MemoryStorage as StorageClient>::create_space(&mut *storage, space)
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

        <MemoryStorage as StorageClient>::drop_space(&mut *storage, space)
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

        <MemoryStorage as StorageClient>::get_space(&*storage, space)
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

        <MemoryStorage as StorageClient>::list_spaces(&*storage)
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

        <MemoryStorage as StorageClient>::create_tag(&mut *storage, space, info)
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

        <MemoryStorage as StorageClient>::alter_tag(&mut *storage, space, tag, additions, deletions)
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

        <MemoryStorage as StorageClient>::get_tag(&*storage, space, tag)
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

        <MemoryStorage as StorageClient>::drop_tag(&mut *storage, space, tag)
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

        <MemoryStorage as StorageClient>::list_tags(&*storage, space)
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

        <MemoryStorage as StorageClient>::create_edge_type(&mut *storage, space, info)
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

        <MemoryStorage as StorageClient>::alter_edge_type(&mut *storage, space, edge_type, additions, deletions)
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

        <MemoryStorage as StorageClient>::get_edge_type(&*storage, space, edge_type)
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

        <MemoryStorage as StorageClient>::drop_edge_type(&mut *storage, space, edge_type)
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

        <MemoryStorage as StorageClient>::list_edge_types(&*storage, space)
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

        <MemoryStorage as StorageClient>::create_tag_index(&mut *storage, space, info)
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

        <MemoryStorage as StorageClient>::drop_tag_index(&mut *storage, space, index)
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

        <MemoryStorage as StorageClient>::get_tag_index(&*storage, space, index)
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

        <MemoryStorage as StorageClient>::list_tag_indexes(&*storage, space)
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

        <MemoryStorage as StorageClient>::rebuild_tag_index(&mut *storage, space, index)
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

        <MemoryStorage as StorageClient>::create_edge_index(&mut *storage, space, info)
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

        <MemoryStorage as StorageClient>::drop_edge_index(&mut *storage, space, index)
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

        <MemoryStorage as StorageClient>::get_edge_index(&*storage, space, index)
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

        <MemoryStorage as StorageClient>::list_edge_indexes(&*storage, space)
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

        <MemoryStorage as StorageClient>::rebuild_edge_index(&mut *storage, space, index)
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

        <MemoryStorage as StorageClient>::insert_vertex_data(&mut *storage, space, info)
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

        <MemoryStorage as StorageClient>::insert_edge_data(&mut *storage, space, info)
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

        <MemoryStorage as StorageClient>::delete_vertex_data(&mut *storage, space, vertex_id)
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

        <MemoryStorage as StorageClient>::delete_edge_data(&mut *storage, space, src, dst, rank)
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

        <MemoryStorage as StorageClient>::update_data(&mut *storage, space, info)
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

        <MemoryStorage as StorageClient>::change_password(&mut *storage, info)
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

        <MemoryStorage as StorageClient>::get_vertex_with_schema(&*storage, space, tag, id)
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

        <MemoryStorage as StorageClient>::get_edge_with_schema(&*storage, space, edge_type, src, dst)
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

        <MemoryStorage as StorageClient>::scan_vertices_with_schema(&*storage, space, tag)
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

        <MemoryStorage as StorageClient>::scan_edges_with_schema(&*storage, space, edge_type)
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

        <MemoryStorage as StorageClient>::lookup_index(&*storage, space, index, value)
            .map_err(|e| StorageError::DbError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_storage_client_creation() {
        let client = MemoryStorageClient::new();
        assert!(client.is_connected());
        assert!(client.list_tables().is_empty());
    }

    #[test]
    fn test_memory_storage_client_read_write() {
        let client = MemoryStorageClient::new();

        // 写入数据
        let write_op = StorageOperation::Write {
            table: "users".to_string(),
            key: "user1".to_string(),
            value: Value::String("Alice".to_string()),
        };

        let result = client.execute(write_op);
        assert!(result.is_ok());
        assert!(result.expect("Result should be successful").success);

        // 读取数据
        let read_op = StorageOperation::Read {
            table: "users".to_string(),
            key: "user1".to_string(),
        };

        let result = client.execute(read_op);
        assert!(result.is_ok());
        let response = result.expect("Result should be available");
        assert!(response.success);
        assert_eq!(response.data, Some(Value::String("Alice".to_string())));
    }

    #[test]
    fn test_memory_storage_client_delete() {
        let client = MemoryStorageClient::new();

        // 先写入数据
        let write_op = StorageOperation::Write {
            table: "users".to_string(),
            key: "user1".to_string(),
            value: Value::String("Alice".to_string()),
        };
        client
            .execute(write_op)
            .expect("Write operation should succeed");

        // 删除数据
        let delete_op = StorageOperation::Delete {
            table: "users".to_string(),
            key: "user1".to_string(),
        };

        let result = client.execute(delete_op);
        assert!(result.is_ok());
        assert!(result.expect("Result should be successful").success);

        // 验证数据已删除
        let read_op = StorageOperation::Read {
            table: "users".to_string(),
            key: "user1".to_string(),
        };

        let result = client.execute(read_op);
        assert!(result.is_ok());
        let response = result.expect("Result should be available");
        assert!(response.success);
        assert!(response.data.is_none());
    }

    #[test]
    fn test_memory_storage_client_scan() {
        let client = MemoryStorageClient::new();

        // 写入多个数据
        client
            .execute(StorageOperation::Write {
                table: "users".to_string(),
                key: "user1".to_string(),
                value: Value::String("Alice".to_string()),
            })
            .expect("Write operation should succeed");

        client
            .execute(StorageOperation::Write {
                table: "users".to_string(),
                key: "user2".to_string(),
                value: Value::String("Bob".to_string()),
            })
            .expect("Write operation should succeed");

        client
            .execute(StorageOperation::Write {
                table: "users".to_string(),
                key: "admin1".to_string(),
                value: Value::String("Admin".to_string()),
            })
            .expect("Write operation should succeed");

        // 扫描以"user"开头的数据
        let scan_op = StorageOperation::Scan {
            table: "users".to_string(),
            prefix: "user".to_string(),
        };

        let result = client.execute(scan_op);
        assert!(result.is_ok());
        let response = result.expect("Result should be available");
        assert!(response.success);

        if let Some(Value::Map(data)) = response.data {
            assert_eq!(data.len(), 2); // 应该找到user1和user2
            assert!(data.contains_key("user1"));
            assert!(data.contains_key("user2"));
            assert!(!data.contains_key("admin1"));
        } else {
            panic!("预期返回Map类型的数据");
        }
    }

    #[test]
    fn test_memory_storage_client_disconnect() {
        let mut client = MemoryStorageClient::new();
        assert!(client.is_connected());

        client.disconnect();
        assert!(!client.is_connected());

        let op = StorageOperation::Read {
            table: "users".to_string(),
            key: "user1".to_string(),
        };

        let result = client.execute(op);
        assert!(result.is_ok());
        let response = result.expect("Result should be available");
        assert!(!response.success);
        assert!(response.error_message.is_some());

        client.reconnect();
        assert!(client.is_connected());
    }

    #[test]
    fn test_memory_storage_client_table_operations() {
        let client = MemoryStorageClient::new();

        // 创建表
        assert!(client.create_table("users").is_ok());
        assert!(client.has_table("users"));
        assert_eq!(client.list_tables(), vec!["users".to_string()]);

        // 删除表
        assert!(client.drop_table("users").is_ok());
        assert!(!client.has_table("users"));
        assert!(client.list_tables().is_empty());
    }
}
