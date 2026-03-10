//! CTE结果缓存管理器模块
//!
//! 提供CTE（Common Table Expression）查询结果的缓存功能，
//! 避免重复计算相同的CTE，提升查询性能。
//!
//! ## 缓存策略
//!
//! - LRU淘汰策略：当缓存满时淘汰最久未使用的条目
//! - 内存预算管理：严格控制缓存使用的内存上限
//! - 智能缓存决策：基于CTE特性决定是否缓存
//!
//! ## 适用场景
//!
//! 1. 递归CTE被多次引用
//! 2. 复杂子查询在单个查询中被多次使用
//! 3. 结果集大小适中（100-10000行）
//! 4. CTE是确定性的（不含随机函数等）

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// CTE缓存条目
#[derive(Debug, Clone)]
pub struct CteCacheEntry {
    /// 结果数据（使用Arc共享）
    pub data: Arc<Vec<u8>>,
    /// 结果行数
    pub row_count: u64,
    /// 结果大小（字节）
    pub data_size: usize,
    /// 创建时间
    pub created_at: Instant,
    /// 最后访问时间
    pub last_accessed: Instant,
    /// 访问次数
    pub access_count: u64,
    /// 估计重用概率
    pub reuse_probability: f64,
    /// CTE定义哈希（用于识别相同的CTE）
    pub cte_hash: String,
    /// CTE定义文本
    pub cte_definition: String,
}

impl CteCacheEntry {
    /// 创建新的缓存条目
    pub fn new(cte_hash: String, cte_definition: String, data: Vec<u8>, row_count: u64) -> Self {
        let data_size = data.len();
        Self {
            data: Arc::new(data),
            row_count,
            data_size,
            created_at: Instant::now(),
            last_accessed: Instant::now(),
            access_count: 0,
            reuse_probability: 0.5,
            cte_hash,
            cte_definition,
        }
    }

    /// 记录访问
    pub fn record_access(&mut self) {
        self.last_accessed = Instant::now();
        self.access_count += 1;
        // 更新重用概率：访问次数越多，重用概率越高
        self.reuse_probability = (self.reuse_probability * 0.7 + 0.3).min(0.95);
    }

    /// 获取缓存年龄
    pub fn age(&self) -> Duration {
        self.created_at.elapsed()
    }

    /// 获取空闲时间
    pub fn idle_time(&self) -> Duration {
        self.last_accessed.elapsed()
    }

    /// 计算缓存得分（用于LRU淘汰决策）
    /// 得分越低越容易被淘汰
    pub fn cache_score(&self) -> f64 {
        let _age_factor = self.age().as_secs_f64() / 3600.0; // 以小时为单位
        let idle_factor = self.idle_time().as_secs_f64() / 60.0; // 以分钟为单位
        let size_factor = (self.data_size as f64 / 1024.0 / 1024.0).max(0.1); // MB为单位
        let access_factor = (self.access_count as f64).sqrt().max(1.0);

        // 综合得分：考虑空闲时间、大小、访问频率
        (idle_factor * size_factor) / (access_factor * self.reuse_probability)
    }
}

/// CTE缓存统计信息
#[derive(Debug, Clone, Default)]
pub struct CteCacheStats {
    /// 缓存命中次数
    pub hit_count: u64,
    /// 缓存未命中次数
    pub miss_count: u64,
    /// 缓存条目数量
    pub entry_count: usize,
    /// 当前使用内存（字节）
    pub current_memory: usize,
    /// 总内存上限（字节）
    pub max_memory: usize,
    /// 淘汰的条目数量
    pub evicted_count: u64,
    /// 被拒绝缓存的条目数量
    pub rejected_count: u64,
}

impl CteCacheStats {
    /// 获取命中率
    pub fn hit_rate(&self) -> f64 {
        let total = self.hit_count + self.miss_count;
        if total == 0 {
            return 0.0;
        }
        self.hit_count as f64 / total as f64
    }

    /// 获取内存使用率
    pub fn memory_usage_ratio(&self) -> f64 {
        if self.max_memory == 0 {
            return 0.0;
        }
        self.current_memory as f64 / self.max_memory as f64
    }

    /// 重置统计
    pub fn reset(&mut self) {
        self.hit_count = 0;
        self.miss_count = 0;
        self.evicted_count = 0;
        self.rejected_count = 0;
    }
}

