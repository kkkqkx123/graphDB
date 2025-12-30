//! 索引管理器实现 - 内存中的索引管理

use super::super::{Index, IndexManager, IndexStatus, IndexType, IndexBuildProgress};
use crate::core::{Value, Vertex, Edge};
use crate::core::error::{ManagerError, ManagerResult};
use super::index_binary::{IndexBinaryEncoder, ValueType};
use crate::storage::StorageEngine;
use std::collections::{HashMap, BTreeMap};
use std::path::PathBuf;
use std::sync::{Arc, RwLock, atomic::{AtomicBool, AtomicU64, Ordering}};
use std::thread;
use std::time::Duration;

/// 索引数据结构 - 使用二进制编码的键
#[derive(Debug)]
struct IndexData {
    vertex_index: BTreeMap<Vec<u8>, Vec<Vertex>>,
    edge_index: BTreeMap<Vec<u8>, Vec<Edge>>,
}

impl IndexData {
    fn new() -> Self {
        Self {
            vertex_index: BTreeMap::new(),
            edge_index: BTreeMap::new(),
        }
    }

    /// 插入顶点索引
    fn insert_vertex(&mut self, key: Vec<u8>, vertex: Vertex) {
        self.vertex_index.entry(key).or_insert_with(Vec::new).push(vertex);
    }

    /// 插入边索引
    fn insert_edge(&mut self, key: Vec<u8>, edge: Edge) {
        self.edge_index.entry(key).or_insert_with(Vec::new).push(edge);
    }

    /// 查找顶点
    fn lookup_vertex(&self, key: &[u8]) -> Option<&Vec<Vertex>> {
        self.vertex_index.get(key)
    }

    /// 查找边
    fn lookup_edge(&self, key: &[u8]) -> Option<&Vec<Edge>> {
        self.edge_index.get(key)
    }

    /// 范围查找顶点
    fn range_lookup_vertex(&self, start: &[u8], end: &[u8]) -> Vec<Vertex> {
        let mut result = Vec::new();
        for (_, vertices) in self.vertex_index.range(start.to_vec()..=end.to_vec()) {
            result.extend(vertices.clone());
        }
        result
    }

    /// 范围查找边
    fn range_lookup_edge(&self, start: &[u8], end: &[u8]) -> Vec<Edge> {
        let mut result = Vec::new();
        for (_, edges) in self.edge_index.range(start.to_vec()..=end.to_vec()) {
            result.extend(edges.clone());
        }
        result
    }
}

/// 索引构建任务
#[derive(Debug)]
struct IndexBuildTask {
    index_id: i32,
    total_count: u64,
    processed_count: Arc<AtomicU64>,
    is_cancelled: Arc<AtomicBool>,
    status: Arc<RwLock<IndexStatus>>,
    error_message: Arc<RwLock<Option<String>>>,
}

/// 内存中的索引管理器实现
#[derive(Clone)]
pub struct MemoryIndexManager {
    indexes: Arc<RwLock<HashMap<String, Index>>>,
    next_index_id: Arc<RwLock<i32>>,
    storage_path: PathBuf,
    build_tasks: Arc<RwLock<HashMap<i32, IndexBuildTask>>>,
    index_data: Arc<RwLock<HashMap<i32, IndexData>>>,
    storage_engine: Option<Arc<dyn StorageEngine>>,
}

impl MemoryIndexManager {
    /// 创建新的内存索引管理器
    pub fn new(storage_path: PathBuf) -> Self {
        let mut manager = Self {
            indexes: Arc::new(RwLock::new(HashMap::new())),
            next_index_id: Arc::new(RwLock::new(1)),
            storage_path,
            build_tasks: Arc::new(RwLock::new(HashMap::new())),
            index_data: Arc::new(RwLock::new(HashMap::new())),
            storage_engine: None,
        };
        let _ = manager.load_from_disk();
        manager
    }

    /// 创建新的内存索引管理器并设置存储引擎
    pub fn with_storage_engine(storage_path: PathBuf, storage_engine: Arc<dyn StorageEngine>) -> Self {
        let manager = Self {
            indexes: Arc::new(RwLock::new(HashMap::new())),
            next_index_id: Arc::new(RwLock::new(1)),
            storage_path,
            build_tasks: Arc::new(RwLock::new(HashMap::new())),
            index_data: Arc::new(RwLock::new(HashMap::new())),
            storage_engine: Some(storage_engine),
        };
        let _ = manager.load_from_disk();
        manager
    }

    /// 设置存储引擎
    pub fn set_storage_engine(&mut self, storage_engine: Arc<dyn StorageEngine>) {
        self.storage_engine = Some(storage_engine);
    }

    /// 添加索引
    pub fn add_index(&self, index: Index) -> ManagerResult<()> {
        let mut indexes = self.indexes.write().map_err(|e| ManagerError::StorageError(e.to_string()))?;
        indexes.insert(index.name.clone(), index);
        Ok(())
    }

    /// 删除索引
    pub fn remove_index(&self, name: &str) -> ManagerResult<()> {
        let mut indexes = self.indexes.write().map_err(|e| ManagerError::StorageError(e.to_string()))?;
        indexes.remove(name);
        Ok(())
    }

