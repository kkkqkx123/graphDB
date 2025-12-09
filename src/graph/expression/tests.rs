use crate::core::{Value, NullType};
use crate::graph::expression::{Expression, ExpressionKind};
use crate::graph::expression::binary::BinaryOperator;
use crate::graph::expression::unary::UnaryOperator;
use serde_json;

#[cfg(test)]
mod expression_serialization_tests {
    use super::*;

    #[test]
    fn test_constant_expression_serialization() {
        let expr = Expression::Constant(Value::Int(42));
        let serialized = serde_json::to_string(&expr).unwrap();
        let deserialized: Expression = serde_json::from_str(&serialized).unwrap();

        assert_eq!(expr, deserialized);
        assert_eq!(deserialized.kind(), ExpressionKind::Constant);
    }

    #[test]
    fn test_property_expression_serialization() {
        let expr = Expression::Property("name".to_string());
        let serialized = serde_json::to_string(&expr).unwrap();
        let deserialized: Expression = serde_json::from_str(&serialized).unwrap();

        assert_eq!(expr, deserialized);
        assert_eq!(deserialized.kind(), ExpressionKind::Variable);
    }

    #[test]
    fn test_binary_op_expression_serialization() {
        let expr = Expression::BinaryOp(
            Box::new(Expression::Constant(Value::Int(10))),
            BinaryOperator::Add,
            Box::new(Expression::Constant(Value::Int(20))),
        );
        let serialized = serde_json::to_string(&expr).unwrap();
        let deserialized: Expression = serde_json::from_str(&serialized).unwrap();

        assert_eq!(expr, deserialized);
        assert_eq!(deserialized.kind(), ExpressionKind::Arithmetic);
    }

    #[test]
    fn test_unary_op_expression_serialization() {
        let expr = Expression::UnaryOp(
            UnaryOperator::Not,
            Box::new(Expression::Constant(Value::Bool(true))),
        );
        let serialized = serde_json::to_string(&expr).unwrap();
        let deserialized: Expression = serde_json::from_str(&serialized).unwrap();

        assert_eq!(expr, deserialized);
        assert_eq!(deserialized.kind(), ExpressionKind::UnaryNot);
    }

    #[test]
    fn test_function_expression_serialization() {
        let expr = Expression::Function(
            "length".to_string(),
            vec![Expression::Constant(Value::String("hello".to_string()))],
        );
        let serialized = serde_json::to_string(&expr).unwrap();
        let deserialized: Expression = serde_json::from_str(&serialized).unwrap();

        assert_eq!(expr, deserialized);
        assert_eq!(deserialized.kind(), ExpressionKind::FunctionCall);
    }

    #[test]
    fn test_list_expression_serialization() {
        let expr = Expression::List(vec![
            Expression::Constant(Value::Int(1)),
            Expression::Constant(Value::Int(2)),
            Expression::Constant(Value::Int(3)),
        ]);
        let serialized = serde_json::to_string(&expr).unwrap();
        let deserialized: Expression = serde_json::from_str(&serialized).unwrap();

        assert_eq!(expr, deserialized);
        assert_eq!(deserialized.kind(), ExpressionKind::List);
    }

    #[test]
    fn test_map_expression_serialization() {
        let mut map_items = Vec::new();
        map_items.push(("key1".to_string(), Expression::Constant(Value::Int(100))));
        map_items.push(("key2".to_string(), Expression::Constant(Value::String("value".to_string()))));
        
        let expr = Expression::Map(map_items);
        let serialized = serde_json::to_string(&expr).unwrap();
        let deserialized: Expression = serde_json::from_str(&serialized).unwrap();

        assert_eq!(expr, deserialized);
        assert_eq!(deserialized.kind(), ExpressionKind::Map);
    }

    #[test]
    fn test_tag_property_expression_serialization() {
        let expr = Expression::TagProperty {
            tag: "person".to_string(),
            prop: "name".to_string(),
        };
        let serialized = serde_json::to_string(&expr).unwrap();
        let deserialized: Expression = serde_json::from_str(&serialized).unwrap();

        assert_eq!(expr, deserialized);
        assert_eq!(deserialized.kind(), ExpressionKind::TagProperty);
    }

    #[test]
    fn test_edge_property_expression_serialization() {
        let expr = Expression::EdgeProperty {
            edge: "friend".to_string(),
            prop: "since".to_string(),
        };
        let serialized = serde_json::to_string(&expr).unwrap();
        let deserialized: Expression = serde_json::from_str(&serialized).unwrap();

        assert_eq!(expr, deserialized);
        assert_eq!(deserialized.kind(), ExpressionKind::EdgeProperty);
    }

    #[test]
    fn test_aggregate_expression_serialization() {
        let expr = Expression::Aggregate {
            func: "count".to_string(),
            arg: Box::new(Expression::Constant(Value::Int(1))),
            distinct: true,
        };
        let serialized = serde_json::to_string(&expr).unwrap();
        let deserialized: Expression = serde_json::from_str(&serialized).unwrap();

        assert_eq!(expr, deserialized);
        assert_eq!(deserialized.kind(), ExpressionKind::Aggregate);
    }

