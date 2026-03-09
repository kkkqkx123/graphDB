//! 控制流节点估算器
//!
//! 为控制流节点提供代价估算：
//! - Loop
//! - Select
//! - PassThrough
//! - Argument

use super::{get_input_rows, NodeEstimator};
use crate::core::error::optimize::CostError;
use crate::query::optimizer::cost::config::CostModelConfig;
use crate::query::optimizer::cost::estimate::NodeCostEstimate;
use crate::query::optimizer::cost::expression_parser::ExpressionParser;
use crate::query::optimizer::cost::CostCalculator;
use crate::query::planner::plan::PlanNodeEnum;
use crate::query::planner::plan::core::nodes::control_flow::control_flow_node::{LoopNode, SelectNode};

/// 控制流节点估算器
pub struct ControlFlowEstimator<'a> {
    cost_calculator: &'a CostCalculator,
    config: CostModelConfig,
    expression_parser: ExpressionParser,
}

impl<'a> ControlFlowEstimator<'a> {
    /// 创建新的控制流估算器
    pub fn new(cost_calculator: &'a CostCalculator, config: CostModelConfig) -> Self {
        let expression_parser = ExpressionParser::new(config);
        Self {
            cost_calculator,
            config,
            expression_parser,
        }
    }

    /// 估算 Loop 节点的迭代次数
    fn estimate_loop_iterations(
        &self,
        node: &LoopNode,
    ) -> u32 {
        let condition = node.condition().to_expression_string();

        // 使用表达式解析器尝试解析迭代次数
        if let Some(iterations) = self.expression_parser.parse_loop_iterations(&condition) {
            return iterations;
        }

        // 默认使用配置值
        self.config.default_loop_iterations
    }

    /// 估算 Select 节点的分支数
    fn estimate_select_branch_count(
        &self,
        node: &SelectNode,
    ) -> usize {
        let mut count = 0;
        if node.if_branch().is_some() {
            count += 1;
        }
        if node.else_branch().is_some() {
            count += 1;
        }

        if count == 0 {
            self.config.default_select_branches
        } else {
            count
        }
    }
}