    /// 更新索引
    pub fn update_index(&self, name: &str, index: Index) -> ManagerResult<()> {
        let mut indexes = self.indexes.write().map_err(|e| ManagerError::StorageError(e.to_string()))?;
        indexes.insert(name.to_string(), index);
        Ok(())
    }

    /// 根据Schema名称获取索引
    pub fn get_indexes_by_schema(&self, schema_name: &str) -> Vec<Index> {
        match self.indexes.read() {
            Ok(indexes) => indexes
                .values()
                .filter(|index| index.schema_name == schema_name)
                .cloned()
                .collect(),
            Err(_) => Vec::new(),
        }
    }

    /// 检查字段是否被索引
    pub fn is_field_indexed(&self, schema_name: &str, field_name: &str) -> bool {
        match self.indexes.read() {
            Ok(indexes) => indexes.values().any(|index| {
                index.schema_name == schema_name && index.fields.contains(&field_name.to_string())
            }),
            Err(_) => false,
        }
    }
}

impl Default for MemoryIndexManager {
    fn default() -> Self {
        Self::new(PathBuf::from("./data/indexes"))
    }
}

impl IndexManager for MemoryIndexManager {
    fn get_index(&self, name: &str) -> Option<Index> {
        let indexes = self.indexes.read().ok()?;
        indexes.get(name).cloned()
    }

    fn list_indexes(&self) -> Vec<String> {
        match self.indexes.read() {
            Ok(indexes) => indexes.keys().cloned().collect(),
            Err(_) => Vec::new(),
        }
    }

    fn has_index(&self, name: &str) -> bool {
        match self.indexes.read() {
            Ok(indexes) => indexes.contains_key(name),
            Err(_) => false,
        }
    }

    fn create_index(&self, space_id: i32, mut index: Index) -> ManagerResult<i32> {
        let mut indexes = self.indexes.write().map_err(|e| ManagerError::StorageError(e.to_string()))?;
        
        if indexes.contains_key(&index.name) {
            return Err(ManagerError::AlreadyExists(format!("索引 {} 已存在", index.name)));
        }
        
        let mut next_id = self.next_index_id.write().map_err(|e| ManagerError::StorageError(e.to_string()))?;
        index.id = *next_id;
        index.space_id = space_id;
        index.status = IndexStatus::Creating;
        *next_id += 1;
        
        indexes.insert(index.name.clone(), index.clone());
        drop(indexes);
        
        let _ = self.save_to_disk();
        
        Ok(index.id)
    }

    fn drop_index(&self, _space_id: i32, index_id: i32) -> ManagerResult<()> {
        let mut indexes = self.indexes.write().map_err(|e| ManagerError::StorageError(e.to_string()))?;
        
        let index_name = indexes
            .values()
            .find(|idx| idx.id == index_id)
            .map(|idx| idx.name.clone())
            .ok_or_else(|| ManagerError::NotFound(format!("索引ID {} 不存在", index_id)))?;
        
        indexes.remove(&index_name);
        drop(indexes);
        
        let _ = self.save_to_disk();
        
        Ok(())
    }

    fn get_index_status(&self, _space_id: i32, index_id: i32) -> Option<IndexStatus> {
        let indexes = self.indexes.read().ok()?;
        indexes.values().find(|idx| idx.id == index_id).map(|idx| idx.status.clone())
    }

    fn list_indexes_by_space(&self, space_id: i32) -> ManagerResult<Vec<Index>> {
        let indexes = self.indexes.read().map_err(|e| ManagerError::StorageError(e.to_string()))?;
        Ok(indexes
            .values()
            .filter(|idx| idx.space_id == space_id)
            .cloned()
            .collect())
    }

    fn load_from_disk(&self) -> ManagerResult<()> {
        use std::fs;
        
        if !self.storage_path.exists() {
            fs::create_dir_all(&self.storage_path).map_err(|e| ManagerError::StorageError(e.to_string()))?;
            return Ok(());
        }
        
        let index_file = self.storage_path.join("indexes.json");
        if !index_file.exists() {
            return Ok(());
        }
        
        let content = fs::read_to_string(&index_file).map_err(|e| ManagerError::StorageError(e.to_string()))?;
        
        let loaded_indexes: Vec<Index> = serde_json::from_str(&content)
            .map_err(|e| ManagerError::IndexError(format!("反序列化索引失败: {}", e)))?;
        
        let mut indexes = self.indexes.write().map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let mut next_id = self.next_index_id.write().map_err(|e| ManagerError::StorageError(e.to_string()))?;
        
        for index in loaded_indexes {
            if index.id >= *next_id {
                *next_id = index.id + 1;
            }
            indexes.insert(index.name.clone(), index);
        }
        
        Ok(())
    }

