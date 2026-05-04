//! Record Cache
//!
//! LRU cache for vertex and edge records with memory tracking.

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

use parking_lot::Mutex;

use crate::core::Value;
use crate::storage::memory::MemoryTracker;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VertexCacheKey {
    pub label_id: u16,
    pub internal_id: u32,
}

impl VertexCacheKey {
    pub fn new(label_id: u16, internal_id: u32) -> Self {
        Self { label_id, internal_id }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EdgeCacheKey {
    pub edge_label_id: u16,
    pub src_vid: u64,
    pub dst_vid: u64,
    pub edge_id: u64,
}

impl EdgeCacheKey {
    pub fn new(edge_label_id: u16, src_vid: u64, dst_vid: u64, edge_id: u64) -> Self {
        Self {
            edge_label_id,
            src_vid,
            dst_vid,
            edge_id,
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
    pub fn estimated_size(&self) -> usize {
        let mut size = std::mem::size_of::<u32>() * 2;
        size += self.external_id.len();
        for (name, value) in &self.properties {
            size += name.len();
            size += value.estimated_size();
        }
        size
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
    pub fn estimated_size(&self) -> usize {
        let mut size = std::mem::size_of::<u64>() * 3;
        for (name, value) in &self.properties {
            size += name.len();
            size += value.estimated_size();
        }
        size
    }
}

#[derive(Debug)]
struct CacheEntry<T> {
    data: T,
    size: usize,
    last_access: Instant,
    access_count: u64,
}

impl<T> CacheEntry<T> {
    fn new(data: T, size: usize) -> Self {
        Self {
            data,
            size,
            last_access: Instant::now(),
            access_count: 1,
        }
    }

    fn touch(&mut self) {
        self.last_access = Instant::now();
        self.access_count += 1;
    }
}

#[derive(Debug)]
struct CacheShard<K, V> {
    entries: HashMap<K, CacheEntry<V>>,
    lru_order: Vec<K>,
    memory_usage: usize,
}

impl<K: Hash + Eq + Clone, V> CacheShard<K, V> {
    fn new() -> Self {
        Self {
            entries: HashMap::new(),
            lru_order: Vec::new(),
            memory_usage: 0,
        }
    }

    fn get(&mut self, key: &K) -> Option<&V> {
        if let Some(entry) = self.entries.get_mut(key) {
            entry.touch();
            if let Some(pos) = self.lru_order.iter().position(|x| x == key) {
                self.lru_order.remove(pos);
                self.lru_order.push(key.clone());
            }
            Some(&entry.data)
        } else {
            None
        }
    }

    fn insert(&mut self, key: K, value: V, size: usize) {
        if let Some(old) = self.entries.remove(&key) {
            self.memory_usage -= old.size;
            self.lru_order.retain(|x| x != &key);
        }

        let entry = CacheEntry::new(value, size);
        self.memory_usage += size;
        self.entries.insert(key.clone(), entry);
        self.lru_order.push(key);
    }

    fn remove(&mut self, key: &K) -> Option<usize> {
        if let Some(entry) = self.entries.remove(key) {
            self.memory_usage -= entry.size;
            self.lru_order.retain(|x| x != key);
            Some(entry.size)
        } else {
            None
        }
    }

    fn evict_lru(&mut self, required: usize, max_memory: usize) -> usize {
        let mut evicted = 0;

        while self.memory_usage + required > max_memory && !self.lru_order.is_empty() {
            if let Some(key) = self.lru_order.first().cloned() {
                if let Some(size) = self.remove(&key) {
                    evicted += size;
                }
            }
        }

        evicted
    }

    fn clear(&mut self) {
        self.entries.clear();
        self.lru_order.clear();
        self.memory_usage = 0;
    }
}

#[derive(Debug, Clone)]
pub struct RecordCacheConfig {
    pub max_memory: usize,
    pub shard_count: usize,
}

impl Default for RecordCacheConfig {
    fn default() -> Self {
        Self {
            max_memory: 128 * 1024 * 1024,
            shard_count: 8,
        }
    }
}

pub struct RecordCache {
    vertex_shards: Vec<Mutex<CacheShard<VertexCacheKey, CachedVertex>>>,
    edge_shards: Vec<Mutex<CacheShard<EdgeCacheKey, CachedEdge>>>,
    config: RecordCacheConfig,
    memory_usage: AtomicUsize,
    hits: AtomicU64,
    misses: AtomicU64,
    evictions: AtomicU64,
    memory_tracker: Option<Arc<MemoryTracker>>,
}

impl std::fmt::Debug for RecordCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RecordCache")
            .field("config", &self.config)
            .field("memory_usage", &self.memory_usage.load(Ordering::Relaxed))
            .field("hits", &self.hits.load(Ordering::Relaxed))
            .field("misses", &self.misses.load(Ordering::Relaxed))
            .field("evictions", &self.evictions.load(Ordering::Relaxed))
            .finish()
    }
}

impl RecordCache {
    pub fn new() -> Self {
        Self::with_config(RecordCacheConfig::default())
    }

    pub fn with_config(config: RecordCacheConfig) -> Self {
        let shard_count = config.shard_count;
        let vertex_shards = (0..shard_count).map(|_| Mutex::new(CacheShard::new())).collect();
        let edge_shards = (0..shard_count).map(|_| Mutex::new(CacheShard::new())).collect();

        Self {
            vertex_shards,
            edge_shards,
            config,
            memory_usage: AtomicUsize::new(0),
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            evictions: AtomicU64::new(0),
            memory_tracker: None,
        }
    }

    pub fn with_memory_tracker(mut self, tracker: Arc<MemoryTracker>) -> Self {
        self.memory_tracker = Some(tracker);
        self
    }

    fn shard_index<K: Hash>(&self, key: &K) -> usize {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        (hasher.finish() as usize) % self.config.shard_count
    }

    fn memory_per_shard(&self) -> usize {
        self.config.max_memory / self.config.shard_count / 2
    }

    pub fn get_vertex(&self, key: &VertexCacheKey) -> Option<CachedVertex> {
        let shard_idx = self.shard_index(key);
        let mut shard = self.vertex_shards[shard_idx].lock();

        if shard.get(key).is_some() {
            self.hits.fetch_add(1, Ordering::Relaxed);
            shard.get(key).cloned()
        } else {
            self.misses.fetch_add(1, Ordering::Relaxed);
            None
        }
    }

    pub fn insert_vertex(&self, key: VertexCacheKey, vertex: CachedVertex) {
        let size = vertex.estimated_size();
        let shard_idx = self.shard_index(&key);
        let max_per_shard = self.memory_per_shard();

        let mut shard = self.vertex_shards[shard_idx].lock();

        if shard.memory_usage + size > max_per_shard {
            let evicted = shard.evict_lru(size, max_per_shard);
            if evicted > 0 {
                self.memory_usage.fetch_sub(evicted, Ordering::Relaxed);
                self.evictions.fetch_add(1, Ordering::Relaxed);
                if let Some(ref tracker) = self.memory_tracker {
                    tracker.release_cache(evicted);
                }
            }
        }

        shard.insert(key, vertex, size);
        self.memory_usage.fetch_add(size, Ordering::Relaxed);

        if let Some(ref tracker) = self.memory_tracker {
            tracker.try_allocate_cache(size);
        }
    }

    pub fn remove_vertex(&self, key: &VertexCacheKey) {
        let shard_idx = self.shard_index(key);
        let mut shard = self.vertex_shards[shard_idx].lock();

        if let Some(size) = shard.remove(key) {
            self.memory_usage.fetch_sub(size, Ordering::Relaxed);
            if let Some(ref tracker) = self.memory_tracker {
                tracker.release_cache(size);
            }
        }
    }

    pub fn get_edge(&self, key: &EdgeCacheKey) -> Option<CachedEdge> {
        let shard_idx = self.shard_index(key);
        let mut shard = self.edge_shards[shard_idx].lock();

        if shard.get(key).is_some() {
            self.hits.fetch_add(1, Ordering::Relaxed);
            shard.get(key).cloned()
        } else {
            self.misses.fetch_add(1, Ordering::Relaxed);
            None
        }
    }

    pub fn insert_edge(&self, key: EdgeCacheKey, edge: CachedEdge) {
        let size = edge.estimated_size();
        let shard_idx = self.shard_index(&key);
        let max_per_shard = self.memory_per_shard();

        let mut shard = self.edge_shards[shard_idx].lock();

        if shard.memory_usage + size > max_per_shard {
            let evicted = shard.evict_lru(size, max_per_shard);
            if evicted > 0 {
                self.memory_usage.fetch_sub(evicted, Ordering::Relaxed);
                self.evictions.fetch_add(1, Ordering::Relaxed);
                if let Some(ref tracker) = self.memory_tracker {
                    tracker.release_cache(evicted);
                }
            }
        }

        shard.insert(key, edge, size);
        self.memory_usage.fetch_add(size, Ordering::Relaxed);

        if let Some(ref tracker) = self.memory_tracker {
            tracker.try_allocate_cache(size);
        }
    }

    pub fn remove_edge(&self, key: &EdgeCacheKey) {
        let shard_idx = self.shard_index(key);
        let mut shard = self.edge_shards[shard_idx].lock();

        if let Some(size) = shard.remove(key) {
            self.memory_usage.fetch_sub(size, Ordering::Relaxed);
            if let Some(ref tracker) = self.memory_tracker {
                tracker.release_cache(size);
            }
        }
    }

    pub fn clear(&self) {
        for shard in &self.vertex_shards {
            shard.lock().clear();
        }
        for shard in &self.edge_shards {
            shard.lock().clear();
        }
        self.memory_usage.store(0, Ordering::Relaxed);
    }

    pub fn memory_usage(&self) -> usize {
        self.memory_usage.load(Ordering::Relaxed)
    }

    pub fn max_memory(&self) -> usize {
        self.config.max_memory
    }

    pub fn stats(&self) -> RecordCacheStats {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = hits + misses;

        RecordCacheStats {
            hits,
            misses,
            hit_rate: if total > 0 { hits as f64 / total as f64 } else { 0.0 },
            evictions: self.evictions.load(Ordering::Relaxed),
            memory_usage: self.memory_usage.load(Ordering::Relaxed),
            max_memory: self.config.max_memory,
        }
    }

    pub fn utilization(&self) -> f32 {
        if self.config.max_memory == 0 {
            return 0.0;
        }
        self.memory_usage.load(Ordering::Relaxed) as f32 / self.config.max_memory as f32
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
    pub evictions: u64,
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
            self.memory_usage as f64 / self.max_memory as f64 * 100.0
        )?;
        writeln!(
            f,
            "  Hits: {}, Misses: {}, Hit Rate: {:.1}%",
            self.hits,
            self.misses,
            self.hit_rate * 100.0
        )?;
        writeln!(f, "  Evictions: {}", self.evictions)
    }
}

pub type SharedRecordCache = Arc<RecordCache>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex_cache_basic() {
        let cache = RecordCache::new();

        let key = VertexCacheKey::new(1, 100);
        let vertex = CachedVertex {
            internal_id: 100,
            external_id: "test_vertex".to_string(),
            properties: vec![("name".to_string(), Value::String("Alice".to_string()))],
        };

        cache.insert_vertex(key.clone(), vertex.clone());

        let cached = cache.get_vertex(&key);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().external_id, "test_vertex");
    }

