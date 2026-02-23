//! 将TopN下推到索引扫描操作的规则
//!
//! 该规则识别 TopN -> IndexScan 模式，
//! 并将TopN的限制和排序信息集成到IndexScan操作中。

use crate::query::planner::plan::algorithms::index_scan::{IndexScan, OrderByItem};
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
    ///   TopN(count=100, sort_items=[age DESC])
    ///       |
    ///   IndexScan(limit=100, order_by=[age DESC])
    /// ```
    ///
    /// # 适用条件
    ///
    /// - 当前节点为TopN节点
    /// - 子节点为IndexScan节点
    /// - TopN节点只有一个子节点
    /// - IndexScan尚未设置limit，或新limit小于现有limit
    /// - IndexScan尚未设置order_by
    name: PushTopNDownIndexScanRule,
    parent_node: TopN,
    child_node: IndexScan,
    apply: |_ctx, topn_node: &TopNNode, index_scan_node: &IndexScan| {
        // 计算需要获取的总行数（TopN没有offset，只有limit）
        let limit_rows = topn_node.limit();

        // 检查IndexScan是否已有更严格的limit
        if let Some(existing_limit) = index_scan_node.limit {
            if limit_rows >= existing_limit {
                // 现有limit更严格，无需转换
                return Ok(None::<TransformResult>);
            }
        }

        // 检查IndexScan是否已有排序条件
        if !index_scan_node.order_by.is_empty() {
            // 已有排序条件，避免重复下推
            return Ok(None::<TransformResult>);
        }

        // 将TopN的排序项转换为IndexScan的OrderByItem
        let order_by_items: Vec<OrderByItem> = topn_node
            .sort_items()
            .iter()
            .map(|item| OrderByItem::new(item.column.clone(), item.direction.clone()))
            .collect();

        // 创建新的IndexScan节点，设置limit和order_by
        let mut new_index_scan = index_scan_node.clone();
        new_index_scan.set_limit(limit_rows);
        new_index_scan.set_order_by(order_by_items);

        // 创建转换结果
        let mut result = TransformResult::new();
        result.erase_all = true;
        result.add_new_node(PlanNodeEnum::IndexScan(new_index_scan));

        Ok(Some(result))
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
