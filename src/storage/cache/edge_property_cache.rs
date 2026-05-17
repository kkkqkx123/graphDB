//! Optional Edge Property Cache
//!
//! Provides optional caching for edge properties in high-load scenarios.
//! This cache is disabled by default and can be enabled when:
//! - Edge property access frequency exceeds a threshold
//! - Property size is small (< 1KB)
//! - Edge update frequency is low
//!
//! ## Design Rationale
//!
//! Unlike vertex cache, edge property cache is optional because:
//!
//! 1. **CSR is already optimized for traversal**: The CSR structure provides
//!    O(1) edge list access with contiguous memory layout.
//!
//! 2. **PropertyTable provides O(1) access**: Edge properties are stored with
//!    direct offset access.
//!
//! 3. **Memory overhead**: Edge data volume is typically much larger than vertex data.
//!
//! 4. **Cache invalidation complexity**: Edges are updated more frequently.
//!
//! ## When to Enable
//!
//! - Social network scenarios with frequent "friend relationship" queries
//! - High-frequency single-edge property lookups
//! - Read-heavy workloads with low edge update rates

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use dashmap::DashMap;
use lru::LruCache;
use parking_lot::Mutex;

use crate::core::types::EdgeId;
use crate::core::Value;

const DEFAULT_MAX_ENTRIES: usize = 10_000;
const DEFAULT_MAX_MEMORY: usize = 10 * 1024 * 1024;
const DEFAULT_TTL_SECONDS: u64 = 300;
const HOT_ACCESS_THRESHOLD: u64 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EdgePropertyKey {
    pub edge_id: EdgeId,
    pub prop_name_hash: u64,
}

impl EdgePropertyKey {
    pub fn new(edge_id: EdgeId, prop_name: &str) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        prop_name.hash(&mut hasher);

        Self {
            edge_id,
            prop_name_hash: hasher.finish(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CachedEdgeProperty {
    pub value: Value,
    pub size: usize,
    pub created_at: Instant,
    pub last_accessed: Instant,
    pub access_count: u64,
}

impl CachedEdgeProperty {
    pub fn new(value: Value) -> Self {
        let size = std::mem::size_of::<Value>() + value.estimated_size();
        Self {
            value,
            size,
            created_at: Instant::now(),
            last_accessed: Instant::now(),
            access_count: 1,
        }
    }

    pub fn touch(&mut self) {
        self.last_accessed = Instant::now();
        self.access_count += 1;
    }

    pub fn is_expired(&self, ttl: Duration) -> bool {
        self.created_at.elapsed() > ttl
    }

    pub fn is_hot(&self) -> bool {
        self.access_count >= HOT_ACCESS_THRESHOLD
    }
}

#[derive(Debug, Clone)]
pub struct EdgePropertyCacheConfig {
    pub enabled: bool,
    pub max_entries: usize,
    pub max_memory: usize,
    pub ttl: Duration,
    pub min_access_frequency: u32,
    pub max_property_size: usize,
}

impl Default for EdgePropertyCacheConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_entries: DEFAULT_MAX_ENTRIES,
            max_memory: DEFAULT_MAX_MEMORY,
            ttl: Duration::from_secs(DEFAULT_TTL_SECONDS),
            min_access_frequency: 5,
            max_property_size: 1024,
        }
    }
}

impl EdgePropertyCacheConfig {
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            ..Self::default()
        }
    }

    pub fn high_performance() -> Self {
        Self {
            enabled: true,
            max_entries: 100_000,
            max_memory: 100 * 1024 * 1024,
            ttl: Duration::from_secs(600),
            min_access_frequency: 3,
            max_property_size: 2048,
        }
    }
}

#[derive(Debug, Default)]
pub struct EdgePropertyCacheStats {
    pub hits: AtomicU64,
    pub misses: AtomicU64,
    pub evictions: AtomicU64,
    pub expired: AtomicU64,
    pub current_entries: AtomicUsize,
    pub current_memory: AtomicUsize,
}

impl EdgePropertyCacheStats {
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = hits + misses;
        if total > 0 {
            hits as f64 / total as f64
        } else {
            0.0
        }
    }

    pub fn reset(&self) {
        self.hits.store(0, Ordering::Relaxed);
        self.misses.store(0, Ordering::Relaxed);
        self.evictions.store(0, Ordering::Relaxed);
        self.expired.store(0, Ordering::Relaxed);
        self.current_entries.store(0, Ordering::Relaxed);
        self.current_memory.store(0, Ordering::Relaxed);
    }
}

