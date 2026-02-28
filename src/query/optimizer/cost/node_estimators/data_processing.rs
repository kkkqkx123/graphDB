//! 数据处理节点估算器
//!
//! 为数据处理节点提供代价估算：
//! - Filter
//! - Project
//! - Unwind
//! - DataCollect
//! - Start

use crate::core::Expression;
use crate::core::types::BinaryOperator;
use crate::query::planner::plan::PlanNodeEnum;
use crate::query::optimizer::cost::estimate::NodeCostEstimate;
use crate::query::optimizer::cost::CostCalculator;
use crate::query::optimizer::cost::selectivity::SelectivityEstimator;
use crate::query::optimizer::cost::expression_parser::ExpressionParser;
use crate::query::optimizer::cost::config::CostModelConfig;
use crate::core::error::optimize::CostError;
use super::{NodeEstimator, get_input_rows};

/// 数据处理节点估算器
pub struct DataProcessingEstimator<'a> {
    cost_calculator: &'a CostCalculator,
    selectivity_estimator: &'a SelectivityEstimator,
    expression_parser: ExpressionParser,
}

impl<'a> DataProcessingEstimator<'a> {
    /// 创建新的数据处理估算器
    pub fn new(
        cost_calculator: &'a CostCalculator,
        selectivity_estimator: &'a SelectivityEstimator,
        config: CostModelConfig,
    ) -> Self {
        let expression_parser = ExpressionParser::new(config);
        Self {
            cost_calculator,
            selectivity_estimator,
            expression_parser,
        }
    }

    /// 计算过滤条件数量
    pub fn count_filter_conditions(&self, condition: &Expression) -> usize {
        match condition {
            Expression::Binary { op, left, right } => {
                match op {
                    BinaryOperator::And => {
                        self.count_filter_conditions(left) + self.count_filter_conditions(right)
                    }
                    BinaryOperator::Or => {
                        (self.count_filter_conditions(left) + self.count_filter_conditions(right)).max(1)
                    }
                    _ => 1,
                }
            }
            Expression::Unary { .. } => 1,
            Expression::Function { args, .. } => {
                args.iter().map(|_| 1).sum::<usize>().max(1)
            }
            _ => 1,
        }
    }

    /// 估算 Unwind 节点的列表大小
    fn estimate_unwind_list_size(
        &self,
        node: &crate::query::planner::plan::core::nodes::data_processing_node::UnwindNode,
    ) -> f64 {
        let list_expr = node.list_expression();

        // 尝试解析表达式推断列表大小
        if let Some(size) = self.expression_parser.parse_list_size(list_expr) {
            return size;
        }

        // 使用配置默认值
        self.expression_parser.config().default_unwind_list_size
    }
}

impl<'a> NodeEstimator for DataProcessingEstimator<'a> {
    fn estimate(
        &self,
        node: &PlanNodeEnum,
        child_estimates: &[NodeCostEstimate],
    ) -> Result<(f64, u64), CostError> {
        match node {
            PlanNodeEnum::Filter(n) => {
                let input_rows_val = get_input_rows(child_estimates, 0);
                let condition_expr = match n.condition().expression() {
                    Some(meta) => meta.inner().clone(),
                    None => return Ok((0.0, input_rows_val)),
                };
                let condition_count = self.count_filter_conditions(&condition_expr);
                // 估算过滤后的行数
                let selectivity = self.selectivity_estimator.estimate_from_expression(&condition_expr, None);
                let output_rows = (input_rows_val as f64 * selectivity).max(1.0) as u64;
                let cost = self.cost_calculator.calculate_filter_cost(input_rows_val, condition_count);
                Ok((cost, output_rows))
            }
            PlanNodeEnum::Project(n) => {
                let input_rows_val = get_input_rows(child_estimates, 0);
                let columns = n.columns().len();
                let cost = self.cost_calculator.calculate_project_cost(input_rows_val, columns);
                // Project 不改变行数
                Ok((cost, input_rows_val))
            }
            PlanNodeEnum::Unwind(n) => {
                let input_rows_val = get_input_rows(child_estimates, 0);
                let list_size = self.estimate_unwind_list_size(n);
                let cost = self.cost_calculator.calculate_unwind_cost(input_rows_val, list_size);
                // Unwind 将每行展开为列表大小行
                let output_rows = (input_rows_val as f64 * list_size) as u64;
                Ok((cost, output_rows.max(1)))
            }
            PlanNodeEnum::DataCollect(_) => {
                let input_rows_val = get_input_rows(child_estimates, 0);
                let cost = self.cost_calculator.calculate_data_collect_cost(input_rows_val);
                Ok((cost, input_rows_val))
            }
            PlanNodeEnum::Start(_) => Ok((0.0, 0)),
            _ => Err(CostError::UnsupportedNodeType(
                format!("数据处理估算器不支持节点类型: {:?}", std::mem::discriminant(node))
            )),
        }
    }
}
