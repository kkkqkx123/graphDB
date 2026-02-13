//! 移除无操作投影的规则

use crate::query::optimizer::plan::{OptContext, OptGroupNode};
use crate::query::optimizer::rule_patterns::PatternBuilder;
use crate::query::validator::YieldColumn;
use crate::query::visitor::PlanNodeVisitor;

crate::define_elimination_rule! {
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
    pub struct RemoveNoopProjectRule {
        target: Project,
        target_check: is_project,
        pattern: PatternBuilder::project()
    }
    visitor: RemoveNoopProjectVisitor
}

/// 移除无操作投影访问者
///
/// 状态不变量：
/// - `is_eliminated` 为 true 时，`eliminated_node` 必须为 Some
/// - `is_eliminated` 为 false 时，`eliminated_node` 必须为 None
#[derive(Clone)]
struct RemoveNoopProjectVisitor<'a> {
    is_eliminated: bool,
    eliminated_node: Option<OptGroupNode>,
    ctx: &'a OptContext,
}

impl<'a> PlanNodeVisitor for RemoveNoopProjectVisitor<'a> {
    type Result = Self;

    fn visit_default(&mut self) -> Self::Result {
        self.clone()
    }

    fn visit_project(&mut self, node: &crate::query::planner::plan::core::nodes::ProjectNode) -> Self::Result {
        if self.is_eliminated {
            return self.clone();
        }

        let deps = node.dependencies();
        if deps.is_empty() {
            return self.clone();
        }

        let input = if let Some(input) = deps.first() {
            input
        } else {
            return self.clone();
        };
        let input_id = input.id() as usize;

        if let Some(child_node) = self.ctx.find_group_node_by_plan_node_id(input_id) {
            let child_node_ref = child_node.borrow();
            let columns = node.columns();
            let child_col_names = child_node_ref.plan_node.col_names();

            if self.is_noop_projection(&columns, &child_col_names) {
                let mut new_node = child_node_ref.clone();

                if let Some(_output_var) = node.output_var() {
                    new_node.plan_node = (**input).clone();
                }

                drop(child_node_ref);

                self.is_eliminated = true;
                self.eliminated_node = Some(new_node);
            } else {
                drop(child_node_ref);
            }
        }

        self.clone()
    }
}

impl<'a> RemoveNoopProjectVisitor<'a> {
    fn is_noop_projection(
        &self,
        columns: &[YieldColumn],
        child_col_names: &[String],
    ) -> bool {
        if columns.is_empty() {
            return false;
        }

        if columns.len() == 1 {
            if let crate::core::Expression::Variable(var_name) = &columns[0].expression {
                if var_name == "*" {
                    return true;
                }
            }
        }

        if child_col_names.is_empty() {
            return true;
        }

        if self.has_aliases_or_expressions_in_columns(columns) {
            return false;
        }

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

    fn has_aliases_or_expressions_in_columns(
        &self,
        columns: &[YieldColumn],
    ) -> bool {
        for column in columns {
            match &column.expression {
                crate::core::Expression::Variable(_) => {}
                _ => return true,
            }

            if let crate::core::Expression::Variable(var_name) = &column.expression {
                if var_name != &column.alias {
                    return true;
                }
            }
        }

        false
    }
}
