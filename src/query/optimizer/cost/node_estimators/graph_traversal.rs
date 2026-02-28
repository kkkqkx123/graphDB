//! 图遍历操作估算器
//!
//! 为图遍历节点提供代价估算：
//! - Expand
//! - ExpandAll
//! - Traverse
//! - AppendVertices
//! - GetNeighbors
//! - GetVertices
//! - GetEdges

use crate::core::types::EdgeDirection;
use crate::query::planner::plan::PlanNodeEnum;
use crate::query::optimizer::cost::estimate::NodeCostEstimate;
use crate::query::optimizer::cost::CostCalculator;
use crate::core::error::optimize::CostError;
use super::{NodeEstimator, get_input_rows};

/// 图遍历操作估算器
pub struct GraphTraversalEstimator<'a> {
    cost_calculator: &'a CostCalculator,
}

impl<'a> GraphTraversalEstimator<'a> {
    /// 创建新的图遍历估算器
    pub fn new(cost_calculator: &'a CostCalculator) -> Self {
        Self { cost_calculator }
    }

    /// 获取边类型的平均出度
    fn get_avg_out_degree(&self, edge_type: Option<&str>) -> f64 {
        edge_type
            .and_then(|et| self.cost_calculator.statistics_manager().get_edge_stats(et))
            .map(|s| s.avg_out_degree)
            .unwrap_or(2.0)
    }

    /// 获取边类型的平均入度
    fn get_avg_in_degree(&self, edge_type: Option<&str>) -> f64 {
        edge_type
            .and_then(|et| self.cost_calculator.statistics_manager().get_edge_stats(et))
            .map(|s| s.avg_in_degree)
            .unwrap_or(2.0)
    }

    /// 获取边类型的平均度数（出入度平均值）
    fn get_avg_degree(&self, edge_type: Option<&str>) -> f64 {
        edge_type
            .and_then(|et| self.cost_calculator.statistics_manager().get_edge_stats(et))
            .map(|s| (s.avg_out_degree + s.avg_in_degree) / 2.0)
            .unwrap_or(2.0)
    }
}

impl<'a> NodeEstimator for GraphTraversalEstimator<'a> {
    fn estimate(
        &self,
        node: &PlanNodeEnum,
        child_estimates: &[NodeCostEstimate],
    ) -> Result<(f64, u64), CostError> {
        match node {
            PlanNodeEnum::Expand(n) => {
                let start_rows = get_input_rows(child_estimates, 0);
                let edge_type = n.edge_types().first().map(|s| s.as_str());
                // 根据遍历方向选择对应的度数
                let avg_degree = match n.direction() {
                    EdgeDirection::Out => self.get_avg_out_degree(edge_type),
                    EdgeDirection::In => self.get_avg_in_degree(edge_type),
                    EdgeDirection::Both => self.get_avg_degree(edge_type),
                };
                let output_rows = (start_rows as f64 * avg_degree) as u64;
                let cost = self.cost_calculator.calculate_expand_cost(start_rows, edge_type);
                Ok((cost, output_rows.max(1)))
            }
            PlanNodeEnum::ExpandAll(n) => {
                let start_rows = get_input_rows(child_estimates, 0);
                let edge_type = n.edge_types().first().map(|s| s.as_str());
                // ExpandAllNode 使用字符串表示方向，需要解析
                let avg_degree = match n.direction() {
                    "IN" | "in" | "In" => self.get_avg_in_degree(edge_type),
                    "BOTH" | "both" | "Both" => self.get_avg_degree(edge_type),
                    _ => self.get_avg_out_degree(edge_type), // 默认出边
                };
                let output_rows = (start_rows as f64 * avg_degree) as u64;
                let cost = self.cost_calculator.calculate_expand_all_cost(start_rows, edge_type);
                Ok((cost, output_rows.max(1)))
            }
            PlanNodeEnum::Traverse(n) => {
                let start_rows = get_input_rows(child_estimates, 0);
                let edge_type = n.edge_types().first().map(|s| s.as_str());
                let steps = n.max_steps();
                // 根据遍历方向选择度数
                let avg_degree = match n.direction() {
                    EdgeDirection::Out => self.get_avg_out_degree(edge_type),
                    EdgeDirection::In => self.get_avg_in_degree(edge_type),
                    EdgeDirection::Both => self.get_avg_degree(edge_type),
                };
                // 多步遍历的输出行数估算
                let output_rows = (start_rows as f64 * avg_degree.powi(steps as i32)) as u64;
                let cost = self.cost_calculator
                    .calculate_traverse_cost(start_rows, edge_type, steps);
                Ok((cost, output_rows.max(1)))
            }
            PlanNodeEnum::AppendVertices(_) => {
                let input_rows_val = get_input_rows(child_estimates, 0);
                let cost = self.cost_calculator.calculate_append_vertices_cost(input_rows_val);
                // AppendVertices 不改变行数
                Ok((cost, input_rows_val))
            }
            PlanNodeEnum::GetNeighbors(n) => {
                let start_rows = get_input_rows(child_estimates, 0);
                let edge_type = n.edge_types().first().map(|s| s.as_str());
                // GetNeighborsNode 使用字符串表示方向，需要解析
                let avg_degree = match n.direction() {
                    "IN" | "in" | "In" => self.get_avg_in_degree(edge_type),
                    "BOTH" | "both" | "Both" => self.get_avg_degree(edge_type),
                    _ => self.get_avg_out_degree(edge_type), // 默认出边
                };
                let output_rows = (start_rows as f64 * avg_degree) as u64;
                let cost = self.cost_calculator
                    .calculate_get_neighbors_cost(start_rows, edge_type);
                Ok((cost, output_rows.max(1)))
            }
            PlanNodeEnum::GetVertices(n) => {
                let vid_count = n.limit().unwrap_or(100) as u64;
                let cost = self.cost_calculator.calculate_get_vertices_cost(vid_count);
                Ok((cost, vid_count))
            }
            PlanNodeEnum::GetEdges(n) => {
                let edge_count = n.limit().unwrap_or(100) as u64;
                let cost = self.cost_calculator.calculate_get_edges_cost(edge_count);
                Ok((cost, edge_count))
            }
            _ => Err(CostError::UnsupportedNodeType(
                format!("图遍历估算器不支持节点类型: {:?}", std::mem::discriminant(node))
            )),
        }
    }
}
