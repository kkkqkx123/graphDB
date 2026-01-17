//! 索引服务模块
//!
//! 提供统一的索引服务层，整合内存索引缓存和持久化存储
//!
//! 功能：
//! - 索引创建、删除、查询
//! - 内存索引缓存
//! - 与持久化存储层集成

use crate::core::{Value, Vertex};
use crate::index::{ConcurrentIndexStorage, Index, IndexField, IndexInfo, IndexType};
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// 索引服务错误类型
#[derive(Debug, thiserror::Error)]
pub enum IndexServiceError {
    #[error("索引不存在: {0}")]
    NotFound(i32),
    #[error("索引名称已存在: {0}")]
    NameExists(String),
    #[error("存储错误: {0}")]
    StorageError(String),
    #[error("参数错误: {0}")]
    InvalidParameter(String),
    #[error("索引操作失败: {0}")]
    OperationFailed(String),
}

/// 索引服务配置
#[derive(Clone, Debug)]
pub struct IndexServiceConfig {
    pub max_memory_bytes: u64,
    pub enable_auto_cleanup: bool,
    pub cleanup_interval_secs: u64,
}

impl Default for IndexServiceConfig {
    fn default() -> Self {
        Self {
            max_memory_bytes: 1024 * 1024 * 1024, // 1GB
            enable_auto_cleanup: true,
            cleanup_interval_secs: 300, // 5分钟
        }
    }
}

pub struct IndexService {
    space_id: i32,
    index_by_id: DashMap<i32, Arc<ConcurrentIndexStorage>>,
    index_by_name: DashMap<String, i32>,
    index_metadata: DashMap<i32, Index>,
    next_index_id: AtomicU64,
    config: IndexServiceConfig,
}

impl Default for IndexService {
    fn default() -> Self {
        Self::new_with_config(0, IndexServiceConfig::default())
    }
}

impl IndexService {
    pub fn new(space_id: i32) -> Self {
        Self::new_with_config(space_id, IndexServiceConfig::default())
    }

    pub fn new_with_config(space_id: i32, config: IndexServiceConfig) -> Self {
        Self {
            space_id,
            index_by_id: DashMap::new(),
            index_by_name: DashMap::new(),
            index_metadata: DashMap::new(),
            next_index_id: AtomicU64::new(1),
            config,
        }
    }

    pub fn create_index(
        &self,
        name: &str,
        schema_name: &str,
        fields: Vec<IndexField>,
        index_type: IndexType,
        is_unique: bool,
    ) -> Result<i32, IndexServiceError> {
        // 验证参数
        if name.trim().is_empty() {
            return Err(IndexServiceError::InvalidParameter("索引名称不能为空".to_string()));
        }
        if schema_name.trim().is_empty() {
            return Err(IndexServiceError::InvalidParameter("模式名称不能为空".to_string()));
        }
        if fields.is_empty() {
            return Err(IndexServiceError::InvalidParameter("索引字段不能为空".to_string()));
        }

        // 检查名称是否已存在
        if self.index_by_name.contains_key(name) {
            return Err(IndexServiceError::NameExists(name.to_string()));
        }

        let index_id = self.next_index_id.fetch_add(1, Ordering::Relaxed) as i32;
        
        // 创建存储
        let storage = Arc::new(ConcurrentIndexStorage::new(self.space_id, index_id, name.to_string()));

        // 原子性操作：确保所有映射都成功
        self.index_by_id.insert(index_id, Arc::clone(&storage));
        self.index_by_name.insert(name.to_string(), index_id);

        // 创建索引元数据
        let index = Index::new(
            index_id,
            name.to_string(),
            self.space_id,
            schema_name.to_string(),
            fields,
            index_type,
            is_unique,
        );
        self.index_metadata.insert(index_id, index);

        Ok(index_id)
    }

    pub fn drop_index(&self, index_id: i32) -> Result<(), IndexServiceError> {
        if let Some((_, _)) = self.index_by_id.remove(&index_id) {
            if let Some(metadata) = self.index_metadata.get(&index_id) {
                self.index_by_name.remove(&metadata.name);
            }
            self.index_metadata.remove(&index_id);
            Ok(())
        } else {
            Err(IndexServiceError::NotFound(index_id))
        }
    }

    pub fn get_index(&self, index_id: i32) -> Result<Index, IndexServiceError> {
        self.index_metadata
            .get(&index_id)
            .map(|i| i.clone())
            .ok_or_else(|| IndexServiceError::NotFound(index_id))
    }

