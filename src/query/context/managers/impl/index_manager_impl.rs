//! 索引管理器实现 - 内存中的索引管理
//!
//! 采用 BTreeMap + HashMap 混合索引策略：
//! - BTreeMap: 支持范围查询和排序
//! - HashMap: 支持精确匹配的快速查找

use super::super::{Index, IndexManager, IndexStatus, IndexType, IndexStats, IndexOptimization};
use crate::core::error::{ManagerError, ManagerResult};
use crate::core::{Edge, Value, Vertex};
use crate::storage::StorageEngine;
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// 简化的索引数据结构 - 使用 BTreeMap + HashMap 混合索引
///
/// 主索引使用 BTreeMap，支持范围查询和排序
/// 辅助索引使用 HashMap，支持精确匹配的快速查找
#[derive(Debug)]
struct IndexData {
    /// 按标签、属性和属性值索引的顶点 - BTreeMap支持范围查询
    vertex_by_tag_property: BTreeMap<(String, String, Value), Vec<Vertex>>,
    /// 按内部ID精确查找顶点 - HashMap提供O(1)查询
    vertex_by_id: HashMap<i64, Vertex>,
    /// 按边类型、属性和属性值索引的边 - BTreeMap支持范围查询
    edge_by_type_property: BTreeMap<(String, String, Value), Vec<Edge>>,
    /// 按内部ID精确查找边 - HashMap提供O(1)查询
    edge_by_id: HashMap<i64, Edge>,
    /// 查询计数
    query_count: u64,
    /// 总查询时间（毫秒）
    total_query_time_ms: f64,
    /// 最后更新时间
    last_updated: i64,
}

