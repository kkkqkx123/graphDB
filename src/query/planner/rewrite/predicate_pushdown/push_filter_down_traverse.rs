//! 将过滤条件下推到遍历操作的规则
//!
//! 该规则识别 Filter -> Traverse 模式，
//! 并将边属性过滤条件下推到 Traverse 节点中。

use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{RewriteRule, PushDownRule};
use crate::core::Expression;
use crate::query::optimizer::expression_utils::split_filter;
use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;

/// 将过滤条件下推到遍历操作的规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   Filter(e.likeness > 78)
///           |
///   AppendVertices
///           |
///   Traverse
/// ```
///
/// After:
/// ```text
///   AppendVertices
///           |
///   Traverse(eFilter: *.likeness > 78)
/// ```
///
/// # 适用条件
///
/// - 过滤条件包含边属性表达式
/// - Traverse 节点为单步遍历
#[derive(Debug)]
pub struct PushFilterDownTraverseRule;

impl PushFilterDownTraverseRule {
    /// 创建规则实例
    pub fn new() -> Self {
        Self
    }
}

impl Default for PushFilterDownTraverseRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RewriteRule for PushFilterDownTraverseRule {
    fn name(&self) -> &'static str {
        "PushFilterDownTraverseRule"
    }

    fn pattern(&self) -> Pattern {
        Pattern::new_with_name("Filter").with_dependency_name("Traverse")
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

        // 获取输入节点
        let input = filter_node.input();

        // 检查输入节点是否为 Traverse
        let traverse = match input {
            PlanNodeEnum::Traverse(t) => t,
            _ => return Ok(None),
        };

        // 检查是否为单步遍历
        if !traverse.is_one_step() {
            return Ok(None);
        }

        // 获取边别名
        let edge_alias = match traverse.edge_alias() {
            Some(alias) => alias,
            None => return Ok(None),
        };

        // 获取过滤条件
        let filter_condition = filter_node.condition();

        // 定义选择器函数：检查表达式是否包含边属性
        let picker = |expr: &Expression| -> bool {
            is_edge_property_expression(edge_alias, expr)
        };

        // 分割过滤条件
        let (filter_picked, filter_unpicked) = split_filter(filter_condition, picker);

        // 如果没有可以选择的条件，则不进行转换
        let picked = match filter_picked {
            Some(f) => f,
            None => return Ok(None),
        };

        // 创建新的 Traverse 节点
        let mut new_traverse = traverse.clone();

        // 设置或合并 eFilter
        if let Some(existing) = traverse.e_filter() {
            let combined = Expression::Binary {
                left: Box::new(picked),
                op: crate::core::types::operators::BinaryOperator::And,
                right: Box::new(existing.clone()),
            };
            new_traverse.set_e_filter(combined);
        } else {
            new_traverse.set_e_filter(picked);
        }

        // 构建转换结果
        let mut result = TransformResult::new();

        // 如果有未选择的过滤条件，保留 Filter 节点
        if let Some(unpicked) = filter_unpicked {
            result.erase_curr = false;
            // 更新 Filter 节点的条件
            let mut new_filter = filter_node.clone();
            new_filter.set_condition(unpicked);
            result.add_new_node(PlanNodeEnum::Filter(new_filter));
        } else {
            // 完全下推，删除 Filter 节点
            result.erase_curr = true;
        }

        result.add_new_node(PlanNodeEnum::Traverse(new_traverse));

        Ok(Some(result))
    }
}

impl PushDownRule for PushFilterDownTraverseRule {
    fn can_push_down(&self, node: &PlanNodeEnum, target: &PlanNodeEnum) -> bool {
        match (node, target) {
            (PlanNodeEnum::Filter(_), PlanNodeEnum::Traverse(traverse)) => {
                traverse.is_one_step() && traverse.edge_alias().is_some()
            }
            _ => false,
        }
    }

    fn push_down(
        &self,
        ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
        _target: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        self.apply(ctx, node)
    }
}

/// 检查表达式是否为边属性表达式
fn is_edge_property_expression(edge_alias: &str, expr: &Expression) -> bool {
    match expr {
        Expression::Property { object, property: _ } => {
            if let Expression::Variable(name) = object.as_ref() {
                name == edge_alias
            } else {
                is_edge_property_expression(edge_alias, object)
            }
        }
        Expression::Binary { left, op: _, right } => {
            is_edge_property_expression(edge_alias, left)
                || is_edge_property_expression(edge_alias, right)
        }
        Expression::Unary { op: _, operand } => {
            is_edge_property_expression(edge_alias, operand)
        }
        Expression::Function { name: _, args } => {
            args.iter().any(|arg| is_edge_property_expression(edge_alias, arg))
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_name() {
        let rule = PushFilterDownTraverseRule::new();
        assert_eq!(rule.name(), "PushFilterDownTraverseRule");
    }

    #[test]
    fn test_rule_pattern() {
        let rule = PushFilterDownTraverseRule::new();
        let pattern = rule.pattern();
        assert!(pattern.node.is_some());
    }

    #[test]
    fn test_is_edge_property_expression() {
        let edge_alias = "e";

        // 测试边属性表达式
        let prop_expr = Expression::Property {
            object: Box::new(Expression::Variable("e".to_string())),
            property: "likeness".to_string(),
        };
        assert!(is_edge_property_expression(edge_alias, &prop_expr));

        // 测试非边属性表达式
        let var_expr = Expression::Variable("v".to_string());
        assert!(!is_edge_property_expression(edge_alias, &var_expr));
    }
}
