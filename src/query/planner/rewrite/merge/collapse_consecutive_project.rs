//! 合并连续投影规则
//!
//! 当多个 Project 节点连续出现时，合并为一个 Project 节点
//! 减少不必要的中间结果生成
//!
//! 示例:
//! ```
//! Project(a, b) -> Project(c, d)  =>  Project(c, d)
//! ```
//!
//! 适用条件:
//! - 两个 Project 节点连续出现
//! - 上层 Project 不依赖下层 Project 的别名解析

use crate::core::YieldColumn;
use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;
use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;
use crate::query::planner::plan::core::nodes::project_node::ProjectNode;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{MergeRule, RewriteRule};
use crate::query::planner::rewrite::expression_utils::rewrite_contextual_expression;
use std::collections::HashMap;

/// 合并连续投影规则
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
pub struct CollapseConsecutiveProjectRule;

impl CollapseConsecutiveProjectRule {
    /// 创建规则实例
    pub fn new() -> Self {
        Self
    }


    /// 执行合并操作
    fn merge_projects(
        &self,
        parent_proj: &ProjectNode,
        child_proj: &ProjectNode,
        ctx: &RewriteContext,
    ) -> Option<ProjectNode> {
        // 构建列名到表达式的映射（从子Project）
        let mut rewrite_map = HashMap::new();
        for col in child_proj.columns() {
            if !col.alias.is_empty() {
                rewrite_map.insert(col.alias.clone(), col.expression.clone());
            }
        }

        let expr_context = ctx.expr_context();

        // 重写父Project的列表达式
        let new_columns: Vec<YieldColumn> = parent_proj
            .columns()
            .iter()
            .map(|col| YieldColumn {
                expression: rewrite_contextual_expression(&col.expression, &rewrite_map, expr_context.clone()),
                alias: col.alias.clone(),
                is_matched: col.is_matched,
            })
            .collect();

        // 创建新的Project节点，输入为子Project的输入
        let child_input = child_proj.input().clone();
        ProjectNode::new(child_input, new_columns).ok()
    }
}

impl Default for CollapseConsecutiveProjectRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RewriteRule for CollapseConsecutiveProjectRule {
    fn name(&self) -> &'static str {
        "CollapseConsecutiveProjectRule"
    }

    fn pattern(&self) -> Pattern {
        Pattern::new_with_name("Project").with_dependency_name("Project")
    }

    fn apply(
        &self,
        _ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        // 检查是否为Project节点
        let parent_proj = match node {
            PlanNodeEnum::Project(n) => n,
            _ => return Ok(None),
        };

        // 获取子节点
        let child_node = parent_proj.input();
        let child_proj = match child_node {
            PlanNodeEnum::Project(n) => n,
            _ => return Ok(None),
        };

        // 执行合并
        if let Some(new_proj) = self.merge_projects(parent_proj, child_proj) {
            let mut result = TransformResult::new();
            result.erase_curr = true;
            result.add_new_node(PlanNodeEnum::Project(new_proj));
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }
}

impl MergeRule for CollapseConsecutiveProjectRule {
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
    use crate::core::Expression;
    use crate::query::planner::plan::core::nodes::start_node::StartNode;

    #[test]
    fn test_rule_name() {
        let rule = CollapseConsecutiveProjectRule::new();
        assert_eq!(rule.name(), "CollapseConsecutiveProjectRule");
    }

    #[test]
    fn test_rule_pattern() {
        let rule = CollapseConsecutiveProjectRule::new();
        let pattern = rule.pattern();
        assert!(pattern.node.is_some());
    }

    #[test]
    fn test_collapse_consecutive_projects() {
        use std::sync::Arc;
        use crate::core::types::expression::ExpressionMeta;
        use crate::core::types::expression::ExpressionContext;
        use crate::core::types::expression::ExpressionId;
        
        // 创建起始节点
        let start = PlanNodeEnum::Start(StartNode::new());

        // 创建表达式上下文
        let expr_ctx = Arc::new(ExpressionContext::new());

        // 创建下层Project节点
        let a_expr = Expression::Variable("a".to_string());
        let a_meta = ExpressionMeta::new(a_expr);
        let a_id = expr_ctx.register_expression(a_meta);
        let a_ctx_expr = ContextualExpression::new(a_id, expr_ctx.clone());
        
        let b_expr = Expression::Variable("b".to_string());
        let b_meta = ExpressionMeta::new(b_expr);
        let b_id = expr_ctx.register_expression(b_meta);
        let b_ctx_expr = ContextualExpression::new(b_id, expr_ctx.clone());
        
        let child_columns = vec![
            YieldColumn {
                expression: a_ctx_expr,
                alias: "col_a".to_string(),
                is_matched: false,
            },
            YieldColumn {
                expression: b_ctx_expr,
                alias: "col_b".to_string(),
                is_matched: false,
            },
        ];
        let child_proj = ProjectNode::new(start, child_columns).expect("创建下层Project失败");
        let child_node = PlanNodeEnum::Project(child_proj);

        // 创建上层Project节点，引用下层Project的别名
        let col_a_expr = Expression::Variable("col_a".to_string());
        let col_a_meta = ExpressionMeta::new(col_a_expr);
        let col_a_id = expr_ctx.register_expression(col_a_meta);
        let col_a_ctx_expr = ContextualExpression::new(col_a_id, expr_ctx);
        
        let parent_columns = vec![YieldColumn {
            expression: col_a_ctx_expr,
            alias: "result".to_string(),
            is_matched: false,
        }];
        let parent_proj = ProjectNode::new(child_node, parent_columns).expect("创建上层Project失败");
        let parent_node = PlanNodeEnum::Project(parent_proj);

        // 应用规则
        let rule = CollapseConsecutiveProjectRule::new();
        let mut ctx = RewriteContext::new();
        let result = rule.apply(&mut ctx, &parent_node).expect("应用规则失败");

        assert!(
            result.is_some(),
            "应该成功合并连续的Project节点"
        );

        // 验证结果
        let transform_result = result.expect("Failed to apply rewrite rule");
        assert!(transform_result.erase_curr);
        assert_eq!(transform_result.new_nodes.len(), 1);

        // 验证新的Project节点
        if let PlanNodeEnum::Project(ref new_proj) = transform_result.new_nodes[0] {
            let columns = new_proj.columns();
            assert_eq!(columns.len(), 1);
            assert_eq!(columns[0].alias, "result");
            // 验证表达式已被重写为原始引用
            if let Some(expr_meta) = columns[0].expression.expression() {
                if let Expression::Variable(name) = expr_meta.inner() {
                    assert_eq!(name, "a");
                } else {
                    panic!("表达式应该是Variable");
                }
            } else {
                panic!("表达式应该存在");
            }
        } else {
            panic!("转换结果应该是Project节点");
        }
    }
}
