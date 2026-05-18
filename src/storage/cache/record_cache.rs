//! Record Cache
//!
//! High-performance cache for vertex records using Moka.
//! Provides O(1) operations with TinyLFU eviction policy for optimal hit rate.
//!
//! ## Features
//!
//! - Fine-grained statistics per cache type (vertex, id_index)
//! - Eviction listener support for custom cleanup logic
//! - High priority pool configuration for index cache protection
//! - TTL/TTI expiration support
//! - Memory-weighted eviction
//! - Batch operations for improved throughput
//! - Memory pressure response
//!
//! ## Architecture
//!
//! This cache manages two distinct cache types as a unified facade:
//!
//! - **Vertex Cache**: Stores vertex records keyed by (label_id, internal_id)
//! - **ID Index Cache**: Stores external_id to internal_id mappings keyed by (label_id, external_id)
//!
//! ## Design Note: Why No Edge Cache?
//!
//! Edge data is NOT cached because:
//!
//! 1. **CSR is already read-optimized**: The CSR structure provides O(1) edge list
//!    access with contiguous memory layout, which is CPU cache-friendly.
//!
//! 2. **High memory cost**: Edge data volume is typically much larger than vertex data.
//!
//! 3. **Frequent updates**: Edges are updated more frequently than vertices.
//!
//! 4. **Property access is O(1)**: Edge properties are stored in PropertyTable
//!    with direct offset access.

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

use moka::sync::Cache;
use parking_lot::Mutex;

use crate::core::stats::CacheStats;
use crate::core::stats::StatsManager;

use super::batch::*;
use super::config::*;
use super::stats::*;
use super::types::*;

/// Record cache for vertex data and ID index mappings
pub struct RecordCache {
    vertex_cache: Cache<VertexCacheKey, CachedVertex>,
    id_index_cache: Cache<IdIndexCacheKey, u32>,
    config: Mutex<RecordCacheConfig>,
    vertex_stats: Arc<CacheStats>,
    id_index_stats: Arc<CacheStats>,
    eviction_callback: Arc<Mutex<Option<EvictionCallback>>>,
    memory_pressure_config: MemoryPressureConfig,
    original_max_memory: usize,
    current_max_memory: AtomicUsize,
    stats_manager: Option<Arc<StatsManager>>,
    transaction_snapshot: Arc<Mutex<Option<TransactionCacheSnapshot>>>,
    created_at: Instant,
}

struct CacheMemoryAllocation {
    vertex_memory: u64,
    id_index_memory: u64,
}

