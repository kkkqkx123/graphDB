//! 过滤下推到聚合节点的优化规则
//!
//! 此规则将过滤操作下推到聚合节点之前执行，以减少进入聚合的数据量。
//!
//! # 转换示例
//!
//! Before:
//! ```text
//! Filter(condition)
//!       |
//!   Aggregate(group_keys, agg_funcs)
//!       |
//!     Input
//! ```
//!
//! After:
//! ```text
//! Aggregate(group_keys, agg_funcs)
//!             |
//!       Filter(condition)
//!             |
//!           Input
//! ```
//!
//! # 适用条件
//!
//! - Filter 节点的子节点是 Aggregate 节点
//! - Filter 条件不涉及聚合函数（只涉及聚合的输入列）

use crate::query::optimizer::plan::{OptContext, OptGroupNode, OptRule, Pattern, TransformResult, Result};
use crate::query::optimizer::rule_patterns::PatternBuilder;
use crate::query::optimizer::rule_traits::BaseOptRule;
use crate::core::Expression;
use crate::query::planner::plan::core::nodes::filter_node::FilterNode;
use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;
use crate::query::planner::plan::PlanNodeEnum;
use std::cell::RefCell;
use std::rc::Rc;

/// 将过滤下推到聚合之前的规则
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
    ) -> Result<Option<TransformResult>> {
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

        if node_ref.dependencies.is_empty() {
            return Ok(None);
        }

        let agg_child_id = node_ref.dependencies[0];
        let Some(agg_child) = ctx.find_group_node_by_plan_node_id(agg_child_id) else {
            return Ok(None);
        };

        let agg_child_ref = agg_child.borrow();
        let agg_node = match &agg_child_ref.plan_node {
            PlanNodeEnum::Aggregate(n) => n,
            _ => return Ok(None),
        };

        let group_keys = agg_node.group_keys();
        let agg_funcs = agg_node.aggregation_functions();

        if Self::has_aggregate_function_reference(filter_condition, group_keys, agg_funcs) {
            return Ok(None);
        }

        if agg_child_ref.dependencies.is_empty() {
            return Ok(None);
        }

        let input_id = agg_child_ref.dependencies[0];
        let Some(input_child) = ctx.find_group_node_by_plan_node_id(input_id) else {
            return Ok(None);
        };

        let input_child_ref = input_child.borrow();
        let input_plan_node = input_child_ref.plan_node.as_aggregate()
            .and_then(|agg| agg.dependencies().first())
            .map(|p| p.as_ref().clone());

        if input_plan_node.is_none() {
            return Ok(None);
        }

        let input_plan_node = match input_plan_node {
            Some(node) => node,
            None => return Ok(None),
        };

        let new_condition = Self::rewrite_filter_condition(filter_condition, group_keys);

        let mut new_filter_node = match FilterNode::new(
            input_plan_node,
            new_condition,
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

        let mut new_agg_node = agg_node.clone();
        new_agg_node.set_input(PlanNodeEnum::Filter(new_filter_node.clone()));

        if let Some(output_var) = agg_node.output_var() {
            new_agg_node.set_output_var(output_var.clone());
        }
        new_agg_node.set_col_names(agg_node.col_names().to_vec());

        let mut new_agg_group_node = agg_child_ref.clone();
        new_agg_group_node.plan_node = PlanNodeEnum::Aggregate(new_agg_node);
        new_agg_group_node.dependencies = vec![input_id];

        let mut new_filter_group_node = input_child_ref.clone();
        new_filter_group_node.plan_node = PlanNodeEnum::Filter(new_filter_node);
        new_filter_group_node.dependencies = vec![input_id];

        drop(node_ref);
        drop(agg_child_ref);
        drop(input_child_ref);

        let mut result = TransformResult::new();
        result.add_new_group_node(Rc::new(RefCell::new(new_agg_group_node)));
        result.erase_all = true;

        Ok(Some(result))
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::with_dependency("Filter", "Aggregate")
    }
}

impl BaseOptRule for PushFilterDownAggregateRule {}

