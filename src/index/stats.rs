//! 索引统计信息模块
//!
//! 提供索引查询统计和性能监控功能，复用 cache 模块的统计收集器
//!
//! 功能：
//! - 查询计数和命中率统计
//! - 查询延迟监控
//! - 索引使用情况报告
//! - 与全局缓存统计集成

use crate::cache::{Stats, StatsCollector};
use std::sync::RwLock;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryType {
    Exact,
    Range,
    Prefix,
}

#[derive(Debug, Clone)]
pub struct IndexQueryStats {
    pub query_count: Arc<AtomicU64>,
    pub hit_count: Arc<AtomicU64>,
    pub miss_count: Arc<AtomicU64>,
    pub total_query_time_ns: Arc<AtomicU64>,
    pub prefix_query_count: Arc<AtomicU64>,
    pub range_query_count: Arc<AtomicU64>,
    pub exact_query_count: Arc<AtomicU64>,
    pub entry_count: Arc<AtomicU64>,
}

impl Default for IndexQueryStats {
    fn default() -> Self {
        Self {
            query_count: Arc::new(AtomicU64::new(0)),
            hit_count: Arc::new(AtomicU64::new(0)),
            miss_count: Arc::new(AtomicU64::new(0)),
            total_query_time_ns: Arc::new(AtomicU64::new(0)),
            prefix_query_count: Arc::new(AtomicU64::new(0)),
            range_query_count: Arc::new(AtomicU64::new(0)),
            exact_query_count: Arc::new(AtomicU64::new(0)),
            entry_count: Arc::new(AtomicU64::new(0)),
        }
    }
}

impl IndexQueryStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_query(&self, found: bool, duration: Duration, query_type: QueryType) {
        self.query_count.fetch_add(1, Ordering::Relaxed);
        self.total_query_time_ns.fetch_add(duration.as_nanos() as u64, Ordering::Relaxed);

        match query_type {
            QueryType::Exact => { self.exact_query_count.fetch_add(1, Ordering::Relaxed); }
            QueryType::Range => { self.range_query_count.fetch_add(1, Ordering::Relaxed); }
            QueryType::Prefix => { self.prefix_query_count.fetch_add(1, Ordering::Relaxed); }
        }

