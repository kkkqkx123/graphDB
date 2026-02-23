//! 消除冗余过滤操作的规则

use crate::query::planner::plan::core::nodes::filter_node::FilterNode;
use crate::query::planner::rewrite::macros::define_simple_rewrite_elimination_rule;
use crate::query::optimizer::rule_traits::is_expression_tautology;

define_simple_rewrite_elimination_rule! {
    /// 消除冗余过滤操作的规则
    ///
    /// # 转换示例
    ///
    /// Before:
    /// ```text
    ///   Filter(TRUE)
    ///       |
    ///   ScanVertices
    /// ```
    ///
    /// After:
    /// ```text
    ///   ScanVertices
    /// ```
    ///
    /// # 适用条件
    ///
    /// - 过滤条件为永真式（如 TRUE、1=1 等）
    name: EliminateFilterRule,
    node_type: Filter,
    condition: |filter_node: &FilterNode| {
        is_expression_tautology(filter_node.condition())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::planner::rewrite::rule::RewriteRule;

    #[test]
    fn test_eliminate_filter_rule_name() {
        let rule = EliminateFilterRule::new();
        assert_eq!(rule.name(), "EliminateFilterRule");
    }

    #[test]
    fn test_eliminate_filter_rule_pattern() {
        let rule = EliminateFilterRule::new();
        let pattern = rule.pattern();
        assert!(pattern.node.is_some());
    }
}
