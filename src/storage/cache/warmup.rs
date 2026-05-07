//! Cache Warmup Mechanism
//!
//! Provides cache warmup functionality to preload frequently accessed
//! vertices and edges into the cache for improved initial query performance.
//!
//! # Features
//!
//! - Vertex warmup: Preload vertices from specified labels
//! - Edge warmup: Preload edge data for frequently accessed vertices
//! - Degree-based warmup: Prioritize high-degree vertices
//! - Configurable warmup limits
//! - Warmup statistics tracking

use std::time::Instant;

use crate::storage::cache::{
    GraphAwareCache, NeighborCacheKey, CachedNeighbor, NeighborEntry,
    RecordCache, VertexCacheKey, CachedVertex,
};
use crate::storage::cache::{CacheWarmupConfig, WarmupStats};

pub trait WarmupDataProvider: Send + Sync {
    fn get_vertex_ids(&self, label_id: u16, limit: usize) -> Vec<u32>;
    fn get_vertex_record(&self, label_id: u16, internal_id: u32) -> Option<(String, Vec<(String, crate::core::Value)>)>;
    fn get_neighbors(&self, label_id: u16, internal_id: u32) -> Vec<(u16, u64, u64)>;
    fn get_vertex_degree(&self, label_id: u16, internal_id: u32) -> u32;
}

pub struct CacheWarmup {
    config: CacheWarmupConfig,
}

impl CacheWarmup {
    pub fn new(config: CacheWarmupConfig) -> Self {
        Self { config }
    }

    pub fn warmup_graph_cache(
        &self,
        cache: &GraphAwareCache,
        data_provider: &dyn WarmupDataProvider,
        vertex_labels: &[u16],
        edge_labels: &[u16],
    ) -> WarmupStats {
        let start = Instant::now();
        let mut stats = WarmupStats::default();

        if !self.config.enabled {
            return stats;
        }

        let labels_to_warmup = if vertex_labels.is_empty() {
            self.config.warmup_vertex_labels.clone()
        } else {
            vertex_labels.to_vec()
        };

        for &label_id in &labels_to_warmup {
            let vertex_ids = data_provider.get_vertex_ids(label_id, self.config.max_warmup_entries);

            for internal_id in vertex_ids {
                if let Some((external_id, properties)) = data_provider.get_vertex_record(label_id, internal_id) {
                    let vertex = CachedVertex {
                        internal_id,
                        external_id,
                        properties,
                    };

                    let degree = data_provider.get_vertex_degree(label_id, internal_id);
                    cache.update_degree(label_id, internal_id, degree);

                    stats.vertices_loaded += 1;
                    stats.total_bytes += vertex.estimated_size() as usize;
                }

                let neighbor_data = data_provider.get_neighbors(label_id, internal_id);
                if !neighbor_data.is_empty() {
                    let neighbors: Vec<NeighborEntry> = neighbor_data
                        .into_iter()
                        .map(|(edge_label_id, dst_id, edge_id)| NeighborEntry {
                            dst_id,
                            edge_id,
                            edge_label_id,
                        })
                        .collect();

                    let degree = neighbors.len() as u32;
                    let cached_neighbor = CachedNeighbor {
                        neighbors,
                        degree,
                        timestamp: 0,
                    };

                    let estimated_size = cached_neighbor.estimated_size() as usize;

                    let key = NeighborCacheKey::new(label_id, internal_id);
                    cache.insert_neighbor(key, cached_neighbor);

                    stats.edges_loaded += 1;
                    stats.total_bytes += estimated_size;
                }
            }
        }

        let edge_labels_to_warmup = if edge_labels.is_empty() {
            self.config.warmup_edge_labels.clone()
        } else {
            edge_labels.to_vec()
        };

        for _edge_label_id in &edge_labels_to_warmup {
        }

        stats.duration_ms = start.elapsed().as_millis() as u64;
        stats
    }