/// CTE缓存配置
#[derive(Debug, Clone)]
pub struct CteCacheConfig {
    /// 最大缓存大小（字节）
    pub max_size: usize,
    /// 单个条目最大大小（字节）
    pub max_entry_size: usize,
    /// 最小缓存行数（小于此值不缓存）
    pub min_row_count: u64,
    /// 最大缓存行数（大于此值不缓存）
    pub max_row_count: u64,
    /// 条目过期时间（秒）
    pub entry_ttl_seconds: u64,
    /// 启用缓存
    pub enabled: bool,
}

impl Default for CteCacheConfig {
    fn default() -> Self {
        Self {
            max_size: 64 * 1024 * 1024,       // 64MB
            max_entry_size: 10 * 1024 * 1024, // 10MB
            min_row_count: 100,               // 至少100行
            max_row_count: 100_000,           // 最多10万行
            entry_ttl_seconds: 3600,          // 1小时
            enabled: true,
        }
    }
}

impl CteCacheConfig {
    /// 创建小内存配置
    pub fn low_memory() -> Self {
        Self {
            max_size: 16 * 1024 * 1024,       // 16MB
            max_entry_size: 5 * 1024 * 1024,  // 5MB
            min_row_count: 50,
            max_row_count: 50_000,
            entry_ttl_seconds: 1800,          // 30分钟
            enabled: true,
        }
    }

    /// 创建大内存配置
    pub fn high_memory() -> Self {
        Self {
            max_size: 256 * 1024 * 1024,      // 256MB
            max_entry_size: 50 * 1024 * 1024, // 50MB
            min_row_count: 100,
            max_row_count: 500_000,
            entry_ttl_seconds: 7200,          // 2小时
            enabled: true,
        }
    }

    /// 禁用缓存
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Default::default()
        }
    }
}

/// CTE缓存管理器
///
/// 管理CTE查询结果的缓存，提供线程安全的访问
#[derive(Debug)]
pub struct CteCacheManager {
    /// 缓存存储
    cache: RwLock<HashMap<String, CteCacheEntry>>,
    /// 配置
    config: RwLock<CteCacheConfig>,
    /// 统计信息
    stats: RwLock<CteCacheStats>,
    /// 当前使用内存
    current_memory: RwLock<usize>,
}

impl CteCacheManager {
    /// 创建新的缓存管理器
    pub fn new() -> Self {
        Self::with_config(CteCacheConfig::default())
    }

    /// 使用配置创建
    pub fn with_config(config: CteCacheConfig) -> Self {
        let max_memory = config.max_size;
        Self {
            cache: RwLock::new(HashMap::new()),
            config: RwLock::new(config),
            stats: RwLock::new(CteCacheStats {
                max_memory,
                ..Default::default()
            }),
            current_memory: RwLock::new(0),
        }
    }

    /// 获取配置
    pub fn config(&self) -> CteCacheConfig {
        self.config.read().clone()
    }

    /// 更新配置
    pub fn set_config(&self, config: CteCacheConfig) {
        let mut stats = self.stats.write();
        stats.max_memory = config.max_size;
        *self.config.write() = config;

        // 如果新配置更小，可能需要淘汰一些条目
        self.evict_if_needed();
    }

    /// 判断是否启用缓存
    pub fn is_enabled(&self) -> bool {
        self.config.read().enabled
    }

    /// 判断是否缓存CTE结果
    ///
    /// # 参数
    /// - `cte_definition`: CTE定义文本
    /// - `estimated_rows`: 估计行数
    /// - `is_deterministic`: 是否确定性CTE
    pub fn should_cache(&self, cte_definition: &str, estimated_rows: u64, is_deterministic: bool) -> bool {
        let config = self.config.read();

        if !config.enabled {
            return false;
        }

        if !is_deterministic {
            return false;
        }

        // 检查行数范围
        if estimated_rows < config.min_row_count || estimated_rows > config.max_row_count {
            return false;
        }

        // 检查历史重用模式
        let reuse_prob = self.predict_reuse_probability(cte_definition);
        if reuse_prob < 0.3 {
            return false;
        }

        true
    }

