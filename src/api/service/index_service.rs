//! 索引服务模块
//!
//! 提供统一的索引服务层，整合内存索引缓存和持久化存储
//!
//! 功能：
//! - 索引创建、删除、查询
//! - 内存索引缓存（支持细粒度失效和并发优化）
//! - 与持久化存储层集成

use crate::core::{Value, Vertex};
use crate::index::{ConcurrentIndexStorage, Index, IndexField, IndexInfo, IndexType};
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
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

#[derive(Debug)]
pub struct CacheStats {
    pub hits: AtomicU64,
    pub misses: AtomicU64,
    pub evictions: AtomicU64,
    pub insertions: AtomicU64,
    pub invalidations: AtomicU64,
}

impl Clone for CacheStats {
    fn clone(&self) -> Self {
        Self {
            hits: AtomicU64::new(self.hits.load(Ordering::Relaxed)),
            misses: AtomicU64::new(self.misses.load(Ordering::Relaxed)),
            evictions: AtomicU64::new(self.evictions.load(Ordering::Relaxed)),
            insertions: AtomicU64::new(self.insertions.load(Ordering::Relaxed)),
            invalidations: AtomicU64::new(self.invalidations.load(Ordering::Relaxed)),
        }
    }
}

impl Default for CacheStats {
    fn default() -> Self {
        Self {
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            evictions: AtomicU64::new(0),
            insertions: AtomicU64::new(0),
            invalidations: AtomicU64::new(0),
        }
    }
}

impl CacheStats {
    pub fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_eviction(&self) {
        self.evictions.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_insertion(&self) {
        self.insertions.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_invalidation(&self) {
        self.invalidations.fetch_add(1, Ordering::Relaxed);
    }

    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = hits + misses;
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64 * 100.0
        }
    }

    pub fn reset(&self) {
        self.hits.store(0, Ordering::Relaxed);
        self.misses.store(0, Ordering::Relaxed);
        self.evictions.store(0, Ordering::Relaxed);
        self.insertions.store(0, Ordering::Relaxed);
        self.invalidations.store(0, Ordering::Relaxed);
    }
}

#[derive(Debug)]
struct CacheEntry<V> {
    value: V,
    inserted_at: u64,
    last_accessed: AtomicU64,
    access_count: AtomicU64,
}

impl<V: Clone> Clone for CacheEntry<V> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            inserted_at: self.inserted_at,
            last_accessed: AtomicU64::new(self.last_accessed.load(Ordering::Relaxed)),
            access_count: AtomicU64::new(self.access_count.load(Ordering::Relaxed)),
        }
    }
}

impl<V> CacheEntry<V> {
    fn new(value: V) -> Self {
        let now = Instant::now();
        let nanos = now.elapsed().as_nanos() as u64;
        Self {
            value,
            inserted_at: nanos,
            last_accessed: AtomicU64::new(nanos),
            access_count: AtomicU64::new(0),
        }
    }

    fn access(&self) {
        let nanos = Instant::now().elapsed().as_nanos() as u64;
        self.last_accessed.store(nanos, Ordering::Relaxed);
        self.access_count.fetch_add(1, Ordering::Relaxed);
    }

    fn access_count(&self) -> u64 {
        self.access_count.load(Ordering::Relaxed)
    }

