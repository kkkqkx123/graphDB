//! Execution Statistics Context
//!
//! Used to collect execution statistics for all nodes during EXPLAIN ANALYZE and PROFILE operations.
//! Provides global statistics management and per-node statistics collection.

use std::collections::HashMap;
use std::time::Instant;

use parking_lot::Mutex;

use crate::core::stats::utils::micros_to_millis;
use crate::query::executor::base::ExecutorStats;

/// Node-level execution statistics
#[derive(Debug, Clone)]
pub struct NodeExecutionStats {
    pub node_id: i64,
    pub executor_stats: ExecutorStats,
    pub startup_time_us: u64,
}

impl NodeExecutionStats {
    pub fn new(node_id: i64) -> Self {
        Self {
            node_id,
            executor_stats: ExecutorStats::default(),
            startup_time_us: 0,
        }
    }

    pub fn actual_rows(&self) -> usize {
        self.executor_stats.num_rows
    }

    pub fn actual_time_us(&self) -> u64 {
        self.executor_stats.exec_time_us
    }

    pub fn actual_time_ms(&self) -> f64 {
        micros_to_millis(self.executor_stats.exec_time_us)
    }

    pub fn cache_hit_rate(&self) -> f64 {
        self.executor_stats.cache_hit_rate()
    }

    pub fn memory_used(&self) -> usize {
        self.executor_stats.memory_peak
    }

    pub fn io_reads(&self) -> usize {
        self.executor_stats.io_reads
    }

    pub fn io_read_bytes(&self) -> usize {
        self.executor_stats.io_read_bytes
    }

    pub fn io_writes(&self) -> usize {
        self.executor_stats.io_writes
    }

    pub fn io_write_bytes(&self) -> usize {
        self.executor_stats.io_write_bytes
    }

    pub fn total_io_ops(&self) -> usize {
        self.executor_stats.total_io_ops()
    }

    pub fn total_io_bytes(&self) -> usize {
        self.executor_stats.total_io_bytes()
    }
}

impl Default for NodeExecutionStats {
    fn default() -> Self {
        Self::new(0)
    }
}

/// Global execution statistics
#[derive(Debug, Clone, Default)]
pub struct GlobalExecutionStats {
    pub planning_time_ms: f64,
    pub execution_time_ms: f64,
    pub total_rows: usize,
    pub peak_memory: usize,
    pub cache_hit_rate: f64,
}

/// Execution statistics context
///
/// Manages statistics collection for all plan nodes during query execution.
/// Used by EXPLAIN ANALYZE and PROFILE to gather actual execution data.
pub struct ExecutionStatsContext {
    node_stats: Mutex<HashMap<i64, NodeExecutionStats>>,
    global_stats: Mutex<GlobalExecutionStats>,
    start_time: Instant,
}

impl ExecutionStatsContext {
    pub fn new() -> Self {
        Self {
            node_stats: Mutex::new(HashMap::new()),
            global_stats: Mutex::new(GlobalExecutionStats::default()),
            start_time: Instant::now(),
        }
    }

    pub fn with_planning_time(planning_time_ms: f64) -> Self {
        let ctx = Self::new();
        ctx.global_stats.lock().planning_time_ms = planning_time_ms;
        ctx
    }

    pub fn on_node_start(&self, node_id: i64) {
        let mut stats = self.node_stats.lock();
        stats
            .entry(node_id)
            .or_insert_with(|| NodeExecutionStats::new(node_id));
    }

    pub fn on_node_complete(&self, node_id: i64, executor_stats: ExecutorStats) {
        let mut stats = self.node_stats.lock();
        let node_stats = NodeExecutionStats {
            node_id,
            executor_stats,
            startup_time_us: 0,
        };
        stats.insert(node_id, node_stats);
    }

    pub fn record_node_rows(&self, node_id: i64, rows: usize) {
        let mut stats = self.node_stats.lock();
        if let Some(s) = stats.get_mut(&node_id) {
            s.executor_stats.num_rows = rows;
        }
    }

    pub fn record_node_time(&self, node_id: i64, time_us: u64) {
        let mut stats = self.node_stats.lock();
        if let Some(s) = stats.get_mut(&node_id) {
            s.executor_stats.exec_time_us = time_us;
        }
    }

    pub fn record_startup_time(&self, node_id: i64, startup_time_us: u64) {
        let mut stats = self.node_stats.lock();
        if let Some(s) = stats.get_mut(&node_id) {
            s.startup_time_us = startup_time_us;
        }
    }

    pub fn record_global_execution_time(&self, time_ms: f64) {
        self.global_stats.lock().execution_time_ms = time_ms;
    }

    pub fn collect_stats(&self) -> HashMap<i64, NodeExecutionStats> {
        self.node_stats.lock().clone()
    }

    pub fn get_node_stats(&self, node_id: i64) -> Option<NodeExecutionStats> {
        self.node_stats.lock().get(&node_id).cloned()
    }

    pub fn get_global_stats(&self) -> GlobalExecutionStats {
        self.global_stats.lock().clone()
    }

    pub fn total_elapsed_ms(&self) -> f64 {
        self.start_time.elapsed().as_micros() as f64 / 1000.0
    }
}

impl Default for ExecutionStatsContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_stats_context() {
        let ctx = ExecutionStatsContext::new();

        ctx.on_node_start(1);
        let mut exec_stats = ExecutorStats::default();
        exec_stats.num_rows = 100;
        exec_stats.exec_time_us = 5500;
        ctx.on_node_complete(1, exec_stats);

        let collected = ctx.collect_stats();
        assert_eq!(collected.get(&1).unwrap().actual_rows(), 100);
        assert!((collected.get(&1).unwrap().actual_time_ms() - 5.5).abs() < 0.001);
    }

    #[test]
    fn test_node_execution_stats_cache_rate() {
        let mut exec_stats = ExecutorStats::default();
        exec_stats.cache_hits = 90;
        exec_stats.cache_misses = 10;
        let stats = NodeExecutionStats {
            node_id: 0,
            executor_stats: exec_stats,
            startup_time_us: 0,
        };
        assert!((stats.cache_hit_rate() - 0.9).abs() < 0.001);
    }
}
