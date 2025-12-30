//! 存储客户端实现 - 内存中的存储操作

use super::super::{
    DelTags, EdgeKey, ExecResponse, NewEdge, NewVertex, StorageClient, StorageOperation,
    StorageResponse, UpdateResponse, UpdatedProp,
};
use crate::core::error::{ManagerError, ManagerResult};
use crate::core::{Edge, Tag, Value, Vertex};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

/// 内存中的存储客户端实现
#[derive(Debug, Clone)]
pub struct MemoryStorageClient {
    tables: Arc<RwLock<HashMap<String, HashMap<String, Value>>>>,
    vertices: Arc<RwLock<HashMap<i32, HashMap<Value, Vertex>>>>,
    edges: Arc<RwLock<HashMap<i32, Vec<Edge>>>>,
    edge_index: Arc<RwLock<HashMap<i32, HashMap<EdgeKey, usize>>>>,
    connected: bool,
    storage_path: PathBuf,
}

impl MemoryStorageClient {
    /// 创建新的内存存储客户端
    pub fn new() -> Self {
        Self {
            tables: Arc::new(RwLock::new(HashMap::new())),
            vertices: Arc::new(RwLock::new(HashMap::new())),
            edges: Arc::new(RwLock::new(HashMap::new())),
            edge_index: Arc::new(RwLock::new(HashMap::new())),
            connected: true,
            storage_path: PathBuf::from("./data/storage"),
        }
    }

