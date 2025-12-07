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

    #[test]
    fn test_vertex_expression() {
        let vertex = Vertex::new(Value::Int(1), vec![]);
        let value = Value::Vertex(Box::new(vertex.clone()));
        let mut context = DefaultExpressionContext::new();
        context.set_current_vertex(vertex);

        let vertex_expr = Expression::Vertex {
            name: "VERTEX".to_string(),
        };

        let result = vertex_expr.eval(&context).unwrap();
        if let Value::Vertex(result_vertex) = result {
            assert_eq!(result_vertex.vid, Box::new(Value::Int(1)));
        } else {
            panic!("Expected Vertex value");
        }
    }

    #[test]
    fn test_edge_expression() {
        let edge = Edge::new(
            Value::Int(1), // src
            Value::Int(2), // dst
            "friend".to_string(), // type_name
            0, // rank
            std::collections::HashMap::new(), // props
        );
        let value = Value::Edge(edge.clone());
        let mut context = DefaultExpressionContext::new();
        context.set_current_edge(edge);

        let edge_expr = Expression::Edge;

        let result = edge_expr.eval(&context).unwrap();
        if let Value::Edge(result_edge) = result {
            assert_eq!(result_edge.src, Box::new(Value::Int(1)));
            assert_eq!(result_edge.dst, Box::new(Value::Int(2)));
        } else {
            panic!("Expected Edge value");
        }
    }

    #[test]
    fn test_path_build_expression() {
        use std::collections::HashMap;

        // Create vertices for the path
        let v1 = Vertex::new(Value::Int(1), vec![]);
        let v2 = Vertex::new(Value::Int(2), vec![]);

        // Create an edge between them
        let mut props = HashMap::new();
        props.insert("type".to_string(), Value::String("friend".to_string()));
        let edge = Edge::new(
            Value::Int(1), // src
            Value::Int(2), // dst
            "friend".to_string(), // type_name
            0, // rank
            props, // props
        );

        let v1_expr = Expression::Constant(Value::Vertex(Box::new(v1.clone())));
        let v2_expr = Expression::Constant(Value::Vertex(Box::new(v2.clone())));
        let edge_expr = Expression::Constant(Value::Edge(edge.clone()));

        let path_expr = Expression::PathBuild {
            items: vec![v1_expr, edge_expr, v2_expr],
        };

        let context = DefaultExpressionContext::new();
        let result = path_expr.eval(&context).unwrap();

        if let Value::Path(path) = result {
            assert_eq!(path.src.vid, Box::new(Value::Int(1)));
            assert_eq!(path.steps.len(), 1);
            assert_eq!(path.steps[0].dst.vid, Box::new(Value::Int(2)));
        } else {
            panic!("Expected Path value");
        }
    }

    #[test]
    fn test_aggregate_expression() {
        let arg_expr = Box::new(Expression::Constant(Value::Int(42)));
        let agg_expr = Expression::Aggregate {
            name: "COUNT".to_string(),
            arg: Some(arg_expr),
            distinct: false,
        };

        let context = DefaultExpressionContext::new();
        let result = agg_expr.eval(&context).unwrap();

        // This test might need adjustment based on the actual aggregation behavior
        // For now, it just checks that the evaluation doesn't panic
        assert!(matches!(result, Value::Int(_)));
    }

    #[test]
    fn test_list_comprehension_expression() {
        let list_expr = Expression::Constant(Value::List(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
            Value::Int(4),
            Value::Int(5),
        ]));

        let list_comp_expr = Expression::ListComprehension {
            inner_var: "x".to_string(),
            collection: Box::new(list_expr),
            filter: None, // No filter
            mapping: None, // No mapping, just return the items as-is
        };

        let context = DefaultExpressionContext::new();
        let result = list_comp_expr.eval(&context).unwrap();

        if let Value::List(items) = result {
            assert_eq!(items.len(), 5);
            for (i, item) in items.iter().enumerate() {
                if let Value::Int(n) = item {
                    assert_eq!(*n, (i + 1) as i64);
                } else {
                    panic!("Expected Int value at index {}", i);
                }
            }
        } else {
            panic!("Expected List value");
        }
    }
}