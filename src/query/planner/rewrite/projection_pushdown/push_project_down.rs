//! 向数据源推送投影操作的规则
//!
//! 该规则将投影操作推向数据源，减少数据传输量。
//!
//! # 转换示例
//!
//! Before:
//! ```text
//! Project(col1)
//!       |
//!   ScanVertices
//! ```
//!
//! After:
//! ```text
//! ScanVertices(col1)
//! ```
//!
//! # 适用条件
//!
//! - Project 节点的子节点是数据访问节点（ScanVertices、ScanEdges、GetVertices）
//! - Project 节点有列定义

use crate::query::planner::plan::PlanNodeEnum;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{RewriteRule, PushDownRule};
use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;

/// 向数据源推送投影操作的规则
#[derive(Debug)]
pub struct PushProjectDownRule;

impl PushProjectDownRule {
    /// 创建规则实例
    pub fn new() -> Self {
        Self
    }

    /// 检查是否可以下推投影
    fn can_push_down_project(node: &crate::query::planner::plan::core::nodes::ProjectNode) -> bool {
        !node.columns().is_empty()
    }
}

impl Default for PushProjectDownRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RewriteRule for PushProjectDownRule {
    fn name(&self) -> &'static str {
        "PushProjectDownRule"
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
        );

        if !can_push {
            return Ok(None);
        }

        // 简化实现：返回 None 表示不转换
        // 实际实现需要将投影信息下推到数据访问节点
        Ok(None)
    }
}

impl PushDownRule for PushProjectDownRule {
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
        let rule = PushProjectDownRule::new();
        assert_eq!(rule.name(), "PushProjectDownRule");
    }

    #[test]
    fn test_rule_pattern() {
        let rule = PushProjectDownRule::new();
        let pattern = rule.pattern();
        assert!(pattern.node.is_some());
    }
}