    pub fn get_index_by_name(&self, name: &str) -> Result<Index, IndexServiceError> {
        let index_id = self.index_by_name
            .get(name)
            .ok_or_else(|| IndexServiceError::NotFound(-1))?;
        
        self.get_index(*index_id)
    }

    pub fn list_indexes(&self) -> Vec<Index> {
        self.index_metadata
            .iter()
            .map(|i| i.value().clone())
            .collect()
    }

    pub fn exact_lookup(&self, index_name: &str, value: &Value) -> Result<Vec<Vertex>, IndexServiceError> {
        let index_id = self.index_by_name
            .get(index_name)
            .ok_or_else(|| IndexServiceError::NotFound(-1))?;
        
        let storage = self.index_by_id
            .get(&index_id)
            .ok_or_else(|| IndexServiceError::NotFound(*index_id))?;
        
        storage.exact_lookup("", value)
            .map(|(vertices, _, _)| vertices)
            .map_err(|e| IndexServiceError::StorageError(e.to_string()))
    }

    pub fn prefix_lookup(&self, index_name: &str, prefix: &[Value]) -> Result<Vec<Vertex>, IndexServiceError> {
        let index_id = self.index_by_name
            .get(index_name)
            .ok_or_else(|| IndexServiceError::NotFound(-1))?;
        
        let storage = self.index_by_id
            .get(&index_id)
            .ok_or_else(|| IndexServiceError::NotFound(*index_id))?;
        
        storage.prefix_lookup("", prefix)
            .map(|(vertices, _, _)| vertices)
            .map_err(|e| IndexServiceError::StorageError(e.to_string()))
    }

    pub fn range_lookup(&self, index_name: &str, start: &Value, end: &Value) -> Result<Vec<Vertex>, IndexServiceError> {
        let index_id = self.index_by_name
            .get(index_name)
            .ok_or_else(|| IndexServiceError::NotFound(-1))?;
        
        let storage = self.index_by_id
            .get(&index_id)
            .ok_or_else(|| IndexServiceError::NotFound(*index_id))?;
        
        storage.range_lookup("", start, end)
            .map(|(vertices, _, _)| vertices)
            .map_err(|e| IndexServiceError::StorageError(e.to_string()))
    }

    pub fn insert_vertex(&self, index_name: &str, vertex: Vertex) -> Result<(), IndexServiceError> {
        let index_id = self.index_by_name
            .get(index_name)
            .ok_or_else(|| IndexServiceError::NotFound(-1))?;
        
        let storage = self.index_by_id
            .get(&index_id)
            .ok_or_else(|| IndexServiceError::NotFound(*index_id))?;
        
        storage.insert_vertex("", &Value::Int(vertex.id), vertex);
        Ok(())
    }

    pub fn delete_vertex(&self, index_name: &str, vertex: &Vertex) -> Result<(), IndexServiceError> {
        let index_id = self.index_by_name
            .get(index_name)
            .ok_or_else(|| IndexServiceError::NotFound(-1))?;
        
        let storage = self.index_by_id
            .get(&index_id)
            .ok_or_else(|| IndexServiceError::NotFound(*index_id))?;
        
        storage.delete_vertex(vertex);
        Ok(())
    }

    pub fn get_stats(&self, index_id: i32) -> Result<IndexInfo, IndexServiceError> {
        let index = self.get_index(index_id)?;
        let mut info = IndexInfo::new(index_id, index.name.clone());
        
        if let Some(storage) = self.index_by_id.get(&index_id) {
            info.total_entries = storage.get_entry_count();
            info.memory_usage_bytes = storage.get_memory_usage();
        }
        
        Ok(info)
    }

    pub fn get_all_stats(&self) -> Vec<IndexInfo> {
        self.index_metadata
            .iter()
            .map(|i| {
                let index = i.value();
                let mut info = IndexInfo::new(index.id, index.name.clone());
                if let Some(storage) = self.index_by_id.get(&index.id) {
                    info.total_entries = storage.get_entry_count();
                    info.memory_usage_bytes = storage.get_memory_usage();
                }
                info
            })
            .collect()
    }

    /// 创建标签索引的便捷方法
    pub fn create_tag_index(
        &self,
        name: &str,
        tag_name: &str,
        fields: Vec<String>,
        is_unique: bool,
    ) -> Result<i32, IndexServiceError> {
        let index_fields: Vec<IndexField> = fields
            .into_iter()
            .map(|field_name| {
                IndexField::new(field_name, Value::String("string".to_string()), false)
            })
            .collect();

        self.create_index(name, tag_name, index_fields, IndexType::TagIndex, is_unique)
    }

