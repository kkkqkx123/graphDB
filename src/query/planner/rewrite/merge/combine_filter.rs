//! 合并多个过滤操作的规则

use crate::core::Expression;
use crate::query::planner::plan::core::nodes::filter_node::FilterNode;
use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;
use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{MergeRule, RewriteRule};

/// 合并多个过滤操作的规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   Filter(col2 > 200)
///       |
///   Filter(col1 > 100)
///       |
///   ScanVertices
/// ```
///
/// After:
/// ```text
///   Filter(col1 > 100 AND col2 > 200)
///       |
///   ScanVertices
/// ```
///
/// # 适用条件
///
/// - 当前节点为Filter节点
/// - 子节点也为Filter节点
/// - 可以合并两个过滤条件
#[derive(Debug)]
pub struct CombineFilterRule;

impl CombineFilterRule {
    /// 创建规则实例
    pub fn new() -> Self {
        Self
    }

    /// 合并两个条件表达式
    fn combine_conditions(&self, top: &Expression, child: &Expression) -> Expression {
        Expression::Binary {
            left: Box::new(child.clone()),
            op: crate::core::types::operators::BinaryOperator::And,
            right: Box::new(top.clone()),
        }
    }
}

impl Default for CombineFilterRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RewriteRule for CombineFilterRule {
    fn name(&self) -> &'static str {
        "CombineFilterRule"
    }

    fn pattern(&self) -> Pattern {
        Pattern::new_with_name("Filter").with_dependency_name("Filter")
    }

    fn apply(
        &self,
        _ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        // 检查是否为 Filter 节点
        let top_filter = match node {
            PlanNodeEnum::Filter(n) => n,
            _ => return Ok(None),
        };

        // 获取输入节点
        let input = top_filter.input();

        // 检查输入节点是否也是 Filter
        let child_filter = match input {
            PlanNodeEnum::Filter(n) => n,
            _ => return Ok(None),
        };

        // 获取两个过滤条件
        let top_condition = top_filter.condition();
        let child_condition = child_filter.condition();

        // 合并条件
        let combined_condition = self.combine_conditions(top_condition, child_condition);

        // 获取子 Filter 的输入
        let child_input = child_filter.input().clone();

        // 创建合并后的 Filter 节点
        let combined_filter_node = match FilterNode::new(child_input, combined_condition) {
            Ok(node) => node,
            Err(_) => return Ok(None),
        };

        // 创建转换结果
        let mut result = TransformResult::new();
        result.erase_curr = true;
        result.add_new_node(PlanNodeEnum::Filter(combined_filter_node));

        Ok(Some(result))
    }
}

impl MergeRule for CombineFilterRule {
    fn can_merge(&self, parent: &PlanNodeEnum, child: &PlanNodeEnum) -> bool {
        parent.is_filter() && child.is_filter()
    }

    fn create_merged_node(
        &self,
        _ctx: &mut RewriteContext,
        parent: &PlanNodeEnum,
        _child: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        self.apply(_ctx, parent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Expression;
    use crate::query::planner::plan::core::nodes::start_node::StartNode;

    #[test]
    fn test_rule_name() {
        let rule = CombineFilterRule::new();
        assert_eq!(rule.name(), "CombineFilterRule");
    }

    #[test]
    fn test_rule_pattern() {
        let rule = CombineFilterRule::new();
        let pattern = rule.pattern();
        assert!(pattern.node.is_some());
    }

    #[test]
    fn test_combine_filters() {
        let start = PlanNodeEnum::Start(StartNode::new());

        // 下层Filter: col1 > 100
        let child_condition = Expression::Binary {
            left: Box::new(Expression::Variable("col1".to_string())),
            op: crate::core::types::operators::BinaryOperator::GreaterThan,
            right: Box::new(Expression::Literal(crate::core::Value::Int(100))),
        };
        let child_filter =
            FilterNode::new(start, child_condition).expect("创建FilterNode失败");
        let child_node = PlanNodeEnum::Filter(child_filter);

        // 上层Filter: col2 > 200
        let top_condition = Expression::Binary {
            left: Box::new(Expression::Variable("col2".to_string())),
            op: crate::core::types::operators::BinaryOperator::GreaterThan,
            right: Box::new(Expression::Literal(crate::core::Value::Int(200))),
        };
        let top_filter =
            FilterNode::new(child_node.clone(), top_condition).expect("创建FilterNode失败");
        let top_node = PlanNodeEnum::Filter(top_filter);

        // 应用规则
        let rule = CombineFilterRule::new();
        let mut ctx = RewriteContext::new();
        let result = rule.apply(&mut ctx, &top_node).expect("应用规则失败");

        assert!(result.is_some(), "应该成功合并两个Filter节点");

        let transform_result = result.unwrap();
        assert!(transform_result.erase_curr);
        assert_eq!(transform_result.new_nodes.len(), 1);
    }
}
