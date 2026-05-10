//! Cache Manager
//!
//! Manages record cache and memory tracking for the storage engine.
//!
//! ## Design Note: Why No Edge Cache?
//!
//! Edge data is NOT cached separately because:
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

use crate::storage::cache::{
    CachedVertex, RecordCache, RecordCacheConfig,
    RecordCacheStats, SharedRecordCache, VertexCacheKey,
};
use crate::storage::memory::SharedMemoryTracker;
use crate::storage::vertex::LabelId;

/// Manager for storage caches
pub struct CacheManager {
    pub record_cache: Option<SharedRecordCache>,
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
            memory_tracker: Some(memory_tracker),
        }
    }

    pub fn record_cache(&self) -> Option<&SharedRecordCache> {
        self.record_cache.as_ref()
    }

    pub fn memory_tracker(&self) -> Option<&SharedMemoryTracker> {
        self.memory_tracker.as_ref()
    }

    pub fn record_cache_stats(&self) -> Option<RecordCacheStats> {
        self.record_cache
            .as_ref()
            .map(|c: &SharedRecordCache| c.stats())
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

    // ==================== Cache Invalidation ====================

    pub fn invalidate_vertices_by_label(&self, label: LabelId) {
        if let Some(ref rc) = self.record_cache {
            rc.invalidate_vertices_by_label(label);
            rc.invalidate_id_indexes_by_label(label);
        }
    }
}