    /// 预测重用概率
    fn predict_reuse_probability(&self, cte_definition: &str) -> f64 {
        let cache = self.cache.read();
        let cte_hash = Self::compute_hash(cte_definition);

        // 如果已经在缓存中，返回当前的重用概率
        if let Some(entry) = cache.get(&cte_hash) {
            return entry.reuse_probability;
        }

        // 否则基于CTE特征进行预测
        // 简单的启发式：复杂的CTE更可能被重用
        let complexity = cte_definition.len() as f64 / 100.0;
        let base_prob = 0.5;
        let complexity_bonus = (complexity / 10.0).min(0.3);

        base_prob + complexity_bonus
    }

    /// 计算CTE定义的哈希值
    fn compute_hash(cte_definition: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        cte_definition.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }

    /// 将数据存入缓存
    pub fn put(&self, cte_definition: &str, data: Vec<u8>, row_count: u64) -> Option<String> {
        let config = self.config.read();

        if !config.enabled {
            return None;
        }

        // 检查数据大小
        if data.len() > config.max_entry_size {
            let mut stats = self.stats.write();
            stats.rejected_count += 1;
            return None;
        }

        drop(config);

        // 确保有足够空间
        self.evict_if_needed();

        let cte_hash = Self::compute_hash(cte_definition);
        let entry = CteCacheEntry::new(
            cte_hash.clone(),
            cte_definition.to_string(),
            data,
            row_count,
        );

        let entry_size = entry.data_size;
        let mut cache = self.cache.write();

        // 更新内存使用
        *self.current_memory.write() += entry_size;

        // 插入缓存
        cache.insert(cte_hash.clone(), entry);

        // 更新统计
        let mut stats = self.stats.write();
        stats.entry_count = cache.len();
        stats.current_memory = *self.current_memory.read();

        Some(cte_hash)
    }

    /// 从缓存获取数据
    pub fn get(&self, cte_definition: &str) -> Option<Arc<Vec<u8>>> {
        let config = self.config.read();

        if !config.enabled {
            return None;
        }

        drop(config);

        let cte_hash = Self::compute_hash(cte_definition);
        let mut cache = self.cache.write();

        if let Some(entry) = cache.get_mut(&cte_hash) {
            // 检查是否过期
            let config = self.config.read();
            if entry.age().as_secs() > config.entry_ttl_seconds {
                // 过期，移除
                let size = entry.data_size;
                cache.remove(&cte_hash);
                *self.current_memory.write() -= size;

                let mut stats = self.stats.write();
                stats.miss_count += 1;
                stats.entry_count = cache.len();
                stats.current_memory = *self.current_memory.read();
                return None;
            }

            // 记录访问
            entry.record_access();

            // 更新统计
            let mut stats = self.stats.write();
            stats.hit_count += 1;

            Some(entry.data.clone())
        } else {
            let mut stats = self.stats.write();
            stats.miss_count += 1;
            None
        }
    }

    /// 检查缓存中是否存在
    pub fn contains(&self, cte_definition: &str) -> bool {
        let cte_hash = Self::compute_hash(cte_definition);
        self.cache.read().contains_key(&cte_hash)
    }

    /// 使缓存条目失效
    pub fn invalidate(&self, cte_definition: &str) -> bool {
        let cte_hash = Self::compute_hash(cte_definition);
        let mut cache = self.cache.write();

        if let Some(entry) = cache.remove(&cte_hash) {
            *self.current_memory.write() -= entry.data_size;

            let mut stats = self.stats.write();
            stats.entry_count = cache.len();
            stats.current_memory = *self.current_memory.read();
            true
        } else {
            false
        }
    }

