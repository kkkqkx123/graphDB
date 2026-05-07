//! Record Cache
//!
//! High-performance cache for vertex and edge records using Moka.
//! Provides O(1) operations with TinyLFU eviction policy for optimal hit rate.
//!
//! # Features
//!
//! - Fine-grained statistics per cache type (vertex, edge_query, id_index)
//! - Eviction listener support for custom cleanup logic
//! - High priority pool configuration for index cache protection
//! - TTL/TTI expiration support
//! - Memory-weighted eviction
//! - Batch operations for improved throughput
//! - Memory pressure response
//!
//! # Architecture
//!
//! This cache manages three distinct cache types as a unified facade:
//!
//! - **Vertex Cache**: Stores vertex records keyed by (label_id, internal_id)
//! - **Edge Query Cache**: Stores edge query results keyed by (edge_label_id, src_vid, dst_vid)
//! - **ID Index Cache**: Stores external_id to internal_id mappings keyed by (label_id, external_id)
//!
//! ## Design Rationale
//!
//! The three cache types are managed together because:
//! - They share a unified memory budget and configuration
//! - They require coordinated invalidation during updates
//! - They share eviction callback infrastructure
//! - This provides a simpler API for the storage layer
//!
//! ## Memory Allocation
//!
//! Memory is distributed across caches based on `memory_ratio` configuration:
//! - Default ratio: (40% vertex, 40% edge_query, 20% id_index)
//! - High priority pool can add extra memory to id_index cache

use std::sync::Arc;

use moka::sync::Cache;
use parking_lot::RwLock;

use crate::storage::memory::MemoryTracker;

use super::types::*;
use super::stats::*;
use super::config::*;
use super::batch::*;

pub struct RecordCache {
    vertex_cache: Cache<VertexCacheKey, CachedVertex>,
    edge_query_cache: Cache<EdgeQueryKey, CachedEdge>,
    id_index_cache: Cache<IdIndexCacheKey, u32>,
    config: RecordCacheConfig,
    vertex_stats: Arc<CacheTypeStats>,
    edge_query_stats: Arc<CacheTypeStats>,
    id_index_stats: Arc<CacheTypeStats>,
    eviction_callback: Arc<RwLock<Option<EvictionCallback>>>,
    memory_tracker: Option<Arc<MemoryTracker>>,
    memory_pressure_config: MemoryPressureConfig,
    original_max_memory: usize,
}

struct CacheMemoryAllocation {
    vertex_memory: u64,
    edge_query_memory: u64,
    id_index_memory: u64,
}

impl std::fmt::Debug for RecordCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RecordCache")
            .field("config", &self.config)
            .field("vertex_count", &self.vertex_cache.entry_count())
            .field("edge_query_count", &self.edge_query_cache.entry_count())
            .field("id_index_count", &self.id_index_cache.entry_count())
            .field("vertex_stats", &self.vertex_stats)
            .field("edge_query_stats", &self.edge_query_stats)
            .field("id_index_stats", &self.id_index_stats)
            .finish()
    }
}

impl RecordCache {
    pub fn new() -> Self {
        Self::with_config(RecordCacheConfig::default())
    }

    pub fn with_config(config: RecordCacheConfig) -> Self {
        let memory_allocation = Self::calculate_memory_allocation(&config);

        let vertex_stats = Arc::new(CacheTypeStats::new());
        let edge_query_stats = Arc::new(CacheTypeStats::new());
        let id_index_stats = Arc::new(CacheTypeStats::new());

        let eviction_callback = Arc::new(RwLock::new(None::<EvictionCallback>));

        let vertex_cache = Self::build_vertex_cache(
            memory_allocation.vertex_memory,
            vertex_stats.clone(),
            eviction_callback.clone(),
            config.ttl,
            config.tti,
        );

        let edge_query_cache = Self::build_edge_query_cache(
            memory_allocation.edge_query_memory,
            edge_query_stats.clone(),
            eviction_callback.clone(),
            config.ttl,
            config.tti,
        );

        let id_index_cache = Self::build_id_index_cache(
            memory_allocation.id_index_memory,
            id_index_stats.clone(),
            eviction_callback.clone(),
            config.ttl,
            config.tti,
        );

        let original_max_memory = config.max_memory;

        Self {
            vertex_cache,
            edge_query_cache,
            id_index_cache,
            config,
            vertex_stats,
            edge_query_stats,
            id_index_stats,
            eviction_callback,
            memory_tracker: None,
            memory_pressure_config: MemoryPressureConfig::default(),
            original_max_memory,
        }
    }

