//! 将LIMIT下推到获取边操作的规则
//!
//! 该规则识别 Limit -> GetEdges 模式，
//! 并将LIMIT值集成到GetEdges操作中。

use crate::query::planner::plan::core::nodes::graph_scan_node::GetEdgesNode;
use crate::query::planner::plan::core::nodes::sort_node::LimitNode;
use crate::query::planner::rewrite::macros::define_rewrite_pushdown_rule;
use crate::query::planner::rewrite::result::TransformResult;

define_rewrite_pushdown_rule! {
    /// 将LIMIT下推到获取边操作的规则
    ///
    /// # 转换示例
    ///
    /// Before:
    /// ```text
    ///   Limit(offset=10, count=100)
    ///       |
    ///   GetEdges
    /// ```
    ///
    /// After:
    /// ```text
    ///   Limit(offset=10, count=100)
    ///       |
    ///   GetEdges(limit=110)
    /// ```
    ///
    /// # 适用条件
    ///
    /// - 当前节点为Limit节点
    /// - 子节点为GetEdges节点
    /// - Limit节点只有一个子节点
    /// - GetEdges尚未设置limit，或新limit小于现有limit
    name: PushLimitDownGetEdgesRule,
    parent_node: Limit,
    child_node: GetEdges,
    apply: |_ctx, limit_node: &LimitNode, get_edges_node: &GetEdgesNode| {
        // 计算需要获取的总行数（offset + count）
        let limit_rows = limit_node.offset() + limit_node.count();

        // 检查GetEdges是否已有更严格的limit
        if let Some(existing_limit) = get_edges_node.limit() {
            if limit_rows >= existing_limit {
                // 现有limit更严格，无需转换
                return Ok(None::<TransformResult>);
            }
        }

        // 创建新的GetEdges节点，设置limit
        let mut new_get_edges = get_edges_node.clone();
        new_get_edges.set_limit(limit_rows);

        // 创建转换结果
        let mut result = TransformResult::new();
        result.erase_all = true;
        result.add_new_node(PlanNodeEnum::GetEdges(new_get_edges));

        Ok(Some(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::planner::rewrite::rule::RewriteRule;

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
