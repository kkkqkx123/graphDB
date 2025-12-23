//! 统一的表达式求值器
//!
//! 直接使用graph/expression模块，消除重复代码
//! 提供完整的Cypher表达式评估功能

use crate::core::error::DBError;
use crate::core::expressions::ExpressionContext;
use crate::core::ExpressionEvaluator as GraphExpressionEvaluator;
use crate::core::Value;
use crate::query::executor::cypher::context::CypherExecutionContext;
use crate::query::parser::cypher::ast::expressions::Expression;

/// 统一的表达式求值器
///
/// 直接使用graph/expression模块的ExpressionEvaluator，提供完整的Cypher表达式评估功能
/// 这个实现消除了表达式系统的重复，确保了统一性
#[derive(Debug)]
pub struct ExpressionEvaluator {
    inner: GraphExpressionEvaluator,
}

impl ExpressionEvaluator {
    /// 创建新的表达式求值器
    pub fn new() -> Self {
        Self {
            inner: GraphExpressionEvaluator::new(),
        }
    }

    /// 求值Cypher表达式
    pub fn evaluate(
        &self,
        expr: &Expression,
        context: &CypherExecutionContext,
    ) -> Result<Value, DBError> {
        // 将CypherExecutionContext转换为graph/expression模块需要的上下文
        let mut eval_context = self.convert_context(context);

        // 直接使用统一的表达式求值器
        self.inner
            .evaluate_cypher(expr, &mut eval_context)
            .map_err(|e| {
                DBError::Query(crate::core::error::QueryError::ExecutionError(
                    e.to_string(),
                ))
            })
    }

    /// 批量求值Cypher表达式
    pub fn evaluate_batch(
        &self,
        exprs: &[Expression],
        context: &CypherExecutionContext,
    ) -> Result<Vec<Value>, DBError> {
        let mut eval_context = self.convert_context(context);

        // 使用统一的批量评估功能
        self.inner
            .evaluate_cypher_batch(exprs, &mut eval_context)
            .map_err(|e| {
                DBError::Query(crate::core::error::QueryError::ExecutionError(
                    e.to_string(),
                ))
            })
    }

    /// 转换上下文类型
    fn convert_context(
        &self,
        context: &CypherExecutionContext,
    ) -> crate::core::expressions::BasicExpressionContext {
        // 创建新的求值上下文
        let mut eval_context = crate::core::expressions::BasicExpressionContext::default();

        // 复制变量
        for (name, cypher_var) in context.variables() {
            if let Some(value) = &cypher_var.value {
                // 将 Value 转换为 FieldValue
                let field_value = convert_value_to_field_value(value.clone());
                eval_context.set_variable(name.clone(), field_value);
            }
        }

        // 复制基础上下文中的变量
        for (name, value) in &context.base_context().variables {
            if eval_context.get_variable(name).is_none() {
                // 将 Value 转换为 FieldValue
                let field_value = convert_value_to_field_value(value.clone());
                eval_context.set_variable(name.clone(), field_value);
            }
        }

        // 复制参数作为变量
        for (name, value) in context.parameters() {
            if eval_context.get_variable(&format!("${}", name)).is_none() {
                // 将 Value 转换为 FieldValue
                let field_value = convert_value_to_field_value(value.clone());
                eval_context.set_variable(format!("${}", name), field_value);
            }
        }

        // 复制路径信息
        for (name, path) in context.paths() {
            // 将 Path 转换为 FieldValue
            let path_field_value =
                crate::core::types::query::FieldValue::Path(crate::core::types::query::Path {
                    segments: Vec::new(), // 简化实现，实际需要完整转换
                });
            eval_context.set_variable(name.clone(), path_field_value);
        }

        eval_context
    }

    /// 检查表达式是否为常量
    pub fn is_constant(&self, expr: &Expression) -> bool {
        // 使用统一的常量检查功能
        self.inner.is_cypher_constant(expr)
    }

    /// 获取表达式中使用的所有变量
    pub fn get_variables(&self, expr: &Expression) -> Vec<String> {
        // 使用统一的变量收集功能
        self.inner.get_cypher_variables(expr)
    }

