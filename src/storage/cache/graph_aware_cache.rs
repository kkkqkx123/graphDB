//! Graph-Aware Cache Strategy
//!
//! Implements graph-aware caching that prioritizes caching high-degree nodes
//! and frequently accessed neighbor lists for optimal graph traversal performance.
//!
//! # Features
//!
//! - Degree-based priority caching: High-degree nodes get higher cache priority
//! - Neighbor list caching: Specialized cache for neighbor lists
//! - Property cache: Specialized cache for property values
//! - Access pattern tracking: Tracks access frequency for cache decisions
//! - Adaptive eviction: Evicts low-priority entries under memory pressure

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use moka::sync::Cache;
use parking_lot::RwLock;

use crate::core::Value;
use crate::storage::memory::MemoryTracker;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NeighborCacheKey {
    pub label_id: u16,
    pub internal_id: u32,
}

impl NeighborCacheKey {
    pub fn new(label_id: u16, internal_id: u32) -> Self {
        Self {
            label_id,
            internal_id,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PropertyCacheKey {
    pub label_id: u16,
    pub internal_id: u32,
    pub property_index: u16,
}

impl PropertyCacheKey {
    pub fn new(label_id: u16, internal_id: u32, property_index: u16) -> Self {
        Self {
            label_id,
            internal_id,
            property_index,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CachedNeighbor {
    pub neighbors: Vec<NeighborEntry>,
    pub degree: u32,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct NeighborEntry {
    pub dst_id: u64,
    pub edge_id: u64,
    pub edge_label_id: u16,
}

impl CachedNeighbor {
    pub fn estimated_size(&self) -> u32 {
        let mut size = std::mem::size_of::<u32>() * 2 + std::mem::size_of::<u64>();
        size += self.neighbors.len() * std::mem::size_of::<NeighborEntry>();
        size as u32
    }
}

#[derive(Debug, Clone)]
pub struct CachedProperty {
    pub value: Value,
    pub property_name: String,
}

impl CachedProperty {
    pub fn estimated_size(&self) -> u32 {
        let mut size = self.property_name.len();
        size += self.value.estimated_size();
        size as u32
    }
}

#[derive(Debug)]
pub struct AccessFrequency {
    pub access_count: AtomicU64,
    pub last_access: AtomicU64,
}

impl Clone for AccessFrequency {
    fn clone(&self) -> Self {
        Self {
            access_count: AtomicU64::new(self.access_count.load(Ordering::Relaxed)),
            last_access: AtomicU64::new(self.last_access.load(Ordering::Relaxed)),
        }
    }
}

impl AccessFrequency {
    pub fn new() -> Self {
        Self {
            access_count: AtomicU64::new(0),
            last_access: AtomicU64::new(0),
        }
    }

    pub fn record_access(&self, timestamp: u64) {
        self.access_count.fetch_add(1, Ordering::Relaxed);
        self.last_access.store(timestamp, Ordering::Relaxed);
    }

    pub fn get_access_count(&self) -> u64 {
        self.access_count.load(Ordering::Relaxed)
    }

    pub fn get_last_access(&self) -> u64 {
        self.last_access.load(Ordering::Relaxed)
    }

    pub fn priority_score(&self, current_time: u64) -> f64 {
        let count = self.get_access_count() as f64;
        let age = (current_time - self.get_last_access()) as f64;
        let decay = 1.0 / (1.0 + age / 3600.0);
        count * decay
    }
}

#[derive(Debug, Clone)]
pub struct GraphCacheConfig {
    pub max_memory: usize,
    pub neighbor_cache_ratio: f32,
    pub property_cache_ratio: f32,
    pub degree_threshold: u32,
    pub high_degree_priority_multiplier: f32,
    pub ttl: Option<std::time::Duration>,
    pub tti: Option<std::time::Duration>,
}

impl Default for GraphCacheConfig {
    fn default() -> Self {
        Self {
            max_memory: 128 * 1024 * 1024,
            neighbor_cache_ratio: 0.5,
            property_cache_ratio: 0.2,
            degree_threshold: 100,
            high_degree_priority_multiplier: 2.0,
            ttl: Some(std::time::Duration::from_secs(3600)),
            tti: Some(std::time::Duration::from_secs(300)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GraphCacheStats {
    pub neighbor_hits: u64,
    pub neighbor_misses: u64,
    pub neighbor_evictions: u64,
    pub neighbor_count: u64,
    pub property_hits: u64,
    pub property_misses: u64,
    pub property_evictions: u64,
    pub property_count: u64,
    pub memory_usage: usize,
    pub max_memory: usize,
}

impl GraphCacheStats {
    pub fn neighbor_hit_rate(&self) -> f64 {
        let total = self.neighbor_hits + self.neighbor_misses;
        if total > 0 {
            self.neighbor_hits as f64 / total as f64
        } else {
            0.0
        }
    }

    pub fn property_hit_rate(&self) -> f64 {
        let total = self.property_hits + self.property_misses;
        if total > 0 {
            self.property_hits as f64 / total as f64
        } else {
            0.0
        }
    }

    pub fn total_count(&self) -> u64 {
        self.neighbor_count + self.property_count
    }
}

pub struct GraphAwareCache {
    neighbor_cache: Cache<NeighborCacheKey, CachedNeighbor>,
    property_cache: Cache<PropertyCacheKey, CachedProperty>,
    config: GraphCacheConfig,
    access_tracker: Arc<RwLock<HashMap<(u16, u32), Arc<AccessFrequency>>>>,
    degree_info: Arc<RwLock<HashMap<(u16, u32), u32>>>,
    neighbor_stats: Arc<CacheTypeStats>,
    property_stats: Arc<CacheTypeStats>,
    memory_tracker: Option<Arc<MemoryTracker>>,
}

#[derive(Debug, Default)]
pub struct CacheTypeStats {
    pub hits: AtomicU64,
    pub misses: AtomicU64,
    pub evictions: AtomicU64,
}

impl CacheTypeStats {
    pub fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_eviction(&self) {
        self.evictions.fetch_add(1, Ordering::Relaxed);
    }

    pub fn hits(&self) -> u64 {
        self.hits.load(Ordering::Relaxed)
    }

    pub fn misses(&self) -> u64 {
        self.misses.load(Ordering::Relaxed)
    }

    pub fn evictions(&self) -> u64 {
        self.evictions.load(Ordering::Relaxed)
    }
}

impl GraphAwareCache {
    pub fn new() -> Self {
        Self::with_config(GraphCacheConfig::default())
    }

    pub fn with_config(config: GraphCacheConfig) -> Self {
        let max_memory = config.max_memory as u64;
        let neighbor_memory = (max_memory as f32 * config.neighbor_cache_ratio) as u64;
        let property_memory = (max_memory as f32 * config.property_cache_ratio) as u64;

        let neighbor_stats = Arc::new(CacheTypeStats::default());
        let property_stats = Arc::new(CacheTypeStats::default());

        let neighbor_eviction_stats = neighbor_stats.clone();
        let property_eviction_stats = property_stats.clone();

        let neighbor_cache = Cache::builder()
            .max_capacity(neighbor_memory)
            .weigher(|_key: &NeighborCacheKey, value: &CachedNeighbor| value.estimated_size())
            .eviction_listener(move |_key, _value, _cause| {
                neighbor_eviction_stats.record_eviction();
            })
            .build();

        let property_cache = Cache::builder()
            .max_capacity(property_memory)
            .weigher(|_key: &PropertyCacheKey, value: &CachedProperty| value.estimated_size())
            .eviction_listener(move |_key, _value, _cause| {
                property_eviction_stats.record_eviction();
            })
            .build();

        Self {
            neighbor_cache,
            property_cache,
            config,
            access_tracker: Arc::new(RwLock::new(HashMap::new())),
            degree_info: Arc::new(RwLock::new(HashMap::new())),
            neighbor_stats,
            property_stats,
            memory_tracker: None,
        }
    }

    pub fn with_memory_tracker(mut self, tracker: Arc<MemoryTracker>) -> Self {
        self.memory_tracker = Some(tracker);
        self
    }

    pub fn record_access(&self, label_id: u16, internal_id: u32) {
        let key = (label_id, internal_id);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let tracker = self.access_tracker.read();
        if let Some(freq) = tracker.get(&key) {
            freq.record_access(now);
        } else {
            drop(tracker);
            let mut tracker = self.access_tracker.write();
            let freq = Arc::new(AccessFrequency::new());
            freq.record_access(now);
            tracker.insert(key, freq);
        }
    }

    pub fn update_degree(&self, label_id: u16, internal_id: u32, degree: u32) {
        let key = (label_id, internal_id);
        let mut degree_info = self.degree_info.write();
        degree_info.insert(key, degree);
    }

    pub fn get_priority_score(&self, label_id: u16, internal_id: u32) -> f64 {
        let key = (label_id, internal_id);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let tracker = self.access_tracker.read();
        let base_score = if let Some(freq) = tracker.get(&key) {
            freq.priority_score(now)
        } else {
            0.0
        };

        let degree_info = self.degree_info.read();
        let degree_multiplier = if let Some(&degree) = degree_info.get(&key) {
            if degree >= self.config.degree_threshold {
                self.config.high_degree_priority_multiplier as f64
            } else {
                1.0
            }
        } else {
            1.0
        };

        base_score * degree_multiplier
    }

    pub fn get_neighbor(&self, key: &NeighborCacheKey) -> Option<CachedNeighbor> {
        match self.neighbor_cache.get(key) {
            Some(neighbor) => {
                self.neighbor_stats.record_hit();
                self.record_access(key.label_id, key.internal_id);
                Some(neighbor)
            }
            None => {
                self.neighbor_stats.record_miss();
                None
            }
        }
    }

    pub fn insert_neighbor(&self, key: NeighborCacheKey, neighbor: CachedNeighbor) {
        let size = neighbor.estimated_size() as usize;
        self.neighbor_cache.insert(key, neighbor);

        if let Some(ref tracker) = self.memory_tracker {
            tracker.try_allocate_cache(size);
        }
    }

    pub fn remove_neighbor(&self, key: &NeighborCacheKey) {
        if self.neighbor_cache.remove(key).is_some() {
            if let Some(ref tracker) = self.memory_tracker {
                let size = std::mem::size_of::<CachedNeighbor>();
                tracker.release_cache(size);
            }
        }
    }

    pub fn get_property(&self, key: &PropertyCacheKey) -> Option<CachedProperty> {
        match self.property_cache.get(key) {
            Some(property) => {
                self.property_stats.record_hit();
                self.record_access(key.label_id, key.internal_id);
                Some(property)
            }
            None => {
                self.property_stats.record_miss();
                None
            }
        }
    }

    pub fn insert_property(&self, key: PropertyCacheKey, property: CachedProperty) {
        let size = property.estimated_size() as usize;
        self.property_cache.insert(key, property);

        if let Some(ref tracker) = self.memory_tracker {
            tracker.try_allocate_cache(size);
        }
    }

    pub fn remove_property(&self, key: &PropertyCacheKey) {
        if self.property_cache.remove(key).is_some() {
            if let Some(ref tracker) = self.memory_tracker {
                let size = std::mem::size_of::<CachedProperty>();
                tracker.release_cache(size);
            }
        }
    }

    pub fn invalidate_by_label(&self, label_id: u16) {
        self.neighbor_cache.invalidate_entries_if(move |k, _| k.label_id == label_id);
        self.property_cache.invalidate_entries_if(move |k, _| k.label_id == label_id);

        let mut access_tracker = self.access_tracker.write();
        access_tracker.retain(|(label, _), _| *label != label_id);

        let mut degree_info = self.degree_info.write();
        degree_info.retain(|(label, _), _| *label != label_id);
    }

    pub fn invalidate_vertex(&self, label_id: u16, internal_id: u32) {
        self.neighbor_cache
            .invalidate_entries_if(move |k, _| k.label_id == label_id && k.internal_id == internal_id);
        self.property_cache
            .invalidate_entries_if(move |k, _| k.label_id == label_id && k.internal_id == internal_id);

        let key = (label_id, internal_id);
        let mut access_tracker = self.access_tracker.write();
        access_tracker.remove(&key);

        let mut degree_info = self.degree_info.write();
        degree_info.remove(&key);
    }

    pub fn clear(&self) {
        self.neighbor_cache.invalidate_all();
        self.property_cache.invalidate_all();
        self.access_tracker.write().clear();
        self.degree_info.write().clear();
    }

    pub fn stats(&self) -> GraphCacheStats {
        GraphCacheStats {
            neighbor_hits: self.neighbor_stats.hits(),
            neighbor_misses: self.neighbor_stats.misses(),
            neighbor_evictions: self.neighbor_stats.evictions(),
            neighbor_count: self.neighbor_cache.entry_count(),
            property_hits: self.property_stats.hits(),
            property_misses: self.property_stats.misses(),
            property_evictions: self.property_stats.evictions(),
            property_count: self.property_cache.entry_count(),
            memory_usage: self.memory_usage(),
            max_memory: self.config.max_memory,
        }
    }

    pub fn memory_usage(&self) -> usize {
        (self.neighbor_cache.weighted_size() + self.property_cache.weighted_size()) as usize
    }

    pub fn utilization(&self) -> f32 {
        if self.config.max_memory == 0 {
            return 0.0;
        }
        self.memory_usage() as f32 / self.config.max_memory as f32
    }

    pub fn get_top_cached_vertices(&self, limit: usize) -> Vec<(u16, u32, f64)> {
        let access_tracker = self.access_tracker.read();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut scores: Vec<(u16, u32, f64)> = access_tracker
            .iter()
            .map(|(&(label_id, internal_id), freq)| {
                let score = freq.priority_score(now);
                (label_id, internal_id, score)
            })
            .collect();

        scores.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        scores.truncate(limit);
        scores
    }
}

impl Default for GraphAwareCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neighbor_cache_basic() {
        let cache = GraphAwareCache::new();

        let key = NeighborCacheKey::new(1, 100);
        let neighbor = CachedNeighbor {
            neighbors: vec![
                NeighborEntry {
                    dst_id: 200,
                    edge_id: 1,
                    edge_label_id: 1,
                },
                NeighborEntry {
                    dst_id: 300,
                    edge_id: 2,
                    edge_label_id: 1,
                },
            ],
            degree: 2,
            timestamp: 0,
        };

        cache.insert_neighbor(key, neighbor);

        let cached = cache.get_neighbor(&NeighborCacheKey::new(1, 100));
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().degree, 2);
    }

    #[test]
    fn test_property_cache_basic() {
        let cache = GraphAwareCache::new();

        let key = PropertyCacheKey::new(1, 100, 0);
        let property = CachedProperty {
            value: Value::String("test".to_string()),
            property_name: "name".to_string(),
        };

        cache.insert_property(key, property);

        let cached = cache.get_property(&PropertyCacheKey::new(1, 100, 0));
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().property_name, "name");
    }

    #[test]
    fn test_access_frequency() {
        let freq = AccessFrequency::new();

        freq.record_access(1000);
        freq.record_access(2000);
        freq.record_access(3000);

        assert_eq!(freq.get_access_count(), 3);
        assert_eq!(freq.get_last_access(), 3000);

        let score = freq.priority_score(4000);
        assert!(score > 0.0);
    }

    #[test]
    fn test_degree_priority() {
        let cache = GraphAwareCache::new();

        cache.update_degree(1, 100, 1000);
        cache.update_degree(1, 200, 10);

        cache.record_access(1, 100);
        cache.record_access(1, 200);

        let high_degree_score = cache.get_priority_score(1, 100);
        let low_degree_score = cache.get_priority_score(1, 200);

        assert!(high_degree_score > low_degree_score);
    }

    #[test]
    fn test_invalidate_by_label() {
        let cache = GraphAwareCache::new();

        cache.insert_neighbor(
            NeighborCacheKey::new(1, 100),
            CachedNeighbor {
                neighbors: vec![],
                degree: 0,
                timestamp: 0,
            },
        );

        cache.insert_neighbor(
            NeighborCacheKey::new(2, 200),
            CachedNeighbor {
                neighbors: vec![],
                degree: 0,
                timestamp: 0,
            },
        );

        cache.invalidate_by_label(1);

        assert!(cache.get_neighbor(&NeighborCacheKey::new(1, 100)).is_none());
        assert!(cache.get_neighbor(&NeighborCacheKey::new(2, 200)).is_some());
    }

    #[test]
    fn test_cache_stats() {
        let cache = GraphAwareCache::new();

        cache.get_neighbor(&NeighborCacheKey::new(1, 100));
        cache.get_neighbor(&NeighborCacheKey::new(1, 999));

        let stats = cache.stats();
        assert_eq!(stats.neighbor_hits, 0);
        assert_eq!(stats.neighbor_misses, 2);
    }
}
