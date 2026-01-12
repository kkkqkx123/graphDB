//! 全局缓存管理器
//!
//! 负责管理全局缓存管理器实例，提供安全的全局访问接口

use super::registry::CacheRegistry;
use super::stats_collector::CacheStatsCollector;
use crate::cache::CacheConfig;
use std::sync::{Arc, OnceLock};

/// 全局缓存管理器
pub struct GlobalCacheManager {
    registry: CacheRegistry,
    stats_collector: CacheStatsCollector,
    config: CacheConfig,
}

impl GlobalCacheManager {
    /// 创建新的全局缓存管理器
    pub fn new(config: CacheConfig) -> Self {
        Self {
            registry: CacheRegistry::new(),
            stats_collector: CacheStatsCollector::new(),
            config,
        }
    }

    /// 获取缓存注册表
    pub fn registry(&self) -> &CacheRegistry {
        &self.registry
    }

    /// 获取统计收集器
    pub fn stats_collector(&self) -> &CacheStatsCollector {
        &self.stats_collector
    }

    /// 获取配置
    pub fn config(&self) -> &CacheConfig {
        &self.config
    }

    /// 更新配置
    pub fn update_config(&mut self, config: CacheConfig) -> Result<(), String> {
        config.validate()?;
        self.config = config;
        Ok(())
    }

    /// 获取缓存数量
    pub fn cache_count(&self) -> usize {
        self.registry.cache_count()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.registry.cache_count() == 0
    }

    /// 清空所有缓存注册信息
    pub fn clear_all(&self) {
        self.registry.clear_all();
        self.stats_collector.record_cache_count(0);
    }

    /// 获取统计信息快照
    pub fn stats_snapshot(&self) -> super::stats_collector::CacheStats {
        self.stats_collector.snapshot()
    }

    /// 获取命中率
    pub fn hit_rate(&self) -> f64 {
        self.stats_collector.hit_rate()
    }

    /// 获取命中率百分比
    pub fn hit_rate_percentage(&self) -> f64 {
        self.stats_collector.hit_rate_percentage()
    }
}

impl std::fmt::Debug for GlobalCacheManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GlobalCacheManager")
            .field("cache_count", &self.cache_count())
            .field("config", &self.config)
            .field("hit_rate", &self.hit_rate())
            .finish()
    }
}

/// 全局缓存管理器实例
static GLOBAL_CACHE_MANAGER: once_cell::sync::Lazy<Arc<GlobalCacheManager>> =
    once_cell::sync::Lazy::new(|| Arc::new(GlobalCacheManager::new(CacheConfig::default())));

/// 全局缓存管理器实例（可初始化版本）
static GLOBAL_CACHE_MANAGER_MUT: OnceLock<Arc<GlobalCacheManager>> = OnceLock::new();

/// 获取全局缓存管理器
pub fn global_cache_manager() -> Arc<GlobalCacheManager> {
    GLOBAL_CACHE_MANAGER_MUT
        .get()
        .cloned()
        .unwrap_or_else(|| GLOBAL_CACHE_MANAGER.clone())
}

/// 初始化全局缓存管理器
pub fn init_global_cache_manager(config: CacheConfig) -> Result<(), String> {
    config.validate()?;
    let manager = Arc::new(GlobalCacheManager::new(config));
    GLOBAL_CACHE_MANAGER_MUT
        .set(manager)
        .map_err(|_| "Global cache manager already initialized".to_string())
}

/// 重新初始化全局缓存管理器（仅在测试中使用）
#[cfg(test)]
pub fn reset_global_cache_manager() {
    // OnceLock不支持重置，所以这个函数现在只能用于测试场景
    // 在实际使用中，不应该重置全局状态
}

/// 检查全局缓存管理器是否已初始化
pub fn is_global_cache_manager_initialized() -> bool {
    GLOBAL_CACHE_MANAGER_MUT.get().is_some()
}

/// 获取全局缓存注册表
pub fn global_cache_registry() -> CacheRegistry {
    let manager = global_cache_manager();
    manager.registry().clone()
}