    pub fn warmup_record_cache(
        &self,
        cache: &RecordCache,
        data_provider: &dyn WarmupDataProvider,
        vertex_labels: &[u16],
    ) -> WarmupStats {
        let start = Instant::now();
        let mut stats = WarmupStats::default();

        if !self.config.enabled {
            return stats;
        }

        let labels_to_warmup = if vertex_labels.is_empty() {
            self.config.warmup_vertex_labels.clone()
        } else {
            vertex_labels.to_vec()
        };

        for &label_id in &labels_to_warmup {
            let vertex_ids = data_provider.get_vertex_ids(label_id, self.config.max_warmup_entries);

            for internal_id in vertex_ids {
                if let Some((external_id, properties)) = data_provider.get_vertex_record(label_id, internal_id) {
                    let vertex = CachedVertex {
                        internal_id,
                        external_id: external_id.clone(),
                        properties,
                    };

                    let estimated_size = vertex.estimated_size() as usize;

                    let key = VertexCacheKey::new(label_id, internal_id);
                    cache.insert_vertex(key, vertex);

                    cache.insert_id_index(label_id, &external_id, internal_id);

                    stats.vertices_loaded += 1;
                    stats.id_indexes_loaded += 1;
                    stats.total_bytes += estimated_size;
                }
            }
        }

        stats.duration_ms = start.elapsed().as_millis() as u64;
        stats
    }

    pub fn warmup_high_degree_vertices(
        &self,
        cache: &GraphAwareCache,
        data_provider: &dyn WarmupDataProvider,
        label_id: u16,
        degree_threshold: u32,
        limit: usize,
    ) -> WarmupStats {
        let start = Instant::now();
        let mut stats = WarmupStats::default();

        let vertex_ids = data_provider.get_vertex_ids(label_id, limit * 2);

        let mut vertex_degrees: Vec<(u32, u32)> = vertex_ids
            .into_iter()
            .map(|id| {
                let degree = data_provider.get_vertex_degree(label_id, id);
                (id, degree)
            })
            .filter(|(_, degree)| *degree >= degree_threshold)
            .collect();

        vertex_degrees.sort_by(|a, b| b.1.cmp(&a.1));
        vertex_degrees.truncate(limit);

        for (internal_id, degree) in vertex_degrees {
            cache.update_degree(label_id, internal_id, degree);

            if let Some((external_id, properties)) = data_provider.get_vertex_record(label_id, internal_id) {
                let vertex = CachedVertex {
                    internal_id,
                    external_id,
                    properties,
                };
                stats.vertices_loaded += 1;
                stats.total_bytes += vertex.estimated_size() as usize;
            }

            let neighbor_data = data_provider.get_neighbors(label_id, internal_id);
            if !neighbor_data.is_empty() {
                let neighbors: Vec<NeighborEntry> = neighbor_data
                    .into_iter()
                    .map(|(edge_label_id, dst_id, edge_id)| NeighborEntry {
                        dst_id,
                        edge_id,
                        edge_label_id,
                    })
                    .collect();

                let degree = neighbors.len() as u32;
                let cached_neighbor = CachedNeighbor {
                    neighbors,
                    degree,
                    timestamp: 0,
                };

                let estimated_size = cached_neighbor.estimated_size() as usize;

                let key = NeighborCacheKey::new(label_id, internal_id);
                cache.insert_neighbor(key, cached_neighbor);

                stats.edges_loaded += 1;
                stats.total_bytes += estimated_size;
            }
        }

        stats.duration_ms = start.elapsed().as_millis() as u64;
        stats
    }
}

