//! 将TopN下推到索引扫描操作的规则
//!
//! 该规则识别 TopN -> IndexScan 模式，
//! 并将TopN的限制和排序信息集成到IndexScan操作中。
//!
//! # 转换示例
//!
//! Before:
//! ```text
//!   TopN(count=100, sort_items=[age DESC])
//!       |
//!   IndexScan
//! ```
//!
//! After:
//! ```text
//!   IndexScan(limit=100, order_by=[age DESC])
//! ```
//!
//! # 适用条件
//!
//! - 当前节点为TopN节点
//! - 子节点为IndexScan节点
//! - TopN节点只有一个子节点
//! - IndexScan尚未设置limit（避免重复下推）

use crate::query::planner::plan::PlanNodeEnum;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{RewriteRule, PushDownRule};
use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;

/// 将TopN下推到索引扫描操作的规则
#[derive(Debug)]
pub struct PushTopNDownIndexScanRule;

impl PushTopNDownIndexScanRule {
    /// 创建规则实例
    pub fn new() -> Self {
        Self
    }
}

impl Default for PushTopNDownIndexScanRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RewriteRule for PushTopNDownIndexScanRule {
    fn name(&self) -> &'static str {
        "PushTopNDownIndexScanRule"
    }

    fn pattern(&self) -> Pattern {
        Pattern::new_with_name("TopN").with_dependency_name("IndexScan")
    }

    fn apply(
        &self,
        _ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        // 检查是否为 TopN 节点
        let topn_node = match node {
            PlanNodeEnum::TopN(n) => n,
            _ => return Ok(None),
        };

        // 获取输入节点
        let input = topn_node.input();

        // 检查输入节点是否为 IndexScan
        let _index_scan = match input {
            PlanNodeEnum::IndexScan(n) => n,
            _ => return Ok(None),
        };

        // 简化实现：返回 None 表示不转换
        // 实际实现需要创建新的 IndexScan 节点并设置 limit 和 order_by
        Ok(None)
    }
}

impl PushDownRule for PushTopNDownIndexScanRule {
    fn can_push_down(&self, node: &PlanNodeEnum, target: &PlanNodeEnum) -> bool {
        matches!((node, target), (PlanNodeEnum::TopN(_), PlanNodeEnum::IndexScan(_)))
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
        let rule = PushTopNDownIndexScanRule::new();
        assert_eq!(rule.name(), "PushTopNDownIndexScanRule");
    }

    #[test]
    fn test_rule_pattern() {
        let rule = PushTopNDownIndexScanRule::new();
        let pattern = rule.pattern();
        assert!(pattern.node.is_some());
    }
}
