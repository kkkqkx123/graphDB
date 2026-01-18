//! 索引服务模块
//!
//! 提供统一的索引服务层
//!
//! 功能：
//! - 索引创建、删除、查询
//! - 标签索引和边索引支持
//! - 与存储层集成

use crate::core::{Value, Vertex};
use crate::index::cache::{CacheStats, CacheStatsSnapshot, VersionedCache};
use crate::index::{ConcurrentIndexStorage, Index, IndexField, IndexInfo, IndexType};
use crate::storage::MemoryStorage;
use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;
use std::time::Instant;

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
    #[error("缓存错误: {0}")]
    CacheError(String),
}

#[derive(Clone, Debug)]
pub struct IndexServiceConfig {
    pub max_memory_bytes: u64,
    pub enable_auto_cleanup: bool,
    pub cleanup_interval_secs: u64,
    pub exact_lookup_cache_size: usize,
    pub enable_cache_stats: bool,
    pub cache_ttl_secs: u64,
}

impl Default for IndexServiceConfig {
    fn default() -> Self {
        Self {
            max_memory_bytes: 1024 * 1024 * 1024,
            enable_auto_cleanup: true,
            cleanup_interval_secs: 300,
            exact_lookup_cache_size: 10000,
            enable_cache_stats: true,
            cache_ttl_secs: 3600,
        }
    }
}

pub struct IndexService {
    space_id: i32,
    index_by_id: DashMap<i32, Arc<ConcurrentIndexStorage>>,
    index_by_name: DashMap<String, i32>,
    index_metadata: DashMap<i32, Index>,
    storage: crate::index::storage::StorageRef,
    next_index_id: AtomicU64,
    config: IndexServiceConfig,
    exact_lookup_cache: Arc<VersionedCache<Vec<Vertex>>>,
    cache_stats: Arc<CacheStats>,
    last_cleanup: RwLock<Instant>,
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
        let stats = Arc::new(CacheStats::default());
        let storage: crate::index::storage::StorageRef = Arc::new(Mutex::new(MemoryStorage::new().unwrap()));
        Self {
            space_id,
            index_by_id: DashMap::new(),
            index_by_name: DashMap::new(),
            index_metadata: DashMap::new(),
            storage,
            next_index_id: AtomicU64::new(1),
            config: config.clone(),
            exact_lookup_cache: Arc::new(VersionedCache::new(config, Arc::clone(&stats))),
            cache_stats: stats,
            last_cleanup: RwLock::new(Instant::now()),
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
        if name.trim().is_empty() {
            return Err(IndexServiceError::InvalidParameter("索引名称不能为空".to_string()));
        }
        if schema_name.trim().is_empty() {
            return Err(IndexServiceError::InvalidParameter("模式名称不能为空".to_string()));
        }
        if fields.is_empty() {
            return Err(IndexServiceError::InvalidParameter("索引字段不能为空".to_string()));
        }

        if self.index_by_name.contains_key(name) {
            return Err(IndexServiceError::NameExists(name.to_string()));
        }

        let index_id = self.next_index_id.fetch_add(1, Ordering::Relaxed) as i32;

        let storage = Arc::new(ConcurrentIndexStorage::new(self.space_id, index_id, name.to_string(), self.storage.clone()));

        self.index_by_id.insert(index_id, Arc::clone(&storage));
        self.index_by_name.insert(name.to_string(), index_id);

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

            self.exact_lookup_cache.invalidate_index(index_id);

            self.trigger_cleanup_if_needed();
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

        if let Some(cached) = self.exact_lookup_cache.get(*index_id, value) {
            return Ok(cached);
        }

        let storage = self.index_by_id
            .get(&index_id)
            .ok_or_else(|| IndexServiceError::NotFound(*index_id))?;

        let result = storage.exact_lookup("", value)
            .map(|(vertices, _, _)| vertices)
            .map_err(|e| IndexServiceError::StorageError(e.to_string()))?;

        self.exact_lookup_cache.insert(*index_id, value.clone(), result.clone());

        Ok(result)
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

        let vertex_id = vertex.id;

        let storage = self.index_by_id
            .get(&index_id)
            .ok_or_else(|| IndexServiceError::NotFound(*index_id))?;

        storage.insert_vertex("", &Value::Int(vertex_id), &vertex);

        self.exact_lookup_cache.invalidate(*index_id, &Value::Int(vertex_id));

        self.trigger_cleanup_if_needed();
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

        self.exact_lookup_cache.invalidate(*index_id, &Value::Int(vertex.id));

        self.trigger_cleanup_if_needed();
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

    pub fn get_cache_stats(&self) -> CacheStatsSnapshot {
        CacheStatsSnapshot {
            hits: self.cache_stats.hits.load(Ordering::Relaxed),
            misses: self.cache_stats.misses.load(Ordering::Relaxed),
            evictions: self.cache_stats.evictions.load(Ordering::Relaxed),
            insertions: self.cache_stats.insertions.load(Ordering::Relaxed),
            invalidations: self.cache_stats.invalidations.load(Ordering::Relaxed),
            hit_rate: self.cache_stats.hit_rate(),
            cache_size: self.exact_lookup_cache.size(),
        }
    }

    pub fn clear_cache(&self) {
        self.exact_lookup_cache.clear();
    }

    pub fn clear_cache_for_index(&self, index_name: &str) -> Result<(), IndexServiceError> {
        let index_id = self.index_by_name
            .get(index_name)
            .ok_or_else(|| IndexServiceError::NotFound(-1))?;

        self.exact_lookup_cache.invalidate_index(*index_id);
        Ok(())
    }

    pub fn reset_cache_stats(&self) {
        self.cache_stats.reset();
    }

    fn trigger_cleanup_if_needed(&self) {
        if !self.config.enable_auto_cleanup {
            return;
        }

        let now = Instant::now();
        let mut last_cleanup = self.last_cleanup.write().unwrap();

        if now.duration_since(*last_cleanup) > Duration::from_secs(self.config.cleanup_interval_secs) {
            *last_cleanup = now;
            self.perform_cleanup();
        }
    }

    fn perform_cleanup(&self) {
        if self.exact_lookup_cache.size() == 0 {
            return;
        }
    }

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

    pub fn index_exists(&self, index_id: i32) -> bool {
        self.index_metadata.contains_key(&index_id)
    }

    pub fn index_name_exists(&self, name: &str) -> bool {
        self.index_by_name.contains_key(name)
    }

    pub fn get_config(&self) -> &IndexServiceConfig {
        &self.config
    }

    pub fn get_space_id(&self) -> i32 {
        self.space_id
    }

    pub fn tag_index_exists(&self, index_name: &str) -> bool {
        self.get_index_by_name(index_name).is_ok()
    }

    pub fn edge_index_exists(&self, index_name: &str) -> bool {
        self.get_index_by_name(index_name).is_ok()
    }

    pub fn get_tag_indexes_for_tag(&self, tag_name: &str) -> Vec<Index> {
        self.list_indexes()
            .into_iter()
            .filter(|index| index.schema_name == tag_name && matches!(index.index_type, crate::index::IndexType::TagIndex))
            .collect()
    }

    pub fn get_edge_indexes_for_edge_type(&self, edge_type_name: &str) -> Vec<Index> {
        self.list_indexes()
            .into_iter()
            .filter(|index| index.schema_name == edge_type_name && matches!(index.index_type, crate::index::IndexType::EdgeIndex))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Tag;

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
            IndexType::TagIndex,
            false,
        );

        assert_eq!(index_id.unwrap(), 1);

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

        service.create_index("test_idx", "person", fields.clone(), IndexType::TagIndex, false);

        assert!(service.get_index(1).is_ok());
        assert!(service.drop_index(1).is_ok());
        assert!(service.get_index(1).is_err());
    }

