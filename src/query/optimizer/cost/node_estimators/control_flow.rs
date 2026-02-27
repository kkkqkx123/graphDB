//! 控制流节点估算器
//!
//! 为控制流节点提供代价估算：
//! - Loop
//! - Select
//! - PassThrough
//! - Argument

use crate::query::planner::plan::PlanNodeEnum;
use crate::query::optimizer::cost::estimate::NodeCostEstimate;
use crate::query::optimizer::cost::CostCalculator;
use crate::query::optimizer::cost::expression_parser::ExpressionParser;
use crate::query::optimizer::cost::config::CostModelConfig;
use crate::core::error::optimize::CostError;
use super::{NodeEstimator, get_input_rows};

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
        node: &crate::query::planner::plan::core::nodes::control_flow_node::LoopNode,
    ) -> u32 {
        let condition = node.condition().trim();

        // 使用表达式解析器尝试解析迭代次数
        if let Some(iterations) = self.expression_parser.parse_loop_iterations(condition) {
            return iterations;
        }

        // 默认使用配置值
        self.config.default_loop_iterations
    }

    /// 估算 Select 节点的分支数
    fn estimate_select_branch_count(
        &self,
        node: &crate::query::planner::plan::core::nodes::control_flow_node::SelectNode,
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
                let body_estimate = child_estimates.first().copied().unwrap_or(NodeCostEstimate::new(0.0, 0.0, 1));
                let iterations = self.estimate_loop_iterations(n);
                let cost = self.cost_calculator.calculate_loop_cost(body_estimate.total_cost, iterations);
                // Loop 输出行数为循环体输出行数乘以迭代次数
                let output_rows = body_estimate.output_rows.saturating_mul(iterations as u64);
                Ok((cost, output_rows.max(1)))
            }
            PlanNodeEnum::Select(n) => {
                let input_rows_val = get_input_rows(child_estimates, 0);
                let branch_count = self.estimate_select_branch_count(n);
                let cost = self.cost_calculator.calculate_select_cost(input_rows_val, branch_count);
                // Select 输出行数为输入行数（假设平均选择一个分支）
                Ok((cost, input_rows_val))
            }
            PlanNodeEnum::PassThrough(_) => {
                let input_rows_val = get_input_rows(child_estimates, 0);
                let cost = self.cost_calculator.calculate_pass_through_cost(input_rows_val);
                Ok((cost, input_rows_val))
            }
            PlanNodeEnum::Argument(_) => Ok((0.0, 1)),
            _ => Err(CostError::UnsupportedNodeType(
                format!("控制流估算器不支持节点类型: {:?}", std::mem::discriminant(node))
            )),
        }
    }
}
