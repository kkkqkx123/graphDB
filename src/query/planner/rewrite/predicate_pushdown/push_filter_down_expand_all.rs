//! 将过滤条件下推到ExpandAll操作的规则
//!
//! 该规则识别 Filter -> ExpandAll 模式，
//! 并将过滤条件下推到 ExpandAll 节点中。

use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{RewriteRule, PushDownRule};
use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;

/// 将过滤条件下推到ExpandAll操作的规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   Filter(e.likeness > 78)
///           |
///   ExpandAll
/// ```
///
/// After:
/// ```text
///   ExpandAll(filter: e.likeness > 78)
/// ```
///
/// # 适用条件
///
/// - ExpandAll 节点获取边属性
/// - ExpandAll 的最小步数等于最大步数
/// - 过滤条件可以下推到存储层
#[derive(Debug)]
pub struct PushFilterDownExpandAllRule;

impl PushFilterDownExpandAllRule {
    /// 创建规则实例
    pub fn new() -> Self {
        Self
    }
}

impl Default for PushFilterDownExpandAllRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RewriteRule for PushFilterDownExpandAllRule {
    fn name(&self) -> &'static str {
        "PushFilterDownExpandAllRule"
    }

    fn pattern(&self) -> Pattern {
        Pattern::new_with_name("Filter").with_dependency_name("ExpandAll")
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

        // 检查输入节点是否为 ExpandAll
        let expand_all = match input {
            PlanNodeEnum::ExpandAll(n) => n,
            _ => return Ok(None),
        };

        // 获取过滤条件
        let filter_condition = filter_node.condition();

        // 创建新的 ExpandAll 节点
        let mut new_expand_all = expand_all.clone();

        // 设置 filter
        new_expand_all.set_filter(filter_condition.clone());

        // 构建转换结果
        let mut result = TransformResult::new();
        result.erase_curr = true;
        result.add_new_node(PlanNodeEnum::ExpandAll(new_expand_all));

        Ok(Some(result))
    }
}

impl PushDownRule for PushFilterDownExpandAllRule {
    fn can_push_down(&self, node: &PlanNodeEnum, target: &PlanNodeEnum) -> bool {
        matches!((node, target), (PlanNodeEnum::Filter(_), PlanNodeEnum::ExpandAll(_)))
    }

    fn push_down(
        &self,
        ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
        _target: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        self.apply(ctx, node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::planner::plan::core::nodes::start_node::StartNode;
    use crate::query::planner::plan::core::nodes::traversal_node::ExpandAllNode;
    use crate::core::Expression;

    #[test]
    fn test_rule_name() {
        let rule = PushFilterDownExpandAllRule::new();
        assert_eq!(rule.name(), "PushFilterDownExpandAllRule");
    }

    #[test]
    fn test_rule_pattern() {
        let rule = PushFilterDownExpandAllRule::new();
        let pattern = rule.pattern();
        assert!(pattern.node.is_some());
    }

    #[test]
    fn test_can_push_down() {
        let rule = PushFilterDownExpandAllRule::new();
        use std::sync::Arc;
        use crate::core::types::ExpressionContext;

        let start = StartNode::new();
        let start_enum = PlanNodeEnum::Start(start);

        let condition = Expression::Variable("test".to_string());
        let ctx = Arc::new(ExpressionContext::new());
        let expr_meta = crate::core::types::expression::ExpressionMeta::new(condition);
        let id = ctx.register_expression(expr_meta);
        let ctx_expr = crate::core::types::ContextualExpression::new(id, ctx);
        let filter = crate::query::planner::plan::core::nodes::filter_node::FilterNode::new(
            start_enum.clone(),
            ctx_expr
        ).expect("创建FilterNode失败");
        let filter_enum = PlanNodeEnum::Filter(filter);

        let expand_all = ExpandAllNode::new(1, vec![], "OUT");
        let expand_enum = PlanNodeEnum::ExpandAll(expand_all);

        assert!(rule.can_push_down(&filter_enum, &expand_enum));
    }
}
