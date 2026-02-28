//! 节点估算器模块
//!
//! 为不同类型的计划节点提供代价估算功能

use crate::query::planner::plan::PlanNodeEnum;
use crate::query::optimizer::cost::estimate::NodeCostEstimate;
use crate::core::error::optimize::CostError;

pub mod scan;
pub mod graph_traversal;
pub mod join;
pub mod sort_limit;
pub mod control_flow;
pub mod graph_algorithm;
pub mod data_processing;

pub use scan::ScanEstimator;
pub use graph_traversal::GraphTraversalEstimator;
pub use join::JoinEstimator;
pub use sort_limit::SortLimitEstimator;
pub use control_flow::ControlFlowEstimator;
pub use graph_algorithm::GraphAlgorithmEstimator;
pub use data_processing::DataProcessingEstimator;

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