    fn save_to_disk(&self) -> ManagerResult<()> {
        use std::fs;
        
        if !self.storage_path.exists() {
            fs::create_dir_all(&self.storage_path).map_err(|e| ManagerError::StorageError(e.to_string()))?;
        }
        
        let indexes = self.indexes.read().map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let index_list: Vec<Index> = indexes.values().cloned().collect();
        
        let content = serde_json::to_string_pretty(&index_list)
            .map_err(|e| ManagerError::IndexError(format!("序列化索引失败: {}", e)))?;
        
        let index_file = self.storage_path.join("indexes.json");
        fs::write(&index_file, content).map_err(|e| ManagerError::StorageError(e.to_string()))?;
        
        Ok(())
    }

    fn build_index_async(&self, space_id: i32, index_id: i32) -> ManagerResult<()> {
        let (index_name, index_type, schema_name, fields) = {
            let indexes = self.indexes.read().map_err(|e| ManagerError::StorageError(e.to_string()))?;
            let index = indexes
                .values()
                .find(|idx| idx.id == index_id && idx.space_id == space_id)
                .ok_or_else(|| ManagerError::NotFound(format!("索引ID {} 在Space {} 中不存在", index_id, space_id)))?;
            
            if index.status != IndexStatus::Creating {
                return Err(ManagerError::IndexError(format!("索引 {} 状态为 {:?}，无法开始构建", index.name, index.status)));
            }
            
            (index.name.clone(), index.index_type.clone(), index.schema_name.clone(), index.fields.clone())
        };
        
        let mut indexes = self.indexes.write().map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let index = indexes.get_mut(&index_name).expect("索引不存在");
        index.status = IndexStatus::Building;
        drop(indexes);
        
        let processed_count = Arc::new(AtomicU64::new(0));
        let is_cancelled = Arc::new(AtomicBool::new(false));
        let status = Arc::new(RwLock::new(IndexStatus::Building));
        let error_message = Arc::new(RwLock::new(None));
        
        let storage_engine = self.storage_engine.clone();
        
        let indexes_clone = self.indexes.clone();
        let build_tasks_clone = self.build_tasks.clone();
        let index_data_clone = self.index_data.clone();
        let index_name_clone = index_name.clone();
        let index_type_clone = index_type.clone();
        let schema_name_clone = schema_name.clone();
        let fields_clone = fields.clone();
        
        let build_task = IndexBuildTask {
            index_id,
            total_count: 0,
            processed_count: processed_count.clone(),
            is_cancelled: is_cancelled.clone(),
            status: status.clone(),
            error_message: error_message.clone(),
        };
        
        let mut build_tasks = self.build_tasks.write().map_err(|e| ManagerError::StorageError(e.to_string()))?;
        build_tasks.insert(index_id, build_task);
        drop(build_tasks);
        
        thread::spawn(move || {
            let result = if let Some(storage) = storage_engine {
                Self::build_index_from_storage(
                    &storage,
                    index_id,
                    &index_type_clone,
                    &schema_name_clone,
                    &fields_clone,
                    &processed_count,
                    &is_cancelled,
                    &index_data_clone,
                )
            } else {
                Err("存储引擎未设置".to_string())
            };
            
            match result {
                Ok(total_count) => {
                    let mut build_tasks = build_tasks_clone.write().expect("无法获取构建任务锁");
                    if let Some(task) = build_tasks.get_mut(&index_id) {
                        task.total_count = total_count;
                    }
                    drop(build_tasks);
                    
                    let mut status_guard = status.write().expect("无法获取状态锁");
                    *status_guard = IndexStatus::Active;
                    
                    let mut indexes = indexes_clone.write().expect("无法获取索引锁");
                    if let Some(index) = indexes.get_mut(&index_name_clone) {
                        index.status = IndexStatus::Active;
                    }
                    drop(indexes);
                    
                    let mut build_tasks = build_tasks_clone.write().expect("无法获取构建任务锁");
                    build_tasks.remove(&index_id);
                }
                Err(error) => {
                    let mut error_guard = error_message.write().expect("无法获取错误信息锁");
                    *error_guard = Some(error.clone());
                    drop(error_guard);
                    
                    let mut status_guard = status.write().expect("无法获取状态锁");
                    *status_guard = IndexStatus::Failed;
                    drop(status_guard);
                    
                    let mut indexes = indexes_clone.write().expect("无法获取索引锁");
                    if let Some(index) = indexes.get_mut(&index_name_clone) {
                        index.status = IndexStatus::Failed;
                    }
                    drop(indexes);
                    
                    let mut build_tasks = build_tasks_clone.write().expect("无法获取构建任务锁");
                    build_tasks.remove(&index_id);
                }
            }
        });
        
        Ok(())
    }
}

impl MemoryIndexManager {
    /// 从存储引擎构建索引
    fn build_index_from_storage(
        storage: &Arc<dyn StorageEngine>,
        index_id: i32,
        index_type: &IndexType,
        schema_name: &str,
        fields: &[String],
        processed_count: &Arc<AtomicU64>,
        is_cancelled: &Arc<AtomicBool>,
        index_data: &Arc<RwLock<HashMap<i32, IndexData>>>,
    ) -> Result<u64, String> {
        match index_type {
            IndexType::TagIndex => {
                Self::build_vertex_index(storage, index_id, schema_name, fields, processed_count, is_cancelled, index_data)
            }
            IndexType::EdgeIndex => {
                Self::build_edge_index(storage, index_id, schema_name, fields, processed_count, is_cancelled, index_data)
            }
            IndexType::FulltextIndex => {
                return Err("全文索引暂不支持".to_string());
            }
        }
    }

