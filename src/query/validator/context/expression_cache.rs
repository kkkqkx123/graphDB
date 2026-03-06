//! 全局表达式缓存
//!
//! 提供跨查询共享的表达式分析结果缓存，支持 LRU 淘汰策略。
//!
//! # 设计目标
//!
//! 1. 跨查询共享表达式分析结果（类型推导、常量折叠等）
//! 2. 限制内存使用，防止无限制增长
//! 3. 线程安全，支持高并发访问
//!
//! # 使用场景
//!
//! - 相同查询模板的重复执行
//! - 复杂表达式的类型推导结果复用
//! - 常量表达式的计算结果复用

use dashmap::DashMap;
use lru::LruCache;
use parking_lot::Mutex;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::core::types::expression::{ExpressionId, ExpressionMeta};
use crate::core::types::DataType;
use crate::core::Value;
use crate::query::optimizer::analysis::ExpressionAnalysis;

/// 全局表达式缓存配置
#[derive(Debug, Clone)]
pub struct ExpressionCacheConfig {
    /// 最大表达式条目数（表达式注册表）
    pub max_expressions: usize,
    /// 类型缓存大小
    pub type_cache_size: usize,
    /// 常量缓存大小
    pub constant_cache_size: usize,
    /// 分析缓存大小
    pub analysis_cache_size: usize,
    /// 条目最大存活时间（秒）
    pub ttl_seconds: u64,
}

impl Default for ExpressionCacheConfig {
    fn default() -> Self {
        Self {
            max_expressions: 10000,
            type_cache_size: 10000,
            constant_cache_size: 5000,
            analysis_cache_size: 5000,
            ttl_seconds: 3600, // 1小时
        }
    }
}

/// 缓存项包装器，包含创建时间和访问统计
#[derive(Debug, Clone)]
struct CachedItem<T> {
    value: T,
    created_at: Instant,
    last_accessed: Instant,
    access_count: u64,
}

impl<T> CachedItem<T> {
    fn new(value: T) -> Self {
        let now = Instant::now();
        Self {
            value,
            created_at: now,
            last_accessed: now,
            access_count: 1,
        }
    }

    fn record_access(&mut self) {
        self.last_accessed = Instant::now();
        self.access_count += 1;
    }

    fn is_expired(&self, ttl: Duration) -> bool {
        self.created_at.elapsed() > ttl
    }
}

/// 全局表达式缓存统计
#[derive(Debug, Clone, Default)]
pub struct ExpressionCacheStats {
    /// 表达式注册表命中次数
    pub expr_hits: u64,
    /// 表达式注册表未命中次数
    pub expr_misses: u64,
    /// 类型缓存命中次数
    pub type_hits: u64,
    /// 类型缓存未命中次数
    pub type_misses: u64,
    /// 常量缓存命中次数
    pub constant_hits: u64,
    /// 常量缓存未命中次数
    pub constant_misses: u64,
    /// 分析缓存命中次数
    pub analysis_hits: u64,
    /// 分析缓存未命中次数
    pub analysis_misses: u64,
    /// 过期条目数
    pub expirations: u64,
}

impl ExpressionCacheStats {
    /// 总命中率
    pub fn overall_hit_rate(&self) -> f64 {
        let total = self.expr_hits + self.expr_misses
            + self.type_hits + self.type_misses
            + self.constant_hits + self.constant_misses
            + self.analysis_hits + self.analysis_misses;
        if total == 0 {
            0.0
        } else {
            let hits = self.expr_hits + self.type_hits + self.constant_hits + self.analysis_hits;
            hits as f64 / total as f64
        }
    }

    /// 类型缓存命中率
    pub fn type_hit_rate(&self) -> f64 {
        let total = self.type_hits + self.type_misses;
        if total == 0 {
            0.0
        } else {
            self.type_hits as f64 / total as f64
        }
    }
}

/// 全局表达式缓存
///
/// 跨查询共享的表达式分析结果缓存，使用 LRU 策略管理内存。
pub struct GlobalExpressionCache {
    /// 表达式注册表（全局共享，使用 DashMap 支持并发）
    expressions: Arc<DashMap<ExpressionId, Arc<ExpressionMeta>>>,

