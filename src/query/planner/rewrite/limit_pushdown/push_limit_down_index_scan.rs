//! 将LIMIT下推到索引扫描操作的规则
//!
//! 该规则识别 Limit -> IndexScan 模式，
//! 并将LIMIT值集成到IndexScan操作中。

use crate::query::planner::plan::algorithms::index_scan::IndexScan;
use crate::query::planner::plan::core::nodes::sort_node::LimitNode;
use crate::query::planner::rewrite::macros::define_rewrite_pushdown_rule;
use crate::query::planner::rewrite::result::TransformResult;

define_rewrite_pushdown_rule! {
    /// 将LIMIT下推到索引扫描操作的规则
    ///
    /// # 转换示例
    ///
    /// Before:
    /// ```text
    ///   Limit(100)
    ///       |
    ///   IndexScan
    /// ```
    ///
    /// After:
    /// ```text
    ///   IndexScan(limit=100)
    /// ```
    ///
    /// # 适用条件
    ///
    /// - 当前节点为Limit节点
    /// - 子节点为IndexScan节点
    /// - Limit节点只有一个子节点
    name: PushLimitDownIndexScanRule,
    parent_node: Limit,
    child_node: IndexScan,
    apply: |_ctx, _limit_node: &LimitNode, _index_scan_node: &IndexScan| {
        // 简化实现：返回 None 表示不转换
        // 实际实现需要创建新的 IndexScan 节点并设置 limit
        Ok(None::<TransformResult>)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::planner::rewrite::rule::RewriteRule;

    #[test]
    fn test_rule_name() {
        let rule = PushLimitDownIndexScanRule::new();
        assert_eq!(rule.name(), "PushLimitDownIndexScanRule");
    }

    #[test]
    fn test_rule_pattern() {
        let rule = PushLimitDownIndexScanRule::new();
        let pattern = rule.pattern();
        assert!(pattern.node.is_some());
    }
}
