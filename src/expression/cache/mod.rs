//! 表达式缓存管理模块
//!
//! 提供表达式求值过程中的缓存功能，包括函数结果缓存、表达式解析缓存等

use crate::cache::cache_impl::*;
use crate::cache::stats_marker::StatsEnabled;
use crate::cache::{Cache, CacheConfig, CacheFactory, StatsCache, StatsCacheWrapper};
use crate::core::types::expression::Expression;
use crate::core::Value;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// 表达式缓存管理器
#[derive(Debug)]
pub struct ExpressionCacheManager {
    /// 函数执行结果缓存
    function_cache:
        Arc<StatsCacheWrapper<String, Value, ConcurrentLruCache<String, Value>, StatsEnabled>>,
    /// 表达式解析结果缓存
    expression_cache: Arc<
        StatsCacheWrapper<String, Expression, ConcurrentLruCache<String, Expression>, StatsEnabled>,
    >,
    /// 变量查找缓存
    variable_cache:
        Arc<StatsCacheWrapper<String, Value, ConcurrentLruCache<String, Value>, StatsEnabled>>,
    /// 缓存配置
    config: CacheConfig,
}

impl ExpressionCacheManager {
    /// 创建新的表达式缓存管理器
    pub fn new(config: CacheConfig) -> Self {
        let function_cache =
            CacheFactory::create_lru_cache(config.parser_cache.expression_cache_capacity);
        let function_cache = CacheFactory::create_stats_wrapper(function_cache);

        let expression_cache =
            CacheFactory::create_lru_cache(config.parser_cache.expression_cache_capacity);
        let expression_cache = CacheFactory::create_stats_wrapper(expression_cache);

        let variable_cache =
            CacheFactory::create_lru_cache(config.parser_cache.expression_cache_capacity);
        let variable_cache = CacheFactory::create_stats_wrapper(variable_cache);

        Self {
            function_cache,
            expression_cache,
            variable_cache,
            config,
        }
    }

    /// 获取函数执行结果
    pub fn get_function_result(&self, key: &str) -> Option<Value> {
        if self.config.enabled {
            self.function_cache.get(&key.to_string())
        } else {
            None
        }
    }

    /// 缓存函数执行结果
    pub fn cache_function_result(&self, key: &str, result: Value) {
        if self.config.enabled {
            self.function_cache.put(key.to_string(), result);
        }
    }

    /// 获取表达式解析结果
    pub fn get_expression(&self, key: &str) -> Option<Expression> {
        if self.config.enabled {
            self.expression_cache.get(&key.to_string())
        } else {
            None
        }
    }

    /// 缓存表达式解析结果
    pub fn cache_expression(&self, key: &str, expression: Expression) {
        if self.config.enabled {
            self.expression_cache.put(key.to_string(), expression);
        }
    }

    /// 获取变量查找结果
    pub fn get_variable(&self, key: &str) -> Option<Value> {
        if self.config.enabled {
            self.variable_cache.get(&key.to_string())
        } else {
            None
        }
    }

    /// 缓存变量查找结果
    pub fn cache_variable(&self, key: &str, value: Value) {
        if self.config.enabled {
            self.variable_cache.put(key.to_string(), value);
        }
    }

    /// 获取缓存统计信息
    pub fn get_cache_stats(&self) -> ExpressionCacheStats {
        ExpressionCacheStats {
            function_cache_hits: self.function_cache.hits(),
            function_cache_misses: self.function_cache.misses(),
            function_cache_hit_rate: self.function_cache.hit_rate(),
            expression_cache_hits: self.expression_cache.hits(),
            expression_cache_misses: self.expression_cache.misses(),
            expression_cache_hit_rate: self.expression_cache.hit_rate(),
            variable_cache_hits: self.variable_cache.hits(),
            variable_cache_misses: self.variable_cache.misses(),
            variable_cache_hit_rate: self.variable_cache.hit_rate(),
        }
    }

    /// 清空所有缓存
    pub fn clear_all(&self) {
        self.function_cache.clear();
        self.expression_cache.clear();
        self.variable_cache.clear();
    }

    /// 重置统计信息
    pub fn reset_stats(&self) {
        self.function_cache.reset_stats();
        self.expression_cache.reset_stats();
        self.variable_cache.reset_stats();
    }
}

/// 表达式缓存统计信息
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpressionCacheStats {
    /// 函数缓存命中次数
    pub function_cache_hits: u64,
    /// 函数缓存未命中次数
    pub function_cache_misses: u64,
    /// 函数缓存命中率
    pub function_cache_hit_rate: f64,
    /// 表达式缓存命中次数
    pub expression_cache_hits: u64,
    /// 表达式缓存未命中次数
    pub expression_cache_misses: u64,
    /// 表达式缓存命中率
    pub expression_cache_hit_rate: f64,
    /// 变量缓存命中次数
    pub variable_cache_hits: u64,
    /// 变量缓存未命中次数
    pub variable_cache_misses: u64,
    /// 变量缓存命中率
    pub variable_cache_hit_rate: f64,
}
