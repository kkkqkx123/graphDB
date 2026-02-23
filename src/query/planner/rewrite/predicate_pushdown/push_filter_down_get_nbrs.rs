//! 将过滤条件下推到GetNeighbors操作的规则
//!
//! 该规则识别 Filter -> GetNeighbors 模式，
//! 并将过滤条件下推到 GetNeighbors 节点中。

use crate::query::planner::plan::core::nodes::filter_node::FilterNode;
use crate::query::planner::plan::core::nodes::graph_scan_node::GetNeighborsNode;
use crate::query::planner::rewrite::macros::define_rewrite_pushdown_rule;
use crate::query::planner::rewrite::result::TransformResult;

define_rewrite_pushdown_rule! {
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
    name: PushFilterDownGetNbrsRule,
    parent_node: Filter,
    child_node: GetNeighbors,
    apply: |_ctx, _filter_node: &FilterNode, _get_neighbors_node: &GetNeighborsNode| {
        // 简化实现：返回 None 表示不转换
        // 实际实现需要创建新的 GetNeighbors 节点并设置 filter
        Ok(None::<TransformResult>)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::planner::rewrite::rule::RewriteRule;

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