    /// 创建边索引的便捷方法
    pub fn create_edge_index(
        &self,
        name: &str,
        edge_type_name: &str,
        fields: Vec<String>,
        is_unique: bool,
    ) -> Result<i32, IndexServiceError> {
        let index_fields: Vec<IndexField> = fields
            .into_iter()
            .map(|field_name| {
                IndexField::new(field_name, Value::String("string".to_string()), false)
            })
            .collect();

        self.create_index(name, edge_type_name, index_fields, IndexType::EdgeIndex, is_unique)
    }

    /// 获取所有索引列表
    pub fn list_indexes(&self) -> Vec<Index> {
        self.index_metadata
            .iter()
            .map(|i| i.value().clone())
            .collect()
    }

    /// 检查索引是否存在
    pub fn index_exists(&self, index_id: i32) -> bool {
        self.index_metadata.contains_key(&index_id)
    }

    /// 检查索引名称是否存在
    pub fn index_name_exists(&self, name: &str) -> bool {
        self.index_by_name.contains_key(name)
    }

    /// 获取配置信息
    pub fn get_config(&self) -> &IndexServiceConfig {
        &self.config
    }

    /// 获取空间ID
    pub fn get_space_id(&self) -> i32 {
        self.space_id
    }

    /// 创建边索引的便捷方法
    pub fn create_edge_index(
        &self,
        name: &str,
        edge_type_name: &str,
        fields: Vec<String>,
        is_unique: bool,
    ) -> i32 {
        let index_fields: Vec<IndexField> = fields
            .into_iter()
            .map(|field_name| {
                IndexField::new(field_name, Value::String("string".to_string()), false)
            })
            .collect();

        self.create_index(name, edge_type_name, index_fields, is_unique)
    }

    /// 检查标签索引是否存在
    pub fn tag_index_exists(&self, index_name: &str) -> bool {
        self.get_index_by_name(index_name).is_some()
    }

    /// 检查边索引是否存在
    pub fn edge_index_exists(&self, index_name: &str) -> bool {
        self.get_index_by_name(index_name).is_some()
    }

    /// 获取指定标签的所有索引
    pub fn get_tag_indexes_for_tag(&self, tag_name: &str) -> Vec<Index> {
        self.list_indexes()
            .into_iter()
            .filter(|index| index.schema_name == tag_name && matches!(index.index_type, crate::index::IndexType::TagIndex))
            .collect()
    }

    /// 获取指定边类型的所有索引
    pub fn get_edge_indexes_for_edge_type(&self, edge_type_name: &str) -> Vec<Index> {
        self.list_indexes()
            .into_iter()
            .filter(|index| index.schema_name == edge_type_name && matches!(index.index_type, crate::index::IndexType::EdgeIndex))
            .collect()
    }
}

pub struct MemoryIndexCache {
    label_cache: DashMap<String, Vec<Value>>,
    property_cache: DashMap<String, HashMap<Value, Vec<Value>>>,
    access_count: DashMap<String, AtomicU64>,
    last_accessed: DashMap<String, Instant>,
}

impl Default for MemoryIndexCache {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryIndexCache {
    pub fn new() -> Self {
        Self {
            label_cache: DashMap::new(),
            property_cache: DashMap::new(),
            access_count: DashMap::new(),
            last_accessed: DashMap::new(),
        }
    }

    pub fn cache_by_label(&self, label: &str, node_ids: Vec<Value>) {
        let mut entry = self.label_cache.entry(label.to_string()).or_default();
        *entry = node_ids;
        self.update_access(label);
    }

    pub fn get_by_label(&self, label: &str) -> Option<Vec<Value>> {
        self.update_access(label);
        self.label_cache.get(label).map(|e| e.clone())
    }

    pub fn cache_by_property(&self, property: &str, value: &Value, node_ids: Vec<Value>) {
        let key = format!("{}.{}", property, value_to_string(value));
        let mut entry = self.property_cache.entry(property.to_string()).or_default();
        entry.insert(value.clone(), node_ids);
        self.update_access(&key);
    }

    pub fn get_by_property(&self, property: &str, value: &Value) -> Option<Vec<Value>> {
        let key = format!("{}.{}", property, value_to_string(value));
        self.update_access(&key);
        self.property_cache
            .get(property)
            .and_then(|p| p.get(value).map(|ids| ids.clone()))
    }

