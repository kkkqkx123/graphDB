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
    ///   Limit(offset=10, count=100)
    ///       |
    ///   ScanVertices
    /// ```
    ///
    /// After:
    /// ```text
    ///   Limit(offset=10, count=100)
    ///       |
    ///   ScanVertices(limit=110)
    /// ```
    ///
    /// # 适用条件
    ///
    /// - 当前节点为Limit节点
    /// - 子节点为ScanVertices节点
    /// - Limit节点只有一个子节点
    /// - ScanVertices尚未设置limit，或新limit小于现有limit
    name: PushLimitDownScanVerticesRule,
    parent_node: Limit,
    child_node: ScanVertices,
    apply: |_ctx, limit_node: &LimitNode, scan_vertices_node: &ScanVerticesNode| {
        // 计算需要获取的总行数（offset + count）
        let limit_rows = limit_node.offset() + limit_node.count();

        // 检查ScanVertices是否已有更严格的limit
        if let Some(existing_limit) = scan_vertices_node.limit() {
            if limit_rows >= existing_limit {
                // 现有limit更严格，无需转换
                return Ok(None::<TransformResult>);
            }
        }

        // 创建新的ScanVertices节点，设置limit
        let mut new_scan_vertices = scan_vertices_node.clone();
        new_scan_vertices.set_limit(limit_rows);

        // 创建转换结果
        let mut result = TransformResult::new();
        result.erase_all = true;
        result.add_new_node(PlanNodeEnum::ScanVertices(new_scan_vertices));

        Ok(Some(result))
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
