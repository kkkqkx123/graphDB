//! 投影下推优化规则
//!
//! 这些规则负责将投影操作下推到计划树的底层，以减少数据传输量。
//!
//! # 转换示例
//!
//! Before:
//! ```text
//! Project(col1, col2)
//!         |
//!       ScanVertices
//! ```
//!
//! After:
//! ```text
//! ScanVertices(col1, col2)
//! ```
//!
//! # 适用条件
//!
//! - Project 节点的子节点是数据访问节点（ScanVertices、ScanEdges、GetVertices、GetEdges、GetNeighbors）
//! - Project 节点有列定义

use crate::query::planner::plan::PlanNodeEnum;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{RewriteRule, PushDownRule};
use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;

/// 投影下推规则
#[derive(Debug)]
pub struct ProjectionPushDownRule;

impl ProjectionPushDownRule {
    /// 创建规则实例
    pub fn new() -> Self {
        Self
    }

    /// 检查是否可以下推投影
    fn can_push_down_project(node: &crate::query::planner::plan::core::nodes::ProjectNode) -> bool {
        !node.columns().is_empty()
    }
}

impl Default for ProjectionPushDownRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RewriteRule for ProjectionPushDownRule {
    fn name(&self) -> &'static str {
        "ProjectionPushDownRule"
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

        // 检查是否可以下推
        if !Self::can_push_down_project(project_node) {
            return Ok(None);
        }

        // 获取输入节点
        let input = project_node.input();

        // 检查输入节点类型
        let can_push = matches!(
            input,
            PlanNodeEnum::ScanVertices(_)
                | PlanNodeEnum::ScanEdges(_)
                | PlanNodeEnum::GetVertices(_)
                | PlanNodeEnum::GetEdges(_)
                | PlanNodeEnum::GetNeighbors(_)
                | PlanNodeEnum::Start(_)
        );

        if !can_push {
            return Ok(None);
        }

        // 简化实现：返回 None 表示不转换
        // 实际实现需要将投影信息下推到数据访问节点
        Ok(None)
    }
}

impl PushDownRule for ProjectionPushDownRule {
    fn can_push_down(&self, node: &PlanNodeEnum, target: &PlanNodeEnum) -> bool {
        match (node, target) {
            (PlanNodeEnum::Project(project), target) => {
                if project.columns().is_empty() {
                    return false;
                }
                matches!(
                    target,
                    PlanNodeEnum::ScanVertices(_)
                        | PlanNodeEnum::ScanEdges(_)
                        | PlanNodeEnum::GetVertices(_)
                        | PlanNodeEnum::GetEdges(_)
                        | PlanNodeEnum::GetNeighbors(_)
                )
            }
            _ => false,
        }
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
        let rule = ProjectionPushDownRule::new();
        assert_eq!(rule.name(), "ProjectionPushDownRule");
    }

    #[test]
    fn test_rule_pattern() {
        let rule = ProjectionPushDownRule::new();
        let pattern = rule.pattern();
        assert!(pattern.node.is_some());
    }
}