    /// 类型缓存（LRU，带过期检查）
    type_cache: Mutex<LruCache<ExpressionId, CachedItem<DataType>>>,

    /// 常量缓存（LRU，带过期检查）
    constant_cache: Mutex<LruCache<ExpressionId, CachedItem<Value>>>,

    /// 分析缓存（LRU，带过期检查）
    analysis_cache: Mutex<LruCache<ExpressionId, CachedItem<ExpressionAnalysis>>>,

    /// 配置
    config: ExpressionCacheConfig,

    /// 统计信息
    stats: Mutex<ExpressionCacheStats>,
}

impl GlobalExpressionCache {
    /// 创建新的全局表达式缓存
    pub fn new(config: ExpressionCacheConfig) -> Self {
        Self {
            expressions: Arc::new(DashMap::new()),
            type_cache: Mutex::new(LruCache::new(
                NonZeroUsize::new(config.type_cache_size).unwrap()
            )),
            constant_cache: Mutex::new(LruCache::new(
                NonZeroUsize::new(config.constant_cache_size).unwrap()
            )),
            analysis_cache: Mutex::new(LruCache::new(
                NonZeroUsize::new(config.analysis_cache_size).unwrap()
            )),
            config,
            stats: Mutex::new(ExpressionCacheStats::default()),
        }
    }

    /// 使用默认配置创建
    pub fn default() -> Self {
        Self::new(ExpressionCacheConfig::default())
    }

    // ==================== 表达式注册表操作 ====================

    /// 获取表达式
    pub fn get_expression(&self, id: &ExpressionId) -> Option<Arc<ExpressionMeta>> {
        if let Some(expr) = self.expressions.get(id) {
            self.stats.lock().expr_hits += 1;
            Some(expr.clone())
        } else {
            self.stats.lock().expr_misses += 1;
            None
        }
    }

    /// 注册表达式
    ///
    /// 如果表达式已存在，返回现有 ID；否则注册新表达式。
    pub fn register_expression(&self, expr: ExpressionMeta) -> ExpressionId {
        let id = expr
            .id()
            .cloned()
            .unwrap_or_else(|| ExpressionId::new(self.expressions.len() as u64));

        self.expressions
            .entry(id.clone())
            .or_insert_with(|| Arc::new(expr));

        id
    }

    /// 检查表达式是否已注册
    pub fn contains_expression(&self, id: &ExpressionId) -> bool {
        self.expressions.contains_key(id)
    }

    /// 获取已注册表达式数量
    pub fn expression_count(&self) -> usize {
        self.expressions.len()
    }

    // ==================== 类型缓存操作 ====================

    /// 获取表达式类型
    pub fn get_type(&self, id: &ExpressionId) -> Option<DataType> {
        let ttl = Duration::from_secs(self.config.ttl_seconds);
        let mut cache = self.type_cache.lock();

        if let Some(item) = cache.get_mut(id) {
            if !item.is_expired(ttl) {
                item.record_access();
                self.stats.lock().type_hits += 1;
                return Some(item.value.clone());
            }
        }

        self.stats.lock().type_misses += 1;
        None
    }

    /// 设置表达式类型
    pub fn set_type(&self, id: ExpressionId, data_type: DataType) {
        let mut cache = self.type_cache.lock();
        cache.put(id, CachedItem::new(data_type));
    }

    // ==================== 常量缓存操作 ====================

    /// 获取常量值
    pub fn get_constant(&self, id: &ExpressionId) -> Option<Value> {
        let ttl = Duration::from_secs(self.config.ttl_seconds);
        let mut cache = self.constant_cache.lock();

        if let Some(item) = cache.get_mut(id) {
            if !item.is_expired(ttl) {
                item.record_access();
                self.stats.lock().constant_hits += 1;
                return Some(item.value.clone());
            }
        }

        self.stats.lock().constant_misses += 1;
        None
    }

    /// 设置常量值
    pub fn set_constant(&self, id: ExpressionId, value: Value) {
        let mut cache = self.constant_cache.lock();
        cache.put(id, CachedItem::new(value));
    }

