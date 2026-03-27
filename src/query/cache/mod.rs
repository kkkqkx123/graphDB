//! Query Cache Module
//!
//! Provide a unified cache management function, including:
//! Query plan cache (Prepared Statement)
//! CTE result caching
//! Other cache types that may be added in the future:
//!
//! # Design Goals
//!
//! Centralized management of all caches facilitates configuration and monitoring.
//! 2. Unified memory budget management
//! 3. Shared caching strategies (LRU, TTL, etc.)
//! 4. Unified collection of statistics and indicators

use std::sync::atomic::Ordering;

// Submodule
pub mod cte_cache;
pub mod global_manager;
pub mod plan_cache;
pub mod warmup;

// Reexport the plan cache type.
pub use plan_cache::{
    CachePriority, CachedPlan, ParamPosition, ParameterizedQueryHandler, PlanCacheConfig,
    PlanCacheKey, PlanCacheStats, QueryPlanCache,
};

// Re-export the CTE cache type
pub use cte_cache::{
    CteCacheConfig, CteCacheDecision, CteCacheDecisionMaker, CteCacheEntry, CteCacheManager,
    CteCacheStats,
};

// Re-export the global cache manager type
pub use global_manager::{CacheAllocations, GlobalCacheManager, GlobalCacheStats};

// Re-export the warmup module type
pub use warmup::{CacheWarmer, QueryStats, WarmupConfig, WarmupError, WarmupResult};

use std::sync::Arc;

/// Unified Cache Manager
///
/// Centralized management of all types of caches, with a unified configuration and monitoring interface available.
#[derive(Debug, Clone)]
pub struct CacheManager {
    /// Query plan cache
    plan_cache: Arc<QueryPlanCache>,
    /// CTE result caching
    cte_cache: Arc<CteCacheManager>,
    /// Global cache manager (optional)
    global_manager: Option<Arc<GlobalCacheManager>>,
}

impl CacheManager {
    /// Create a new cache manager.
    pub fn new() -> Self {
        Self {
            plan_cache: Arc::new(QueryPlanCache::default()),
            cte_cache: Arc::new(CteCacheManager::default()),
            global_manager: None,
        }
    }

    /// Create using the configuration.
    pub fn with_config(plan_config: PlanCacheConfig, cte_config: CteCacheConfig) -> Self {
        Self {
            plan_cache: Arc::new(QueryPlanCache::new(plan_config)),
            cte_cache: Arc::new(CteCacheManager::with_config(cte_config)),
            global_manager: None,
        }
    }

    /// Create with global cache manager
    pub fn with_global_manager(global_manager: Arc<GlobalCacheManager>) -> Self {
        let plan_cache = global_manager.plan_cache();
        let cte_cache = global_manager.cte_cache();

        Self {
            plan_cache,
            cte_cache,
            global_manager: Some(global_manager),
        }
    }

    /// Obtain the query plan cache
    pub fn plan_cache(&self) -> Arc<QueryPlanCache> {
        self.plan_cache.clone()
    }

    /// Obtaining the CTE cache
    pub fn cte_cache(&self) -> Arc<CteCacheManager> {
        self.cte_cache.clone()
    }

    /// Get the global cache manager if available
    pub fn global_manager(&self) -> Option<Arc<GlobalCacheManager>> {
        self.global_manager.clone()
    }

    /// Get the total memory usage (in bytes).
    pub fn total_memory_usage(&self) -> usize {
        if let Some(global) = &self.global_manager {
            global.total_memory_usage()
        } else {
            let plan_stats = self.plan_cache.stats();
            let cte_stats = self.cte_cache.get_stats();

            plan_stats.estimated_memory_bytes() + cte_stats.current_memory
        }
    }

    /// Clear all caches.
    pub fn clear_all(&self) {
        self.plan_cache.clear();
        self.cte_cache.clear();

        if let Some(global) = &self.global_manager {
            global.clear_all();
        }
    }

    /// Obtain a summary of cache statistics.
    pub fn stats_summary(&self) -> CacheStatsSummary {
        let plan_stats = self.plan_cache.stats();
        let cte_stats = self.cte_cache.get_stats();

        CacheStatsSummary {
            plan_cache_entries: plan_stats.current_entries.load(Ordering::Relaxed),
            plan_cache_hit_rate: plan_stats.hit_rate(),
            cte_cache_entries: cte_stats.entry_count,
            cte_cache_hit_rate: cte_stats.hit_rate(),
            total_memory_bytes: plan_stats.estimated_memory_bytes() + cte_stats.current_memory,
        }
    }

    /// Update memory usage statistics
    pub fn update_memory_usage(&self) {
        if let Some(global) = &self.global_manager {
            global.update_memory_usage();
        }
    }
}

impl Default for CacheManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache statistics summary
#[derive(Debug, Clone)]
pub struct CacheStatsSummary {
    /// Number of planned cache entries
    pub plan_cache_entries: usize,
    /// Plan Cache Hit Rate
    pub plan_cache_hit_rate: f64,
    /// Number of CTE (Common Table Expression) cache entries
    pub cte_cache_entries: usize,
    /// CTE Cache Hit Rate
    pub cte_cache_hit_rate: f64,
    /// Total memory usage (in bytes)
    pub total_memory_bytes: usize,
}

impl CacheStatsSummary {
    /// Of course! Please provide the text you would like to have translated.
    pub fn format(&self) -> String {
        format!(
            "Cache Statistics:\n\
             - Plan Cache: {} entries, {:.2}% hit rate\n\
             - CTE Cache: {} entries, {:.2}% hit rate\n\
             - Total Memory: {:.2} MB",
            self.plan_cache_entries,
            self.plan_cache_hit_rate * 100.0,
            self.cte_cache_entries,
            self.cte_cache_hit_rate * 100.0,
            self.total_memory_bytes as f64 / 1024.0 / 1024.0
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_manager_creation() {
        let manager = CacheManager::new();
        assert_eq!(manager.total_memory_usage(), 0);
    }

    #[test]
    fn test_cache_manager_with_config() {
        let plan_config = PlanCacheConfig {
            max_entries: 500,
            memory_budget: 25 * 1024 * 1024,
            max_weight: None,
            enable_parameterized: true,
            ttl_config: plan_cache::TtlConfig {
                base_ttl_seconds: 1800,
                adaptive: true,
                min_ttl_seconds: 300,
                max_ttl_seconds: 86400,
            },
            priority_config: plan_cache::PriorityConfig {
                enable_priority: true,
                track_execution_time: true,
            },
        };

        let cte_config = CteCacheConfig {
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

        let manager = CacheManager::with_config(plan_config, cte_config);
        assert_eq!(manager.total_memory_usage(), 0);
    }

    #[test]
    fn test_cache_stats_summary() {
        let summary = CacheStatsSummary {
            plan_cache_entries: 100,
            plan_cache_hit_rate: 0.85,
            cte_cache_entries: 50,
            cte_cache_hit_rate: 0.75,
            total_memory_bytes: 1024 * 1024 * 10, // 10MB
        };

        let formatted = summary.format();
        assert!(formatted.contains("Plan Cache: 100 entries"));
        assert!(formatted.contains("85.00% hit rate"));
        assert!(formatted.contains("10.00 MB"));
    }
}
