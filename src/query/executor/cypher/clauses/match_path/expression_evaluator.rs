//! 统一的表达式求值器
//!
//! 直接使用graph/expression模块，消除重复代码
//! 提供完整的Cypher表达式评估功能

use crate::core::error::DBError;
use crate::core::Value;
use crate::expression::evaluator::traits::ExpressionContext;
use crate::query::executor::cypher::context::CypherExecutionContext;
use crate::query::executor::cypher::CypherExpressionEvaluator;
use crate::query::parser::cypher::ast::expressions::Expression;

/// 统一的表达式求值器
///
/// 使用CypherExpressionEvaluator提供完整的Cypher表达式评估功能
/// 使用 unit struct 模式，零开销
#[derive(Debug)]
pub struct ExpressionEvaluator;

impl ExpressionEvaluator {
    /// 求值Cypher表达式
    pub fn evaluate(expr: &Expression, context: &CypherExecutionContext) -> Result<Value, DBError> {
        let mut eval_context = Self::convert_context(context);

        CypherExpressionEvaluator
            .evaluate_cypher(expr, &mut eval_context)
            .map_err(|e| {
                DBError::Query(crate::core::error::QueryError::ExecutionError(
                    e.to_string(),
                ))
            })
    }

    /// 批量求值Cypher表达式
    pub fn evaluate_batch(
        exprs: &[Expression],
        context: &CypherExecutionContext,
    ) -> Result<Vec<Value>, DBError> {
        let mut eval_context = Self::convert_context(context);

        CypherExpressionEvaluator
            .evaluate_cypher_batch(exprs, &mut eval_context)
            .map_err(|e| {
                DBError::Query(crate::core::error::QueryError::ExecutionError(
                    e.to_string(),
                ))
            })
    }

    /// 转换上下文类型
    fn convert_context(
        context: &CypherExecutionContext,
    ) -> crate::expression::BasicExpressionContext {
        let mut eval_context = crate::expression::BasicExpressionContext::default();

        for (name, cypher_var) in context.variables() {
            if let Some(value) = &cypher_var.value {
                eval_context.set_variable(name.clone(), value.clone());
            }
        }

        for (name, value) in &context.base_context().variables {
            if eval_context.get_variable(name).is_none() {
                eval_context.set_variable(name.clone(), value.clone());
            }
        }

        for (name, value) in context.parameters() {
            if eval_context.get_variable(&format!("${}", name)).is_none() {
                eval_context.set_variable(format!("${}", name), value.clone());
            }
        }

        for (name, _path) in context.paths() {
            let empty_vertex = crate::core::vertex_edge_path::Vertex::default();
            let empty_path = crate::core::vertex_edge_path::Path {
                src: Box::new(empty_vertex),
                steps: Vec::new(),
            };
            let path_value = crate::core::Value::Path(empty_path);
            eval_context.set_variable(name.clone(), path_value);
        }

        eval_context
    }

    /// 检查表达式是否为常量
    pub fn is_constant(expr: &Expression) -> bool {
        CypherExpressionEvaluator.is_cypher_constant(expr)
    }

    /// 获取表达式中使用的所有变量
    pub fn get_variables(expr: &Expression) -> Vec<String> {
        CypherExpressionEvaluator.get_cypher_variables(expr)
    }

    /// 检查表达式是否包含聚合函数
    pub fn contains_aggregate(expr: &Expression) -> bool {
        CypherExpressionEvaluator.contains_cypher_aggregate(expr)
    }

