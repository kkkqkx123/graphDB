//! 折叠多个投影操作的规则

use crate::core::YieldColumn;
use crate::core::types::expression::contextual::ContextualExpression;
use crate::core::types::expression::ExpressionMeta;
use crate::core::types::expression::ExpressionContext;
use crate::core::types::expression::Expression;
use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;
use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;
use crate::query::planner::plan::core::nodes::project_node::ProjectNode;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{MergeRule, RewriteRule};
use crate::query::planner::rewrite::expression_utils::rewrite_contextual_expression;
use std::sync::Arc;

/// 折叠多个投影操作的规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   Project(col2)
///       |
///   Project(col1)
///       |
///   ScanVertices
/// ```
///
/// After:
/// ```text
///   Project(col2)
///       |
///   ScanVertices
/// ```
///
/// # 适用条件
///
/// - 当前节点为Project节点
/// - 子节点也为Project节点
/// - 上层Project的列引用可以解析为下层Project的输入
#[derive(Debug)]
pub struct CollapseProjectRule;

impl CollapseProjectRule {
    /// 创建规则实例
    pub fn new() -> Self {
        Self
    }

    /// 检查表达式是否为简单的属性引用
    fn is_property_expr(expr: &ContextualExpression) -> bool {
        let expr_meta = match expr.expression() {
            Some(e) => e,
            None => return false,
        };
        let inner_expr = expr_meta.inner();
        matches!(inner_expr, Expression::Variable(_) | Expression::Property { .. })
    }

    /// 收集表达式中所有的属性引用
    fn collect_property_refs(expr: &ContextualExpression, refs: &mut Vec<String>) {
        let expr_meta = match expr.expression() {
            Some(e) => e,
            None => return,
        };
        let inner_expr = expr_meta.inner();
        
        match inner_expr {
            Expression::Variable(name) => refs.push(name.clone()),
            Expression::Property { object, property } => {
                if let Expression::Variable(obj_name) = object.as_ref() {
                    refs.push(format!("{}.{}", obj_name, property));
                } else {
                    refs.push(property.clone());
                }
            }
            Expression::Binary { left, right, .. } => {
                // 需要创建 ContextualExpression 来递归
                let left_meta = ExpressionMeta::new((**left).clone());
                let right_meta = ExpressionMeta::new((**right).clone());
                let left_ctx = Arc::new(ExpressionContext::new());
                let left_id = left_ctx.register_expression(left_meta);
                let right_id = left_ctx.register_expression(right_meta);
                let left_expr = ContextualExpression::new(left_id, left_ctx.clone());
                let right_expr = ContextualExpression::new(right_id, left_ctx);
                Self::collect_property_refs(&left_expr, refs);
                Self::collect_property_refs(&right_expr, refs);
            }
            Expression::Unary { operand, .. } => {
                let operand_meta = ExpressionMeta::new((**operand).clone());
                let ctx = Arc::new(ExpressionContext::new());
                let id = ctx.register_expression(operand_meta);
                let operand_expr = ContextualExpression::new(id, ctx);
                Self::collect_property_refs(&operand_expr, refs);
            }
            Expression::Function { args, .. } => {
                let ctx = Arc::new(ExpressionContext::new());
                for arg in args {
                    let arg_meta = ExpressionMeta::new(arg.clone());
                    let id = ctx.register_expression(arg_meta);
                    let arg_expr = ContextualExpression::new(id, ctx.clone());
                    Self::collect_property_refs(&arg_expr, refs);
                }
            }
            Expression::Aggregate { arg, .. } => {
                let arg_meta = ExpressionMeta::new((**arg).clone());
                let ctx = Arc::new(ExpressionContext::new());
                let id = ctx.register_expression(arg_meta);
                let arg_expr = ContextualExpression::new(id, ctx);
                Self::collect_property_refs(&arg_expr, refs);
            }
            Expression::List(list) => {
                let ctx = Arc::new(ExpressionContext::new());
                for item in list {
                    let item_meta = ExpressionMeta::new(item.clone());
                    let id = ctx.register_expression(item_meta);
                    let item_expr = ContextualExpression::new(id, ctx.clone());
                    Self::collect_property_refs(&item_expr, refs);
                }
            }
            Expression::Map(map) => {
                let ctx = Arc::new(ExpressionContext::new());
                for (_, value) in map {
                    let value_meta = ExpressionMeta::new(value.clone());
                    let id = ctx.register_expression(value_meta);
                    let value_expr = ContextualExpression::new(id, ctx.clone());
                    Self::collect_property_refs(&value_expr, refs);
                }
            }
            Expression::Case {
                test_expr,
                conditions,
                default,
            } => {
                let ctx = Arc::new(ExpressionContext::new());
                if let Some(test) = test_expr {
                    let test_meta = ExpressionMeta::new((**test).clone());
                    let id = ctx.register_expression(test_meta);
                    let test_expr = ContextualExpression::new(id, ctx.clone());
                    Self::collect_property_refs(&test_expr, refs);
                }
                for (when, then) in conditions {
                    let when_meta = ExpressionMeta::new(when.clone());
                    let then_meta = ExpressionMeta::new(then.clone());
                    let when_id = ctx.register_expression(when_meta);
                    let then_id = ctx.register_expression(then_meta);
                    let when_expr = ContextualExpression::new(when_id, ctx.clone());
                    let then_expr = ContextualExpression::new(then_id, ctx.clone());
                    Self::collect_property_refs(&when_expr, refs);
                    Self::collect_property_refs(&then_expr, refs);
                }
                if let Some(else_e) = default {
                    let else_meta = ExpressionMeta::new((**else_e).clone());
                    let id = ctx.register_expression(else_meta);
                    let else_expr = ContextualExpression::new(id, ctx);
                    Self::collect_property_refs(&else_expr, refs);
                }
            }
            _ => {}
        }
    }

}

