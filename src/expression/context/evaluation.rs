//! 表达式求值模块
//!
//! 提供表达式求值过程中的选项配置和统计信息

use crate::cache::CacheConfig;
use crate::expression::cache::ExpressionCacheStats;
use serde::{Deserialize, Serialize};

/// 表达式求值选项
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvaluationOptions {
    /// 是否启用严格模式
    pub strict_mode: bool,
    /// 是否允许隐式类型转换
    pub allow_implicit_conversion: bool,
    /// 最大递归深度
    pub max_recursion_depth: usize,
    /// 超时时间（毫秒）
    pub timeout_ms: Option<u64>,
    /// 缓存配置
    pub cache_config: CacheConfig,
}

impl Default for EvaluationOptions {
    fn default() -> Self {
        Self {
            strict_mode: false,
            allow_implicit_conversion: true,
            max_recursion_depth: 1000,
            timeout_ms: Some(30000), // 30秒
            cache_config: CacheConfig::default(),
        }
    }
}

/// 表达式求值统计
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvaluationStatistics {
    /// 求值的表达式数量
    pub expressions_evaluated: usize,
    /// 函数调用次数
    pub function_calls: usize,
    /// 变量访问次数
    pub variable_accesses: usize,
    /// 总求值时间（微秒）
    pub total_evaluation_time_us: u64,
    /// 平均求值时间（微秒）
    pub average_evaluation_time_us: f64,
    /// 最大递归深度
    pub max_recursion_depth: usize,
    /// 详细的缓存统计信息
    pub cache_stats: Option<crate::expression::cache::ExpressionCacheStats>,
}

impl EvaluationStatistics {
    /// 创建新的求值统计
    pub fn new() -> Self {
        Self {
            expressions_evaluated: 0,
            function_calls: 0,
            variable_accesses: 0,
            total_evaluation_time_us: 0,
            average_evaluation_time_us: 0.0,
            max_recursion_depth: 0,
            cache_stats: None,
        }
    }

    /// 创建带缓存统计的求值统计
    pub fn with_cache_stats(cache_stats: ExpressionCacheStats) -> Self {
        Self {
            expressions_evaluated: 0,
            function_calls: 0,
            variable_accesses: 0,
            total_evaluation_time_us: 0,
            average_evaluation_time_us: 0.0,
            max_recursion_depth: 0,
            cache_stats: Some(cache_stats),
        }
    }

    /// 记录表达式求值
    pub fn record_expression_evaluation(&mut self, evaluation_time_us: u64) {
        self.expressions_evaluated += 1;
        self.total_evaluation_time_us += evaluation_time_us;
        self.average_evaluation_time_us =
            self.total_evaluation_time_us as f64 / self.expressions_evaluated as f64;
    }

    /// 记录函数调用
    pub fn record_function_call(&mut self) {
        self.function_calls += 1;
    }

    /// 记录变量访问
    pub fn record_variable_access(&mut self) {
        self.variable_accesses += 1;
    }

    /// 更新缓存统计信息
    pub fn update_cache_stats(&mut self, cache_stats: Option<ExpressionCacheStats>) {
        self.cache_stats = cache_stats;
    }

    /// 更新最大递归深度
    pub fn update_max_recursion_depth(&mut self, depth: usize) {
        if depth > self.max_recursion_depth {
            self.max_recursion_depth = depth;
        }
    }

    /// 获取总体缓存命中率
    pub fn overall_cache_hit_rate(&self) -> f64 {
        if let Some(ref cache_stats) = self.cache_stats {
            let total_hits = cache_stats.function_cache_hits
                + cache_stats.expression_cache_hits
                + cache_stats.variable_cache_hits;
            let total_misses = cache_stats.function_cache_misses
                + cache_stats.expression_cache_misses
                + cache_stats.variable_cache_misses;
            let total_requests = total_hits + total_misses;

            if total_requests == 0 {
                0.0
            } else {
                total_hits as f64 / total_requests as f64
            }
        } else {
            0.0
        }
    }

    /// 获取函数缓存命中率
    pub fn function_cache_hit_rate(&self) -> f64 {
        self.cache_stats
            .as_ref()
            .map(|stats| stats.function_cache_hit_rate)
            .unwrap_or_else(|| 0.0)
    }

    /// 获取表达式缓存命中率
    pub fn expression_cache_hit_rate(&self) -> f64 {
        self.cache_stats
            .as_ref()
            .map(|stats| stats.expression_cache_hit_rate)
            .unwrap_or_else(|| 0.0)
    }

    /// 获取变量缓存命中率
    pub fn variable_cache_hit_rate(&self) -> f64 {
        self.cache_stats
            .as_ref()
            .map(|stats| stats.variable_cache_hit_rate)
            .unwrap_or_else(|| 0.0)
    }
}

impl Default for EvaluationStatistics {
    fn default() -> Self {
        Self::new()
    }
}
