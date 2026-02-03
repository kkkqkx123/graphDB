//! 过滤下推到聚合节点的优化规则
//!
//! 此规则将过滤操作下推到聚合节点之前执行，以减少进入聚合的数据量

use super::plan::{OptContext, OptGroupNode, OptRule, Pattern, TransformResult, Result as OptResult};
use super::rule_patterns::PatternBuilder;
use super::rule_traits::BaseOptRule;
use crate::core::Expression;
use crate::query::planner::plan::core::nodes::filter_node::FilterNode;
use crate::query::planner::plan::PlanNodeEnum;
use std::cell::RefCell;
use std::rc::Rc;

/// 将过滤下推到聚合之前的规则
///
/// 模式: Filter -> Aggregate -> X
/// 如果 Filter 的条件只涉及聚合的输入列（不涉及聚合函数），可以将 Filter 下推到 Aggregate 之前
#[derive(Debug)]
pub struct PushFilterDownAggregateRule;

impl OptRule for PushFilterDownAggregateRule {
    fn name(&self) -> &str {
        "PushFilterDownAggregateRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        node: &Rc<RefCell<OptGroupNode>>,
    ) -> OptResult<Option<TransformResult>> {
        let node_ref = node.borrow();
        let plan_node = &node_ref.plan_node;

        if !plan_node.is_filter() {
            return Ok(None);
        }

        let filter_node = match plan_node {
            PlanNodeEnum::Filter(n) => n,
            _ => return Ok(None),
        };

        let filter_condition = filter_node.condition();

        if Self::has_aggregate_function_reference(filter_condition) {
            return Ok(None);
        }

        if node_ref.dependencies.is_empty() {
            return Ok(None);
        }

        let agg_child_id = node_ref.dependencies[0];
        let Some(agg_child) = ctx.find_group_node_by_plan_node_id(agg_child_id) else {
            return Ok(None);
        };

        let agg_child_ref = agg_child.borrow();
        let _agg_node = match &agg_child_ref.plan_node {
            PlanNodeEnum::Aggregate(n) => n,
            _ => return Ok(None),
        };

        if agg_child_ref.dependencies.is_empty() {
            return Ok(None);
        }

        let input_id = agg_child_ref.dependencies[0];

        let input_plan_node = agg_child_ref.plan_node.as_aggregate()
            .and_then(|agg| agg.dependencies().first())
            .map(|p| p.as_ref().clone());

        if input_plan_node.is_none() {
            return Ok(None);
        }

        let input_plan_node = input_plan_node.unwrap();

        let mut new_filter_node = match FilterNode::new(
            input_plan_node,
            filter_condition.clone(),
        ) {
            Ok(n) => n,
            Err(_) => {
                return Ok(None);
            }
        };

        if let Some(output_var) = filter_node.output_var() {
            new_filter_node.set_output_var(output_var.clone());
        }
        new_filter_node.set_col_names(filter_node.col_names().to_vec());

        let mut new_agg_node = agg_child_ref.plan_node.as_aggregate().unwrap().clone();
        new_agg_node.add_dependency(PlanNodeEnum::Filter(new_filter_node.clone()));

        if let Some(output_var) = agg_child_ref.plan_node.as_aggregate().unwrap().output_var() {
            new_agg_node.set_output_var(output_var.clone());
        }
        new_agg_node.set_col_names(agg_child_ref.plan_node.as_aggregate().unwrap().col_names().to_vec());

        let mut new_agg_group_node = agg_child_ref.clone();
        new_agg_group_node.plan_node = PlanNodeEnum::Aggregate(new_agg_node);
        new_agg_group_node.dependencies = vec![input_id];

        drop(node_ref);
        drop(agg_child_ref);

        let mut new_filter_group_node = node.borrow().clone();
        new_filter_group_node.plan_node = PlanNodeEnum::Filter(new_filter_node);
        new_filter_group_node.dependencies = vec![agg_child_id];

        ctx.add_plan_node_and_group_node(agg_child_id, Rc::new(RefCell::new(new_agg_group_node)));

        let mut result = TransformResult::new();
        result.add_new_group_node(Rc::new(RefCell::new(new_filter_group_node)));
        Ok(Some(result))
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::with_dependency("Filter", "Aggregate")
    }
}

impl BaseOptRule for PushFilterDownAggregateRule {}

impl PushFilterDownAggregateRule {
    fn has_aggregate_function_reference(condition: &Expression) -> bool {
        fn check_expr(expr: &Expression) -> bool {
            match expr {
                Expression::Aggregate { .. } => true,
                Expression::Binary { left, right, .. } => check_expr(left) || check_expr(right),
                Expression::Unary { operand, .. } => check_expr(operand),
                Expression::Property { object, .. } => check_expr(object),
                Expression::Function { name, args, .. } => {
                    let func_name = name.to_lowercase();
                    matches!(
                        func_name.as_str(),
                        "sum" | "avg" | "count" | "max" | "min" | "collect"
                    ) || args.iter().any(check_expr)
                }
                _ => false,
            }
        }

        check_expr(condition)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Expression;

    #[test]
    fn test_has_aggregate_function_reference_with_aggregate() {
        let condition = Expression::Aggregate {
            func: crate::core::types::operators::AggregateFunction::Count(None),
            arg: Box::new(Expression::Variable("amount".to_string())),
            distinct: false,
        };

        assert!(PushFilterDownAggregateRule::has_aggregate_function_reference(&condition));
    }

    #[test]
    fn test_no_aggregate_function_reference() {
        let condition = Expression::Binary {
            op: crate::core::BinaryOperator::Equal,
            left: Box::new(Expression::Variable("name".to_string())),
            right: Box::new(Expression::Literal(crate::core::Value::String("test".to_string()))),
        };

        assert!(!PushFilterDownAggregateRule::has_aggregate_function_reference(&condition));
    }

    #[test]
    fn test_has_aggregate_function_reference_with_function() {
        let condition = Expression::Function {
            name: "sum".to_string(),
            args: vec![Expression::Variable("amount".to_string())],
        };

        assert!(PushFilterDownAggregateRule::has_aggregate_function_reference(&condition));
    }
}