impl<'a> NodeEstimator for ControlFlowEstimator<'a> {
    fn estimate(
        &self,
        node: &PlanNodeEnum,
        child_estimates: &[NodeCostEstimate],
    ) -> Result<(f64, u64), CostError> {
        match node {
            PlanNodeEnum::Loop(n) => {
                let body_estimate = child_estimates
                    .first()
                    .copied()
                    .unwrap_or(NodeCostEstimate::new(0.0, 0.0, 0));
                let iterations = self.estimate_loop_iterations(n);
                let cost = self
                    .cost_calculator
                    .calculate_loop_cost(body_estimate.total_cost, iterations);
                // Loop 输出行数为循环体输出行数乘以迭代次数
                let output_rows = body_estimate.output_rows.saturating_mul(iterations as u64);
                Ok((cost, output_rows))
            }
            PlanNodeEnum::Select(n) => {
                let input_rows_val = get_input_rows(child_estimates, 0);
                let branch_count = self.estimate_select_branch_count(n);
                let cost = self
                    .cost_calculator
                    .calculate_select_cost(input_rows_val, branch_count);
                // Select 输出行数为输入行数（假设平均选择一个分支）
                Ok((cost, input_rows_val))
            }
            PlanNodeEnum::PassThrough(_) => {
                let input_rows_val = get_input_rows(child_estimates, 0);
                let cost = self
                    .cost_calculator
                    .calculate_pass_through_cost(input_rows_val);
                Ok((cost, input_rows_val))
            }
            PlanNodeEnum::Argument(_) => Ok((0.0, 1)),
            _ => Err(CostError::UnsupportedNodeType(format!(
                "控制流估算器不支持节点类型: {:?}",
                std::mem::discriminant(node)
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::optimizer::cost::config::CostModelConfig;
    use crate::query::planner::plan::core::nodes::control_flow::control_flow_node::*;
    use crate::query::planner::plan::core::nodes::control_flow::start_node::StartNode;
    use std::sync::Arc;

    fn create_test_calculator() -> CostCalculator {
        let stats_manager = Arc::new(crate::query::optimizer::stats::StatisticsManager::new());
        let config = CostModelConfig::default();
        CostCalculator::with_config(stats_manager, config)
    }

    fn create_test_expression() -> crate::core::types::ContextualExpression {
        use crate::core::types::expression::ExpressionMeta;
        use crate::core::Expression;
        use crate::query::validator::context::ExpressionAnalysisContext;

        let ctx = Arc::new(ExpressionAnalysisContext::new());
        let expr_meta = ExpressionMeta::new(Expression::Variable("condition".to_string()));
        let id = ctx.register_expression(expr_meta);
        crate::core::types::ContextualExpression::new(id, ctx)
    }

    #[test]
    fn test_loop_estimation() {
        let calculator = create_test_calculator();
        let config = CostModelConfig::default();
        let estimator = ControlFlowEstimator::new(&calculator, config);

        let condition = create_test_expression();
        let mut node = LoopNode::new(1, condition);
        node.set_body(PlanNodeEnum::Start(StartNode::new()));
        let plan_node = PlanNodeEnum::Loop(node);

        let child_estimates = vec![NodeCostEstimate::new(10.0, 10.0, 100)];
        let result = estimator.estimate(&plan_node, &child_estimates);

        assert!(result.is_ok());
        let (cost, output_rows) = result.expect("Estimation should succeed");
        assert!(cost > 0.0);
        assert!(output_rows >= 1);
    }

    #[test]
    fn test_select_estimation() {
        let calculator = create_test_calculator();
        let config = CostModelConfig::default();
        let estimator = ControlFlowEstimator::new(&calculator, config);

        let condition = create_test_expression();
        let mut node = SelectNode::new(1, condition);
        node.set_if_branch(PlanNodeEnum::Start(StartNode::new()));
        node.set_else_branch(PlanNodeEnum::Start(StartNode::new()));
        let plan_node = PlanNodeEnum::Select(node);

        let child_estimates = vec![NodeCostEstimate::new(10.0, 10.0, 100)];
        let result = estimator.estimate(&plan_node, &child_estimates);

        assert!(result.is_ok());
        let (cost, output_rows) = result.expect("Estimation should succeed");
        assert!(cost > 0.0);
        assert_eq!(output_rows, 100);
    }

    #[test]
    fn test_pass_through_estimation() {
        let calculator = create_test_calculator();
        let config = CostModelConfig::default();
        let estimator = ControlFlowEstimator::new(&calculator, config);

        let node = PassThroughNode::new(1);
        let plan_node = PlanNodeEnum::PassThrough(node);

        let child_estimates = vec![NodeCostEstimate::new(10.0, 10.0, 100)];
        let result = estimator.estimate(&plan_node, &child_estimates);

        assert!(result.is_ok());
        let (cost, output_rows) = result.expect("Estimation should succeed");
        assert!(cost > 0.0);
        assert_eq!(output_rows, 100);
    }

    #[test]
    fn test_argument_estimation() {
        let calculator = create_test_calculator();
        let config = CostModelConfig::default();
        let estimator = ControlFlowEstimator::new(&calculator, config);

        let node = ArgumentNode::new(1, "var_name");
        let plan_node = PlanNodeEnum::Argument(node);

        let child_estimates = vec![];
        let result = estimator.estimate(&plan_node, &child_estimates);

        assert!(result.is_ok());
        let (cost, output_rows) = result.expect("Estimation should succeed");
        assert_eq!(cost, 0.0);
        assert_eq!(output_rows, 1);
    }

    #[test]
    fn test_unsupported_node_type() {
        let calculator = create_test_calculator();
        let config = CostModelConfig::default();
        let estimator = ControlFlowEstimator::new(&calculator, config);

        let node = PlanNodeEnum::Start(StartNode::new());
        let child_estimates = vec![];
        let result = estimator.estimate(&node, &child_estimates);

        assert!(result.is_err());
    }

    #[test]
    fn test_select_without_branches() {
        let calculator = create_test_calculator();
        let config = CostModelConfig::default();
        let estimator = ControlFlowEstimator::new(&calculator, config);

        let condition = create_test_expression();
        let node = SelectNode::new(1, condition);
        let plan_node = PlanNodeEnum::Select(node);

        let child_estimates = vec![NodeCostEstimate::new(10.0, 10.0, 100)];
        let result = estimator.estimate(&plan_node, &child_estimates);

        assert!(result.is_ok());
        let (cost, output_rows) = result.expect("Estimation should succeed");
        assert!(cost > 0.0);
        assert_eq!(output_rows, 100);
    }

    #[test]
    fn test_select_with_only_if_branch() {
        let calculator = create_test_calculator();
        let config = CostModelConfig::default();
        let estimator = ControlFlowEstimator::new(&calculator, config);

        let condition = create_test_expression();
        let mut node = SelectNode::new(1, condition);
        node.set_if_branch(PlanNodeEnum::Start(StartNode::new()));
        let plan_node = PlanNodeEnum::Select(node);

        let child_estimates = vec![NodeCostEstimate::new(10.0, 10.0, 100)];
        let result = estimator.estimate(&plan_node, &child_estimates);

        assert!(result.is_ok());
        let (cost, output_rows) = result.expect("Estimation should succeed");
        assert!(cost > 0.0);
        assert_eq!(output_rows, 100);
    }

    #[test]
    fn test_select_with_only_else_branch() {
        let calculator = create_test_calculator();
        let config = CostModelConfig::default();
        let estimator = ControlFlowEstimator::new(&calculator, config);

        let condition = create_test_expression();
        let mut node = SelectNode::new(1, condition);
        node.set_else_branch(PlanNodeEnum::Start(StartNode::new()));
        let plan_node = PlanNodeEnum::Select(node);

        let child_estimates = vec![NodeCostEstimate::new(10.0, 10.0, 100)];
        let result = estimator.estimate(&plan_node, &child_estimates);

        assert!(result.is_ok());
        let (cost, output_rows) = result.expect("Estimation should succeed");
        assert!(cost > 0.0);
        assert_eq!(output_rows, 100);
    }

    #[test]
    fn test_loop_with_different_iterations() {
        let calculator = create_test_calculator();
        let config = CostModelConfig::default();
        let estimator = ControlFlowEstimator::new(&calculator, config);

        let condition = create_test_expression();
        let mut node = LoopNode::new(1, condition);
        node.set_body(PlanNodeEnum::Start(StartNode::new()));
        let plan_node = PlanNodeEnum::Loop(node);

        let child_estimates = vec![NodeCostEstimate::new(10.0, 10.0, 100)];
        let result = estimator.estimate(&plan_node, &child_estimates);

        assert!(result.is_ok());
        let (cost, output_rows) = result.expect("Estimation should succeed");
        assert!(cost > 0.0);
        assert!(output_rows >= 1);
    }

    #[test]
    fn test_pass_through_with_zero_input() {
        let calculator = create_test_calculator();
        let config = CostModelConfig::default();
        let estimator = ControlFlowEstimator::new(&calculator, config);

        let node = PassThroughNode::new(1);
        let plan_node = PlanNodeEnum::PassThrough(node);

        let child_estimates = vec![NodeCostEstimate::new(0.0, 0.0, 0)];
        let result = estimator.estimate(&plan_node, &child_estimates);

        assert!(result.is_ok());
        let (cost, output_rows) = result.expect("Estimation should succeed");
        assert!(cost >= 0.0);
        assert_eq!(output_rows, 0);
    }

    #[test]
    fn test_pass_through_with_large_input() {
        let calculator = create_test_calculator();
        let config = CostModelConfig::default();
        let estimator = ControlFlowEstimator::new(&calculator, config);

        let node = PassThroughNode::new(1);
        let plan_node = PlanNodeEnum::PassThrough(node);

        let child_estimates = vec![NodeCostEstimate::new(1000.0, 1000.0, 1_000_000)];
        let result = estimator.estimate(&plan_node, &child_estimates);

        assert!(result.is_ok());
        let (cost, output_rows) = result.expect("Estimation should succeed");
        assert!(cost > 0.0);
        assert_eq!(output_rows, 1_000_000);
    }

    #[test]
    fn test_loop_without_child_estimates() {
        let calculator = create_test_calculator();
        let config = CostModelConfig::default();
        let estimator = ControlFlowEstimator::new(&calculator, config);

        let condition = create_test_expression();
        let mut node = LoopNode::new(1, condition);
        node.set_body(PlanNodeEnum::Start(StartNode::new()));
        let plan_node = PlanNodeEnum::Loop(node);

        let child_estimates = vec![];
        let result = estimator.estimate(&plan_node, &child_estimates);

        assert!(result.is_ok());
        let (cost, output_rows) = result.expect("Estimation should succeed");
        assert!(cost >= 0.0);
        assert_eq!(output_rows, 0);
    }
}