    fn calculate_memory_allocation(config: &RecordCacheConfig) -> CacheMemoryAllocation {
        let max_memory = config.max_memory as u64;
        let total_ratio =
            config.memory_ratio.0 + config.memory_ratio.1 + config.memory_ratio.2 + config.memory_ratio.3;

        let base_vertex_memory = max_memory * config.memory_ratio.0 as u64 / total_ratio as u64;
        let base_edge_query_memory = max_memory * config.memory_ratio.2 as u64 / total_ratio as u64;
        let base_id_index_memory = max_memory * config.memory_ratio.3 as u64 / total_ratio as u64;

        let high_priority_extra = if config.high_priority_ratio > 0.0 {
            (max_memory as f64 * config.high_priority_ratio as f64) as u64
        } else {
            0
        };

        CacheMemoryAllocation {
            vertex_memory: base_vertex_memory,
            edge_query_memory: base_edge_query_memory,
            id_index_memory: base_id_index_memory + high_priority_extra,
        }
    }

    fn build_vertex_cache(
        max_capacity: u64,
        stats: Arc<CacheTypeStats>,
        eviction_callback: Arc<RwLock<Option<EvictionCallback>>>,
        ttl: Option<std::time::Duration>,
        tti: Option<std::time::Duration>,
    ) -> Cache<VertexCacheKey, CachedVertex> {
        let mut builder = Cache::builder()
            .max_capacity(max_capacity)
            .weigher(|_key: &VertexCacheKey, value: &CachedVertex| value.estimated_size())
            .eviction_listener(move |_key, _value, cause| {
                stats.record_eviction();
                let cause = EvictionCause::from(cause);
                if let Some(ref callback) = *eviction_callback.read() {
                    callback("vertex", cause);
                }
            });

        if let Some(duration) = ttl {
            builder = builder.time_to_live(duration);
        }
        if let Some(duration) = tti {
            builder = builder.time_to_idle(duration);
        }

        builder.build()
    }

    fn build_edge_query_cache(
        max_capacity: u64,
        stats: Arc<CacheTypeStats>,
        eviction_callback: Arc<RwLock<Option<EvictionCallback>>>,
        ttl: Option<std::time::Duration>,
        tti: Option<std::time::Duration>,
    ) -> Cache<EdgeQueryKey, CachedEdge> {
        let mut builder = Cache::builder()
            .max_capacity(max_capacity)
            .weigher(|_key: &EdgeQueryKey, value: &CachedEdge| value.estimated_size())
            .eviction_listener(move |_key, _value, cause| {
                stats.record_eviction();
                let cause = EvictionCause::from(cause);
                if let Some(ref callback) = *eviction_callback.read() {
                    callback("edge_query", cause);
                }
            });

        if let Some(duration) = ttl {
            builder = builder.time_to_live(duration);
        }
        if let Some(duration) = tti {
            builder = builder.time_to_idle(duration);
        }

        builder.build()
    }

    fn build_id_index_cache(
        max_capacity: u64,
        stats: Arc<CacheTypeStats>,
        eviction_callback: Arc<RwLock<Option<EvictionCallback>>>,
        ttl: Option<std::time::Duration>,
        tti: Option<std::time::Duration>,
    ) -> Cache<IdIndexCacheKey, u32> {
        let mut builder = Cache::builder()
            .max_capacity(max_capacity)
            .weigher(|_key: &IdIndexCacheKey, _value: &u32| std::mem::size_of::<u32>() as u32)
            .eviction_listener(move |_key, _value, cause| {
                stats.record_eviction();
                let cause = EvictionCause::from(cause);
                if let Some(ref callback) = *eviction_callback.read() {
                    callback("id_index", cause);
                }
            });

        if let Some(duration) = ttl {
            builder = builder.time_to_live(duration);
        }
        if let Some(duration) = tti {
            builder = builder.time_to_idle(duration);
        }

        builder.build()
    }

    pub fn with_memory_tracker(mut self, tracker: Arc<MemoryTracker>) -> Self {
        self.memory_tracker = Some(tracker);
        self
    }

    pub fn with_eviction_callback(self, callback: EvictionCallback) -> Self {
        *self.eviction_callback.write() = Some(callback);
        self
    }

    pub fn set_eviction_callback(&self, callback: EvictionCallback) {
        *self.eviction_callback.write() = Some(callback);
    }

    pub fn with_memory_pressure_config(mut self, config: MemoryPressureConfig) -> Self {
        self.memory_pressure_config = config;
        self
    }

    pub fn set_memory_pressure_config(&mut self, config: MemoryPressureConfig) {
        self.memory_pressure_config = config;
    }

    fn notify_eviction(&self, cache_type: &str, cause: EvictionCause) {
        if let Some(ref callback) = *self.eviction_callback.read() {
            callback(cache_type, cause);
        }
    }

    // ==================== Basic Operations ====================

