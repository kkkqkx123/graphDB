//! 将过滤条件下推到GetNeighbors操作的规则
//!
//! 该规则识别 Filter -> GetNeighbors 模式，
//! 并将过滤条件下推到 GetNeighbors 节点中。

use crate::query::planner::plan::PlanNodeEnum;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{RewriteRule, PushDownRule};
use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;

/// 将过滤条件下推到GetNeighbors操作的规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   Filter(e.likeness > 78)
///           |
///   GetNeighbors
/// ```
///
/// After:
/// ```text
///   GetNeighbors(filter: e.likeness > 78)
/// ```
///
/// # 适用条件
///
/// - GetNeighbors 节点获取边属性
/// - 过滤条件可以下推到存储层
#[derive(Debug)]
pub struct PushFilterDownGetNbrsRule;

impl PushFilterDownGetNbrsRule {
    /// 创建规则实例
    pub fn new() -> Self {
        Self
    }
}

impl Default for PushFilterDownGetNbrsRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RewriteRule for PushFilterDownGetNbrsRule {
    fn name(&self) -> &'static str {
        "PushFilterDownGetNbrsRule"
    }

    fn pattern(&self) -> Pattern {
        Pattern::new_with_name("Filter").with_dependency_name("GetNeighbors")
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

        // 检查输入节点是否为 GetNeighbors
        let _get_neighbors = match input {
            PlanNodeEnum::GetNeighbors(n) => n,
            _ => return Ok(None),
        };

        // 简化实现：返回 None 表示不转换
        // 实际实现需要创建新的 GetNeighbors 节点并设置 filter
        Ok(None)
    }
}

impl PushDownRule for PushFilterDownGetNbrsRule {
    fn can_push_down(&self, node: &PlanNodeEnum, target: &PlanNodeEnum) -> bool {
        matches!((node, target), (PlanNodeEnum::Filter(_), PlanNodeEnum::GetNeighbors(_)))
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
        let rule = PushFilterDownGetNbrsRule::new();
        assert_eq!(rule.name(), "PushFilterDownGetNbrsRule");
    }

    #[test]
    fn test_rule_pattern() {
        let rule = PushFilterDownGetNbrsRule::new();
        let pattern = rule.pattern();
        assert!(pattern.node.is_some());
    }
}