impl std::fmt::Debug for RecordCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RecordCache")
            .field("config", &self.config)
            .field("vertex_count", &self.vertex_cache.entry_count())
            .field("id_index_count", &self.id_index_cache.entry_count())
            .field("vertex_stats", &self.vertex_stats)
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

        let vertex_stats = Arc::new(CacheStats::new());
        let id_index_stats = Arc::new(CacheStats::new());

        let eviction_callback = Arc::new(Mutex::new(None::<EvictionCallback>));

        let vertex_cache = Self::build_vertex_cache(
            memory_allocation.vertex_memory,
            vertex_stats.clone(),
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
            id_index_cache,
            config: Mutex::new(config),
            vertex_stats,
            id_index_stats,
            eviction_callback,
            memory_pressure_config: MemoryPressureConfig::default(),
            original_max_memory,
            current_max_memory: AtomicUsize::new(original_max_memory),
            stats_manager: None,
            transaction_snapshot: Arc::new(Mutex::new(None)),
            created_at: Instant::now(),
        }
    }

    fn calculate_memory_allocation(config: &RecordCacheConfig) -> CacheMemoryAllocation {
        let max_memory = config.max_memory as u64;
        let total_ratio = config.memory_ratio.0 + config.memory_ratio.1;

        let base_vertex_memory = max_memory * config.memory_ratio.0 as u64 / total_ratio as u64;
        let base_id_index_memory = max_memory * config.memory_ratio.1 as u64 / total_ratio as u64;

        let high_priority_extra = if config.high_priority_ratio > 0.0 {
            (max_memory as f64 * config.high_priority_ratio as f64) as u64
        } else {
            0
        };

        CacheMemoryAllocation {
            vertex_memory: base_vertex_memory,
            id_index_memory: base_id_index_memory + high_priority_extra,
        }
    }

    fn build_vertex_cache(
        max_capacity: u64,
        stats: Arc<CacheStats>,
        eviction_callback: Arc<Mutex<Option<EvictionCallback>>>,
        ttl: Option<std::time::Duration>,
        tti: Option<std::time::Duration>,
    ) -> Cache<VertexCacheKey, CachedVertex> {
        let mut builder = Cache::builder()
            .max_capacity(max_capacity)
            .weigher(|_key: &VertexCacheKey, value: &CachedVertex| {
                let key_size = std::mem::size_of::<VertexCacheKey>() as u32;
                let value_size = value.estimated_size();
                key_size.wrapping_add(value_size)
            })
            .support_invalidation_closures()
            .eviction_listener(move |_key, _value, cause| {
                stats.record_eviction();
                let cause = EvictionCause::from(cause);
                if let Some(ref callback) = *eviction_callback.lock() {
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

    fn build_id_index_cache(
        max_capacity: u64,
        stats: Arc<CacheStats>,
        eviction_callback: Arc<Mutex<Option<EvictionCallback>>>,
        ttl: Option<std::time::Duration>,
        tti: Option<std::time::Duration>,
    ) -> Cache<IdIndexCacheKey, u32> {
        let mut builder = Cache::builder()
            .max_capacity(max_capacity)
            .weigher(|key: &IdIndexCacheKey, _value: &u32| {
                let key_size = std::mem::size_of::<IdIndexCacheKey>() + key.external_id.len();
                let value_size = std::mem::size_of::<u32>();
                (key_size + value_size) as u32
            })
            .support_invalidation_closures()
            .eviction_listener(move |_key, _value, cause| {
                stats.record_eviction();
                let cause = EvictionCause::from(cause);
                if let Some(ref callback) = *eviction_callback.lock() {
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

    pub fn with_eviction_callback(self, callback: EvictionCallback) -> Self {
        *self.eviction_callback.lock() = Some(callback);
        self
    }

    pub fn set_eviction_callback(&self, callback: EvictionCallback) {
        *self.eviction_callback.lock() = Some(callback);
    }

    pub fn with_memory_pressure_config(mut self, config: MemoryPressureConfig) -> Self {
        self.memory_pressure_config = config;
        self
    }

    pub fn with_stats_manager(mut self, stats_manager: Arc<StatsManager>) -> Self {
        self.stats_manager = Some(stats_manager);
        self
    }

    pub fn set_memory_pressure_config(&mut self, config: MemoryPressureConfig) {
        self.memory_pressure_config = config;
    }

    fn notify_eviction(&self, cache_type: &str, cause: EvictionCause) {
        if let Some(ref callback) = *self.eviction_callback.lock() {
            callback(cache_type, cause);
        }
    }

    // ==================== ID Index Operations ====================

    pub fn get_id_index(&self, label_id: u32, external_id: &str) -> Option<u32> {
        let key = IdIndexCacheKey::new(label_id, Arc::from(external_id));
        let result = match self.id_index_cache.get(&key) {
            Some(internal_id) => {
                self.id_index_stats.record_hit();
                Some(internal_id)
            }
            None => {
                self.id_index_stats.record_miss();
                None
            }
        };
        if let Some(ref sm) = self.stats_manager {
            sm.record_storage_cache_hit(result.is_some());
        }
        result
    }

    pub fn insert_id_index(&self, label_id: u32, external_id: &str, internal_id: u32) {
        let key = IdIndexCacheKey::new(label_id, Arc::from(external_id));
        if let Some(ref mut snapshot) = *self.transaction_snapshot.lock() {
            let old_value = self.id_index_cache.get(&key);
            snapshot.record_id_index(key.clone(), old_value);
        }
        self.id_index_cache.insert(key, internal_id);
    }

    pub fn remove_id_index(&self, label_id: u32, external_id: &str) {
        let key = IdIndexCacheKey::new(label_id, Arc::from(external_id));
        if let Some(ref mut snapshot) = *self.transaction_snapshot.lock() {
            let old_value = self.id_index_cache.get(&key);
            snapshot.record_id_index(key.clone(), old_value);
        }
        if self.id_index_cache.remove(&key).is_some() {
            self.notify_eviction("id_index", EvictionCause::Explicit);
        }
    }

    // ==================== Vertex Operations ====================

    pub fn get_vertex(&self, key: &VertexCacheKey) -> Option<CachedVertex> {
        let result = match self.vertex_cache.get(key) {
            Some(vertex) => {
                self.vertex_stats.record_hit();
                Some(vertex)
            }
            None => {
                self.vertex_stats.record_miss();
                None
            }
        };
        if let Some(ref sm) = self.stats_manager {
            sm.record_storage_cache_hit(result.is_some());
        }
        result
    }

    pub fn insert_vertex(&self, key: VertexCacheKey, vertex: CachedVertex) {
        if let Some(ref mut snapshot) = *self.transaction_snapshot.lock() {
            let old_value = self.vertex_cache.get(&key);
            snapshot.record_vertex(key, old_value);
        }
        self.vertex_cache.insert(key, vertex);
    }

    pub fn remove_vertex(&self, key: &VertexCacheKey) {
        if let Some(ref mut snapshot) = *self.transaction_snapshot.lock() {
            let old_value = self.vertex_cache.get(key);
            snapshot.record_vertex(*key, old_value);
        }
        if let Some(_vertex) = self.vertex_cache.remove(key) {
            self.notify_eviction("vertex", EvictionCause::Explicit);
        }
    }

    // ==================== Invalidation ====================

    pub fn invalidate_vertices_by_label(&self, label_id: u32) {
        let _ = self
            .vertex_cache
            .invalidate_entries_if(move |k, _| k.label_id == label_id);
        self.vertex_cache.run_pending_tasks();
    }

    pub fn invalidate_id_indexes_by_label(&self, label_id: u32) {
        let _ = self
            .id_index_cache
            .invalidate_entries_if(move |k, _| k.label_id == label_id);
        self.id_index_cache.run_pending_tasks();
    }

    pub fn clear(&self) {
        self.vertex_cache.invalidate_all();
        self.id_index_cache.invalidate_all();
        self.vertex_cache.run_pending_tasks();
        self.id_index_cache.run_pending_tasks();
    }

    pub fn run_pending_tasks(&self) {
        self.vertex_cache.run_pending_tasks();
        self.id_index_cache.run_pending_tasks();
    }

    // ==================== Statistics ====================

    pub fn memory_usage(&self) -> usize {
        (self.vertex_cache.weighted_size() + self.id_index_cache.weighted_size()) as usize
    }

    pub fn max_memory(&self) -> usize {
        self.current_max_memory.load(Ordering::Relaxed)
    }

    pub fn stats(&self) -> RecordCacheStats {
        let vertex_weighted = self.vertex_cache.weighted_size();
        let id_index_weighted = self.id_index_cache.weighted_size();

        let vertex_snapshot = CacheTypeStatsSnapshot::from_stats(
            &self.vertex_stats,
            self.vertex_cache.entry_count(),
            vertex_weighted,
        );
        let id_index_snapshot = CacheTypeStatsSnapshot::from_stats(
            &self.id_index_stats,
            self.id_index_cache.entry_count(),
            id_index_weighted,
        );

        let total_hits = vertex_snapshot.hits + id_index_snapshot.hits;
        let total_misses = vertex_snapshot.misses + id_index_snapshot.misses;
        let total_evictions = vertex_snapshot.evictions + id_index_snapshot.evictions;
        let total_operations = total_hits + total_misses + total_evictions;

        let memory_usage = (vertex_weighted + id_index_weighted) as usize;
        let max_memory = self.current_max_memory.load(Ordering::Relaxed);

        // Estimate memory fragmentation: ratio of weighted size to expected size
        let vertex_count = self.vertex_cache.entry_count();
        let id_index_count = self.id_index_cache.entry_count();
        let expected_vertex_memory = vertex_count * std::mem::size_of::<CachedVertex>() as u64;
        let expected_id_index_memory =
            id_index_count * std::mem::size_of::<IdIndexCacheKey>() as u64;
        let fragmentation = if expected_vertex_memory + expected_id_index_memory > 0 {
            1.0 - (expected_vertex_memory + expected_id_index_memory) as f64
                / (vertex_weighted + id_index_weighted).max(1) as f64
        } else {
            0.0
        };

        RecordCacheStats {
            vertex: vertex_snapshot,
            id_index: id_index_snapshot,
            total_hits,
            total_misses,
            total_evictions,
            hit_rate: if total_hits + total_misses > 0 {
                total_hits as f64 / (total_hits + total_misses) as f64
            } else {
                0.0
            },
            eviction_rate: if total_operations > 0 {
                total_evictions as f64 / total_operations as f64
            } else {
                0.0
            },
            memory_usage,
            max_memory,
            uptime_seconds: self.created_at.elapsed().as_secs(),
            memory_fragmentation_estimate: fragmentation,
            vertex_memory_bytes: vertex_weighted,
            id_index_memory_bytes: id_index_weighted,
        }
    }

    pub fn utilization(&self) -> f32 {
        let max_memory = self.current_max_memory.load(Ordering::Relaxed);
        if max_memory == 0 {
            return 0.0;
        }
        self.memory_usage() as f32 / max_memory as f32
    }

    pub fn vertex_stats(&self) -> &CacheStats {
        &self.vertex_stats
    }

    pub fn id_index_stats(&self) -> &CacheStats {
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

        BatchGetResult {
            results,
            hits,
            misses,
        }
    }

    pub fn insert_vertices_batch(
        &self,
        entries: Vec<(VertexCacheKey, CachedVertex)>,
    ) -> BatchInsertResult {
        let mut total_size = 0usize;
        let mut pre_checked = Vec::with_capacity(entries.len());

        for (key, vertex) in entries {
            let size = vertex.estimated_size() as usize;
            total_size += size;
            pre_checked.push((key, vertex, size));
        }

        let mut inserted = 0usize;
        for (key, vertex, _size) in pre_checked {
            self.vertex_cache.insert(key, vertex);
            inserted += 1;
        }

        BatchInsertResult {
            inserted,
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
                CacheKeyRef::IdIndex(label_id, external_id) => {
                    let cache_key = IdIndexCacheKey::new(*label_id, Arc::from(*external_id));
                    if self.id_index_cache.remove(&cache_key).is_some() {
                        invalidated += 1;
                    }
                }
            }
        }

        invalidated
    }

    // ==================== Memory Pressure ====================

    pub fn handle_memory_pressure(&self, level: MemoryPressureLevel) {
        if !self.memory_pressure_config.enabled {
            return;
        }

        match level {
            MemoryPressureLevel::Normal => {}
            MemoryPressureLevel::Warning => {
                let factor = self.memory_pressure_config.reduction_factor;
                let retain_ratio = factor.max(0.1_f32);

                let vertex_target = (self.vertex_cache.entry_count() as f32 * retain_ratio) as u64;
                let id_index_target =
                    (self.id_index_cache.entry_count() as f32 * retain_ratio) as u64;

                let vertex_count = AtomicU64::new(0);
                let _ = self.vertex_cache.invalidate_entries_if(move |_, _| {
                    let c = vertex_count.fetch_add(1, Ordering::Relaxed);
                    c >= vertex_target
                });

                let id_index_count = AtomicU64::new(0);
                let _ = self.id_index_cache.invalidate_entries_if(move |_, _| {
                    let c = id_index_count.fetch_add(1, Ordering::Relaxed);
                    c >= id_index_target
                });

                self.vertex_cache.run_pending_tasks();
                self.id_index_cache.run_pending_tasks();

                let mut config = self.config.lock();
                let new_max = (config.max_memory as f64 * factor as f64) as usize;
                config.max_memory = new_max;
                self.current_max_memory.store(new_max, Ordering::Relaxed);
            }
            MemoryPressureLevel::Critical => {
                self.clear();
                let mut config = self.config.lock();
                config.max_memory = 0;
                self.current_max_memory.store(0, Ordering::Relaxed);
            }
        }
    }

    pub fn restore_memory(&self) {
        self.current_max_memory
            .fetch_max(self.original_max_memory, Ordering::Relaxed);
    }

    // ==================== Runtime Configuration ====================

    pub fn update_config(&self, new_config: RecordCacheConfig) {
        let mut config = self.config.lock();
        config.max_memory = new_config.max_memory;
        config.memory_ratio = new_config.memory_ratio;
        config.high_priority_ratio = new_config.high_priority_ratio;
        self.current_max_memory
            .store(new_config.max_memory, Ordering::Relaxed);
    }

    pub fn set_max_memory(&self, max_memory: usize) {
        self.config.lock().max_memory = max_memory;
        self.current_max_memory.store(max_memory, Ordering::Relaxed);
    }

    pub fn set_memory_ratio(&self, vertex_ratio: u32, id_index_ratio: u32) {
        self.config.lock().memory_ratio = (vertex_ratio, id_index_ratio);
    }

    pub fn config(&self) -> RecordCacheConfig {
        self.config.lock().clone()
    }

    // ==================== Transaction Support ====================

    pub fn begin_transaction(&self) {
        *self.transaction_snapshot.lock() = Some(TransactionCacheSnapshot::new());
    }

    pub fn commit_transaction(&self) {
        *self.transaction_snapshot.lock() = None;
    }

    pub fn rollback_transaction(&self) {
        let snapshot = self.transaction_snapshot.lock().take();
        if let Some(snapshot) = snapshot {
            for entry in snapshot.into_entries() {
                match entry {
                    CacheSnapshotEntry::Vertex(key, Some(old_value)) => {
                        self.vertex_cache.insert(key, old_value);
                    }
                    CacheSnapshotEntry::Vertex(key, None) => {
                        self.vertex_cache.remove(&key);
                    }
                    CacheSnapshotEntry::IdIndex(key, Some(old_value)) => {
                        self.id_index_cache.insert(key, old_value);
                    }
                    CacheSnapshotEntry::IdIndex(key, None) => {
                        self.id_index_cache.remove(&key);
                    }
                }
            }
        }
    }
}

impl Default for RecordCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Shared record cache type alias
pub type SharedRecordCache = Arc<RecordCache>;
