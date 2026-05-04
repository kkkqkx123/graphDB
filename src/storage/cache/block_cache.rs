//! Block Cache
//!
//! LRU cache for storage blocks with sharding for concurrent access.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Instant;

use parking_lot::Mutex;

/// Type of table the block belongs to
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TableType {
    Vertex,
    Edge,
    Property,
    Index,
    Schema,
}

/// Unique identifier for a cache block
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId {
    /// Type of table
    pub table_type: TableType,
    /// Label ID (for vertex/edge tables)
    pub label_id: u32,
    /// Block number within the table
    pub block_number: u64,
}

impl BlockId {
    /// Create a new block ID
    pub fn new(table_type: TableType, label_id: u32, block_number: u64) -> Self {
        Self {
            table_type,
            label_id,
            block_number,
        }
    }

    /// Create a vertex block ID
    pub fn vertex(label_id: u32, block_number: u64) -> Self {
        Self::new(TableType::Vertex, label_id, block_number)
    }

    /// Create an edge block ID
    pub fn edge(label_id: u32, block_number: u64) -> Self {
        Self::new(TableType::Edge, label_id, block_number)
    }

    /// Create a property block ID
    pub fn property(label_id: u32, block_number: u64) -> Self {
        Self::new(TableType::Property, label_id, block_number)
    }

    /// Create an index block ID
    pub fn index(label_id: u32, block_number: u64) -> Self {
        Self::new(TableType::Index, label_id, block_number)
    }

    /// Create a schema block ID
    pub fn schema(block_number: u64) -> Self {
        Self::new(TableType::Schema, 0, block_number)
    }
}

/// Cache entry with LRU tracking
#[derive(Debug)]
struct CacheEntry {
    /// Cached data
    data: Arc<[u8]>,
    /// Size of the data
    size: usize,
    /// Last access time (for LRU)
    last_access: Instant,
    /// Access count (for LFU)
    access_count: u64,
}

impl CacheEntry {
    fn new(data: Vec<u8>) -> Self {
        let size = data.len();
        Self {
            data: Arc::from(data.into_boxed_slice()),
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

/// Single cache shard for reduced lock contention
#[derive(Debug)]
struct CacheShard {
    /// Entries in this shard
    entries: HashMap<BlockId, CacheEntry>,
    /// LRU order (oldest first)
    lru_order: Vec<BlockId>,
    /// Current memory usage of this shard
    memory_usage: usize,
}

impl CacheShard {
    fn new() -> Self {
        Self {
            entries: HashMap::new(),
            lru_order: Vec::new(),
            memory_usage: 0,
        }
    }

    fn get(&mut self, id: &BlockId) -> Option<Arc<[u8]>> {
        if let Some(entry) = self.entries.get_mut(id) {
            entry.touch();
            // Move to end of LRU (most recently used)
            if let Some(pos) = self.lru_order.iter().position(|x| x == id) {
                self.lru_order.remove(pos);
                self.lru_order.push(id.clone());
            }
            Some(entry.data.clone())
        } else {
            None
        }
    }

    fn insert(&mut self, id: BlockId, data: Vec<u8>) -> usize {
        let size = data.len();

        // Remove old entry if exists
        if let Some(old) = self.entries.remove(&id) {
            self.memory_usage -= old.size;
            self.lru_order.retain(|x| x != &id);
        }

        let entry = CacheEntry::new(data);
        self.memory_usage += size;
        self.entries.insert(id.clone(), entry);
        self.lru_order.push(id);

        size
    }

    fn remove(&mut self, id: &BlockId) -> Option<usize> {
        if let Some(entry) = self.entries.remove(id) {
            self.memory_usage -= entry.size;
            self.lru_order.retain(|x| x != id);
            Some(entry.size)
        } else {
            None
        }
    }

    fn evict_lru(&mut self, required: usize, max_memory: usize) -> usize {
        let mut evicted = 0;

        while self.memory_usage + required > max_memory && !self.lru_order.is_empty() {
            if let Some(id) = self.lru_order.first().cloned() {
                if let Some(size) = self.remove(&id) {
                    evicted += size;
                }
            }
        }

        evicted
    }

    fn can_fit(&self, size: usize, max_memory: usize) -> bool {
        size <= max_memory || self.entries.is_empty()
    }

    fn clear(&mut self) {
        self.entries.clear();
        self.lru_order.clear();
        self.memory_usage = 0;
    }
}

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum memory usage in bytes
    pub max_memory: usize,
    /// Number of shards (should be power of 2)
    pub shard_count: usize,
    /// Enable strict capacity limit
    pub strict_capacity_limit: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_memory: 256 * 1024 * 1024, // 256MB default
            shard_count: 16,
            strict_capacity_limit: false,
        }
    }
}