impl IndexData {
    fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        Self {
            vertex_by_tag_property: BTreeMap::new(),
            vertex_by_id: HashMap::new(),
            edge_by_type_property: BTreeMap::new(),
            edge_by_id: HashMap::new(),
            query_count: 0,
            total_query_time_ms: 0.0,
            last_updated: now,
        }
    }

    /// 插入顶点到索引
    fn insert_vertex(
        &mut self,
        tag_name: &str,
        field_name: &str,
        field_value: &Value,
        vertex: Vertex,
    ) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        self.last_updated = now;
        
        let key = (
            tag_name.to_string(),
            field_name.to_string(),
            field_value.clone(),
        );
        self.vertex_by_tag_property
            .entry(key)
            .or_insert_with(Vec::new)
            .push(vertex.clone());
        self.vertex_by_id.insert(vertex.id(), vertex);
    }

    /// 插入边到索引
    fn insert_edge(&mut self, edge_type: &str, field_name: &str, field_value: &Value, edge: Edge) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        self.last_updated = now;
        
        let key = (
            edge_type.to_string(),
            field_name.to_string(),
            field_value.clone(),
        );
        self.edge_by_type_property
            .entry(key)
            .or_insert_with(Vec::new)
            .push(edge.clone());
        self.edge_by_id.insert(edge.id, edge);
    }

    /// 精确查找顶点 - 使用HashMap O(1)
    fn lookup_vertex_by_id(&self, id: i64) -> Option<&Vertex> {
        self.vertex_by_id.get(&id)
    }

    /// 精确查找边 - 使用HashMap O(1)
    fn lookup_edge_by_id(&self, id: i64) -> Option<&Edge> {
        self.edge_by_id.get(&id)
    }

    /// 按标签、属性和属性值精确查找顶点 - 使用BTreeMap
    fn lookup_vertex_by_property(
        &self,
        tag_name: &str,
        field_name: &str,
        field_value: &Value,
    ) -> Option<&Vec<Vertex>> {
        let key = (
            tag_name.to_string(),
            field_name.to_string(),
            field_value.clone(),
        );
        self.vertex_by_tag_property.get(&key)
    }

    /// 按边类型、属性和属性值精确查找边 - 使用BTreeMap
    fn lookup_edge_by_property(
        &self,
        edge_type: &str,
        field_name: &str,
        field_value: &Value,
    ) -> Option<&Vec<Edge>> {
        let key = (
            edge_type.to_string(),
            field_name.to_string(),
            field_value.clone(),
        );
        self.edge_by_type_property.get(&key)
    }

    /// 范围查找顶点 - BTreeMap天然支持范围迭代
    fn range_lookup_vertex(
        &self,
        tag_name: &str,
        field_name: &str,
        start_value: &Value,
        end_value: &Value,
    ) -> Vec<Vertex> {
        let start_key = (
            tag_name.to_string(),
            field_name.to_string(),
            start_value.clone(),
        );
        let end_key = (
            tag_name.to_string(),
            field_name.to_string(),
            end_value.clone(),
        );

        let mut result = Vec::new();
        for (_, vertices) in self.vertex_by_tag_property.range(start_key..=end_key) {
            result.extend(vertices.iter().cloned());
        }
        result
    }

    /// 范围查找边 - BTreeMap天然支持范围迭代
    fn range_lookup_edge(
        &self,
        edge_type: &str,
        field_name: &str,
        start_value: &Value,
        end_value: &Value,
    ) -> Vec<Edge> {
        let start_key = (
            edge_type.to_string(),
            field_name.to_string(),
            start_value.clone(),
        );
        let end_key = (
            edge_type.to_string(),
            field_name.to_string(),
            end_value.clone(),
        );

        let mut result = Vec::new();
        for (_, edges) in self.edge_by_type_property.range(start_key..=end_key) {
            result.extend(edges.iter().cloned());
        }
        result
    }

    /// 删除顶点从索引
    fn delete_vertex(&mut self, vertex: &Vertex) {
        let vertex_id = vertex.id();
        
        self.vertex_by_id.remove(&vertex_id);
        
        for tag in &vertex.tags {
            for (field_name, field_value) in &tag.properties {
                let key = (
                    tag.name.clone(),
                    field_name.clone(),
                    field_value.clone(),
                );
                if let Some(vertices) = self.vertex_by_tag_property.get_mut(&key) {
                    vertices.retain(|v| v.id() != vertex_id);
                }
            }
        }
    }

    /// 删除边从索引
    fn delete_edge(&mut self, edge: &Edge) {
        let edge_id = edge.id;
        
        self.edge_by_id.remove(&edge_id);
        
        for (field_name, field_value) in &edge.props {
            let key = (
                edge.edge_type.clone(),
                field_name.clone(),
                field_value.clone(),
            );
            if let Some(edges) = self.edge_by_type_property.get_mut(&key) {
                edges.retain(|e| e.id != edge_id);
            }
        }
    }

    /// 获取内存使用量（字节）
    fn get_memory_usage(&self) -> usize {
        let mut size = std::mem::size_of_val(self);
        
        for (_, vertices) in &self.vertex_by_tag_property {
            size += std::mem::size_of_val(vertices);
            for vertex in vertices {
                size += std::mem::size_of_val(vertex);
            }
        }
        
        for (_, vertex) in &self.vertex_by_id {
            size += std::mem::size_of_val(vertex);
        }
        
        for (_, edges) in &self.edge_by_type_property {
            size += std::mem::size_of_val(edges);
            for edge in edges {
                size += std::mem::size_of_val(edge);
            }
        }
        
        for (_, edge) in &self.edge_by_id {
            size += std::mem::size_of_val(edge);
        }
        
        size
    }

    /// 获取唯一条目数
    fn get_unique_entries(&self) -> usize {
        self.vertex_by_tag_property.len() + self.edge_by_type_property.len()
    }

    /// 获取总条目数
    fn get_total_entries(&self) -> usize {
        self.vertex_by_id.len() + self.edge_by_id.len()
    }

    /// 清空索引数据
    fn clear(&mut self) {
        self.vertex_by_tag_property.clear();
        self.vertex_by_id.clear();
        self.edge_by_type_property.clear();
        self.edge_by_id.clear();
        self.query_count = 0;
        self.total_query_time_ms = 0.0;
        self.last_updated = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
    }
}

/// 内存中的索引管理器实现
#[derive(Clone)]
pub struct MemoryIndexManager {
    indexes: Arc<RwLock<HashMap<String, Index>>>,
    next_index_id: Arc<RwLock<i32>>,
    storage_path: PathBuf,
    index_data: Arc<RwLock<HashMap<i32, IndexData>>>,
    storage_engine: Option<Arc<dyn StorageEngine>>,
    index_stats: Arc<RwLock<HashMap<i32, IndexStats>>>,
}

impl fmt::Debug for MemoryIndexManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MemoryIndexManager")
            .field("indexes", &self.indexes)
            .field("next_index_id", &self.next_index_id)
            .field("storage_path", &self.storage_path)
            .field("index_data", &self.index_data)
            .field("storage_engine", &"[redacted]")
            .field("index_stats", &self.index_stats)
            .finish()
    }
}