    fn update_access(&self, key: &str) {
        let count = self.access_count.entry(key.to_string()).or_default();
        count.fetch_add(1, Ordering::Relaxed);
        let mut last = self.last_accessed.entry(key.to_string()).or_insert_with(Instant::now);
        *last = Instant::now();
    }

    pub fn invalidate(&self, key: &str) {
        self.label_cache.remove(key);
        self.property_cache.remove(key);
        self.access_count.remove(key);
        self.last_accessed.remove(key);
    }

    pub fn clear(&self) {
        self.label_cache.clear();
        self.property_cache.clear();
        self.access_count.clear();
        self.last_accessed.clear();
    }

    pub fn size(&self) -> usize {
        self.label_cache.len() + self.property_cache.len()
    }
}

fn value_to_string(value: &Value) -> String {
    match value {
        Value::Empty => "empty".to_string(),
        Value::Null(_) => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::String(s) => s.clone(),
        Value::Date(d) => format!("{}-{}-{}", d.year, d.month, d.day),
        Value::Time(t) => format!("{}:{}:{}", t.hour, t.minute, t.sec),
        Value::DateTime(dt) => format!(
            "{}-{}-{} {}:{}:{}",
            dt.year, dt.month, dt.day, dt.hour, dt.minute, dt.sec
        ),
        _ => format!("{:?}", value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Tag;

    fn create_test_vertex(id: i64, name: &str) -> Vertex {
        Vertex {
            vid: Box::new(Value::Int(id)),
            id,
            tags: vec![Tag {
                name: "person".to_string(),
                properties: vec![("name".to_string(), Value::String(name.to_string()))]
                    .into_iter()
                    .collect(),
            }],
            properties: vec![("name".to_string(), Value::String(name.to_string()))]
                .into_iter()
                .collect(),
        }
    }

    #[test]
    fn test_index_service_creation() {
        let service = IndexService::new(1);
        assert_eq!(service.space_id, 1);
    }

    #[test]
    fn test_create_index() {
        let service = IndexService::new(1);

        let fields = vec![IndexField::new(
            "name".to_string(),
            Value::String("string".to_string()),
            false,
        )];

        let index_id = service.create_index(
            "person_name_idx",
            "person",
            fields,
            false,
        );

        assert_eq!(index_id, 1);

        let index = service.get_index(1).unwrap();
        assert_eq!(index.name, "person_name_idx");
        assert_eq!(index.schema_name, "person");
    }

    #[test]
    fn test_drop_index() {
        let service = IndexService::new(1);

        let fields = vec![IndexField::new(
            "name".to_string(),
            Value::String("string".to_string()),
            false,
        )];

        service.create_index("test_idx", "person", fields.clone(), false);

        assert!(service.get_index(1).is_some());
        assert!(service.drop_index(1));
        assert!(service.get_index(1).is_none());
    }

    #[test]
    fn test_list_indexes() {
        let service = IndexService::new(1);

        let fields = vec![IndexField::new(
            "name".to_string(),
            Value::String("string".to_string()),
            false,
        )];

        service.create_index("idx1", "person", fields.clone(), false);
        service.create_index("idx2", "person", fields.clone(), false);

        let indexes = service.list_indexes();
        assert_eq!(indexes.len(), 2);
    }

    #[test]
    fn test_memory_index_cache() {
        let cache = MemoryIndexCache::new();

        let node_ids = vec![Value::Int(1), Value::Int(2), Value::Int(3)];
        cache.cache_by_label("person", node_ids.clone());

        let cached = cache.get_by_label("person").unwrap();
        assert_eq!(cached.len(), 3);
    }

    #[test]
    fn test_memory_index_cache_invalidate() {
        let cache = MemoryIndexCache::new();

        cache.cache_by_label("person", vec![Value::Int(1)]);
        assert!(cache.get_by_label("person").is_some());

        cache.invalidate("person");
        assert!(cache.get_by_label("person").is_none());
    }

    #[test]
    fn test_memory_index_cache_clear() {
        let cache = MemoryIndexCache::new();

        cache.cache_by_label("person", vec![Value::Int(1)]);
        cache.cache_by_property("name", &Value::String("test".to_string()), vec![Value::Int(2)]);

        assert!(cache.size() > 0);

        cache.clear();
        assert_eq!(cache.size(), 0);
    }
}