impl CacheConfig {
    /// Create a new cache configuration with specified memory limit
    pub fn with_memory(max_memory: usize) -> Self {
        Self {
            max_memory,
            ..Default::default()
        }
    }

    /// Get memory limit per shard
    pub fn memory_per_shard(&self) -> usize {
        self.max_memory / self.shard_count
    }
}

/// Block cache with LRU eviction and sharding
#[derive(Debug)]
pub struct BlockCache {
    /// Cache shards
    shards: Vec<Mutex<CacheShard>>,
    /// Configuration
    config: CacheConfig,
    /// Total memory usage
    memory_usage: AtomicUsize,
    /// Hit count
    hits: AtomicU64,
    /// Miss count
    misses: AtomicU64,
    /// Eviction count
    evictions: AtomicU64,
}

impl BlockCache {
    /// Create a new block cache with default configuration
    pub fn new() -> Self {
        Self::with_config(CacheConfig::default())
    }

    /// Create a new block cache with specified memory limit
    pub fn with_memory(max_memory: usize) -> Self {
        Self::with_config(CacheConfig::with_memory(max_memory))
    }

    /// Create a new block cache with specified configuration
    pub fn with_config(config: CacheConfig) -> Self {
        let shard_count = config.shard_count;
        let shards = (0..shard_count).map(|_| Mutex::new(CacheShard::new())).collect();

        Self {
            shards,
            config,
            memory_usage: AtomicUsize::new(0),
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            evictions: AtomicU64::new(0),
        }
    }

    /// Get the shard index for a block ID
    fn shard_index(&self, id: &BlockId) -> usize {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        id.hash(&mut hasher);
        (hasher.finish() as usize) % self.shards.len()
    }

    /// Get a block from the cache
    pub fn get(&self, id: &BlockId) -> Option<Arc<[u8]>> {
        let shard_idx = self.shard_index(id);
        let mut shard = self.shards[shard_idx].lock();

        if shard.get(id).is_some() {
            self.hits.fetch_add(1, Ordering::Relaxed);
            shard.get(id)
        } else {
            self.misses.fetch_add(1, Ordering::Relaxed);
            None
        }
    }

    /// Insert a block into the cache
    /// Returns true if insertion succeeded
    pub fn insert(&self, id: BlockId, data: Vec<u8>) -> bool {
        let size = data.len();
        let shard_idx = self.shard_index(&id);
        let max_per_shard = self.config.memory_per_shard();

        let mut shard = self.shards[shard_idx].lock();

        // Check if we need to evict
        if shard.memory_usage + size > max_per_shard {
            let evicted = shard.evict_lru(size, max_per_shard);
            if evicted > 0 {
                self.memory_usage.fetch_sub(evicted, Ordering::Relaxed);
                self.evictions.fetch_add(1, Ordering::Relaxed);
            }
        }

        // Check if block can fit (allow oversized blocks if shard is empty)
        if !shard.can_fit(size, max_per_shard) {
            return false;
        }

        // Check strict capacity limit
        if self.config.strict_capacity_limit {
            let current = self.memory_usage.load(Ordering::Relaxed);
            if current + size > self.config.max_memory {
                return false;
            }
        }

        shard.insert(id, data);
        self.memory_usage.fetch_add(size, Ordering::Relaxed);
        true
    }

    /// Remove a block from the cache
    pub fn remove(&self, id: &BlockId) -> bool {
        let shard_idx = self.shard_index(id);
        let mut shard = self.shards[shard_idx].lock();

        if let Some(size) = shard.remove(id) {
            self.memory_usage.fetch_sub(size, Ordering::Relaxed);
            true
        } else {
            false
        }
    }