    /// 优化表达式
    pub fn optimize_expression(expr: &Expression) -> Expression {
        CypherExpressionEvaluator.optimize_cypher_expression(expr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::parser::cypher::ast::expressions::{
        BinaryExpression, Literal, UnaryExpression,
    };
    use crate::query::parser::cypher::ast::{BinaryOperator, UnaryOperator};

    #[test]
    fn test_evaluate_literal() {
        let context = CypherExecutionContext::new();

        let string_expr = Expression::Literal(Literal::String("test".to_string()));
        let result = ExpressionEvaluator::evaluate(&string_expr, &context)
            .expect("Failed to evaluate string expression");
        assert_eq!(result, Value::String("test".to_string()));

        let int_expr = Expression::Literal(Literal::Integer(42));
        let result = ExpressionEvaluator::evaluate(&int_expr, &context)
            .expect("Failed to evaluate int expression");
        assert_eq!(result, Value::Int(42));

        let bool_expr = Expression::Literal(Literal::Boolean(true));
        let result = ExpressionEvaluator::evaluate(&bool_expr, &context)
            .expect("Failed to evaluate bool expression");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_evaluate_binary_expression() {
        let context = CypherExecutionContext::new();

        let equal_expr = Expression::Binary(BinaryExpression {
            left: Box::new(Expression::Literal(Literal::Integer(42))),
            operator: BinaryOperator::Equal,
            right: Box::new(Expression::Literal(Literal::Integer(42))),
        });
        let result = ExpressionEvaluator::evaluate(&equal_expr, &context)
            .expect("Failed to evaluate equal expression");
        assert_eq!(result, Value::Bool(true));

        let not_equal_expr = Expression::Binary(BinaryExpression {
            left: Box::new(Expression::Literal(Literal::Integer(42))),
            operator: BinaryOperator::NotEqual,
            right: Box::new(Expression::Literal(Literal::Integer(43))),
        });
        let result = ExpressionEvaluator::evaluate(&not_equal_expr, &context)
            .expect("Failed to evaluate not equal expression");
        assert_eq!(result, Value::Bool(true));

        let and_expr = Expression::Binary(BinaryExpression {
            left: Box::new(Expression::Literal(Literal::Boolean(true))),
            operator: BinaryOperator::And,
            right: Box::new(Expression::Literal(Literal::Boolean(true))),
        });
        let result = ExpressionEvaluator::evaluate(&and_expr, &context)
            .expect("Failed to evaluate and expression");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_evaluate_unary_expression() {
        let context = CypherExecutionContext::new();

        let not_expr = Expression::Unary(UnaryExpression {
            operator: UnaryOperator::Not,
            expression: Box::new(Expression::Literal(Literal::Boolean(true))),
        });
        let result = ExpressionEvaluator::evaluate(&not_expr, &context)
            .expect("Failed to evaluate not expression");
        assert_eq!(result, Value::Bool(false));

        let neg_expr = Expression::Unary(UnaryExpression {
            operator: UnaryOperator::Minus,
            expression: Box::new(Expression::Literal(Literal::Integer(42))),
        });
        let result = ExpressionEvaluator::evaluate(&neg_expr, &context)
            .expect("Failed to evaluate neg expression");
        assert_eq!(result, Value::Int(-42));
    }

    #[test]
    fn test_arithmetic_operations() {
        let context = CypherExecutionContext::new();

        let add_expr = Expression::Binary(BinaryExpression {
            left: Box::new(Expression::Literal(Literal::Integer(10))),
            operator: BinaryOperator::Add,
            right: Box::new(Expression::Literal(Literal::Integer(5))),
        });
        let result = ExpressionEvaluator::evaluate(&add_expr, &context)
            .expect("Failed to evaluate add expression");
        assert_eq!(result, Value::Int(15));

        let concat_expr = Expression::Binary(BinaryExpression {
            left: Box::new(Expression::Literal(Literal::String("Hello".to_string()))),
            operator: BinaryOperator::Add,
            right: Box::new(Expression::Literal(Literal::String(" World".to_string()))),
        });
        let result = ExpressionEvaluator::evaluate(&concat_expr, &context)
            .expect("Failed to evaluate concat expression");
        assert_eq!(result, Value::String("Hello World".to_string()));
    }

    #[test]
    fn test_variable_collection() {
        let var_expr = Expression::Variable("test_var".to_string());
        let variables = ExpressionEvaluator::get_variables(&var_expr);
        assert_eq!(variables, vec!["test_var"]);

        let binary_expr = Expression::Binary(BinaryExpression {
            left: Box::new(Expression::Variable("a".to_string())),
            operator: BinaryOperator::Add,
            right: Box::new(Expression::Variable("b".to_string())),
        });
        let variables = ExpressionEvaluator::get_variables(&binary_expr);
        assert_eq!(variables, vec!["a", "b"]);
    }

    #[test]
    fn test_constant_check() {
        let literal_expr = Expression::Literal(Literal::Integer(42));
        assert!(ExpressionEvaluator::is_constant(&literal_expr));

        let var_expr = Expression::Variable("test".to_string());
        assert!(!ExpressionEvaluator::is_constant(&var_expr));
    }

    #[test]
    fn test_aggregate_check() {
        let count_expr = Expression::FunctionCall(
            crate::query::parser::cypher::ast::expressions::FunctionCall {
                function_name: "count".to_string(),
                arguments: vec![Expression::Variable("x".to_string())],
            },
        );
        assert!(ExpressionEvaluator::contains_aggregate(&count_expr));

        let add_expr = Expression::Binary(BinaryExpression {
            left: Box::new(Expression::Variable("a".to_string())),
            operator: BinaryOperator::Add,
            right: Box::new(Expression::Variable("b".to_string())),
        });
        assert!(!ExpressionEvaluator::contains_aggregate(&add_expr));
    }

    #[test]
    fn test_batch_evaluation() {
        let context = CypherExecutionContext::new();

        let exprs = vec![
            Expression::Literal(Literal::Integer(1)),
            Expression::Literal(Literal::Integer(2)),
            Expression::Literal(Literal::Integer(3)),
        ];

        let results = ExpressionEvaluator::evaluate_batch(&exprs, &context)
            .expect("Failed to evaluate batch expressions");
        assert_eq!(results.len(), 3);
        assert_eq!(results[0], Value::Int(1));
        assert_eq!(results[1], Value::Int(2));
        assert_eq!(results[2], Value::Int(3));
    }
}