    pub fn get_id_index(&self, label_id: u16, external_id: &str) -> Option<u32> {
        let key = IdIndexCacheKey::new(label_id, external_id.to_string());
        match self.id_index_cache.get(&key) {
            Some(internal_id) => {
                self.id_index_stats.record_hit();
                Some(internal_id)
            }
            None => {
                self.id_index_stats.record_miss();
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
            self.notify_eviction("id_index", EvictionCause::Explicit);
            if let Some(ref tracker) = self.memory_tracker {
                tracker.release_cache(std::mem::size_of::<u32>());
            }
        }
    }

    pub fn get_vertex(&self, key: &VertexCacheKey) -> Option<CachedVertex> {
        match self.vertex_cache.get(key) {
            Some(vertex) => {
                self.vertex_stats.record_hit();
                Some(vertex)
            }
            None => {
                self.vertex_stats.record_miss();
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
            self.notify_eviction("vertex", EvictionCause::Explicit);
            let size = vertex.estimated_size() as usize;
            if let Some(ref tracker) = self.memory_tracker {
                tracker.release_cache(size);
            }
        }
    }

    pub fn get_edge_by_query(&self, key: &EdgeQueryKey) -> Option<CachedEdge> {
        match self.edge_query_cache.get(key) {
            Some(edge) => {
                self.edge_query_stats.record_hit();
                Some(edge)
            }
            None => {
                self.edge_query_stats.record_miss();
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
            self.notify_eviction("edge_query", EvictionCause::Explicit);
            let size = edge.estimated_size() as usize;
            if let Some(ref tracker) = self.memory_tracker {
                tracker.release_cache(size);
            }
        }
    }

    // ==================== Invalidation ====================

    pub fn invalidate_vertices_by_label(&self, label_id: u16) {
        let _ = self.vertex_cache.invalidate_entries_if(move |k, _| k.label_id == label_id);
    }

    pub fn invalidate_edges_by_label(&self, edge_label_id: u16) {
        let _ = self.edge_query_cache.invalidate_entries_if(move |k, _| k.edge_label_id == edge_label_id);
    }

    pub fn invalidate_edges_by_src(&self, src_vid: u64) {
        let _ = self.edge_query_cache.invalidate_entries_if(move |k, _| k.src_vid == src_vid);
    }

    pub fn invalidate_edges_by_dst(&self, dst_vid: u64) {
        let _ = self.edge_query_cache.invalidate_entries_if(move |k, _| k.dst_vid == dst_vid);
    }

    pub fn invalidate_id_indexes_by_label(&self, label_id: u16) {
        let _ = self.id_index_cache.invalidate_entries_if(move |k, _| k.label_id == label_id);
    }

    pub fn clear(&self) {
        self.vertex_cache.invalidate_all();
        self.edge_query_cache.invalidate_all();
        self.id_index_cache.invalidate_all();
    }

    // ==================== Statistics ====================

    pub fn memory_usage(&self) -> usize {
        (self.vertex_cache.weighted_size()
            + self.edge_query_cache.weighted_size()
            + self.id_index_cache.weighted_size()) as usize
    }

    pub fn max_memory(&self) -> usize {
        self.config.max_memory
    }

    pub fn stats(&self) -> RecordCacheStats {
        let vertex_snapshot = CacheTypeStatsSnapshot::from_stats(
            &self.vertex_stats,
            self.vertex_cache.entry_count(),
            self.vertex_cache.weighted_size(),
        );
        let edge_query_snapshot = CacheTypeStatsSnapshot::from_stats(
            &self.edge_query_stats,
            self.edge_query_cache.entry_count(),
            self.edge_query_cache.weighted_size(),
        );
        let id_index_snapshot = CacheTypeStatsSnapshot::from_stats(
            &self.id_index_stats,
            self.id_index_cache.entry_count(),
            self.id_index_cache.weighted_size(),
        );

        let total_hits = vertex_snapshot.hits + edge_query_snapshot.hits + id_index_snapshot.hits;
        let total_misses = vertex_snapshot.misses + edge_query_snapshot.misses + id_index_snapshot.misses;
        let total_evictions = vertex_snapshot.evictions + edge_query_snapshot.evictions + id_index_snapshot.evictions;

        RecordCacheStats {
            vertex: vertex_snapshot,
            edge_query: edge_query_snapshot,
            id_index: id_index_snapshot,
            total_hits,
            total_misses,
            total_evictions,
            hit_rate: if total_hits + total_misses > 0 {
                total_hits as f64 / (total_hits + total_misses) as f64
            } else {
                0.0
            },
            memory_usage: self.memory_usage(),
            max_memory: self.config.max_memory,
        }
    }

    pub fn utilization(&self) -> f32 {
        if self.config.max_memory == 0 {
            return 0.0;
        }
        self.memory_usage() as f32 / self.config.max_memory as f32
    }

    pub fn vertex_stats(&self) -> &CacheTypeStats {
        &self.vertex_stats
    }

    pub fn edge_query_stats(&self) -> &CacheTypeStats {
        &self.edge_query_stats
    }

    pub fn id_index_stats(&self) -> &CacheTypeStats {
        &self.id_index_stats
    }

    // ==================== Batch Operations ====================

    pub fn get_vertices_batch(&self, keys: &[VertexCacheKey]) -> BatchGetResult<CachedVertex> {
        let mut results = Vec::with_capacity(keys.len());
        let mut hits = 0usize;
        let mut misses = 0usize;

        for key in keys {
            match self.vertex_cache.get(key) {
                Some(vertex) => {
                    self.vertex_stats.record_hit();
                    hits += 1;
                    results.push(Some(vertex));
                }
                None => {
                    self.vertex_stats.record_miss();
                    misses += 1;
                    results.push(None);
                }
            }
        }

        BatchGetResult { results, hits, misses }
    }

    pub fn insert_vertices_batch(&self, entries: Vec<(VertexCacheKey, CachedVertex)>) -> BatchInsertResult {
        let mut total_size = 0usize;

        for (key, vertex) in entries {
            let size = vertex.estimated_size() as usize;
            total_size += size;
            self.vertex_cache.insert(key, vertex);
        }

        if let Some(ref tracker) = self.memory_tracker {
            tracker.try_allocate_cache(total_size);
        }

        BatchInsertResult {
            inserted: self.vertex_cache.entry_count() as usize,
            total_size,
        }
    }

    pub fn get_edge_queries_batch(&self, keys: &[EdgeQueryKey]) -> BatchGetResult<CachedEdge> {
        let mut results = Vec::with_capacity(keys.len());
        let mut hits = 0usize;
        let mut misses = 0usize;

        for key in keys {
            match self.edge_query_cache.get(key) {
                Some(edge) => {
                    self.edge_query_stats.record_hit();
                    hits += 1;
                    results.push(Some(edge));
                }
                None => {
                    self.edge_query_stats.record_miss();
                    misses += 1;
                    results.push(None);
                }
            }
        }

        BatchGetResult { results, hits, misses }
    }

    pub fn insert_edge_queries_batch(&self, entries: Vec<(EdgeQueryKey, CachedEdge)>) -> BatchInsertResult {
        let mut total_size = 0usize;

        for (key, edge) in entries {
            let size = edge.estimated_size() as usize;
            total_size += size;
            self.edge_query_cache.insert(key, edge);
        }

        if let Some(ref tracker) = self.memory_tracker {
            tracker.try_allocate_cache(total_size);
        }

        BatchInsertResult {
            inserted: self.edge_query_cache.entry_count() as usize,
            total_size,
        }
    }

    pub fn invalidate_batch(&self, keys: &[CacheKeyRef<'_>]) -> usize {
        let mut invalidated = 0usize;

        for key in keys {
            match key {
                CacheKeyRef::Vertex(k) => {
                    if self.vertex_cache.remove(k).is_some() {
                        invalidated += 1;
                    }
                }
                CacheKeyRef::EdgeQuery(k) => {
                    if self.edge_query_cache.remove(k).is_some() {
                        invalidated += 1;
                    }
                }
                CacheKeyRef::IdIndex(label_id, external_id) => {
                    let cache_key = IdIndexCacheKey::new(*label_id, external_id.to_string());
                    if self.id_index_cache.remove(&cache_key).is_some() {
                        invalidated += 1;
                    }
                }
            }
        }

        invalidated
    }

    // ==================== Memory Pressure ====================

    pub fn handle_memory_pressure(&mut self, level: MemoryPressureLevel) {
        if !self.memory_pressure_config.enabled {
            return;
        }

        match level {
            MemoryPressureLevel::Normal => {}
            MemoryPressureLevel::Warning => {
                self.reduce_memory(self.memory_pressure_config.reduction_factor);
            }
            MemoryPressureLevel::Critical => {
                self.clear();
            }
        }
    }

    fn reduce_memory(&mut self, factor: f32) {
        let new_max = (self.original_max_memory as f32 * factor) as usize;
        self.config.max_memory = new_max;
    }

    pub fn memory_pressure_level(&self) -> MemoryPressureLevel {
        let utilization = self.utilization();
        if utilization >= self.memory_pressure_config.high_watermark {
            MemoryPressureLevel::Critical
        } else if utilization >= self.memory_pressure_config.low_watermark {
            MemoryPressureLevel::Warning
        } else {
            MemoryPressureLevel::Normal
        }
    }
}

impl Default for RecordCache {
    fn default() -> Self {
        Self::new()
    }
}

pub type SharedRecordCache = Arc<RecordCache>;
