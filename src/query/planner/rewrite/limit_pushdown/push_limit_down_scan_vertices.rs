//! 将LIMIT下推到扫描顶点操作的规则
//!
//! 该规则识别 Limit -> ScanVertices 模式，
//! 并将LIMIT值集成到ScanVertices操作中。

use crate::query::planner::plan::core::nodes::graph_scan_node::ScanVerticesNode;
use crate::query::planner::plan::core::nodes::sort_node::LimitNode;
use crate::query::planner::rewrite::macros::define_rewrite_pushdown_rule;
use crate::query::planner::rewrite::result::TransformResult;

define_rewrite_pushdown_rule! {
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
    name: PushLimitDownScanVerticesRule,
    parent_node: Limit,
    child_node: ScanVertices,
    apply: |_ctx, _limit_node: &LimitNode, _scan_vertices_node: &ScanVerticesNode| {
        // 简化实现：返回 None 表示不转换
        // 实际实现需要创建新的 ScanVertices 节点并设置 limit
        Ok(None::<TransformResult>)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::planner::rewrite::rule::RewriteRule;

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
