//! 将TopN下推到索引扫描操作的规则
//!
//! 该规则识别 TopN -> IndexScan 模式，
//! 并将TopN的限制和排序信息集成到IndexScan操作中。

use crate::query::planner::plan::algorithms::index_scan::IndexScan;
use crate::query::planner::plan::core::nodes::sort_node::TopNNode;
use crate::query::planner::rewrite::macros::define_rewrite_pushdown_rule;
use crate::query::planner::rewrite::result::TransformResult;

define_rewrite_pushdown_rule! {
    /// 将TopN下推到索引扫描操作的规则
    ///
    /// # 转换示例
    ///
    /// Before:
    /// ```text
    ///   TopN(count=100, sort_items=[age DESC])
    ///       |
    ///   IndexScan
    /// ```
    ///
    /// After:
    /// ```text
    ///   IndexScan(limit=100, order_by=[age DESC])
    /// ```
    ///
    /// # 适用条件
    ///
    /// - 当前节点为TopN节点
    /// - 子节点为IndexScan节点
    /// - TopN节点只有一个子节点
    /// - IndexScan尚未设置limit（避免重复下推）
    name: PushTopNDownIndexScanRule,
    parent_node: TopN,
    child_node: IndexScan,
    apply: |_ctx, _topn_node: &TopNNode, _index_scan_node: &IndexScan| {
        // 简化实现：返回 None 表示不转换
        // 实际实现需要创建新的 IndexScan 节点并设置 limit 和 order_by
        Ok(None::<TransformResult>)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::planner::rewrite::rule::RewriteRule;

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