    /// 创建带存储路径的内存存储客户端
    pub fn with_path(storage_path: PathBuf) -> Self {
        Self {
            tables: Arc::new(RwLock::new(HashMap::new())),
            vertices: Arc::new(RwLock::new(HashMap::new())),
            edges: Arc::new(RwLock::new(HashMap::new())),
            edge_index: Arc::new(RwLock::new(HashMap::new())),
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
        let tables = self.tables.read().ok()?;
        tables.get(table_name).cloned()
    }

    /// 列出所有表名
    pub fn list_tables(&self) -> Vec<String> {
        match self.tables.read() {
            Ok(tables) => tables.keys().cloned().collect(),
            Err(_) => Vec::new(),
        }
    }

    /// 检查表是否存在
    pub fn has_table(&self, table_name: &str) -> bool {
        match self.tables.read() {
            Ok(tables) => tables.contains_key(table_name),
            Err(_) => false,
        }
    }

    /// 创建表
    pub fn create_table(&self, table_name: &str) -> ManagerResult<()> {
        let mut tables = self
            .tables
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        if tables.contains_key(table_name) {
            return Err(ManagerError::AlreadyExists(format!(
                "表 {} 已存在",
                table_name
            )));
        }
        tables.insert(table_name.to_string(), HashMap::new());
        Ok(())
    }

    /// 删除表
    pub fn drop_table(&self, table_name: &str) -> ManagerResult<()> {
        let mut tables = self
            .tables
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        tables.remove(table_name);
        Ok(())
    }

    /// 从磁盘加载数据
    pub fn load_from_disk(&self) -> ManagerResult<()> {
        use std::fs;

        if !self.storage_path.exists() {
            return Ok(());
        }

        let vertices_file = self.storage_path.join("vertices.json");
        if vertices_file.exists() {
            let content = fs::read_to_string(&vertices_file)
                .map_err(|e| ManagerError::StorageError(e.to_string()))?;
            let loaded_vertices: HashMap<i32, HashMap<String, Vertex>> =
                serde_json::from_str(&content)
                    .map_err(|e| ManagerError::StorageError(format!("反序列化顶点失败: {}", e)))?;

            let mut vertices = self
                .vertices
                .write()
                .map_err(|e| ManagerError::StorageError(e.to_string()))?;
            for (space_id, space_vertices) in loaded_vertices {
                let mut vertex_map: HashMap<Value, Vertex> = HashMap::new();
                for (vid_str, vertex) in space_vertices {
                    let vid: Value = serde_json::from_str(&vid_str).map_err(|e| {
                        ManagerError::StorageError(format!("反序列化顶点ID失败: {}", e))
                    })?;
                    vertex_map.insert(vid, vertex);
                }
                vertices.insert(space_id, vertex_map);
            }
        }

        let edges_file = self.storage_path.join("edges.json");
        if edges_file.exists() {
            let content = fs::read_to_string(&edges_file)
                .map_err(|e| ManagerError::StorageError(e.to_string()))?;
            let loaded_edges: HashMap<i32, Vec<Edge>> = serde_json::from_str(&content)
                .map_err(|e| ManagerError::StorageError(format!("反序列化边失败: {}", e)))?;

            let mut edges = self
                .edges
                .write()
                .map_err(|e| ManagerError::StorageError(e.to_string()))?;
            *edges = loaded_edges;

            let mut edge_index = self
                .edge_index
                .write()
                .map_err(|e| ManagerError::StorageError(e.to_string()))?;
            edge_index.clear();
            for (space_id, space_edges) in edges.iter() {
                let mut space_index = HashMap::new();
                for (idx, edge) in space_edges.iter().enumerate() {
                    let key = EdgeKey {
                        src: edge.src.as_ref().clone(),
                        edge_type: edge.edge_type.clone(),
                        ranking: edge.ranking,
                        dst: edge.dst.as_ref().clone(),
                    };
                    space_index.insert(key, idx);
                }
                edge_index.insert(*space_id, space_index);
            }
        }

        Ok(())
    }

    /// 保存数据到磁盘
    pub fn save_to_disk(&self) -> ManagerResult<()> {
        use std::fs;

        if !self.storage_path.exists() {
            fs::create_dir_all(&self.storage_path)
                .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        }

        let vertices = self
            .vertices
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let mut vertices_to_save: HashMap<i32, HashMap<String, Vertex>> = HashMap::new();
        for (space_id, space_vertices) in vertices.iter() {
            let mut vertex_map: HashMap<String, Vertex> = HashMap::new();
            for (vid, vertex) in space_vertices.iter() {
                let vid_str = serde_json::to_string(vid)
                    .map_err(|e| ManagerError::StorageError(e.to_string()))?;
                vertex_map.insert(vid_str, vertex.clone());
            }
            vertices_to_save.insert(*space_id, vertex_map);
        }
        let vertices_content = serde_json::to_string_pretty(&vertices_to_save)
            .map_err(|e| ManagerError::StorageError(format!("序列化顶点失败: {}", e)))?;
        let vertices_file = self.storage_path.join("vertices.json");
        fs::write(&vertices_file, vertices_content)
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        let edges = self
            .edges
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let edges_content = serde_json::to_string_pretty(&*edges)
            .map_err(|e| ManagerError::StorageError(format!("序列化边失败: {}", e)))?;
        let edges_file = self.storage_path.join("edges.json");
        fs::write(&edges_file, edges_content)
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

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

        let mut tables = self
            .tables
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        match operation {
            StorageOperation::Read { table, key } => {
                let table_data = tables
                    .get_mut(&table)
                    .ok_or_else(|| ManagerError::NotFound(format!("表 {} 不存在", table)))?;
                let data = table_data.get(&key).cloned();

                Ok(StorageResponse {
                    success: true,
                    data,
                    error_message: None,
                })
            }

            StorageOperation::Write { table, key, value } => {
                let table_data = tables.entry(table).or_insert_with(HashMap::new);
                table_data.insert(key, value);

                Ok(StorageResponse {
                    success: true,
                    data: None,
                    error_message: None,
                })
            }

            StorageOperation::Delete { table, key } => {
                if let Some(table_data) = tables.get_mut(&table) {
                    table_data.remove(&key);
                    Ok(StorageResponse {
                        success: true,
                        data: None,
                        error_message: None,
                    })
                } else {
                    Ok(StorageResponse {
                        success: false,
                        data: None,
                        error_message: Some(format!("表 {} 不存在", table)),
                    })
                }
            }

            StorageOperation::Scan { table, prefix } => {
                let table_data = tables
                    .get(&table)
                    .ok_or_else(|| ManagerError::NotFound(format!("表 {} 不存在", table)))?;

                let mut results = HashMap::new();
                for (key, value) in table_data {
                    if key.starts_with(&prefix) {
                        results.insert(key.clone(), value.clone());
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

    fn add_vertex(&self, space_id: i32, vertex: Vertex) -> ManagerResult<ExecResponse> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let mut vertices = self
            .vertices
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let space_vertices = vertices.entry(space_id).or_insert_with(HashMap::new);
        let vid = vertex.vid().clone();
        space_vertices.insert(vid, vertex);

        Ok(ExecResponse::ok())
    }

    fn add_vertices(
        &self,
        space_id: i32,
        new_vertices: Vec<NewVertex>,
    ) -> ManagerResult<ExecResponse> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let mut vertices = self
            .vertices
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let space_vertices = vertices.entry(space_id).or_insert_with(HashMap::new);

        for new_vertex in new_vertices {
            let tags: Vec<Tag> = new_vertex
                .tags
                .into_iter()
                .map(|new_tag| Tag::new(format!("tag_{}", new_tag.tag_id), HashMap::new()))
                .collect();

            let vertex = Vertex::new(new_vertex.id, tags);
            space_vertices.insert(vertex.vid().clone(), vertex);
        }

        Ok(ExecResponse::ok())
    }

    fn get_vertex(&self, space_id: i32, vid: &Value) -> ManagerResult<Option<Vertex>> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let vertices = self
            .vertices
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        if let Some(space_vertices) = vertices.get(&space_id) {
            Ok(space_vertices.get(vid).cloned())
        } else {
            Ok(None)
        }
    }

    fn get_vertices(&self, space_id: i32, vids: &[Value]) -> ManagerResult<Vec<Option<Vertex>>> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let vertices = self
            .vertices
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let space_vertices = vertices.get(&space_id);

        let mut results = Vec::with_capacity(vids.len());
        for vid in vids {
            if let Some(space_vertices) = space_vertices {
                results.push(space_vertices.get(vid).cloned());
            } else {
                results.push(None);
            }
        }

        Ok(results)
    }

    fn delete_vertex(&self, space_id: i32, vid: &Value) -> ManagerResult<ExecResponse> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let mut vertices = self
            .vertices
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        if let Some(space_vertices) = vertices.get_mut(&space_id) {
            space_vertices.remove(vid);

            let mut edges = self
                .edges
                .write()
                .map_err(|e| ManagerError::StorageError(e.to_string()))?;
            if let Some(space_edges) = edges.get_mut(&space_id) {
                space_edges.retain(|edge| edge.src.as_ref() != vid && edge.dst.as_ref() != vid);

                let mut edge_index = self
                    .edge_index
                    .write()
                    .map_err(|e| ManagerError::StorageError(e.to_string()))?;
                if let Some(space_index) = edge_index.get_mut(&space_id) {
                    space_index.retain(|key, _| &key.src != vid && &key.dst != vid);
                }
            }

            Ok(ExecResponse::ok())
        } else {
            Ok(ExecResponse::ok())
        }
    }

    fn delete_vertices(&self, space_id: i32, vids: &[Value]) -> ManagerResult<ExecResponse> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let mut vertices = self
            .vertices
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        if let Some(space_vertices) = vertices.get_mut(&space_id) {
            for vid in vids {
                space_vertices.remove(vid);
            }
        }

        let mut edges = self
            .edges
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        if let Some(space_edges) = edges.get_mut(&space_id) {
            space_edges.retain(|edge| {
                !vids.contains(edge.src.as_ref()) && !vids.contains(edge.dst.as_ref())
            });

            let mut edge_index = self
                .edge_index
                .write()
                .map_err(|e| ManagerError::StorageError(e.to_string()))?;
            if let Some(space_index) = edge_index.get_mut(&space_id) {
                space_index.retain(|key, _| !vids.contains(&key.src) && !vids.contains(&key.dst));
            }
        }

        Ok(ExecResponse::ok())
    }

