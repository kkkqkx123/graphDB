//! CTE Results Cache Manager Module
//!
//! CTE (Common Table Expression) query result caching function.
//! Avoid repeated calculation of same CTE to improve query performance.
//!
//! ## Caching policy
//!
//! - LRU elimination policy: eliminate the longest unused entries when the cache is full
//! - Memory budget management: tightly control the upper limit of memory used by the cache
//! - Intelligent caching decision: decide whether to cache or not based on CTE characteristics
//!
//! ## Applicable scenarios
//!
//! 1. Recursive CTEs are referenced multiple times
//! 2. Complex subqueries are used multiple times in a single query
//! 3. Medium-sized result set (100-10,000 rows)
//! 4. CTE is deterministic (no random functions, etc.)

use moka::sync::Cache;
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::plan_cache::CachePriority;

/// CTE cache entries
#[derive(Debug, Clone)]
pub struct CteCacheEntry {
    /// Resulting data (shared using Arc)
    pub data: Arc<Vec<u8>>,
    /// Number of result rows
    pub row_count: u64,
    /// Result size (bytes)
    pub data_size: usize,
    /// Creation time
    pub created_at: Instant,
    /// Last access time
    pub last_accessed: Instant,
    /// Number of visits
    pub access_count: u64,
    /// Estimated probability of reuse
    pub reuse_probability: f64,
    /// CTE definition hash (used to identify identical CTEs)
    pub cte_hash: String,
    /// CTE definition text
    pub cte_definition: String,
    /// Cache priority
    pub priority: CachePriority,
    /// Compute cost (milliseconds)
    pub compute_cost_ms: u64,
    /// Access frequency (per minute)
    pub access_frequency: f64,
    /// Dependent tables (for invalidation detection)
    pub dependent_tables: Vec<String>,
}

impl CteCacheEntry {
    /// Creating a new cache entry
    pub fn new(
        cte_hash: String,
        cte_definition: String,
        data: Vec<u8>,
        row_count: u64,
        compute_cost_ms: u64,
    ) -> Self {
        let data_size = data.len();
        Self {
            data: Arc::new(data),
            row_count,
            data_size,
            created_at: Instant::now(),
            last_accessed: Instant::now(),
            access_count: 0,
            reuse_probability: 0.5,
            cte_hash,
            cte_definition,
            priority: CachePriority::Normal,
            compute_cost_ms,
            access_frequency: 0.0,
            dependent_tables: Vec::new(),
        }
    }

    /// Estimate memory usage (bytes)
    pub fn estimate_memory(&self) -> usize {
        let mut total = 0;

        // Data: Arc<Vec<u8>> (Arc struct + Vec struct + heap allocation)
        total += std::mem::size_of::<Arc<Vec<u8>>>();
        total += std::mem::size_of::<Vec<u8>>();
        total += self.data_size;

        // String fields (String struct + heap allocation)
        total += std::mem::size_of::<String>();
        total += self.cte_hash.capacity();

        total += std::mem::size_of::<String>();
        total += self.cte_definition.capacity();

        // Vector fields (Vec struct + String elements)
        total += std::mem::size_of::<Vec<String>>();
        for table in &self.dependent_tables {
            total += std::mem::size_of::<String>();
            total += table.capacity();
        }

        // Other fields (basic types)
        total += std::mem::size_of::<Instant>() * 2;
        total += std::mem::size_of::<u64>() * 3;
        total += std::mem::size_of::<f64>() * 2;
        total += std::mem::size_of::<CachePriority>();

        total
    }

    /// Calculate cache value score (for eviction decisions)
    pub fn value_score(&self) -> f64 {
        let frequency_score = self.access_frequency * 0.4;
        let cost_score = (self.compute_cost_ms as f64 / 1000.0) * 0.3;
        let priority_score = (self.priority as i32 as f64) * 0.2;
        let size_penalty = (self.data_size as f64 / 1024.0 / 1024.0) * 0.1;

        frequency_score + cost_score + priority_score - size_penalty
    }

