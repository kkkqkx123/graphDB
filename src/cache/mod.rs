//! 全局缓存模块
//!
//! 提供统一的缓存架构和实现，支持多种缓存策略和解析器特化优化

pub mod config;
pub mod cache_impl;
pub mod manager;
pub mod parser_cache;
pub mod traits;

// 重新导出主要类型
pub use cache_impl::*;
pub use config::*;
pub use manager::{CacheManager, CachePolicy, CacheStrategy, CacheStats, CacheBuilder, global_cache_manager, init_global_cache_manager};
pub use parser_cache::*;
pub use traits::*;

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
        let cache = CacheIntegrationFactory::create_testing_integration();
        assert_eq!(cache.config().keyword_cache_capacity, 100);
    }
}