    /// 构建顶点索引
    fn build_vertex_index(
        storage: &Arc<dyn StorageEngine>,
        index_id: i32,
        tag_name: &str,
        fields: &[String],
        processed_count: &Arc<AtomicU64>,
        is_cancelled: &Arc<AtomicBool>,
        index_data: &Arc<RwLock<HashMap<i32, IndexData>>>,
    ) -> Result<u64, String> {
        let vertices = storage.scan_vertices_by_tag(tag_name)
            .map_err(|e| format!("扫描顶点失败: {}", e))?;
        
        let total_count = vertices.len() as u64;
        
        let mut index_data_guard = index_data.write()
            .map_err(|e| format!("获取索引数据锁失败: {}", e))?;
        
        let data = index_data_guard.entry(index_id).or_insert_with(IndexData::new);
        
        for vertex in vertices {
            if is_cancelled.load(Ordering::Relaxed) {
                return Err("索引构建已取消".to_string());
            }
            
            let index_values = Self::extract_vertex_index_values(&vertex, tag_name, fields);
            let key = IndexBinaryEncoder::encode(&index_values);
            
            data.insert_vertex(key, vertex);
            
            let count = processed_count.fetch_add(1, Ordering::Relaxed) + 1;
        }
        
        drop(index_data_guard);
        
        Ok(total_count)
    }

    /// 构建边索引
    fn build_edge_index(
        storage: &Arc<dyn StorageEngine>,
        index_id: i32,
        edge_type: &str,
        fields: &[String],
        processed_count: &Arc<AtomicU64>,
        is_cancelled: &Arc<AtomicBool>,
        index_data: &Arc<RwLock<HashMap<i32, IndexData>>>,
    ) -> Result<u64, String> {
        let edges = storage.scan_edges_by_type(edge_type)
            .map_err(|e| format!("扫描边失败: {}", e))?;
        
        let total_count = edges.len() as u64;
        
        let mut index_data_guard = index_data.write()
            .map_err(|e| format!("获取索引数据锁失败: {}", e))?;
        
        let data = index_data_guard.entry(index_id).or_insert_with(IndexData::new);
        
        for edge in edges {
            if is_cancelled.load(Ordering::Relaxed) {
                return Err("索引构建已取消".to_string());
            }
            
            let index_values = Self::extract_edge_index_values(&edge, edge_type, fields);
            let key = IndexBinaryEncoder::encode(&index_values);
            
            data.insert_edge(key, edge);
            
            let count = processed_count.fetch_add(1, Ordering::Relaxed) + 1;
        }
        
        drop(index_data_guard);
        
        Ok(total_count)
    }

    /// 从顶点提取索引值
    fn extract_vertex_index_values(vertex: &Vertex, tag_name: &str, fields: &[String]) -> Vec<Value> {
        let mut values = Vec::new();
        
        if let Some(tag) = vertex.tags.iter().find(|t| t.name == tag_name) {
            for field_name in fields {
                if let Some(value) = tag.properties.get(field_name) {
                    values.push(value.clone());
                }
            }
        }
        
        values
    }

    /// 从边提取索引值
    fn extract_edge_index_values(edge: &Edge, edge_type: &str, fields: &[String]) -> Vec<Value> {
        let mut values = Vec::new();
        
        if edge.edge_type == edge_type {
            for field_name in fields {
                if let Some(value) = edge.props.get(field_name) {
                    values.push(value.clone());
                }
            }
        }
        
        values
    }

    fn get_build_progress(&self, space_id: i32, index_id: i32) -> Option<IndexBuildProgress> {
        let indexes = self.indexes.read().ok()?;
        let index = indexes
            .values()
            .find(|idx| idx.id == index_id && idx.space_id == space_id)?;
        
        let build_tasks = self.build_tasks.read().ok()?;
        let task = build_tasks.get(&index_id)?;
        
        let status_guard = task.status.read().ok()?;
        let error_guard = task.error_message.read().ok()?;
        
        Some(IndexBuildProgress {
            index_id: task.index_id,
            index_name: index.name.clone(),
            total_count: task.total_count,
            processed_count: task.processed_count.load(Ordering::Relaxed),
            progress_percent: if task.total_count > 0 {
                (task.processed_count.load(Ordering::Relaxed) as f64 / task.total_count as f64) * 100.0
            } else {
                100.0
            },
            status: status_guard.clone(),
            error_message: error_guard.clone(),
        })
    }

    fn cancel_build(&self, space_id: i32, index_id: i32) -> ManagerResult<()> {
        let index_name = {
            let indexes = self.indexes.read().map_err(|e| ManagerError::StorageError(e.to_string()))?;
            let index = indexes
                .values()
                .find(|idx| idx.id == index_id && idx.space_id == space_id)
                .ok_or_else(|| ManagerError::NotFound(format!("索引ID {} 在Space {} 中不存在", index_id, space_id)))?;
            
            if index.status != IndexStatus::Building {
                return Err(ManagerError::IndexError(format!("索引 {} 状态为 {:?}，无法取消构建", index.name, index.status)));
            }
            
            index.name.clone()
        };
        
        let build_tasks = self.build_tasks.read().map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let task = build_tasks
            .get(&index_id)
            .ok_or_else(|| ManagerError::NotFound(format!("索引 {} 没有正在进行的构建任务", index_name)))?;
        
        task.is_cancelled.store(true, Ordering::Relaxed);
        
        Ok(())
    }

