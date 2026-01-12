//! 全局缓存模块
//!
//! 提供统一的缓存架构和实现，支持多种缓存策略和解析器特化优化

pub mod cache_impl;
pub mod config;
pub mod factory;
pub mod global_manager;
pub mod manager;
pub mod parser_cache;
pub mod registry;
pub mod stats_collector;
pub mod stats_marker;
pub mod traits;

// 重新导出主要类型
pub use cache_impl::*;
pub use config::*;
pub use factory::CacheFactory;
pub use global_manager::{
    global_cache_manager, global_cache_registry, global_stats_collector, init_global_cache_manager,
    is_global_cache_manager_initialized, GlobalCacheManager,
};
pub use manager::{CacheBuilder, CacheManager, CacheStrategy};
pub use parser_cache::*;
pub use registry::{CacheRegistry, CacheRegistryInfo};
pub use stats_collector::{CacheStats, CacheStatsCollector};
pub use stats_marker::{StatsDisabled, StatsEnabled, StatsMode};
pub use traits::*;

#[cfg(test)]
pub use global_manager::reset_global_cache_manager;

/// 缓存模块版本
pub const CACHE_VERSION: &str = "1.0.0";

/// 初始化缓存模块
pub fn init_cache(config: CacheConfig) -> Result<(), String> {
    config.validate()?;
    init_global_cache_manager(config)
}

/// 获取默认的解析器缓存
pub fn create_default_parser_cache() -> ParserCache {
    ParserCache::new(CacheConfig::default())
}

/// 获取开发环境的解析器缓存
pub fn create_development_parser_cache() -> ParserCache {
    ParserCache::new(CacheConfig::development())
}

/// 获取生产环境的解析器缓存
pub fn create_production_parser_cache() -> ParserCache {
    ParserCache::new(CacheConfig::production())
}

/// 获取测试环境的解析器缓存
pub fn create_testing_parser_cache() -> ParserCache {
    ParserCache::new(CacheConfig::testing())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_version() {
        assert_eq!(CACHE_VERSION, "1.0.0");
    }

    #[test]
    fn test_init_cache() {
        let config = CacheConfig::default();
        assert!(init_cache(config).is_ok());
    }

    #[test]
    fn test_create_parser_caches() {
        let default_cache = create_default_parser_cache();
        let dev_cache = create_development_parser_cache();
        let prod_cache = create_production_parser_cache();
        let test_cache = create_testing_parser_cache();

        // 验证不同环境的配置差异
        assert_eq!(default_cache.config().keyword_cache_capacity, 1000);
        assert_eq!(dev_cache.config().keyword_cache_capacity, 500);
        assert_eq!(prod_cache.config().keyword_cache_capacity, 2000);
        assert_eq!(test_cache.config().keyword_cache_capacity, 100);
    }

    #[test]
    fn test_cache_config_validation() {
        let mut config = CacheConfig::default();
        assert!(config.validate().is_ok());

        config.default_capacity = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_integration_factory() {
        let cache = create_testing_parser_cache();
        assert_eq!(cache.config().keyword_cache_capacity, 100);
    }
}