    /// Check if a block exists in the cache
    pub fn contains(&self, id: &BlockId) -> bool {
        let shard_idx = self.shard_index(id);
        let shard = self.shards[shard_idx].lock();
        shard.entries.contains_key(id)
    }

    /// Clear all entries from the cache
    pub fn clear(&self) {
        for shard in &self.shards {
            shard.lock().clear();
        }
        self.memory_usage.store(0, Ordering::Relaxed);
    }

    /// Get current memory usage
    pub fn memory_usage(&self) -> usize {
        self.memory_usage.load(Ordering::Relaxed)
    }

    /// Get maximum memory
    pub fn max_memory(&self) -> usize {
        self.config.max_memory
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = hits + misses;

        CacheStats {
            hits,
            misses,
            hit_rate: if total > 0 { hits as f64 / total as f64 } else { 0.0 },
            evictions: self.evictions.load(Ordering::Relaxed),
            memory_usage: self.memory_usage.load(Ordering::Relaxed),
            max_memory: self.config.max_memory,
            entry_count: self.entry_count(),
        }
    }

    /// Get total entry count
    fn entry_count(&self) -> usize {
        self.shards
            .iter()
            .map(|s| s.lock().entries.len())
            .sum()
    }

    /// Get memory utilization (0.0 - 1.0)
    pub fn utilization(&self) -> f32 {
        if self.config.max_memory == 0 {
            return 0.0;
        }
        self.memory_usage.load(Ordering::Relaxed) as f32 / self.config.max_memory as f32
    }

    /// Prune entries to reduce memory usage below threshold
    pub fn prune(&self, target_usage: usize) {
        let current = self.memory_usage.load(Ordering::Relaxed);
        if current <= target_usage {
            return;
        }

        let to_evict = current - target_usage;
        let mut evicted = 0;

        // Evict from each shard proportionally
        for shard in &self.shards {
            if evicted >= to_evict {
                break;
            }
            let mut shard = shard.lock();
            while evicted < to_evict && !shard.lru_order.is_empty() {
                if let Some(id) = shard.lru_order.first().cloned() {
                    if let Some(size) = shard.remove(&id) {
                        evicted += size;
                        self.memory_usage.fetch_sub(size, Ordering::Relaxed);
                        self.evictions.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
        }
    }
}

impl Default for BlockCache {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for BlockCache {
    fn clone(&self) -> Self {
        Self::with_config(self.config.clone())
    }
}

/// Cache statistics
#[derive(Debug, Clone, Copy)]
pub struct CacheStats {
    /// Number of cache hits
    pub hits: u64,
    /// Number of cache misses
    pub misses: u64,
    /// Hit rate (0.0 - 1.0)
    pub hit_rate: f64,
    /// Number of evictions
    pub evictions: u64,
    /// Current memory usage
    pub memory_usage: usize,
    /// Maximum memory
    pub max_memory: usize,
    /// Number of entries
    pub entry_count: usize,
}

impl CacheStats {
    /// Format bytes as human-readable string
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

impl std::fmt::Display for CacheStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "Cache Stats: {}/{} ({:.1}%)",
            Self::format_bytes(self.memory_usage),
            Self::format_bytes(self.max_memory),
            self.memory_usage as f64 / self.max_memory as f64 * 100.0
        )?;
        writeln!(
            f,
            "  Hits: {}, Misses: {}, Hit Rate: {:.1}%",
            self.hits, self.misses, self.hit_rate * 100.0
        )?;
        writeln!(f, "  Evictions: {}", self.evictions)?;
        write!(f, "  Entries: {}", self.entry_count)
    }
}

/// Shared block cache wrapper
pub type SharedBlockCache = Arc<BlockCache>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        let cache = BlockCache::with_memory(1024);

        let id = BlockId::vertex(1, 0);
        let data = vec![1, 2, 3, 4, 5];

        assert!(cache.insert(id.clone(), data.clone()));
        assert!(cache.contains(&id));

        let cached = cache.get(&id);
        assert!(cached.is_some());
        assert_eq!(&*cached.unwrap(), &data[..]);
    }

