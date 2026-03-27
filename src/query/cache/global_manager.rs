//! Global Cache Manager Module
//!
//! Unified management of all caches, coordinating memory allocation,
//! providing unified monitoring interfaces.
//!
//! # Design Goals
//!
//! 1. Global memory budget management
//! 2. Unified monitoring and statistics
//! 3. Intelligent eviction policies
//! 4. Emergency eviction when memory pressure is high

use parking_lot::RwLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use super::cte_cache::{CteCacheConfig, CteCacheManager, CteCacheStats};
use super::plan_cache::{PlanCacheConfig, PlanCacheStats, QueryPlanCache};

/// Cache allocation configuration
#[derive(Debug, Clone)]
pub struct CacheAllocations {
    /// Plan cache allocation ratio (0.0 - 1.0)
    pub plan_cache_ratio: f64,
    /// CTE cache allocation ratio (0.0 - 1.0)
    pub cte_cache_ratio: f64,
    /// Reserve ratio for burst allocations (0.0 - 1.0)
    pub reserve_ratio: f64,
}

impl Default for CacheAllocations {
    fn default() -> Self {
        Self {
            plan_cache_ratio: 0.4,
            cte_cache_ratio: 0.4,
            reserve_ratio: 0.2,
        }
    }
}

impl CacheAllocations {
    /// Validate the allocation ratios
    pub fn validate(&self) -> bool {
        let total = self.plan_cache_ratio + self.cte_cache_ratio + self.reserve_ratio;
        (total - 1.0).abs() < 0.01
            && self.plan_cache_ratio > 0.0
            && self.cte_cache_ratio > 0.0
            && self.reserve_ratio >= 0.0
    }

    /// Calculate plan cache budget from total budget
    pub fn plan_budget(&self, total_budget: usize) -> usize {
        (total_budget as f64 * self.plan_cache_ratio) as usize
    }

    /// Calculate CTE cache budget from total budget
    pub fn cte_budget(&self, total_budget: usize) -> usize {
        (total_budget as f64 * self.cte_cache_ratio) as usize
    }
}

/// Global cache statistics
#[derive(Debug, Clone, Default)]
pub struct GlobalCacheStats {
    /// Total hit count
    pub total_hits: u64,
    /// Total miss count
    pub total_misses: u64,
    /// Total memory usage (bytes)
    pub total_memory: usize,
    /// Total memory budget (bytes)
    pub total_budget: usize,
    /// Eviction count
    pub evictions: u64,
    /// Plan cache statistics
    pub plan_cache_stats: PlanCacheStats,
    /// CTE cache statistics
    pub cte_cache_stats: CteCacheStats,
}

impl GlobalCacheStats {
    /// Calculate global hit rate
    pub fn hit_rate(&self) -> f64 {
        let total = self.total_hits + self.total_misses;
        if total == 0 {
            0.0
        } else {
            self.total_hits as f64 / total as f64
        }
    }

    /// Calculate memory usage ratio
    pub fn memory_usage_ratio(&self) -> f64 {
        if self.total_budget == 0 {
            0.0
        } else {
            self.total_memory as f64 / self.total_budget as f64
        }
    }

    /// Format statistics for display
    pub fn format(&self) -> String {
        format!(
            "Global Cache Statistics:\n\
             - Hit Rate: {:.2}%\n\
             - Memory Usage: {:.2} MB / {:.2} MB ({:.1}%)\n\
             - Evictions: {}\n\
             - Plan Cache: {} entries, {:.2}% hit rate\n\
             - CTE Cache: {} entries, {:.2}% hit rate",
            self.hit_rate() * 100.0,
            self.total_memory as f64 / 1024.0 / 1024.0,
            self.total_budget as f64 / 1024.0 / 1024.0,
            self.memory_usage_ratio() * 100.0,
            self.evictions,
            self.plan_cache_stats
                .current_entries
                .load(Ordering::Relaxed),
            self.plan_cache_stats.hit_rate() * 100.0,
            self.cte_cache_stats.entry_count,
            self.cte_cache_stats.hit_rate() * 100.0
        )
    }
}