    fn lookup_vertex_by_index(
        &self,
        space_id: i32,
        index_name: &str,
        values: &[Value],
    ) -> ManagerResult<Vec<Vertex>> {
        let indexes = self.indexes.read().map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let index = indexes
            .get(index_name)
            .ok_or_else(|| ManagerError::NotFound(format!("索引 {} 不存在", index_name)))?;
        
        if index.space_id != space_id {
            return Err(ManagerError::InvalidInput(format!("索引 {} 不属于Space {}", index_name, space_id)));
        }
        
        if index.index_type != IndexType::TagIndex {
            return Err(ManagerError::InvalidInput(format!("索引 {} 不是顶点索引", index_name)));
        }
        
        if index.status != IndexStatus::Active {
            return Err(ManagerError::InvalidInput(format!("索引 {} 状态为 {:?}，不可用", index_name, index.status)));
        }
        
        let key = IndexBinaryEncoder::encode(values);
        
        let index_data = self.index_data.read().map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let data = index_data
            .get(&index.id)
            .ok_or_else(|| ManagerError::NotFound(format!("索引 {} 的数据不存在", index_name)))?;
        
        data.lookup_vertex(&key)
            .map(|vertices| vertices.clone())
            .ok_or_else(|| ManagerError::NotFound(format!("索引 {} 中未找到匹配的顶点", index_name)))
    }

    fn lookup_edge_by_index(
        &self,
        space_id: i32,
        index_name: &str,
        values: &[Value],
    ) -> ManagerResult<Vec<Edge>> {
        let indexes = self.indexes.read().map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let index = indexes
            .get(index_name)
            .ok_or_else(|| ManagerError::NotFound(format!("索引 {} 不存在", index_name)))?;
        
        if index.space_id != space_id {
            return Err(ManagerError::InvalidInput(format!("索引 {} 不属于Space {}", index_name, space_id)));
        }
        
        if index.index_type != IndexType::EdgeIndex {
            return Err(ManagerError::InvalidInput(format!("索引 {} 不是边索引", index_name)));
        }
        
        if index.status != IndexStatus::Active {
            return Err(ManagerError::InvalidInput(format!("索引 {} 状态为 {:?}，不可用", index_name, index.status)));
        }
        
        let key = IndexBinaryEncoder::encode(values);
        
        let index_data = self.index_data.read().map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let data = index_data
            .get(&index.id)
            .ok_or_else(|| ManagerError::NotFound(format!("索引 {} 的数据不存在", index_name)))?;
        
        data.lookup_edge(&key)
            .map(|edges| edges.clone())
            .ok_or_else(|| ManagerError::NotFound(format!("索引 {} 中未找到匹配的边", index_name)))
    }

    fn range_lookup_vertex(
        &self,
        space_id: i32,
        index_name: &str,
        start: &Value,
        end: &Value,
    ) -> ManagerResult<Vec<Vertex>> {
        let indexes = self.indexes.read().map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let index = indexes
            .get(index_name)
            .ok_or_else(|| ManagerError::NotFound(format!("索引 {} 不存在", index_name)))?;
        
        if index.space_id != space_id {
            return Err(ManagerError::InvalidInput(format!("索引 {} 不属于Space {}", index_name, space_id)));
        }
        
        if index.index_type != IndexType::TagIndex {
            return Err(ManagerError::InvalidInput(format!("索引 {} 不是顶点索引", index_name)));
        }
        
        if index.status != IndexStatus::Active {
            return Err(ManagerError::InvalidInput(format!("索引 {} 状态为 {:?}，不可用", index_name, index.status)));
        }
        
        let start_key = IndexBinaryEncoder::encode(&[start.clone()]);
        let end_key = IndexBinaryEncoder::encode(&[end.clone()]);
        
        let index_data = self.index_data.read().map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let data = index_data
            .get(&index.id)
            .ok_or_else(|| ManagerError::NotFound(format!("索引 {} 的数据不存在", index_name)))?;
        
        Ok(data.range_lookup_vertex(&start_key, &end_key))
    }