    #[test]
    fn test_edge_cache_basic() {
        let cache = RecordCache::new();

        let key = EdgeCacheKey::new(1, 100, 200, 1);
        let edge = CachedEdge {
            edge_id: 1,
            src_vid: 100,
            dst_vid: 200,
            properties: vec![("weight".to_string(), Value::Double(1.5))],
        };

        cache.insert_edge(key.clone(), edge.clone());

        let cached = cache.get_edge(&key);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().edge_id, 1);
    }

    #[test]
    fn test_cache_eviction() {
        let config = RecordCacheConfig {
            max_memory: 1024,
            shard_count: 1,
        };
        let cache = RecordCache::with_config(config);

        for i in 0..100 {
            let key = VertexCacheKey::new(1, i);
            let vertex = CachedVertex {
                internal_id: i,
                external_id: format!("vertex_{}", i),
                properties: vec![("data".to_string(), Value::String("x".repeat(20)))],
            };
            cache.insert_vertex(key, vertex);
        }

        let stats = cache.stats();
        assert!(stats.evictions > 0);
    }

    #[test]
    fn test_cache_stats() {
        let cache = RecordCache::new();

        let key = VertexCacheKey::new(1, 100);
        let vertex = CachedVertex {
            internal_id: 100,
            external_id: "test".to_string(),
            properties: vec![],
        };

        cache.insert_vertex(key.clone(), vertex);

        cache.get_vertex(&key);
        cache.get_vertex(&VertexCacheKey::new(1, 999));

        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn test_cache_remove() {
        let cache = RecordCache::new();

        let key = VertexCacheKey::new(1, 100);
        let vertex = CachedVertex {
            internal_id: 100,
            external_id: "test".to_string(),
            properties: vec![],
        };

        cache.insert_vertex(key.clone(), vertex);
        assert!(cache.get_vertex(&key).is_some());

        cache.remove_vertex(&key);
        assert!(cache.get_vertex(&key).is_none());
    }

    #[test]
    fn test_cache_clear() {
        let cache = RecordCache::new();

        for i in 0..10 {
            let key = VertexCacheKey::new(1, i);
            let vertex = CachedVertex {
                internal_id: i,
                external_id: format!("v{}", i),
                properties: vec![],
            };
            cache.insert_vertex(key, vertex);
        }

        assert!(cache.memory_usage() > 0);
        cache.clear();
        assert_eq!(cache.memory_usage(), 0);
    }
}