    #[test]
    fn test_case_expression_serialization() {
        let expr = Expression::Case {
            conditions: vec![
                (Expression::Constant(Value::Bool(true)), Expression::Constant(Value::Int(1)))
            ],
            default: Some(Box::new(Expression::Constant(Value::Int(0)))),
        };
        let serialized = serde_json::to_string(&expr).unwrap();
        let deserialized: Expression = serde_json::from_str(&serialized).unwrap();

        assert_eq!(expr, deserialized);
        assert_eq!(deserialized.kind(), ExpressionKind::Relational);
    }

    #[test]
    fn test_complex_nested_expression_serialization() {
        let inner_expr = Expression::BinaryOp(
            Box::new(Expression::Constant(Value::Int(10))),
            BinaryOperator::Mul,
            Box::new(Expression::Constant(Value::Int(5))),
        );
        
        let expr = Expression::BinaryOp(
            Box::new(inner_expr),
            BinaryOperator::Add,
            Box::new(Expression::Constant(Value::Int(2))),
        );
        
        let serialized = serde_json::to_string(&expr).unwrap();
        let deserialized: Expression = serde_json::from_str(&serialized).unwrap();

        assert_eq!(expr, deserialized);
    }

    #[test]
    fn test_extended_unary_expressions_serialization() {
        // 测试扩展的一元表达式
        let exprs = vec![
            Expression::UnaryPlus(Box::new(Expression::Constant(Value::Int(5)))),
            Expression::UnaryNegate(Box::new(Expression::Constant(Value::Int(5)))),
            Expression::UnaryNot(Box::new(Expression::Constant(Value::Bool(true)))),
            Expression::UnaryIncr(Box::new(Expression::Constant(Value::Int(1)))),
            Expression::UnaryDecr(Box::new(Expression::Constant(Value::Int(1)))),
            Expression::IsNull(Box::new(Expression::Constant(Value::Null(NullType::Null)))),
            Expression::IsNotNull(Box::new(Expression::Constant(Value::Int(42)))),
            Expression::IsEmpty(Box::new(Expression::List(vec![]))),
            Expression::IsNotEmpty(Box::new(Expression::List(vec![Expression::Constant(Value::Int(1))]))),
        ];

        for expr in exprs {
            let serialized = serde_json::to_string(&expr).unwrap();
            let deserialized: Expression = serde_json::from_str(&serialized).unwrap();

            assert_eq!(expr, deserialized);
        }
    }

    #[test]
    fn test_type_casting_expression_serialization() {
        let expr = Expression::TypeCasting {
            expr: Box::new(Expression::Constant(Value::String("123".to_string()))),
            target_type: "INT".to_string(),
        };
        let serialized = serde_json::to_string(&expr).unwrap();
        let deserialized: Expression = serde_json::from_str(&serialized).unwrap();

        assert_eq!(expr, deserialized);
        assert_eq!(deserialized.kind(), ExpressionKind::TypeCasting);
    }

    #[test]
    fn test_list_comprehension_expression_serialization() {
        let expr = Expression::ListComprehension {
            generator: Box::new(Expression::List(vec![Expression::Constant(Value::Int(1))])), 
            condition: Some(Box::new(Expression::Constant(Value::Bool(true)))),
        };
        let serialized = serde_json::to_string(&expr).unwrap();
        let deserialized: Expression = serde_json::from_str(&serialized).unwrap();

        assert_eq!(expr, deserialized);
        assert_eq!(deserialized.kind(), ExpressionKind::Container);
    }

    #[test]
    fn test_various_expression_types_serialization() {
        // 测试各种表达式类型
        let exprs = vec![
            Expression::UUID,
            Expression::Variable("x".to_string()),
            Expression::Label("Person".to_string()),
            Expression::ESQuery("search query".to_string()),
        ];

        for expr in exprs {
            let serialized = serde_json::to_string(&expr).unwrap();
            let deserialized: Expression = serde_json::from_str(&serialized).unwrap();

            assert_eq!(expr, deserialized);
        }
    }

    #[test]
    fn test_subscript_expressions_serialization() {
        let collection = Expression::List(vec![
            Expression::Constant(Value::Int(10)),
            Expression::Constant(Value::Int(20)),
        ]);
        
        let index = Expression::Constant(Value::Int(0));
        
        let expr = Expression::Subscript {
            collection: Box::new(collection),
            index: Box::new(index),
        };
        
        let serialized = serde_json::to_string(&expr).unwrap();
        let deserialized: Expression = serde_json::from_str(&serialized).unwrap();

        assert_eq!(expr, deserialized);
        assert_eq!(deserialized.kind(), ExpressionKind::Relational);
    }

    #[test]
    fn test_subscript_range_expression_serialization() {
        let collection = Expression::List(vec![
            Expression::Constant(Value::Int(10)),
            Expression::Constant(Value::Int(20)),
            Expression::Constant(Value::Int(30)),
        ]);
        
        let expr = Expression::SubscriptRange {
            collection: Box::new(collection),
            start: Some(Box::new(Expression::Constant(Value::Int(0)))),
            end: Some(Box::new(Expression::Constant(Value::Int(2)))),
        };
        
        let serialized = serde_json::to_string(&expr).unwrap();
        let deserialized: Expression = serde_json::from_str(&serialized).unwrap();

        assert_eq!(expr, deserialized);
        assert_eq!(deserialized.kind(), ExpressionKind::Relational);
    }
}