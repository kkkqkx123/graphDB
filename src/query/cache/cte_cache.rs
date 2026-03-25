//! CTE Results Cache Manager Module
//!
//! CTE (Common Table Expression) query result caching function.
//! Avoid repeated calculation of the same CTE to improve query performance.
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

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

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
}

impl CteCacheEntry {
    /// Creating a new cache entry
    pub fn new(cte_hash: String, cte_definition: String, data: Vec<u8>, row_count: u64) -> Self {
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
        }
    }

    /// Recorded visits
    pub fn record_access(&mut self) {
        self.last_accessed = Instant::now();
        self.access_count += 1;
        // Update reuse probability: the more visits, the higher the reuse probability
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
}

impl Default for CteCacheConfig {
    fn default() -> Self {
        Self {
            max_size: 64 * 1024 * 1024,       // 64MB
            max_entry_size: 10 * 1024 * 1024, // 10MB
            min_row_count: 100,               // At least 100 lines
            max_row_count: 100_000,           // Up to 100,000 lines.
            entry_ttl_seconds: 3600,          // 1 hour
            enabled: true,
        }
    }
}

impl CteCacheConfig {
    /// Create a small memory configuration.
    pub fn low_memory() -> Self {
        Self {
            max_size: 16 * 1024 * 1024,      // 16MB
            max_entry_size: 5 * 1024 * 1024, // 5MB
            min_row_count: 50,
            max_row_count: 50_000,
            entry_ttl_seconds: 1800, // 30 minutes
            enabled: true,
        }
    }