impl Default for CollapseProjectRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RewriteRule for CollapseProjectRule {
    fn name(&self) -> &'static str {
        "CollapseProjectRule"
    }

    fn pattern(&self) -> Pattern {
        Pattern::new_with_name("Project").with_dependency_name("Project")
    }

    fn apply(
        &self,
        ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        let parent_proj = match node {
            PlanNodeEnum::Project(n) => n,
            _ => return Ok(None),
        };

        let child_node = parent_proj.input();
        let child_proj = match child_node {
            PlanNodeEnum::Project(n) => n,
            _ => return Ok(None),
        };

        let parent_cols = parent_proj.columns();
        let child_cols = child_proj.columns();

        // 收集上层Project中所有的属性引用
        let mut all_prop_refs: Vec<String> = Vec::new();
        for col in parent_cols {
            Self::collect_property_refs(&col.expression, &mut all_prop_refs);
        }

        // 检查是否有重复引用
        let mut unique_refs = std::collections::HashSet::new();
        let mut multi_ref_cols = std::collections::HashSet::new();
        for prop_ref in &all_prop_refs {
            if !unique_refs.insert(prop_ref.clone()) {
                multi_ref_cols.insert(prop_ref.clone());
            }
        }

        // 构建重写映射：列名 -> ContextualExpression
        let mut rewrite_map = std::collections::HashMap::new();
        let child_col_names = child_proj.col_names();

        for (i, col_name) in child_col_names.iter().enumerate() {
            if unique_refs.contains(col_name) {
                let col_expr = &child_cols[i].expression;
                // 如果列被多次引用且不是简单属性表达式，则禁用此优化
                if !Self::is_property_expr(col_expr) && multi_ref_cols.contains(col_name) {
                    return Ok(None);
                }
                rewrite_map.insert(col_name.clone(), col_expr.clone());
            }
        }

        let expr_context = ctx.expr_context();

        // 重写上层Project的列
        let new_columns: Vec<YieldColumn> = parent_cols
            .iter()
            .map(|col| YieldColumn {
                expression: rewrite_contextual_expression(&col.expression, &rewrite_map, expr_context.clone()),
                alias: col.alias.clone(),
                is_matched: col.is_matched,
            })
            .collect();

        // 创建新的Project节点，输入为下层Project的输入
        let new_input = child_proj.input().clone();
        let new_proj = match ProjectNode::new(new_input, new_columns) {
            Ok(node) => node,
            Err(_) => return Ok(None),
        };

        let mut result = TransformResult::new();
        result.erase_curr = true;
        result.add_new_node(PlanNodeEnum::Project(new_proj));

        Ok(Some(result))
    }
}

impl MergeRule for CollapseProjectRule {
    fn can_merge(&self, parent: &PlanNodeEnum, child: &PlanNodeEnum) -> bool {
        parent.is_project() && child.is_project()
    }

    fn create_merged_node(
        &self,
        ctx: &mut RewriteContext,
        parent: &PlanNodeEnum,
        _child: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        self.apply(ctx, parent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::YieldColumn;
    use crate::query::planner::plan::core::nodes::start_node::StartNode;
    use crate::core::types::expression::ExpressionMeta;
    use crate::core::types::expression::ExpressionContext;
    use crate::core::types::expression::ExpressionId;
    use std::sync::Arc;

    #[test]
    fn test_rule_name() {
        let rule = CollapseProjectRule::new();
        assert_eq!(rule.name(), "CollapseProjectRule");
    }

    #[test]
    fn test_rule_pattern() {
        let rule = CollapseProjectRule::new();
        let pattern = rule.pattern();
        assert!(pattern.node.is_some());
    }

    #[test]
    fn test_collapse_simple_project() {
        let start = PlanNodeEnum::Start(StartNode::new());
        let expr_ctx = Arc::new(ExpressionContext::new());

        // 下层Project: col1
        let child_expr = Expression::Variable("a".to_string());
        let child_meta = ExpressionMeta::new(child_expr);
        let child_id = expr_ctx.register_expression(child_meta);
        let child_ctx_expr = ContextualExpression::new(child_id, expr_ctx.clone());
        
        let child_columns = vec![YieldColumn {
            expression: child_ctx_expr,
            alias: "col1".to_string(),
            is_matched: false,
        }];
        let child_proj = ProjectNode::new(start, child_columns).expect("创建ProjectNode失败");
        let child_node = PlanNodeEnum::Project(child_proj);

        // 上层Project: col2 = col1
        let parent_expr = Expression::Variable("col1".to_string());
        let parent_meta = ExpressionMeta::new(parent_expr);
        let parent_id = expr_ctx.register_expression(parent_meta);
        let parent_ctx_expr = ContextualExpression::new(parent_id, expr_ctx);
        
        let parent_columns = vec![YieldColumn {
            expression: parent_ctx_expr,
            alias: "col2".to_string(),
            is_matched: false,
        }];
        let parent_proj =
            ProjectNode::new(child_node.clone(), parent_columns).expect("创建ProjectNode失败");
        let parent_node = PlanNodeEnum::Project(parent_proj);

        // 应用规则
        let rule = CollapseProjectRule::new();
        let mut ctx = RewriteContext::new();
        let result = rule.apply(&mut ctx, &parent_node).expect("应用规则失败");

        assert!(
            result.is_some(),
            "应该成功折叠两个Project节点"
        );
    }
}
