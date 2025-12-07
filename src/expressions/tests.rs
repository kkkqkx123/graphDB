#[cfg(test)]
mod tests {
    use crate::expressions::{Expression, ExpressionContext, DefaultExpressionContext, EvaluationError, UnaryOp, BinaryOp};
    use crate::core::{Value, Vertex, Edge, Tag};

    #[test]
    fn test_constant_expression() {
        let context = DefaultExpressionContext::new();
        let const_expr = Expression::Constant(Value::Int(42));
        let result = const_expr.eval(&context).unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_variable_expression() {
        let mut context = DefaultExpressionContext::new();
        context.set_variable("x".to_string(), Value::Int(42));

        let var_expr = Expression::Variable {
            name: "x".to_string(),
        };

        let result = var_expr.eval(&context).unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_unary_expression() {
        let context = DefaultExpressionContext::new();

        let unary_expr = Expression::Unary {
            op: UnaryOp::Minus,
            operand: Box::new(Expression::Constant(Value::Int(42))),
        };

        let result = unary_expr.eval(&context).unwrap();
        assert_eq!(result, Value::Int(-42));
    }

    #[test]
    fn test_binary_expression() {
        let context = DefaultExpressionContext::new();

        let binary_expr = Expression::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expression::Constant(Value::Int(10))),
            right: Box::new(Expression::Constant(Value::Int(32))),
        };

        let result = binary_expr.eval(&context).unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_list_container_expression() {
        let context = DefaultExpressionContext::new();

        let list_expr = Expression::List(vec![
            Expression::Constant(Value::Int(1)),
            Expression::Constant(Value::Int(2)),
            Expression::Constant(Value::Int(3)),
        ]);

        let result = list_expr.eval(&context).unwrap();
        match result {
            Value::List(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0], Value::Int(1));
                assert_eq!(items[1], Value::Int(2));
                assert_eq!(items[2], Value::Int(3));
            },
            _ => panic!("Expected List value"),
        }
    }

    #[test]
    fn test_map_container_expression() {
        let context = DefaultExpressionContext::new();

        let map_expr = Expression::Map(vec![
            (Expression::Constant(Value::String("name".to_string())),
             Expression::Constant(Value::String("Alice".to_string()))),
            (Expression::Constant(Value::String("age".to_string())),
             Expression::Constant(Value::Int(30))),
        ]);

        let result = map_expr.eval(&context).unwrap();
        match result {
            Value::Map(map) => {
                assert_eq!(map.len(), 2);
                assert_eq!(map.get("name"), Some(&Value::String("Alice".to_string())));
                assert_eq!(map.get("age"), Some(&Value::Int(30)));
            },
            _ => panic!("Expected Map value"),
        }
    }

    #[test]
    fn test_set_container_expression() {
        let context = DefaultExpressionContext::new();

        let set_expr = Expression::Set(vec![
            Expression::Constant(Value::Int(1)),
            Expression::Constant(Value::Int(2)),
            Expression::Constant(Value::Int(3)),
            Expression::Constant(Value::Int(1)), // Duplicate
        ]);

        let result = set_expr.eval(&context).unwrap();
        match result {
            Value::Set(set) => {
                assert_eq!(set.len(), 3); // Should only have unique values
                assert!(set.contains(&Value::Int(1)));
                assert!(set.contains(&Value::Int(2)));
                assert!(set.contains(&Value::Int(3)));
            },
            _ => panic!("Expected Set value"),
        }
    }

    #[test]
    fn test_function_call_expression() {
        let context = DefaultExpressionContext::new();

        let func_expr = Expression::FunctionCall {
            name: "strlen".to_string(),
            args: vec![
                Expression::Constant(Value::String("hello".to_string())),
            ],
        };

        let result = func_expr.eval(&context).unwrap();
        assert_eq!(result, Value::Int(5));
    }

    #[test]
    fn test_binary_operation_in() {
        let context = DefaultExpressionContext::new();

        let in_expr = Expression::Binary {
            op: BinaryOp::In,
            left: Box::new(Expression::Constant(Value::Int(2))),
            right: Box::new(Expression::List(vec![
                Expression::Constant(Value::Int(1)),
                Expression::Constant(Value::Int(2)),
                Expression::Constant(Value::Int(3)),
            ])),
        };

        let result = in_expr.eval(&context).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_property_access_on_vertex() {
        use std::collections::HashMap;

        let mut props = HashMap::new();
        props.insert("age".to_string(), Value::Int(25));
        let tag = Tag::new("person".to_string(), props);
        let vertex = Vertex::new(Value::Int(1), vec![tag]);
        let value = Value::Vertex(Box::new(vertex));

        let mut context = DefaultExpressionContext::new();
        context.set_variable("v".to_string(), value);

        // Test property access expression
        let property_expr = Expression::Property {
            entity: Box::new(Expression::Variable { name: "v".to_string() }),
            property: "id".to_string(),
        };

        let result = property_expr.eval(&context).unwrap();
        assert_eq!(result, Value::String("1".to_string()));
    }

    #[test]
    fn test_case_expression() {
        let context = DefaultExpressionContext::new();

        let case_expr = Expression::Case {
            conditions: vec![
                (
                    Expression::Binary {
                        op: BinaryOp::Eq,
                        left: Box::new(Expression::Constant(Value::Int(5))),
                        right: Box::new(Expression::Constant(Value::Int(3))),
                    },
                    Expression::Constant(Value::String("not equal".to_string())),
                ),
                (
                    Expression::Binary {
                        op: BinaryOp::Eq,
                        left: Box::new(Expression::Constant(Value::Int(5))),
                        right: Box::new(Expression::Constant(Value::Int(5))),
                    },
                    Expression::Constant(Value::String("equal".to_string())),
                ),
            ],
            default: Some(Box::new(Expression::Constant(Value::String("default".to_string())))),
        };

        let result = case_expr.eval(&context).unwrap();
        assert_eq!(result, Value::String("equal".to_string()));
    }
}