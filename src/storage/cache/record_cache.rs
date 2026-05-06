//! Record Cache
//!
//! High-performance cache for vertex and edge records using Moka.
//! Provides O(1) operations with TinyLFU eviction policy for optimal hit rate.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use moka::sync::Cache;

use crate::core::Value;
use crate::storage::memory::MemoryTracker;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VertexCacheKey {
    pub label_id: u16,
    pub internal_id: u32,
    pub timestamp: u64,
}

impl VertexCacheKey {
    pub fn new(label_id: u16, internal_id: u32, timestamp: u64) -> Self {
        Self {
            label_id,
            internal_id,
            timestamp,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EdgeCacheKey {
    pub edge_label_id: u16,
    pub src_vid: u64,
    pub dst_vid: u64,
    pub edge_id: u64,
    pub timestamp: u64,
}

impl EdgeCacheKey {
    pub fn new(
        edge_label_id: u16,
        src_vid: u64,
        dst_vid: u64,
        edge_id: u64,
        timestamp: u64,
    ) -> Self {
        Self {
            edge_label_id,
            src_vid,
            dst_vid,
            edge_id,
            timestamp,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EdgeQueryKey {
    pub edge_label_id: u16,
    pub src_vid: u64,
    pub dst_vid: u64,
    pub timestamp: u64,
}

impl EdgeQueryKey {
    pub fn new(edge_label_id: u16, src_vid: u64, dst_vid: u64, timestamp: u64) -> Self {
        Self {
            edge_label_id,
            src_vid,
            dst_vid,
            timestamp,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IdIndexCacheKey {
    pub label_id: u16,
    pub external_id: String,
}

impl IdIndexCacheKey {
    pub fn new(label_id: u16, external_id: String) -> Self {
        Self {
            label_id,
            external_id,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CachedVertex {
    pub internal_id: u32,
    pub external_id: String,
    pub properties: Vec<(String, Value)>,
}

impl CachedVertex {
    pub fn estimated_size(&self) -> u32 {
        let mut size = std::mem::size_of::<u32>() * 2;
        size += self.external_id.len();
        for (name, value) in &self.properties {
            size += name.len();
            size += value.estimated_size();
        }
        size as u32
    }
}

#[derive(Debug, Clone)]
pub struct CachedEdge {
    pub edge_id: u64,
    pub src_vid: u64,
    pub dst_vid: u64,
    pub properties: Vec<(String, Value)>,
}

impl CachedEdge {
    pub fn estimated_size(&self) -> u32 {
        let mut size = std::mem::size_of::<u64>() * 3;
        for (name, value) in &self.properties {
            size += name.len();
            size += value.estimated_size();
        }
        size as u32
    }
}

#[derive(Debug, Clone)]
pub struct RecordCacheConfig {
    pub max_memory: usize,
    pub memory_ratio: (u32, u32, u32, u32),
    pub ttl: Option<Duration>,
    pub tti: Option<Duration>,
}

impl Default for RecordCacheConfig {
    fn default() -> Self {
        Self {
            max_memory: 128 * 1024 * 1024,
            memory_ratio: (40, 30, 20, 10),
            ttl: Some(Duration::from_secs(3600)),
            tti: Some(Duration::from_secs(300)),
        }
    }
}

pub struct RecordCache {
    vertex_cache: Cache<VertexCacheKey, CachedVertex>,
    edge_cache: Cache<EdgeCacheKey, CachedEdge>,
    edge_query_cache: Cache<EdgeQueryKey, CachedEdge>,
    id_index_cache: Cache<IdIndexCacheKey, u32>,
    config: RecordCacheConfig,
    hits: AtomicU64,
    misses: AtomicU64,
    memory_tracker: Option<Arc<MemoryTracker>>,
}

impl std::fmt::Debug for RecordCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RecordCache")
            .field("config", &self.config)
            .field("vertex_count", &self.vertex_cache.entry_count())
            .field("edge_count", &self.edge_cache.entry_count())
            .field("edge_query_count", &self.edge_query_cache.entry_count())
            .field("id_index_count", &self.id_index_cache.entry_count())
            .field("hits", &self.hits.load(Ordering::Relaxed))
            .field("misses", &self.misses.load(Ordering::Relaxed))
            .finish()
    }
}

impl RecordCache {
    pub fn new() -> Self {
        Self::with_config(RecordCacheConfig::default())
    }

    pub fn with_config(config: RecordCacheConfig) -> Self {
        let max_memory = config.max_memory as u64;
        let total_ratio = config.memory_ratio.0 + config.memory_ratio.1 + config.memory_ratio.2 + config.memory_ratio.3;
        let vertex_memory = max_memory * config.memory_ratio.0 as u64 / total_ratio as u64;
        let edge_memory = max_memory * config.memory_ratio.1 as u64 / total_ratio as u64;
        let edge_query_memory = max_memory * config.memory_ratio.2 as u64 / total_ratio as u64;
        let id_index_memory = max_memory * config.memory_ratio.3 as u64 / total_ratio as u64;

        let mut vertex_builder = Cache::builder()
            .max_capacity(vertex_memory)
            .weigher(|_key: &VertexCacheKey, value: &CachedVertex| value.estimated_size());

        let mut edge_builder = Cache::builder()
            .max_capacity(edge_memory)
            .weigher(|_key: &EdgeCacheKey, value: &CachedEdge| value.estimated_size());

        let mut edge_query_builder = Cache::builder()
            .max_capacity(edge_query_memory)
            .weigher(|_key: &EdgeQueryKey, value: &CachedEdge| value.estimated_size());

        let mut id_index_builder = Cache::builder()
            .max_capacity(id_index_memory)
            .weigher(|_key: &IdIndexCacheKey, value: &u32| {
                std::mem::size_of::<u32>() as u32
            });

        if let Some(ttl) = config.ttl {
            vertex_builder = vertex_builder.time_to_live(ttl);
            edge_builder = edge_builder.time_to_live(ttl);
            edge_query_builder = edge_query_builder.time_to_live(ttl);
            id_index_builder = id_index_builder.time_to_live(ttl);
        }

        if let Some(tti) = config.tti {
            vertex_builder = vertex_builder.time_to_idle(tti);
            edge_builder = edge_builder.time_to_idle(tti);
            edge_query_builder = edge_query_builder.time_to_idle(tti);
            id_index_builder = id_index_builder.time_to_idle(tti);
        }

        let vertex_cache = vertex_builder.build();
        let edge_cache = edge_builder.build();
        let edge_query_cache = edge_query_builder.build();
        let id_index_cache = id_index_builder.build();

        Self {
            vertex_cache,
            edge_cache,
            edge_query_cache,
            id_index_cache,
            config,
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            memory_tracker: None,
        }
    }

    pub fn with_memory_tracker(mut self, tracker: Arc<MemoryTracker>) -> Self {
        self.memory_tracker = Some(tracker);
        self
    }

    pub fn get_id_index(&self, label_id: u16, external_id: &str) -> Option<u32> {
        let key = IdIndexCacheKey::new(label_id, external_id.to_string());
        match self.id_index_cache.get(&key) {
            Some(internal_id) => {
                self.hits.fetch_add(1, Ordering::Relaxed);
                Some(internal_id)
            }
            None => {
                self.misses.fetch_add(1, Ordering::Relaxed);
                None
            }
        }
    }

    pub fn insert_id_index(&self, label_id: u16, external_id: &str, internal_id: u32) {
        let key = IdIndexCacheKey::new(label_id, external_id.to_string());
        self.id_index_cache.insert(key, internal_id);

        if let Some(ref tracker) = self.memory_tracker {
            tracker.try_allocate_cache(std::mem::size_of::<u32>());
        }
    }

    pub fn remove_id_index(&self, label_id: u16, external_id: &str) {
        let key = IdIndexCacheKey::new(label_id, external_id.to_string());
        if self.id_index_cache.remove(&key).is_some() {
            if let Some(ref tracker) = self.memory_tracker {
                tracker.release_cache(std::mem::size_of::<u32>());
            }
        }
    }

    pub fn get_vertex(&self, key: &VertexCacheKey) -> Option<CachedVertex> {
        match self.vertex_cache.get(key) {
            Some(vertex) => {
                self.hits.fetch_add(1, Ordering::Relaxed);
                Some(vertex)
            }
            None => {
                self.misses.fetch_add(1, Ordering::Relaxed);
                None
            }
        }
    }

    pub fn insert_vertex(&self, key: VertexCacheKey, vertex: CachedVertex) {
        let size = vertex.estimated_size() as usize;
        self.vertex_cache.insert(key, vertex);

        if let Some(ref tracker) = self.memory_tracker {
            tracker.try_allocate_cache(size);
        }
    }

    pub fn remove_vertex(&self, key: &VertexCacheKey) {
        if let Some(vertex) = self.vertex_cache.remove(key) {
            let size = vertex.estimated_size() as usize;
            if let Some(ref tracker) = self.memory_tracker {
                tracker.release_cache(size);
            }
        }
    }

    pub fn get_edge(&self, key: &EdgeCacheKey) -> Option<CachedEdge> {
        match self.edge_cache.get(key) {
            Some(edge) => {
                self.hits.fetch_add(1, Ordering::Relaxed);
                Some(edge)
            }
            None => {
                self.misses.fetch_add(1, Ordering::Relaxed);
                None
            }
        }
    }

    pub fn insert_edge(&self, key: EdgeCacheKey, edge: CachedEdge) {
        let size = edge.estimated_size() as usize;
        self.edge_cache.insert(key, edge);

        if let Some(ref tracker) = self.memory_tracker {
            tracker.try_allocate_cache(size);
        }
    }

    pub fn remove_edge(&self, key: &EdgeCacheKey) {
        if let Some(edge) = self.edge_cache.remove(key) {
            let size = edge.estimated_size() as usize;
            if let Some(ref tracker) = self.memory_tracker {
                tracker.release_cache(size);
            }
        }
    }

    pub fn get_edge_by_query(&self, key: &EdgeQueryKey) -> Option<CachedEdge> {
        match self.edge_query_cache.get(key) {
            Some(edge) => {
                self.hits.fetch_add(1, Ordering::Relaxed);
                Some(edge)
            }
            None => {
                self.misses.fetch_add(1, Ordering::Relaxed);
                None
            }
        }
    }

    pub fn insert_edge_query(&self, key: EdgeQueryKey, edge: CachedEdge) {
        let size = edge.estimated_size() as usize;
        self.edge_query_cache.insert(key, edge);

        if let Some(ref tracker) = self.memory_tracker {
            tracker.try_allocate_cache(size);
        }
    }

    pub fn remove_edge_query(&self, key: &EdgeQueryKey) {
        if let Some(edge) = self.edge_query_cache.remove(key) {
            let size = edge.estimated_size() as usize;
            if let Some(ref tracker) = self.memory_tracker {
                tracker.release_cache(size);
            }
        }
    }

    pub fn invalidate_vertices_by_label(&self, label_id: u16) {
        self.vertex_cache.invalidate_entries_if(move |k, _| k.label_id == label_id);
    }

    pub fn invalidate_edges_by_label(&self, edge_label_id: u16) {
        self.edge_cache.invalidate_entries_if(move |k, _| k.edge_label_id == edge_label_id);
        self.edge_query_cache.invalidate_entries_if(move |k, _| k.edge_label_id == edge_label_id);
    }

    pub fn invalidate_edges_by_src(&self, src_vid: u64) {
        self.edge_cache.invalidate_entries_if(move |k, _| k.src_vid == src_vid);
        self.edge_query_cache.invalidate_entries_if(move |k, _| k.src_vid == src_vid);
    }

    pub fn invalidate_edges_by_dst(&self, dst_vid: u64) {
        self.edge_cache.invalidate_entries_if(move |k, _| k.dst_vid == dst_vid);
        self.edge_query_cache.invalidate_entries_if(move |k, _| k.dst_vid == dst_vid);
    }

    pub fn invalidate_id_indexes_by_label(&self, label_id: u16) {
        self.id_index_cache.invalidate_entries_if(move |k, _| k.label_id == label_id);
    }

    pub fn clear(&self) {
        self.vertex_cache.invalidate_all();
        self.edge_cache.invalidate_all();
        self.edge_query_cache.invalidate_all();
        self.id_index_cache.invalidate_all();
    }

    pub fn memory_usage(&self) -> usize {
        (self.vertex_cache.weighted_size()
            + self.edge_cache.weighted_size()
            + self.edge_query_cache.weighted_size()
            + self.id_index_cache.weighted_size()) as usize
    }

    pub fn max_memory(&self) -> usize {
        self.config.max_memory
    }

    pub fn stats(&self) -> RecordCacheStats {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = hits + misses;
        let memory_usage = self.memory_usage();

        RecordCacheStats {
            hits,
            misses,
            hit_rate: if total > 0 {
                hits as f64 / total as f64
            } else {
                0.0
            },
            vertex_count: self.vertex_cache.entry_count(),
            edge_count: self.edge_cache.entry_count(),
            edge_query_count: self.edge_query_cache.entry_count(),
            id_index_count: self.id_index_cache.entry_count(),
            memory_usage,
            max_memory: self.config.max_memory,
        }
    }

    pub fn utilization(&self) -> f32 {
        if self.config.max_memory == 0 {
            return 0.0;
        }
        self.memory_usage() as f32 / self.config.max_memory as f32
    }
}

impl Default for RecordCache {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RecordCacheStats {
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
    pub vertex_count: u64,
    pub edge_count: u64,
    pub edge_query_count: u64,
    pub id_index_count: u64,
    pub memory_usage: usize,
    pub max_memory: usize,
}

impl RecordCacheStats {
    pub fn format_bytes(bytes: usize) -> String {
        const KB: usize = 1024;
        const MB: usize = KB * 1024;
        const GB: usize = MB * 1024;

        if bytes >= GB {
            format!("{:.2} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.2} MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.2} KB", bytes as f64 / KB as f64)
        } else {
            format!("{} B", bytes)
        }
    }
}

impl std::fmt::Display for RecordCacheStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "Record Cache: {}/{} ({:.1}%)",
            Self::format_bytes(self.memory_usage),
            Self::format_bytes(self.max_memory),
            if self.max_memory > 0 {
                self.memory_usage as f64 / self.max_memory as f64 * 100.0
            } else {
                0.0
            }
        )?;
        writeln!(
            f,
            "  Vertices: {}, Edges: {}, EdgeQueries: {}, IdIndexes: {}",
            self.vertex_count, self.edge_count, self.edge_query_count, self.id_index_count
        )?;
        writeln!(
            f,
            "  Hits: {}, Misses: {}, Hit Rate: {:.1}%",
            self.hits,
            self.misses,
            self.hit_rate * 100.0
        )
    }
}

pub type SharedRecordCache = Arc<RecordCache>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex_cache_basic() {
        let cache = RecordCache::new();

        let key = VertexCacheKey::new(1, 100, 1000);
        let vertex = CachedVertex {
            internal_id: 100,
            external_id: "test_vertex".to_string(),
            properties: vec![("name".to_string(), Value::String("Alice".to_string()))],
        };

        cache.insert_vertex(key, vertex);

        let cached = cache.get_vertex(&VertexCacheKey::new(1, 100, 1000));
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().external_id, "test_vertex");
    }

    #[test]
    fn test_edge_cache_basic() {
        let cache = RecordCache::new();

        let key = EdgeCacheKey::new(1, 100, 200, 1, 1000);
        let edge = CachedEdge {
            edge_id: 1,
            src_vid: 100,
            dst_vid: 200,
            properties: vec![("weight".to_string(), Value::Double(1.5))],
        };

        cache.insert_edge(key, edge);

        let cached = cache.get_edge(&EdgeCacheKey::new(1, 100, 200, 1, 1000));
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().edge_id, 1);
    }

    #[test]
    fn test_cache_stats() {
        let cache = RecordCache::new();

        let key = VertexCacheKey::new(1, 100, 1000);
        let vertex = CachedVertex {
            internal_id: 100,
            external_id: "test".to_string(),
            properties: vec![],
        };

        cache.insert_vertex(key, vertex);

        cache.get_vertex(&VertexCacheKey::new(1, 100, 1000));
        cache.get_vertex(&VertexCacheKey::new(1, 999, 1000));

        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn test_cache_remove() {
        let cache = RecordCache::new();

        let key = VertexCacheKey::new(1, 100, 1000);
        let vertex = CachedVertex {
            internal_id: 100,
            external_id: "test".to_string(),
            properties: vec![],
        };

        cache.insert_vertex(key, vertex);
        assert!(cache.get_vertex(&VertexCacheKey::new(1, 100, 1000)).is_some());

        cache.remove_vertex(&VertexCacheKey::new(1, 100, 1000));
        assert!(cache.get_vertex(&VertexCacheKey::new(1, 100, 1000)).is_none());
    }

    #[test]
    fn test_cache_clear() {
        let cache = RecordCache::new();

        for i in 0..10u32 {
            let key = VertexCacheKey::new(1, i, 1000);
            let vertex = CachedVertex {
                internal_id: i,
                external_id: format!("v{}", i),
                properties: vec![],
            };
            cache.insert_vertex(key, vertex);
        }

        cache.clear();

        let stats = cache.stats();
        assert_eq!(stats.vertex_count, 0);
    }

    #[test]
    fn test_memory_weighted_eviction() {
        let config = RecordCacheConfig {
            max_memory: 1024,
            ..Default::default()
        };
        let cache = RecordCache::with_config(config);

        for i in 0..100u32 {
            let key = VertexCacheKey::new(1, i, 1000);
            let vertex = CachedVertex {
                internal_id: i,
                external_id: format!("vertex_{}", i),
                properties: vec![("data".to_string(), Value::String("x".repeat(50)))],
            };
            cache.insert_vertex(key, vertex);
        }

        cache.vertex_cache.run_pending_tasks();

        let stats = cache.stats();
        assert!(stats.vertex_count < 100, "Cache should have evicted entries");
    }

    #[test]
    fn test_mvcc_timestamp_versioning() {
        let cache = RecordCache::new();

        let key_v1 = VertexCacheKey::new(1, 100, 1000);
        let vertex_v1 = CachedVertex {
            internal_id: 100,
            external_id: "test".to_string(),
            properties: vec![("version".to_string(), Value::String("v1".to_string()))],
        };

        let key_v2 = VertexCacheKey::new(1, 100, 2000);
        let vertex_v2 = CachedVertex {
            internal_id: 100,
            external_id: "test".to_string(),
            properties: vec![("version".to_string(), Value::String("v2".to_string()))],
        };

        cache.insert_vertex(key_v1, vertex_v1);
        cache.insert_vertex(key_v2, vertex_v2);

        let cached_v1 = cache.get_vertex(&VertexCacheKey::new(1, 100, 1000));
        let cached_v2 = cache.get_vertex(&VertexCacheKey::new(1, 100, 2000));

        assert!(cached_v1.is_some());
        assert!(cached_v2.is_some());

        let v1_props = cached_v1.unwrap().properties;
        let v2_props = cached_v2.unwrap().properties;

        assert_eq!(v1_props[0].1, Value::String("v1".to_string()));
        assert_eq!(v2_props[0].1, Value::String("v2".to_string()));
    }

    #[test]
    fn test_id_index_cache() {
        let cache = RecordCache::new();

        cache.insert_id_index(1, "user_001", 100);
        cache.insert_id_index(1, "user_002", 200);
        cache.insert_id_index(2, "product_001", 300);

        assert_eq!(cache.get_id_index(1, "user_001"), Some(100));
        assert_eq!(cache.get_id_index(1, "user_002"), Some(200));
        assert_eq!(cache.get_id_index(2, "product_001"), Some(300));
        assert_eq!(cache.get_id_index(1, "nonexistent"), None);

        cache.remove_id_index(1, "user_001");
        assert_eq!(cache.get_id_index(1, "user_001"), None);
        assert_eq!(cache.get_id_index(1, "user_002"), Some(200));
    }

    #[test]
    fn test_cache_config_with_ttl() {
        use std::time::Duration;

        let config = RecordCacheConfig {
            max_memory: 1024 * 1024,
            memory_ratio: (40, 30, 20, 10),
            ttl: Some(Duration::from_secs(60)),
            tti: Some(Duration::from_secs(30)),
        };
        let cache = RecordCache::with_config(config);

        let key = VertexCacheKey::new(1, 100, 1000);
        let vertex = CachedVertex {
            internal_id: 100,
            external_id: "test".to_string(),
            properties: vec![],
        };
        cache.insert_vertex(key, vertex);

        assert!(cache.get_vertex(&VertexCacheKey::new(1, 100, 1000)).is_some());
    }
}
