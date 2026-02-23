//! 将过滤条件下推到Traverse/AppendVertices节点的规则
//!
//! 该规则识别 Traverse/AppendVertices 节点中的 vFilter，
//! 并将可下推的过滤条件下推到数据源。

use crate::query::planner::plan::PlanNodeEnum;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{RewriteRule, PushDownRule};

/// 将过滤条件下推到Traverse/AppendVertices节点的规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   Traverse(vFilter: v.age > 18)
/// ```
///
/// After:
/// ```text
///   Traverse(vFilter: <remained>, firstStepFilter: v.age > 18)
/// ```
///
/// # 适用条件
///
/// - Traverse 或 AppendVertices 节点存在 vFilter
/// - vFilter 可以部分下推到 firstStepFilter
#[derive(Debug)]
pub struct PushFilterDownNodeRule;

impl PushFilterDownNodeRule {
    /// 创建规则实例
    pub fn new() -> Self {
        Self
    }
}

impl Default for PushFilterDownNodeRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RewriteRule for PushFilterDownNodeRule {
    fn name(&self) -> &'static str {
        "PushFilterDownNodeRule"
    }

    fn pattern(&self) -> Pattern {
        Pattern::new_with_name("Traverse")
    }

    fn apply(
        &self,
        _ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        // 检查是否为 Traverse 或 AppendVertices 节点
        let _v_filter = match node {
            PlanNodeEnum::Traverse(traverse) => traverse.v_filter().cloned(),
            PlanNodeEnum::AppendVertices(append) => append.v_filter().cloned(),
            _ => return Ok(None),
        };

        // 检查是否存在 vFilter
        let _v_filter = match _v_filter {
            Some(filter) => filter,
            None => return Ok(None),
        };

        // 简化实现：返回 None 表示不转换
        // 实际实现需要将 vFilter 部分下推到 firstStepFilter
        Ok(None)
    }
}

impl PushDownRule for PushFilterDownNodeRule {
    fn can_push_down(&self, node: &PlanNodeEnum, _target: &PlanNodeEnum) -> bool {
        matches!(node, PlanNodeEnum::Traverse(_) | PlanNodeEnum::AppendVertices(_))
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
        let rule = PushFilterDownNodeRule::new();
        assert_eq!(rule.name(), "PushFilterDownNodeRule");
    }

    #[test]
    fn test_rule_pattern() {
        let rule = PushFilterDownNodeRule::new();
        let pattern = rule.pattern();
        assert!(pattern.node.is_some());
    }
}
