//! 节点估算器模块
//!
//! 为不同类型的计划节点提供代价估算功能

use crate::core::error::optimize::CostError;
use crate::query::optimizer::cost::estimate::NodeCostEstimate;
use crate::query::planner::plan::PlanNodeEnum;

pub mod control_flow;
pub mod data_processing;
pub mod graph_algorithm;
pub mod graph_traversal;
pub mod join;
pub mod scan;
pub mod sort_limit;

pub use control_flow::ControlFlowEstimator;
pub use data_processing::DataProcessingEstimator;
pub use graph_algorithm::GraphAlgorithmEstimator;
pub use graph_traversal::GraphTraversalEstimator;
pub use join::JoinEstimator;
pub use scan::ScanEstimator;
pub use sort_limit::SortLimitEstimator;

/// 节点估算器 trait
///
/// 所有节点估算器都需要实现此 trait
pub trait NodeEstimator {
    /// 估算节点的代价和输出行数
    ///
    /// # 参数
    /// - `node`: 计划节点
    /// - `child_estimates`: 子节点的估算结果
    ///
    /// # 返回
    /// - `(node_cost, output_rows)`: 节点自身代价和估算输出行数
    fn estimate(
        &self,
        node: &PlanNodeEnum,
        child_estimates: &[NodeCostEstimate],
    ) -> Result<(f64, u64), CostError>;
}

/// 获取子节点的输入行数
pub fn get_input_rows(child_estimates: &[NodeCostEstimate], index: usize) -> u64 {
    child_estimates
        .get(index)
        .map(|e| e.output_rows)
        .unwrap_or(1)
}

/// 计算子节点的累计代价
pub fn sum_child_costs(child_estimates: &[NodeCostEstimate]) -> f64 {
    child_estimates.iter().map(|e| e.total_cost).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_input_rows() {
        let estimates = vec![
            NodeCostEstimate::new(10.0, 5.0, 100),
            NodeCostEstimate::new(20.0, 10.0, 200),
            NodeCostEstimate::new(30.0, 15.0, 300),
        ];

        assert_eq!(get_input_rows(&estimates, 0), 100);
        assert_eq!(get_input_rows(&estimates, 1), 200);
        assert_eq!(get_input_rows(&estimates, 2), 300);
    }

    #[test]
    fn test_get_input_rows_empty() {
        let estimates: Vec<NodeCostEstimate> = vec![];
        assert_eq!(get_input_rows(&estimates, 0), 1);
    }

    #[test]
    fn test_get_input_rows_out_of_bounds() {
        let estimates = vec![NodeCostEstimate::new(10.0, 5.0, 100)];
        assert_eq!(get_input_rows(&estimates, 1), 1);
    }

    #[test]
    fn test_sum_child_costs() {
        let estimates = vec![
            NodeCostEstimate::new(10.0, 5.0, 100),
            NodeCostEstimate::new(20.0, 10.0, 200),
            NodeCostEstimate::new(30.0, 15.0, 300),
        ];

        let sum = sum_child_costs(&estimates);
        assert_eq!(sum, 30.0);
    }

    #[test]
    fn test_sum_child_costs_empty() {
        let estimates: Vec<NodeCostEstimate> = vec![];
        let sum = sum_child_costs(&estimates);
        assert_eq!(sum, 0.0);
    }

    #[test]
    fn test_sum_child_costs_single() {
        let estimates = vec![NodeCostEstimate::new(10.0, 5.0, 100)];
        let sum = sum_child_costs(&estimates);
        assert_eq!(sum, 5.0);
    }

    #[test]
    fn test_node_cost_estimate_creation() {
        let estimate = NodeCostEstimate::new(10.0, 5.0, 100);
        assert_eq!(estimate.node_cost, 10.0);
        assert_eq!(estimate.total_cost, 5.0);
        assert_eq!(estimate.output_rows, 100);
    }

    #[test]
    fn test_node_cost_estimate_with_zero_values() {
        let estimate = NodeCostEstimate::new(0.0, 0.0, 0);
        assert_eq!(estimate.node_cost, 0.0);
        assert_eq!(estimate.total_cost, 0.0);
        assert_eq!(estimate.output_rows, 0);
    }

    #[test]
    fn test_node_cost_estimate_with_large_values() {
        let estimate = NodeCostEstimate::new(1_000_000.0, 500_000.0, 1_000_000);
        assert_eq!(estimate.node_cost, 1_000_000.0);
        assert_eq!(estimate.total_cost, 500_000.0);
        assert_eq!(estimate.output_rows, 1_000_000);
    }
}