/// Global cache manager
///
/// Unified management of all caches, coordinating memory allocation,
/// providing unified monitoring interfaces.
pub struct GlobalCacheManager {
    /// Total memory budget
    total_budget: usize,
    /// Cache allocation ratios
    allocations: CacheAllocations,
    /// Plan cache
    plan_cache: Arc<QueryPlanCache>,
    /// CTE cache
    cte_cache: Arc<CteCacheManager>,
    /// Current memory usage
    current_usage: AtomicUsize,
    /// Statistics
    stats: RwLock<GlobalCacheStats>,
}

impl std::fmt::Debug for GlobalCacheManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GlobalCacheManager")
            .field("total_budget", &self.total_budget)
            .field("allocations", &self.allocations)
            .field("current_usage", &self.current_usage.load(Ordering::Relaxed))
            .field("stats", &self.stats.read())
            .finish()
    }
}

impl GlobalCacheManager {
    /// Create a new global cache manager
    pub fn new(total_budget: usize, allocations: CacheAllocations) -> Self {
        if !allocations.validate() {
            panic!("Invalid cache allocations: ratios must sum to 1.0 and be non-negative");
        }

        let plan_budget = allocations.plan_budget(total_budget);
        let cte_budget = allocations.cte_budget(total_budget);

        let plan_config = PlanCacheConfig {
            max_entries: 1000,
            memory_budget: plan_budget,
            max_weight: None,
            enable_parameterized: true,
            ttl_config: super::plan_cache::TtlConfig {
                base_ttl_seconds: 3600,
                adaptive: true,
                min_ttl_seconds: 300,
                max_ttl_seconds: 86400,
            },
            priority_config: super::plan_cache::PriorityConfig {
                enable_priority: true,
                track_execution_time: true,
            },
        };

        let cte_config = CteCacheConfig {
            max_size: cte_budget,
            max_entries: Some(10000),
            max_entry_size: 10 * 1024 * 1024,
            min_row_count: 100,
            max_row_count: 100_000,
            entry_ttl_seconds: 3600,
            enabled: true,
            adaptive: true,
            enable_priority: true,
        };

        let plan_cache = Arc::new(QueryPlanCache::new(plan_config));
        let cte_cache = Arc::new(CteCacheManager::with_config(cte_config));

        Self {
            total_budget,
            allocations,
            plan_cache,
            cte_cache,
            current_usage: AtomicUsize::new(0),
            stats: RwLock::new(GlobalCacheStats {
                total_budget,
                ..Default::default()
            }),
        }
    }

    /// Get the plan cache
    pub fn plan_cache(&self) -> Arc<QueryPlanCache> {
        self.plan_cache.clone()
    }

    /// Get the CTE cache
    pub fn cte_cache(&self) -> Arc<CteCacheManager> {
        self.cte_cache.clone()
    }

    /// Update current memory usage
    pub fn update_memory_usage(&self) {
        let plan_stats = self.plan_cache.stats();
        let cte_stats = self.cte_cache.get_stats();

        let total_memory = plan_stats.estimated_memory_bytes() + cte_stats.current_memory;
        self.current_usage.store(total_memory, Ordering::Relaxed);

        let mut stats = self.stats.write();
        stats.total_memory = total_memory;
        stats.plan_cache_stats = plan_stats.clone();
        stats.cte_cache_stats = cte_stats.clone();
        stats.total_hits = plan_stats.hits.load(Ordering::Relaxed) + cte_stats.hit_count;
        stats.total_misses = plan_stats.misses.load(Ordering::Relaxed) + cte_stats.miss_count;
        stats.evictions = plan_stats.evictions.load(Ordering::Relaxed) + cte_stats.evicted_count;
    }

    /// Get statistics
    pub fn stats(&self) -> GlobalCacheStats {
        self.update_memory_usage();
        self.stats.read().clone()
    }

    /// Clear all caches
    pub fn clear_all(&self) {
        self.plan_cache.clear();
        self.cte_cache.clear();
        self.current_usage.store(0, Ordering::Relaxed);

        let mut stats = self.stats.write();
        stats.total_memory = 0;
        stats.plan_cache_stats = self.plan_cache.stats();
        stats.cte_cache_stats = self.cte_cache.get_stats();
    }

    /// Get total memory usage
    pub fn total_memory_usage(&self) -> usize {
        self.current_usage.load(Ordering::Relaxed)
    }

    /// Get total budget
    pub fn total_budget(&self) -> usize {
        self.total_budget
    }

