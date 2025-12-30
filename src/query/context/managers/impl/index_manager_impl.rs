//! 索引管理器实现 - 内存中的索引管理
//!
//! 采用 BTreeMap + HashMap 混合索引策略：
//! - BTreeMap: 支持范围查询和排序
//! - HashMap: 支持精确匹配的快速查找

use super::super::{Index, IndexManager, IndexStatus, IndexType};
use crate::core::error::{ManagerError, ManagerResult};
use crate::core::{Edge, Value, Vertex};
use crate::storage::StorageEngine;
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

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
}

impl IndexData {
    fn new() -> Self {
        Self {
            vertex_by_tag_property: BTreeMap::new(),
            vertex_by_id: HashMap::new(),
            edge_by_type_property: BTreeMap::new(),
            edge_by_id: HashMap::new(),
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
}

/// 内存中的索引管理器实现
#[derive(Clone)]
pub struct MemoryIndexManager {
    indexes: Arc<RwLock<HashMap<String, Index>>>,
    next_index_id: Arc<RwLock<i32>>,
    storage_path: PathBuf,
    index_data: Arc<RwLock<HashMap<i32, IndexData>>>,
    storage_engine: Option<Arc<dyn StorageEngine>>,
}

impl fmt::Debug for MemoryIndexManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MemoryIndexManager")
            .field("indexes", &self.indexes)
            .field("next_index_id", &self.next_index_id)
            .field("storage_path", &self.storage_path)
            .field("index_data", &self.index_data)
            .field("storage_engine", &"[redacted]")
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
                data.vertex_by_id.remove(&vertex.id());
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
                data.edge_by_id.remove(&edge.id);
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