    /// 清空所有缓存
    pub fn clear(&self) {
        let mut cache = self.cache.write();
        cache.clear();
        *self.current_memory.write() = 0;

        let mut stats = self.stats.write();
        stats.entry_count = 0;
        stats.current_memory = 0;
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> CteCacheStats {
        let mut stats = self.stats.read().clone();
        stats.entry_count = self.cache.read().len();
        stats.current_memory = *self.current_memory.read();
        stats
    }

    /// 重置统计
    pub fn reset_stats(&self) {
        self.stats.write().reset();
    }

    /// 获取当前内存使用
    pub fn current_memory(&self) -> usize {
        *self.current_memory.read()
    }

    /// 获取缓存条目数量
    pub fn entry_count(&self) -> usize {
        self.cache.read().len()
    }

    /// 如果需要，执行淘汰
    fn evict_if_needed(&self) {
        let config = self.config.read();
        let max_size = config.max_size;
        drop(config);

        let mut current = *self.current_memory.read();
        let mut evicted = 0u64;

        while current > max_size && current > 0 {
            // 找到得分最低的条目进行淘汰
            let to_evict = {
                let cache = self.cache.read();
                cache
                    .iter()
                    .min_by(|a, b| {
                        a.1.cache_score()
                            .partial_cmp(&b.1.cache_score())
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .map(|(k, _)| k.clone())
            };

            if let Some(key) = to_evict {
                let mut cache = self.cache.write();
                if let Some(entry) = cache.remove(&key) {
                    current -= entry.data_size;
                    evicted += 1;
                }
            } else {
                break;
            }
        }

        if evicted > 0 {
            *self.current_memory.write() = current;

            let mut stats = self.stats.write();
            stats.evicted_count += evicted;
            stats.entry_count = self.cache.read().len();
            stats.current_memory = current;
        }
    }

    /// 清理过期条目
    pub fn cleanup_expired(&self) -> usize {
        let config = self.config.read();
        let ttl = config.entry_ttl_seconds;
        drop(config);

        let mut cache = self.cache.write();
        let now = Instant::now();
        let to_remove: Vec<String> = cache
            .iter()
            .filter(|(_, entry)| now.duration_since(entry.created_at).as_secs() > ttl)
            .map(|(k, _)| k.clone())
            .collect();

        let mut freed_memory = 0usize;
        for key in &to_remove {
            if let Some(entry) = cache.remove(key) {
                freed_memory += entry.data_size;
            }
        }

        if freed_memory > 0 {
            *self.current_memory.write() -= freed_memory;

            let mut stats = self.stats.write();
            stats.entry_count = cache.len();
            stats.current_memory = *self.current_memory.read();
        }

        to_remove.len()
    }
}

impl Default for CteCacheManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for CteCacheManager {
    fn clone(&self) -> Self {
        Self {
            cache: RwLock::new(self.cache.read().clone()),
            config: RwLock::new(self.config.read().clone()),
            stats: RwLock::new(self.stats.read().clone()),
            current_memory: RwLock::new(*self.current_memory.read()),
        }
    }
}

/// CTE缓存决策器
///
/// 基于查询特征决定是否使用缓存
#[derive(Debug, Clone)]
pub struct CteCacheDecision {
    /// 是否使用缓存
    pub should_cache: bool,
    /// 决策原因
    pub reason: String,
    /// 估计重用概率
    pub reuse_probability: f64,
    /// 估计缓存收益
    pub estimated_benefit: f64,
}

/// CTE缓存决策器
#[derive(Debug)]
pub struct CteCacheDecisionMaker {
    /// 缓存管理器
    cache_manager: Arc<CteCacheManager>,
    /// 最小重用概率阈值
    min_reuse_probability: f64,
    /// 最小估计收益
    min_benefit: f64,
}

impl CteCacheDecisionMaker {
    /// 创建新的决策器
    pub fn new(cache_manager: Arc<CteCacheManager>) -> Self {
        Self {
            cache_manager,
            min_reuse_probability: 0.3,
            min_benefit: 1.0,
        }
    }

    /// 设置参数
    pub fn with_params(mut self, min_reuse_probability: f64, min_benefit: f64) -> Self {
        self.min_reuse_probability = min_reuse_probability;
        self.min_benefit = min_benefit;
        self
    }

    /// 做出缓存决策
    pub fn decide(&self, cte_definition: &str, estimated_rows: u64, compute_cost: f64) -> CteCacheDecision {
        if !self.cache_manager.is_enabled() {
            return CteCacheDecision {
                should_cache: false,
                reason: "缓存已禁用".to_string(),
                reuse_probability: 0.0,
                estimated_benefit: 0.0,
            };
        }

        let reuse_prob = self.cache_manager.predict_reuse_probability(cte_definition);

        if reuse_prob < self.min_reuse_probability {
            return CteCacheDecision {
                should_cache: false,
                reason: format!("重用概率过低: {:.2}", reuse_prob),
                reuse_probability: reuse_prob,
                estimated_benefit: 0.0,
            };
        }

        // 估计缓存收益 = 重用概率 * 计算代价 - 缓存开销
        let cache_overhead = estimated_rows as f64 * 0.001; // 假设每行缓存开销0.001ms
        let estimated_benefit = reuse_prob * compute_cost - cache_overhead;

        if estimated_benefit < self.min_benefit {
            return CteCacheDecision {
                should_cache: false,
                reason: format!("估计收益过低: {:.2}", estimated_benefit),
                reuse_probability: reuse_prob,
                estimated_benefit,
            };
        }

        CteCacheDecision {
            should_cache: true,
            reason: "收益分析通过".to_string(),
            reuse_probability: reuse_prob,
            estimated_benefit,
        }
    }
}

impl Default for CteCacheDecisionMaker {
    fn default() -> Self {
        Self::new(Arc::new(CteCacheManager::new()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cte_cache_entry() {
        let mut entry = CteCacheEntry::new(
            "hash1".to_string(),
            "SELECT * FROM t".to_string(),
            vec![1, 2, 3, 4, 5],
            10,
        );

        assert_eq!(entry.row_count, 10);
        assert_eq!(entry.data_size, 5);

        entry.record_access();
        assert_eq!(entry.access_count, 1);
        assert!(entry.reuse_probability > 0.5);
    }

    #[test]
    fn test_cte_cache_manager() {
        let manager = CteCacheManager::new();

        // 测试缓存决策
        assert!(manager.should_cache("SELECT * FROM t", 500, true));
        assert!(!manager.should_cache("SELECT * FROM t", 10, true)); // 行数太少
        assert!(!manager.should_cache("SELECT * FROM t", 500, false)); // 非确定性

        // 测试存入和获取
        let data = vec![1, 2, 3, 4, 5];
        let key = manager.put("SELECT * FROM t", data.clone(), 100);
        assert!(key.is_some());

        let retrieved = manager.get("SELECT * FROM t");
        assert!(retrieved.is_some());
        assert_eq!(*retrieved.unwrap(), data);

        // 测试统计
        let stats = manager.get_stats();
        assert_eq!(stats.hit_count, 1);
        assert_eq!(stats.miss_count, 1); // 之前未命中一次
        assert_eq!(stats.entry_count, 1);
    }

    #[test]
    fn test_cte_cache_eviction() {
        let config = CteCacheConfig {
            max_size: 100, // 很小的缓存
            max_entry_size: 50,
            min_row_count: 1,
            max_row_count: 1000,
            entry_ttl_seconds: 3600,
            enabled: true,
        };

        let manager = CteCacheManager::with_config(config);

        // 存入多个条目，触发淘汰
        let data1 = vec![1u8; 40]; // 40字节
        let data2 = vec![2u8; 40]; // 40字节
        let data3 = vec![3u8; 40]; // 40字节

        manager.put("query1", data1, 10);
        manager.put("query2", data2, 10);

        // 访问query1，提升其得分
        manager.get("query1");

        // 存入query3，应该淘汰query2
        manager.put("query3", data3, 10);

        let stats = manager.get_stats();
        assert!(stats.evicted_count >= 1);
    }

    #[test]
    fn test_cte_cache_decision_maker() {
        let manager = Arc::new(CteCacheManager::new());
        let decision_maker = CteCacheDecisionMaker::new(manager);

        // 测试决策
        let decision = decision_maker.decide("SELECT * FROM large_table", 1000, 100.0);
        // 由于重用概率可能较低，结果可能是true或false
        assert!(decision.reuse_probability >= 0.0 && decision.reuse_probability <= 1.0);

        // 测试低重用概率的情况
        let decision = decision_maker.decide("SELECT 1", 100, 0.1);
        assert!(!decision.should_cache); // 简单查询不应缓存
    }

    #[test]
    fn test_cte_cache_stats() {
        let mut stats = CteCacheStats::default();

        assert_eq!(stats.hit_rate(), 0.0);
        assert_eq!(stats.memory_usage_ratio(), 0.0);

        stats.hit_count = 80;
        stats.miss_count = 20;
        stats.current_memory = 50;
        stats.max_memory = 100;

        assert_eq!(stats.hit_rate(), 0.8);
        assert_eq!(stats.memory_usage_ratio(), 0.5);

        stats.reset();
        assert_eq!(stats.hit_count, 0);
        assert_eq!(stats.miss_count, 0);
    }
}
