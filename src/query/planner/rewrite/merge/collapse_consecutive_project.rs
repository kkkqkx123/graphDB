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

use crate::query::planner::plan::PlanNodeEnum;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{RewriteRule, MergeRule};

/// 合并连续投影规则
#[derive(Debug)]
pub struct CollapseConsecutiveProjectRule;

impl CollapseConsecutiveProjectRule {
    /// 创建规则实例
    pub fn new() -> Self {
        Self
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
        // 检查是否为 Project 节点
        let _project_node = match node {
            PlanNodeEnum::Project(n) => n,
            _ => return Ok(None),
        };

        // 简化实现：返回 None 表示不转换
        // 实际实现需要检查下层节点并执行合并
        Ok(None)
    }
}

impl MergeRule for CollapseConsecutiveProjectRule {
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
        let rule = CollapseConsecutiveProjectRule::new();
        assert_eq!(rule.name(), "CollapseConsecutiveProjectRule");
    }

    #[test]
    fn test_rule_pattern() {
        let rule = CollapseConsecutiveProjectRule::new();
        let pattern = rule.pattern();
        assert!(pattern.node.is_some());
    }
}
