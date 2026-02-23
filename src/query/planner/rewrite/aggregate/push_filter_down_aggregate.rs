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

use crate::query::planner::plan::PlanNodeEnum;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{RewriteRule, PushDownRule};
use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;
use crate::core::Expression;
use crate::core::types::operators::AggregateFunction;

/// 将过滤下推到聚合之前的规则
#[derive(Debug)]
pub struct PushFilterDownAggregateRule;

impl PushFilterDownAggregateRule {
    /// 创建规则实例
    pub fn new() -> Self {
        Self
    }

    /// 检查条件是否包含聚合函数引用
    fn has_aggregate_function_reference(
        condition: &Expression,
        group_keys: &[String],
        agg_funcs: &[AggregateFunction],
    ) -> bool {
        fn check_expr(expr: &Expression, group_keys: &[String], agg_funcs: &[AggregateFunction]) -> bool {
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

    /// 重写过滤条件
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

impl Default for PushFilterDownAggregateRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RewriteRule for PushFilterDownAggregateRule {
    fn name(&self) -> &'static str {
        "PushFilterDownAggregateRule"
    }

    fn pattern(&self) -> Pattern {
        Pattern::new_with_name("Filter").with_dependency_name("Aggregate")
    }

    fn apply(
        &self,
        _ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        // 检查是否为 Filter 节点
        let filter_node = match node {
            PlanNodeEnum::Filter(n) => n,
            _ => return Ok(None),
        };

        // 获取过滤条件
        let filter_condition = filter_node.condition();

        // 获取输入节点
        let input = filter_node.input();

        // 检查输入节点是否为 Aggregate
        let agg_node = match input {
            PlanNodeEnum::Aggregate(n) => n,
            _ => return Ok(None),
        };

        // 获取聚合的分组键和聚合函数
        let group_keys = agg_node.group_keys();
        let agg_funcs = agg_node.aggregation_functions();

        // 检查过滤条件是否包含聚合函数引用
        if Self::has_aggregate_function_reference(filter_condition, group_keys, agg_funcs) {
            return Ok(None);
        }

        // 简化实现：返回 None 表示不转换
        // 实际实现需要创建新的 Aggregate 节点并在其输入上添加 Filter
        Ok(None)
    }
}

impl PushDownRule for PushFilterDownAggregateRule {
    fn can_push_down(&self, node: &PlanNodeEnum, target: &PlanNodeEnum) -> bool {
        matches!((node, target), (PlanNodeEnum::Filter(_), PlanNodeEnum::Aggregate(_)))
    }

    fn push_down(
        &self,
        _ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
        _target: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        self.apply(_ctx, node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_name() {
        let rule = PushFilterDownAggregateRule::new();
        assert_eq!(rule.name(), "PushFilterDownAggregateRule");
    }

    #[test]
    fn test_rule_pattern() {
        let rule = PushFilterDownAggregateRule::new();
        let pattern = rule.pattern();
        assert!(pattern.node.is_some());
    }

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