pub struct EdgePropertyCache {
    config: EdgePropertyCacheConfig,
    cache: Mutex<LruCache<EdgePropertyKey, CachedEdgeProperty>>,
    stats: Arc<EdgePropertyCacheStats>,
    read_tracker: Arc<DashMap<EdgePropertyKey, u32>>,
    edge_index: Arc<DashMap<EdgeId, Vec<EdgePropertyKey>>>,
}

impl EdgePropertyCache {
    pub fn new(config: EdgePropertyCacheConfig) -> Self {
        let cache = LruCache::new(std::num::NonZeroUsize::new(config.max_entries).unwrap());

        Self {
            config,
            cache: Mutex::new(cache),
            stats: Arc::new(EdgePropertyCacheStats::default()),
            read_tracker: Arc::new(DashMap::new()),
            edge_index: Arc::new(DashMap::new()),
        }
    }

    pub fn disabled() -> Self {
        Self::new(EdgePropertyCacheConfig::default())
    }

    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    pub fn stats(&self) -> Arc<EdgePropertyCacheStats> {
        self.stats.clone()
    }

    pub fn get(&self, edge_id: EdgeId, prop_name: &str) -> Option<Value> {
        if !self.config.enabled {
            return None;
        }

        let key = EdgePropertyKey::new(edge_id, prop_name);
        let mut cache = self.cache.lock();

        if let Some(entry) = cache.get_mut(&key) {
            if entry.is_expired(self.config.ttl) {
                let size = entry.size;
                cache.pop(&key);
                drop(cache);
                self.stats.expired.fetch_add(1, Ordering::Relaxed);
                self.stats.current_entries.fetch_sub(1, Ordering::Relaxed);
                self.stats.current_memory.fetch_sub(size, Ordering::Relaxed);
                self.stats.misses.fetch_add(1, Ordering::Relaxed);
                return None;
            }

            entry.touch();
            self.stats.hits.fetch_add(1, Ordering::Relaxed);
            return Some(entry.value.clone());
        }

        drop(cache);
        self.track_read(&key);
        self.stats.misses.fetch_add(1, Ordering::Relaxed);
        None
    }

    pub fn put(&self, edge_id: EdgeId, prop_name: &str, value: Value) -> bool {
        if !self.config.enabled {
            return false;
        }

        let size = std::mem::size_of::<Value>() + value.estimated_size();
        if size > self.config.max_property_size {
            return false;
        }

        let key = EdgePropertyKey::new(edge_id, prop_name);

        let mut cache = self.cache.lock();

        // Check if this property is read frequently enough to warrant caching.
        // Performed under cache lock to prevent TOCTOU races between check and insert.
        let should_cache = self
            .read_tracker
            .get(&key)
            .is_some_and(|count| *count >= self.config.min_access_frequency);
        if !should_cache {
            return false;
        }

        if let Some(old) = cache.put(key, CachedEdgeProperty::new(value)) {
            self.stats.current_memory.fetch_sub(old.size, Ordering::Relaxed);
        } else {
            self.stats.current_entries.fetch_add(1, Ordering::Relaxed);
            self.edge_index.entry(edge_id).or_default().push(key);
        }
        self.stats.current_memory.fetch_add(size, Ordering::Relaxed);

        self.evict_if_needed(&mut cache);

        true
    }

    pub fn invalidate(&self, edge_id: EdgeId) {
        if !self.config.enabled {
            return;
        }

        let mut cache = self.cache.lock();

        if let Some((_, keys)) = self.edge_index.remove(&edge_id) {
            for key in keys {
                if let Some(entry) = cache.pop(&key) {
                    self.stats.evictions.fetch_add(1, Ordering::Relaxed);
                    self.stats.current_entries.fetch_sub(1, Ordering::Relaxed);
                    self.stats.current_memory.fetch_sub(entry.size, Ordering::Relaxed);
                }
            }
        }
        drop(cache);

        self.read_tracker.retain(|k, _| k.edge_id != edge_id);
    }

    pub fn invalidate_property(&self, edge_id: EdgeId, prop_name: &str) {
        if !self.config.enabled {
            return;
        }

        let key = EdgePropertyKey::new(edge_id, prop_name);
        let mut cache = self.cache.lock();

        if let Some(entry) = cache.pop(&key) {
            self.stats.evictions.fetch_add(1, Ordering::Relaxed);
            self.stats.current_entries.fetch_sub(1, Ordering::Relaxed);
            self.stats.current_memory.fetch_sub(entry.size, Ordering::Relaxed);
        }
        drop(cache);

        self.read_tracker.remove(&key);
        if let Some(mut keys) = self.edge_index.get_mut(&edge_id) {
            keys.retain(|k| *k != key);
            if keys.is_empty() {
                drop(keys);
                self.edge_index.remove(&edge_id);
            }
        }
    }