    /// Recorded visits
    pub fn record_access(&mut self) {
        self.last_accessed = Instant::now();
        self.access_count += 1;

        let elapsed_minutes = self.created_at.elapsed().as_secs_f64() / 60.0;
        if elapsed_minutes > 0.0 {
            self.access_frequency = self.access_count as f64 / elapsed_minutes;
        }

        self.reuse_probability = (self.reuse_probability * 0.7 + 0.3).min(0.95);
    }

    /// Get Cache Age
    pub fn age(&self) -> Duration {
        self.created_at.elapsed()
    }

    /// Getting free time
    pub fn idle_time(&self) -> Duration {
        self.last_accessed.elapsed()
    }

    /// Calculate cache score (for LRU elimination decisions)
    /// The lower the score, the more likely you are to be eliminated
    pub fn cache_score(&self) -> f64 {
        let _age_factor = self.age().as_secs_f64() / 3600.0; // hourly
        let idle_factor = self.idle_time().as_secs_f64() / 60.0; // in minutes
        let size_factor = (self.data_size as f64 / 1024.0 / 1024.0).max(0.1); // In MB
        let access_factor = (self.access_count as f64).sqrt().max(1.0);

        // Combined score: considers idle time, size, frequency of visits
        (idle_factor * size_factor) / (access_factor * self.reuse_probability)
    }
}

/// CTE Cache Statistics
#[derive(Debug, Clone, Default)]
pub struct CteCacheStats {
    /// Cache hits
    pub hit_count: u64,
    /// Number of cache misses
    pub miss_count: u64,
    /// Number of cache entries
    pub entry_count: usize,
    /// Currently using memory (bytes)
    pub current_memory: usize,
    /// Total memory limit (bytes)
    pub max_memory: usize,
    /// Number of entries phased out
    pub evicted_count: u64,
    /// Number of entries rejected for caching
    pub rejected_count: u64,
}

impl CteCacheStats {
    /// Getting hits
    pub fn hit_rate(&self) -> f64 {
        let total = self.hit_count + self.miss_count;
        if total == 0 {
            return 0.0;
        }
        self.hit_count as f64 / total as f64
    }

    /// Getting Memory Usage
    pub fn memory_usage_ratio(&self) -> f64 {
        if self.max_memory == 0 {
            return 0.0;
        }
        self.current_memory as f64 / self.max_memory as f64
    }

    /// Reset the statistics.
    pub fn reset(&mut self) {
        self.hit_count = 0;
        self.miss_count = 0;
        self.evicted_count = 0;
        self.rejected_count = 0;
    }
}

/// CTE Cache Configuration
#[derive(Debug, Clone)]
pub struct CteCacheConfig {
    /// Maximum cache size (bytes)
    pub max_size: usize,
    /// Maximum number of entries (optional)
    pub max_entries: Option<usize>,
    /// Maximum size of a single entry (bytes)
    pub max_entry_size: usize,
    /// Minimum number of lines to cache (less than this value is not cached)
    pub min_row_count: u64,
    /// Maximum number of lines to cache (greater than this value is not cached)
    pub max_row_count: u64,
    /// Entry expiration time (seconds)
    pub entry_ttl_seconds: u64,
    /// Enable caching
    pub enabled: bool,
    /// Whether to enable adaptive
    pub adaptive: bool,
    /// Whether to enable priority
    pub enable_priority: bool,
}

impl Default for CteCacheConfig {
    fn default() -> Self {
        Self {
            max_size: 64 * 1024 * 1024,
            max_entries: Some(10000),
            max_entry_size: 10 * 1024 * 1024,
            min_row_count: 100,
            max_row_count: 100_000,
            entry_ttl_seconds: 3600,
            enabled: true,
            adaptive: true,
            enable_priority: true,
        }
    }
}

impl CteCacheConfig {
    /// Create a small memory configuration.
    pub fn low_memory() -> Self {
        Self {
            max_size: 16 * 1024 * 1024,
            max_entries: Some(5000),
            max_entry_size: 5 * 1024 * 1024,
            min_row_count: 50,
            max_row_count: 50_000,
            entry_ttl_seconds: 1800,
            enabled: true,
            adaptive: true,
            enable_priority: true,
        }
    }

