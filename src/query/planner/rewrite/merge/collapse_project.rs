//! 折叠多个投影操作的规则

use crate::query::planner::plan::PlanNodeEnum;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{RewriteRule, MergeRule};

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
/// - 可以折叠两个投影操作
#[derive(Debug)]
pub struct CollapseProjectRule;

impl CollapseProjectRule {
    /// 创建规则实例
    pub fn new() -> Self {
        Self
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
        // 检查是否为 Project 节点
        let _project_node = match node {
            PlanNodeEnum::Project(n) => n,
            _ => return Ok(None),
        };

        // 简化实现：返回 None 表示不转换
        // 实际实现需要检查下层节点并执行折叠
        Ok(None)
    }
}

impl MergeRule for CollapseProjectRule {
    fn can_merge(&self, parent: &PlanNodeEnum, child: &PlanNodeEnum) -> bool {
        parent.is_project() && child.is_project()
    }

    fn create_merged_node(
        &self,
        _ctx: &mut RewriteContext,
        parent: &PlanNodeEnum,
        _child: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        // 简化实现：直接返回父节点
        let mut result = TransformResult::new();
        result.add_new_node(parent.clone());
        Ok(Some(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