    pub fn clear(&self) {
        let mut cache = self.cache.lock();
        let entries = cache.len();
        cache.clear();
        drop(cache);

        self.read_tracker.clear();
        self.edge_index.clear();

        self.stats.current_entries.store(0, Ordering::Relaxed);
        self.stats.current_memory.store(0, Ordering::Relaxed);
        self.stats.evictions.fetch_add(entries as u64, Ordering::Relaxed);
    }

    fn track_read(&self, key: &EdgePropertyKey) {
        let mut entry = self.read_tracker.entry(*key).or_insert(0);
        *entry += 1;
    }

    fn evict_if_needed(&self, cache: &mut LruCache<EdgePropertyKey, CachedEdgeProperty>) {
        let current_memory = self.stats.current_memory.load(Ordering::Relaxed);
        if current_memory > self.config.max_memory {
            let target_memory = (self.config.max_memory as f64 * 0.8) as usize;
            let mut freed = 0;

            while freed < (current_memory - target_memory) {
                if let Some((key, entry)) = cache.pop_lru() {
                    freed += entry.size;
                    self.stats.evictions.fetch_add(1, Ordering::Relaxed);
                    self.stats.current_entries.fetch_sub(1, Ordering::Relaxed);
                    self.stats.current_memory.fetch_sub(entry.size, Ordering::Relaxed);
                    if let Some(mut keys) = self.edge_index.get_mut(&key.edge_id) {
                        keys.retain(|k| *k != key);
                        if keys.is_empty() {
                            drop(keys);
                            self.edge_index.remove(&key.edge_id);
                        }
                    }
                } else {
                    break;
                }
            }
        }
    }

    pub fn memory_usage(&self) -> usize {
        self.stats.current_memory.load(Ordering::Relaxed)
    }

    pub fn entry_count(&self) -> usize {
        self.stats.current_entries.load(Ordering::Relaxed)
    }

    pub fn should_cache(&self, edge_id: EdgeId, prop_name: &str) -> bool {
        if !self.config.enabled {
            return false;
        }

        let key = EdgePropertyKey::new(edge_id, prop_name);
        self.read_tracker
            .get(&key)
            .is_some_and(|count| *count >= self.config.min_access_frequency)
    }
}

impl std::fmt::Debug for EdgePropertyCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EdgePropertyCache")
            .field("enabled", &self.config.enabled)
            .field("max_entries", &self.config.max_entries)
            .field("max_memory", &self.config.max_memory)
            .field("entry_count", &self.entry_count())
            .field("memory_usage", &self.memory_usage())
            .field("hit_rate", &self.stats.hit_rate())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_property_cache_disabled() {
        let cache = EdgePropertyCache::disabled();
        assert!(!cache.is_enabled());

        assert!(cache.get(1, "prop").is_none());
        assert!(!cache.put(1, "prop", Value::Int(42)));
    }

    #[test]
    fn test_edge_property_cache_basic() {
        let config = EdgePropertyCacheConfig::enabled();
        let cache = EdgePropertyCache::new(config);

        assert!(cache.is_enabled());

        assert!(cache.get(1, "prop").is_none());
        assert_eq!(cache.stats().misses.load(Ordering::Relaxed), 1);

        for _ in 0..10 {
            cache.get(1, "prop");
        }

        assert!(cache.put(1, "prop", Value::Int(42)));

        let value = cache.get(1, "prop");
        assert!(value.is_some());
        assert_eq!(value.unwrap(), Value::Int(42));
        assert!(cache.stats().hits.load(Ordering::Relaxed) > 0);
    }

    #[test]
    fn test_edge_property_cache_invalidation() {
        let config = EdgePropertyCacheConfig::enabled();
        let cache = EdgePropertyCache::new(config);

        for _ in 0..10 {
            cache.get(1, "prop1");
            cache.get(1, "prop2");
        }

        cache.put(1, "prop1", Value::Int(1));
        cache.put(1, "prop2", Value::Int(2));

        assert!(cache.get(1, "prop1").is_some());
        assert!(cache.get(1, "prop2").is_some());

        cache.invalidate(1);

        assert!(cache.get(1, "prop1").is_none());
        assert!(cache.get(1, "prop2").is_none());
    }

    #[test]
    fn test_edge_property_cache_property_size_limit() {
        let mut config = EdgePropertyCacheConfig::enabled();
        config.max_property_size = 10;
        let cache = EdgePropertyCache::new(config);

        for _ in 0..10 {
            cache.get(1, "prop");
        }

        let large_value = Value::String("x".repeat(100));
        assert!(!cache.put(1, "prop", large_value));
    }
}