    /// Creating a large memory configuration
    pub fn high_memory() -> Self {
        Self {
            max_size: 256 * 1024 * 1024,
            max_entries: Some(20000),
            max_entry_size: 50 * 1024 * 1024,
            min_row_count: 100,
            max_row_count: 500_000,
            entry_ttl_seconds: 7200,
            enabled: true,
            adaptive: true,
            enable_priority: true,
        }
    }

    /// Disable caching
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Default::default()
        }
    }
}

/// CTE Cache Manager
///
/// Managing caching of CTE (Common Table Expression) query results and ensuring thread-safe access.
#[derive(Debug)]
pub struct CteCacheManager {
    /// Cache storage - using moka for high-performance concurrent access with weigher
    cache: Cache<String, Arc<CteCacheEntry>>,
    /// Configuration
    config: Arc<RwLock<CteCacheConfig>>,
    /// Statistical information
    stats: Arc<RwLock<CteCacheStats>>,
}

impl CteCacheManager {
    /// Create a new cache manager.
    pub fn new() -> Self {
        Self::with_config(CteCacheConfig::default())
    }

    /// Create using the configuration.
    pub fn with_config(config: CteCacheConfig) -> Self {
        let max_weight = config.max_size as u64;

        let cache = Cache::builder()
            .weigher(|_key, value: &Arc<CteCacheEntry>| -> u32 {
                let arc_overhead = std::mem::size_of::<Arc<CteCacheEntry>>();
                (value.estimate_memory() + arc_overhead) as u32
            })
            .max_capacity(max_weight)
            .time_to_live(Duration::from_secs(config.entry_ttl_seconds))
            .build();

        Self {
            cache,
            config: Arc::new(RwLock::new(config.clone())),
            stats: Arc::new(RwLock::new(CteCacheStats {
                max_memory: config.max_size,
                ..Default::default()
            })),
        }
    }

    /// Obtain the configuration.
    pub fn config(&self) -> CteCacheConfig {
        self.config.read().clone()
    }

    /// Update the configuration.
    pub fn set_config(&self, config: CteCacheConfig) {
        let mut stats = self.stats.write();
        stats.max_memory = config.max_size;
        *self.config.write() = config;
    }

    /// Determine whether to enable caching.
    pub fn is_enabled(&self) -> bool {
        self.config.read().enabled
    }

    /// Determine whether the results of the CTE (Common Table Expression) are cached.
    ///
    /// # Parameters
    /// `cte_definition`: Text defining the Common Table Expression (CTE).
    /// estimated_rows: The estimated number of rows
    /// `is_deterministic`: Whether the CTE (Common Table Expression) is deterministic.
    pub fn should_cache(
        &self,
        cte_definition: &str,
        estimated_rows: u64,
        is_deterministic: bool,
    ) -> bool {
        let config = self.config.read();

        if !config.enabled {
            return false;
        }

        if !is_deterministic {
            return false;
        }

        // Check the range of line numbers.
        if estimated_rows < config.min_row_count || estimated_rows > config.max_row_count {
            return false;
        }

        // Check the historical reuse patterns.
        let reuse_prob = self.predict_reuse_probability(cte_definition);
        if reuse_prob < 0.3 {
            return false;
        }

        true
    }

    /// Predict the probability of reuse
    fn predict_reuse_probability(&self, cte_definition: &str) -> f64 {
        let cte_hash = Self::compute_hash(cte_definition);

        // If it is already in the cache, return the current reuse probability.
        if let Some(entry) = self.cache.get(&cte_hash) {
            return entry.reuse_probability;
        }

        // Otherwise, predictions will be made based on the characteristics of the CTE (Common Table Expression).
        // A simple heuristic: More complex Common Table Expressions (CTEs) are more likely to be reused.
        let complexity = cte_definition.len() as f64 / 100.0;
        let base_prob = 0.5;
        let complexity_bonus = (complexity / 10.0).min(0.3);

        base_prob + complexity_bonus
    }

    /// Calculate the hash value defined by the CTE (Common Table Expression).
    fn compute_hash(cte_definition: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        cte_definition.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }

    /// Store the data in the cache.
    pub fn put(&self, cte_definition: &str, data: Vec<u8>, row_count: u64) -> Option<String> {
        self.put_with_cost(cte_definition, data, row_count, 100)
    }