        if found {
            self.hit_count.fetch_add(1, Ordering::Relaxed);
        } else {
            self.miss_count.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn hit_rate(&self) -> f64 {
        let hits = self.hit_count.load(Ordering::Relaxed);
        let total = self.query_count.load(Ordering::Relaxed);
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }

    pub fn avg_query_time_ns(&self) -> f64 {
        let total = self.total_query_time_ns.load(Ordering::Relaxed);
        let count = self.query_count.load(Ordering::Relaxed);
        if count == 0 {
            0.0
        } else {
            total as f64 / count as f64
        }
    }

    pub fn reset(&self) {
        self.query_count.store(0, Ordering::Relaxed);
        self.hit_count.store(0, Ordering::Relaxed);
        self.miss_count.store(0, Ordering::Relaxed);
        self.total_query_time_ns.store(0, Ordering::Relaxed);
        self.prefix_query_count.store(0, Ordering::Relaxed);
        self.range_query_count.store(0, Ordering::Relaxed);
        self.exact_query_count.store(0, Ordering::Relaxed);
    }

    pub fn to_stats(&self) -> Stats {
        Stats {
            total_hits: self.hit_count.load(Ordering::Relaxed),
            total_misses: self.miss_count.load(Ordering::Relaxed),
            total_evictions: 0,
            total_operations: self.query_count.load(Ordering::Relaxed),
            memory_usage: 0,
            item_count: self.entry_count.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone)]
pub struct IndexStats {
    pub index_id: i32,
    pub index_name: String,
    pub query_stats: IndexQueryStats,
    pub entry_count: Arc<AtomicU64>,
    pub memory_usage_bytes: Arc<AtomicU64>,
    pub last_accessed: Arc<RwLock<Instant>>,
}

impl IndexStats {
    pub fn new(index_id: i32, index_name: String) -> Self {
        Self {
            index_id,
            index_name,
            query_stats: IndexQueryStats::new(),
            entry_count: Arc::new(AtomicU64::new(0)),
            memory_usage_bytes: Arc::new(AtomicU64::new(0)),
            last_accessed: Arc::new(RwLock::new(Instant::now())),
        }
    }

    pub fn touch(&self) {
        let mut last = self.last_accessed.write().unwrap();
        *last = Instant::now();
    }

    pub fn is_recently_used(&self, threshold: Duration) -> bool {
        let last = self.last_accessed.read().unwrap();
        Instant::now().duration_since(*last) < threshold
    }
}

pub struct IndexStatsManager {
    global_stats: Arc<IndexQueryStats>,
    per_index_stats: Arc<RwLock<HashMap<i32, IndexStats>>>,
    stats_collector: Arc<RwLock<StatsCollector>>,
}

impl Default for IndexStatsManager {
    fn default() -> Self {
        Self::new()
    }
}

impl IndexStatsManager {
    pub fn new() -> Self {
        Self {
            global_stats: Arc::new(IndexQueryStats::new()),
            per_index_stats: Arc::new(RwLock::new(HashMap::new())),
            stats_collector: Arc::new(RwLock::new(StatsCollector::new())),
        }
    }

    pub fn register_index(&self, index_id: i32, index_name: String) {
        let mut stats = self.per_index_stats.write().unwrap();
        if !stats.contains_key(&index_id) {
            stats.insert(index_id, IndexStats::new(index_id, index_name));
        }
    }

    pub fn unregister_index(&self, index_id: i32) {
        let mut stats = self.per_index_stats.write().unwrap();
        stats.remove(&index_id);
    }

    pub fn get_index_stats(&self, index_id: i32) -> Option<IndexStats> {
        let stats = self.per_index_stats.read().unwrap();
        stats.get(&index_id).cloned()
    }

    pub fn record_query(&self, index_id: i32, found: bool, duration: Duration, query_type: QueryType) {
        self.global_stats.record_query(found, duration, query_type);
        
        if let Some(stats) = self.get_index_stats(index_id) {
            stats.query_stats.record_query(found, duration, query_type);
            stats.touch();
        }
    }

    pub fn update_entry_count(&self, index_id: i32, count: u64) {
        if let Some(stats) = self.get_index_stats(index_id) {
            stats.entry_count.store(count, Ordering::Relaxed);
            stats.query_stats.entry_count.store(count, Ordering::Relaxed);
        }
    }

    pub fn update_memory_usage(&self, index_id: i32, bytes: u64) {
        if let Some(stats) = self.get_index_stats(index_id) {
            stats.memory_usage_bytes.store(bytes, Ordering::Relaxed);
        }
    }

    pub fn global_hit_rate(&self) -> f64 {
        self.global_stats.hit_rate()
    }

    pub fn global_avg_query_time_ms(&self) -> f64 {
        self.global_stats.avg_query_time_ns() / 1_000_000.0
    }

    pub fn get_all_index_stats(&self) -> Vec<IndexStats> {
        let stats = self.per_index_stats.read().unwrap();
        stats.values().cloned().collect()
    }

    pub fn sync_to_stats(&self) {
        let mut collector = self.stats_collector.write().unwrap();
        let global_stats = self.global_stats.to_stats();
        collector.merge_stats(&global_stats);
    }

    pub fn get_stats(&self) -> Stats {
        self.stats_collector.read().unwrap().snapshot()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_query_stats() {
        let stats = IndexQueryStats::new();
        
        stats.record_query(true, Duration::from_micros(100), QueryType::Exact);
        stats.record_query(false, Duration::from_micros(50), QueryType::Range);
        stats.record_query(true, Duration::from_micros(200), QueryType::Prefix);

        assert_eq!(stats.query_count.load(Ordering::Relaxed), 3);
        assert_eq!(stats.hit_count.load(Ordering::Relaxed), 2);
        assert_eq!(stats.miss_count.load(Ordering::Relaxed), 1);
        assert_eq!(stats.exact_query_count.load(Ordering::Relaxed), 1);
        assert_eq!(stats.range_query_count.load(Ordering::Relaxed), 1);
        assert_eq!(stats.prefix_query_count.load(Ordering::Relaxed), 1);

        assert!((stats.hit_rate() - 2.0/3.0).abs() < 0.01);
    }

    #[test]
    fn test_index_stats_manager() {
        let manager = IndexStatsManager::new();

        manager.register_index(1, "test_index".to_string());

        manager.record_query(1, true, Duration::from_micros(100), QueryType::Exact);
        manager.record_query(1, false, Duration::from_micros(50), QueryType::Range);

        let index_stats = manager.get_index_stats(1).expect("Failed to get index stats for test");
        assert_eq!(index_stats.query_stats.query_count.load(Ordering::Relaxed), 2);
        assert_eq!(index_stats.query_stats.hit_count.load(Ordering::Relaxed), 1);

        assert!((manager.global_hit_rate() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_stats_collector_conversion() {
        let stats = IndexQueryStats::new();
        stats.record_query(true, Duration::from_micros(100), QueryType::Exact);
        stats.record_query(false, Duration::from_micros(50), QueryType::Range);

        let converted = stats.to_stats();
        assert_eq!(converted.total_hits, 1);
        assert_eq!(converted.total_misses, 1);
        assert_eq!(converted.total_operations, 2);
    }
}
