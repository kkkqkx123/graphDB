//! 查询计划缓存模块
//!
//! 提供 Prepared Statement 风格的查询计划缓存，支持参数化查询。
//!
//! # 设计目标
//!
//! 1. 缓存查询计划的解析、验证和规划结果
//! 2. 支持参数化查询（Prepared Statement）
//! 3. 限制内存使用，防止无限制增长
//! 4. 线程安全，支持高并发访问
//!
//! # 使用场景
//!
//! - 相同查询模板的重复执行
//! - 批量插入/更新操作
//! - 应用程序使用 Prepared Statement

use lru::LruCache;
use parking_lot::Mutex;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::core::error::{DBError, DBResult};
use crate::query::planner::plan::ExecutionPlan;

/// 查询计划缓存配置
#[derive(Debug, Clone)]
pub struct PlanCacheConfig {
    /// 最大缓存条目数
    pub max_entries: usize,
    /// 条目最大存活时间（秒）
    pub ttl_seconds: u64,
    /// 是否启用参数化查询支持
    pub enable_parameterized: bool,
}

impl Default for PlanCacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            ttl_seconds: 3600, // 1小时
            enable_parameterized: true,
        }
    }
}

/// 缓存的查询计划条目
#[derive(Debug, Clone)]
pub struct CachedPlan {
    /// 查询模板（参数化形式）
    pub query_template: String,
    /// 执行计划
    pub plan: ExecutionPlan,
    /// 参数位置信息（用于参数绑定）
    pub param_positions: Vec<ParamPosition>,
    /// 创建时间
    pub created_at: Instant,
    /// 最后访问时间
    pub last_accessed: Instant,
    /// 访问次数
    pub access_count: u64,
    /// 平均执行时间（毫秒）
    pub avg_execution_time_ms: f64,
    /// 执行次数
    pub execution_count: u64,
}

/// 参数位置信息
#[derive(Debug, Clone)]
pub struct ParamPosition {
    /// 参数索引
    pub index: usize,
    /// 参数名称（命名参数）
    pub name: Option<String>,
    /// 参数在查询中的位置
    pub position: usize,
    /// 期望的数据类型
    pub expected_type: Option<crate::core::types::DataType>,
}

/// 查询计划缓存键
///
/// 使用查询文本的哈希作为键，支持快速查找。
/// 同时存储查询文本用于冲突检测。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlanCacheKey {
    /// 查询文本的哈希值
    pub hash: u64,
    /// 查询文本（用于冲突检测，不只是调试）
    query_text: String,
}

impl PlanCacheKey {
    /// 从查询文本创建缓存键
    pub fn from_query(query: &str) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        query.hash(&mut hasher);
        let hash = hasher.finish();

        Self {
            hash,
            query_text: query.to_string(),
        }
    }

    /// 验证查询文本是否匹配（用于冲突检测）
    pub fn verify_query(&self, query: &str) -> bool {
        self.query_text == query
    }

    /// 获取查询文本（用于调试或日志）
    pub fn query_text(&self) -> &str {
        &self.query_text
    }
}

/// 查询计划缓存统计
#[derive(Debug, Clone, Default)]
pub struct PlanCacheStats {
    /// 命中次数
    pub hits: u64,
    /// 未命中次数
    pub misses: u64,
    /// 淘汰次数
    pub evictions: u64,
    /// 过期次数
    pub expirations: u64,
    /// 当前缓存条目数
    pub current_entries: usize,
    /// 平均查询模板大小（字节）
    pub avg_query_template_bytes: usize,
    /// 总查询模板大小（字节）
    pub total_query_template_bytes: usize,
}

impl PlanCacheStats {
    /// 命中率
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    /// 估算总内存占用（基于平均模板大小和条目数）
    pub fn estimated_memory_bytes(&self) -> usize {
        // 基础开销 + 每个条目的估算内存
        // 每个条目大约包含：CachedPlan结构体 + ExecutionPlan + 查询模板字符串
        const PER_ENTRY_OVERHEAD: usize = 1024; // 结构体和执行计划的估算开销
        self.total_query_template_bytes + (self.current_entries * PER_ENTRY_OVERHEAD)
    }
}

/// 查询计划缓存
///
/// 提供 Prepared Statement 风格的查询计划缓存
pub struct QueryPlanCache {
    /// 缓存存储
    cache: Mutex<LruCache<PlanCacheKey, Arc<CachedPlan>>>,
    /// 配置
    config: PlanCacheConfig,
    /// 统计信息
    stats: Mutex<PlanCacheStats>,
}

impl std::fmt::Debug for QueryPlanCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let stats = self.stats.lock();
        f.debug_struct("QueryPlanCache")
            .field("config", &self.config)
            .field("stats", &*stats)
            .finish()
    }
}

