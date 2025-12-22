//! 缓存统计收集器
//!
//! 负责收集和管理缓存系统的统计信息

use std::sync::{Arc, RwLock};

/// 缓存统计信息
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub total_hits: u64,
    pub total_misses: u64,
    pub total_evictions: u64,
    pub total_operations: u64,
    pub memory_usage: usize,
    pub cache_count: usize,
}

impl CacheStats {
    /// 创建新的统计信息
    pub fn new() -> Self {
        Self::default()
    }

    /// 计算命中率
    pub fn hit_rate(&self) -> f64 {
        if self.total_hits + self.total_misses == 0 {
            0.0
        } else {
            self.total_hits as f64 / (self.total_hits + self.total_misses) as f64
        }
    }

    /// 重置统计信息
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// 合并统计信息
    pub fn merge(&mut self, other: &CacheStats) {
        self.total_hits += other.total_hits;
        self.total_misses += other.total_misses;
        self.total_evictions += other.total_evictions;
        self.total_operations += other.total_operations;
        self.memory_usage += other.memory_usage;
        self.cache_count += other.cache_count;
    }

    /// 记录命中
    pub fn record_hit(&mut self) {
        self.total_hits += 1;
        self.total_operations += 1;
    }

    /// 记录未命中
    pub fn record_miss(&mut self) {
        self.total_misses += 1;
        self.total_operations += 1;
    }

    /// 记录驱逐
    pub fn record_eviction(&mut self) {
        self.total_evictions += 1;
    }

    /// 记录内存使用
    pub fn record_memory_usage(&mut self, usage: usize) {
        self.memory_usage = usage;
    }

    /// 记录缓存数量
    pub fn record_cache_count(&mut self, count: usize) {
        self.cache_count = count;
    }

    /// 获取操作总数
    pub fn total_operations(&self) -> u64 {
        self.total_operations
    }

    /// 获取命中率百分比
    pub fn hit_rate_percentage(&self) -> f64 {
        self.hit_rate() * 100.0
    }

    /// 检查统计信息是否为空
    pub fn is_empty(&self) -> bool {
        self.total_operations == 0
    }
}

/// 缓存统计收集器
#[derive(Clone)]
pub struct CacheStatsCollector {
    stats: Arc<RwLock<CacheStats>>,
}

impl CacheStatsCollector {
    /// 创建新的统计收集器
    pub fn new() -> Self {
        Self {
            stats: Arc::new(RwLock::new(CacheStats::new())),
        }
    }

    /// 获取统计信息
    pub fn stats(&self) -> Arc<RwLock<CacheStats>> {
        self.stats.clone()
    }

    /// 记录命中
    pub fn record_hit(&self) {
        let mut stats = self.stats.write().expect("Stats write lock was poisoned");
        stats.record_hit();
    }

    /// 记录未命中
    pub fn record_miss(&self) {
        let mut stats = self.stats.write().expect("Stats write lock was poisoned");
        stats.record_miss();
    }

    /// 记录驱逐
    pub fn record_eviction(&self) {
        let mut stats = self.stats.write().expect("Stats write lock was poisoned");
        stats.record_eviction();
    }

    /// 记录内存使用
    pub fn record_memory_usage(&self, usage: usize) {
        let mut stats = self.stats.write().expect("Stats write lock was poisoned");
        stats.record_memory_usage(usage);
    }

    /// 记录缓存数量
    pub fn record_cache_count(&self, count: usize) {
        let mut stats = self.stats.write().expect("Stats write lock was poisoned");
        stats.record_cache_count(count);
    }

    /// 重置统计信息
    pub fn reset_stats(&self) {
        let mut stats = self.stats.write().expect("Stats write lock was poisoned");
        stats.reset();
    }

    /// 合并统计信息
    pub fn merge_stats(&self, other: &CacheStats) {
        let mut stats = self.stats.write().expect("Stats write lock was poisoned");
        stats.merge(other);
    }

    /// 获取当前统计信息快照
    pub fn snapshot(&self) -> CacheStats {
        let stats = self.stats.read().expect("Stats read lock was poisoned");
        stats.clone()
    }

    /// 获取命中率
    pub fn hit_rate(&self) -> f64 {
        let stats = self.stats.read().expect("Stats read lock was poisoned");
        stats.hit_rate()
    }

    /// 获取命中率百分比
    pub fn hit_rate_percentage(&self) -> f64 {
        let stats = self.stats.read().expect("Stats read lock was poisoned");
        stats.hit_rate_percentage()
    }

