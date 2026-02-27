//! 集合操作估算器
//!
//! 为集合操作节点提供代价估算：
//! - Union
//! - Minus
//! - Intersect

use crate::query::planner::plan::PlanNodeEnum;
use crate::query::optimizer::cost::estimate::NodeCostEstimate;
use crate::query::optimizer::cost::CostCalculator;
use crate::core::error::optimize::CostError;
use super::{NodeEstimator, get_input_rows};

/// 集合操作估算器
pub struct SetOperationEstimator<'a> {
    cost_calculator: &'a CostCalculator,
}

impl<'a> SetOperationEstimator<'a> {
    /// 创建新的集合操作估算器
    pub fn new(cost_calculator: &'a CostCalculator) -> Self {
        Self { cost_calculator }
    }
}

impl<'a> NodeEstimator for SetOperationEstimator<'a> {
    fn estimate(
        &self,
        node: &PlanNodeEnum,
        child_estimates: &[NodeCostEstimate],
    ) -> Result<(f64, u64), CostError> {
        let left_rows = get_input_rows(child_estimates, 0);
        let right_rows = get_input_rows(child_estimates, 1);

        match node {
            PlanNodeEnum::Union(n) => {
                let cost = self.cost_calculator.calculate_union_cost(left_rows, right_rows, n.distinct());
                // Union 输出行数为两者之和（去重时减少）
                let output_rows = if n.distinct() {
                    left_rows.max(right_rows) // 去重后估算为较大值
                } else {
                    left_rows.saturating_add(right_rows)
                };
                Ok((cost, output_rows.max(1)))
            }
            PlanNodeEnum::Minus(_) => {
                let cost = self.cost_calculator.calculate_minus_cost(left_rows, right_rows);
                // Minus 输出行数为左表减去交集（假设为左表的 70%）
                let output_rows = (left_rows as f64 * 0.7).max(1.0) as u64;
                Ok((cost, output_rows))
            }
            PlanNodeEnum::Intersect(_) => {
                let cost = self.cost_calculator.calculate_intersect_cost(left_rows, right_rows);
                // Intersect 输出行数为交集（假设为较小表的 30%）
                let output_rows = (left_rows.min(right_rows) as f64 * 0.3).max(1.0) as u64;
                Ok((cost, output_rows))
            }
            _ => Err(CostError::UnsupportedNodeType(
                format!("集合操作估算器不支持节点类型: {:?}", std::mem::discriminant(node))
            )),
        }
    }
}
