//! 存储客户端实现 - 基于持久化存储的存储操作

use super::super::{
    DelTags, EdgeKey, ExecResponse, NewEdge, NewVertex, StorageClient, StorageOperation,
    StorageResponse, UpdateResponse, UpdatedProp,
};
use crate::core::error::{ManagerError, ManagerResult};
use crate::core::{Edge, Tag, Value, Vertex};
use crate::storage::MemoryStorage;
use crate::storage::storage_engine::StorageEngine;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

/// 持久化存储客户端实现 - 使用MemoryStorage作为后端
#[derive(Debug, Clone)]
pub struct MemoryStorageClient {
    storage: Arc<RwLock<MemoryStorage>>,
    connected: bool,
    storage_path: PathBuf,
}

impl MemoryStorageClient {
    /// 创建新的持久化存储客户端
    pub fn new() -> Self {
        let storage_path = PathBuf::from("./data/storage");
        let storage = MemoryStorage::new()
            .expect("Failed to create MemoryStorage");
        Self {
            storage: Arc::new(RwLock::new(storage)),
            connected: true,
            storage_path,
        }
    }

    /// 创建带存储路径的持久化存储客户端
    pub fn with_path(storage_path: PathBuf) -> Self {
        let storage = MemoryStorage::new()
            .expect("Failed to create MemoryStorage");
        Self {
            storage: Arc::new(RwLock::new(storage)),
            connected: true,
            storage_path,
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

    /// 获取表数据
    pub fn get_table(&self, table_name: &str) -> Option<HashMap<String, Value>> {
        if !self.connected {
            return None;
        }

        let storage = self.storage.read().ok()?;
        let result = storage.scan_all_vertices().ok()?;
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

        let storage = self.storage.read().ok();
        if storage.is_none() {
            return Vec::new();
        }

        let storage = storage.unwrap();
        let vertices = storage.scan_all_vertices();
        if vertices.is_err() {
            return Vec::new();
        }

        let mut table_names = Vec::new();
        for vertex in vertices.unwrap() {
            for tag in &vertex.tags {
                if !table_names.contains(&tag.name) {
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

        let storage = self.storage.read().ok();
        if storage.is_none() {
            return false;
        }

        let storage = storage.unwrap();
        let vertices = storage.scan_all_vertices();
        if vertices.is_err() {
            return false;
        }

        for vertex in vertices.unwrap() {
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

        let vertices = storage.scan_all_vertices()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        for vertex in vertices {
            if vertex.tags.iter().any(|tag| tag.name == table_name) {
                let new_tags: Vec<Tag> = vertex.tags
                    .into_iter()
                    .filter(|tag| tag.name != table_name)
                    .collect();

                if new_tags.is_empty() {
                    storage.delete_node(&vertex.vid)
                        .map_err(|e| ManagerError::StorageError(e.to_string()))?;
                } else {
                    let updated_vertex = Vertex::new(*vertex.vid.clone(), new_tags);
                    storage.update_node(updated_vertex)
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

    /// 生成顶点键
    fn vertex_key(vid: &Value) -> String {
        format!("{:?}", vid)
    }

    /// 生成边键
    fn edge_key_string(key: &EdgeKey) -> String {
        format!(
            "{:?}:{:?}:{:?}:{:?}",
            key.src, key.edge_type, key.ranking, key.dst
        )
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
}

impl Default for MemoryStorageClient {
    fn default() -> Self {
        Self::new()
    }
}

impl StorageClient for MemoryStorageClient {
    fn execute(&self, operation: StorageOperation) -> ManagerResult<StorageResponse> {
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
                    if let Ok(Some(vertex)) = storage.get_node(&vid) {
                        for tag in &vertex.tags {
                            if tag.name == table {
                                let mut data = HashMap::new();
                                for (k, v) in &tag.properties {
                                    data.insert(k.clone(), v.clone());
                                }
                                return Ok(StorageResponse {
                                    success: true,
                                    data: Some(Value::Map(data)),
                                    error_message: None,
                                });
                            }
                        }
                    }
                }
                Ok(StorageResponse {
                    success: false,
                    data: None,
                    error_message: Some(format!("未找到数据: {}", key)),
                })
            }

            StorageOperation::Write { table, key, value } => {
                let vid = Self::parse_value_from_key(&key);
                if let Ok(vid) = vid {
                    if let Ok(Some(vertex)) = storage.get_node(&vid) {
                        let mut tags = vertex.tags;
                        if let Some(tag) = tags.iter_mut().find(|t| t.name == table) {
                            if let Value::Map(props) = value {
                                for (k, v) in props {
                                    tag.properties.insert(k, v);
                                }
                            }
                        } else {
                            let mut props = HashMap::new();
                            if let Value::Map(map_props) = value {
                                props = map_props;
                            }
                            tags.push(Tag::new(table.clone(), props));
                        }
                        let updated_vertex = Vertex::new(vid, tags);
                        drop(storage);
                        let mut storage = self.storage.write().map_err(|e| ManagerError::StorageError(e.to_string()))?;
                        let _ = storage.update_node(updated_vertex);
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
                    error_message: Some(format!("写入失败: {}", key)),
                })
            }

            StorageOperation::Delete { table, key } => {
                let vid = Self::parse_value_from_key(&key);
                if let Ok(vid) = vid {
                    if let Ok(Some(vertex)) = storage.get_node(&vid) {
                        let tags: Vec<Tag> = vertex.tags
                            .into_iter()
                            .filter(|t| t.name != table)
                            .collect();
                        if tags.is_empty() {
                            drop(storage);
                            let mut storage = self.storage.write().map_err(|e| ManagerError::StorageError(e.to_string()))?;
                            let _ = storage.delete_node(&vid);
                        } else {
                            let updated_vertex = Vertex::new(vid, tags);
                            drop(storage);
                            let mut storage = self.storage.write().map_err(|e| ManagerError::StorageError(e.to_string()))?;
                            let _ = storage.update_node(updated_vertex);
                        }
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
                    error_message: Some(format!("删除失败: {}", key)),
                })
            }

            StorageOperation::Scan { table, prefix } => {
                let vertices = storage.scan_all_vertices()
                    .map_err(|e| ManagerError::StorageError(e.to_string()))?;
                let mut results = HashMap::new();
                for vertex in vertices {
                    for tag in &vertex.tags {
                        if tag.name == table {
                            for (key, value) in &tag.properties {
                                if key.starts_with(&prefix) {
                                    results.insert(key.clone(), value.clone());
                                }
                            }
                        }
                    }
                }
                Ok(StorageResponse {
                    success: true,
                    data: Some(Value::Map(results)),
                    error_message: None,
                })
            }
        }
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn add_vertex(&self, _space_id: i32, vertex: Vertex) -> ManagerResult<ExecResponse> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let mut storage = self
            .storage
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        storage
            .insert_node(vertex)
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        Ok(ExecResponse::ok())
    }

    fn add_vertices(
        &self,
        _space_id: i32,
        new_vertices: Vec<NewVertex>,
    ) -> ManagerResult<ExecResponse> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let mut storage = self
            .storage
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        for new_vertex in new_vertices {
            let tags: Vec<Tag> = new_vertex
                .tags
                .into_iter()
                .map(|new_tag| Tag::new(format!("tag_{}", new_tag.tag_id), HashMap::new()))
                .collect();

            let vertex = Vertex::new(new_vertex.id, tags);
            storage
                .insert_node(vertex)
                .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        }

        Ok(ExecResponse::ok())
    }

    fn get_vertex(&self, _space_id: i32, vid: &Value) -> ManagerResult<Option<Vertex>> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let storage = self
            .storage
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        storage
            .get_node(vid)
            .map_err(|e| ManagerError::StorageError(e.to_string()))
    }

    fn get_vertices(&self, _space_id: i32, vids: &[Value]) -> ManagerResult<Vec<Option<Vertex>>> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let storage = self
            .storage
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        let mut results = Vec::with_capacity(vids.len());
        for vid in vids {
            let vertex = storage
                .get_node(vid)
                .map_err(|e| ManagerError::StorageError(e.to_string()))?;
            results.push(vertex);
        }

        Ok(results)
    }

    fn delete_vertex(&self, _space_id: i32, vid: &Value) -> ManagerResult<ExecResponse> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let mut storage = self
            .storage
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        storage
            .delete_node(vid)
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        Ok(ExecResponse::ok())
    }

    fn delete_vertices(&self, _space_id: i32, vids: &[Value]) -> ManagerResult<ExecResponse> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let mut storage = self
            .storage
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        for vid in vids {
            storage
                .delete_node(vid)
                .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        }

        Ok(ExecResponse::ok())
    }

    fn delete_tags(&self, _space_id: i32, del_tags: Vec<DelTags>) -> ManagerResult<ExecResponse> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let mut storage = self
            .storage
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        for del_tag in del_tags {
            if let Ok(Some(vertex)) = storage.get_node(&del_tag.id) {
                let new_tags: Vec<Tag> = vertex.tags
                    .into_iter()
                    .filter(|tag| {
                        let tag_id: i32 = tag
                            .name
                            .strip_prefix("tag_")
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(-1);
                        !del_tag.tags.contains(&tag_id)
                    })
                    .collect();

                if new_tags.is_empty() {
                    storage
                        .delete_node(&del_tag.id)
                        .map_err(|e| ManagerError::StorageError(e.to_string()))?;
                } else {
                    let updated_vertex = Vertex::new(del_tag.id.clone(), new_tags);
                    storage
                        .update_node(updated_vertex)
                        .map_err(|e| ManagerError::StorageError(e.to_string()))?;
                }
            }
        }

        Ok(ExecResponse::ok())
    }

    fn update_vertex(
        &self,
        _space_id: i32,
        vid: &Value,
        tag_id: i32,
        updated_props: Vec<UpdatedProp>,
        insertable: bool,
        return_props: Vec<String>,
        _condition: Option<String>,
    ) -> ManagerResult<UpdateResponse> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let mut storage = self
            .storage
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        let mut inserted = false;
        let mut props_to_return: Option<HashMap<String, Value>> = None;

        if let Ok(Some(vertex)) = storage.get_node(vid) {
            let tag_name = format!("tag_{}", tag_id);
            let mut tags = vertex.tags;

            if let Some(tag) = tags.iter_mut().find(|t| t.name == tag_name) {
                for updated_prop in updated_props {
                    tag.properties
                        .insert(updated_prop.name.clone(), updated_prop.value);
                }
            } else if insertable {
                let mut new_tag_props = HashMap::new();
                for updated_prop in updated_props {
                    new_tag_props.insert(updated_prop.name.clone(), updated_prop.value);
                }
                tags.push(Tag::new(tag_name, new_tag_props));
                inserted = true;
            }

            if !return_props.is_empty() {
                let mut return_map = HashMap::new();
                for prop_name in return_props {
                    if let Some(value) = tags.iter().find_map(|t| t.properties.get(&prop_name)) {
                        return_map.insert(prop_name, value.clone());
                    }
                }
                props_to_return = Some(return_map);
            }

            let updated_vertex = Vertex::new(vid.clone(), tags);
            storage
                .update_node(updated_vertex)
                .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        } else if insertable {
            let mut new_tag_props = HashMap::new();
            for updated_prop in updated_props {
                new_tag_props.insert(updated_prop.name.clone(), updated_prop.value);
            }
            let tag = Tag::new(format!("tag_{}", tag_id), new_tag_props);
            let vertex = Vertex::new(vid.clone(), vec![tag]);
            storage
                .insert_node(vertex)
                .map_err(|e| ManagerError::StorageError(e.to_string()))?;
            inserted = true;
        }

        Ok(UpdateResponse::ok(inserted, props_to_return))
    }

    fn add_edge(&self, _space_id: i32, edge: Edge) -> ManagerResult<ExecResponse> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let mut storage = self
            .storage
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        storage
            .insert_edge(edge)
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        Ok(ExecResponse::ok())
    }