/// 获取全局统计收集器
pub fn global_stats_collector() -> CacheStatsCollector {
    let manager = global_cache_manager();
    manager.stats_collector().clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::CacheStrategy;

    #[test]
    fn test_global_cache_manager_creation() {
        let config = CacheConfig::default();
        let manager = GlobalCacheManager::new(config);

        assert!(manager.is_empty());
        assert_eq!(manager.cache_count(), 0);
        assert_eq!(manager.hit_rate(), 0.0);
    }

    #[test]
    fn test_global_cache_manager_registry() {
        let manager = GlobalCacheManager::new(CacheConfig::default());

        let registry = manager.registry();
        assert_eq!(registry.cache_count(), 0);

        registry
            .register_cache("test", "LRU", 100, CacheStrategy::LRU)
            .expect("Registration should succeed");
        assert_eq!(manager.cache_count(), 1);
        assert!(!manager.is_empty());
    }

    #[test]
    fn test_global_cache_manager_stats() {
        let manager = GlobalCacheManager::new(CacheConfig::default());

        let stats_collector = manager.stats_collector();
        stats_collector.record_hit();
        stats_collector.record_miss();

        assert_eq!(manager.hit_rate(), 0.5);
        assert_eq!(manager.hit_rate_percentage(), 50.0);
    }

    #[test]
    fn test_global_cache_manager_config() {
        let config = CacheConfig::development();
        let manager = GlobalCacheManager::new(config.clone());

        assert_eq!(manager.config().default_capacity, config.default_capacity);
        assert_eq!(manager.config().default_policy, config.default_policy);
    }

    #[test]
    fn test_global_cache_manager_update_config() {
        let mut manager = GlobalCacheManager::new(CacheConfig::default());

        let new_config = CacheConfig::production();
        manager
            .update_config(new_config.clone())
            .expect("Config update should succeed");

        assert_eq!(
            manager.config().default_capacity,
            new_config.default_capacity
        );
    }

    #[test]
    fn test_global_cache_manager_clear_all() {
        let manager = GlobalCacheManager::new(CacheConfig::default());

        let registry = manager.registry();
        registry
            .register_cache("test1", "LRU", 100, CacheStrategy::LRU)
            .expect("Registration should succeed");
        registry
            .register_cache("test2", "LFU", 200, CacheStrategy::LFU)
            .expect("Registration should succeed");

        assert_eq!(manager.cache_count(), 2);

        manager.clear_all();
        assert_eq!(manager.cache_count(), 0);
        assert!(manager.is_empty());
    }

    #[test]
    fn test_global_cache_manager_debug() {
        let manager = GlobalCacheManager::new(CacheConfig::default());

        let debug_output = format!("{:?}", manager);
        assert!(debug_output.contains("GlobalCacheManager"));
        assert!(debug_output.contains("cache_count"));
        assert!(debug_output.contains("hit_rate"));
    }

    #[test]
    fn test_global_cache_manager_functions() {
        // 测试未初始化状态
        assert!(!is_global_cache_manager_initialized());

        // 初始化全局缓存管理器
        let config = CacheConfig::development();
        init_global_cache_manager(config)
            .expect("Global cache manager initialization should succeed");
        assert!(is_global_cache_manager_initialized());

        // 尝试再次初始化应该失败
        let config2 = CacheConfig::production();
        let result = init_global_cache_manager(config2);
        assert!(result.is_err());

        // 获取全局管理器
        let manager = global_cache_manager();
        assert_eq!(manager.config().default_capacity, 500);

        // 获取全局注册表
        let registry = global_cache_registry();
        assert_eq!(registry.cache_count(), 0);

        // 获取全局统计收集器
        let stats_collector = global_stats_collector();
        assert!(stats_collector.is_empty());
    }

    #[test]
    fn test_global_cache_manager_stats_snapshot() {
        let manager = GlobalCacheManager::new(CacheConfig::default());

        let stats_collector = manager.stats_collector();
        stats_collector.record_hit();
        stats_collector.record_miss();
        stats_collector.record_eviction();

        let snapshot = manager.stats_snapshot();
        assert_eq!(snapshot.total_hits, 1);
        assert_eq!(snapshot.total_misses, 1);
        assert_eq!(snapshot.total_evictions, 1);
        assert_eq!(snapshot.hit_rate(), 0.5);
    }
}
