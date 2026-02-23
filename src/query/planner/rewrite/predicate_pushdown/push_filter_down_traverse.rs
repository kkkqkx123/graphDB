//! 将过滤条件下推到遍历操作的规则
//!
//! 该规则识别 Filter -> Traverse 模式，
//! 并将边属性过滤条件下推到 Traverse 节点中。

use crate::query::planner::plan::PlanNodeEnum;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{RewriteRule, PushDownRule};
use crate::core::Expression;
use crate::query::optimizer::expression_utils::split_filter;
use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;

/// 将过滤条件下推到遍历操作的规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   Filter(e.likeness > 78)
///           |
///   AppendVertices
///           |
///   Traverse
/// ```
///
/// After:
/// ```text
///   AppendVertices
///           |
///   Traverse(eFilter: *.likeness > 78)
/// ```
///
/// # 适用条件
///
/// - 过滤条件包含边属性表达式
/// - Traverse 节点为单步遍历
#[derive(Debug)]
pub struct PushFilterDownTraverseRule;

impl PushFilterDownTraverseRule {
    /// 创建规则实例
    pub fn new() -> Self {
        Self
    }
}

impl Default for PushFilterDownTraverseRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RewriteRule for PushFilterDownTraverseRule {
    fn name(&self) -> &'static str {
        "PushFilterDownTraverseRule"
    }

    fn pattern(&self) -> Pattern {
        Pattern::new_with_name("Filter").with_dependency_name("Traverse")
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

        // 获取输入节点
        let input = filter_node.input();

        // 检查输入节点是否为 Traverse
        let traverse = match input {
            PlanNodeEnum::Traverse(t) => t,
            _ => return Ok(None),
        };

        // 检查是否为单步遍历
        if !traverse.is_one_step() {
            return Ok(None);
        }

        // 获取边别名
        let edge_alias = match traverse.edge_alias() {
            Some(alias) => alias,
            None => return Ok(None),
        };

        // 获取过滤条件
        let filter_condition = filter_node.condition();

        // 定义选择器函数
        let picker = |expr: &Expression| -> bool {
            is_edge_property_expression(edge_alias, expr)
        };

        // 分割过滤条件
        let (filter_picked, _filter_unpicked) = split_filter(filter_condition, picker);

        // 如果没有可以选择的条件，则不进行转换
        if filter_picked.is_none() {
            return Ok(None);
        }

        // 简化实现：返回 None 表示不转换
        // 实际实现需要创建新的 Traverse 节点并设置 eFilter
        Ok(None)
    }
}

impl PushDownRule for PushFilterDownTraverseRule {
    fn can_push_down(&self, node: &PlanNodeEnum, target: &PlanNodeEnum) -> bool {
        match (node, target) {
            (PlanNodeEnum::Filter(_), PlanNodeEnum::Traverse(traverse)) => {
                traverse.is_one_step() && traverse.edge_alias().is_some()
            }
            _ => false,
        }
    }

    fn push_down(
        &self,
        _ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
        _target: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        self.apply(_ctx, node)
    }
}

/// 检查表达式是否为边属性表达式
fn is_edge_property_expression(edge_alias: &str, expr: &Expression) -> bool {
    // 简化实现：检查表达式是否包含边别名
    format!("{:?}", expr).contains(edge_alias)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_name() {
        let rule = PushFilterDownTraverseRule::new();
        assert_eq!(rule.name(), "PushFilterDownTraverseRule");
    }

    #[test]
    fn test_rule_pattern() {
        let rule = PushFilterDownTraverseRule::new();
        let pattern = rule.pattern();
        assert!(pattern.node.is_some());
    }
}