impl MemoryIndexManager {
    /// 创建新的内存索引管理器
    pub fn new(storage_path: PathBuf) -> Self {
        let manager = Self {
            indexes: Arc::new(RwLock::new(HashMap::new())),
            next_index_id: Arc::new(RwLock::new(1)),
            storage_path,
            index_data: Arc::new(RwLock::new(HashMap::new())),
            storage_engine: None,
            index_stats: Arc::new(RwLock::new(HashMap::new())),
        };
        let _ = manager.load_from_disk();
        manager
    }

    /// 创建新的内存索引管理器并设置存储引擎
    pub fn with_storage_engine(
        storage_path: PathBuf,
        storage_engine: Arc<dyn StorageEngine>,
    ) -> Self {
        let manager = Self {
            indexes: Arc::new(RwLock::new(HashMap::new())),
            next_index_id: Arc::new(RwLock::new(1)),
            storage_path,
            index_data: Arc::new(RwLock::new(HashMap::new())),
            storage_engine: Some(storage_engine),
            index_stats: Arc::new(RwLock::new(HashMap::new())),
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
        let mut indexes = self
            .indexes
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        indexes.insert(index.name.clone(), index);
        Ok(())
    }

    /// 删除索引
    pub fn remove_index(&self, name: &str) -> ManagerResult<()> {
        let mut indexes = self
            .indexes
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        indexes.remove(name);
        Ok(())
    }

    /// 更新索引
    pub fn update_index(&self, name: &str, index: Index) -> ManagerResult<()> {
        let mut indexes = self
            .indexes
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
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
        let mut indexes = self
            .indexes
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        if indexes.contains_key(&index.name) {
            return Err(ManagerError::AlreadyExists(format!(
                "索引 {} 已存在",
                index.name
            )));
        }

        let mut next_id = self
            .next_index_id
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        index.id = *next_id;
        index.space_id = space_id;
        index.status = IndexStatus::Active;
        *next_id += 1;

        indexes.insert(index.name.clone(), index.clone());
        drop(indexes);

        let _ = self.save_to_disk();

        Ok(index.id)
    }

    fn drop_index(&self, _space_id: i32, index_id: i32) -> ManagerResult<()> {
        let mut indexes = self
            .indexes
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

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
        indexes
            .values()
            .find(|idx| idx.id == index_id)
            .map(|idx| idx.status.clone())
    }

    fn list_indexes_by_space(&self, space_id: i32) -> ManagerResult<Vec<Index>> {
        let indexes = self
            .indexes
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        Ok(indexes
            .values()
            .filter(|idx| idx.space_id == space_id)
            .cloned()
            .collect())
    }

    fn load_from_disk(&self) -> ManagerResult<()> {
        use std::fs;

        if !self.storage_path.exists() {
            fs::create_dir_all(&self.storage_path)
                .map_err(|e| ManagerError::StorageError(e.to_string()))?;
            return Ok(());
        }

        let index_file = self.storage_path.join("indexes.json");
        if !index_file.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&index_file)
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        let loaded_indexes: Vec<Index> = serde_json::from_str(&content)
            .map_err(|e| ManagerError::IndexError(format!("反序列化索引失败: {}", e)))?;

        let mut indexes = self
            .indexes
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let mut next_id = self
            .next_index_id
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

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
            fs::create_dir_all(&self.storage_path)
                .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        }

        let indexes = self
            .indexes
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let index_list: Vec<Index> = indexes.values().cloned().collect();

        let content = serde_json::to_string_pretty(&index_list)
            .map_err(|e| ManagerError::IndexError(format!("序列化索引失败: {}", e)))?;

        let index_file = self.storage_path.join("indexes.json");
        fs::write(&index_file, content).map_err(|e| ManagerError::StorageError(e.to_string()))?;

