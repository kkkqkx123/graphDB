//! 代价计算器模块
//!
//! 针对图数据库特性设计的轻量级代价计算

use std::sync::Arc;

use crate::query::optimizer::stats::StatisticsManager;

/// 代价计算器
///
/// 针对图数据库特性设计的轻量级代价计算
#[derive(Debug)]
pub struct CostCalculator {
    stats_manager: Arc<StatisticsManager>,
}

impl CostCalculator {
    /// 创建新的代价计算器
    pub fn new(stats_manager: Arc<StatisticsManager>) -> Self {
        Self { stats_manager }
    }

    /// 计算全表扫描代价
    pub fn calculate_scan_cost(&self, tag_name: &str) -> f64 {
        let row_count = self.stats_manager.get_vertex_count(tag_name);
        row_count as f64
    }

    /// 计算索引扫描代价
    pub fn calculate_index_scan_cost(
        &self,
        tag_name: &str,
        selectivity: f64,
    ) -> f64 {
        let table_rows = self.stats_manager.get_vertex_count(tag_name);
        let matching_rows = (selectivity * table_rows as f64) as u64;
        let index_pages = (matching_rows / 10).max(1);

        index_pages as f64 * 0.1 + matching_rows as f64
    }

    /// 计算单步扩展代价
    pub fn calculate_expand_cost(
        &self,
        start_nodes: u64,
        edge_type: Option<&str>,
    ) -> f64 {
        let avg_degree = match edge_type {
            Some(et) => {
                self.stats_manager.get_edge_stats(et)
                    .map(|s| s.avg_out_degree)
                    .unwrap_or(1.0)
            }
            None => 2.0,
        };

        start_nodes as f64 * avg_degree
    }

    /// 计算多步遍历代价
    pub fn calculate_traverse_cost(
        &self,
        start_nodes: u64,
        edge_type: Option<&str>,
        steps: u32,
    ) -> f64 {
        let avg_degree = match edge_type {
            Some(et) => {
                self.stats_manager.get_edge_stats(et)
                    .map(|s| (s.avg_out_degree + s.avg_in_degree) / 2.0)
                    .unwrap_or(1.0)
            }
            None => 2.0,
        };

        start_nodes as f64 * avg_degree.powi(steps as i32)
    }

    /// 计算过滤代价
    pub fn calculate_filter_cost(&self, input_rows: u64, condition_count: usize) -> f64 {
        input_rows as f64 * condition_count as f64 * 0.01
    }

    /// 计算哈希连接代价
    pub fn calculate_hash_join_cost(&self, left_rows: u64, right_rows: u64) -> f64 {
        let build_cost = left_rows as f64;
        let probe_cost = right_rows as f64;
        let hash_overhead = left_rows as f64 * 0.1;

        build_cost + probe_cost + hash_overhead
    }

    /// 获取统计信息管理器
    pub fn statistics_manager(&self) -> Arc<StatisticsManager> {
        self.stats_manager.clone()
    }
}

impl Clone for CostCalculator {
    fn clone(&self) -> Self {
        Self {
            stats_manager: self.stats_manager.clone(),
        }
    }
}