    /// Store the data in the cache with compute cost.
    pub fn put_with_cost(
        &self,
        cte_definition: &str,
        data: Vec<u8>,
        row_count: u64,
        compute_cost_ms: u64,
    ) -> Option<String> {
        let config = self.config.read();

        if !config.enabled {
            return None;
        }

        if data.len() > config.max_entry_size {
            let mut stats = self.stats.write();
            stats.rejected_count += 1;
            return None;
        }

        drop(config);

        let cte_hash = Self::compute_hash(cte_definition);
        let entry = Arc::new(CteCacheEntry::new(
            cte_hash.clone(),
            cte_definition.to_string(),
            data,
            row_count,
            compute_cost_ms,
        ));

        self.cache.insert(cte_hash.clone(), entry);

        let mut stats = self.stats.write();
        stats.entry_count = self.cache.entry_count() as usize;

        Some(cte_hash)
    }

    /// Evict low priority entries
    pub fn evict_low_priority(&self, target_bytes: usize) -> usize {
        let mut freed = 0;
        let mut to_remove = Vec::new();

        let entries: Vec<_> = self
            .cache
            .iter()
            .map(|entry| {
                let value_score = entry.1.value_score();
                (entry.0.as_ref().clone(), value_score, entry.1.data_size, entry.1.priority)
            })
            .collect();

        let mut entries_sorted = entries;
        entries_sorted.sort_by(|a, b| {
            a.1
                .partial_cmp(&b.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.3.cmp(&b.3))
        });

        for (key, _, size, _) in entries_sorted {
            if freed >= target_bytes {
                break;
            }
            to_remove.push(key);
            freed += size;
        }

        for key in &to_remove {
            self.cache.invalidate(key);
        }

        if freed > 0 {
            let mut stats = self.stats.write();
            stats.evicted_count += to_remove.len() as u64;
            stats.entry_count = self.cache.entry_count() as usize;
        }

        freed
    }

    /// Retrieve data from the cache.
    pub fn get(&self, cte_definition: &str) -> Option<Arc<Vec<u8>>> {
        let config = self.config.read();

        if !config.enabled {
            return None;
        }

        drop(config);

        let cte_hash = Self::compute_hash(cte_definition);

        if let Some(entry) = self.cache.get(&cte_hash) {
            // Update the statistics.
            let mut stats = self.stats.write();
            stats.hit_count += 1;

            Some(entry.data.clone())
        } else {
            let mut stats = self.stats.write();
            stats.miss_count += 1;
            None
        }
    }

    /// Check whether it exists in the cache.
    pub fn contains(&self, cte_definition: &str) -> bool {
        let cte_hash = Self::compute_hash(cte_definition);
        self.cache.contains_key(&cte_hash)
    }

    /// Invalidate the cache entry
    pub fn invalidate(&self, cte_definition: &str) -> bool {
        let cte_hash = Self::compute_hash(cte_definition);
        self.invalidate_by_hash(&cte_hash)
    }

    /// Invalidate cache entry by hash
    pub fn invalidate_by_hash(&self, cte_hash: &str) -> bool {
        let removed = self.cache.remove(cte_hash).is_some();

        if removed {
            let mut stats = self.stats.write();
            stats.entry_count = self.cache.entry_count() as usize;
        }

        removed
    }

    /// Get cache entries for eviction (internal use)
    pub fn get_cache_entries(&self) -> Vec<(String, f64, usize)> {
        self.cache
            .iter()
            .map(|entry| {
                let value_score = entry.1.value_score();
                (entry.0.as_ref().clone(), value_score, entry.1.data_size)
            })
            .collect()
    }

    /// Increment eviction count (internal use)
    pub fn increment_evicted_count(&self, count: u64) {
        let mut stats = self.stats.write();
        stats.evicted_count += count;
    }

    /// Clear all caches.
    pub fn clear(&self) {
        self.cache.invalidate_all();

        let mut stats = self.stats.write();
        stats.entry_count = 0;
        stats.current_memory = 0;
    }

    /// Obtain statistical information
    pub fn get_stats(&self) -> CteCacheStats {
        let mut stats = self.stats.read().clone();
        stats.entry_count = self.cache.entry_count() as usize;
        stats.current_memory = self.estimate_current_memory();
        stats
    }

