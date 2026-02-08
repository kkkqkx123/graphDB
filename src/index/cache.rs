//! 索引缓存模块
//!
//! 提供索引查询结果的内存缓存支持
//!
//! 功能：
//! - 精确查找缓存（带版本控制）
//! - 标签和属性缓存（LRU淘汰）
//! - 缓存统计和监控

use crate::core::Value;
use crate::index::IndexServiceConfig;
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

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
    
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct CacheKey {
    pub index_id: i32,
    pub value: Value,
}

impl CacheKey {
    pub fn new(index_id: i32, value: Value) -> Self {
        Self { index_id, value }
    }
}

pub struct VersionedCache<V> {
    cache: DashMap<CacheKey, CacheEntry<V>>,
    index_versions: DashMap<i32, u64>,
    config: IndexServiceConfig,
    stats: Arc<CacheStats>,
}

impl<V: Clone> VersionedCache<V> {
    pub fn new(config: IndexServiceConfig, stats: Arc<CacheStats>) -> Self {
        Self {
            cache: DashMap::new(),
            index_versions: DashMap::new(),
            config,
            stats,
        }
    }

    pub fn get(&self, index_id: i32, value: &Value) -> Option<V> {
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

    pub fn insert(&self, index_id: i32, value: Value, result: V) {
        let key = CacheKey::new(index_id, value);
        let entry = CacheEntry::new(result);
        self.cache.insert(key, entry);
        self.stats.record_insertion();
    }

    pub fn invalidate_index(&self, index_id: i32) {
        let current_version = self.index_versions.get(&index_id)
            .map(|v| *v)
            .unwrap_or(0);
        self.index_versions.insert(index_id, current_version + 1);
        self.stats.record_invalidation();

        self.cache.retain(|key, _| key.index_id != index_id);
    }

    pub fn invalidate(&self, index_id: i32, value: &Value) {
        let key = CacheKey::new(index_id, value.clone());
        self.cache.remove(&key);
        self.stats.record_invalidation();
    }

    pub fn clear(&self) {
        self.cache.clear();
        self.index_versions.clear();
    }

    pub fn size(&self) -> usize {
        self.cache.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
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

    #[test]
    fn test_cache_stats() {
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
        assert!((snapshot.hit_rate - 66.66666666666666).abs() < 0.01);
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
}
