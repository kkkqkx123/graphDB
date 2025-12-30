//! Cypher表达式处理模块
//!
//! 提供Cypher表达式处理的统一接口

use crate::core::value::Value;
use crate::expression::ExpressionContext;
use crate::expression::ExpressionError;

/// Cypher表达式处理的统一接口
///
/// 提供一个简单的接口来使用Cypher表达式处理功能，
/// 隐藏内部模块的复杂性。
pub struct CypherProcessor;

impl CypherProcessor {
    pub fn process<C: ExpressionContext>(
        cypher_expr: &crate::query::parser::cypher::ast::expressions::Expression,
        context: &mut C,
    ) -> Result<Value, ExpressionError> {
        let optimized_expr =
            super::expression_optimizer::CypherExpressionOptimizer::optimize_cypher_expression(
                cypher_expr,
            );

        let unified_expr =
            super::expression_converter::ExpressionConverter::convert_cypher_to_unified(
                &optimized_expr,
            )?;

        crate::expression::evaluator::ExpressionEvaluator::evaluate(&unified_expr, context)
    }

    pub fn process_with_optimization<C: ExpressionContext>(
        cypher_expr: &crate::query::parser::cypher::ast::expressions::Expression,
        context: &mut C,
    ) -> Result<Value, ExpressionError> {
        let optimized_expr =
            super::expression_optimizer::CypherExpressionOptimizer::optimize_cypher_expression(
                cypher_expr,
            );

        let unified_expr =
            super::expression_converter::ExpressionConverter::convert_cypher_to_unified(
                &optimized_expr,
            )?;

        crate::expression::evaluator::ExpressionEvaluator::evaluate(&unified_expr, context)
    }

    pub fn evaluate_direct<C: ExpressionContext>(
        cypher_expr: &crate::query::parser::cypher::ast::expressions::Expression,
        context: &mut C,
    ) -> Result<Value, ExpressionError> {
        super::expression_evaluator::CypherEvaluator::evaluate_cypher(cypher_expr, context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expression::BasicExpressionContext;
    use crate::query::parser::cypher::ast::expressions::{
        Expression as CypherExpression, Literal as CypherLiteral, UnaryExpression,
    };
    use crate::query::parser::cypher::ast::UnaryOperator;

    #[test]
    fn test_cypher_processor_basic() {
        let mut context = BasicExpressionContext::default();
        let cypher_expr = CypherExpression::Literal(CypherLiteral::Integer(42));

        let result = CypherProcessor::process(&cypher_expr, &mut context)
            .expect("Cypher processing should succeed for literal values");
        assert_eq!(result, crate::core::value::Value::Int(42));
    }

    #[test]
    fn test_cypher_processor_with_optimization() {
        let mut context = BasicExpressionContext::default();
        let expr = CypherExpression::Unary(UnaryExpression {
            operator: UnaryOperator::Not,
            expression: Box::new(CypherExpression::Literal(CypherLiteral::Boolean(true))),
        });

        let result = CypherProcessor::process_with_optimization(&expr, &mut context)
            .expect("Cypher processing with optimization should succeed");
        assert_eq!(result, crate::core::value::Value::Bool(false));
    }

    #[test]
    fn test_cypher_processor_evaluate_direct() {
        let mut context = BasicExpressionContext::default();
        let value = crate::core::value::Value::Int(100);
        context.set_variable("x".to_string(), value);

        let cypher_expr = CypherExpression::Variable("x".to_string());
        let result = CypherProcessor::evaluate_direct(&cypher_expr, &mut context)
            .expect("Cypher direct evaluation should succeed");
        assert_eq!(result, crate::core::value::Value::Int(100));
    }
}
