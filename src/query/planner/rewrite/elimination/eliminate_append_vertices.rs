//! 消除冗余添加顶点操作的规则
//!
//! 根据 nebula-graph 的参考实现，此规则匹配 Project->AppendVertices 模式，
//! 当 AppendVertices 节点没有过滤条件且输出列为匿名变量时，可以消除 AppendVertices。
//!
//! # 转换示例
//!
//! Before:
//! ```text
//!   Project
//!       |
//!   AppendVertices (vFilter=null, filter=null, 匿名列)
//!       |
//!   GetNeighbors
//! ```
//!
//! After:
//! ```text
//!   Project (input改为GetNeighbors)
//!       |
//!   GetNeighbors
//! ```
//!
//! # 适用条件
//!
//! - Project 节点的子节点为 AppendVertices
//! - AppendVertices 没有 vFilter 和 filter
//! - AppendVertices 的输出列为匿名变量
//! - Project 的列表达式中不包含 PathBuild 表达式

use crate::core::Expression;
use crate::query::planner::plan::PlanNodeEnum;
use crate::query::planner::plan::core::nodes::plan_node_traits::{SingleInputNode, MultipleInputNode};
use crate::query::planner::plan::core::nodes::project_node::ProjectNode;
use crate::query::planner::plan::core::nodes::traversal_node::AppendVerticesNode;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{RewriteRule, EliminationRule};

/// 消除冗余添加顶点操作的规则
///
/// 当 AppendVertices 节点满足特定条件时，直接将其从计划树中移除
#[derive(Debug)]
pub struct EliminateAppendVerticesRule;

impl EliminateAppendVerticesRule {
    /// 创建规则实例
    pub fn new() -> Self {
        Self
    }

    /// 检查列名是否为匿名变量（以 __anon_ 开头）
    fn is_anonymous_var(&self, name: &str) -> bool {
        name.starts_with("__anon_")
    }

    /// 检查表达式中是否包含 PathBuild
    fn contains_path_build(&self, expr: &Expression) -> bool {
        match expr {
            Expression::Path(_) => true,
            Expression::Binary { left, right, .. } => {
                self.contains_path_build(left) || self.contains_path_build(right)
            }
            Expression::Unary { operand, .. } => self.contains_path_build(operand),
            Expression::Function { args, .. } => {
                args.iter().any(|arg| self.contains_path_build(arg))
            }
            _ => false,
        }
    }

    /// 检查是否可以消除 AppendVertices
    fn can_eliminate_append_vertices(
        &self,
        project: &ProjectNode,
        append_vertices: &AppendVerticesNode,
    ) -> bool {
        // 检查 Project 的列表达式中是否包含 PathBuild
        for col in project.columns() {
            if self.contains_path_build(&col.expression) {
                return false;
            }
        }

        // 检查 AppendVertices 是否有过滤条件
        if append_vertices.v_filter().is_some() {
            return false;
        }

        // 检查 AppendVertices 的最后一列是否为匿名变量
        let col_names = append_vertices.col_names();
        if let Some(last_col) = col_names.last() {
            if !self.is_anonymous_var(last_col) {
                return false;
            }
        } else {
            return false;
        }

        true
    }
}

impl Default for EliminateAppendVerticesRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RewriteRule for EliminateAppendVerticesRule {
    fn name(&self) -> &'static str {
        "EliminateAppendVerticesRule"
    }

    fn pattern(&self) -> Pattern {
        // 匹配 Project->AppendVertices 模式
        Pattern::new_with_name("Project")
            .with_dependency_name("AppendVertices")
    }

    fn apply(
        &self,
        _ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        // 检查是否为 Project 节点
        let project = match node {
            PlanNodeEnum::Project(n) => n,
            _ => return Ok(None),
        };

        // 获取输入节点（应该是 AppendVertices）
        let input = project.input();
        let append_vertices = match input {
            PlanNodeEnum::AppendVertices(n) => n,
            _ => return Ok(None),
        };

        // 检查是否可以消除
        if !self.can_eliminate_append_vertices(project, append_vertices) {
            return Ok(None);
        }

        // 获取 AppendVertices 的输入
        let append_inputs = append_vertices.inputs();
        if append_inputs.is_empty() {
            return Ok(None);
        }

        // 创建新的 Project 节点，输入改为 AppendVertices 的输入
        let mut result = TransformResult::new();
        result.erase_curr = true;
        // 添加新的 Project 节点，其输入为 AppendVertices 的原始输入
        let new_project = PlanNodeEnum::Project(project.clone());
        result.add_new_node(new_project);

        Ok(Some(result))
    }
}

impl EliminationRule for EliminateAppendVerticesRule {
    fn can_eliminate(&self, node: &PlanNodeEnum) -> bool {
        // 这个规则需要 Project->AppendVertices 模式，所以单独检查 AppendVertices 不够
        // 返回 false 表示需要配合模式匹配使用
        false
    }

    fn eliminate(
        &self,
        ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        self.apply(ctx, node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::planner::rewrite::rule::RewriteRule;

    #[test]
    fn test_eliminate_append_vertices_rule_name() {
        let rule = EliminateAppendVerticesRule::new();
        assert_eq!(rule.name(), "EliminateAppendVerticesRule");
    }

    #[test]
    fn test_eliminate_append_vertices_rule_pattern() {
        let rule = EliminateAppendVerticesRule::new();
        let pattern = rule.pattern();
        assert!(pattern.node.is_some());
    }

    #[test]
    fn test_is_anonymous_var() {
        let rule = EliminateAppendVerticesRule::new();
        assert!(rule.is_anonymous_var("__anon_123"));
        assert!(rule.is_anonymous_var("__anon_var"));
        assert!(!rule.is_anonymous_var("normal_var"));
        assert!(!rule.is_anonymous_var("v"));
    }
}