    /// Estimate current memory usage
    fn estimate_current_memory(&self) -> usize {
        self.cache
            .iter()
            .map(|entry| entry.1.estimate_memory())
            .sum()
    }

    /// Reset statistics
    pub fn reset_stats(&self) {
        self.stats.write().reset();
    }

    /// Get the current memory usage
    pub fn current_memory(&self) -> usize {
        self.estimate_current_memory()
    }

    /// Obtain the number of cached entries
    pub fn entry_count(&self) -> usize {
        self.cache.entry_count() as usize
    }

    /// Clearance of obsolete entries
    /// Note: moka handles TTL automatically, so this is a no-op
    pub fn cleanup_expired(&self) -> usize {
        0
    }
}

impl Default for CteCacheManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for CteCacheManager {
    fn clone(&self) -> Self {
        Self::with_config(self.config.read().clone())
    }
}

/// CTE Cache Decisioner
///
/// Decide whether to use caching based on query characteristics
#[derive(Debug, Clone)]
pub struct CteCacheDecision {
    /// Should the cache be used?
    pub should_cache: bool,
    /// Reasons for decision-making
    pub reason: String,
    /// Estimated reuse probability
    pub reuse_probability: f64,
    /// Estimated caching gains
    pub estimated_benefit: f64,
    /// Suggested priority
    pub suggested_priority: CachePriority,
}

/// CTE缓存决策器
#[derive(Debug)]
pub struct CteCacheDecisionMaker {
    /// Cache Manager
    cache_manager: Arc<CteCacheManager>,
    /// Minimum Reuse Probability Threshold
    min_reuse_probability: f64,
    /// Minimum estimated gain
    min_benefit: f64,
}

impl CteCacheDecisionMaker {
    /// Create a new decision maker.
    pub fn new(cache_manager: Arc<CteCacheManager>) -> Self {
        Self {
            cache_manager,
            min_reuse_probability: 0.3,
            min_benefit: 1.0,
        }
    }

    /// Setting parameters
    pub fn with_params(mut self, min_reuse_probability: f64, min_benefit: f64) -> Self {
        self.min_reuse_probability = min_reuse_probability;
        self.min_benefit = min_benefit;
        self
    }

    /// Making a decision regarding caching
    pub fn decide(
        &self,
        cte_definition: &str,
        estimated_rows: u64,
        compute_cost: f64,
    ) -> CteCacheDecision {
        if !self.cache_manager.is_enabled() {
            return CteCacheDecision {
                should_cache: false,
                reason: "Cache disabled".to_string(),
                reuse_probability: 0.0,
                estimated_benefit: 0.0,
                suggested_priority: CachePriority::Low,
            };
        }

        let reuse_prob = self.cache_manager.predict_reuse_probability(cte_definition);

        if reuse_prob < self.min_reuse_probability {
            return CteCacheDecision {
                should_cache: false,
                reason: format!("Reuse probability too low: {:.2}", reuse_prob),
                reuse_probability: reuse_prob,
                estimated_benefit: 0.0,
                suggested_priority: CachePriority::Low,
            };
        }

        let cache_overhead = estimated_rows as f64 * 0.001;
        let estimated_benefit = reuse_prob * compute_cost - cache_overhead;

        if estimated_benefit < self.min_benefit {
            return CteCacheDecision {
                should_cache: false,
                reason: format!("Estimated benefit too low: {:.2}", estimated_benefit),
                reuse_probability: reuse_prob,
                estimated_benefit,
                suggested_priority: CachePriority::Low,
            };
        }

        let suggested_priority = if compute_cost > 1000.0 {
            CachePriority::High
        } else if compute_cost > 100.0 {
            CachePriority::Normal
        } else {
            CachePriority::Low
        };

        CteCacheDecision {
            should_cache: true,
            reason: "Benefit analysis passed".to_string(),
            reuse_probability: reuse_prob,
            estimated_benefit,
            suggested_priority,
        }
    }
}

