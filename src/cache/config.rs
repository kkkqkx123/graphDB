//! 缓存配置管理
//!
//! 提供缓存系统的配置选项和默认值

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// 全局缓存配置
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CacheConfig {
    /// 是否启用缓存
    pub enabled: bool,

    /// 默认缓存容量
    pub default_capacity: usize,

    /// 默认TTL
    pub default_ttl: Duration,

    /// 缓存策略
    pub default_policy: CachePolicy,

    /// 统计信息收集
    pub collect_stats: bool,

    /// 特化缓存配置
    pub parser_cache: ParserCacheConfig,

    /// 内存限制 (字节)
    pub memory_limit: Option<usize>,

    /// 清理间隔
    pub cleanup_interval: Duration,

    /// 预取配置
    pub prefetch_config: PrefetchConfig,
}

/// 解析器缓存配置
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParserCacheConfig {
    /// 关键字缓存容量
    pub keyword_cache_capacity: usize,

    /// 标记缓存容量
    pub token_cache_capacity: usize,

    /// 表达式缓存容量
    pub expression_cache_capacity: usize,

    /// 模式缓存容量
    pub pattern_cache_capacity: usize,

    /// 预取窗口大小
    pub prefetch_window: usize,

    /// 关键字缓存TTL
    pub keyword_cache_ttl: Duration,

    /// 标记缓存TTL
    pub token_cache_ttl: Duration,

    /// 表达式缓存TTL
    pub expression_cache_ttl: Duration,

    /// 模式缓存TTL
    pub pattern_cache_ttl: Duration,
}

/// 预取配置
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrefetchConfig {
    /// 是否启用预取
    pub enabled: bool,

    /// 预取窗口大小
    pub window_size: usize,

    /// 预取触发阈值
    pub trigger_threshold: usize,

    /// 预取策略
    pub strategy: PrefetchStrategy,
}

/// 预取策略
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PrefetchStrategy {
    /// 固定窗口预取
    Fixed,
    /// 自适应预取
    Adaptive,
    /// 基于访问模式的预取
    PatternBased,
}

/// 缓存策略
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CachePolicy {
    /// LRU (Least Recently Used)
    LRU,
    /// LFU (Least Frequently Used)
    LFU,
    /// FIFO (First In First Out)
    FIFO,
    /// TTL (Time To Live)
    TTL(Duration),
    /// 自适应策略
    Adaptive,
    /// 无驱逐策略
    None,
}

/// 缓存统计配置
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StatsConfig {
    /// 是否启用统计
    pub enabled: bool,

    /// 统计信息保留时间
    pub retention_time: Duration,

    /// 是否记录详细统计
    pub detailed: bool,

    /// 统计信息更新间隔
    pub update_interval: Duration,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_capacity: 1000,
            default_ttl: Duration::from_secs(300), // 5分钟
            default_policy: CachePolicy::LRU,
            collect_stats: true,
            parser_cache: ParserCacheConfig::default(),
            memory_limit: Some(100 * 1024 * 1024),     // 100MB
            cleanup_interval: Duration::from_secs(60), // 1分钟
            prefetch_config: PrefetchConfig::default(),
        }
    }
}

impl Default for ParserCacheConfig {
    fn default() -> Self {
        Self {
            keyword_cache_capacity: 1000,
            token_cache_capacity: 500,
            expression_cache_capacity: 200,
            pattern_cache_capacity: 100,
            prefetch_window: 10,
            keyword_cache_ttl: Duration::from_secs(600), // 10分钟
            token_cache_ttl: Duration::from_secs(300),   // 5分钟
            expression_cache_ttl: Duration::from_secs(180), // 3分钟
            pattern_cache_ttl: Duration::from_secs(240), // 4分钟
        }
    }
}

impl Default for PrefetchConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            window_size: 10,
            trigger_threshold: 3,
            strategy: PrefetchStrategy::Adaptive,
        }
    }
}

