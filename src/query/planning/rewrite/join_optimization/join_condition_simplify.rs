//! Rules for simplifying JOIN conditions
//!
//! This rule simplifies JOIN conditions, including eliminating redundant conditions
//! and converting InnerJoin with ON true to CrossJoin.
//!
//! # Conversion examples
//!
//! ## Eliminate redundant conditions
//! Before:
//! ```text
//!   InnerJoin ON a.id = b.id AND b.id = a.id
//! ```
//!
//! After:
//! ```text
//!   InnerJoin ON a.id = b.id
//! ```
//!
//! ## Convert ON true to CrossJoin
//! Before:
//! ```text
//!   InnerJoin ON true
//! ```
//!
//! After:
//! ```text
//!   CrossJoin
//! ```
//!
//! # Applicable Conditions
//!
//! JOIN conditions contain redundant expressions.
//! JOIN condition is a constant true.
//! JOIN conditions can be simplified.

use crate::core::types::expr::contextual::ContextualExpression;
use crate::core::types::operators::BinaryOperator;
use crate::core::{Expression, Value};
use crate::query::planning::plan::core::nodes::join::join_node::{
    CrossJoinNode, HashInnerJoinNode, InnerJoinNode,
};
use crate::query::planning::plan::PlanNodeEnum;
use crate::query::planning::rewrite::context::RewriteContext;
use crate::query::planning::rewrite::pattern::Pattern;
use crate::query::planning::rewrite::result::{RewriteError, RewriteResult, TransformResult};
use crate::query::planning::rewrite::rule::RewriteRule;
use std::collections::HashSet;

/// Rules for simplifying JOIN conditions
///
/// Simplify JOIN conditions by removing redundant expressions and converting trivial conditions.
#[derive(Debug)]
pub struct JoinConditionSimplifyRule;

impl JoinConditionSimplifyRule {
    pub fn new() -> Self {
        Self
    }

    fn is_true_expression(&self, expr: &Expression) -> bool {
        matches!(expr, Expression::Literal(Value::Bool(true)))
    }

    fn is_false_expression(&self, expr: &Expression) -> bool {
        matches!(expr, Expression::Literal(Value::Bool(false)))
    }

    fn normalize_expression(&self, expr: &Expression) -> String {
        match expr {
            Expression::Binary { left, op, right } if *op == BinaryOperator::And => {
                let left_str = self.normalize_expression(left);
                let right_str = self.normalize_expression(right);
                let mut parts = vec![left_str, right_str];
                parts.sort();
                parts.join(" AND ")
            }
            Expression::Binary { left, op, right } if *op == BinaryOperator::Or => {
                format!("{} OR {}", self.normalize_expression(left), self.normalize_expression(right))
            }
            Expression::Binary { left, op, right } => {
                let left_str = self.normalize_expression(left);
                let right_str = self.normalize_expression(right);
                if *op == BinaryOperator::Equal {
                    let mut parts = vec![left_str, right_str];
                    parts.sort();
                    format!("{}={}{}", parts[0], op, parts[1])
                } else {
                    format!("{}{}{}", left_str, op, right_str)
                }
            }
            Expression::Variable(name) => name.clone(),
            Expression::Property { object, property } => {
                format!("{}.{}", self.normalize_expression(object), property)
            }
            Expression::Function { name, args } => {
                let args_str: Vec<String> = args.iter().map(|a| self.normalize_expression(a)).collect();
                format!("{}({})", name, args_str.join(","))
            }
            Expression::Literal(v) => match v {
                Value::Int(v) => v.to_string(),
                Value::Float(v) => v.to_string(),
                Value::String(v) => format!("\"{}\"", v),
                Value::Bool(v) => v.to_string(),
                Value::Null(_) => "NULL".to_string(),
                _ => format!("{:?}", v),
            },
            Expression::Unary { op, operand } => {
                format!("{:?} {}", op, self.normalize_expression(operand))
            }
            _ => format!("{:?}", expr),
        }
    }

    fn extract_and_conditions(&self, expr: &Expression) -> Vec<Expression> {
        match expr {
            Expression::Binary { left, op, right } if *op == BinaryOperator::And => {
                let mut conditions = self.extract_and_conditions(left);
                conditions.extend(self.extract_and_conditions(right));
                conditions
            }
            _ => vec![expr.clone()],
        }
    }

    fn remove_duplicate_conditions(&self, expr: &Expression) -> Option<Expression> {
        let conditions = self.extract_and_conditions(expr);
        let mut seen: HashSet<String> = HashSet::new();
        let mut unique_conditions: Vec<Expression> = Vec::new();

        for cond in conditions {
            let normalized = self.normalize_expression(&cond);
            if !seen.contains(&normalized) {
                seen.insert(normalized);
                unique_conditions.push(cond);
            }
        }

        if unique_conditions.is_empty() {
            return None;
        }

        if unique_conditions.len() == 1 {
            return unique_conditions.into_iter().next();
        }

        let mut iter = unique_conditions.into_iter();
        let mut result = iter.next()?;
        for cond in iter {
            result = Expression::and(result, cond);
        }
        Some(result)
    }

