//! 子查询优化规则
//! 优化子查询的执行方式

use super::engine::OptimizerError;
use super::plan::{OptContext, OptGroupNode, OptRule, Pattern};
use super::rule_traits::BaseOptRule;
use crate::query::planner::plan::core::nodes::PlanNodeEnum;
use crate::query::visitor::PlanNodeVisitor;

/// 子查询优化规则
///
/// 将相关子查询转换为连接操作，提高查询性能。
#[derive(Debug)]
pub struct SubQueryOptimizationRule;

impl OptRule for SubQueryOptimizationRule {
    fn name(&self) -> &str {
        "SubQueryOptimizationRule"
    }

    fn apply(
        &self,
        _ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        let mut visitor = SubQueryOptimizationVisitor {
            optimized: false,
            new_node: None,
        };

        let result = visitor.visit(&node.plan_node);
        if result.optimized {
            Ok(result.new_node)
        } else {
            Ok(None)
        }
    }

    fn pattern(&self) -> Pattern {
        Pattern::new("Filter")
    }
}

impl BaseOptRule for SubQueryOptimizationRule {}

/// 子查询优化访问者
struct SubQueryOptimizationVisitor {
    optimized: bool,
    new_node: Option<OptGroupNode>,
}

impl PlanNodeVisitor for SubQueryOptimizationVisitor {
    type Result = Self;

    fn visit_default(&mut self) -> Self::Result {
        self.clone()
    }

    fn visit_filter(&mut self, node: &crate::query::planner::plan::core::nodes::FilterNode) -> Self::Result {
        use crate::core::Expression;

        let condition = node.condition();

        if let Expression::Function { name, args } = condition {
            if name.to_lowercase() == "exists" && !args.is_empty() {
                if let Some(subquery) = Self::optimize_exists_subquery(&args[0]) {
                    let mut new_node = node.clone();
                    new_node.set_condition(subquery);

                    let mut opt_node = OptGroupNode::new(node.id() as usize, PlanNodeEnum::Filter(new_node));
                    opt_node.dependencies = node.dependencies().iter().map(|d| d.id() as usize).collect();

                    self.optimized = true;
                    self.new_node = Some(opt_node);
                }
            }
        }

        self.clone()
    }
}

impl Clone for SubQueryOptimizationVisitor {
    fn clone(&self) -> Self {
        Self {
            optimized: self.optimized,
            new_node: self.new_node.clone(),
        }
    }
}

impl SubQueryOptimizationVisitor {
    fn optimize_exists_subquery(expr: &crate::core::Expression) -> Option<crate::core::Expression> {
        use crate::core::{Expression, types::operators::BinaryOperator};

        match expr {
            Expression::Binary { op: BinaryOperator::Equal, left, right } => {
                if let (Expression::Variable(_), Expression::Literal(_)) = (left.as_ref(), right.as_ref()) {
                    Some(Expression::Binary {
                        op: BinaryOperator::Equal,
                        left: left.clone(),
                        right: right.clone(),
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::context::execution::QueryContext;
    use crate::query::optimizer::plan::{OptContext, OptGroupNode};
    use crate::query::planner::plan::core::nodes::{FilterNode, StartNode};
    use crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode;

    fn create_test_context() -> OptContext {
        let query_context = QueryContext::new();
        OptContext::new(query_context)
    }

    #[test]
    fn test_sub_query_optimization_rule() {
        let rule = SubQueryOptimizationRule;
        let mut ctx = create_test_context();

        let filter_node = FilterNode::new(
            PlanNodeEnum::Start(StartNode::new()),
            crate::core::Expression::Function {
                name: "EXISTS".to_string(),
                args: vec![crate::core::Expression::Binary {
                    op: crate::core::types::operators::BinaryOperator::Equal,
                    left: Box::new(crate::core::Expression::Variable("x".to_string())),
                    right: Box::new(crate::core::Expression::Literal(crate::core::Value::String("1".to_string()))),
                }],
            },
        )
        .expect("Filter node should be created successfully");
        let opt_node = OptGroupNode::new(1, filter_node.into_enum());

        let result = rule
            .apply(&mut ctx, &opt_node)
            .expect("Rule should apply successfully");
        assert!(result.is_some());
    }
}
