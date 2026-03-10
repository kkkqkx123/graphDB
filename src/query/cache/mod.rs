//! 查询缓存模块
//!
//! 提供统一的缓存管理功能，包括：
//! - 查询计划缓存（Prepared Statement）
//! - CTE结果缓存
//! - 未来可能扩展的其他缓存类型
//!
//! # 设计目标
//!
//! 1. 集中管理所有缓存，便于配置和监控
//! 2. 统一的内存预算管理
//! 3. 共享缓存策略（LRU、TTL等）
//! 4. 统一的统计和指标收集

// 子模块
pub mod cte_cache;
pub mod plan_cache;

// 重新导出计划缓存类型
pub use plan_cache::{
    CachedPlan, ParamPosition, PlanCacheConfig, PlanCacheKey, PlanCacheStats,
    ParameterizedQueryHandler, QueryPlanCache,
};

// 重新导出CTE缓存类型
pub use cte_cache::{
    CteCacheConfig, CteCacheDecision, CteCacheDecisionMaker, CteCacheEntry, CteCacheManager,
    CteCacheStats,
};

use std::sync::Arc;

/// 统一缓存管理器
///
/// 集中管理所有类型的缓存，提供统一的配置和监控接口
#[derive(Debug, Clone)]
pub struct CacheManager {
    /// 查询计划缓存
    plan_cache: Arc<QueryPlanCache>,
    /// CTE结果缓存
    cte_cache: Arc<CteCacheManager>,
}

impl CacheManager {
    /// 创建新的缓存管理器
    pub fn new() -> Self {
        Self {
            plan_cache: Arc::new(QueryPlanCache::default()),
            cte_cache: Arc::new(CteCacheManager::default()),
        }
    }

    /// 使用配置创建
    pub fn with_config(plan_config: PlanCacheConfig, cte_config: CteCacheConfig) -> Self {
        Self {
            plan_cache: Arc::new(QueryPlanCache::new(plan_config)),
            cte_cache: Arc::new(CteCacheManager::with_config(cte_config)),
        }
    }

    /// 获取查询计划缓存
    pub fn plan_cache(&self) -> Arc<QueryPlanCache> {
        self.plan_cache.clone()
    }

    /// 获取CTE缓存
    pub fn cte_cache(&self) -> Arc<CteCacheManager> {
        self.cte_cache.clone()
    }

    /// 获取总内存使用（字节）
    pub fn total_memory_usage(&self) -> usize {
        let plan_stats = self.plan_cache.stats();
        let cte_stats = self.cte_cache.get_stats();

        plan_stats.estimated_memory_bytes() + cte_stats.current_memory
    }

    /// 清空所有缓存
    pub fn clear_all(&self) {
        self.plan_cache.clear();
        self.cte_cache.clear();
    }

    /// 获取缓存统计摘要
    pub fn stats_summary(&self) -> CacheStatsSummary {
        let plan_stats = self.plan_cache.stats();
        let cte_stats = self.cte_cache.get_stats();

        CacheStatsSummary {
            plan_cache_entries: plan_stats.current_entries,
            plan_cache_hit_rate: plan_stats.hit_rate(),
            cte_cache_entries: cte_stats.entry_count,
            cte_cache_hit_rate: cte_stats.hit_rate(),
            total_memory_bytes: plan_stats.estimated_memory_bytes() + cte_stats.current_memory,
        }
    }
}

impl Default for CacheManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 缓存统计摘要
#[derive(Debug, Clone)]
pub struct CacheStatsSummary {
    /// 计划缓存条目数
    pub plan_cache_entries: usize,
    /// 计划缓存命中率
    pub plan_cache_hit_rate: f64,
    /// CTE缓存条目数
    pub cte_cache_entries: usize,
    /// CTE缓存命中率
    pub cte_cache_hit_rate: f64,
    /// 总内存使用（字节）
    pub total_memory_bytes: usize,
}

impl CacheStatsSummary {
    /// 格式化输出
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
            ttl_seconds: 1800,
            enable_parameterized: true,
        };

        let cte_config = CteCacheConfig {
            max_size: 32 * 1024 * 1024,
            max_entry_size: 5 * 1024 * 1024,
            min_row_count: 50,
            max_row_count: 50_000,
            entry_ttl_seconds: 1800,
            enabled: true,
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
