//! Cache Manager
//!
//! Manages record cache, edge property cache, and memory tracking for the storage engine.
//!
//! ## Design Note: Edge Property Cache
//!
//! Edge property cache is optional and disabled by default. Enable it when:
//! - Edge property access frequency exceeds threshold
//! - Property size is small (< 1KB)
//! - Edge update frequency is low
//!
//! Edge data structure (CSR) is NOT cached separately because:
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

use std::sync::Arc;
use std::sync::Mutex;

use crate::core::types::{EdgeId, LabelId};
use crate::core::Value;
use crate::storage::cache::{
    CachedVertex, EdgePropertyCache, EdgePropertyCacheConfig, EdgePropertyCacheStats,
    RecordCache, RecordCacheConfig, RecordCacheStats, SharedRecordCache, VertexCacheKey,
};
use crate::storage::memory::SharedMemoryTracker;

/// Manager for storage caches
pub struct CacheManager {
    pub record_cache: Option<SharedRecordCache>,
    pub edge_property_cache: Mutex<Option<Arc<EdgePropertyCache>>>,
    pub memory_tracker: Option<SharedMemoryTracker>,
}

impl CacheManager {
    pub fn new(enable_cache: bool, cache_memory: usize, memory_tracker: SharedMemoryTracker) -> Self {
        let record_cache = if enable_cache {
            let config = RecordCacheConfig {
                max_memory: cache_memory,
                ..Default::default()
            };
            Some(SharedRecordCache::new(
                RecordCache::with_config(config)
                    .with_memory_tracker(memory_tracker.clone()),
            ))
        } else {
            None
        };

        Self {
            record_cache,
            edge_property_cache: Mutex::new(None),
            memory_tracker: Some(memory_tracker),
        }
    }

    pub fn with_edge_property_cache(self, config: EdgePropertyCacheConfig) -> Self {
        if config.enabled {
            *self.edge_property_cache.lock().unwrap() = Some(Arc::new(EdgePropertyCache::new(config)));
        }
        self
    }

    pub fn set_edge_property_cache(&self, config: EdgePropertyCacheConfig) {
        if config.enabled {
            *self.edge_property_cache.lock().unwrap() = Some(Arc::new(EdgePropertyCache::new(config)));
        } else {
            *self.edge_property_cache.lock().unwrap() = None;
        }
    }

    pub fn record_cache(&self) -> Option<&SharedRecordCache> {
        self.record_cache.as_ref()
    }

    pub fn edge_property_cache(&self) -> Option<Arc<EdgePropertyCache>> {
        self.edge_property_cache.lock().unwrap().clone()
    }

    pub fn memory_tracker(&self) -> Option<&SharedMemoryTracker> {
        self.memory_tracker.as_ref()
    }

    pub fn record_cache_stats(&self) -> Option<RecordCacheStats> {
        self.record_cache
            .as_ref()
            .map(|c: &SharedRecordCache| c.stats())
    }

    pub fn edge_cache_stats(&self) -> Option<Arc<EdgePropertyCacheStats>> {
        self.edge_property_cache
            .lock()
            .unwrap()
            .as_ref()
            .map(|c: &Arc<EdgePropertyCache>| c.stats())
    }

    pub fn memory_stats(&self) -> Option<crate::storage::memory::MemoryStats> {
        self.memory_tracker
            .as_ref()
            .map(|t: &SharedMemoryTracker| t.stats())
    }

    pub fn clear_cache(&self) {
        if let Some(ref record_cache) = self.record_cache {
            record_cache.clear();
        }
        if let Some(ref edge_cache) = *self.edge_property_cache.lock().unwrap() {
            edge_cache.clear();
        }
    }

    // ==================== ID Index Cache Operations ====================

    pub fn get_cached_vertex_id(
        &self,
        label: LabelId,
        external_id: &str,
    ) -> Option<u32> {
        self.record_cache
            .as_ref()
            .and_then(|rc| rc.get_id_index(label, external_id))
    }

    pub fn cache_vertex_id(&self, label: LabelId, external_id: &str, internal_id: u32) {
        if let Some(ref rc) = self.record_cache {
            rc.insert_id_index(label, external_id, internal_id);
        }
    }

    pub fn remove_cached_vertex_id(&self, label: LabelId, external_id: &str) {
        if let Some(ref rc) = self.record_cache {
            rc.remove_id_index(label, external_id);
        }
    }

    // ==================== Vertex Cache Operations ====================

    pub fn get_cached_vertex(
        &self,
        label: LabelId,
        internal_id: u32,
    ) -> Option<CachedVertex> {
        self.record_cache
            .as_ref()
            .and_then(|rc| {
                let key = VertexCacheKey::new(label, internal_id);
                rc.get_vertex(&key)
            })
    }

    pub fn cache_vertex(&self, label: LabelId, internal_id: u32, external_id: String, properties: Vec<(String, crate::core::Value)>) {
        if let Some(ref rc) = self.record_cache {
            let key = VertexCacheKey::new(label, internal_id);
            let cached = CachedVertex {
                internal_id,
                external_id,
                properties,
            };
            rc.insert_vertex(key, cached);
        }
    }

    pub fn remove_cached_vertex(&self, label: LabelId, internal_id: u32) {
        if let Some(ref rc) = self.record_cache {
            let key = VertexCacheKey::new(label, internal_id);
            rc.remove_vertex(&key);
        }
    }

    // ==================== Edge Property Cache Operations ====================

    pub fn get_edge_property(&self, edge_id: EdgeId, prop_name: &str) -> Option<Value> {
        self.edge_property_cache
            .lock()
            .unwrap()
            .as_ref()
            .and_then(|c| c.get(edge_id, prop_name))
    }

    pub fn cache_edge_property(&self, edge_id: EdgeId, prop_name: &str, value: Value) -> bool {
        self.edge_property_cache
            .lock()
            .unwrap()
            .as_ref()
            .map(|c| c.put(edge_id, prop_name, value))
            .unwrap_or(false)
    }

    pub fn invalidate_edge(&self, edge_id: EdgeId) {
        if let Some(ref cache) = *self.edge_property_cache.lock().unwrap() {
            cache.invalidate(edge_id);
        }
    }

    pub fn invalidate_edge_property(&self, edge_id: EdgeId, prop_name: &str) {
        if let Some(ref cache) = *self.edge_property_cache.lock().unwrap() {
            cache.invalidate_property(edge_id, prop_name);
        }
    }

    // ==================== Cache Invalidation ====================

    pub fn invalidate_vertices_by_label(&self, label: LabelId) {
        if let Some(ref rc) = self.record_cache {
            rc.invalidate_vertices_by_label(label);
            rc.invalidate_id_indexes_by_label(label);
        }
    }
}