    fn range_lookup_edge(
        &self,
        space_id: i32,
        index_name: &str,
        start: &Value,
        end: &Value,
    ) -> ManagerResult<Vec<Edge>> {
        let indexes = self.indexes.read().map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let index = indexes
            .get(index_name)
            .ok_or_else(|| ManagerError::NotFound(format!("索引 {} 不存在", index_name)))?;
        
        if index.space_id != space_id {
            return Err(ManagerError::InvalidInput(format!("索引 {} 不属于Space {}", index_name, space_id)));
        }
        
        if index.index_type != IndexType::EdgeIndex {
            return Err(ManagerError::InvalidInput(format!("索引 {} 不是边索引", index_name)));
        }
        
        if index.status != IndexStatus::Active {
            return Err(ManagerError::InvalidInput(format!("索引 {} 状态为 {:?}，不可用", index_name, index.status)));
        }
        
        let start_key = IndexBinaryEncoder::encode(&[start.clone()]);
        let end_key = IndexBinaryEncoder::encode(&[end.clone()]);
        
        let index_data = self.index_data.read().map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let data = index_data
            .get(&index.id)
            .ok_or_else(|| ManagerError::NotFound(format!("索引 {} 的数据不存在", index_name)))?;
        
        Ok(data.range_lookup_edge(&start_key, &end_key))
    }

    fn insert_vertex_to_index(&self, space_id: i32, vertex: &Vertex) -> ManagerResult<()> {
        let indexes = self.indexes.read().map_err(|e| ManagerError::StorageError(e.to_string()))?;
        
        let vertex_indexes: Vec<_> = indexes
            .values()
            .filter(|index| {
                index.space_id == space_id
                    && index.index_type == IndexType::TagIndex
                    && index.status == IndexStatus::Active
                    && vertex.tags.iter().any(|tag| tag.name == index.schema_name)
            })
            .collect();
        
        drop(indexes);
        
        let mut index_data = self.index_data.write().map_err(|e| ManagerError::StorageError(e.to_string()))?;
        
        for index in vertex_indexes {
            if let Some(data) = index_data.get_mut(&index.id) {
                let index_values = Self::extract_vertex_index_values(vertex, &index.schema_name, &index.fields);
                let key = IndexBinaryEncoder::encode(&index_values);
                data.insert_vertex(key, vertex.clone());
            }
        }
        
        Ok(())
    }

    fn delete_vertex_from_index(&self, space_id: i32, vertex: &Vertex) -> ManagerResult<()> {
        let indexes = self.indexes.read().map_err(|e| ManagerError::StorageError(e.to_string()))?;
        
        let vertex_indexes: Vec<_> = indexes
            .values()
            .filter(|index| {
                index.space_id == space_id
                    && index.index_type == IndexType::TagIndex
                    && index.status == IndexStatus::Active
                    && vertex.tags.iter().any(|tag| tag.name == index.schema_name)
            })
            .collect();
        
        drop(indexes);
        
        let mut index_data = self.index_data.write().map_err(|e| ManagerError::StorageError(e.to_string()))?;
        
        for index in vertex_indexes {
            if let Some(data) = index_data.get_mut(&index.id) {
                let index_values = Self::extract_vertex_index_values(vertex, &index.schema_name, &index.fields);
                let key = IndexBinaryEncoder::encode(&index_values);
                
                if let Some(vertices) = data.vertex_index.get_mut(&key) {
                    vertices.retain(|v| v.id() != vertex.id());
                    if vertices.is_empty() {
                        data.vertex_index.remove(&key);
                    }
                }
            }
        }
        
        Ok(())
    }

    fn update_vertex_in_index(&self, space_id: i32, old_vertex: &Vertex, new_vertex: &Vertex) -> ManagerResult<()> {
        self.delete_vertex_from_index(space_id, old_vertex)?;
        self.insert_vertex_to_index(space_id, new_vertex)?;
        Ok(())
    }

    fn insert_edge_to_index(&self, space_id: i32, edge: &Edge) -> ManagerResult<()> {
        let indexes = self.indexes.read().map_err(|e| ManagerError::StorageError(e.to_string()))?;
        
        let edge_indexes: Vec<_> = indexes
            .values()
            .filter(|index| {
                index.space_id == space_id
                    && index.index_type == IndexType::EdgeIndex
                    && index.status == IndexStatus::Active
                    && index.schema_name == edge.edge_type
            })
            .collect();
        
        drop(indexes);
        
        let mut index_data = self.index_data.write().map_err(|e| ManagerError::StorageError(e.to_string()))?;
        
        for index in edge_indexes {
            if let Some(data) = index_data.get_mut(&index.id) {
                let index_values = Self::extract_edge_index_values(edge, &index.schema_name, &index.fields);
                let key = IndexBinaryEncoder::encode(&index_values);
                data.insert_edge(key, edge.clone());
            }
        }
        
        Ok(())
    }

    fn delete_edge_from_index(&self, space_id: i32, edge: &Edge) -> ManagerResult<()> {
        let indexes = self.indexes.read().map_err(|e| ManagerError::StorageError(e.to_string()))?;
        
        let edge_indexes: Vec<_> = indexes
            .values()
            .filter(|index| {
                index.space_id == space_id
                    && index.index_type == IndexType::EdgeIndex
                    && index.status == IndexStatus::Active
                    && index.schema_name == edge.edge_type
            })
            .collect();
        
        drop(indexes);
        
        let mut index_data = self.index_data.write().map_err(|e| ManagerError::StorageError(e.to_string()))?;
        
        for index in edge_indexes {
            if let Some(data) = index_data.get_mut(&index.id) {
                let index_values = Self::extract_edge_index_values(edge, &index.schema_name, &index.fields);
                let key = IndexBinaryEncoder::encode(&index_values);
                
                if let Some(edges) = data.edge_index.get_mut(&key) {
                    edges.retain(|e| e.src != edge.src || e.dst != edge.dst || e.edge_type != edge.edge_type);
                    if edges.is_empty() {
                        data.edge_index.remove(&key);
                    }
                }
            }
        }
        
        Ok(())
    }

