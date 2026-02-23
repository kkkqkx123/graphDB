//! 将LIMIT下推到扫描边操作的规则
//!
//! 该规则识别 Limit -> ScanEdges 模式，
//! 并将LIMIT值集成到ScanEdges操作中。

use crate::query::planner::plan::core::nodes::graph_scan_node::ScanEdgesNode;
use crate::query::planner::plan::core::nodes::sort_node::LimitNode;
use crate::query::planner::rewrite::macros::define_rewrite_pushdown_rule;
use crate::query::planner::rewrite::result::TransformResult;

define_rewrite_pushdown_rule! {
    /// 将LIMIT下推到扫描边操作的规则
    ///
    /// # 转换示例
    ///
    /// Before:
    /// ```text
    ///   Limit(offset=10, count=100)
    ///       |
    ///   ScanEdges
    /// ```
    ///
    /// After:
    /// ```text
    ///   Limit(offset=10, count=100)
    ///       |
    ///   ScanEdges(limit=110)
    /// ```
    ///
    /// # 适用条件
    ///
    /// - 当前节点为Limit节点
    /// - 子节点为ScanEdges节点
    /// - Limit节点只有一个子节点
    /// - ScanEdges尚未设置limit，或新limit小于现有limit
    name: PushLimitDownScanEdgesRule,
    parent_node: Limit,
    child_node: ScanEdges,
    apply: |_ctx, limit_node: &LimitNode, scan_edges_node: &ScanEdgesNode| {
        // 计算需要获取的总行数（offset + count）
        let limit_rows = limit_node.offset() + limit_node.count();

        // 检查ScanEdges是否已有更严格的limit
        if let Some(existing_limit) = scan_edges_node.limit() {
            if limit_rows >= existing_limit {
                // 现有limit更严格，无需转换
                return Ok(None::<TransformResult>);
            }
        }

        // 创建新的ScanEdges节点，设置limit
        let mut new_scan_edges = scan_edges_node.clone();
        new_scan_edges.set_limit(limit_rows);

        // 创建转换结果
        let mut result = TransformResult::new();
        result.erase_all = true;
        result.add_new_node(PlanNodeEnum::ScanEdges(new_scan_edges));

        Ok(Some(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::planner::rewrite::rule::RewriteRule;

    #[test]
    fn test_rule_name() {
        let rule = PushLimitDownScanEdgesRule::new();
        assert_eq!(rule.name(), "PushLimitDownScanEdgesRule");
    }

    #[test]
    fn test_rule_pattern() {
        let rule = PushLimitDownScanEdgesRule::new();
        let pattern = rule.pattern();
        assert!(pattern.node.is_some());
    }
}
