//! 存储指标收集器
//!
//! 收集存储引擎的性能指标，包括迭代器统计、缓存命中率等

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// 存储指标快照
#[derive(Debug, Clone, Default)]
pub struct StorageMetricsSnapshot {
    /// 扫描的项目数
    pub items_scanned: u64,
    /// 返回的项目数
    pub items_returned: u64,
    /// 缓存命中次数
    pub cache_hits: u64,
    /// 缓存未命中次数
    pub cache_misses: u64,
    /// 各操作类型的计数
    pub operation_counts: HashMap<String, u64>,
}

impl StorageMetricsSnapshot {
    pub fn new() -> Self {
        Self::default()
    }

    /// 计算缓存命中率
    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total > 0 {
            self.cache_hits as f64 / total as f64
        } else {
            0.0
        }
    }

    /// 计算扫描效率（返回数/扫描数）
    pub fn scan_efficiency(&self) -> f64 {
        if self.items_scanned > 0 {
            self.items_returned as f64 / self.items_scanned as f64
        } else {
            0.0
        }
    }
}

/// 存储指标收集器
#[derive(Debug)]
pub struct StorageMetricsCollector {
    /// 扫描的项目数
    items_scanned: AtomicU64,
    /// 返回的项目数
    items_returned: AtomicU64,
    /// 缓存命中次数
    cache_hits: AtomicU64,
    /// 缓存未命中次数
    cache_misses: AtomicU64,
    /// 各操作类型的计数
    operation_counts: dashmap::DashMap<String, AtomicU64>,
}

impl StorageMetricsCollector {
    pub fn new() -> Self {
        Self {
            items_scanned: AtomicU64::new(0),
            items_returned: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            operation_counts: dashmap::DashMap::new(),
        }
    }

    /// 记录扫描操作
    pub fn record_scan(&self, count: u64) {
        self.items_scanned.fetch_add(count, Ordering::Relaxed);
    }

    /// 记录返回操作
    pub fn record_return(&self, count: u64) {
        self.items_returned.fetch_add(count, Ordering::Relaxed);
    }

    /// 记录缓存命中
    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    /// 记录缓存未命中
    pub fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    /// 记录操作
    pub fn record_operation(&self, operation: &str) {
        let counter = self
            .operation_counts
            .entry(operation.to_string())
            .or_insert_with(|| AtomicU64::new(0));
        counter.fetch_add(1, Ordering::Relaxed);
    }

    /// 获取当前指标快照
    pub fn snapshot(&self) -> StorageMetricsSnapshot {
        let mut operation_counts = HashMap::new();
        for entry in self.operation_counts.iter() {
            operation_counts.insert(
                entry.key().clone(),
                entry.value().load(Ordering::Relaxed),
            );
        }

        StorageMetricsSnapshot {
            items_scanned: self.items_scanned.load(Ordering::Relaxed),
            items_returned: self.items_returned.load(Ordering::Relaxed),
            cache_hits: self.cache_hits.load(Ordering::Relaxed),
            cache_misses: self.cache_misses.load(Ordering::Relaxed),
            operation_counts,
        }
    }

    /// 重置所有指标
    pub fn reset(&self) {
        self.items_scanned.store(0, Ordering::Relaxed);
        self.items_returned.store(0, Ordering::Relaxed);
        self.cache_hits.store(0, Ordering::Relaxed);
        self.cache_misses.store(0, Ordering::Relaxed);
        self.operation_counts.clear();
    }
}

impl Default for StorageMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_metrics_collector() {
        let collector = StorageMetricsCollector::new();

        collector.record_scan(100);
        collector.record_return(50);
        collector.record_cache_hit();
        collector.record_cache_hit();
        collector.record_cache_miss();
        collector.record_operation("scan_vertices");
        collector.record_operation("scan_vertices");

        let snapshot = collector.snapshot();

        assert_eq!(snapshot.items_scanned, 100);
        assert_eq!(snapshot.items_returned, 50);
        assert_eq!(snapshot.cache_hits, 2);
        assert_eq!(snapshot.cache_misses, 1);
        assert_eq!(snapshot.operation_counts.get("scan_vertices"), Some(&2));
        assert!((snapshot.cache_hit_rate() - 0.666).abs() < 0.01);
        assert_eq!(snapshot.scan_efficiency(), 0.5);
    }

    #[test]
    fn test_reset() {
        let collector = StorageMetricsCollector::new();

        collector.record_scan(100);
        collector.record_cache_hit();

        collector.reset();

        let snapshot = collector.snapshot();
        assert_eq!(snapshot.items_scanned, 0);
        assert_eq!(snapshot.cache_hits, 0);
    }
}