    /// Creating a large memory configuration
    pub fn high_memory() -> Self {
        Self {
            max_size: 256 * 1024 * 1024,      // 256MB
            max_entry_size: 50 * 1024 * 1024, // 50MB
            min_row_count: 100,
            max_row_count: 500_000,
            entry_ttl_seconds: 7200, // 2 hours
            enabled: true,
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
/// Managing the caching of CTE (Common Table Expression) query results and ensuring thread-safe access.
#[derive(Debug)]
pub struct CteCacheManager {
    /// Cache storage
    cache: RwLock<HashMap<String, CteCacheEntry>>,
    /// Configuration
    config: RwLock<CteCacheConfig>,
    /// Statistical information
    stats: RwLock<CteCacheStats>,
    /// Memory currently in use
    current_memory: RwLock<usize>,
}

impl CteCacheManager {
    /// Create a new cache manager.
    pub fn new() -> Self {
        Self::with_config(CteCacheConfig::default())
    }

    /// Create using the configuration.
    pub fn with_config(config: CteCacheConfig) -> Self {
        let max_memory = config.max_size;
        Self {
            cache: RwLock::new(HashMap::new()),
            config: RwLock::new(config),
            stats: RwLock::new(CteCacheStats {
                max_memory,
                ..Default::default()
            }),
            current_memory: RwLock::new(0),
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

        // If the new configuration requires less space, it may be necessary to eliminate some entries.
        self.evict_if_needed();
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
        let cache = self.cache.read();
        let cte_hash = Self::compute_hash(cte_definition);

        // If it is already in the cache, return the current reuse probability.
        if let Some(entry) = cache.get(&cte_hash) {
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
        let config = self.config.read();

        if !config.enabled {
            return None;
        }

        // Check the data size.
        if data.len() > config.max_entry_size {
            let mut stats = self.stats.write();
            stats.rejected_count += 1;
            return None;
        }

        drop(config);

        // Make sure there is enough space.
        self.evict_if_needed();

        let cte_hash = Self::compute_hash(cte_definition);
        let entry = CteCacheEntry::new(
            cte_hash.clone(),
            cte_definition.to_string(),
            data,
            row_count,
        );

        let entry_size = entry.data_size;
        let mut cache = self.cache.write();

        // Update memory usage
        *self.current_memory.write() += entry_size;

        // Insert the cache
        cache.insert(cte_hash.clone(), entry);

        // Update the statistics.
        let mut stats = self.stats.write();
        stats.entry_count = cache.len();
        stats.current_memory = *self.current_memory.read();

        Some(cte_hash)
    }

    /// Retrieve data from the cache.
    pub fn get(&self, cte_definition: &str) -> Option<Arc<Vec<u8>>> {
        let config = self.config.read();

        if !config.enabled {
            return None;
        }

        drop(config);

        let cte_hash = Self::compute_hash(cte_definition);
        let mut cache = self.cache.write();

        if let Some(entry) = cache.get_mut(&cte_hash) {
            // Check whether it has expired.
            let config = self.config.read();
            if entry.age().as_secs() > config.entry_ttl_seconds {
                // Expired; removed.
                let size = entry.data_size;
                cache.remove(&cte_hash);
                *self.current_memory.write() -= size;

                let mut stats = self.stats.write();
                stats.miss_count += 1;
                stats.entry_count = cache.len();
                stats.current_memory = *self.current_memory.read();
                return None;
            }

            // Record visits
            entry.record_access();

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
        self.cache.read().contains_key(&cte_hash)
    }

    /// Invalidate the cache entry
    pub fn invalidate(&self, cte_definition: &str) -> bool {
        let cte_hash = Self::compute_hash(cte_definition);
        let mut cache = self.cache.write();

        if let Some(entry) = cache.remove(&cte_hash) {
            *self.current_memory.write() -= entry.data_size;

            let mut stats = self.stats.write();
            stats.entry_count = cache.len();
            stats.current_memory = *self.current_memory.read();
            true
        } else {
            false
        }
    }

    /// Clear all caches.
    pub fn clear(&self) {
        let mut cache = self.cache.write();
        cache.clear();
        *self.current_memory.write() = 0;

        let mut stats = self.stats.write();
        stats.entry_count = 0;
        stats.current_memory = 0;
    }

    /// Obtain statistical information
    pub fn get_stats(&self) -> CteCacheStats {
        let mut stats = self.stats.read().clone();
        stats.entry_count = self.cache.read().len();
        stats.current_memory = *self.current_memory.read();
        stats
    }

    /// 重置统计
    pub fn reset_stats(&self) {
        self.stats.write().reset();
    }

    /// Get the current memory usage
    pub fn current_memory(&self) -> usize {
        *self.current_memory.read()
    }

    /// Obtain the number of cached entries
    pub fn entry_count(&self) -> usize {
        self.cache.read().len()
    }

    /// Implementation of phase-out, if required
    fn evict_if_needed(&self) {
        let config = self.config.read();
        let max_size = config.max_size;
        drop(config);

        let mut current = *self.current_memory.read();
        let mut evicted = 0u64;

        while current > max_size && current > 0 {
            // Find the entry with the lowest score and eliminate it.
            let to_evict = {
                let cache = self.cache.read();
                cache
                    .iter()
                    .min_by(|a, b| {
                        a.1.cache_score()
                            .partial_cmp(&b.1.cache_score())
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .map(|(k, _)| k.clone())
            };

            if let Some(key) = to_evict {
                let mut cache = self.cache.write();
                if let Some(entry) = cache.remove(&key) {
                    current -= entry.data_size;
                    evicted += 1;
                }
            } else {
                break;
            }
        }

        if evicted > 0 {
            *self.current_memory.write() = current;

            let mut stats = self.stats.write();
            stats.evicted_count += evicted;
            stats.entry_count = self.cache.read().len();
            stats.current_memory = current;
        }
    }

    /// Clearance of obsolete entries
    pub fn cleanup_expired(&self) -> usize {
        let config = self.config.read();
        let ttl = config.entry_ttl_seconds;
        drop(config);

        let mut cache = self.cache.write();
        let now = Instant::now();
        let to_remove: Vec<String> = cache
            .iter()
            .filter(|(_, entry)| now.duration_since(entry.created_at).as_secs() > ttl)
            .map(|(k, _)| k.clone())
            .collect();

        let mut freed_memory = 0usize;
        for key in &to_remove {
            if let Some(entry) = cache.remove(key) {
                freed_memory += entry.data_size;
            }
        }

        if freed_memory > 0 {
            *self.current_memory.write() -= freed_memory;

            let mut stats = self.stats.write();
            stats.entry_count = cache.len();
            stats.current_memory = *self.current_memory.read();
        }

        to_remove.len()
    }
}

impl Default for CteCacheManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for CteCacheManager {
    fn clone(&self) -> Self {
        Self {
            cache: RwLock::new(self.cache.read().clone()),
            config: RwLock::new(self.config.read().clone()),
            stats: RwLock::new(self.stats.read().clone()),
            current_memory: RwLock::new(*self.current_memory.read()),
        }
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
    /// 估计重用概率
    pub reuse_probability: f64,
    /// Estimated caching gains
    pub estimated_benefit: f64,
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
                reason: "缓存已禁用".to_string(),
                reuse_probability: 0.0,
                estimated_benefit: 0.0,
            };
        }

        let reuse_prob = self.cache_manager.predict_reuse_probability(cte_definition);

        if reuse_prob < self.min_reuse_probability {
            return CteCacheDecision {
                should_cache: false,
                reason: format!("重用概率过低: {:.2}", reuse_prob),
                reuse_probability: reuse_prob,
                estimated_benefit: 0.0,
            };
        }

        // Estimated Cache Benefit = Probability of Reuse * Computation Cost - Cache Overhead
        let cache_overhead = estimated_rows as f64 * 0.001; // 假设每行缓存开销0.001ms
        let estimated_benefit = reuse_prob * compute_cost - cache_overhead;

        if estimated_benefit < self.min_benefit {
            return CteCacheDecision {
                should_cache: false,
                reason: format!("估计收益过低: {:.2}", estimated_benefit),
                reuse_probability: reuse_prob,
                estimated_benefit,
            };
        }

        CteCacheDecision {
            should_cache: true,
            reason: "收益分析通过".to_string(),
            reuse_probability: reuse_prob,
            estimated_benefit,
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
        assert_eq!(stats.entry_count, 1);
    }

    #[test]
    fn test_cte_cache_eviction() {
        let config = CteCacheConfig {
            max_size: 100, // Very small cache
            max_entry_size: 50,
            min_row_count: 1,
            max_row_count: 1000,
            entry_ttl_seconds: 3600,
            enabled: true,
        };

        let manager = CteCacheManager::with_config(config);

        // Entering multiple entries triggers the elimination process.
        let data1 = vec![1u8; 40]; // 40 bytes
        let data2 = vec![2u8; 40]; // 40 bytes
        let data3 = vec![3u8; 40]; // 40 bytes
        let data4 = vec![4u8; 40]; // 40 bytes

        manager.put("query1", data1, 10);
        manager.put("query2", data2, 10);

        // Visit query1 to boost its score
        manager.get("query1");

        // Query3 should be stored, while query2 should be eliminated.
        manager.put("query3", data3, 10);

        // Stored in query4 to ensure triggering of elimination
        manager.put("query4", data4, 10);

        let stats = manager.get_stats();
        assert!(stats.evicted_count >= 1);
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
}
