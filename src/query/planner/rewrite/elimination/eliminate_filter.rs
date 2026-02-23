//! 消除冗余过滤操作的规则

use crate::query::planner::plan::PlanNodeEnum;
use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{RewriteRule, EliminationRule};
use crate::query::optimizer::rule_traits::is_expression_tautology;

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
#[derive(Debug)]
pub struct EliminateFilterRule;

impl EliminateFilterRule {
    /// 创建规则实例
    pub fn new() -> Self {
        Self
    }
}

impl Default for EliminateFilterRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RewriteRule for EliminateFilterRule {
    fn name(&self) -> &'static str {
        "EliminateFilterRule"
    }

    fn pattern(&self) -> Pattern {
        Pattern::new_with_name("Filter")
    }

    fn apply(
        &self,
        _ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        // 检查是否为 Filter 节点
        let filter_node = match node {
            PlanNodeEnum::Filter(n) => n,
            _ => return Ok(None),
        };

        // 检查过滤条件是否为永真式
        let condition = filter_node.condition();
        if !is_expression_tautology(condition) {
            return Ok(None);
        }

        // 获取输入节点
        let input = filter_node.input();

        // 创建转换结果，用输入节点替换当前 Filter 节点
        let mut result = TransformResult::new();
        result.erase_curr = true;
        result.add_new_node(input.clone());

        Ok(Some(result))
    }
}

impl EliminationRule for EliminateFilterRule {
    fn can_eliminate(&self, node: &PlanNodeEnum) -> bool {
        match node {
            PlanNodeEnum::Filter(n) => {
                let condition = n.condition();
                is_expression_tautology(condition)
            }
            _ => false,
        }
    }

    fn eliminate(
        &self,
        _ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        self.apply(_ctx, node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
