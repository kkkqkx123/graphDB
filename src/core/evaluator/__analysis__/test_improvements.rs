//! 表达式求值器改进测试
//!
//! 测试新实现的核心功能

use crate::core::expressions::default_context::ExpressionContextCore;
use crate::core::expressions::BasicExpressionContext;
use crate::core::evaluator::ExpressionEvaluator;
use crate::core::types::expression::{Expression, LiteralValue, DataType};
use crate::core::types::operators::{BinaryOperator, UnaryOperator, AggregateFunction};
use crate::core::Value;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_context() -> BasicExpressionContext {
        let mut context = BasicExpressionContext::new();
        
        // 添加测试变量
        context.set_variable("x".to_string(), crate::core::types::query::FieldValue::Scalar(
            crate::core::types::query::ScalarValue::Int(10)
        ));
        context.set_variable("y".to_string(), crate::core::types::query::FieldValue::Scalar(
            crate::core::types::query::ScalarValue::Int(20)
        ));
        context.set_variable("name".to_string(), crate::core::types::query::FieldValue::Scalar(
            crate::core::types::query::ScalarValue::String("test".to_string())
        ));
        
        context
    }

    #[test]
    fn test_literal_evaluation() {
        let evaluator = ExpressionEvaluator::new();
        let context = create_test_context();
        
        // 测试整数字面量
        let expr = Expression::Literal(LiteralValue::Int(42));
        let result = evaluator.evaluate(&expr, &context).unwrap();
        assert_eq!(result, Value::Int(42));
        
        // 测试字符串字面量
        let expr = Expression::Literal(LiteralValue::String("hello".to_string()));
        let result = evaluator.evaluate(&expr, &context).unwrap();
        assert_eq!(result, Value::String("hello".to_string()));
        
        // 测试布尔字面量
        let expr = Expression::Literal(LiteralValue::Bool(true));
        let result = evaluator.evaluate(&expr, &context).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_variable_evaluation() {
        let evaluator = ExpressionEvaluator::new();
        let context = create_test_context();
        
        // 测试变量访问
        let expr = Expression::Variable("x".to_string());
        let result = evaluator.evaluate(&expr, &context).unwrap();
        assert_eq!(result, Value::Int(10));
        
        // 测试未定义变量
        let expr = Expression::Variable("undefined".to_string());
        let result = evaluator.evaluate(&expr, &context);
        assert!(result.is_err());
    }

    #[test]
    fn test_binary_operations() {
        let evaluator = ExpressionEvaluator::new();
        let context = create_test_context();
        
        // 测试加法
        let expr = Expression::Binary {
            left: Box::new(Expression::Variable("x".to_string())),
            op: BinaryOperator::Add,
            right: Box::new(Expression::Variable("y".to_string())),
        };
        let result = evaluator.evaluate(&expr, &context).unwrap();
        assert_eq!(result, Value::Int(30));
        
        // 测试乘法
        let expr = Expression::Binary {
            left: Box::new(Expression::Literal(LiteralValue::Int(5))),
            op: BinaryOperator::Multiply,
            right: Box::new(Expression::Literal(LiteralValue::Int(6))),
        };
        let result = evaluator.evaluate(&expr, &context).unwrap();
        assert_eq!(result, Value::Int(30));
        
        // 测试比较运算
        let expr = Expression::Binary {
            left: Box::new(Expression::Variable("x".to_string())),
            op: BinaryOperator::LessThan,
            right: Box::new(Expression::Variable("y".to_string())),
        };
        let result = evaluator.evaluate(&expr, &context).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_unary_operations() {
        let evaluator = ExpressionEvaluator::new();
        let context = create_test_context();
        
        // 测试负号
        let expr = Expression::Unary {
            op: UnaryOperator::Minus,
            operand: Box::new(Expression::Literal(LiteralValue::Int(10))),
        };
        let result = evaluator.evaluate(&expr, &context).unwrap();
        assert_eq!(result, Value::Int(-10));
        
        // 测试逻辑非
        let expr = Expression::Unary {
            op: UnaryOperator::Not,
            operand: Box::new(Expression::Literal(LiteralValue::Bool(true))),
        };
        let result = evaluator.evaluate(&expr, &context).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_type_casting() {
        let evaluator = ExpressionEvaluator::new();
        let context = create_test_context();
        
        // 测试整数转字符串
        let expr = Expression::TypeCast {
            expr: Box::new(Expression::Literal(LiteralValue::Int(42))),
            target_type: DataType::String,
        };
        let result = evaluator.evaluate(&expr, &context).unwrap();
        assert_eq!(result, Value::String("42".to_string()));
        
        // 测试字符串转整数
        let expr = Expression::TypeCast {
            expr: Box::new(Expression::Literal(LiteralValue::String("123".to_string()))),
            target_type: DataType::Int,
        };
        let result = evaluator.evaluate(&expr, &context).unwrap();
        assert_eq!(result, Value::Int(123));
    }

    #[test]
    fn test_function_calls() {
        let evaluator = ExpressionEvaluator::new();
        let context = create_test_context();
        
        // 测试abs函数
        let expr = Expression::Function {
            name: "abs".to_string(),
            args: vec![Expression::Literal(LiteralValue::Int(-5))],
        };
        let result = evaluator.evaluate(&expr, &context).unwrap();
        assert_eq!(result, Value::Int(5));
        
        // 测试length函数
        let expr = Expression::Function {
            name: "length".to_string(),
            args: vec![Expression::Literal(LiteralValue::String("hello".to_string()))],
        };
        let result = evaluator.evaluate(&expr, &context).unwrap();
        assert_eq!(result, Value::Int(5));
    }

    #[test]
    fn test_aggregate_functions() {
        let evaluator = ExpressionEvaluator::new();
        let context = create_test_context();
        
        // 测试count函数
        let expr = Expression::Aggregate {
            func: AggregateFunction::Count,
            arg: Box::new(Expression::Literal(LiteralValue::Int(42))),
            distinct: false,
        };
        let result = evaluator.evaluate(&expr, &context).unwrap();
        assert_eq!(result, Value::Int(1));
        
        // 测试sum函数
        let expr = Expression::Aggregate {
            func: AggregateFunction::Sum,
            arg: Box::new(Expression::Literal(LiteralValue::Int(42))),
            distinct: false,
        };
        let result = evaluator.evaluate(&expr, &context).unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_list_and_map() {
        let evaluator = ExpressionEvaluator::new();
        let context = create_test_context();
        
        // 测试列表
        let expr = Expression::List(vec![
            Expression::Literal(LiteralValue::Int(1)),
            Expression::Literal(LiteralValue::Int(2)),
            Expression::Literal(LiteralValue::Int(3)),
        ]);
        let result = evaluator.evaluate(&expr, &context).unwrap();
        assert_eq!(result, Value::List(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
        ]));
        
        // 测试映射
        let expr = Expression::Map(vec![
            ("a".to_string(), Expression::Literal(LiteralValue::Int(1))),
            ("b".to_string(), Expression::Literal(LiteralValue::Int(2))),
        ]);
        let result = evaluator.evaluate(&expr, &context).unwrap();
        let mut expected_map = HashMap::new();
        expected_map.insert("a".to_string(), Value::Int(1));
        expected_map.insert("b".to_string(), Value::Int(2));
        assert_eq!(result, Value::Map(expected_map));
    }

    #[test]
    fn test_property_access() {
        let evaluator = ExpressionEvaluator::new();
        let context = create_test_context();
        
        // 测试映射属性访问
        let map_expr = Expression::Map(vec![
            ("name".to_string(), Expression::Literal(LiteralValue::String("test".to_string()))),
            ("age".to_string(), Expression::Literal(LiteralValue::Int(25))),
        ]);
        
        let prop_expr = Expression::Property {
            object: Box::new(map_expr),
            property: "name".to_string(),
        };
        
        let result = evaluator.evaluate(&prop_expr, &context).unwrap();
        assert_eq!(result, Value::String("test".to_string()));
        
        // 测试列表索引访问
        let list_expr = Expression::List(vec![
            Expression::Literal(LiteralValue::Int(10)),
            Expression::Literal(LiteralValue::Int(20)),
            Expression::Literal(LiteralValue::Int(30)),
        ]);
        
        let prop_expr = Expression::Property {
            object: Box::new(list_expr),
            property: "1".to_string(),
        };
        
        let result = evaluator.evaluate(&prop_expr, &context).unwrap();
        assert_eq!(result, Value::Int(20));
    }

    #[test]
    fn test_case_expression() {
        let evaluator = ExpressionEvaluator::new();
        let context = create_test_context();
        
        let expr = Expression::Case {
            conditions: vec![
                (
                    Expression::Binary {
                        left: Box::new(Expression::Variable("x".to_string())),
                        op: BinaryOperator::GreaterThan,
                        right: Box::new(Expression::Literal(LiteralValue::Int(5))),
                    },
                    Expression::Literal(LiteralValue::String("greater".to_string())),
                ),
                (
                    Expression::Binary {
                        left: Box::new(Expression::Variable("x".to_string())),
                        op: BinaryOperator::LessThan,
                        right: Box::new(Expression::Literal(LiteralValue::Int(5))),
                    },
                    Expression::Literal(LiteralValue::String("less".to_string())),
                ),
            ],
            default: Some(Box::new(Expression::Literal(LiteralValue::String("equal".to_string())))),
        };
        
        let result = evaluator.evaluate(&expr, &context).unwrap();
        assert_eq!(result, Value::String("greater".to_string()));
    }

    #[test]
    fn test_batch_evaluation() {
        let evaluator = ExpressionEvaluator::new();
        let context = create_test_context();
        
        let expressions = vec![
            Expression::Literal(LiteralValue::Int(1)),
            Expression::Literal(LiteralValue::Int(2)),
            Expression::Literal(LiteralValue::Int(3)),
        ];
        
        let results = evaluator.evaluate_batch(&expressions, &context).unwrap();
        assert_eq!(results, vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
        ]);
    }

    #[test]
    fn test_like_operation() {
        let evaluator = ExpressionEvaluator::new();
        let context = create_test_context();
        
        // 测试 % 通配符
        let expr = Expression::Binary {
            left: Box::new(Expression::Literal(LiteralValue::String("hello world".to_string()))),
            op: BinaryOperator::Like,
            right: Box::new(Expression::Literal(LiteralValue::String("hello%".to_string())),
        };
        let result = evaluator.evaluate(&expr, &context).unwrap();
        assert_eq!(result, Value::Bool(true));
        
        // 测试 % 通配符（中间匹配）
        let expr = Expression::Binary {
            left: Box::new(Expression::Literal(LiteralValue::String("hello world".to_string()))),
            op: BinaryOperator::Like,
            right: Box::new(Expression::Literal(LiteralValue::String("h%o%ld".to_string())),
        };
        let result = evaluator.evaluate(&expr, &context).unwrap();
        assert_eq!(result, Value::Bool(true));
        
        // 测试 _ 通配符
        let expr = Expression::Binary {
            left: Box::new(Expression::Literal(LiteralValue::String("hello".to_string()))),
            op: BinaryOperator::Like,
            right: Box::new(Expression::Literal(LiteralValue::String("h_llo".to_string())),
        };
        let result = evaluator.evaluate(&expr, &context).unwrap();
        assert_eq!(result, Value::Bool(true));
        
        // 测试转义字符
        let expr = Expression::Binary {
            left: Box::new(Expression::Literal(LiteralValue::String("hello%world".to_string()))),
            op: BinaryOperator::Like,
            right: Box::new(Expression::Literal(LiteralValue::String("hello\\%world".to_string())),
        };
        let result = evaluator.evaluate(&expr, &context).unwrap();
        assert_eq!(result, Value::Bool(true));
        
        // 测试不匹配的情况
        let expr = Expression::Binary {
            left: Box::new(Expression::Literal(LiteralValue::String("hello".to_string()))),
            op: BinaryOperator::Like,
            right: Box::new(Expression::Literal(LiteralValue::String("world".to_string())),
        };
        let result = evaluator.evaluate(&expr, &context).unwrap();
        assert_eq!(result, Value::Bool(false));
        
        // 测试空字符串匹配
        let expr = Expression::Binary {
            left: Box::new(Expression::Literal(LiteralValue::String("".to_string()))),
            op: BinaryOperator::Like,
            right: Box::new(Expression::Literal(LiteralValue::String("%".to_string())),
        };
        let result = evaluator.evaluate(&expr, &context).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_complex_expressions() {
        let evaluator = ExpressionEvaluator::new();
        let context = create_test_context();
        
        // 测试复杂的嵌套表达式
        let expr = Expression::Binary {
            left: Box::new(Expression::Binary {
                left: Box::new(Expression::Variable("x".to_string())),
                op: BinaryOperator::Add,
                right: Box::new(Expression::Variable("y".to_string())),
            }),
            op: BinaryOperator::Multiply,
            right: Box::new(Expression::Unary {
                op: UnaryOperator::Minus,
                operand: Box::new(Expression::Literal(LiteralValue::Int(2))),
            }),
        };
        let result = evaluator.evaluate(&expr, &context).unwrap();
        assert_eq!(result, Value::Int(-60)); // (10 + 20) * -2 = -60
        
        // 测试逻辑表达式
        let expr = Expression::Binary {
            left: Box::new(Expression::Binary {
                left: Box::new(Expression::Variable("x".to_string())),
                op: BinaryOperator::GreaterThan,
                right: Box::new(Expression::Literal(LiteralValue::Int(5))),
            }),
            op: BinaryOperator::And,
            right: Box::new(Expression::Binary {
                left: Box::new(Expression::Variable("y".to_string())),
                op: BinaryOperator::LessThan,
                right: Box::new(Expression::Literal(LiteralValue::Int(25))),
            }),
        };
        let result = evaluator.evaluate(&expr, &context).unwrap();
        assert_eq!(result, Value::Bool(true)); // 10 > 5 AND 20 < 25 = true
    }

    #[test]
    fn test_error_handling() {
        let evaluator = ExpressionEvaluator::new();
        let context = create_test_context();
        
        // 测试除零错误
        let expr = Expression::Binary {
            left: Box::new(Expression::Literal(LiteralValue::Int(10))),
            op: BinaryOperator::Divide,
            right: Box::new(Expression::Literal(LiteralValue::Int(0))),
        };
        let result = evaluator.evaluate(&expr, &context);
        assert!(result.is_err());
        
        // 测试类型错误
        let expr = Expression::Binary {
            left: Box::new(Expression::Literal(LiteralValue::String("hello".to_string()))),
            op: BinaryOperator::Add,
            right: Box::new(Expression::Literal(LiteralValue::Int(10))),
        };
        let result = evaluator.evaluate(&expr, &context);
        assert!(result.is_err());
        
        // 测试未定义变量
        let expr = Expression::Variable("undefined".to_string());
        let result = evaluator.evaluate(&expr, &context);
        assert!(result.is_err());
    }
}