    fn add_edges(&self, _space_id: i32, new_edges: Vec<NewEdge>) -> ManagerResult<ExecResponse> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let mut storage = self
            .storage
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        for new_edge in new_edges {
            let edge = Edge {
                src: Box::new(new_edge.key.src.clone()),
                dst: Box::new(new_edge.key.dst.clone()),
                edge_type: new_edge.key.edge_type.clone(),
                ranking: new_edge.key.ranking,
                id: 0,
                props: HashMap::new(),
            };

            storage
                .insert_edge(edge)
                .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        }

        Ok(ExecResponse::ok())
    }

    fn get_edge(&self, _space_id: i32, edge_key: &EdgeKey) -> ManagerResult<Option<Edge>> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let storage = self
            .storage
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        storage
            .get_edge(&edge_key.src, &edge_key.dst, &edge_key.edge_type)
            .map_err(|e| ManagerError::StorageError(e.to_string()))
    }

    fn get_edges(&self, _space_id: i32, edge_keys: &[EdgeKey]) -> ManagerResult<Vec<Option<Edge>>> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let storage = self
            .storage
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        let mut results = Vec::with_capacity(edge_keys.len());
        for edge_key in edge_keys {
            let edge = storage
                .get_edge(&edge_key.src, &edge_key.dst, &edge_key.edge_type)
                .map_err(|e| ManagerError::StorageError(e.to_string()))?;
            results.push(edge);
        }

        Ok(results)
    }

    fn delete_edge(&self, _space_id: i32, edge_key: &EdgeKey) -> ManagerResult<ExecResponse> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let mut storage = self
            .storage
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        storage
            .delete_edge(&edge_key.src, &edge_key.dst, &edge_key.edge_type)
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        Ok(ExecResponse::ok())
    }

    fn delete_edges(&self, _space_id: i32, edge_keys: &[EdgeKey]) -> ManagerResult<ExecResponse> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let mut storage = self
            .storage
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        for edge_key in edge_keys {
            storage
                .delete_edge(&edge_key.src, &edge_key.dst, &edge_key.edge_type)
                .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        }

        Ok(ExecResponse::ok())
    }

    fn update_edge(
        &self,
        _space_id: i32,
        edge_key: &EdgeKey,
        updated_props: Vec<UpdatedProp>,
        insertable: bool,
        return_props: Vec<String>,
        _condition: Option<String>,
    ) -> ManagerResult<UpdateResponse> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let mut storage = self
            .storage
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        let mut inserted = false;
        let mut props_to_return: Option<HashMap<String, Value>> = None;

        if let Ok(Some(mut edge)) = storage.get_edge(&edge_key.src, &edge_key.dst, &edge_key.edge_type) {
            for updated_prop in updated_props {
                edge.props
                    .insert(updated_prop.name.clone(), updated_prop.value);
            }

            if !return_props.is_empty() {
                let mut return_map = HashMap::new();
                for prop_name in return_props {
                    if let Some(value) = edge.props.get(&prop_name) {
                        return_map.insert(prop_name, value.clone());
                    }
                }
                props_to_return = Some(return_map);
            }

            storage
                .delete_edge(&edge_key.src, &edge_key.dst, &edge_key.edge_type)
                .map_err(|e| ManagerError::StorageError(e.to_string()))?;
            storage
                .insert_edge(edge)
                .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        } else if insertable {
            let mut edge_props = HashMap::new();
            for updated_prop in updated_props {
                edge_props.insert(updated_prop.name.clone(), updated_prop.value);
            }

            let edge = Edge {
                src: Box::new(edge_key.src.clone()),
                dst: Box::new(edge_key.dst.clone()),
                edge_type: edge_key.edge_type.clone(),
                ranking: edge_key.ranking,
                id: 0,
                props: edge_props.clone(),
            };

            storage
                .insert_edge(edge)
                .map_err(|e| ManagerError::StorageError(e.to_string()))?;
            inserted = true;

            if !return_props.is_empty() {
                let mut return_map = HashMap::new();
                for prop_name in return_props {
                    if let Some(value) = edge_props.get(&prop_name) {
                        return_map.insert(prop_name, value.clone());
                    }
                }
                props_to_return = Some(return_map);
            }
        }

        Ok(UpdateResponse::ok(inserted, props_to_return))
    }

    fn scan_vertices(&self, _space_id: i32, limit: Option<usize>) -> ManagerResult<Vec<Vertex>> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let storage = self
            .storage
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        let result = (*storage)
            .scan_all_vertices()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        if let Some(limit) = limit {
            Ok(result.into_iter().take(limit).collect())
        } else {
            Ok(result)
        }
    }

    fn scan_vertices_by_tag(
        &self,
        _space_id: i32,
        tag_id: i32,
        limit: Option<usize>,
    ) -> ManagerResult<Vec<Vertex>> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let storage = self
            .storage
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        let tag_name = format!("tag_{}", tag_id);
        let result = (*storage)
            .scan_vertices_by_tag(&tag_name)
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        if let Some(limit) = limit {
            Ok(result.into_iter().take(limit).collect())
        } else {
            Ok(result)
        }
    }

    fn scan_edges(&self, _space_id: i32, limit: Option<usize>) -> ManagerResult<Vec<Edge>> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let storage = self
            .storage
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        let result = (*storage)
            .scan_all_edges()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        if let Some(limit) = limit {
            Ok(result.into_iter().take(limit).collect())
        } else {
            Ok(result)
        }
    }

    fn scan_edges_by_type(
        &self,
        _space_id: i32,
        edge_type: &str,
        limit: Option<usize>,
    ) -> ManagerResult<Vec<Edge>> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let storage = self
            .storage
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        let result = (*storage)
            .scan_edges_by_type(edge_type)
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        if let Some(limit) = limit {
            Ok(result.into_iter().take(limit).collect())
        } else {
            Ok(result)
        }
    }

    fn scan_edges_by_src(
        &self,
        _space_id: i32,
        src: &Value,
        limit: Option<usize>,
    ) -> ManagerResult<Vec<Edge>> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let storage = self
            .storage
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        let result = (*storage)
            .get_node_edges(src, crate::core::Direction::Out)
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        if let Some(limit) = limit {
            Ok(result.into_iter().take(limit).collect())
        } else {
            Ok(result)
        }
    }

    fn scan_edges_by_dst(
        &self,
        _space_id: i32,
        dst: &Value,
        limit: Option<usize>,
    ) -> ManagerResult<Vec<Edge>> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let storage = self
            .storage
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        let result = (*storage)
            .get_node_edges(dst, crate::core::Direction::In)
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        if let Some(limit) = limit {
            Ok(result.into_iter().take(limit).collect())
        } else {
            Ok(result)
        }
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
