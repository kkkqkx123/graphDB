//! 求值器上下文定义
//!
//! 提供表达式求值过程中的上下文管理

use crate::core::context::base::ContextBase;
use crate::core::context::expression::{
    BasicExpressionContext, EvaluationOptions, EvaluationStatistics, ExpressionContext,
    ExpressionError,
};
use crate::core::types::expression::Expression;
use crate::core::types::query::FieldValue;
use crate::cache::{Cache, StatsCache, ConcurrentLruCache, StatsCacheWrapper, CacheConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// 求值器上下文
#[derive(Debug)]
pub struct EvaluationContext {
    /// 基础表达式上下文
    pub expression_context: Arc<BasicExpressionContext>,
    /// 求值选项
    pub options: EvaluationOptions,
    /// 求值统计
    pub statistics: EvaluationStatistics,
    /// 求值缓存 - 使用全局缓存模块
    pub cache: Arc<StatsCacheWrapper<String, FieldValue, ConcurrentLruCache<String, FieldValue>>>,
    /// 求值历史
    pub history: Arc<RwLock<Vec<EvaluationRecord>>>,
    /// 求值深度
    pub depth: usize,
    /// 开始时间
    pub start_time: std::time::Instant,
}

/// 求值记录
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvaluationRecord {
    /// 表达式
    pub expression: Expression,
    /// 求值结果
    pub result: Result<FieldValue, ExpressionError>,
    /// 求值时间（微秒）
    pub evaluation_time_us: u64,
    /// 求值深度
    pub depth: usize,
    /// 时间戳
    pub timestamp: std::time::SystemTime,
    /// 上下文快照
    pub context_snapshot: Option<ContextSnapshot>,
}

/// 上下文快照
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextSnapshot {
    /// 变量名
    pub variable_names: Vec<String>,
    /// 函数名
    pub function_names: Vec<String>,
    /// 上下文深度
    pub depth: usize,
}

impl EvaluationContext {
    /// 创建新的求值上下文
    pub fn new(expression_context: Arc<BasicExpressionContext>) -> Self {
        let cache_config = CacheConfig::default();
        let lru_cache = Arc::new(ConcurrentLruCache::new(cache_config.default_capacity));
        let stats_cache = Arc::new(StatsCacheWrapper::new(lru_cache));
        
        Self {
            expression_context,
            options: EvaluationOptions::default(),
            statistics: EvaluationStatistics::default(),
            cache: stats_cache,
            history: Arc::new(RwLock::new(Vec::new())),
            depth: 0,
            start_time: std::time::Instant::now(),
        }
    }

    /// 创建带选项的求值上下文
    pub fn with_options(
        expression_context: Arc<BasicExpressionContext>,
        options: EvaluationOptions,
    ) -> Self {
        let cache_config = CacheConfig::default();
        let lru_cache = Arc::new(ConcurrentLruCache::new(cache_config.default_capacity));
        let stats_cache = Arc::new(StatsCacheWrapper::new(lru_cache));
        
        Self {
            expression_context,
            options,
            statistics: EvaluationStatistics::default(),
            cache: stats_cache,
            history: Arc::new(RwLock::new(Vec::new())),
            depth: 0,
            start_time: std::time::Instant::now(),
        }
    }

    /// 创建子上下文
    pub fn create_child_context(&self) -> Self {
        let child_context = BasicExpressionContext::with_parent((*self.expression_context).clone());
        Self {
            expression_context: Arc::new(child_context),
            options: self.options.clone(),
            statistics: EvaluationStatistics::default(),
            cache: Arc::clone(&self.cache),
            history: Arc::clone(&self.history),
            depth: self.depth + 1,
            start_time: std::time::Instant::now(),
        }
    }

    /// 检查缓存
    pub fn check_cache(&self, expression: &Expression) -> Option<FieldValue> {
        if !self.options.enable_cache {
            return None;
        }

        let cache_key = self.generate_cache_key(expression);
        self.cache.get(&cache_key)
    }

    /// 添加到缓存
    pub fn add_to_cache(&self, expression: &Expression, value: &FieldValue) {
        if !self.options.enable_cache {
            return;
        }

        let cache_key = self.generate_cache_key(expression);
        self.cache.put(cache_key, value.clone());
    }

    /// 记录求值结果
    pub fn record_evaluation(
        &self,
        expression: &Expression,
        result: &Result<FieldValue, ExpressionError>,
        evaluation_time_us: u64,
    ) {
        // 添加到历史记录
        let record = EvaluationRecord {
            expression: expression.clone(),
            result: result.clone(),
            evaluation_time_us,
            depth: self.depth,
            timestamp: std::time::SystemTime::now(),
            context_snapshot: Some(ContextSnapshot {
                variable_names: self
                    .expression_context
                    .get_variable_names()
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect(),
                function_names: Vec::new(), // 需要从上下文获取函数名
                depth: self.expression_context.depth(),
            }),
        };

        if let Ok(mut history) = self.history.write() {
            history.push(record);

            // 限制历史记录大小
            if history.len() > 1000 {
                history.remove(0);
            }
        }
    }

    /// 记录求值结果（可变版本）
    pub fn record_evaluation_mut(
        &mut self,
        expression: &Expression,
        result: &Result<FieldValue, ExpressionError>,
        evaluation_time_us: u64,
    ) {
        // 更新统计信息
        self.statistics
            .record_expression_evaluation(evaluation_time_us);
        self.statistics.update_max_recursion_depth(self.depth);

        // 添加到历史记录
        let record = EvaluationRecord {
            expression: expression.clone(),
            result: result.clone(),
            evaluation_time_us,
            depth: self.depth,
            timestamp: std::time::SystemTime::now(),
            context_snapshot: Some(ContextSnapshot {
                variable_names: self
                    .expression_context
                    .get_variable_names()
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect(),
                function_names: Vec::new(), // 需要从上下文获取函数名
                depth: self.expression_context.depth(),
            }),
        };

        if let Ok(mut history) = self.history.write() {
            history.push(record);

            // 限制历史记录大小
            if history.len() > 1000 {
                history.remove(0);
            }
        }
    }

    /// 生成缓存键
    fn generate_cache_key(&self, expression: &Expression) -> String {
        // 简单的哈希实现，实际应用中可能需要更复杂的逻辑
        format!("{:?}", expression)
    }

    /// 估算值的大小（字节）
    fn estimate_value_size(&self, value: &FieldValue) -> usize {
        match value {
            FieldValue::Scalar(scalar) => std::mem::size_of_val(scalar),
            FieldValue::List(list) => {
                list.iter()
                    .map(|v| self.estimate_value_size(v))
                    .sum::<usize>()
                    + std::mem::size_of::<Vec<FieldValue>>()
            }
            FieldValue::Map(map) => {
                map.iter()
                    .map(|(k, v)| k.len() + self.estimate_value_size(v))
                    .sum::<usize>()
                    + std::mem::size_of::<Vec<(String, FieldValue)>>()
            }
            FieldValue::Vertex(vertex) => {
                vertex.id.len()
                    + vertex.tags.iter().map(|t| t.len()).sum::<usize>()
                    + vertex
                        .properties
                        .iter()
                        .map(|(k, v)| k.len() + std::mem::size_of_val(v))
                        .sum::<usize>()
                    + std::mem::size_of::<crate::core::types::query::Vertex>()
            }
            FieldValue::Edge(edge) => {
                edge.id.len()
                    + edge.edge_type.len()
                    + edge.src.len()
                    + edge.dst.len()
                    + edge
                        .properties
                        .iter()
                        .map(|(k, v)| k.len() + std::mem::size_of_val(v))
                        .sum::<usize>()
                    + std::mem::size_of::<crate::core::types::query::Edge>()
            }
            FieldValue::Path(path) => {
                path.segments.len() * std::mem::size_of::<crate::core::types::query::PathSegment>()
                    + std::mem::size_of::<crate::core::types::query::Path>()
            }
        }
    }

    /// 获取求值时间（毫秒）
    pub fn elapsed_ms(&self) -> u64 {
        self.start_time.elapsed().as_millis() as u64
    }

    /// 检查是否超时
    pub fn is_timeout(&self) -> bool {
        if let Some(timeout_ms) = self.options.timeout_ms {
            self.elapsed_ms() > timeout_ms
        } else {
            false
        }
    }

    /// 检查是否超出递归深度
    pub fn exceeds_recursion_depth(&self) -> bool {
        self.depth > self.options.max_recursion_depth
    }

    /// 获取缓存统计
    pub fn get_cache_statistics(&self) -> CacheStatistics {
        CacheStatistics {
            size: self.cache.len(),
            max_size: 1000, // 默认容量，全局缓存模块不直接提供
            hits: self.cache.hits(),
            misses: self.cache.misses(),
            hit_rate: self.cache.hit_rate(),
            strategy: crate::cache::CacheStrategy::LRU,
            total_memory_bytes: 0, // 全局缓存模块不提供此信息
        }
    }

    /// 清空缓存
    pub fn clear_cache(&self) {
        self.cache.clear();
    }

    /// 获取求值历史
    pub fn get_evaluation_history(&self) -> Vec<EvaluationRecord> {
        if let Ok(history) = self.history.read() {
            history.clone()
        } else {
            Vec::new()
        }
    }

    /// 清空求值历史
    pub fn clear_history(&self) {
        if let Ok(mut history) = self.history.write() {
            history.clear();
        }
    }
}


/// 缓存统计信息
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CacheStatistics {
    /// 当前缓存大小
    pub size: usize,
    /// 最大缓存大小
    pub max_size: usize,
    /// 命中次数
    pub hits: u64,
    /// 未命中次数
    pub misses: u64,
    /// 命中率
    pub hit_rate: f64,
    /// 缓存策略
    pub strategy: crate::cache::CacheStrategy,
    /// 总内存使用量（字节）
    pub total_memory_bytes: usize,
}

impl Default for CacheStatistics {
    fn default() -> Self {
        Self {
            size: 0,
            max_size: 0,
            hits: 0,
            misses: 0,
            hit_rate: 0.0,
            strategy: crate::cache::CacheStrategy::LRU,
            total_memory_bytes: 0,
        }
    }
}

impl Clone for EvaluationContext {
    fn clone(&self) -> Self {
        Self {
            expression_context: Arc::clone(&self.expression_context),
            options: self.options.clone(),
            statistics: self.statistics.clone(),
            cache: Arc::clone(&self.cache),
            history: Arc::clone(&self.history),
            depth: self.depth,
            start_time: std::time::Instant::now(),
        }
    }
}
