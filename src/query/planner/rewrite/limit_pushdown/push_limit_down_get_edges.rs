//! 将LIMIT下推到获取边操作的规则
//!
//! 该规则识别 Limit -> GetEdges 模式，
//! 并将LIMIT值集成到GetEdges操作中。

use crate::query::planner::plan::PlanNodeEnum;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{RewriteRule, PushDownRule};
use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;

/// 将LIMIT下推到获取边操作的规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   Limit(100)
///       |
///   GetEdges
/// ```
///
/// After:
/// ```text
///   GetEdges(limit=100)
/// ```
///
/// # 适用条件
///
/// - 当前节点为Limit节点
/// - 子节点为GetEdges节点
/// - Limit节点只有一个子节点
#[derive(Debug)]
pub struct PushLimitDownGetEdgesRule;

impl PushLimitDownGetEdgesRule {
    /// 创建规则实例
    pub fn new() -> Self {
        Self
    }
}

impl Default for PushLimitDownGetEdgesRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RewriteRule for PushLimitDownGetEdgesRule {
    fn name(&self) -> &'static str {
        "PushLimitDownGetEdgesRule"
    }

    fn pattern(&self) -> Pattern {
        Pattern::new_with_name("Limit").with_dependency_name("GetEdges")
    }

    fn apply(
        &self,
        _ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        // 检查是否为 Limit 节点
        let limit_node = match node {
            PlanNodeEnum::Limit(n) => n,
            _ => return Ok(None),
        };

        // 获取输入节点
        let input = limit_node.input();

        // 检查输入节点是否为 GetEdges
        let _get_edges = match input {
            PlanNodeEnum::GetEdges(n) => n,
            _ => return Ok(None),
        };

        // 简化实现：返回 None 表示不转换
        // 实际实现需要创建新的 GetEdges 节点并设置 limit
        Ok(None)
    }
}

impl PushDownRule for PushLimitDownGetEdgesRule {
    fn can_push_down(&self, node: &PlanNodeEnum, target: &PlanNodeEnum) -> bool {
        matches!((node, target), (PlanNodeEnum::Limit(_), PlanNodeEnum::GetEdges(_)))
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
        let rule = PushLimitDownGetEdgesRule::new();
        assert_eq!(rule.name(), "PushLimitDownGetEdgesRule");
    }

    #[test]
    fn test_rule_pattern() {
        let rule = PushLimitDownGetEdgesRule::new();
        let pattern = rule.pattern();
        assert!(pattern.node.is_some());
    }
}