    fn delete_tags(&self, space_id: i32, del_tags: Vec<DelTags>) -> ManagerResult<ExecResponse> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let mut vertices = self
            .vertices
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        if let Some(space_vertices) = vertices.get_mut(&space_id) {
            for del_tag in del_tags {
                if let Some(vertex) = space_vertices.get_mut(&del_tag.id) {
                    vertex.tags.retain(|tag| {
                        let tag_id: i32 = tag
                            .name
                            .strip_prefix("tag_")
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(-1);
                        !del_tag.tags.contains(&tag_id)
                    });
                }
            }
        }

        Ok(ExecResponse::ok())
    }

    fn update_vertex(
        &self,
        space_id: i32,
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

        let mut vertices = self
            .vertices
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let space_vertices = vertices.entry(space_id).or_insert_with(HashMap::new);

        let mut inserted = false;
        let mut props_to_return: Option<HashMap<String, Value>> = None;

        if let Some(vertex) = space_vertices.get_mut(vid) {
            let tag_name = format!("tag_{}", tag_id);
            if let Some(tag) = vertex.tags.iter_mut().find(|t| t.name == tag_name) {
                for updated_prop in updated_props {
                    tag.properties
                        .insert(updated_prop.name.clone(), updated_prop.value);
                }
            } else if insertable {
                let mut new_tag_props = HashMap::new();
                for updated_prop in updated_props {
                    new_tag_props.insert(updated_prop.name.clone(), updated_prop.value);
                }
                vertex.tags.push(Tag::new(tag_name, new_tag_props));
                inserted = true;
            }

            if !return_props.is_empty() {
                let mut return_map = HashMap::new();
                for prop_name in return_props {
                    if let Some(value) = vertex.get_property_any(&prop_name) {
                        return_map.insert(prop_name, value.clone());
                    }
                }
                props_to_return = Some(return_map);
            }
        } else if insertable {
            let mut new_tag_props = HashMap::new();
            for updated_prop in updated_props {
                new_tag_props.insert(updated_prop.name.clone(), updated_prop.value);
            }
            let tag = Tag::new(format!("tag_{}", tag_id), new_tag_props);
            let vertex = Vertex::new(vid.clone(), vec![tag]);
            space_vertices.insert(vid.clone(), vertex);
            inserted = true;
        }

        Ok(UpdateResponse::ok(inserted, props_to_return))
    }

    fn add_edge(&self, space_id: i32, edge: Edge) -> ManagerResult<ExecResponse> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let mut edges = self
            .edges
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let space_edges = edges.entry(space_id).or_insert_with(Vec::new);

        let key = EdgeKey {
            src: edge.src.as_ref().clone(),
            edge_type: edge.edge_type.clone(),
            ranking: edge.ranking,
            dst: edge.dst.as_ref().clone(),
        };

        let mut edge_index = self
            .edge_index
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let space_index = edge_index.entry(space_id).or_insert_with(HashMap::new);

        if let Some(&idx) = space_index.get(&key) {
            space_edges[idx] = edge;
        } else {
            let idx = space_edges.len();
            space_edges.push(edge);
            space_index.insert(key, idx);
        }

        Ok(ExecResponse::ok())
    }

    fn add_edges(&self, space_id: i32, new_edges: Vec<NewEdge>) -> ManagerResult<ExecResponse> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let mut edges = self
            .edges
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let space_edges = edges.entry(space_id).or_insert_with(Vec::new);

        let mut edge_index = self
            .edge_index
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let space_index = edge_index.entry(space_id).or_insert_with(HashMap::new);

        for new_edge in new_edges {
            let idx = space_edges.len();
            let edge = Edge {
                src: Box::new(new_edge.key.src.clone()),
                dst: Box::new(new_edge.key.dst.clone()),
                edge_type: new_edge.key.edge_type.clone(),
                ranking: new_edge.key.ranking,
                id: idx as i64,
                props: HashMap::new(),
            };

            if let Some(&existing_idx) = space_index.get(&new_edge.key) {
                space_edges[existing_idx] = edge;
            } else {
                space_edges.push(edge);
                space_index.insert(new_edge.key, idx);
            }
        }

        Ok(ExecResponse::ok())
    }

    fn get_edge(&self, space_id: i32, edge_key: &EdgeKey) -> ManagerResult<Option<Edge>> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let edge_index = self
            .edge_index
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        if let Some(space_index) = edge_index.get(&space_id) {
            if let Some(&idx) = space_index.get(edge_key) {
                let edges = self
                    .edges
                    .read()
                    .map_err(|e| ManagerError::StorageError(e.to_string()))?;
                if let Some(space_edges) = edges.get(&space_id) {
                    if idx < space_edges.len() {
                        return Ok(Some(space_edges[idx].clone()));
                    }
                }
            }
        }

        Ok(None)
    }

    fn get_edges(&self, space_id: i32, edge_keys: &[EdgeKey]) -> ManagerResult<Vec<Option<Edge>>> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let edge_index = self
            .edge_index
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let space_index = edge_index.get(&space_id);
        let edges = self
            .edges
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let space_edges = edges.get(&space_id);

        let mut results = Vec::with_capacity(edge_keys.len());
        for edge_key in edge_keys {
            if let (Some(space_index), Some(space_edges)) = (space_index, space_edges) {
                if let Some(&idx) = space_index.get(edge_key) {
                    if idx < space_edges.len() {
                        results.push(Some(space_edges[idx].clone()));
                        continue;
                    }
                }
            }
            results.push(None);
        }

        Ok(results)
    }

    fn delete_edge(&self, space_id: i32, edge_key: &EdgeKey) -> ManagerResult<ExecResponse> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let mut edge_index = self
            .edge_index
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        if let Some(space_index) = edge_index.get_mut(&space_id) {
            if let Some(idx) = space_index.remove(edge_key) {
                let mut edges = self
                    .edges
                    .write()
                    .map_err(|e| ManagerError::StorageError(e.to_string()))?;
                if let Some(space_edges) = edges.get_mut(&space_id) {
                    if idx < space_edges.len() {
                        space_edges.remove(idx);

                        for key in space_index.values_mut() {
                            if *key > idx {
                                *key -= 1;
                            }
                        }
                    }
                }
            }
        }

        Ok(ExecResponse::ok())
    }

    fn delete_edges(&self, space_id: i32, edge_keys: &[EdgeKey]) -> ManagerResult<ExecResponse> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        for edge_key in edge_keys {
            self.delete_edge(space_id, edge_key)?;
        }

        Ok(ExecResponse::ok())
    }

    fn update_edge(
        &self,
        space_id: i32,
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

        let mut edges = self
            .edges
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let mut edge_index = self
            .edge_index
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let space_index = edge_index.entry(space_id).or_insert_with(HashMap::new);
        let space_edges = edges.entry(space_id).or_insert_with(Vec::new);

        let mut inserted = false;
        let mut props_to_return: Option<HashMap<String, Value>> = None;

        if let Some(&idx) = space_index.get(edge_key) {
            if idx < space_edges.len() {
                for updated_prop in updated_props {
                    space_edges[idx]
                        .props
                        .insert(updated_prop.name.clone(), updated_prop.value);
                }

                if !return_props.is_empty() {
                    let mut return_map = HashMap::new();
                    for prop_name in return_props {
                        if let Some(value) = space_edges[idx].props.get(&prop_name) {
                            return_map.insert(prop_name, value.clone());
                        }
                    }
                    props_to_return = Some(return_map);
                }
            }
        } else if insertable {
            let mut edge_props = HashMap::new();
            for updated_prop in updated_props {
                edge_props.insert(updated_prop.name.clone(), updated_prop.value);
            }
            let idx = space_edges.len();
            let edge = Edge {
                src: Box::new(edge_key.src.clone()),
                dst: Box::new(edge_key.dst.clone()),
                edge_type: edge_key.edge_type.clone(),
                ranking: edge_key.ranking,
                id: idx as i64,
                props: edge_props.clone(),
            };
            space_edges.push(edge);
            space_index.insert(edge_key.clone(), idx);
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

    fn scan_vertices(&self, space_id: i32, limit: Option<usize>) -> ManagerResult<Vec<Vertex>> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let vertices = self
            .vertices
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        if let Some(space_vertices) = vertices.get(&space_id) {
            let result: Vec<Vertex> = space_vertices.values().cloned().collect();
            if let Some(limit) = limit {
                Ok(result.into_iter().take(limit).collect())
            } else {
                Ok(result)
            }
        } else {
            Ok(Vec::new())
        }
    }

    fn scan_vertices_by_tag(
        &self,
        space_id: i32,
        tag_id: i32,
        limit: Option<usize>,
    ) -> ManagerResult<Vec<Vertex>> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let vertices = self
            .vertices
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        if let Some(space_vertices) = vertices.get(&space_id) {
            let tag_name = format!("tag_{}", tag_id);
            let result: Vec<Vertex> = space_vertices
                .values()
                .filter(|v| v.has_tag(&tag_name))
                .cloned()
                .collect();

            if let Some(limit) = limit {
                Ok(result.into_iter().take(limit).collect())
            } else {
                Ok(result)
            }
        } else {
            Ok(Vec::new())
        }
    }

    fn scan_edges(&self, space_id: i32, limit: Option<usize>) -> ManagerResult<Vec<Edge>> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let edges = self
            .edges
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        if let Some(space_edges) = edges.get(&space_id) {
            let result: Vec<Edge> = space_edges.clone();
            if let Some(limit) = limit {
                Ok(result.into_iter().take(limit).collect())
            } else {
                Ok(result)
            }
        } else {
            Ok(Vec::new())
        }
    }

    fn scan_edges_by_type(
        &self,
        space_id: i32,
        edge_type: &str,
        limit: Option<usize>,
    ) -> ManagerResult<Vec<Edge>> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let edges = self
            .edges
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        if let Some(space_edges) = edges.get(&space_id) {
            let result: Vec<Edge> = space_edges
                .iter()
                .filter(|e| e.edge_type == edge_type)
                .cloned()
                .collect();

            if let Some(limit) = limit {
                Ok(result.into_iter().take(limit).collect())
            } else {
                Ok(result)
            }
        } else {
            Ok(Vec::new())
        }
    }

    fn scan_edges_by_src(
        &self,
        space_id: i32,
        src: &Value,
        limit: Option<usize>,
    ) -> ManagerResult<Vec<Edge>> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let edges = self
            .edges
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        if let Some(space_edges) = edges.get(&space_id) {
            let result: Vec<Edge> = space_edges
                .iter()
                .filter(|e| e.src.as_ref() == src)
                .cloned()
                .collect();

            if let Some(limit) = limit {
                Ok(result.into_iter().take(limit).collect())
            } else {
                Ok(result)
            }
        } else {
            Ok(Vec::new())
        }
    }

    fn scan_edges_by_dst(
        &self,
        space_id: i32,
        dst: &Value,
        limit: Option<usize>,
    ) -> ManagerResult<Vec<Edge>> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "存储客户端未连接".to_string(),
            ));
        }

        let edges = self
            .edges
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        if let Some(space_edges) = edges.get(&space_id) {
            let result: Vec<Edge> = space_edges
                .iter()
                .filter(|e| e.dst.as_ref() == dst)
                .cloned()
                .collect();

            if let Some(limit) = limit {
                Ok(result.into_iter().take(limit).collect())
            } else {
                Ok(result)
            }
        } else {
            Ok(Vec::new())
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
