//! 折叠多个投影操作的规则

use crate::core::{Expression, YieldColumn};
use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;
use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;
use crate::query::planner::plan::core::nodes::project_node::ProjectNode;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{MergeRule, RewriteRule};
use crate::query::planner::rewrite::expression_utils::rewrite_expression;

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
    fn is_property_expr(expr: &Expression) -> bool {
        matches!(expr, Expression::Variable(_) | Expression::Property { .. })
    }

    /// 收集表达式中所有的属性引用
    fn collect_property_refs(expr: &Expression, refs: &mut Vec<String>) {
        match expr {
            Expression::Variable(name) => refs.push(name.clone()),
            Expression::Property { object, property } => {
                if let Expression::Variable(obj_name) = object.as_ref() {
                    refs.push(format!("{}.{}", obj_name, property));
                } else {
                    refs.push(property.clone());
                }
            }
            Expression::Binary { left, right, .. } => {
                Self::collect_property_refs(left, refs);
                Self::collect_property_refs(right, refs);
            }
            Expression::Unary { operand, .. } => {
                Self::collect_property_refs(operand, refs);
            }
            Expression::Function { args, .. } => {
                for arg in args {
                    Self::collect_property_refs(arg, refs);
                }
            }
            Expression::Aggregate { arg, .. } => {
                Self::collect_property_refs(arg, refs);
            }
            Expression::List(list) => {
                for item in list {
                    Self::collect_property_refs(item, refs);
                }
            }
            Expression::Map(map) => {
                for (_, value) in map {
                    Self::collect_property_refs(value, refs);
                }
            }
            Expression::Case {
                test_expr,
                conditions,
                default,
            } => {
                if let Some(test) = test_expr {
                    Self::collect_property_refs(test, refs);
                }
                for (when, then) in conditions {
                    Self::collect_property_refs(when, refs);
                    Self::collect_property_refs(then, refs);
                }
                if let Some(else_e) = default {
                    Self::collect_property_refs(else_e, refs);
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
        _ctx: &mut RewriteContext,
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

        // 构建重写映射：列名 -> 表达式
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

        // 重写上层Project的列
        let new_columns: Vec<YieldColumn> = parent_cols
            .iter()
            .map(|col| YieldColumn {
                expression: rewrite_expression(&col.expression, &rewrite_map),
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

        // 下层Project: col1
        let child_columns = vec![YieldColumn {
            expression: Expression::Variable("a".to_string()),
            alias: "col1".to_string(),
            is_matched: false,
        }];
        let child_proj = ProjectNode::new(start, child_columns).expect("创建ProjectNode失败");
        let child_node = PlanNodeEnum::Project(child_proj);

        // 上层Project: col2 = col1
        let parent_columns = vec![YieldColumn {
            expression: Expression::Variable("col1".to_string()),
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