    /// Get cache allocations
    pub fn allocations(&self) -> CacheAllocations {
        self.allocations.clone()
    }
}

impl GlobalCacheManager {
    /// Minimal memory configuration - for embedded environments
    pub fn minimal() -> Self {
        Self::new(
            32 * 1024 * 1024,
            CacheAllocations {
                plan_cache_ratio: 0.5,
                cte_cache_ratio: 0.3,
                reserve_ratio: 0.2,
            },
        )
    }

    /// Balanced configuration - default
    pub fn balanced() -> Self {
        Self::new(128 * 1024 * 1024, CacheAllocations::default())
    }

    /// High performance configuration - for server environments
    pub fn high_performance() -> Self {
        Self::new(
            512 * 1024 * 1024,
            CacheAllocations {
                plan_cache_ratio: 0.35,
                cte_cache_ratio: 0.45,
                reserve_ratio: 0.2,
            },
        )
    }
}

impl Default for GlobalCacheManager {
    fn default() -> Self {
        Self::balanced()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU64;

    #[test]
    fn test_allocations_default() {
        let alloc = CacheAllocations::default();
        assert!(alloc.validate());
        assert_eq!(alloc.plan_cache_ratio, 0.4);
        assert_eq!(alloc.cte_cache_ratio, 0.4);
        assert_eq!(alloc.reserve_ratio, 0.2);
    }

    #[test]
    fn test_allocations_validation() {
        let valid = CacheAllocations::default();
        assert!(valid.validate());

        let invalid = CacheAllocations {
            plan_cache_ratio: 0.5,
            cte_cache_ratio: 0.6,
            reserve_ratio: 0.2,
        };
        assert!(!invalid.validate());
    }

    #[test]
    fn test_allocations_budget_calculation() {
        let alloc = CacheAllocations::default();
        let total = 100 * 1024 * 1024;

        let plan_budget = alloc.plan_budget(total);
        let cte_budget = alloc.cte_budget(total);

        assert_eq!(plan_budget, 40 * 1024 * 1024);
        assert_eq!(cte_budget, 40 * 1024 * 1024);
    }

    #[test]
    fn test_global_manager_creation() {
        let manager = GlobalCacheManager::balanced();
        assert_eq!(manager.total_budget(), 128 * 1024 * 1024);
        assert_eq!(manager.total_memory_usage(), 0);
    }

    #[test]
    fn test_global_manager_minimal() {
        let manager = GlobalCacheManager::minimal();
        assert_eq!(manager.total_budget(), 32 * 1024 * 1024);
    }

    #[test]
    fn test_global_manager_high_performance() {
        let manager = GlobalCacheManager::high_performance();
        assert_eq!(manager.total_budget(), 512 * 1024 * 1024);
    }

    #[test]
    fn test_global_stats_format() {
        let stats = GlobalCacheStats {
            total_hits: 850,
            total_misses: 150,
            total_memory: 50 * 1024 * 1024,
            total_budget: 100 * 1024 * 1024,
            evictions: 10,
            plan_cache_stats: PlanCacheStats {
                current_entries: Arc::new(AtomicUsize::new(100)),
                hits: Arc::new(AtomicU64::new(600)),
                misses: Arc::new(AtomicU64::new(100)),
                ..Default::default()
            },
            cte_cache_stats: CteCacheStats {
                entry_count: 50,
                hit_count: 250,
                miss_count: 50,
                current_memory: 20 * 1024 * 1024,
                max_memory: 40 * 1024 * 1024,
                ..Default::default()
            },
            ..Default::default()
        };

        let formatted = stats.format();
        assert!(formatted.contains("Hit Rate: 85.00%"));
        assert!(formatted.contains("Memory Usage: 50.00 MB / 100.00 MB"));
    }

    #[test]
    fn test_global_stats_hit_rate() {
        let stats = GlobalCacheStats {
            total_hits: 80,
            total_misses: 20,
            ..Default::default()
        };

        assert_eq!(stats.hit_rate(), 0.8);
    }

    #[test]
    fn test_global_stats_memory_usage_ratio() {
        let stats = GlobalCacheStats {
            total_memory: 50 * 1024 * 1024,
            total_budget: 100 * 1024 * 1024,
            ..Default::default()
        };

        assert_eq!(stats.memory_usage_ratio(), 0.5);
    }
}