impl Default for CacheWarmup {
    fn default() -> Self {
        Self::new(CacheWarmupConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;

    struct MockDataProvider {
        vertices: std::collections::HashMap<(u16, u32), (String, Vec<(String, Value)>)>,
        neighbors: std::collections::HashMap<(u16, u32), Vec<(u16, u64, u64)>>,
        degrees: std::collections::HashMap<(u16, u32), u32>,
    }

    impl MockDataProvider {
        fn new() -> Self {
            Self {
                vertices: std::collections::HashMap::new(),
                neighbors: std::collections::HashMap::new(),
                degrees: std::collections::HashMap::new(),
            }
        }

        fn add_vertex(&mut self, label_id: u16, internal_id: u32, external_id: String, properties: Vec<(String, Value)>) {
            self.vertices.insert((label_id, internal_id), (external_id, properties));
            self.degrees.insert((label_id, internal_id), 0);
        }

        fn add_neighbor(&mut self, label_id: u16, internal_id: u32, edge_label_id: u16, dst_id: u64, edge_id: u64) {
            self.neighbors
                .entry((label_id, internal_id))
                .or_insert_with(Vec::new)
                .push((edge_label_id, dst_id, edge_id));

            if let Some(degree) = self.degrees.get_mut(&(label_id, internal_id)) {
                *degree += 1;
            }
        }
    }

    impl WarmupDataProvider for MockDataProvider {
        fn get_vertex_ids(&self, label_id: u16, limit: usize) -> Vec<u32> {
            self.vertices
                .keys()
                .filter(|(l, _)| *l == label_id)
                .map(|(_, id)| *id)
                .take(limit)
                .collect()
        }

        fn get_vertex_record(&self, label_id: u16, internal_id: u32) -> Option<(String, Vec<(String, Value)>)> {
            self.vertices.get(&(label_id, internal_id)).cloned()
        }

        fn get_neighbors(&self, label_id: u16, internal_id: u32) -> Vec<(u16, u64, u64)> {
            self.neighbors
                .get(&(label_id, internal_id))
                .cloned()
                .unwrap_or_default()
        }

        fn get_vertex_degree(&self, label_id: u16, internal_id: u32) -> u32 {
            self.degrees.get(&(label_id, internal_id)).copied().unwrap_or(0)
        }
    }

    #[test]
    fn test_warmup_graph_cache() {
        let mut provider = MockDataProvider::new();
        provider.add_vertex(1, 100, "v1".to_string(), vec![("name".to_string(), Value::String("Alice".to_string()))]);
        provider.add_vertex(1, 200, "v2".to_string(), vec![("name".to_string(), Value::String("Bob".to_string()))]);
        provider.add_neighbor(1, 100, 1, 200, 1);
        provider.add_neighbor(1, 200, 1, 100, 2);

        let cache = GraphAwareCache::new();
        let warmup = CacheWarmup::new(CacheWarmupConfig {
            enabled: true,
            warmup_vertex_labels: vec![1],
            warmup_edge_labels: vec![],
            max_warmup_entries: 100,
        });

        let stats = warmup.warmup_graph_cache(&cache, &provider, &[1], &[]);

        assert_eq!(stats.vertices_loaded, 2);
        assert_eq!(stats.edges_loaded, 2);
        assert!(stats.duration_ms > 0);
    }

    #[test]
    fn test_warmup_disabled() {
        let provider = MockDataProvider::new();
        let cache = GraphAwareCache::new();
        let warmup = CacheWarmup::new(CacheWarmupConfig {
            enabled: false,
            warmup_vertex_labels: vec![],
            warmup_edge_labels: vec![],
            max_warmup_entries: 100,
        });

        let stats = warmup.warmup_graph_cache(&cache, &provider, &[], &[]);

        assert_eq!(stats.vertices_loaded, 0);
        assert_eq!(stats.edges_loaded, 0);
    }

    #[test]
    fn test_warmup_high_degree_vertices() {
        let mut provider = MockDataProvider::new();
        provider.add_vertex(1, 100, "high_degree".to_string(), vec![]);
        provider.add_vertex(1, 200, "low_degree".to_string(), vec![]);

        for i in 0..50 {
            provider.add_neighbor(1, 100, 1, i as u64, i as u64);
        }
        for i in 0..5 {
            provider.add_neighbor(1, 200, 1, i as u64, i as u64);
        }

        let cache = GraphAwareCache::new();
        let warmup = CacheWarmup::new(CacheWarmupConfig::default());

        let stats = warmup.warmup_high_degree_vertices(&cache, &provider, 1, 10, 10);

        assert_eq!(stats.vertices_loaded, 1);
        assert_eq!(stats.edges_loaded, 1);
    }
}
