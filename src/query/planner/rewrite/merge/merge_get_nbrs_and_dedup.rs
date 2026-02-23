//! 合并获取邻居和去重操作的规则

use crate::query::planner::plan::PlanNodeEnum;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{RewriteRule, MergeRule};

/// 合并获取邻居和去重操作的规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   GetNeighbors
///       |
///   Dedup
///       |
///   ScanVertices
/// ```
///
/// After:
/// ```text
///   GetNeighbors
///       |
///   ScanVertices
/// ```
///
/// # 适用条件
///
/// - 当前节点为GetNeighbors节点
/// - 子节点为Dedup节点
/// - 可以将去重操作合并到GetNeighbors中
#[derive(Debug)]
pub struct MergeGetNbrsAndDedupRule;

impl MergeGetNbrsAndDedupRule {
    /// 创建规则实例
    pub fn new() -> Self {
        Self
    }
}

impl Default for MergeGetNbrsAndDedupRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RewriteRule for MergeGetNbrsAndDedupRule {
    fn name(&self) -> &'static str {
        "MergeGetNbrsAndDedupRule"
    }

    fn pattern(&self) -> Pattern {
        Pattern::new_with_name("GetNeighbors").with_dependency_name("Dedup")
    }

    fn apply(
        &self,
        _ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        // 检查是否为 GetNeighbors 节点
        let _get_neighbors_node = match node {
            PlanNodeEnum::GetNeighbors(n) => n,
            _ => return Ok(None),
        };

        // 简化实现：返回 None 表示不转换
        // 实际实现需要检查下层节点并执行合并
        Ok(None)
    }
}

impl MergeRule for MergeGetNbrsAndDedupRule {
    fn can_merge(&self, parent: &PlanNodeEnum, child: &PlanNodeEnum) -> bool {
        parent.is_get_neighbors() && child.is_dedup()
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
        let rule = MergeGetNbrsAndDedupRule::new();
        assert_eq!(rule.name(), "MergeGetNbrsAndDedupRule");
    }

    #[test]
    fn test_rule_pattern() {
        let rule = MergeGetNbrsAndDedupRule::new();
        let pattern = rule.pattern();
        assert!(pattern.node.is_some());
    }
}