    fn update_edge_in_index(&self, space_id: i32, old_edge: &Edge, new_edge: &Edge) -> ManagerResult<()> {
        self.delete_edge_from_index(space_id, old_edge)?;
        self.insert_edge_to_index(space_id, new_edge)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_memory_index_manager_creation() {
        let temp_dir = tempdir().unwrap();
        let storage_path = temp_dir.path().to_path_buf();
        let manager = MemoryIndexManager::new(storage_path);
        assert!(manager.list_indexes().is_empty());
    }

    #[test]
    fn test_memory_index_manager_add_index() {
        let temp_dir = tempdir().unwrap();
        let storage_path = temp_dir.path().to_path_buf();
        let manager = MemoryIndexManager::new(storage_path);

        let index = Index {
            id: 1,
            name: "idx_users_id".to_string(),
            space_id: 1,
            schema_name: "users".to_string(),
            fields: vec!["id".to_string()],
            index_type: IndexType::TagIndex,
            status: IndexStatus::Active,
            is_unique: true,
            comment: None,
        };

        assert!(manager.add_index(index).is_ok());
        assert!(manager.has_index("idx_users_id"));
        assert_eq!(manager.list_indexes(), vec!["idx_users_id".to_string()]);
    }

    #[test]
    fn test_memory_index_manager_get_index() {
        let temp_dir = tempdir().unwrap();
        let storage_path = temp_dir.path().to_path_buf();
        let manager = MemoryIndexManager::new(storage_path);

        let index = Index {
            id: 1,
            name: "idx_users_id".to_string(),
            space_id: 1,
            schema_name: "users".to_string(),
            fields: vec!["id".to_string()],
            index_type: IndexType::TagIndex,
            status: IndexStatus::Active,
            is_unique: true,
            comment: None,
        };

        manager
            .add_index(index.clone())
            .expect("Failed to add index");

        let retrieved = manager.get_index("idx_users_id");
        assert!(retrieved.is_some());
        assert_eq!(
            retrieved.expect("Failed to retrieve index").name,
            "idx_users_id"
        );
    }

    #[test]
    fn test_memory_index_manager_remove_index() {
        let temp_dir = tempdir().unwrap();
        let storage_path = temp_dir.path().to_path_buf();
        let manager = MemoryIndexManager::new(storage_path);

        let index = Index {
            id: 1,
            name: "idx_users_id".to_string(),
            space_id: 1,
            schema_name: "users".to_string(),
            fields: vec!["id".to_string()],
            index_type: IndexType::TagIndex,
            status: IndexStatus::Active,
            is_unique: true,
            comment: None,
        };

        manager.add_index(index).expect("Failed to add index");
        assert!(manager.has_index("idx_users_id"));

        manager
            .remove_index("idx_users_id")
            .expect("Failed to remove index");
        assert!(!manager.has_index("idx_users_id"));
    }

    #[test]
    fn test_memory_index_manager_get_indexes_by_schema() {
        let temp_dir = tempdir().unwrap();
        let storage_path = temp_dir.path().to_path_buf();
        let manager = MemoryIndexManager::new(storage_path);

        let index1 = Index {
            id: 1,
            name: "idx_users_id".to_string(),
            space_id: 1,
            schema_name: "users".to_string(),
            fields: vec!["id".to_string()],
            index_type: IndexType::TagIndex,
            status: IndexStatus::Active,
            is_unique: true,
            comment: None,
        };

        let index2 = Index {
            id: 2,
            name: "idx_users_name".to_string(),
            space_id: 1,
            schema_name: "users".to_string(),
            fields: vec!["name".to_string()],
            index_type: IndexType::TagIndex,
            status: IndexStatus::Active,
            is_unique: false,
            comment: None,
        };

        let index3 = Index {
            id: 3,
            name: "idx_orders_id".to_string(),
            space_id: 1,
            schema_name: "orders".to_string(),
            fields: vec!["id".to_string()],
            index_type: IndexType::TagIndex,
            status: IndexStatus::Active,
            is_unique: true,
            comment: None,
        };

        manager.add_index(index1).expect("Failed to add index1");
        manager.add_index(index2).expect("Failed to add index2");
        manager.add_index(index3).expect("Failed to add index3");

        let user_indexes = manager.get_indexes_by_schema("users");
        assert_eq!(user_indexes.len(), 2);

        let order_indexes = manager.get_indexes_by_schema("orders");
        assert_eq!(order_indexes.len(), 1);
    }

    #[test]
    fn test_memory_index_manager_is_field_indexed() {
        let temp_dir = tempdir().unwrap();
        let storage_path = temp_dir.path().to_path_buf();
        let manager = MemoryIndexManager::new(storage_path);

        let index = Index {
            id: 1,
            name: "idx_users_id".to_string(),
            space_id: 1,
            schema_name: "users".to_string(),
            fields: vec!["id".to_string()],
            index_type: IndexType::TagIndex,
            status: IndexStatus::Active,
            is_unique: true,
            comment: None,
        };

        manager.add_index(index).expect("Failed to add index");

        assert!(manager.is_field_indexed("users", "id"));
        assert!(!manager.is_field_indexed("users", "name"));
        assert!(!manager.is_field_indexed("orders", "id"));
    }

    #[test]
    fn test_memory_index_manager_create_index() {
        let temp_dir = tempdir().unwrap();
        let storage_path = temp_dir.path().to_path_buf();
        let manager = MemoryIndexManager::new(storage_path);

        let index = Index {
            id: 0,
            name: "idx_users_id".to_string(),
            space_id: 1,
            schema_name: "users".to_string(),
            fields: vec!["id".to_string()],
            index_type: IndexType::TagIndex,
            status: IndexStatus::Active,
            is_unique: true,
            comment: None,
        };

        let index_id = manager.create_index(1, index).expect("Failed to create index");
        assert_eq!(index_id, 1);
        assert!(manager.has_index("idx_users_id"));
    }

    #[test]
    fn test_memory_index_manager_drop_index() {
        let temp_dir = tempdir().unwrap();
        let storage_path = temp_dir.path().to_path_buf();
        let manager = MemoryIndexManager::new(storage_path);

        let index = Index {
            id: 0,
            name: "idx_users_id".to_string(),
            space_id: 1,
            schema_name: "users".to_string(),
            fields: vec!["id".to_string()],
            index_type: IndexType::TagIndex,
            status: IndexStatus::Active,
            is_unique: true,
            comment: None,
        };

        let index_id = manager.create_index(1, index).expect("Failed to create index");
        assert!(manager.has_index("idx_users_id"));

        manager.drop_index(1, index_id).expect("Failed to drop index");
        assert!(!manager.has_index("idx_users_id"));
    }

    #[test]
    fn test_memory_index_manager_get_index_status() {
        let temp_dir = tempdir().unwrap();
        let storage_path = temp_dir.path().to_path_buf();
        let manager = MemoryIndexManager::new(storage_path);

        let index = Index {
            id: 0,
            name: "idx_users_id".to_string(),
            space_id: 1,
            schema_name: "users".to_string(),
            fields: vec!["id".to_string()],
            index_type: IndexType::TagIndex,
            status: IndexStatus::Active,
            is_unique: true,
            comment: None,
        };

        let index_id = manager.create_index(1, index).expect("Failed to create index");
        let status = manager.get_index_status(1, index_id);
        assert!(status.is_some());
        assert_eq!(status.unwrap(), IndexStatus::Creating);
    }

    #[test]
    fn test_memory_index_manager_list_indexes_by_space() {
        let temp_dir = tempdir().unwrap();
        let storage_path = temp_dir.path().to_path_buf();
        let manager = MemoryIndexManager::new(storage_path);

        let index1 = Index {
            id: 0,
            name: "idx_space1_id".to_string(),
            space_id: 1,
            schema_name: "users".to_string(),
            fields: vec!["id".to_string()],
            index_type: IndexType::TagIndex,
            status: IndexStatus::Active,
            is_unique: true,
            comment: None,
        };

        let index2 = Index {
            id: 0,
            name: "idx_space2_id".to_string(),
            space_id: 2,
            schema_name: "orders".to_string(),
            fields: vec!["id".to_string()],
            index_type: IndexType::TagIndex,
            status: IndexStatus::Active,
            is_unique: true,
            comment: None,
        };

        manager.create_index(1, index1).expect("Failed to create index1");
        manager.create_index(2, index2).expect("Failed to create index2");

        let space1_indexes = manager.list_indexes_by_space(1).expect("Failed to list indexes");
        assert_eq!(space1_indexes.len(), 1);
        assert_eq!(space1_indexes[0].space_id, 1);

        let space2_indexes = manager.list_indexes_by_space(2).expect("Failed to list indexes");
        assert_eq!(space2_indexes.len(), 1);
        assert_eq!(space2_indexes[0].space_id, 2);
    }

    #[test]
    fn test_memory_index_manager_persistence() {
        let temp_dir = tempdir().unwrap();
        let storage_path = temp_dir.path().to_path_buf();
        
        let index = Index {
            id: 0,
            name: "idx_users_id".to_string(),
            space_id: 1,
            schema_name: "users".to_string(),
            fields: vec!["id".to_string()],
            index_type: IndexType::TagIndex,
            status: IndexStatus::Active,
            is_unique: true,
            comment: None,
        };

        {
            let manager = MemoryIndexManager::new(storage_path.clone());
            manager.create_index(1, index).expect("Failed to create index");
            manager.save_to_disk().expect("Failed to save to disk");
        }

        {
            let manager = MemoryIndexManager::new(storage_path);
            assert!(manager.has_index("idx_users_id"));
            let loaded_index = manager.get_index("idx_users_id");
            assert!(loaded_index.is_some());
            assert_eq!(loaded_index.unwrap().space_id, 1);
        }
    }
}
