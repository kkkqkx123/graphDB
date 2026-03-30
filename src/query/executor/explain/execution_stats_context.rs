//! Execution Statistics Context
//!
//! Used to collect execution statistics for all nodes during EXPLAIN ANALYZE and PROFILE operations.
//! Provides global statistics management and per-node statistics collection.

use std::collections::HashMap;
use std::time::Instant;

use parking_lot::Mutex;

/// Node-level execution statistics
#[derive(Debug, Clone, Default)]
pub struct NodeExecutionStats {
    pub node_id: i64,
    pub actual_rows: usize,
    pub actual_time_ms: f64,
    pub startup_time_ms: f64,
    pub total_time_ms: f64,
    pub memory_used: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub io_reads: usize,
    pub io_read_bytes: usize,
}

impl NodeExecutionStats {
    pub fn new(node_id: i64) -> Self {
        Self {
            node_id,
            ..Default::default()
        }
    }

    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total > 0 {
            self.cache_hits as f64 / total as f64
        } else {
            0.0
        }
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

    pub fn on_node_complete(&self, node_id: i64, node_stats: NodeExecutionStats) {
        let mut stats = self.node_stats.lock();
        stats.insert(node_id, node_stats);
    }

    pub fn record_node_rows(&self, node_id: i64, rows: usize) {
        let mut stats = self.node_stats.lock();
        if let Some(s) = stats.get_mut(&node_id) {
            s.actual_rows = rows;
        }
    }

    pub fn record_node_time(&self, node_id: i64, time_ms: f64) {
        let mut stats = self.node_stats.lock();
        if let Some(s) = stats.get_mut(&node_id) {
            s.actual_time_ms = time_ms;
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
        let stats = NodeExecutionStats {
            node_id: 1,
            actual_rows: 100,
            actual_time_ms: 5.5,
            ..Default::default()
        };
        ctx.on_node_complete(1, stats);

        let collected = ctx.collect_stats();
        assert_eq!(collected.get(&1).unwrap().actual_rows, 100);
        assert_eq!(collected.get(&1).unwrap().actual_time_ms, 5.5);
    }

    #[test]
    fn test_node_execution_stats_cache_rate() {
        let stats = NodeExecutionStats {
            cache_hits: 90,
            cache_misses: 10,
            ..Default::default()
        };
        assert!((stats.cache_hit_rate() - 0.9).abs() < 0.001);
    }
}
