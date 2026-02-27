//! 决策缓存实现
//!
//! 提供基于 LRU 的优化决策缓存，支持版本感知和统计。

use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, Instant};

use lru::LruCache;
use parking_lot::Mutex;

use crate::query::planner::planner::SentenceKind;
use crate::query::optimizer::decision::types::OptimizationDecision;

/// 决策缓存错误
#[derive(Debug, thiserror::Error)]
pub enum DecisionCacheError {
    #[error("缓存键构建失败: {0}")]
    KeyBuildFailed(String),
    #[error("决策计算失败: {0}")]
    ComputationFailed(String),
    #[error("缓存操作失败: {0}")]
    OperationFailed(String),
}

/// 决策缓存键
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct DecisionCacheKey {
    /// 查询模板哈希
    query_template_hash: u64,
    /// 图空间ID
    space_id: Option<i32>,
    /// 语句类型
    statement_type: SentenceKind,
    /// 模式指纹
    pattern_fingerprint: Option<String>,
}

impl DecisionCacheKey {
    /// 创建新的缓存键
    pub fn new(
        query_template_hash: u64,
        space_id: Option<i32>,
        statement_type: SentenceKind,
        pattern_fingerprint: Option<String>,
    ) -> Self {
        Self {
            query_template_hash,
            space_id,
            statement_type,
            pattern_fingerprint,
        }
    }

    /// 从查询模板创建哈希
    pub fn hash_template(template: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        template.hash(&mut hasher);
        hasher.finish()
    }
}

/// 缓存的决策项
#[derive(Debug, Clone)]
pub struct CachedDecision {
    /// 优化决策
    pub decision: OptimizationDecision,
    /// 创建时间
    pub created_at: Instant,
    /// 最后访问时间
    pub last_accessed: Instant,
    /// 访问次数
    pub access_count: u64,
}

impl CachedDecision {
    /// 创建新的缓存项
    pub fn new(decision: OptimizationDecision) -> Self {
        let now = Instant::now();
        Self {
            decision,
            created_at: now,
            last_accessed: now,
            access_count: 1,
        }
    }

    /// 记录访问
    pub fn record_access(&mut self) {
        self.last_accessed = Instant::now();
        self.access_count += 1;
    }

    /// 计算缓存价值分数（用于淘汰策略）
    /// 分数越高越应该保留
    pub fn value_score(&self) -> f64 {
        let age_secs = self.created_at.elapsed().as_secs() as f64;
        let recency = 1.0 / (1.0 + age_secs / 3600.0);
        let frequency = (self.access_count as f64).ln_1p();
        recency * frequency
    }

    /// 检查是否过期
    pub fn is_expired(&self, ttl: Duration) -> bool {
        self.created_at.elapsed() > ttl
    }
}

/// 决策缓存统计
#[derive(Debug, Clone, Default)]
pub struct DecisionCacheStats {
    /// 命中次数
    pub hits: u64,
    /// 未命中次数
    pub misses: u64,
    /// 插入次数
    pub inserts: u64,
    /// 淘汰次数
    pub evictions: u64,
    /// 版本不匹配次数
    pub version_mismatches: u64,
    /// 过期次数
    pub expirations: u64,
}

impl DecisionCacheStats {
    /// 总查询次数
    pub fn total_queries(&self) -> u64 {
        self.hits + self.misses
    }

    /// 命中率
    pub fn hit_rate(&self) -> f64 {
        let total = self.total_queries();
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

/// 决策缓存配置
#[derive(Debug, Clone)]
pub struct DecisionCacheConfig {
    /// 最大缓存条目数
    pub max_entries: usize,
    /// 条目最大存活时间（秒）
    pub ttl_seconds: u64,
    /// 启用统计
    pub enable_stats: bool,
}

impl Default for DecisionCacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            ttl_seconds: 3600, // 1小时
            enable_stats: true,
        }
    }
}

/// 决策缓存
#[derive(Debug)]
pub struct DecisionCache {
    /// LRU 缓存
    cache: Mutex<LruCache<DecisionCacheKey, CachedDecision>>,
    /// 统计信息
    stats: Mutex<DecisionCacheStats>,
    /// 配置
    config: DecisionCacheConfig,
}

