//! 将LIMIT下推到获取顶点操作的规则
//!
//! 该规则识别 Limit -> GetVertices 模式，
//! 并将LIMIT值集成到GetVertices操作中。

use crate::query::planner::plan::core::nodes::graph_scan_node::GetVerticesNode;
use crate::query::planner::plan::core::nodes::sort_node::LimitNode;
use crate::query::planner::rewrite::macros::define_rewrite_pushdown_rule;
use crate::query::planner::rewrite::result::TransformResult;

define_rewrite_pushdown_rule! {
    /// 将LIMIT下推到获取顶点操作的规则
    ///
    /// # 转换示例
    ///
    /// Before:
    /// ```text
    ///   Limit(100)
    ///       |
    ///   GetVertices
    /// ```
    ///
    /// After:
    /// ```text
    ///   GetVertices(limit=100)
    /// ```
    ///
    /// # 适用条件
    ///
    /// - 当前节点为Limit节点
    /// - 子节点为GetVertices节点
    /// - Limit节点只有一个子节点
    name: PushLimitDownGetVerticesRule,
    parent_node: Limit,
    child_node: GetVertices,
    apply: |_ctx, _limit_node: &LimitNode, _get_vertices_node: &GetVerticesNode| {
        // 简化实现：返回 None 表示不转换
        // 实际实现需要创建新的 GetVertices 节点并设置 limit
        Ok(None::<TransformResult>)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::planner::rewrite::rule::RewriteRule;

    #[test]
    fn test_rule_name() {
        let rule = PushLimitDownGetVerticesRule::new();
        assert_eq!(rule.name(), "PushLimitDownGetVerticesRule");
    }

    #[test]
    fn test_rule_pattern() {
        let rule = PushLimitDownGetVerticesRule::new();
        let pattern = rule.pattern();
        assert!(pattern.node.is_some());
    }
}