        Ok(())
    }

    fn lookup_vertex_by_index(
        &self,
        _space_id: i32,
        index_name: &str,
        values: &[Value],
    ) -> ManagerResult<Vec<Vertex>> {
        let indexes = self
            .indexes
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let index = indexes
            .get(index_name)
            .ok_or_else(|| ManagerError::NotFound(format!("索引 {} 不存在", index_name)))?;

        if index.status != IndexStatus::Active {
            return Err(ManagerError::IndexError(format!(
                "索引 {} 状态为 {:?}，不可用",
                index_name, index.status
            )));
        }

        let index_data = self
            .index_data
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let data = index_data
            .get(&index.id)
            .ok_or_else(|| ManagerError::NotFound(format!("索引数据 {} 不存在", index_name)))?;

        let field_name = index
            .fields
            .get(0)
            .ok_or_else(|| ManagerError::IndexError("索引字段为空".to_string()))?;

        let field_value = values
            .get(0)
            .ok_or_else(|| ManagerError::IndexError("索引值为空".to_string()))?;

        let vertices = data
            .lookup_vertex_by_property(&index.schema_name, field_name, field_value)
            .ok_or_else(|| ManagerError::NotFound("未找到匹配的顶点".to_string()))?;

        Ok(vertices.clone())
    }

    fn lookup_edge_by_index(
        &self,
        _space_id: i32,
        index_name: &str,
        values: &[Value],
    ) -> ManagerResult<Vec<Edge>> {
        let indexes = self
            .indexes
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let index = indexes
            .get(index_name)
            .ok_or_else(|| ManagerError::NotFound(format!("索引 {} 不存在", index_name)))?;

        if index.status != IndexStatus::Active {
            return Err(ManagerError::IndexError(format!(
                "索引 {} 状态为 {:?}，不可用",
                index_name, index.status
            )));
        }

        let index_data = self
            .index_data
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let data = index_data
            .get(&index.id)
            .ok_or_else(|| ManagerError::NotFound(format!("索引数据 {} 不存在", index_name)))?;

        let field_name = index
            .fields
            .get(0)
            .ok_or_else(|| ManagerError::IndexError("索引字段为空".to_string()))?;

        let field_value = values
            .get(0)
            .ok_or_else(|| ManagerError::IndexError("索引值为空".to_string()))?;

        let edges = data
            .lookup_edge_by_property(&index.schema_name, field_name, field_value)
            .ok_or_else(|| ManagerError::NotFound("未找到匹配的边".to_string()))?;

        Ok(edges.clone())
    }

    fn range_lookup_vertex(
        &self,
        _space_id: i32,
        index_name: &str,
        start: &Value,
        end: &Value,
    ) -> ManagerResult<Vec<Vertex>> {
        let indexes = self
            .indexes
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let index = indexes
            .get(index_name)
            .ok_or_else(|| ManagerError::NotFound(format!("索引 {} 不存在", index_name)))?;

        if index.status != IndexStatus::Active {
            return Err(ManagerError::IndexError(format!(
                "索引 {} 状态为 {:?}，不可用",
                index_name, index.status
            )));
        }

        let index_data = self
            .index_data
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let data = index_data
            .get(&index.id)
            .ok_or_else(|| ManagerError::NotFound(format!("索引数据 {} 不存在", index_name)))?;

        let field_name = index
            .fields
            .get(0)
            .ok_or_else(|| ManagerError::IndexError("索引字段为空".to_string()))?;

        let vertices = data.range_lookup_vertex(&index.schema_name, field_name, start, end);

        Ok(vertices)
    }

    fn range_lookup_edge(
        &self,
        _space_id: i32,
        index_name: &str,
        start: &Value,
        end: &Value,
    ) -> ManagerResult<Vec<Edge>> {
        let indexes = self
            .indexes
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let index = indexes
            .get(index_name)
            .ok_or_else(|| ManagerError::NotFound(format!("索引 {} 不存在", index_name)))?;

        if index.status != IndexStatus::Active {
            return Err(ManagerError::IndexError(format!(
                "索引 {} 状态为 {:?}，不可用",
                index_name, index.status
            )));
        }

        let index_data = self
            .index_data
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let data = index_data
            .get(&index.id)
            .ok_or_else(|| ManagerError::NotFound(format!("索引数据 {} 不存在", index_name)))?;

        let field_name = index
            .fields
            .get(0)
            .ok_or_else(|| ManagerError::IndexError("索引字段为空".to_string()))?;

        let edges = data.range_lookup_edge(&index.schema_name, field_name, start, end);

        Ok(edges)
    }

    fn insert_vertex_to_index(&self, _space_id: i32, vertex: &Vertex) -> ManagerResult<()> {
        let indexes = self
            .indexes
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        for index in indexes.values() {
            if index.status != IndexStatus::Active {
                continue;
            }

            if index.index_type != IndexType::TagIndex {
                continue;
            }

            if let Some(tag) = vertex.tags.iter().find(|t| t.name == index.schema_name) {
                for field_name in &index.fields {
                    if let Some(field_value) = tag.properties.get(field_name) {
                        let mut index_data = self
                            .index_data
                            .write()
                            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
                        let data = index_data.entry(index.id).or_insert_with(IndexData::new);
                        data.insert_vertex(
                            &index.schema_name,
                            field_name,
                            field_value,
                            vertex.clone(),
                        );
                    }
                }
            }
        }

        Ok(())
    }

    fn delete_vertex_from_index(&self, _space_id: i32, vertex: &Vertex) -> ManagerResult<()> {
        let indexes = self
            .indexes
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        for index in indexes.values() {
            if index.status != IndexStatus::Active {
                continue;
            }

            if index.index_type != IndexType::TagIndex {
                continue;
            }

            let mut index_data = self
                .index_data
                .write()
                .map_err(|e| ManagerError::StorageError(e.to_string()))?;
            if let Some(data) = index_data.get_mut(&index.id) {
                data.delete_vertex(vertex);
            }
        }

        Ok(())
    }

    fn update_vertex_in_index(
        &self,
        space_id: i32,
        old_vertex: &Vertex,
        new_vertex: &Vertex,
    ) -> ManagerResult<()> {
        self.delete_vertex_from_index(space_id, old_vertex)?;
        self.insert_vertex_to_index(space_id, new_vertex)?;
        Ok(())
    }

    fn insert_edge_to_index(&self, _space_id: i32, edge: &Edge) -> ManagerResult<()> {
        let indexes = self
            .indexes
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        for index in indexes.values() {
            if index.status != IndexStatus::Active {
                continue;
            }

            if index.index_type != IndexType::EdgeIndex {
                continue;
            }

            if index.schema_name == edge.edge_type {
                for field_name in &index.fields {
                    if let Some(field_value) = edge.props.get(field_name) {
                        let mut index_data = self
                            .index_data
                            .write()
                            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
                        let data = index_data.entry(index.id).or_insert_with(IndexData::new);
                        data.insert_edge(&index.schema_name, field_name, field_value, edge.clone());
                    }
                }
            }
        }

        Ok(())
    }

    fn delete_edge_from_index(&self, _space_id: i32, edge: &Edge) -> ManagerResult<()> {
        let indexes = self
            .indexes
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        for index in indexes.values() {
            if index.status != IndexStatus::Active {
                continue;
            }

            if index.index_type != IndexType::EdgeIndex {
                continue;
            }

            let mut index_data = self
                .index_data
                .write()
                .map_err(|e| ManagerError::StorageError(e.to_string()))?;
            if let Some(data) = index_data.get_mut(&index.id) {
                data.delete_edge(edge);
            }
        }

        Ok(())
    }

    fn update_edge_in_index(
        &self,
        space_id: i32,
        old_edge: &Edge,
        new_edge: &Edge,
    ) -> ManagerResult<()> {
        self.delete_edge_from_index(space_id, old_edge)?;
        self.insert_edge_to_index(space_id, new_edge)?;
        Ok(())
    }

    fn rebuild_index(&self, space_id: i32, index_id: i32) -> ManagerResult<()> {
        let indexes = self
            .indexes
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let index = indexes
            .values()
            .find(|idx| idx.id == index_id)
            .ok_or_else(|| ManagerError::NotFound(format!("索引ID {} 不存在", index_id)))?;

        if index.space_id != space_id {
            return Err(ManagerError::InvalidInput(format!(
                "索引 {} 不属于空间 {}",
                index_id, space_id
            )));
        }

        drop(indexes);

        let storage_engine = self
            .storage_engine
            .as_ref()
            .ok_or_else(|| ManagerError::StorageError("存储引擎未设置".to_string()))?;

        let mut index_data = self
            .index_data
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        
        let data = index_data.entry(index_id).or_insert_with(IndexData::new);
        data.clear();

        match index.index_type {
            IndexType::TagIndex => {
                if let Ok(Some(space_info)) = storage_engine.get_space(space_id) {
                    for tag_def in &space_info.tags {
                        if tag_def.tag_name == index.schema_name {
                            let vertices = storage_engine
                                .scan_vertices()
                                .map_err(|e| ManagerError::StorageError(e.to_string()))?;
                            
                            for vertex in vertices {
                                if let Some(tag) = vertex.tags.iter().find(|t| t.name == index.schema_name) {
                                    for field_name in &index.fields {
                                        if let Some(field_value) = tag.properties.get(field_name) {
                                            data.insert_vertex(
                                                &index.schema_name,
                                                field_name,
                                                field_value,
                                                vertex.clone(),
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            IndexType::EdgeIndex => {
                let edges = storage_engine
                    .scan_edges()
                    .map_err(|e| ManagerError::StorageError(e.to_string()))?;
                
                for edge in edges {
                    if edge.edge_type == index.schema_name {
                        for field_name in &index.fields {
                            if let Some(field_value) = edge.props.get(field_name) {
                                data.insert_edge(
                                    &index.schema_name,
                                    field_name,
                                    field_value,
                                    edge.clone(),
                                );
                            }
                        }
                    }
                }
            }
            IndexType::FulltextIndex => {
                return Err(ManagerError::IndexError(
                    "全文索引暂不支持重建".to_string(),
                ));
            }
        }

        Ok(())
    }

    fn rebuild_all_indexes(&self, space_id: i32) -> ManagerResult<()> {
        let indexes = self
            .indexes
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let space_indexes: Vec<i32> = indexes
            .values()
            .filter(|idx| idx.space_id == space_id)
            .map(|idx| idx.id)
            .collect();
        drop(indexes);

        for index_id in space_indexes {
            self.rebuild_index(space_id, index_id)?;
        }

        Ok(())
    }

    fn get_index_stats(&self, space_id: i32, index_id: i32) -> ManagerResult<IndexStats> {
        let indexes = self
            .indexes
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let index = indexes
            .values()
            .find(|idx| idx.id == index_id)
            .ok_or_else(|| ManagerError::NotFound(format!("索引ID {} 不存在", index_id)))?;

        if index.space_id != space_id {
            return Err(ManagerError::InvalidInput(format!(
                "索引 {} 不属于空间 {}",
                index_id, space_id
            )));
        }

        let index_data = self
            .index_data
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        
        let data = index_data
            .get(&index_id)
            .ok_or_else(|| ManagerError::NotFound(format!("索引数据 {} 不存在", index_id)))?;

        let avg_query_time = if data.query_count > 0 {
            data.total_query_time_ms / data.query_count as f64
        } else {
            0.0
        };

        let stats = IndexStats {
            index_id,
            index_name: index.name.clone(),
            total_entries: data.get_total_entries(),
            unique_entries: data.get_unique_entries(),
            last_updated: data.last_updated,
            memory_usage_bytes: data.get_memory_usage(),
            query_count: data.query_count,
            avg_query_time_ms: avg_query_time,
        };

        Ok(stats)
    }

    fn get_all_index_stats(&self, space_id: i32) -> ManagerResult<Vec<IndexStats>> {
        let indexes = self
            .indexes
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let space_indexes: Vec<Index> = indexes
            .values()
            .filter(|idx| idx.space_id == space_id)
            .cloned()
            .collect();
        drop(indexes);

        let mut stats = Vec::new();
        for index in space_indexes {
            stats.push(self.get_index_stats(space_id, index.id)?);
        }

        Ok(stats)
    }

    fn analyze_index(&self, space_id: i32, index_id: i32) -> ManagerResult<IndexOptimization> {
        let stats = self.get_index_stats(space_id, index_id)?;
        let indexes = self
            .indexes
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let index = indexes
            .values()
            .find(|idx| idx.id == index_id)
            .ok_or_else(|| ManagerError::NotFound(format!("索引ID {} 不存在", index_id)))?;

        let mut suggestions = Vec::new();
        let mut priority = "low".to_string();

        if stats.total_entries == 0 {
            suggestions.push("索引为空，考虑删除".to_string());
            priority = "medium".to_string();
        }

        if stats.unique_entries < stats.total_entries / 2 {
            suggestions.push(format!(
                "索引重复率较高（{:.1}%），考虑优化索引字段",
                (1.0 - stats.unique_entries as f64 / stats.total_entries as f64) * 100.0
            ));
            priority = "high".to_string();
        }

        if stats.memory_usage_bytes > 100 * 1024 * 1024 {
            suggestions.push(format!(
                "索引内存占用较大（{} MB），考虑使用更高效的索引结构",
                stats.memory_usage_bytes / (1024 * 1024)
            ));
            priority = "high".to_string();
        }

        if stats.query_count > 1000 && stats.avg_query_time_ms > 10.0 {
            suggestions.push(format!(
                "索引查询性能较低（平均 {:.2} ms），考虑重建索引",
                stats.avg_query_time_ms
            ));
            priority = "high".to_string();
        }

        if index.fields.len() > 3 {
            suggestions.push("索引字段较多，考虑拆分为多个单字段索引".to_string());
            priority = "medium".to_string();
        }

        if suggestions.is_empty() {
            suggestions.push("索引状态良好，无需优化".to_string());
        }

        Ok(IndexOptimization {
            index_id,
            index_name: index.name.clone(),
            suggestions,
            priority,
        })
    }

    fn analyze_all_indexes(&self, space_id: i32) -> ManagerResult<Vec<IndexOptimization>> {
        let indexes = self
            .indexes
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let space_indexes: Vec<i32> = indexes
            .values()
            .filter(|idx| idx.space_id == space_id)
            .map(|idx| idx.id)
            .collect();
        drop(indexes);

        let mut optimizations = Vec::new();
        for index_id in space_indexes {
            optimizations.push(self.analyze_index(space_id, index_id)?);
        }

        Ok(optimizations)
    }

    fn check_index_consistency(&self, space_id: i32, index_id: i32) -> ManagerResult<bool> {
        let indexes = self
            .indexes
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let index = indexes
            .values()
            .find(|idx| idx.id == index_id)
            .ok_or_else(|| ManagerError::NotFound(format!("索引ID {} 不存在", index_id)))?;

        if index.space_id != space_id {
            return Err(ManagerError::InvalidInput(format!(
                "索引 {} 不属于空间 {}",
                index_id, space_id
            )));
        }

        let index_data = self
            .index_data
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        
        let data = index_data
            .get(&index_id)
            .ok_or_else(|| ManagerError::NotFound(format!("索引数据 {} 不存在", index_id)))?;

        let storage_engine = self
            .storage_engine
            .as_ref()
            .ok_or_else(|| ManagerError::StorageError("存储引擎未设置".to_string()))?;

        match index.index_type {
            IndexType::TagIndex => {
                for (key, vertices) in &data.vertex_by_tag_property {
                    for vertex in vertices {
                        if let Ok(Some(stored_vertex)) = storage_engine.get_node(&vertex.id()) {
                            if stored_vertex.id() != vertex.id() {
                                return Ok(false);
                            }
                        } else {
                            return Ok(false);
                        }
                    }
                }
            }
            IndexType::EdgeIndex => {
                for (key, edges) in &data.edge_by_type_property {
                    for edge in edges {
                        if let Ok(Some(stored_edge)) = storage_engine.get_edge(edge.id) {
                            if stored_edge.id != edge.id {
                                return Ok(false);
                            }
                        } else {
                            return Ok(false);
                        }
                    }
                }
            }
            IndexType::FulltextIndex => {
                return Ok(true);
            }
        }

        Ok(true)
    }

    fn repair_index(&self, space_id: i32, index_id: i32) -> ManagerResult<()> {
        self.rebuild_index(space_id, index_id)
    }

    fn cleanup_index(&self, space_id: i32, index_id: i32) -> ManagerResult<()> {
        let indexes = self
            .indexes
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        let index = indexes
            .values()
            .find(|idx| idx.id == index_id)
            .ok_or_else(|| ManagerError::NotFound(format!("索引ID {} 不存在", index_id)))?;

        if index.space_id != space_id {
            return Err(ManagerError::InvalidInput(format!(
                "索引 {} 不属于空间 {}",
                index_id, space_id
            )));
        }

        let mut index_data = self
            .index_data
            .write()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;
        
        if let Some(data) = index_data.get_mut(&index_id) {
            data.clear();
        }

        Ok(())
    }

    fn batch_insert_vertices(&self, _space_id: i32, vertices: &[Vertex]) -> ManagerResult<()> {
        let indexes = self
            .indexes
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        for vertex in vertices {
            for index in indexes.values() {
                if index.status != IndexStatus::Active {
                    continue;
                }

                if index.index_type != IndexType::TagIndex {
                    continue;
                }

                if let Some(tag) = vertex.tags.iter().find(|t| t.name == index.schema_name) {
                    for field_name in &index.fields {
                        if let Some(field_value) = tag.properties.get(field_name) {
                            let mut index_data = self
                                .index_data
                                .write()
                                .map_err(|e| ManagerError::StorageError(e.to_string()))?;
                            let data = index_data.entry(index.id).or_insert_with(IndexData::new);
                            data.insert_vertex(
                                &index.schema_name,
                                field_name,
                                field_value,
                                vertex.clone(),
                            );
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn batch_delete_vertices(&self, _space_id: i32, vertices: &[Vertex]) -> ManagerResult<()> {
        let indexes = self
            .indexes
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        for vertex in vertices {
            for index in indexes.values() {
                if index.status != IndexStatus::Active {
                    continue;
                }

                if index.index_type != IndexType::TagIndex {
                    continue;
                }

                let mut index_data = self
                    .index_data
                    .write()
                    .map_err(|e| ManagerError::StorageError(e.to_string()))?;
                if let Some(data) = index_data.get_mut(&index.id) {
                    data.delete_vertex(vertex);
                }
            }
        }

        Ok(())
    }

    fn batch_insert_edges(&self, _space_id: i32, edges: &[Edge]) -> ManagerResult<()> {
        let indexes = self
            .indexes
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        for edge in edges {
            for index in indexes.values() {
                if index.status != IndexStatus::Active {
                    continue;
                }

                if index.index_type != IndexType::EdgeIndex {
                    continue;
                }

                if index.schema_name == edge.edge_type {
                    for field_name in &index.fields {
                        if let Some(field_value) = edge.props.get(field_name) {
                            let mut index_data = self
                                .index_data
                                .write()
                                .map_err(|e| ManagerError::StorageError(e.to_string()))?;
                            let data = index_data.entry(index.id).or_insert_with(IndexData::new);
                            data.insert_edge(&index.schema_name, field_name, field_value, edge.clone());
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn batch_delete_edges(&self, _space_id: i32, edges: &[Edge]) -> ManagerResult<()> {
        let indexes = self
            .indexes
            .read()
            .map_err(|e| ManagerError::StorageError(e.to_string()))?;

        for edge in edges {
            for index in indexes.values() {
                if index.status != IndexStatus::Active {
                    continue;
                }

                if index.index_type != IndexType::EdgeIndex {
                    continue;
                }

                let mut index_data = self
                    .index_data
                    .write()
                    .map_err(|e| ManagerError::StorageError(e.to_string()))?;
                if let Some(data) = index_data.get_mut(&index.id) {
                    data.delete_edge(edge);
                }
            }
        }

        Ok(())
    }
}

impl MemoryIndexManager {
    /// 检查顶点是否匹配给定的值
    fn vertex_matches_values(
        vertex: &Vertex,
        tag_name: &str,
        fields: &[String],
        values: &[Value],
    ) -> bool {
        if let Some(tag) = vertex.tags.iter().find(|t| t.name == tag_name) {
            for (field_name, expected_value) in fields.iter().zip(values.iter()) {
                if let Some(actual_value) = tag.properties.get(field_name) {
                    if actual_value != expected_value {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }

    /// 检查边是否匹配给定的值
    fn edge_matches_values(
        edge: &Edge,
        edge_type: &str,
        fields: &[String],
        values: &[Value],
    ) -> bool {
        if edge.edge_type != edge_type {
            return false;
        }

        for (field_name, expected_value) in fields.iter().zip(values.iter()) {
            if let Some(actual_value) = edge.props.get(field_name) {
                if actual_value != expected_value {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }

    /// 从顶点提取字段值 - 简化版本，无需二进制编码
    fn extract_vertex_field_value(
        vertex: &Vertex,
        tag_name: &str,
        field_name: &str,
    ) -> Option<(Value, String)> {
        if let Some(tag) = vertex.tags.iter().find(|t| t.name == tag_name) {
            if let Some(value) = tag.properties.get(field_name) {
                return Some((value.clone(), field_name.to_string()));
            }
        }
        None
    }

    /// 从边提取字段值 - 简化版本，无需二进制编码
    fn extract_edge_field_value(
        edge: &Edge,
        edge_type: &str,
        field_name: &str,
    ) -> Option<(Value, String)> {
        if edge.edge_type == edge_type {
            if let Some(value) = edge.props.get(field_name) {
                return Some((value.clone(), field_name.to_string()));
            }
        }
        None
    }
}