impl DecisionCache {
    /// 创建新的决策缓存
    pub fn new(config: DecisionCacheConfig) -> Result<Self, DecisionCacheError> {
        if config.max_entries == 0 {
            return Err(DecisionCacheError::OperationFailed(
                "缓存大小必须大于0".to_string(),
            ));
        }

        let cache_size = NonZeroUsize::new(config.max_entries)
            .ok_or_else(|| DecisionCacheError::OperationFailed(
                "创建缓存失败".to_string(),
            ))?;

        Ok(Self {
            cache: Mutex::new(LruCache::new(cache_size)),
            stats: Mutex::new(DecisionCacheStats::default()),
            config,
        })
    }

    /// 使用默认配置创建
    pub fn with_default_config() -> Result<Self, DecisionCacheError> {
        Self::new(DecisionCacheConfig::default())
    }

    /// 获取决策（带版本检查）
    pub fn get(
        &self,
        key: &DecisionCacheKey,
        current_stats_version: u64,
        current_index_version: u64,
    ) -> Result<Option<OptimizationDecision>, DecisionCacheError> {
        let mut cache = self.cache.lock();
        let ttl = Duration::from_secs(self.config.ttl_seconds);

        if let Some(cached) = cache.get_mut(key) {
            // 检查是否过期
            if cached.is_expired(ttl) {
                drop(cache);
                self.record_expiration();
                return Ok(None);
            }

            // 检查版本是否匹配
            if !cached.decision.is_valid(current_stats_version, current_index_version) {
                drop(cache);
                self.record_version_mismatch();
                return Ok(None);
            }

            // 命中
            cached.record_access();
            let decision = cached.decision.clone();
            drop(cache);
            self.record_hit();
            Ok(Some(decision))
        } else {
            drop(cache);
            self.record_miss();
            Ok(None)
        }
    }

    /// 插入决策到缓存
    pub fn insert(
        &self,
        key: DecisionCacheKey,
        decision: OptimizationDecision,
    ) -> Result<(), DecisionCacheError> {
        let cached = CachedDecision::new(decision);
        let cache = self.cache.lock();

        if cache.len() >= self.config.max_entries && !cache.contains(&key) {
            drop(cache);
            self.record_eviction();
        } else {
            drop(cache);
        }

        let mut cache = self.cache.lock();
        cache.push(key, cached);
        drop(cache);

        self.record_insert();
        Ok(())
    }

    /// 获取或计算决策
    pub fn get_or_compute<F>(
        &self,
        key: DecisionCacheKey,
        current_stats_version: u64,
        current_index_version: u64,
        compute: F,
    ) -> Result<OptimizationDecision, DecisionCacheError>
    where
        F: FnOnce() -> Result<OptimizationDecision, DecisionCacheError>,
    {
        // 尝试从缓存获取
        if let Some(decision) = self.get(&key, current_stats_version, current_index_version)? {
            return Ok(decision);
        }

        // 计算新决策
        let decision = compute()?;

        // 存入缓存
        self.insert(key, decision.clone())?;

        Ok(decision)
    }

    /// 移除缓存项
    pub fn remove(&self, key: &DecisionCacheKey) -> Result<(), DecisionCacheError> {
        let mut cache = self.cache.lock();
        cache.pop(key);
        Ok(())
    }

    /// 清空缓存
    pub fn clear(&self) -> Result<(), DecisionCacheError> {
        let mut cache = self.cache.lock();
        cache.clear();

        let mut stats = self.stats.lock();
        *stats = DecisionCacheStats::default();

        Ok(())
    }

    /// 获取缓存大小
    pub fn size(&self) -> usize {
        let cache = self.cache.lock();
        cache.len()
    }

    /// 获取统计信息
    pub fn stats(&self) -> DecisionCacheStats {
        let stats = self.stats.lock();
        stats.clone()
    }

