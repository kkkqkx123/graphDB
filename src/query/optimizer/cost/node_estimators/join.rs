//! 连接操作估算器
//!
//! 为连接节点提供代价估算：
//! - HashInnerJoin
//! - HashLeftJoin
//! - InnerJoin
//! - LeftJoin
//! - CrossJoin
//! - FullOuterJoin

use crate::query::planner::plan::PlanNodeEnum;
use crate::query::optimizer::cost::estimate::NodeCostEstimate;
use crate::query::optimizer::cost::CostCalculator;
use crate::core::error::optimize::CostError;
use super::{NodeEstimator, get_input_rows};

/// 连接操作估算器
pub struct JoinEstimator<'a> {
    cost_calculator: &'a CostCalculator,
}

impl<'a> JoinEstimator<'a> {
    /// 创建新的连接估算器
    pub fn new(cost_calculator: &'a CostCalculator) -> Self {
        Self { cost_calculator }
    }
}

impl<'a> NodeEstimator for JoinEstimator<'a> {
    fn estimate(
        &self,
        node: &PlanNodeEnum,
        child_estimates: &[NodeCostEstimate],
    ) -> Result<(f64, u64), CostError> {
        let left_rows = get_input_rows(child_estimates, 0);
        let right_rows = get_input_rows(child_estimates, 1);

        match node {
            PlanNodeEnum::HashInnerJoin(_) => {
                // 内连接输出行数估算（假设选择性为 0.3）
                let output_rows = (left_rows.min(right_rows) as f64 * 0.3).max(1.0) as u64;
                let cost = self.cost_calculator.calculate_hash_join_cost(left_rows, right_rows);
                Ok((cost, output_rows))
            }
            PlanNodeEnum::HashLeftJoin(_) => {
                // 左连接保持左表所有行
                let output_rows = left_rows;
                let cost = self.cost_calculator
                    .calculate_hash_left_join_cost(left_rows, right_rows);
                Ok((cost, output_rows))
            }
            PlanNodeEnum::InnerJoin(_) => {
                let output_rows = (left_rows.min(right_rows) as f64 * 0.3).max(1.0) as u64;
                let cost = self.cost_calculator.calculate_inner_join_cost(left_rows, right_rows);
                Ok((cost, output_rows))
            }
            PlanNodeEnum::LeftJoin(_) => {
                let output_rows = left_rows;
                let cost = self.cost_calculator.calculate_left_join_cost(left_rows, right_rows);
                Ok((cost, output_rows))
            }
            PlanNodeEnum::CrossJoin(_) => {
                let output_rows = left_rows.saturating_mul(right_rows);
                let cost = self.cost_calculator.calculate_cross_join_cost(left_rows, right_rows);
                Ok((cost, output_rows.max(1)))
            }
            PlanNodeEnum::FullOuterJoin(_) => {
                let output_rows = left_rows.saturating_add(right_rows);
                let cost = self.cost_calculator
                    .calculate_full_outer_join_cost(left_rows, right_rows);
                Ok((cost, output_rows.max(1)))
            }
            _ => Err(CostError::UnsupportedNodeType(
                format!("连接估算器不支持节点类型: {:?}", std::mem::discriminant(node))
            )),
        }
    }
}
