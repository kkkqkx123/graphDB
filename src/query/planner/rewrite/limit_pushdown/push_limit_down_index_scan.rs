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
    ///   Limit(offset=10, count=100)
    ///       |
    ///   IndexScan
    /// ```
    ///
    /// After:
    /// ```text
    ///   Limit(offset=10, count=100)
    ///       |
    ///   IndexScan(limit=110)
    /// ```
    ///
    /// # 适用条件
    ///
    /// - 当前节点为Limit节点
    /// - 子节点为IndexScan节点
    /// - Limit节点只有一个子节点
    /// - IndexScan尚未设置limit，或新limit小于现有limit
    name: PushLimitDownIndexScanRule,
    parent_node: Limit,
    child_node: IndexScan,
    apply: |_ctx, limit_node: &LimitNode, index_scan_node: &IndexScan| {
        // 计算需要获取的总行数（offset + count）
        let limit_rows = limit_node.offset() + limit_node.count();

        // 检查IndexScan是否已有更严格的limit
        if let Some(existing_limit) = index_scan_node.limit {
            if limit_rows >= existing_limit {
                // 现有limit更严格，无需转换
                return Ok(None::<TransformResult>);
            }
        }

        // 创建新的IndexScan节点，设置limit
        let mut new_index_scan = index_scan_node.clone();
        new_index_scan.set_limit(limit_rows);

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