impl Default for StatsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            retention_time: Duration::from_secs(3600), // 1小时
            detailed: false,
            update_interval: Duration::from_secs(10), // 10秒
        }
    }
}

impl CacheConfig {
    /// 创建开发环境配置
    pub fn development() -> Self {
        Self {
            enabled: true,
            default_capacity: 500,
            default_ttl: Duration::from_secs(120), // 2分钟
            default_policy: CachePolicy::LRU,
            collect_stats: true,
            parser_cache: ParserCacheConfig::development(),
            memory_limit: Some(50 * 1024 * 1024),      // 50MB
            cleanup_interval: Duration::from_secs(30), // 30秒
            prefetch_config: PrefetchConfig::development(),
        }
    }

    /// 创建生产环境配置
    pub fn production() -> Self {
        Self {
            enabled: true,
            default_capacity: 2000,
            default_ttl: Duration::from_secs(600), // 10分钟
            default_policy: CachePolicy::LRU,
            collect_stats: true,
            parser_cache: ParserCacheConfig::production(),
            memory_limit: Some(200 * 1024 * 1024),      // 200MB
            cleanup_interval: Duration::from_secs(120), // 2分钟
            prefetch_config: PrefetchConfig::production(),
        }
    }

    /// 创建测试环境配置
    pub fn testing() -> Self {
        Self {
            enabled: true,
            default_capacity: 100,
            default_ttl: Duration::from_secs(60), // 1分钟
            default_policy: CachePolicy::LRU,
            collect_stats: false,
            parser_cache: ParserCacheConfig::testing(),
            memory_limit: Some(10 * 1024 * 1024),      // 10MB
            cleanup_interval: Duration::from_secs(10), // 10秒
            prefetch_config: PrefetchConfig::testing(),
        }
    }

    /// 验证配置
    pub fn validate(&self) -> Result<(), String> {
        if self.default_capacity == 0 {
            return Err("默认缓存容量必须大于0".to_string());
        }

        if let Some(memory_limit) = self.memory_limit {
            if memory_limit == 0 {
                return Err("内存限制必须大于0".to_string());
            }
        }

        self.parser_cache.validate()?;
        self.prefetch_config.validate()?;

        Ok(())
    }

    /// 获取总内存使用估算
    pub fn estimated_memory_usage(&self) -> usize {
        let parser_usage = self.parser_cache.estimated_memory_usage();
        let base_usage = 1024 * 1024; // 1MB 基础开销

        base_usage + parser_usage
    }
}

impl ParserCacheConfig {
    /// 创建开发环境配置
    pub fn development() -> Self {
        Self {
            keyword_cache_capacity: 500,
            token_cache_capacity: 250,
            expression_cache_capacity: 100,
            pattern_cache_capacity: 50,
            prefetch_window: 5,
            keyword_cache_ttl: Duration::from_secs(300), // 5分钟
            token_cache_ttl: Duration::from_secs(150),   // 2.5分钟
            expression_cache_ttl: Duration::from_secs(90), // 1.5分钟
            pattern_cache_ttl: Duration::from_secs(120), // 2分钟
        }
    }

    /// 创建生产环境配置
    pub fn production() -> Self {
        Self {
            keyword_cache_capacity: 2000,
            token_cache_capacity: 1000,
            expression_cache_capacity: 500,
            pattern_cache_capacity: 250,
            prefetch_window: 20,
            keyword_cache_ttl: Duration::from_secs(1200), // 20分钟
            token_cache_ttl: Duration::from_secs(600),    // 10分钟
            expression_cache_ttl: Duration::from_secs(360), // 6分钟
            pattern_cache_ttl: Duration::from_secs(480),  // 8分钟
        }
    }

    /// 创建测试环境配置
    pub fn testing() -> Self {
        Self {
            keyword_cache_capacity: 100,
            token_cache_capacity: 50,
            expression_cache_capacity: 25,
            pattern_cache_capacity: 10,
            prefetch_window: 3,
            keyword_cache_ttl: Duration::from_secs(60), // 1分钟
            token_cache_ttl: Duration::from_secs(30),   // 30秒
            expression_cache_ttl: Duration::from_secs(20), // 20秒
            pattern_cache_ttl: Duration::from_secs(25), // 25秒
        }
    }

