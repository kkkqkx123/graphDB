//! 将过滤条件下推到连接操作的规则
//!
//! 该规则识别 Filter -> Join 模式，
//! 并将过滤条件下推到连接的一侧或两侧。

use crate::query::planner::plan::PlanNodeEnum;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{RewriteRule, PushDownRule};
use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;

/// 将过滤条件下推到连接操作的规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   Filter(a.col1 > 10)
///           |
///   HashInnerJoin
///   /          \
/// Left      Right
/// ```
///
/// After:
/// ```text
///   HashInnerJoin
///   /          \
/// Filter      Right
/// (a.col1>10)
///   |
/// Left
/// ```
///
/// # 适用条件
///
/// - 过滤条件仅涉及连接的一侧
/// - 可以安全地将条件下推
#[derive(Debug)]
pub struct PushFilterDownJoinRule;

impl PushFilterDownJoinRule {
    /// 创建规则实例
    pub fn new() -> Self {
        Self
    }
}

impl Default for PushFilterDownJoinRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RewriteRule for PushFilterDownJoinRule {
    fn name(&self) -> &'static str {
        "PushFilterDownJoinRule"
    }

    fn pattern(&self) -> Pattern {
        Pattern::new_with_name("Filter").with_dependency_name("Join")
    }

    fn apply(
        &self,
        _ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        // 检查是否为 Filter 节点
        let filter_node = match node {
            PlanNodeEnum::Filter(n) => n,
            _ => return Ok(None),
        };

        // 获取输入节点
        let input = filter_node.input();

        // 检查输入节点是否为 Join
        let is_join = input.is_hash_inner_join()
            || input.is_hash_left_join()
            || input.is_inner_join()
            || input.is_left_join();

        if !is_join {
            return Ok(None);
        }

        // 获取过滤条件
        let _filter_condition = filter_node.condition();

        // 简化实现：返回 None 表示不转换
        // 实际实现需要创建新的 Join 节点并在左侧添加 Filter
        Ok(None)
    }
}

impl PushDownRule for PushFilterDownJoinRule {
    fn can_push_down(&self, node: &PlanNodeEnum, target: &PlanNodeEnum) -> bool {
        match (node, target) {
            (PlanNodeEnum::Filter(_), target) => {
                target.is_hash_inner_join()
                    || target.is_hash_left_join()
                    || target.is_inner_join()
                    || target.is_left_join()
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
        let rule = PushFilterDownJoinRule::new();
        assert_eq!(rule.name(), "PushFilterDownJoinRule");
    }

    #[test]
    fn test_rule_pattern() {
        let rule = PushFilterDownJoinRule::new();
        let pattern = rule.pattern();
        assert!(pattern.node.is_some());
    }
}