    /// 获取操作总数
    pub fn total_operations(&self) -> u64 {
        let stats = self.stats.read().expect("Stats read lock was poisoned");
        stats.total_operations()
    }

    /// 检查统计信息是否为空
    pub fn is_empty(&self) -> bool {
        let stats = self.stats.read().expect("Stats read lock was poisoned");
        stats.is_empty()
    }
}

impl Default for CacheStatsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for CacheStatsCollector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let stats = self.stats.read().expect("Stats read lock was poisoned");
        f.debug_struct("CacheStatsCollector")
            .field("total_operations", &stats.total_operations)
            .field("hit_rate", &stats.hit_rate())
            .field("cache_count", &stats.cache_count)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_stats_basic() {
        let mut stats = CacheStats::new();
        assert_eq!(stats.hit_rate(), 0.0);
        assert_eq!(stats.total_operations(), 0);
        assert!(stats.is_empty());

        stats.record_hit();
        stats.record_miss();
        stats.record_eviction();
        stats.record_memory_usage(1024);
        stats.record_cache_count(5);

        assert_eq!(stats.total_hits, 1);
        assert_eq!(stats.total_misses, 1);
        assert_eq!(stats.total_evictions, 1);
        assert_eq!(stats.total_operations(), 2);
        assert_eq!(stats.memory_usage, 1024);
        assert_eq!(stats.cache_count, 5);
        assert_eq!(stats.hit_rate(), 0.5);
        assert_eq!(stats.hit_rate_percentage(), 50.0);
        assert!(!stats.is_empty());
    }

    #[test]
    fn test_cache_stats_reset() {
        let mut stats = CacheStats::new();
        stats.record_hit();
        stats.record_miss();

        assert_eq!(stats.total_operations(), 2);
        assert!(!stats.is_empty());

        stats.reset();
        assert_eq!(stats.total_operations(), 0);
        assert!(stats.is_empty());
    }

    #[test]
    fn test_cache_stats_merge() {
        let mut stats1 = CacheStats::new();
        stats1.record_hit();
        stats1.record_miss();

        let mut stats2 = CacheStats::new();
        stats2.record_hit();
        stats2.record_hit();

        stats1.merge(&stats2);

        assert_eq!(stats1.total_hits, 3);
        assert_eq!(stats1.total_misses, 1);
        assert_eq!(stats1.total_operations(), 4);
        assert_eq!(stats1.hit_rate(), 0.75);
    }

    #[test]
    fn test_cache_stats_collector_basic() {
        let collector = CacheStatsCollector::new();

        assert!(collector.is_empty());
        assert_eq!(collector.total_operations(), 0);
        assert_eq!(collector.hit_rate(), 0.0);

        collector.record_hit();
        collector.record_miss();
        collector.record_eviction();
        collector.record_memory_usage(2048);
        collector.record_cache_count(10);

        assert!(!collector.is_empty());
        assert_eq!(collector.total_operations(), 2);
        assert_eq!(collector.hit_rate(), 0.5);
        assert_eq!(collector.hit_rate_percentage(), 50.0);

        let snapshot = collector.snapshot();
        assert_eq!(snapshot.total_hits, 1);
        assert_eq!(snapshot.total_misses, 1);
        assert_eq!(snapshot.total_evictions, 1);
        assert_eq!(snapshot.memory_usage, 2048);
        assert_eq!(snapshot.cache_count, 10);
    }

    #[test]
    fn test_cache_stats_collector_reset() {
        let collector = CacheStatsCollector::new();

        collector.record_hit();
        collector.record_miss();
        assert!(!collector.is_empty());

        collector.reset_stats();
        assert!(collector.is_empty());
        assert_eq!(collector.total_operations(), 0);
    }

    #[test]
    fn test_cache_stats_collector_merge() {
        let collector1 = CacheStatsCollector::new();
        collector1.record_hit();
        collector1.record_miss();

        let collector2 = CacheStatsCollector::new();
        collector2.record_hit();
        collector2.record_hit();

        let stats2 = collector2.snapshot();
        collector1.merge_stats(&stats2);

        assert_eq!(collector1.total_operations(), 4);
        assert_eq!(collector1.hit_rate(), 0.75);
    }

    #[test]
    fn test_cache_stats_collector_debug() {
        let collector = CacheStatsCollector::new();
        collector.record_hit();
        collector.record_miss();

        let debug_output = format!("{:?}", collector);
        assert!(debug_output.contains("CacheStatsCollector"));
        assert!(debug_output.contains("total_operations"));
        assert!(debug_output.contains("hit_rate"));
    }
}
