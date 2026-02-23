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
    ///   Limit(offset=10, count=100)
    ///       |
    ///   GetVertices
    /// ```
    ///
    /// After:
    /// ```text
    ///   Limit(offset=10, count=100)
    ///       |
    ///   GetVertices(limit=110)
    /// ```
    ///
    /// # 适用条件
    ///
    /// - 当前节点为Limit节点
    /// - 子节点为GetVertices节点
    /// - Limit节点只有一个子节点
    /// - GetVertices尚未设置limit，或新limit小于现有limit
    name: PushLimitDownGetVerticesRule,
    parent_node: Limit,
    child_node: GetVertices,
    apply: |_ctx, limit_node: &LimitNode, get_vertices_node: &GetVerticesNode| {
        // 计算需要获取的总行数（offset + count）
        let limit_rows = limit_node.offset() + limit_node.count();

        // 检查GetVertices是否已有更严格的limit
        if let Some(existing_limit) = get_vertices_node.limit() {
            if limit_rows >= existing_limit {
                // 现有limit更严格，无需转换
                return Ok(None::<TransformResult>);
            }
        }

        // 创建新的GetVertices节点，设置limit
        let mut new_get_vertices = get_vertices_node.clone();
        new_get_vertices.set_limit(limit_rows);

        // 创建转换结果
        let mut result = TransformResult::new();
        result.erase_all = true;
        result.add_new_node(PlanNodeEnum::GetVertices(new_get_vertices));

        Ok(Some(result))
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
