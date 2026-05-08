use crate::storage::cache::{
    CachedEdge, CachedVertex, EdgeQueryKey, RecordCache, RecordCacheConfig,
    RecordCacheStats, SharedRecordCache, VertexCacheKey,
};
use crate::storage::memory::SharedMemoryTracker;
use crate::storage::vertex::LabelId;

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

    pub fn get_cached_edge(
        &self,
        edge_label: LabelId,
        src_internal: u64,
        dst_internal: u64,
    ) -> Option<CachedEdge> {
        self.record_cache
            .as_ref()
            .and_then(|rc| {
                let key = EdgeQueryKey::new(edge_label, src_internal, dst_internal);
                rc.get_edge_by_query(&key)
            })
    }

    pub fn cache_edge(&self, edge_label: LabelId, src_internal: u64, dst_internal: u64, edge_id: u64, properties: Vec<(String, crate::core::Value)>) {
        if let Some(ref rc) = self.record_cache {
            let key = EdgeQueryKey::new(edge_label, src_internal, dst_internal);
            let cached = CachedEdge {
                edge_id,
                src_vid: src_internal,
                dst_vid: dst_internal,
                properties,
            };
            rc.insert_edge_query(key, cached);
        }
    }

    pub fn remove_cached_edge(&self, edge_label: LabelId, src_internal: u64, dst_internal: u64) {
        if let Some(ref rc) = self.record_cache {
            let key = EdgeQueryKey::new(edge_label, src_internal, dst_internal);
            rc.remove_edge_query(&key);
        }
    }
}