    #[test]
    fn test_list_indexes() {
        let service = IndexService::new(1);

        let fields = vec![IndexField::new(
            "name".to_string(),
            Value::String("string".to_string()),
            false,
        )];

        service.create_index("idx1", "person", fields.clone(), IndexType::TagIndex, false);
        service.create_index("idx2", "person", fields.clone(), IndexType::TagIndex, false);

        let indexes = service.list_indexes();
        assert_eq!(indexes.len(), 2);
    }

    #[test]
    fn test_cache_stats() {
        let service = IndexService::new(1);
        let stats = service.get_cache_stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.hit_rate, 0.0);
    }

    #[test]
    fn test_cache_clear() {
        let service = IndexService::new(1);

        let fields = vec![IndexField::new(
            "name".to_string(),
            Value::String("string".to_string()),
            false,
        )];

        service.create_index("test_idx", "person", fields.clone(), IndexType::TagIndex, false);

        service.clear_cache();
        assert!(service.exact_lookup_cache.is_empty());
    }

    #[test]
    fn test_cache_clear_for_index() {
        let service = IndexService::new(1);

        let fields = vec![IndexField::new(
            "name".to_string(),
            Value::String("string".to_string()),
            false,
        )];

        service.create_index("test_idx", "person", fields.clone(), IndexType::TagIndex, false);
        assert!(service.clear_cache_for_index("test_idx").is_ok());
    }

    #[test]
    fn test_cache_clear_for_index_not_found() {
        let service = IndexService::new(1);
        assert!(service.clear_cache_for_index("non_existent").is_err());
    }
}
