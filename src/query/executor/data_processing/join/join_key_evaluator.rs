//! Join键求值器
//!
//! 专门用于Join操作的键求值，支持表达式求值到可哈希的Value类型

use crate::core::error::ExpressionError;
use crate::core::types::expression::Expression;
use crate::core::Value;
use crate::expression::evaluator::expression_evaluator::ExpressionEvaluator;
use crate::expression::evaluator::traits::ExpressionContext;

/// Join键求值器
///
/// 专门为Join操作设计的表达式求值器，将表达式求值为可哈希的Value类型
#[derive(Debug)]
pub struct JoinKeyEvaluator {
    evaluator: ExpressionEvaluator,
}

impl JoinKeyEvaluator {
    pub fn new() -> Self {
        Self {
            evaluator: ExpressionEvaluator::new(),
        }
    }

    pub fn with_evaluator(evaluator: ExpressionEvaluator) -> Self {
        Self { evaluator }
    }

    pub fn evaluator(&self) -> &ExpressionEvaluator {
        &self.evaluator
    }

    pub fn evaluator_mut(&mut self) -> &mut ExpressionEvaluator {
        &mut self.evaluator
    }

    pub fn evaluate_key<C: ExpressionContext>(
        &self,
        expr: &Expression,
        context: &mut C,
    ) -> Result<Value, ExpressionError> {
        self.evaluator.evaluate(expr, context)
    }

    pub fn evaluate_keys<C: ExpressionContext>(
        &self,
        exprs: &[Expression],
        context: &mut C,
    ) -> Result<Vec<Value>, ExpressionError> {
        let mut keys = Vec::with_capacity(exprs.len());
        for expr in exprs {
            keys.push(self.evaluate_key(expr, context)?);
        }
        Ok(keys)
    }

    pub fn evaluate_key_dyn(
        &self,
        expr: &Expression,
        context: &mut dyn ExpressionContext,
    ) -> Result<Value, ExpressionError> {
        self.evaluator.evaluate(expr, context)
    }

    pub fn evaluate_keys_dyn(
        &self,
        exprs: &[Expression],
        context: &mut dyn ExpressionContext,
    ) -> Result<Vec<Value>, ExpressionError> {
        let mut keys = Vec::with_capacity(exprs.len());
        for expr in exprs {
            keys.push(self.evaluate_key_dyn(expr, context)?);
        }
        Ok(keys)
    }

    pub fn is_simple_variable(expr: &Expression) -> bool {
        matches!(expr, Expression::Variable(_))
    }

    pub fn is_simple_property(expr: &Expression) -> bool {
        matches!(expr, Expression::Property { .. })
    }

    pub fn get_variable_name(expr: &Expression) -> Option<&str> {
        match expr {
            Expression::Variable(name) => Some(name),
            _ => None,
        }
    }

    pub fn get_property_info(expr: &Expression) -> Option<(&Expression, &str)> {
        match expr {
            Expression::Property { object, property } => Some((object.as_ref(), property)),
            _ => None,
        }
    }
}

impl Default for JoinKeyEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::expression::Expression;

    #[test]
    fn test_is_simple_variable() {
        let var_expr = Expression::Variable("id".to_string());
        assert!(JoinKeyEvaluator::is_simple_variable(&var_expr));

        let lit_expr = Expression::Literal(Value::Int(42));
        assert!(!JoinKeyEvaluator::is_simple_variable(&lit_expr));
    }

    #[test]
    fn test_get_variable_name() {
        let var_expr = Expression::Variable("name".to_string());
        assert_eq!(JoinKeyEvaluator::get_variable_name(&var_expr), Some("name"));

        let lit_expr = Expression::Literal(Value::Int(42));
        assert_eq!(JoinKeyEvaluator::get_variable_name(&lit_expr), None);
    }

    #[test]
    fn test_is_simple_property() {
        let prop_expr = Expression::Property {
            object: Box::new(Expression::Variable("person".to_string())),
            property: "age".to_string(),
        };
        assert!(JoinKeyEvaluator::is_simple_property(&prop_expr));

        let var_expr = Expression::Variable("id".to_string());
        assert!(!JoinKeyEvaluator::is_simple_property(&var_expr));
    }

    #[test]
    fn test_get_property_info() {
        let prop_expr = Expression::Property {
            object: Box::new(Expression::Variable("person".to_string())),
            property: "age".to_string(),
        };
        let (object, property) = JoinKeyEvaluator::get_property_info(&prop_expr).unwrap();
        assert!(matches!(object, Expression::Variable(_)));
        assert_eq!(property, "age");
    }
}