    // ==================== 分析缓存操作 ====================

    /// 获取表达式分析结果
    pub fn get_analysis(&self, id: &ExpressionId) -> Option<ExpressionAnalysis> {
        let ttl = Duration::from_secs(self.config.ttl_seconds);
        let mut cache = self.analysis_cache.lock();

        if let Some(item) = cache.get_mut(id) {
            if !item.is_expired(ttl) {
                item.record_access();
                self.stats.lock().analysis_hits += 1;
                return Some(item.value.clone());
            }
        }

        self.stats.lock().analysis_misses += 1;
        None
    }

    /// 设置表达式分析结果
    pub fn set_analysis(&self, id: ExpressionId, analysis: ExpressionAnalysis) {
        let mut cache = self.analysis_cache.lock();
        cache.put(id, CachedItem::new(analysis));
    }

    // ==================== 统计和清理 ====================

    /// 获取统计信息
    pub fn stats(&self) -> ExpressionCacheStats {
        self.stats.lock().clone()
    }

    /// 清理过期条目
    pub fn cleanup_expired(&self) {
        let ttl = Duration::from_secs(self.config.ttl_seconds);

        // 清理类型缓存
        {
            let mut cache = self.type_cache.lock();
            let expired: Vec<_> = cache
                .iter()
                .filter(|(_, item)| item.is_expired(ttl))
                .map(|(k, _)| k.clone())
                .collect();
            for key in expired {
                cache.pop(&key);
                self.stats.lock().expirations += 1;
            }
        }

        // 清理常量缓存
        {
            let mut cache = self.constant_cache.lock();
            let expired: Vec<_> = cache
                .iter()
                .filter(|(_, item)| item.is_expired(ttl))
                .map(|(k, _)| k.clone())
                .collect();
            for key in expired {
                cache.pop(&key);
                self.stats.lock().expirations += 1;
            }
        }

        // 清理分析缓存
        {
            let mut cache = self.analysis_cache.lock();
            let expired: Vec<_> = cache
                .iter()
                .filter(|(_, item)| item.is_expired(ttl))
                .map(|(k, _)| k.clone())
                .collect();
            for key in expired {
                cache.pop(&key);
                self.stats.lock().expirations += 1;
            }
        }
    }

    /// 清空所有缓存
    pub fn clear(&self) {
        self.expressions.clear();
        self.type_cache.lock().clear();
        self.constant_cache.lock().clear();
        self.analysis_cache.lock().clear();
        *self.stats.lock() = ExpressionCacheStats::default();
    }
}

impl Default for GlobalExpressionCache {
    fn default() -> Self {
        Self::new(ExpressionCacheConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::expression::Expression;

    #[test]
    fn test_global_cache_creation() {
        let cache = GlobalExpressionCache::default();
        assert_eq!(cache.expression_count(), 0);
    }

    #[test]
    fn test_expression_registration() {
        let cache = GlobalExpressionCache::default();
        let expr = Expression::literal(42);
        let meta = ExpressionMeta::new(expr);

        let id = cache.register_expression(meta.clone());
        assert_eq!(cache.expression_count(), 1);

        let retrieved = cache.get_expression(&id);
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_type_caching() {
        let cache = GlobalExpressionCache::default();
        let id = ExpressionId::new(1);

        // 未设置时返回 None
        assert!(cache.get_type(&id).is_none());

        // 设置类型
        cache.set_type(id.clone(), DataType::Int);

        // 获取类型
        let data_type = cache.get_type(&id);
        assert_eq!(data_type, Some(DataType::Int));

        // 检查统计
        let stats = cache.stats();
        assert_eq!(stats.type_hits, 1);
        assert_eq!(stats.type_misses, 1);
    }

    #[test]
    fn test_constant_caching() {
        let cache = GlobalExpressionCache::default();
        let id = ExpressionId::new(1);

        // 设置常量
        cache.set_constant(id.clone(), Value::Int(42));

        // 获取常量
        let value = cache.get_constant(&id);
        assert_eq!(value, Some(Value::Int(42)));
    }
}
