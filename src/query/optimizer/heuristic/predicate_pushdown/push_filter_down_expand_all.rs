//! Rules that push the filtering conditions to the ExpandAll operation
//!
//! This rule identifies the “Filter -> ExpandAll” mode.
//! And push the filtering criteria up to the ExpandAll node.

use crate::query::optimizer::heuristic::context::RewriteContext;
use crate::query::optimizer::heuristic::pattern::Pattern;
use crate::query::optimizer::heuristic::result::{RewriteResult, TransformResult};
use crate::query::optimizer::heuristic::rule::{PushDownRule, RewriteRule};
use crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum;
use crate::query::planning::plan::core::nodes::base::plan_node_traits::SingleInputNode;

/// Rules that push the filtering criteria forward to the ExpandAll operation
///
/// # Conversion example
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
/// # Applicable Conditions
///
/// The “ExpandAll” node is used to retrieve the properties of the edges.
/// The minimum step size for “ExpandAll” is equal to the maximum step size.
/// The filtering criteria can be pushed down to the storage layer.
#[derive(Debug)]
pub struct PushFilterDownExpandAllRule;

impl PushFilterDownExpandAllRule {
    /// Create a rule instance.
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
        // Check whether it is a Filter node.
        let filter_node = match node {
            PlanNodeEnum::Filter(n) => n,
            _ => return Ok(None),
        };

        // Obtain the input node
        let input = filter_node.input();

        // Check whether the input node is of the ExpandAll type.
        let expand_all = match input {
            PlanNodeEnum::ExpandAll(n) => n,
            _ => return Ok(None),
        };

        // Obtain the filtering criteria
        let filter_condition = filter_node.condition();

        // Create a new ExpandAll node.
        let mut new_expand_all = expand_all.clone();

        // Set the filter
        new_expand_all.set_filter(filter_condition.clone());

        // Construct the translation result.
        let mut result = TransformResult::new();
        result.erase_curr = true;
        result.add_new_node(PlanNodeEnum::ExpandAll(new_expand_all));

        Ok(Some(result))
    }
}

impl PushDownRule for PushFilterDownExpandAllRule {
    fn can_push_down(&self, node: &PlanNodeEnum, target: &PlanNodeEnum) -> bool {
        matches!(
            (node, target),
            (PlanNodeEnum::Filter(_), PlanNodeEnum::ExpandAll(_))
        )
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
    use crate::core::Expression;
    use crate::query::planning::plan::core::nodes::control_flow::start_node::StartNode;
    use crate::query::planning::plan::core::nodes::traversal::traversal_node::ExpandAllNode;

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
        use crate::query::validator::context::ExpressionAnalysisContext;
        use std::sync::Arc;

        let start = StartNode::new();
        let start_enum = PlanNodeEnum::Start(start);

        let condition = Expression::Variable("test".to_string());
        let ctx = Arc::new(ExpressionAnalysisContext::new());
        let expr_meta = crate::core::types::expr::ExpressionMeta::new(condition);
        let id = ctx.register_expression(expr_meta);
        let ctx_expr = crate::core::types::ContextualExpression::new(id, ctx);
        let filter =
            crate::query::planning::plan::core::nodes::operation::filter_node::FilterNode::new(
                start_enum.clone(),
                ctx_expr,
            )
            .expect("创建FilterNode失败");
        let filter_enum = PlanNodeEnum::Filter(filter);

        let expand_all = ExpandAllNode::new(1, vec![], "OUT");
        let expand_enum = PlanNodeEnum::ExpandAll(expand_all);

        assert!(rule.can_push_down(&filter_enum, &expand_enum));
    }
}