impl QueryPlanCache {
    /// 创建新的查询计划缓存
    pub fn new(config: PlanCacheConfig) -> Self {
        Self {
            cache: Mutex::new(LruCache::new(
                NonZeroUsize::new(config.max_entries).expect("缓存条目数必须大于0"),
            )),
            config,
            stats: Mutex::new(PlanCacheStats::default()),
        }
    }

    /// 使用默认配置创建
    pub fn default() -> Self {
        Self::new(PlanCacheConfig::default())
    }

    /// 获取缓存的计划
    ///
    /// # 参数
    /// - `query`: 查询文本
    ///
    /// # 返回
    /// - `Some(Arc<CachedPlan>)`: 缓存的计划
    /// - `None`: 未找到或哈希冲突
    pub fn get(&self, query: &str) -> Option<Arc<CachedPlan>> {
        let key = PlanCacheKey::from_query(query);
        let ttl = Duration::from_secs(self.config.ttl_seconds);

        let mut cache = self.cache.lock();
        let mut stats = self.stats.lock();

        if let Some(plan) = cache.get(&key) {
            // 检查是否过期
            if plan.created_at.elapsed() > ttl {
                // 过期，移除并返回 None
                cache.pop(&key);
                stats.expirations += 1;
                stats.misses += 1;
                return None;
            }

            // 哈希冲突检测：验证查询文本是否匹配
            if plan.query_template != query {
                // 发生哈希冲突，记录日志并返回 None
                log::warn!(
                    "查询计划缓存哈希冲突 detected: hash={}, expected_query={}, cached_query={}",
                    key.hash,
                    query,
                    plan.query_template
                );
                stats.misses += 1;
                return None;
            }

            // 更新访问统计
            stats.hits += 1;
            return Some(plan.clone());
        }

        stats.misses += 1;
        None
    }

    /// 将计划放入缓存
    ///
    /// # 参数
    /// - `query`: 查询文本
    /// - `plan`: 执行计划
    /// - `param_positions`: 参数位置信息
    pub fn put(&self, query: &str, plan: ExecutionPlan, param_positions: Vec<ParamPosition>) {
        let key = PlanCacheKey::from_query(query);
        let query_bytes = query.len();

        let cached_plan = Arc::new(CachedPlan {
            query_template: query.to_string(),
            plan,
            param_positions,
            created_at: Instant::now(),
            last_accessed: Instant::now(),
            access_count: 0,
            avg_execution_time_ms: 0.0,
            execution_count: 0,
        });

        let mut cache = self.cache.lock();
        let old_len = cache.len();

        // 检查是否是更新已有条目
        let is_update = cache.contains(&key);

        cache.put(key, cached_plan);
        let new_len = cache.len();

        // 更新统计
        let mut stats = self.stats.lock();
        if new_len <= old_len && old_len > 0 && !is_update {
            // 发生了淘汰
            stats.evictions += 1;
        }

        // 更新大小统计
        if is_update {
            // 更新已有条目，重新计算总大小
            stats.total_query_template_bytes = cache
                .iter()
                .map(|(_, plan)| plan.query_template.len())
                .sum();
        } else {
            // 新条目
            stats.total_query_template_bytes += query_bytes;
        }

        stats.current_entries = new_len;
        if stats.current_entries > 0 {
            stats.avg_query_template_bytes =
                stats.total_query_template_bytes / stats.current_entries;
        }
    }

    /// 记录计划执行统计
    ///
    /// # 参数
    /// - `query`: 查询文本
    /// - `execution_time_ms`: 执行时间（毫秒）
    pub fn record_execution(&self, query: &str, execution_time_ms: f64) {
        let key = PlanCacheKey::from_query(query);

        let mut cache = self.cache.lock();
        if let Some(plan) = cache.get_mut(&key) {
            // 使用 Arc::make_mut 获取可变引用
            let plan_mut = Arc::make_mut(plan);
            plan_mut.execution_count += 1;

            // 更新平均执行时间（指数移动平均）
            let alpha = 0.1; // 平滑因子
            plan_mut.avg_execution_time_ms =
                plan_mut.avg_execution_time_ms * (1.0 - alpha) + execution_time_ms * alpha;
        }
    }

    /// 检查查询是否已缓存
    pub fn contains(&self, query: &str) -> bool {
        let key = PlanCacheKey::from_query(query);
        let cache = self.cache.lock();
        cache.contains(&key)
    }

    /// 使缓存条目失效
    pub fn invalidate(&self, query: &str) -> bool {
        let key = PlanCacheKey::from_query(query);
        let mut cache = self.cache.lock();
        let removed = cache.pop(&key).is_some();

        if removed {
            let mut stats = self.stats.lock();
            stats.current_entries = cache.len();
        }

        removed
    }

