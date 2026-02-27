//! 图算法节点估算器
//!
//! 为图算法节点提供代价估算：
//! - ShortestPath
//! - AllPaths
//! - MultiShortestPath
//! - BFSShortest

use crate::query::planner::plan::PlanNodeEnum;
use crate::query::optimizer::cost::estimate::NodeCostEstimate;
use crate::query::optimizer::cost::CostCalculator;
use crate::core::error::optimize::CostError;
use super::NodeEstimator;

/// 图算法节点估算器
pub struct GraphAlgorithmEstimator<'a> {
    cost_calculator: &'a CostCalculator,
}

impl<'a> GraphAlgorithmEstimator<'a> {
    /// 创建新的图算法估算器
    pub fn new(cost_calculator: &'a CostCalculator) -> Self {
        Self { cost_calculator }
    }
}

impl<'a> NodeEstimator for GraphAlgorithmEstimator<'a> {
    fn estimate(
        &self,
        node: &PlanNodeEnum,
        _child_estimates: &[NodeCostEstimate],
    ) -> Result<(f64, u64), CostError> {
        match node {
            PlanNodeEnum::ShortestPath(n) => {
                let max_depth = n.max_step() as u32;
                let cost = self.cost_calculator.calculate_shortest_path_cost(1, max_depth);
                // 最短路径返回一条路径
                Ok((cost, 1))
            }
            PlanNodeEnum::AllPaths(n) => {
                let max_depth = n.max_hop() as u32;
                let cost = self.cost_calculator.calculate_all_paths_cost(1, max_depth);
                // 所有路径可能返回多条路径（估算）
                let output_rows = 2_u64.pow(max_depth.min(10));
                Ok((cost, output_rows))
            }
            PlanNodeEnum::MultiShortestPath(n) => {
                let max_depth = n.steps() as u32;
                let cost = self.cost_calculator.calculate_multi_shortest_path_cost(2, max_depth);
                // 多源最短路径返回多条路径
                let output_rows = 2_u64.pow(max_depth.min(10));
                Ok((cost, output_rows))
            }
            PlanNodeEnum::BFSShortest(n) => {
                let max_depth = n.steps() as u32;
                let cost = self.cost_calculator.calculate_shortest_path_cost(1, max_depth);
                Ok((cost, 1))
            }
            _ => Err(CostError::UnsupportedNodeType(
                format!("图算法估算器不支持节点类型: {:?}", std::mem::discriminant(node))
            )),
        }
    }
}
