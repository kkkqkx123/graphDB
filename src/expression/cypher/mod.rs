//! Cypher表达式处理模块
//!
//! 这个模块提供了Cypher表达式的转换、评估和优化功能，
//! 将原本集中在cypher_compat.rs中的功能分离为三个独立的模块，
//! 以提高代码的可维护性和清晰度。

pub mod cypher_evaluator;
pub mod expression_converter;
pub mod expression_optimizer;

// 重新导出主要类型和函数，以保持向后兼容性
pub use cypher_evaluator::CypherEvaluator;
pub use expression_converter::ExpressionConverter;
pub use expression_optimizer::CypherExpressionOptimizer;

// 为了保持向后兼容性，提供一些常用的函数别名
pub use cypher_evaluator::CypherEvaluator as CypherExpressionEvaluator;
pub use expression_converter::ExpressionConverter as CypherConverter;
pub use expression_optimizer::CypherExpressionOptimizer as CypherOptimizer;

/// Cypher表达式处理的统一接口
///
/// 提供一个简单的接口来使用Cypher表达式处理功能，
/// 隐藏内部模块的复杂性。
pub struct CypherProcessor;

impl CypherProcessor {
    /// 处理Cypher表达式的完整流程：转换 -> 优化 -> 评估
    pub fn process(
        cypher_expr: &crate::query::parser::cypher::ast::expressions::Expression,
        context: &mut dyn crate::core::expressions::ExpressionContext,
    ) -> Result<crate::core::Value, crate::core::expressions::ExpressionError> {
        // 1. 转换为统一表达式
        let unified_expr = ExpressionConverter::convert_cypher_to_unified(cypher_expr)?;

        // 2. 评估统一表达式
        crate::core::evaluator::ExpressionEvaluator::new().evaluate(&unified_expr, context)
    }

    /// 处理Cypher表达式的优化流程：优化 -> 转换 -> 评估
    pub fn process_with_optimization(
        cypher_expr: &crate::query::parser::cypher::ast::expressions::Expression,
        context: &mut dyn crate::core::expressions::ExpressionContext,
    ) -> Result<crate::core::Value, crate::core::expressions::ExpressionError> {
        // 1. 优化Cypher表达式
        let optimized_expr = CypherExpressionOptimizer::optimize_cypher_expression(cypher_expr);

        // 2. 转换为统一表达式
        let unified_expr = ExpressionConverter::convert_cypher_to_unified(&optimized_expr)?;

        // 3. 评估统一表达式
        crate::core::evaluator::ExpressionEvaluator::new().evaluate(&unified_expr, context)
    }

    /// 直接评估Cypher表达式（不进行转换）
    pub fn evaluate_direct(
        cypher_expr: &crate::query::parser::cypher::ast::expressions::Expression,
        context: &mut dyn crate::core::expressions::ExpressionContext,
    ) -> Result<crate::core::Value, crate::core::expressions::ExpressionError> {
        // 使用动态分发调用泛型函数
        CypherEvaluator::evaluate_cypher(cypher_expr, context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::expressions::BasicExpressionContext;
    use crate::query::parser::cypher::ast::{UnaryOperator};
    use crate::query::parser::cypher::ast::expressions::{
        Expression as CypherExpression, Literal as CypherLiteral, UnaryExpression,
    };

    #[test]
    fn test_cypher_processor_basic() {
        let mut context = BasicExpressionContext::default();
        let cypher_expr = CypherExpression::Literal(CypherLiteral::Integer(42));

        let result = CypherProcessor::process(&cypher_expr, &mut context).expect("Cypher processing should succeed for literal values");
        assert_eq!(result, crate::core::Value::Int(42));
    }

    #[test]
    fn test_cypher_processor_with_optimization() {
        let mut context = BasicExpressionContext::default();
        let expr = CypherExpression::Unary(UnaryExpression {
            operator: UnaryOperator::Not,
            expression: Box::new(CypherExpression::Literal(CypherLiteral::Boolean(true))),
        });

        let result = CypherProcessor::process_with_optimization(&expr, &mut context).expect("Cypher processing with optimization should succeed");
        assert_eq!(result, crate::core::Value::Bool(false));
    }

    #[test]
    fn test_cypher_processor_evaluate_direct() {
        let mut context = BasicExpressionContext::default();
        // 将 Value 转换为 FieldValue
        let field_value = crate::core::types::query::FieldValue::Scalar(
            crate::core::types::query::ScalarValue::Int(100),
        );
        context.set_variable("x".to_string(), field_value);

        let cypher_expr = CypherExpression::Variable("x".to_string());
        let result = CypherProcessor::evaluate_direct(&cypher_expr, &mut context).expect("Cypher direct evaluation should succeed");
        assert_eq!(result, crate::core::Value::Int(100));
    }
}