    fn last_accessed(&self) -> Instant {
        let nanos = self.last_accessed.load(Ordering::Relaxed);
        Instant::now() - Duration::from_nanos(nanos)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct CacheKey {
    index_id: i32,
    value: Value,
}

impl CacheKey {
    fn new(index_id: i32, value: Value) -> Self {
        Self { index_id, value }
    }
}

struct VersionedCache<V> {
    cache: DashMap<CacheKey, CacheEntry<V>>,
    index_versions: DashMap<i32, u64>,
    config: IndexServiceConfig,
    stats: Arc<CacheStats>,
}

impl<V: Clone> VersionedCache<V> {
    fn new(config: IndexServiceConfig, stats: Arc<CacheStats>) -> Self {
        Self {
            cache: DashMap::new(),
            index_versions: DashMap::new(),
            config,
            stats,
        }
    }

    fn get(&self, index_id: i32, value: &Value) -> Option<V> {
        let key = CacheKey::new(index_id, value.clone());
        if let Some(entry) = self.cache.get(&key) {
            let entry = entry.value();
            entry.access();

            let age_nanos = Instant::now().elapsed().as_nanos() as u64;
            let inserted_at = entry.inserted_at;
            if age_nanos.saturating_sub(inserted_at) > self.config.cache_ttl_secs * 1_000_000_000 {
                self.cache.remove(&key);
                self.stats.record_invalidation();
                return None;
            }

            self.stats.record_hit();
            return Some(entry.value.clone());
        }
        self.stats.record_miss();
        None
    }

    fn insert(&self, index_id: i32, value: Value, result: V) {
        let key = CacheKey::new(index_id, value);
        let entry = CacheEntry::new(result);
        self.cache.insert(key, entry);
        self.stats.record_insertion();
    }

    fn invalidate_index(&self, index_id: i32) {
        let current_version = self.index_versions.get(&index_id)
            .map(|v| *v)
            .unwrap_or(0);
        self.index_versions.insert(index_id, current_version + 1);
        self.stats.record_invalidation();

        self.cache.retain(|key, _| key.index_id != index_id);
    }

    fn invalidate(&self, index_id: i32, value: &Value) {
        let key = CacheKey::new(index_id, value.clone());
        self.cache.remove(&key);
        self.stats.record_invalidation();
    }

    fn clear(&self) {
        self.cache.clear();
        self.index_versions.clear();
    }

    fn size(&self) -> usize {
        self.cache.len()
    }

    fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

pub struct IndexService {
    space_id: i32,
    index_by_id: DashMap<i32, Arc<ConcurrentIndexStorage>>,
    index_by_name: DashMap<String, i32>,
    index_metadata: DashMap<i32, Index>,
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
        Self {
            space_id,
            index_by_id: DashMap::new(),
            index_by_name: DashMap::new(),
            index_metadata: DashMap::new(),
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

        let storage = Arc::new(ConcurrentIndexStorage::new(self.space_id, index_id, name.to_string()));

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

        storage.insert_vertex("", &Value::Int(vertex_id), vertex);

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

#[derive(Clone, Debug)]
pub struct CacheStatsSnapshot {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub insertions: u64,
    pub invalidations: u64,
    pub hit_rate: f64,
    pub cache_size: usize,
}

pub struct MemoryIndexCache {
    label_cache: DashMap<String, Vec<Value>>,
    property_cache: DashMap<String, HashMap<Value, Vec<Value>>>,
    access_count: DashMap<String, AtomicU64>,
    last_accessed: DashMap<String, u64>,
    max_size: usize,
    stats: Arc<CacheStats>,
}

impl Default for MemoryIndexCache {
    fn default() -> Self {
        Self::new_with_size(10000, Arc::new(CacheStats::default()))
    }
}

impl MemoryIndexCache {
    pub fn new() -> Self {
        Self::new_with_size(10000, Arc::new(CacheStats::default()))
    }

    pub fn new_with_size(max_size: usize, stats: Arc<CacheStats>) -> Self {
        Self {
            label_cache: DashMap::new(),
            property_cache: DashMap::new(),
            access_count: DashMap::new(),
            last_accessed: DashMap::new(),
            max_size,
            stats,
        }
    }

    pub fn cache_by_label(&self, label: &str, node_ids: Vec<Value>) {
        if self.label_cache.len() >= self.max_size {
            self.evict_lru();
        }
        let mut entry = self.label_cache.entry(label.to_string()).or_default();
        *entry = node_ids;
        self.update_access(label);
    }

    pub fn get_by_label(&self, label: &str) -> Option<Vec<Value>> {
        self.update_access(label);
        self.label_cache.get(label).map(|e| e.clone())
    }

    pub fn cache_by_property(&self, property: &str, value: &Value, node_ids: Vec<Value>) {
        if self.property_cache.len() >= self.max_size {
            self.evict_lru_property();
        }
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
        let mut last = self.last_accessed.entry(key.to_string()).or_insert_with(|| Instant::now().elapsed().as_nanos() as u64);
        *last = Instant::now().elapsed().as_nanos() as u64;
    }

    fn evict_lru(&self) {
        let mut min_key = None;
        let mut min_value = u64::MAX;

        for entry in self.last_accessed.iter() {
            let key = entry.key().clone();
            let value = *entry.value();
            if value < min_value {
                min_value = value;
                min_key = Some(key);
            }
        }

        if let Some(key) = min_key {
            self.label_cache.remove(&key);
            self.access_count.remove(&key);
            self.last_accessed.remove(&key);
            self.stats.record_eviction();
        }
    }

    fn evict_lru_property(&self) {
        let mut min_key = None;
        let mut min_value = u64::MAX;

        for entry in self.last_accessed.iter() {
            let key = entry.key().clone();
            let value = *entry.value();
            if value < min_value {
                min_value = value;
                min_key = Some(key);
            }
        }

        if let Some(key) = min_key {
            self.property_cache.remove(&key);
            self.access_count.remove(&key);
            self.last_accessed.remove(&key);
            self.stats.record_eviction();
        }
    }

    pub fn invalidate(&self, key: &str) {
        self.label_cache.remove(key);
        self.property_cache.remove(key);
        self.access_count.remove(key);
        self.last_accessed.remove(key);
        self.stats.record_invalidation();
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

    pub fn max_size(&self) -> usize {
        self.max_size
    }

    pub fn set_max_size(&mut self, max_size: usize) {
        self.max_size = max_size;
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

    #[test]
    fn test_versioned_cache_get_insert() {
        let config = IndexServiceConfig::default();
        let stats = Arc::new(CacheStats::default());
        let cache: VersionedCache<Vec<()>> = VersionedCache::new(config, stats);

        let result = cache.get(1, &Value::Int(100));
        assert!(result.is_none());

        cache.insert(1, Value::Int(100), vec![()]);
        let result = cache.get(1, &Value::Int(100));
        assert!(result.is_some());
    }

    #[test]
    fn test_versioned_cache_invalidate_index() {
        let config = IndexServiceConfig::default();
        let stats = Arc::new(CacheStats::default());
        let cache: VersionedCache<Vec<()>> = VersionedCache::new(config, stats);

        cache.insert(1, Value::Int(100), vec![()]);
        cache.insert(1, Value::Int(200), vec![()]);
        cache.insert(2, Value::Int(100), vec![()]);

        cache.invalidate_index(1);

        assert!(cache.get(1, &Value::Int(100)).is_none());
        assert!(cache.get(1, &Value::Int(200)).is_none());
        assert!(cache.get(2, &Value::Int(100)).is_some());
    }

    #[test]
    fn test_memory_index_cache_lru() {
        let stats = Arc::new(CacheStats::default());
        let cache = MemoryIndexCache::new_with_size(2, stats);

        cache.cache_by_label("label1", vec![Value::Int(1)]);
        cache.cache_by_label("label2", vec![Value::Int(2)]);

        assert_eq!(cache.size(), 2);

        cache.cache_by_label("label3", vec![Value::Int(3)]);

        assert_eq!(cache.size(), 2);
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

    #[test]
    fn test_cache_stats_recording() {
        let stats = Arc::new(CacheStats::default());

        stats.record_hit();
        stats.record_hit();
        stats.record_miss();
        stats.record_eviction();
        stats.record_insertion();
        stats.record_invalidation();

        let snapshot = CacheStatsSnapshot {
            hits: stats.hits.load(Ordering::Relaxed),
            misses: stats.misses.load(Ordering::Relaxed),
            evictions: stats.evictions.load(Ordering::Relaxed),
            insertions: stats.insertions.load(Ordering::Relaxed),
            invalidations: stats.invalidations.load(Ordering::Relaxed),
            hit_rate: stats.hit_rate(),
            cache_size: 0,
        };

        assert_eq!(snapshot.hits, 2);
        assert_eq!(snapshot.misses, 1);
        assert_eq!(snapshot.hit_rate, 66.66666666666666);
    }

    #[test]
    fn test_cache_entry_access() {
        let entry = CacheEntry::new(vec![1, 2, 3]);
        assert_eq!(entry.access_count(), 0);

        entry.access();
        assert_eq!(entry.access_count(), 1);

        entry.access();
        assert_eq!(entry.access_count(), 2);
    }

    #[test]
    fn test_cache_key_equality() {
        let key1 = CacheKey::new(1, Value::Int(100));
        let key2 = CacheKey::new(1, Value::Int(100));
        let key3 = CacheKey::new(2, Value::Int(100));
        let key4 = CacheKey::new(1, Value::Int(200));

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
        assert_ne!(key1, key4);
    }
}