    /// 检查表达式是否包含聚合函数
    pub fn contains_aggregate(&self, expr: &Expression) -> bool {
        // 使用统一的聚合函数检查功能
        self.inner.contains_cypher_aggregate(expr)
    }

    /// 优化表达式
    pub fn optimize_expression(&self, expr: &Expression) -> Expression {
        // 使用统一的表达式优化功能
        match self.inner.optimize_cypher_expression(expr) {
            Ok(opt_expr) => opt_expr,
            Err(_) => expr.clone(), // 如果优化失败，返回原始表达式
        }
    }
}

impl Default for ExpressionEvaluator {
    fn default() -> Self {
        Self::new()
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
        let evaluator = ExpressionEvaluator::new();
        let mut context = CypherExecutionContext::new();

        let string_expr = Expression::Literal(Literal::String("test".to_string()));
        let result = evaluator
            .evaluate(&string_expr, &mut context)
            .expect("Failed to evaluate string expression");
        assert_eq!(result, Value::String("test".to_string()));

        let int_expr = Expression::Literal(Literal::Integer(42));
        let result = evaluator
            .evaluate(&int_expr, &mut context)
            .expect("Failed to evaluate int expression");
        assert_eq!(result, Value::Int(42));

        let bool_expr = Expression::Literal(Literal::Boolean(true));
        let result = evaluator
            .evaluate(&bool_expr, &mut context)
            .expect("Failed to evaluate bool expression");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_evaluate_binary_expression() {
        let evaluator = ExpressionEvaluator::new();
        let mut context = CypherExecutionContext::new();

        // 测试相等比较
        let equal_expr = Expression::Binary(BinaryExpression {
            left: Box::new(Expression::Literal(Literal::Integer(42))),
            operator: BinaryOperator::Equal,
            right: Box::new(Expression::Literal(Literal::Integer(42))),
        });
        let result = evaluator
            .evaluate(&equal_expr, &mut context)
            .expect("Failed to evaluate equal expression");
        assert_eq!(result, Value::Bool(true));

        // 测试不相等比较
        let not_equal_expr = Expression::Binary(BinaryExpression {
            left: Box::new(Expression::Literal(Literal::Integer(42))),
            operator: BinaryOperator::NotEqual,
            right: Box::new(Expression::Literal(Literal::Integer(43))),
        });
        let result = evaluator
            .evaluate(&not_equal_expr, &mut context)
            .expect("Failed to evaluate not equal expression");
        assert_eq!(result, Value::Bool(true));

        // 测试AND操作
        let and_expr = Expression::Binary(BinaryExpression {
            left: Box::new(Expression::Literal(Literal::Boolean(true))),
            operator: BinaryOperator::And,
            right: Box::new(Expression::Literal(Literal::Boolean(true))),
        });
        let result = evaluator
            .evaluate(&and_expr, &mut context)
            .expect("Failed to evaluate and expression");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_evaluate_unary_expression() {
        let evaluator = ExpressionEvaluator::new();
        let mut context = CypherExecutionContext::new();

        // 测试NOT操作
        let not_expr = Expression::Unary(UnaryExpression {
            operator: UnaryOperator::Not,
            expression: Box::new(Expression::Literal(Literal::Boolean(true))),
        });
        let result = evaluator
            .evaluate(&not_expr, &mut context)
            .expect("Failed to evaluate not expression");
        assert_eq!(result, Value::Bool(false));

        // 测试负号
        let neg_expr = Expression::Unary(UnaryExpression {
            operator: UnaryOperator::Minus,
            expression: Box::new(Expression::Literal(Literal::Integer(42))),
        });
        let result = evaluator
            .evaluate(&neg_expr, &mut context)
            .expect("Failed to evaluate neg expression");
        assert_eq!(result, Value::Int(-42));
    }

    #[test]
    fn test_arithmetic_operations() {
        let evaluator = ExpressionEvaluator::new();
        let mut context = CypherExecutionContext::new();

        // 测试加法
        let add_expr = Expression::Binary(BinaryExpression {
            left: Box::new(Expression::Literal(Literal::Integer(10))),
            operator: BinaryOperator::Add,
            right: Box::new(Expression::Literal(Literal::Integer(5))),
        });
        let result = evaluator
            .evaluate(&add_expr, &mut context)
            .expect("Failed to evaluate add expression");
        assert_eq!(result, Value::Int(15));

        // 测试字符串连接
        let concat_expr = Expression::Binary(BinaryExpression {
            left: Box::new(Expression::Literal(Literal::String("Hello".to_string()))),
            operator: BinaryOperator::Add,
            right: Box::new(Expression::Literal(Literal::String(" World".to_string()))),
        });
        let result = evaluator
            .evaluate(&concat_expr, &mut context)
            .expect("Failed to evaluate concat expression");
        assert_eq!(result, Value::String("Hello World".to_string()));
    }

    #[test]
    fn test_variable_collection() {
        let evaluator = ExpressionEvaluator::new();

        let var_expr = Expression::Variable("test_var".to_string());
        let variables = evaluator.get_variables(&var_expr);
        assert_eq!(variables, vec!["test_var"]);

        let binary_expr = Expression::Binary(BinaryExpression {
            left: Box::new(Expression::Variable("a".to_string())),
            operator: BinaryOperator::Add,
            right: Box::new(Expression::Variable("b".to_string())),
        });
        let variables = evaluator.get_variables(&binary_expr);
        assert_eq!(variables, vec!["a", "b"]);
    }

    #[test]
    fn test_constant_check() {
        let evaluator = ExpressionEvaluator::new();

        let literal_expr = Expression::Literal(Literal::Integer(42));
        assert!(evaluator.is_constant(&literal_expr));

        let var_expr = Expression::Variable("test".to_string());
        assert!(!evaluator.is_constant(&var_expr));
    }

    #[test]
    fn test_aggregate_check() {
        let evaluator = ExpressionEvaluator::new();

        let count_expr = Expression::FunctionCall(
            crate::query::parser::cypher::ast::expressions::FunctionCall {
                function_name: "count".to_string(),
                arguments: vec![Expression::Variable("x".to_string())],
            },
        );
        assert!(evaluator.contains_aggregate(&count_expr));

        let add_expr = Expression::Binary(BinaryExpression {
            left: Box::new(Expression::Variable("a".to_string())),
            operator: BinaryOperator::Add,
            right: Box::new(Expression::Variable("b".to_string())),
        });
        assert!(!evaluator.contains_aggregate(&add_expr));
    }

    #[test]
    fn test_batch_evaluation() {
        let evaluator = ExpressionEvaluator::new();
        let mut context = CypherExecutionContext::new();

        let exprs = vec![
            Expression::Literal(Literal::Integer(1)),
            Expression::Literal(Literal::Integer(2)),
            Expression::Literal(Literal::Integer(3)),
        ];

        let results = evaluator
            .evaluate_batch(&exprs, &context)
            .expect("Failed to evaluate batch expressions");
        assert_eq!(results.len(), 3);
        assert_eq!(results[0], Value::Int(1));
        assert_eq!(results[1], Value::Int(2));
        assert_eq!(results[2], Value::Int(3));
    }
}

/// 将 Value 转换为 FieldValue
fn convert_value_to_field_value(value: Value) -> crate::core::types::query::FieldValue {
    match value {
        Value::Bool(b) => crate::core::types::query::FieldValue::Scalar(
            crate::core::types::query::ScalarValue::Bool(b),
        ),
        Value::Int(i) => crate::core::types::query::FieldValue::Scalar(
            crate::core::types::query::ScalarValue::Int(i),
        ),
        Value::Float(f) => crate::core::types::query::FieldValue::Scalar(
            crate::core::types::query::ScalarValue::Float(f),
        ),
        Value::String(s) => crate::core::types::query::FieldValue::Scalar(
            crate::core::types::query::ScalarValue::String(s),
        ),
        Value::Null(_) => crate::core::types::query::FieldValue::Scalar(
            crate::core::types::query::ScalarValue::Null,
        ),
        _ => {
            // 对于复杂类型，暂时返回空值
            crate::core::types::query::FieldValue::Scalar(
                crate::core::types::query::ScalarValue::Null,
            )
        }
    }
}
