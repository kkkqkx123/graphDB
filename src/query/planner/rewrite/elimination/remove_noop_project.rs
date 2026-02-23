//! 移除无操作投影的规则

use crate::query::planner::plan::PlanNodeEnum;
use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{RewriteRule, EliminationRule};
use crate::core::YieldColumn;
use crate::core::Expression;

/// 移除无操作投影的规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   Project(v1, v2, v3)
///       |
///   ScanVertices (输出 v1, v2, v3)
/// ```
///
/// After:
/// ```text
///   ScanVertices
/// ```
///
/// # 适用条件
///
/// - Project 节点的输出列与子节点的输出列完全相同
/// - Project 节点不包含别名或表达式
#[derive(Debug)]
pub struct RemoveNoopProjectRule;

impl RemoveNoopProjectRule {
    /// 创建规则实例
    pub fn new() -> Self {
        Self
    }

    /// 检查是否为无操作投影
    fn is_noop_projection(
        &self,
        columns: &[YieldColumn],
        child_col_names: &[String],
    ) -> bool {
        if columns.is_empty() {
            return false;
        }

        // 检查是否为通配符投影
        if columns.len() == 1 {
            if let Expression::Variable(var_name) = &columns[0].expression {
                if var_name == "*" {
                    return true;
                }
            }
        }

        // 如果子节点没有列名，认为是无操作
        if child_col_names.is_empty() {
            return true;
        }

        // 检查是否包含别名或表达式
        if self.has_aliases_or_expressions_in_columns(columns) {
            return false;
        }

        // 比较投影列和子节点列名
        let projected_columns: Vec<String> = columns.iter().map(|col| col.alias.clone()).collect();

        if projected_columns.len() == child_col_names.len() {
            for (i, col_name) in projected_columns.iter().enumerate() {
                if i < child_col_names.len() && col_name != &child_col_names[i] {
                    return false;
                }
            }
            return true;
        }

        false
    }

    /// 检查列中是否包含别名或表达式
    fn has_aliases_or_expressions_in_columns(
        &self,
        columns: &[YieldColumn],
    ) -> bool {
        for column in columns {
            match &column.expression {
                Expression::Variable(_) => {}
                _ => return true,
            }

            if let Expression::Variable(var_name) = &column.expression {
                if var_name != &column.alias {
                    return true;
                }
            }
        }

        false
    }
}

impl Default for RemoveNoopProjectRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RewriteRule for RemoveNoopProjectRule {
    fn name(&self) -> &'static str {
        "RemoveNoopProjectRule"
    }

    fn pattern(&self) -> Pattern {
        Pattern::new_with_name("Project")
    }

    fn apply(
        &self,
        _ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        // 检查是否为 Project 节点
        let project_node = match node {
            PlanNodeEnum::Project(n) => n,
            _ => return Ok(None),
        };

        // 获取输入节点
        let input = project_node.input();
        let columns = project_node.columns();
        let child_col_names = input.col_names();

        // 检查是否为无操作投影
        if !self.is_noop_projection(&columns, &child_col_names) {
            return Ok(None);
        }

        // 创建转换结果，用输入节点替换当前 Project 节点
        let mut result = TransformResult::new();
        result.erase_curr = true;
        result.add_new_node(input.clone());

        Ok(Some(result))
    }
}

impl EliminationRule for RemoveNoopProjectRule {
    fn can_eliminate(&self, node: &PlanNodeEnum) -> bool {
        match node {
            PlanNodeEnum::Project(n) => {
                let input = n.input();
                let columns = n.columns();
                let child_col_names = input.col_names();
                self.is_noop_projection(&columns, &child_col_names)
            }
            _ => false,
        }
    }

    fn eliminate(
        &self,
        _ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        self.apply(_ctx, node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_noop_project_rule_name() {
        let rule = RemoveNoopProjectRule::new();
        assert_eq!(rule.name(), "RemoveNoopProjectRule");
    }

    #[test]
    fn test_remove_noop_project_rule_pattern() {
        let rule = RemoveNoopProjectRule::new();
        let pattern = rule.pattern();
        assert!(pattern.node.is_some());
    }
}