    /// 验证配置
    pub fn validate(&self) -> Result<(), String> {
        if self.keyword_cache_capacity == 0 {
            return Err("关键字缓存容量必须大于0".to_string());
        }

        if self.token_cache_capacity == 0 {
            return Err("标记缓存容量必须大于0".to_string());
        }

        if self.expression_cache_capacity == 0 {
            return Err("表达式缓存容量必须大于0".to_string());
        }

        if self.pattern_cache_capacity == 0 {
            return Err("模式缓存容量必须大于0".to_string());
        }

        if self.prefetch_window == 0 {
            return Err("预取窗口必须大于0".to_string());
        }

        Ok(())
    }

    /// 估算内存使用
    pub fn estimated_memory_usage(&self) -> usize {
        // 估算每个缓存项的平均大小
        const KEYWORD_ENTRY_SIZE: usize = 50; // 平均50字节
        const TOKEN_ENTRY_SIZE: usize = 100; // 平均100字节
        const EXPRESSION_ENTRY_SIZE: usize = 500; // 平均500字节
        const PATTERN_ENTRY_SIZE: usize = 800; // 平均800字节

        self.keyword_cache_capacity * KEYWORD_ENTRY_SIZE
            + self.token_cache_capacity * TOKEN_ENTRY_SIZE
            + self.expression_cache_capacity * EXPRESSION_ENTRY_SIZE
            + self.pattern_cache_capacity * PATTERN_ENTRY_SIZE
    }
}

impl PrefetchConfig {
    /// 创建开发环境配置
    pub fn development() -> Self {
        Self {
            enabled: true,
            window_size: 5,
            trigger_threshold: 2,
            strategy: PrefetchStrategy::Adaptive,
        }
    }

    /// 创建生产环境配置
    pub fn production() -> Self {
        Self {
            enabled: true,
            window_size: 20,
            trigger_threshold: 5,
            strategy: PrefetchStrategy::PatternBased,
        }
    }

    /// 创建测试环境配置
    pub fn testing() -> Self {
        Self {
            enabled: false,
            window_size: 3,
            trigger_threshold: 1,
            strategy: PrefetchStrategy::Fixed,
        }
    }

    /// 验证配置
    pub fn validate(&self) -> Result<(), String> {
        if self.window_size == 0 {
            return Err("预取窗口大小必须大于0".to_string());
        }

        if self.trigger_threshold == 0 {
            return Err("预取触发阈值必须大于0".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CacheConfig::default();
        assert!(config.enabled);
        assert_eq!(config.default_capacity, 1000);
        assert_eq!(config.default_policy, CachePolicy::LRU);
        assert!(config.collect_stats);
    }

    #[test]
    fn test_development_config() {
        let config = CacheConfig::development();
        assert!(config.enabled);
        assert_eq!(config.default_capacity, 500);
        assert_eq!(config.parser_cache.keyword_cache_capacity, 500);
    }

    #[test]
    fn test_production_config() {
        let config = CacheConfig::production();
        assert!(config.enabled);
        assert_eq!(config.default_capacity, 2000);
        assert_eq!(config.parser_cache.keyword_cache_capacity, 2000);
    }

    #[test]
    fn test_testing_config() {
        let config = CacheConfig::testing();
        assert!(config.enabled);
        assert_eq!(config.default_capacity, 100);
        assert!(!config.collect_stats);
    }

    #[test]
    fn test_config_validation() {
        let mut config = CacheConfig::default();
        assert!(config.validate().is_ok());

        config.default_capacity = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_memory_estimation() {
        let config = CacheConfig::default();
        let usage = config.estimated_memory_usage();
        assert!(usage > 0);

        let parser_usage = config.parser_cache.estimated_memory_usage();
        assert!(parser_usage > 0);
    }
}
