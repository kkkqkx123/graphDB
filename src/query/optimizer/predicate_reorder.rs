//! 谓词重排序优化规则
//! 重新排列谓词顺序以优化查询性能

use super::engine::OptimizerError;
use super::plan::{OptContext, OptGroupNode, OptRule, Pattern};
use super::rule_patterns::PatternBuilder;
use super::rule_traits::BaseOptRule;
use crate::query::planner::plan::core::nodes::PlanNodeEnum;
use crate::query::visitor::PlanNodeVisitor;

/// 谓词重排序规则
///
/// 将过滤条件按照选择性从高到低排序，尽早过滤掉更多数据。
#[derive(Debug)]
pub struct PredicateReorderRule;

impl OptRule for PredicateReorderRule {
    fn name(&self) -> &str {
        "PredicateReorderRule"
    }

    fn apply(
        &self,
        _ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        let mut visitor = PredicateReorderVisitor {
            reordered: false,
            new_node: None,
        };

        let result = visitor.visit(&node.plan_node);
        if result.reordered {
            Ok(result.new_node)
        } else {
            Ok(None)
        }
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::filter()
    }
}

impl BaseOptRule for PredicateReorderRule {}

/// 谓词重排序访问者
struct PredicateReorderVisitor {
    reordered: bool,
    new_node: Option<OptGroupNode>,
}

impl PlanNodeVisitor for PredicateReorderVisitor {
    type Result = Self;

    fn visit_default(&mut self) -> Self::Result {
        self.clone()
    }

    fn visit_filter(&mut self, node: &crate::query::planner::plan::core::nodes::FilterNode) -> Self::Result {
        use crate::core::Expression;

        let condition = node.condition();

        if let Expression::Binary { op: crate::core::types::operators::BinaryOperator::And, left, right } = condition {
            let left_selectivity = Self::estimate_selectivity(left);
            let right_selectivity = Self::estimate_selectivity(right);

            if left_selectivity < right_selectivity {
                let reordered_condition = Expression::Binary {
                    op: crate::core::types::operators::BinaryOperator::And,
                    left: right.clone(),
                    right: left.clone(),
                };

                let mut new_node = node.clone();
                new_node.set_condition(reordered_condition);

                let mut opt_node = OptGroupNode::new(node.id() as usize, PlanNodeEnum::Filter(new_node));
                opt_node.dependencies = node.dependencies().iter().map(|d| d.id() as usize).collect();

                self.reordered = true;
                self.new_node = Some(opt_node);
            }
        }

        self.clone()
    }
}

impl Clone for PredicateReorderVisitor {
    fn clone(&self) -> Self {
        Self {
            reordered: self.reordered,
            new_node: self.new_node.clone(),
        }
    }
}

impl PredicateReorderVisitor {
    fn estimate_selectivity(expr: &crate::core::Expression) -> f64 {
        use crate::core::{Expression, types::operators::BinaryOperator};

        match expr {
            Expression::Binary { op, left, right } => {
                match op {
                    BinaryOperator::Equal => {
                        let left_is_literal = matches!(left.as_ref(), Expression::Literal(_));
                        let right_is_literal = matches!(right.as_ref(), Expression::Literal(_));
                        if left_is_literal || right_is_literal {
                            0.01
                        } else {
                            0.1
                        }
                    }
                    BinaryOperator::NotEqual => 0.9,
                    BinaryOperator::LessThan | BinaryOperator::LessThanOrEqual | 
                    BinaryOperator::GreaterThan | BinaryOperator::GreaterThanOrEqual => 0.33,
                    BinaryOperator::And => {
                        Self::estimate_selectivity(left) * Self::estimate_selectivity(right)
                    }
                    BinaryOperator::Or => {
                        1.0 - (1.0 - Self::estimate_selectivity(left)) * (1.0 - Self::estimate_selectivity(right))
                    }
                    _ => 0.5,
                }
            }
            Expression::Function { name, .. } => {
                match name.to_lowercase().as_str() {
                    "id" => 0.01,
                    "exists" => 0.5,
                    _ => 0.1,
                }
            }
            _ => 0.5,
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
    fn test_predicate_reorder_rule() {
        let rule = PredicateReorderRule;
        let mut ctx = create_test_context();

        let filter_node = FilterNode::new(
            PlanNodeEnum::Start(StartNode::new()),
            crate::core::Expression::Binary {
                op: crate::core::BinaryOperator::And,
                left: Box::new(crate::core::Expression::Binary {
                    op: crate::core::BinaryOperator::Equal,
                    left: Box::new(crate::core::Expression::Literal(crate::core::Value::String("value1".to_string()))),
                    right: Box::new(crate::core::Expression::Variable("col1".to_string())),
                }),
                right: Box::new(crate::core::Expression::Binary {
                    op: crate::core::BinaryOperator::LessThan,
                    left: Box::new(crate::core::Expression::Variable("col2".to_string())),
                    right: Box::new(crate::core::Expression::Literal(crate::core::Value::Int(100))),
                }),
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