impl PushFilterDownAggregateRule {
    fn has_aggregate_function_reference(
        condition: &Expression,
        group_keys: &[String],
        agg_funcs: &[crate::core::types::operators::AggregateFunction],
    ) -> bool {
        fn check_expr(expr: &Expression, group_keys: &[String], agg_funcs: &[crate::core::types::operators::AggregateFunction]) -> bool {
            match expr {
                Expression::Aggregate { .. } => true,
                Expression::Binary { left, right, .. } => {
                    check_expr(left, group_keys, agg_funcs) || check_expr(right, group_keys, agg_funcs)
                }
                Expression::Unary { operand, .. } => check_expr(operand, group_keys, agg_funcs),
                Expression::Property { object, .. } => check_expr(object, group_keys, agg_funcs),
                Expression::Function { name, args, .. } => {
                    let func_name = name.to_lowercase();
                    matches!(
                        func_name.as_str(),
                        "sum" | "avg" | "count" | "max" | "min" | "collect"
                    ) || args.iter().any(|arg| check_expr(arg, group_keys, agg_funcs))
                }
                Expression::Variable(name) => {
                    !group_keys.contains(name)
                }
                _ => false,
            }
        }

        check_expr(condition, group_keys, agg_funcs)
    }

    fn rewrite_filter_condition(condition: &Expression, group_keys: &[String]) -> Expression {
        fn rewrite(expr: &Expression, group_keys: &[String]) -> Expression {
            match expr {
                Expression::Binary { op, left, right } => {
                    Expression::Binary {
                        op: op.clone(),
                        left: Box::new(rewrite(left, group_keys)),
                        right: Box::new(rewrite(right, group_keys)),
                    }
                }
                Expression::Unary { op, operand } => {
                    Expression::Unary {
                        op: op.clone(),
                        operand: Box::new(rewrite(operand, group_keys)),
                    }
                }
                Expression::Property { object, property } => {
                    Expression::Property {
                        object: Box::new(rewrite(object, group_keys)),
                        property: property.clone(),
                    }
                }
                Expression::Function { name, args } => {
                    Expression::Function {
                        name: name.clone(),
                        args: args.iter().map(|arg| rewrite(arg, group_keys)).collect(),
                    }
                }
                Expression::Variable(name) => {
                    if group_keys.contains(name) {
                        Expression::Variable(name.clone())
                    } else {
                        expr.clone()
                    }
                }
                _ => expr.clone(),
            }
        }

        rewrite(condition, group_keys)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Expression;
    use crate::core::types::operators::AggregateFunction;

    #[test]
    fn test_has_aggregate_function_reference_with_aggregate() {
        let condition = Expression::Aggregate {
            func: AggregateFunction::Count(None),
            arg: Box::new(Expression::Variable("amount".to_string())),
            distinct: false,
        };

        assert!(PushFilterDownAggregateRule::has_aggregate_function_reference(
            &condition,
            &[],
            &[AggregateFunction::Count(None)]
        ));
    }

    #[test]
    fn test_no_aggregate_function_reference() {
        let condition = Expression::Binary {
            op: crate::core::BinaryOperator::Equal,
            left: Box::new(Expression::Variable("name".to_string())),
            right: Box::new(Expression::Literal(crate::core::Value::String("test".to_string()))),
        };

        assert!(!PushFilterDownAggregateRule::has_aggregate_function_reference(
            &condition,
            &["name".to_string()],
            &[]
        ));
    }

    #[test]
    fn test_has_aggregate_function_reference_with_function() {
        let condition = Expression::Function {
            name: "sum".to_string(),
            args: vec![Expression::Variable("amount".to_string())],
        };

        assert!(PushFilterDownAggregateRule::has_aggregate_function_reference(
            &condition,
            &[],
            &[AggregateFunction::Sum("amount".to_string())]
        ));
    }

    #[test]
    fn test_rewrite_filter_condition() {
        let condition = Expression::Binary {
            op: crate::core::BinaryOperator::Equal,
            left: Box::new(Expression::Variable("name".to_string())),
            right: Box::new(Expression::Literal(crate::core::Value::String("test".to_string()))),
        };

        let rewritten = PushFilterDownAggregateRule::rewrite_filter_condition(
            &condition,
            &["name".to_string()]
        );

        assert_eq!(rewritten, condition);
    }
}
