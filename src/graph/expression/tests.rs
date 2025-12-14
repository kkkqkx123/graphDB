//! 表达式模块的测试

use super::*;

#[cfg(test)]
mod expression_tests {
    use super::*;

    #[test]
    fn test_expression_creation() {
        // 测试字面量创建
        let expr = Expression::int(42);
        assert_eq!(expr, Expression::Literal(LiteralValue::Int(42)));

        // 测试变量创建
        let expr = Expression::variable("x");
        assert_eq!(expr, Expression::Variable("x".to_string()));

        // 测试属性访问创建
        let expr = Expression::property(Expression::variable("a"), "name");
        assert_eq!(
            expr,
            Expression::Property {
                object: Box::new(Expression::Variable("a".to_string())),
                property: "name".to_string(),
            }
        );
    }

    #[test]
    fn test_binary_operations() {
        let left = Expression::int(10);
        let right = Expression::int(20);

        // 测试加法
        let expr = Expression::add(left.clone(), right.clone());
        assert_eq!(
            expr,
            Expression::Binary {
                left: Box::new(left.clone()),
                op: BinaryOperator::Add,
                right: Box::new(right.clone()),
            }
        );

        // 测试比较
        let expr = Expression::lt(left, right);
        assert_eq!(
            expr,
            Expression::Binary {
                left: Box::new(Expression::int(10)),
                op: BinaryOperator::LessThan,
                right: Box::new(Expression::int(20)),
            }
        );
    }

    #[test]
    fn test_unary_operations() {
        let expr = Expression::not(Expression::bool(true));
        assert_eq!(
            expr,
            Expression::Unary {
                op: UnaryOperator::Not,
                operand: Box::new(Expression::bool(true)),
            }
        );
    }

    #[test]
    fn test_function_calls() {
        let expr = Expression::function("count", vec![Expression::variable("x")]);
        assert_eq!(
            expr,
            Expression::Function {
                name: "count".to_string(),
                args: vec![Expression::Variable("x".to_string())],
            }
        );
    }

    #[test]
    fn test_aggregate_functions() {
        let expr =
            Expression::aggregate(AggregateFunction::Count, Expression::variable("x"), false);
        assert_eq!(
            expr,
            Expression::Aggregate {
                func: AggregateFunction::Count,
                arg: Box::new(Expression::Variable("x".to_string())),
                distinct: false,
            }
        );
    }

    #[test]
    fn test_containers() {
        // 测试列表
        let list = Expression::list(vec![
            Expression::int(1),
            Expression::int(2),
            Expression::int(3),
        ]);
        assert_eq!(
            list,
            Expression::List(vec![
                Expression::Literal(LiteralValue::Int(1)),
                Expression::Literal(LiteralValue::Int(2)),
                Expression::Literal(LiteralValue::Int(3)),
            ])
        );

        // 测试映射
        let map = Expression::map(vec![
            ("a", Expression::int(1)),
            ("b", Expression::string("hello")),
        ]);
        assert_eq!(
            map,
            Expression::Map(vec![
                ("a".to_string(), Expression::Literal(LiteralValue::Int(1))),
                (
                    "b".to_string(),
                    Expression::Literal(LiteralValue::String("hello".to_string()))
                ),
            ])
        );
    }

    #[test]
    fn test_expression_properties() {
        // 测试常量检查
        assert!(Expression::int(42).is_constant());
        assert!(Expression::bool(true).is_constant());
        assert!(!Expression::variable("x").is_constant());

        // 测试聚合函数检查
        let agg_expr =
            Expression::aggregate(AggregateFunction::Count, Expression::variable("x"), false);
        assert!(agg_expr.contains_aggregate());

        let simple_expr = Expression::add(Expression::int(1), Expression::int(2));
        assert!(!simple_expr.contains_aggregate());

        // 测试变量提取
        let complex_expr = Expression::add(
            Expression::variable("x"),
            Expression::mul(Expression::variable("y"), Expression::int(2)),
        );
        let vars = complex_expr.get_variables();
        assert_eq!(vars, vec!["x", "y"]);
    }

    #[test]
    fn test_type_conversions() {
        // 测试从基本类型到 LiteralValue 的转换
        let lit: LiteralValue = 42i64.into();
        assert_eq!(lit, LiteralValue::Int(42));

        let lit: LiteralValue = 3.14f64.into();
        assert_eq!(lit, LiteralValue::Float(3.14));

        let lit: LiteralValue = true.into();
        assert_eq!(lit, LiteralValue::Bool(true));

        let lit: LiteralValue = "hello".into();
        assert_eq!(lit, LiteralValue::String("hello".to_string()));

        // 测试从 LiteralValue 到 Expression 的转换
        let expr: Expression = LiteralValue::Int(42).into();
        assert_eq!(expr, Expression::Literal(LiteralValue::Int(42)));
    }
}
