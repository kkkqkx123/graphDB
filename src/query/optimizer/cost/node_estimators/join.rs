//! 连接操作估算器
//!
//! 为连接节点提供代价估算：
//! - HashInnerJoin
//! - HashLeftJoin
//! - InnerJoin
//! - LeftJoin
//! - CrossJoin
//! - FullOuterJoin

use super::{get_input_rows, NodeEstimator};
use crate::core::error::optimize::CostError;
use crate::query::optimizer::cost::estimate::NodeCostEstimate;
use crate::query::optimizer::cost::CostCalculator;
use crate::query::planner::plan::PlanNodeEnum;

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
                let cost = self
                    .cost_calculator
                    .calculate_hash_join_cost(left_rows, right_rows);
                Ok((cost, output_rows))
            }
            PlanNodeEnum::HashLeftJoin(_) => {
                // 左连接保持左表所有行
                let output_rows = left_rows;
                let cost = self
                    .cost_calculator
                    .calculate_hash_left_join_cost(left_rows, right_rows);
                Ok((cost, output_rows))
            }
            PlanNodeEnum::InnerJoin(_) => {
                let output_rows = (left_rows.min(right_rows) as f64 * 0.3).max(1.0) as u64;
                let cost = self
                    .cost_calculator
                    .calculate_inner_join_cost(left_rows, right_rows);
                Ok((cost, output_rows))
            }
            PlanNodeEnum::LeftJoin(_) => {
                let output_rows = left_rows;
                let cost = self
                    .cost_calculator
                    .calculate_left_join_cost(left_rows, right_rows);
                Ok((cost, output_rows))
            }
            PlanNodeEnum::CrossJoin(_) => {
                let output_rows = left_rows.saturating_mul(right_rows);
                let cost = self
                    .cost_calculator
                    .calculate_cross_join_cost(left_rows, right_rows);
                Ok((cost, output_rows.max(1)))
            }
            PlanNodeEnum::FullOuterJoin(_) => {
                let output_rows = left_rows.saturating_add(right_rows);
                let cost = self
                    .cost_calculator
                    .calculate_full_outer_join_cost(left_rows, right_rows);
                Ok((cost, output_rows.max(1)))
            }
            _ => Err(CostError::UnsupportedNodeType(format!(
                "连接估算器不支持节点类型: {:?}",
                std::mem::discriminant(node)
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::optimizer::cost::config::CostModelConfig;
    use crate::query::planner::plan::core::nodes::join_node::*;
    use crate::query::planner::plan::core::nodes::start_node::StartNode;
    use std::sync::Arc;

    fn create_test_calculator() -> CostCalculator {
        let stats_manager = Arc::new(crate::query::optimizer::stats::StatisticsManager::new());
        let config = CostModelConfig::default();
        CostCalculator::with_config(stats_manager, config)
    }

    fn create_test_start_node() -> PlanNodeEnum {
        PlanNodeEnum::Start(StartNode::new())
    }

    #[test]
    fn test_hash_inner_join_estimation() {
        let calculator = create_test_calculator();
        let estimator = JoinEstimator::new(&calculator);

        let left = create_test_start_node();
        let right = create_test_start_node();
        let node = HashInnerJoinNode::new(left, right, vec![], vec![]).unwrap();
        let plan_node = PlanNodeEnum::HashInnerJoin(node);

        let child_estimates = vec![
            NodeCostEstimate::new(10.0, 10.0, 100),
            NodeCostEstimate::new(20.0, 20.0, 200),
        ];
        let result = estimator.estimate(&plan_node, &child_estimates);

        assert!(result.is_ok());
        let (cost, output_rows) = result.unwrap();
        assert!(cost > 0.0);
        assert!(output_rows >= 1);
        assert!(output_rows <= 100);
    }

    #[test]
    fn test_hash_left_join_estimation() {
        let calculator = create_test_calculator();
        let estimator = JoinEstimator::new(&calculator);

        let left = create_test_start_node();
        let right = create_test_start_node();
        let node = HashLeftJoinNode::new(left, right, vec![], vec![]).unwrap();
        let plan_node = PlanNodeEnum::HashLeftJoin(node);

        let child_estimates = vec![
            NodeCostEstimate::new(10.0, 10.0, 100),
            NodeCostEstimate::new(20.0, 20.0, 200),
        ];
        let result = estimator.estimate(&plan_node, &child_estimates);

        assert!(result.is_ok());
        let (cost, output_rows) = result.unwrap();
        assert!(cost > 0.0);
        assert_eq!(output_rows, 100);
    }

    #[test]
    fn test_inner_join_estimation() {
        let calculator = create_test_calculator();
        let estimator = JoinEstimator::new(&calculator);

        let left = create_test_start_node();
        let right = create_test_start_node();
        let node = InnerJoinNode::new(left, right, vec![], vec![]).unwrap();
        let plan_node = PlanNodeEnum::InnerJoin(node);

        let child_estimates = vec![
            NodeCostEstimate::new(10.0, 10.0, 100),
            NodeCostEstimate::new(20.0, 20.0, 200),
        ];
        let result = estimator.estimate(&plan_node, &child_estimates);

        assert!(result.is_ok());
        let (cost, output_rows) = result.unwrap();
        assert!(cost > 0.0);
        assert!(output_rows >= 1);
        assert!(output_rows <= 100);
    }

    #[test]
    fn test_left_join_estimation() {
        let calculator = create_test_calculator();
        let estimator = JoinEstimator::new(&calculator);

        let left = create_test_start_node();
        let right = create_test_start_node();
        let node = LeftJoinNode::new(left, right, vec![], vec![]).unwrap();
        let plan_node = PlanNodeEnum::LeftJoin(node);

        let child_estimates = vec![
            NodeCostEstimate::new(10.0, 10.0, 100),
            NodeCostEstimate::new(20.0, 20.0, 200),
        ];
        let result = estimator.estimate(&plan_node, &child_estimates);

        assert!(result.is_ok());
        let (cost, output_rows) = result.unwrap();
        assert!(cost > 0.0);
        assert_eq!(output_rows, 100);
    }

    #[test]
    fn test_cross_join_estimation() {
        let calculator = create_test_calculator();
        let estimator = JoinEstimator::new(&calculator);

        let left = create_test_start_node();
        let right = create_test_start_node();
        let node = CrossJoinNode::new(left, right).unwrap();
        let plan_node = PlanNodeEnum::CrossJoin(node);

        let child_estimates = vec![
            NodeCostEstimate::new(10.0, 10.0, 100),
            NodeCostEstimate::new(20.0, 20.0, 200),
        ];
        let result = estimator.estimate(&plan_node, &child_estimates);

        assert!(result.is_ok());
        let (cost, output_rows) = result.unwrap();
        assert!(cost > 0.0);
        assert_eq!(output_rows, 20000);
    }

    #[test]
    fn test_full_outer_join_estimation() {
        let calculator = create_test_calculator();
        let estimator = JoinEstimator::new(&calculator);

        let left = create_test_start_node();
        let right = create_test_start_node();
        let node = FullOuterJoinNode::new(left, right, vec![], vec![]).unwrap();
        let plan_node = PlanNodeEnum::FullOuterJoin(node);

        let child_estimates = vec![
            NodeCostEstimate::new(10.0, 10.0, 100),
            NodeCostEstimate::new(20.0, 20.0, 200),
        ];
        let result = estimator.estimate(&plan_node, &child_estimates);

        assert!(result.is_ok());
        let (cost, output_rows) = result.unwrap();
        assert!(cost > 0.0);
        assert_eq!(output_rows, 300);
    }

    #[test]
    fn test_unsupported_node_type() {
        let calculator = create_test_calculator();
        let estimator = JoinEstimator::new(&calculator);

        let node = PlanNodeEnum::Start(StartNode::new());
        let child_estimates = vec![];
        let result = estimator.estimate(&node, &child_estimates);

        assert!(result.is_err());
    }

    #[test]
    fn test_join_with_different_input_sizes() {
        let calculator = create_test_calculator();
        let estimator = JoinEstimator::new(&calculator);

        for (left_rows, right_rows) in [(10, 20), (100, 200), (1000, 500)] {
            let left = create_test_start_node();
            let right = create_test_start_node();
            let node = HashInnerJoinNode::new(left, right, vec![], vec![]).unwrap();
            let plan_node = PlanNodeEnum::HashInnerJoin(node);

            let child_estimates = vec![
                NodeCostEstimate::new(10.0, 10.0, left_rows),
                NodeCostEstimate::new(20.0, 20.0, right_rows),
            ];
            let result = estimator.estimate(&plan_node, &child_estimates);

            assert!(result.is_ok());
            let (cost, output_rows) = result.unwrap();
            assert!(cost > 0.0);
            assert!(output_rows >= 1);
        }
    }

    #[test]
    fn test_cross_join_large_inputs() {
        let calculator = create_test_calculator();
        let estimator = JoinEstimator::new(&calculator);

        let left = create_test_start_node();
        let right = create_test_start_node();
        let node = CrossJoinNode::new(left, right).unwrap();
        let plan_node = PlanNodeEnum::CrossJoin(node);

        let child_estimates = vec![
            NodeCostEstimate::new(10.0, 10.0, 1000),
            NodeCostEstimate::new(20.0, 20.0, 1000),
        ];
        let result = estimator.estimate(&plan_node, &child_estimates);

        assert!(result.is_ok());
        let (cost, output_rows) = result.unwrap();
        assert!(cost > 0.0);
        assert_eq!(output_rows, 1_000_000);
    }

    #[test]
    fn test_join_with_zero_input_rows() {
        let calculator = create_test_calculator();
        let estimator = JoinEstimator::new(&calculator);

        let left = create_test_start_node();
        let right = create_test_start_node();
        let node = HashInnerJoinNode::new(left, right, vec![], vec![]).unwrap();
        let plan_node = PlanNodeEnum::HashInnerJoin(node);

        let child_estimates = vec![
            NodeCostEstimate::new(0.0, 0.0, 0),
            NodeCostEstimate::new(0.0, 0.0, 0),
        ];
        let result = estimator.estimate(&plan_node, &child_estimates);

        assert!(result.is_ok());
        let (cost, output_rows) = result.unwrap();
        assert!(cost >= 0.0);
        assert_eq!(output_rows, 1);
    }

    #[test]
    fn test_left_join_preserves_left_rows() {
        let calculator = create_test_calculator();
        let estimator = JoinEstimator::new(&calculator);

        let left = create_test_start_node();
        let right = create_test_start_node();
        let node = LeftJoinNode::new(left, right, vec![], vec![]).unwrap();
        let plan_node = PlanNodeEnum::LeftJoin(node);

        let child_estimates = vec![
            NodeCostEstimate::new(10.0, 10.0, 500),
            NodeCostEstimate::new(20.0, 20.0, 1000),
        ];
        let result = estimator.estimate(&plan_node, &child_estimates);

        assert!(result.is_ok());
        let (_, output_rows) = result.unwrap();
        assert_eq!(output_rows, 500);
    }

    #[test]
    fn test_hash_left_join_preserves_left_rows() {
        let calculator = create_test_calculator();
        let estimator = JoinEstimator::new(&calculator);

        let left = create_test_start_node();
        let right = create_test_start_node();
        let node = HashLeftJoinNode::new(left, right, vec![], vec![]).unwrap();
        let plan_node = PlanNodeEnum::HashLeftJoin(node);

        let child_estimates = vec![
            NodeCostEstimate::new(10.0, 10.0, 750),
            NodeCostEstimate::new(20.0, 20.0, 1500),
        ];
        let result = estimator.estimate(&plan_node, &child_estimates);

        assert!(result.is_ok());
        let (_, output_rows) = result.unwrap();
        assert_eq!(output_rows, 750);
    }
}
