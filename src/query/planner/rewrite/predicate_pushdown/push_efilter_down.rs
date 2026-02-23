//! 将边过滤条件下推到Traverse节点的规则
//!
//! 该规则识别 Traverse 节点中的 eFilter，
//! 并将其重写为具体的边属性表达式。

use crate::query::planner::plan::PlanNodeEnum;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{RewriteRule, PushDownRule};

/// 将边过滤条件下推到Traverse节点的规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   Traverse(eFilter: *.likeness > 78)
/// ```
///
/// After:
/// ```text
///   Traverse(filter: e.likeness > 78)
/// ```
///
/// # 适用条件
///
/// - Traverse 节点存在 eFilter
/// - eFilter 包含通配符边属性表达式
/// - Traverse 不为零步遍历
#[derive(Debug)]
pub struct PushEFilterDownRule;

impl PushEFilterDownRule {
    /// 创建规则实例
    pub fn new() -> Self {
        Self
    }
}

impl Default for PushEFilterDownRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RewriteRule for PushEFilterDownRule {
    fn name(&self) -> &'static str {
        "PushEFilterDownRule"
    }

    fn pattern(&self) -> Pattern {
        Pattern::new_with_name("Traverse")
    }

    fn apply(
        &self,
        _ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        // 检查是否为 Traverse 节点
        let traverse = match node {
            PlanNodeEnum::Traverse(t) => t,
            _ => return Ok(None),
        };

        // 获取 eFilter
        let _e_filter = match traverse.e_filter() {
            Some(filter) => filter,
            None => return Ok(None),
        };

        // 简化实现：返回 None 表示不转换
        // 实际实现需要将通配符边属性表达式重写为具体的边属性表达式
        Ok(None)
    }
}

impl PushDownRule for PushEFilterDownRule {
    fn can_push_down(&self, node: &PlanNodeEnum, _target: &PlanNodeEnum) -> bool {
        match node {
            PlanNodeEnum::Traverse(traverse) => {
                traverse.e_filter().is_some() && traverse.min_steps() > 0
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
        let rule = PushEFilterDownRule::new();
        assert_eq!(rule.name(), "PushEFilterDownRule");
    }

    #[test]
    fn test_rule_pattern() {
        let rule = PushEFilterDownRule::new();
        let pattern = rule.pattern();
        assert!(pattern.node.is_some());
    }
}