    #[test]
    fn test_lru_eviction() {
        // Use a single shard cache to ensure all blocks go to the same shard
        let config = CacheConfig {
            max_memory: 200,
            shard_count: 1,
            strict_capacity_limit: false,
        };
        let cache = BlockCache::with_config(config);

        // Insert blocks that exceed capacity
        // With 1 shard, all blocks go to the same shard with 200 bytes limit
        cache.insert(BlockId::vertex(1, 0), vec![0u8; 80]);
        cache.insert(BlockId::vertex(1, 1), vec![0u8; 80]);
        cache.insert(BlockId::vertex(1, 2), vec![0u8; 80]); // Should trigger eviction

        // First block should be evicted
        assert!(!cache.contains(&BlockId::vertex(1, 0)));
        // Later blocks should still exist
        assert!(cache.contains(&BlockId::vertex(1, 1)));
        assert!(cache.contains(&BlockId::vertex(1, 2)));
    }

    #[test]
    fn test_cache_stats() {
        let cache = BlockCache::with_memory(1024);

        let id = BlockId::vertex(1, 0);
        let data = vec![1, 2, 3, 4, 5];

        cache.insert(id.clone(), data);

        // Hit
        cache.get(&id);
        // Miss
        cache.get(&BlockId::vertex(2, 0));

        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert!((stats.hit_rate - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_different_table_types() {
        let cache = BlockCache::with_memory(1024);

        let vertex_id = BlockId::vertex(1, 0);
        let edge_id = BlockId::edge(1, 0);
        let property_id = BlockId::property(1, 0);
        let index_id = BlockId::index(1, 0);
        let schema_id = BlockId::schema(0);

        cache.insert(vertex_id.clone(), vec![1]);
        cache.insert(edge_id.clone(), vec![2]);
        cache.insert(property_id.clone(), vec![3]);
        cache.insert(index_id.clone(), vec![4]);
        cache.insert(schema_id.clone(), vec![5]);

        assert!(cache.contains(&vertex_id));
        assert!(cache.contains(&edge_id));
        assert!(cache.contains(&property_id));
        assert!(cache.contains(&index_id));
        assert!(cache.contains(&schema_id));
    }

    #[test]
    fn test_clear() {
        let cache = BlockCache::with_memory(1024);

        cache.insert(BlockId::vertex(1, 0), vec![1, 2, 3]);
        cache.insert(BlockId::vertex(1, 1), vec![4, 5, 6]);

        cache.clear();

        assert!(!cache.contains(&BlockId::vertex(1, 0)));
        assert!(!cache.contains(&BlockId::vertex(1, 1)));
        assert_eq!(cache.memory_usage(), 0);
    }

    #[test]
    fn test_remove() {
        let cache = BlockCache::with_memory(1024);

        let id = BlockId::vertex(1, 0);
        cache.insert(id.clone(), vec![1, 2, 3]);

        assert!(cache.remove(&id));
        assert!(!cache.contains(&id));
        assert!(!cache.remove(&id)); // Already removed
    }

    #[test]
    fn test_prune() {
        let cache = BlockCache::with_memory(1000);

        // Insert multiple blocks
        for i in 0..10 {
            cache.insert(BlockId::vertex(1, i), vec![0u8; 100]);
        }

        // Prune to half capacity
        cache.prune(500);

        assert!(cache.memory_usage() <= 500);
    }

    #[test]
    fn test_concurrent_access() {
        use std::sync::Arc;
        use std::thread;

        let cache = Arc::new(BlockCache::with_memory(10000));
        let mut handles = vec![];

        for t in 0..4 {
            let cache = cache.clone();
            handles.push(thread::spawn(move || {
                for i in 0..100 {
                    let id = BlockId::vertex(t, i);
                    cache.insert(id.clone(), vec![0u8; 10]);
                    cache.get(&id);
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let stats = cache.stats();
        assert!(stats.entry_count > 0);
    }

    #[test]
    fn test_memory_utilization() {
        let cache = BlockCache::with_memory(1000);

        assert_eq!(cache.utilization(), 0.0);

        cache.insert(BlockId::vertex(1, 0), vec![0u8; 500]);
        assert!((cache.utilization() - 0.5).abs() < 0.01);

        cache.insert(BlockId::vertex(1, 1), vec![0u8; 500]);
        assert!((cache.utilization() - 1.0).abs() < 0.01);
    }
}