    /// 清空所有缓存
    pub fn clear(&self) {
        let mut cache = self.cache.lock();
        cache.clear();

        let mut stats = self.stats.lock();
        stats.current_entries = 0;
        stats.total_query_template_bytes = 0;
        stats.avg_query_template_bytes = 0;
    }

    /// 获取统计信息
    pub fn stats(&self) -> PlanCacheStats {
        let mut stats = self.stats.lock();
        stats.current_entries = self.cache.lock().len();
        stats.clone()
    }

    /// 清理过期条目
    pub fn cleanup_expired(&self) {
        let ttl = Duration::from_secs(self.config.ttl_seconds);
        let mut cache = self.cache.lock();
        let mut stats = self.stats.lock();

        let expired_keys: Vec<_> = cache
            .iter()
            .filter(|(_, plan)| plan.created_at.elapsed() > ttl)
            .map(|(k, _)| k.clone())
            .collect();

        for key in expired_keys {
            cache.pop(&key);
            stats.expirations += 1;
        }

        stats.current_entries = cache.len();
    }

    /// 获取缓存条目数
    pub fn len(&self) -> usize {
        self.cache.lock().len()
    }

    /// 检查缓存是否为空
    pub fn is_empty(&self) -> bool {
        self.cache.lock().is_empty()
    }
}

impl Default for QueryPlanCache {
    fn default() -> Self {
        Self::new(PlanCacheConfig::default())
    }
}

/// 参数化查询处理器
///
/// 处理参数化查询的解析和绑定
pub struct ParameterizedQueryHandler {
    /// 参数占位符模式
    placeholder_pattern: regex::Regex,
}

impl ParameterizedQueryHandler {
    /// 创建新的参数化查询处理器
    pub fn new() -> Self {
        Self {
            placeholder_pattern: regex::Regex::new(r"\$(\d+|[a-zA-Z_][a-zA-Z0-9_]*)")
                .expect("占位符正则表达式编译失败"),
        }
    }

    /// 提取查询中的参数位置
    ///
    /// # 参数
    /// - `query`: 查询文本
    ///
    /// # 返回
    /// 参数位置列表
    pub fn extract_params(&self, query: &str) -> Vec<ParamPosition> {
        let mut positions = Vec::new();

        for (idx, cap) in self.placeholder_pattern.captures_iter(query).enumerate() {
            let mat = cap.get(0).expect("正则表达式捕获组不应为空");
            let param_str = &cap[1];

            // 判断是位置参数还是命名参数
            let (name, index) = if let Ok(num) = param_str.parse::<usize>() {
                (None, num.saturating_sub(1)) // $1 对应索引 0
            } else {
                (Some(param_str.to_string()), idx)
            };

            positions.push(ParamPosition {
                index,
                name,
                position: mat.start(),
                expected_type: None, // 类型在验证阶段确定
            });
        }

        positions
    }

    /// 将参数绑定到查询模板
    ///
    /// # 参数
    /// - `template`: 查询模板
    /// - `params`: 参数值
    ///
    /// # 返回
    /// 绑定后的完整查询
    pub fn bind_params(&self, template: &str, params: &[crate::core::Value]) -> DBResult<String> {
        let positions = self.extract_params(template);

        if positions.len() != params.len() {
            return Err(DBError::Validation(format!(
                "参数数量不匹配: 期望 {}, 实际 {}",
                positions.len(),
                params.len()
            )));
        }

        let mut result = template.to_string();
        let param_strings: Vec<String> = params.iter().map(|v| format!("{}", v)).collect();

        // 从后向前替换，避免位置偏移
        for (pos, value) in positions.iter().zip(param_strings.iter()).rev() {
            result.replace_range(pos.position..pos.position + 2, value);
        }

        Ok(result)
    }
}

impl Default for ParameterizedQueryHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_cache_creation() {
        let cache = QueryPlanCache::default();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_plan_cache_key() {
        let key1 = PlanCacheKey::from_query("MATCH (n) RETURN n");
        let key2 = PlanCacheKey::from_query("MATCH (n) RETURN n");
        let key3 = PlanCacheKey::from_query("MATCH (m) RETURN m");

        assert_eq!(key1.hash, key2.hash);
        assert_ne!(key1.hash, key3.hash);
    }

    #[test]
    fn test_cache_stats() {
        let cache = QueryPlanCache::default();
        let stats = cache.stats();

        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.hit_rate(), 0.0);
    }

    #[test]
    fn test_param_handler_creation() {
        let handler = ParameterizedQueryHandler::new();
        let positions = handler.extract_params("SELECT * FROM t WHERE id = $1 AND name = $2");

        assert_eq!(positions.len(), 2);
        assert_eq!(positions[0].index, 0);
        assert_eq!(positions[1].index, 1);
    }
}
