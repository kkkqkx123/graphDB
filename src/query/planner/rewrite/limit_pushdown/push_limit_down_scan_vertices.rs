//! 将LIMIT下推到扫描顶点操作的规则
//!
//! 该规则识别 Limit -> ScanVertices 模式，
//! 并将LIMIT值集成到ScanVertices操作中。

use crate::query::planner::plan::PlanNodeEnum;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{RewriteRule, PushDownRule};
use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;

/// 将LIMIT下推到扫描顶点操作的规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   Limit(100)
///       |
///   ScanVertices
/// ```
///
/// After:
/// ```text
///   ScanVertices(limit=100)
/// ```
///
/// # 适用条件
///
/// - 当前节点为Limit节点
/// - 子节点为ScanVertices节点
/// - Limit节点只有一个子节点
#[derive(Debug)]
pub struct PushLimitDownScanVerticesRule;

impl PushLimitDownScanVerticesRule {
    /// 创建规则实例
    pub fn new() -> Self {
        Self
    }
}

impl Default for PushLimitDownScanVerticesRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RewriteRule for PushLimitDownScanVerticesRule {
    fn name(&self) -> &'static str {
        "PushLimitDownScanVerticesRule"
    }

    fn pattern(&self) -> Pattern {
        Pattern::new_with_name("Limit").with_dependency_name("ScanVertices")
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

        // 检查输入节点是否为 ScanVertices
        let _scan_vertices = match input {
            PlanNodeEnum::ScanVertices(n) => n,
            _ => return Ok(None),
        };

        // 简化实现：返回 None 表示不转换
        // 实际实现需要创建新的 ScanVertices 节点并设置 limit
        Ok(None)
    }
}

impl PushDownRule for PushLimitDownScanVerticesRule {
    fn can_push_down(&self, node: &PlanNodeEnum, target: &PlanNodeEnum) -> bool {
        matches!((node, target), (PlanNodeEnum::Limit(_), PlanNodeEnum::ScanVertices(_)))
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
        let rule = PushLimitDownScanVerticesRule::new();
        assert_eq!(rule.name(), "PushLimitDownScanVerticesRule");
    }

    #[test]
    fn test_rule_pattern() {
        let rule = PushLimitDownScanVerticesRule::new();
        let pattern = rule.pattern();
        assert!(pattern.node.is_some());
    }
}
