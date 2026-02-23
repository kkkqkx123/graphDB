//! 空集操作优化规则
//!
//! 优化集合操作中的空集情况：
//! - Minus: 如果减输入为空，直接返回主输入
//! - Intersect: 如果任一输入为空，返回空集

use crate::query::planner::plan::PlanNodeEnum;
use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;
use crate::query::planner::plan::core::nodes::start_node::StartNode;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{RewriteRule, EliminationRule};

/// 空集操作优化规则
///
/// 优化集合操作中的空集情况，避免不必要的计算
#[derive(Debug)]
pub struct EliminateEmptySetOperationRule;

impl EliminateEmptySetOperationRule {
    /// 创建规则实例
    pub fn new() -> Self {
        Self
    }

    /// 判断节点是否为空集节点
    fn is_empty_node(&self, node: &PlanNodeEnum) -> bool {
        match node {
            PlanNodeEnum::Start(_) => true,
            PlanNodeEnum::ScanVertices(n) => n.limit().map_or(false, |l| l == 0),
            PlanNodeEnum::ScanEdges(n) => n.limit().map_or(false, |l| l == 0),
            PlanNodeEnum::GetVertices(n) => n.limit().map_or(false, |l| l == 0),
            PlanNodeEnum::GetEdges(n) => n.limit().map_or(false, |l| l == 0),
            PlanNodeEnum::Limit(n) => n.count() == 0,
            PlanNodeEnum::Filter(n) => self.is_empty_node(n.input()),
            PlanNodeEnum::Project(n) => self.is_empty_node(n.input()),
            PlanNodeEnum::Dedup(n) => self.is_empty_node(n.input()),
            _ => false,
        }
    }

    /// 创建空集节点
    fn create_empty_node(&self) -> PlanNodeEnum {
        PlanNodeEnum::Start(StartNode::new())
    }
}

impl Default for EliminateEmptySetOperationRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RewriteRule for EliminateEmptySetOperationRule {
    fn name(&self) -> &'static str {
        "EliminateEmptySetOperationRule"
    }

    fn pattern(&self) -> Pattern {
        // 匹配 Minus 或 Intersect 节点
        Pattern::multi(vec!["Minus", "Intersect"])
    }

    fn apply(
        &self,
        _ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        match node {
            // 处理 Minus 节点
            PlanNodeEnum::Minus(minus_node) => {
                let minus_input = minus_node.minus_input();

                // 如果减输入为空，直接返回主输入
                if self.is_empty_node(minus_input) {
                    let mut result = TransformResult::new();
                    result.erase_curr = true;
                    result.add_new_node(minus_node.input().clone());
                    return Ok(Some(result));
                }

                Ok(None)
            }
            // 处理 Intersect 节点
            PlanNodeEnum::Intersect(intersect_node) => {
                let input = intersect_node.input();
                let intersect_input = intersect_node.intersect_input();

                // 如果任一输入为空，返回空集
                if self.is_empty_node(input) || self.is_empty_node(intersect_input) {
                    let mut result = TransformResult::new();
                    result.erase_curr = true;
                    result.add_new_node(self.create_empty_node());
                    return Ok(Some(result));
                }

                Ok(None)
            }
            _ => Ok(None),
        }
    }
}

impl EliminationRule for EliminateEmptySetOperationRule {
    fn can_eliminate(&self, node: &PlanNodeEnum) -> bool {
        match node {
            PlanNodeEnum::Minus(minus_node) => {
                self.is_empty_node(minus_node.minus_input())
            }
            PlanNodeEnum::Intersect(intersect_node) => {
                self.is_empty_node(intersect_node.input())
                    || self.is_empty_node(intersect_node.intersect_input())
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
    fn test_eliminate_empty_set_operation_rule_name() {
        let rule = EliminateEmptySetOperationRule::new();
        assert_eq!(rule.name(), "EliminateEmptySetOperationRule");
    }

    #[test]
    fn test_eliminate_empty_set_operation_rule_pattern() {
        let rule = EliminateEmptySetOperationRule::new();
        let pattern = rule.pattern();
        assert!(pattern.node.is_some());
    }
}