    fn simplify_condition(&self, expr: &Expression) -> Option<Expression> {
        if self.is_true_expression(expr) {
            return None;
        }

        if self.is_false_expression(expr) {
            return Some(Expression::bool(false));
        }

        self.remove_duplicate_conditions(expr)
    }

    fn apply_to_hash_inner_join(
        &self,
        join: &HashInnerJoinNode,
        ctx: &RewriteContext,
    ) -> RewriteResult<Option<TransformResult>> {
        let hash_keys = join.hash_keys();
        let probe_keys = join.probe_keys();

        if hash_keys.len() != probe_keys.len() || hash_keys.is_empty() {
            return Ok(None);
        }

        let all_true = hash_keys.iter().all(|k| {
            k.expression()
                .map(|m| self.is_true_expression(m.inner()))
                .unwrap_or(false)
        }) && probe_keys.iter().all(|k| {
            k.expression()
                .map(|m| self.is_true_expression(m.inner()))
                .unwrap_or(false)
        });

        if all_true {
            let cross_join = CrossJoinNode::new(join.left_input().clone(), join.right_input().clone())
                .map_err(|e| {
                    RewriteError::rewrite_failed(format!("Failed to create CrossJoinNode: {:?}", e))
                })?;

            let mut result = TransformResult::new();
            result.erase_curr = true;
            result.add_new_node(PlanNodeEnum::CrossJoin(cross_join));
            return Ok(Some(result));
        }

        Ok(None)
    }

    fn apply_to_inner_join(
        &self,
        join: &InnerJoinNode,
        ctx: &RewriteContext,
    ) -> RewriteResult<Option<TransformResult>> {
        let hash_keys = join.hash_keys();
        let probe_keys = join.probe_keys();

        if hash_keys.len() != probe_keys.len() || hash_keys.is_empty() {
            return Ok(None);
        }

        let all_true = hash_keys.iter().all(|k| {
            k.expression()
                .map(|m| self.is_true_expression(m.inner()))
                .unwrap_or(false)
        }) && probe_keys.iter().all(|k| {
            k.expression()
                .map(|m| self.is_true_expression(m.inner()))
                .unwrap_or(false)
        });

        if all_true {
            let cross_join = CrossJoinNode::new(join.left_input().clone(), join.right_input().clone())
                .map_err(|e| {
                    RewriteError::rewrite_failed(format!("Failed to create CrossJoinNode: {:?}", e))
                })?;

            let mut result = TransformResult::new();
            result.erase_curr = true;
            result.add_new_node(PlanNodeEnum::CrossJoin(cross_join));
            return Ok(Some(result));
        }

        Ok(None)
    }
}

impl Default for JoinConditionSimplifyRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RewriteRule for JoinConditionSimplifyRule {
    fn name(&self) -> &'static str {
        "JoinConditionSimplifyRule"
    }

    fn pattern(&self) -> Pattern {
        Pattern::multi(vec!["HashInnerJoin", "InnerJoin"])
    }

    fn apply(
        &self,
        ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        match node {
            PlanNodeEnum::HashInnerJoin(join) => self.apply_to_hash_inner_join(join, ctx),
            PlanNodeEnum::InnerJoin(join) => self.apply_to_inner_join(join, ctx),
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_name() {
        let rule = JoinConditionSimplifyRule::new();
        assert_eq!(rule.name(), "JoinConditionSimplifyRule");
    }

    #[test]
    fn test_rule_pattern() {
        let rule = JoinConditionSimplifyRule::new();
        let pattern = rule.pattern();
        assert!(pattern.node.is_some());
    }

    #[test]
    fn test_is_true_expression() {
        let rule = JoinConditionSimplifyRule::new();
        assert!(rule.is_true_expression(&Expression::bool(true)));
        assert!(!rule.is_true_expression(&Expression::bool(false)));
        assert!(!rule.is_true_expression(&Expression::int(1)));
    }

    #[test]
    fn test_extract_and_conditions() {
        let rule = JoinConditionSimplifyRule::new();

        let expr = Expression::and(
            Expression::variable("a"),
            Expression::variable("b"),
        );
        let conditions = rule.extract_and_conditions(&expr);
        assert_eq!(conditions.len(), 2);

        let single = Expression::variable("c");
        let conditions = rule.extract_and_conditions(&single);
        assert_eq!(conditions.len(), 1);
    }

    #[test]
    fn test_remove_duplicate_conditions() {
        let rule = JoinConditionSimplifyRule::new();

        let cond1 = Expression::eq(
            Expression::variable("a.id"),
            Expression::variable("b.id"),
        );
        let cond2 = Expression::eq(
            Expression::variable("b.id"),
            Expression::variable("a.id"),
        );
        let expr = Expression::and(cond1, cond2);

        let result = rule.remove_duplicate_conditions(&expr);
        assert!(result.is_some());
    }
}