    /// 使过期决策失效
    pub fn invalidate_outdated(
        &self,
        current_stats_version: u64,
        current_index_version: u64,
    ) -> Result<usize, DecisionCacheError> {
        let mut cache = self.cache.lock();
        let keys_to_remove: Vec<_> = cache
            .iter()
            .filter(|(_, cached)| !cached.decision.is_valid(current_stats_version, current_index_version))
            .map(|(key, _)| key.clone())
            .collect();

        let count = keys_to_remove.len();
        for key in keys_to_remove {
            cache.pop(&key);
        }

        Ok(count)
    }

    // ==================== 私有方法 ====================

    fn record_hit(&self) {
        if self.config.enable_stats {
            let mut stats = self.stats.lock();
            stats.hits += 1;
        }
    }

    fn record_miss(&self) {
        if self.config.enable_stats {
            let mut stats = self.stats.lock();
            stats.misses += 1;
        }
    }

    fn record_insert(&self) {
        if self.config.enable_stats {
            let mut stats = self.stats.lock();
            stats.inserts += 1;
        }
    }

    fn record_eviction(&self) {
        if self.config.enable_stats {
            let mut stats = self.stats.lock();
            stats.evictions += 1;
        }
    }

    fn record_version_mismatch(&self) {
        if self.config.enable_stats {
            let mut stats = self.stats.lock();
            stats.version_mismatches += 1;
        }
    }

    fn record_expiration(&self) {
        if self.config.enable_stats {
            let mut stats = self.stats.lock();
            stats.expirations += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::optimizer::decision::types::{
        AccessPath, EntityType, IndexSelectionDecision, JoinOrderDecision, TraversalStartDecision,
    };

    fn create_test_decision(stats_version: u64, index_version: u64) -> OptimizationDecision {
        OptimizationDecision::new(
            TraversalStartDecision::new(
                "n".to_string(),
                AccessPath::FullScan {
                    entity_type: EntityType::Vertex { tag_name: None },
                },
                1.0,
                1000.0,
            ),
            IndexSelectionDecision::empty(),
            JoinOrderDecision::empty(),
            stats_version,
            index_version,
        )
    }

    #[test]
    fn test_decision_cache_basic() {
        let cache = DecisionCache::with_default_config().expect("创建缓存失败");
        let key = DecisionCacheKey::new(12345, Some(1), SentenceKind::Match, None);
        let decision = create_test_decision(1, 1);

        // 插入
        cache.insert(key.clone(), decision.clone()).expect("插入失败");
        assert_eq!(cache.size(), 1);

        // 获取（版本匹配）
        let retrieved = cache.get(&key, 1, 1).expect("获取失败");
        assert!(retrieved.is_some());

        // 获取（版本不匹配）
        let retrieved = cache.get(&key, 2, 1).expect("获取失败");
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_decision_cache_stats() {
        let cache = DecisionCache::with_default_config().expect("创建缓存失败");
        let key = DecisionCacheKey::new(12345, Some(1), SentenceKind::Match, None);
        let decision = create_test_decision(1, 1);

        // 未命中
        let _ = cache.get(&key, 1, 1);

        // 插入
        cache.insert(key.clone(), decision).expect("插入失败");

        // 命中
        let _ = cache.get(&key, 1, 1);
        let _ = cache.get(&key, 1, 1);

        let stats = cache.stats();
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.inserts, 1);
    }

    #[test]
    fn test_get_or_compute() {
        let cache = DecisionCache::with_default_config().expect("创建缓存失败");
        let key = DecisionCacheKey::new(12345, Some(1), SentenceKind::Match, None);

        let compute_count = Arc::new(Mutex::new(0));
        let compute_count_clone = compute_count.clone();

        // 第一次调用，应该执行计算
        let decision1 = cache
            .get_or_compute(
                key.clone(),
                1,
                1,
                || {
                    *compute_count_clone.lock() += 1;
                    Ok(create_test_decision(1, 1))
                },
            )
            .expect("获取失败");

        // 第二次调用，应该使用缓存
        let decision2 = cache
            .get_or_compute(
                key.clone(),
                1,
                1,
                || {
                    *compute_count_clone.lock() += 1;
                    Ok(create_test_decision(1, 1))
                },
            )
            .expect("获取失败");

        assert_eq!(*compute_count.lock(), 1);
        assert_eq!(decision1.stats_version, decision2.stats_version);
    }
}
