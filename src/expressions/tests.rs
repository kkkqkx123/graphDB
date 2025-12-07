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

// eval_context.rs 模块的单元测试
#[cfg(test)]
mod eval_context_tests {
    use crate::expressions::{
        SimpleExpressionContext, SimpleExpressionEvaluator,
        Expression, EvaluationError, UnaryOp, BinaryOp
    };
    use crate::core::Value;

    #[test]
    fn test_simple_expression_context_new() {
        let context = SimpleExpressionContext::new();
        assert!(context.variable_names().is_empty());
    }

    #[test]
    fn test_simple_expression_context_set_and_get_variable() {
        let mut context = SimpleExpressionContext::new();
        
        // 设置变量
        context.set_variable("x".to_string(), Value::Int(42));
        
        // 获取变量
        let result = context.get_variable_direct("x").unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_simple_expression_context_get_undefined_variable() {
        let context = SimpleExpressionContext::new();
        
        // 获取未定义的变量应该返回错误
        let result = context.get_variable_direct("undefined");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), EvaluationError::UndefinedVariable(_)));
    }

    #[test]
    fn test_simple_expression_context_has_variable() {
        let mut context = SimpleExpressionContext::new();
        
        // 初始时变量不存在
        assert!(!context.has_variable("x"));
        
        // 设置变量后应该存在
        context.set_variable("x".to_string(), Value::Int(42));
        assert!(context.has_variable("x"));
    }

    #[test]
    fn test_simple_expression_context_variable_names() {
        let mut context = SimpleExpressionContext::new();
        
        // 设置多个变量
        context.set_variable("x".to_string(), Value::Int(42));
        context.set_variable("y".to_string(), Value::String("hello".to_string()));
        context.set_variable("z".to_string(), Value::Bool(true));
        
        // 获取所有变量名
        let names = context.variable_names();
        assert_eq!(names.len(), 3);
        assert!(names.contains(&"x".to_string()));
        assert!(names.contains(&"y".to_string()));
        assert!(names.contains(&"z".to_string()));
    }

    #[test]
    fn test_simple_expression_context_set_variables() {
        let mut context = SimpleExpressionContext::new();
        
        // 批量设置变量
        let mut vars = std::collections::HashMap::new();
        vars.insert("a".to_string(), Value::Int(1));
        vars.insert("b".to_string(), Value::Int(2));
        context.set_variables(vars);
        
        // 验证变量已设置
        assert_eq!(context.get_variable_direct("a").unwrap(), Value::Int(1));
        assert_eq!(context.get_variable_direct("b").unwrap(), Value::Int(2));
        assert_eq!(context.variable_names().len(), 2);
    }

    #[test]
    fn test_simple_expression_context_clear() {
        let mut context = SimpleExpressionContext::new();
        
        // 设置变量
        context.set_variable("x".to_string(), Value::Int(42));
        assert!(context.has_variable("x"));
        
        // 清除所有变量
        context.clear();
        assert!(!context.has_variable("x"));
        assert!(context.variable_names().is_empty());
    }

    #[test]
    fn test_simple_expression_evaluator_new() {
        let evaluator = SimpleExpressionEvaluator::new();
        assert!(evaluator.context().variable_names().is_empty());
    }

    #[test]
    fn test_simple_expression_evaluator_with_context() {
        let mut context = SimpleExpressionContext::new();
        context.set_variable("x".to_string(), Value::Int(42));
        
        let evaluator = SimpleExpressionEvaluator::with_context(context);
        assert_eq!(evaluator.context().variable_names().len(), 1);
        assert!(evaluator.context().has_variable("x"));
    }

    #[test]
    fn test_simple_expression_evaluator_set_variable() {
        let mut evaluator = SimpleExpressionEvaluator::new();
        
        // 设置变量
        evaluator.set_variable("x".to_string(), Value::Int(42));
        
        // 验证变量已设置
        assert!(evaluator.context().has_variable("x"));
        assert_eq!(evaluator.context().get_variable_direct("x").unwrap(), Value::Int(42));
    }

    #[test]
    fn test_simple_expression_evaluator_evaluate_constant() {
        let evaluator = SimpleExpressionEvaluator::new();
        let expr = Expression::Constant(Value::Int(42));
        
        let result = evaluator.evaluate(&expr).unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_simple_expression_evaluator_evaluate_variable() {
        let mut evaluator = SimpleExpressionEvaluator::new();
        evaluator.set_variable("x".to_string(), Value::Int(42));
        
        let expr = Expression::Variable {
            name: "x".to_string(),
        };
        
        let result = evaluator.evaluate(&expr).unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_simple_expression_evaluator_evaluate_unary() {
        let evaluator = SimpleExpressionEvaluator::new();
        let expr = Expression::Unary {
            op: UnaryOp::Minus,
            operand: Box::new(Expression::Constant(Value::Int(42))),
        };
        
        let result = evaluator.evaluate(&expr).unwrap();
        assert_eq!(result, Value::Int(-42));
    }

    #[test]
    fn test_simple_expression_evaluator_evaluate_binary() {
        let evaluator = SimpleExpressionEvaluator::new();
        let expr = Expression::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expression::Constant(Value::Int(10))),
            right: Box::new(Expression::Constant(Value::Int(32))),
        };
        
        let result = evaluator.evaluate(&expr).unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_simple_expression_evaluator_evaluate_complex() {
        let mut evaluator = SimpleExpressionEvaluator::new();
        evaluator.set_variable("x".to_string(), Value::Int(10));
        evaluator.set_variable("y".to_string(), Value::Int(5));
        
        // 表达式: (x + y) * 2
        let expr = Expression::Binary {
            op: BinaryOp::Mul,
            left: Box::new(Expression::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expression::Variable { name: "x".to_string() }),
                right: Box::new(Expression::Variable { name: "y".to_string() }),
            }),
            right: Box::new(Expression::Constant(Value::Int(2))),
        };
        
        let result = evaluator.evaluate(&expr).unwrap();
        assert_eq!(result, Value::Int(30)); // (10 + 5) * 2 = 30
    }

    #[test]
    fn test_simple_expression_evaluator_context_mut() {
        let mut evaluator = SimpleExpressionEvaluator::new();
        
        // 通过可变引用设置变量
        evaluator.context_mut().set_variable("x".to_string(), Value::Int(42));
        
        // 验证变量已设置
        assert!(evaluator.context().has_variable("x"));
        assert_eq!(evaluator.context().get_variable_direct("x").unwrap(), Value::Int(42));
    }

    #[test]
    fn test_simple_expression_context_default() {
        let context = SimpleExpressionContext::default();
        assert!(context.variable_names().is_empty());
    }

    #[test]
    fn test_simple_expression_evaluator_default() {
        let evaluator = SimpleExpressionEvaluator::default();
        assert!(evaluator.context().variable_names().is_empty());
    }

    #[test]
    fn test_simple_expression_context_clone() {
        let mut context = SimpleExpressionContext::new();
        context.set_variable("x".to_string(), Value::Int(42));
        
        let cloned = context.clone();
        assert!(cloned.has_variable("x"));
        assert_eq!(cloned.get_variable_direct("x").unwrap(), Value::Int(42));
        
        // 修改原始上下文不应影响克隆
        context.set_variable("y".to_string(), Value::Int(100));
        assert!(!cloned.has_variable("y"));
    }

    #[test]
    fn test_simple_expression_context_graph_operations_not_supported() {
        use crate::expressions::ExpressionContext;
        
        let context = SimpleExpressionContext::new();
        
        // 所有图数据库特定操作都应该返回错误
        assert!(context.get_tag_property("person", "name").is_err());
        assert!(context.get_edge_property("friend", "since").is_err());
        assert!(context.get_src_vertex().is_err());
        assert!(context.get_dst_vertex().is_err());
        assert!(context.get_current_vertex().is_err());
        assert!(context.get_current_edge().is_err());
    }
}