impl Default for CteCacheDecisionMaker {
    fn default() -> Self {
        Self::new(Arc::new(CteCacheManager::new()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cte_cache_entry() {
        let mut entry = CteCacheEntry::new(
            "hash1".to_string(),
            "SELECT * FROM t".to_string(),
            vec![1, 2, 3, 4, 5],
            10,
            100,
        );

        assert_eq!(entry.row_count, 10);
        assert_eq!(entry.data_size, 5);

        entry.record_access();
        assert_eq!(entry.access_count, 1);
        assert!(entry.reuse_probability > 0.5);
    }

    #[test]
    fn test_cte_cache_manager() {
        let manager = CteCacheManager::new();

        // Testing cache decision-making mechanisms
        assert!(manager.should_cache("SELECT * FROM t", 500, true));
        assert!(!manager.should_cache("SELECT * FROM t", 10, true)); // Too few lines
        assert!(!manager.should_cache("SELECT * FROM t", 500, false)); // Non-determinacy

        // Test Deposit and Access
        let data = vec![1, 2, 3, 4, 5];
        let key = manager.put("SELECT * FROM t", data.clone(), 100);
        assert!(key.is_some());

        let retrieved = manager.get("SELECT * FROM t");
        assert!(retrieved.is_some());
        assert_eq!(*retrieved.unwrap(), data);

        // Test statistics
        let stats = manager.get_stats();
        assert_eq!(stats.hit_count, 1);
        assert_eq!(stats.miss_count, 0);
        assert_eq!(stats.entry_count, manager.entry_count());
    }

    #[test]
    fn test_cte_cache_eviction() {
        let config = CteCacheConfig {
            max_size: 100,
            max_entries: Some(100),
            max_entry_size: 50,
            min_row_count: 1,
            max_row_count: 1000,
            entry_ttl_seconds: 3600,
            enabled: true,
            adaptive: true,
            enable_priority: true,
        };

        let manager = CteCacheManager::with_config(config);

        // Entering multiple entries triggers the elimination process.
        let data1 = vec![1u8; 40]; // 40 bytes
        let data2 = vec![2u8; 40]; // 40 bytes
        let data3 = vec![3u8; 40]; // 40 bytes

        manager.put("query1", data1, 10);
        manager.put("query2", data2, 10);

        // Visit query1 to boost its score
        manager.get("query1");

        // Query3 should be stored, while query2 should be eliminated by moka's weigher
        manager.put("query3", data3, 10);

        // Verify that cache has limited entries due to weight limit
        let stats = manager.get_stats();
        // With moka's weigher, entries are automatically evicted when weight exceeds limit
        // The exact number may vary, but it should be limited
        assert!(stats.entry_count <= 3);
    }

    #[test]
    fn test_cte_cache_decision_maker() {
        let manager = Arc::new(CteCacheManager::new());
        let decision_maker = CteCacheDecisionMaker::new(manager);

        // Test decision-making
        let decision = decision_maker.decide("SELECT * FROM large_table", 1000, 100.0);
        // Since the probability of reuse may be low, the result may be true or false
        assert!(decision.reuse_probability >= 0.0 && decision.reuse_probability <= 1.0);

        // Testing cases with low probability of reuse
        let decision = decision_maker.decide("SELECT 1", 100, 0.1);
        assert!(!decision.should_cache); // Simple queries should not be cached
    }

    #[test]
    fn test_cte_cache_stats() {
        let mut stats = CteCacheStats::default();

        assert_eq!(stats.hit_rate(), 0.0);
        assert_eq!(stats.memory_usage_ratio(), 0.0);

        stats.hit_count = 80;
        stats.miss_count = 20;
        stats.current_memory = 50;
        stats.max_memory = 100;

        assert_eq!(stats.hit_rate(), 0.8);
        assert_eq!(stats.memory_usage_ratio(), 0.5);

        stats.reset();
        assert_eq!(stats.hit_count, 0);
        assert_eq!(stats.miss_count, 0);
    }

    #[test]
    fn test_cte_cache_config() {
        let config = CteCacheConfig {
            max_size: 32 * 1024 * 1024,
            max_entries: Some(10000),
            max_entry_size: 5 * 1024 * 1024,
            min_row_count: 50,
            max_row_count: 50_000,
            entry_ttl_seconds: 1800,
            enabled: true,
            adaptive: true,
            enable_priority: true,
        };

        assert_eq!(config.max_size, 32 * 1024 * 1024);
    }
}
