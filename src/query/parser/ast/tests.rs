//! AST v2 测试模块
#[cfg(test)]
use super::*;
use crate::core::Value;
use crate::query::parser::core::position::Position;

mod expr_tests {
    use super::*;

    #[test]
    fn test_constant_expression() {
        let expression = Expression::Constant(ConstantExpression::new(Value::Int(42), Span::default()));
        assert!(expression.is_constant());
        assert_eq!(expression.to_string(), "Int(42)");
    }

    #[test]
    fn test_variable_expression() {
        let expression = Expression::Variable(VariableExpression::new("x".to_string(), Span::default()));
        assert!(!expression.is_constant());
        assert_eq!(expression.to_string(), "x");
    }

    #[test]
    fn test_binary_expression() {
        let left = Expression::Constant(ConstantExpression::new(Value::Int(5), Span::default()));
        let right = Expression::Constant(ConstantExpression::new(Value::Int(3), Span::default()));
        let expression = Expression::Binary(BinaryExpression::new(left, BinaryOp::Add, right, Span::default()));

        assert!(expression.is_constant());
        assert_eq!(expression.to_string(), "(Int(5) + Int(3))");
    }

    #[test]
    fn test_function_call_expression() {
        let args = vec![
            Expression::Variable(VariableExpression::new("x".to_string(), Span::default())),
            Expression::Variable(VariableExpression::new("y".to_string(), Span::default())),
        ];
        let expression = Expression::FunctionCall(FunctionCallExpression::new(
            "SUM".to_string(),
            args,
            false,
            Span::default(),
        ));

        assert!(!expression.is_constant());
        assert_eq!(expression.to_string(), "SUM(x, y)");
    }

    #[test]
    fn test_property_access_expression() {
        let object = Expression::Variable(VariableExpression::new("node".to_string(), Span::default()));
        let expression = Expression::PropertyAccess(PropertyAccessExpression::new(
            object,
            "name".to_string(),
            Span::default(),
        ));

        assert!(!expression.is_constant());
        assert_eq!(expression.to_string(), "node.name");
    }

    #[test]
    fn test_list_expression() {
        let elements = vec![
            Expression::Constant(ConstantExpression::new(Value::Int(1), Span::default())),
            Expression::Constant(ConstantExpression::new(Value::Int(2), Span::default())),
            Expression::Constant(ConstantExpression::new(Value::Int(3), Span::default())),
        ];
        let expression = Expression::List(ListExpression::new(elements, Span::default()));

        assert!(expression.is_constant());
        assert_eq!(expression.to_string(), "[Int(1), Int(2), Int(3)]");
    }

    #[test]
    fn test_map_expression() {
        let pairs = vec![
            (
                "name".to_string(),
                Expression::Constant(ConstantExpression::new(
                    Value::String("John".to_string()),
                    Span::default(),
                )),
            ),
            (
                "age".to_string(),
                Expression::Constant(ConstantExpression::new(Value::Int(30), Span::default())),
            ),
        ];
        let expression = Expression::Map(MapExpression::new(pairs, Span::default()));

        assert!(expression.is_constant());
        assert_eq!(expression.to_string(), "{name: String(\"John\"), age: Int(30)}");
    }

    #[test]
    fn test_case_expression() {
        let match_expression = Some(Expression::Variable(VariableExpression::new(
            "score".to_string(),
            Span::default(),
        )));
        let when_then_pairs = vec![
            (
                Expression::Constant(ConstantExpression::new(Value::Int(90), Span::default())),
                Expression::Constant(ConstantExpression::new(
                    Value::String("A".to_string()),
                    Span::default(),
                )),
            ),
            (
                Expression::Constant(ConstantExpression::new(Value::Int(80), Span::default())),
                Expression::Constant(ConstantExpression::new(
                    Value::String("B".to_string()),
                    Span::default(),
                )),
            ),
        ];
        let default = Some(Expression::Constant(ConstantExpression::new(
            Value::String("F".to_string()),
            Span::default(),
        )));

        let expression = Expression::Case(CaseExpression::new(
            match_expression,
            when_then_pairs,
            default,
            Span::default(),
        ));

        assert!(!expression.is_constant());
        assert!(expression.to_string().contains("CASE score"));
        assert!(expression.to_string().contains("WHEN Int(90) THEN String(\"A\")"));
        assert!(expression.to_string().contains("ELSE String(\"F\")"));
    }

    #[test]
    fn test_subscript_expression() {
        let collection = Expression::Variable(VariableExpression::new("array".to_string(), Span::default()));
        let index = Expression::Constant(ConstantExpression::new(Value::Int(0), Span::default()));
        let expression = Expression::Subscript(SubscriptExpression::new(collection, index, Span::default()));

        assert!(!expression.is_constant());
        assert_eq!(expression.to_string(), "array[Int(0)]");
    }
}

#[cfg(test)]
mod stmt_tests {
    use super::*;

    #[test]
    fn test_create_node_stmt() {
        let stmt = Stmt::Create(CreateStmt {
            span: Span::default(),
            target: CreateTarget::Node {
                variable: Some("n".to_string()),
                labels: vec!["Person".to_string()],
                properties: None,
            },
        });

        assert!(matches!(stmt, Stmt::Create(_)));
    }

    #[test]
    fn test_match_stmt() {
        let stmt = Stmt::Match(MatchStmt {
            span: Span::default(),
            patterns: vec![],
            where_clause: None,
            return_clause: None,
            order_by: None,
            limit: None,
            skip: None,
        });

        assert!(matches!(stmt, Stmt::Match(_)));
    }

    #[test]
    fn test_lookup_stmt() {
        let stmt = Stmt::Lookup(LookupStmt {
            span: Span::default(),
            target: LookupTarget::Tag("Person".to_string()),
            where_clause: None,
            yield_clause: None,
        });

        assert!(matches!(stmt, Stmt::Lookup(_)));
    }

    #[test]
    fn test_subgraph_stmt() {
        let stmt = Stmt::Subgraph(SubgraphStmt {
            span: Span::default(),
            steps: Steps::Fixed(1),
            from: FromClause {
                span: Span::default(),
                vertices: vec![],
            },
            over: None,
            where_clause: None,
            yield_clause: None,
        });

        assert!(matches!(stmt, Stmt::Subgraph(_)));
    }

    #[test]
    fn test_find_path_stmt() {
        let stmt = Stmt::FindPath(FindPathStmt {
            span: Span::default(),
            from: FromClause {
                span: Span::default(),
                vertices: vec![],
            },
            to: Expression::Variable(VariableExpression::new("target".to_string(), Span::default())),
            over: None,
            where_clause: None,
            shortest: true,
            yield_clause: None,
        });

        assert!(matches!(stmt, Stmt::FindPath(_)));
    }
}

#[cfg(test)]
mod pattern_tests {
    use super::*;

    #[test]
    fn test_node_pattern() {
        let pattern = Pattern::Node(NodePattern::new(
            Some("n".to_string()),
            vec!["Person".to_string()],
            None,
            vec![],
            Span::default(),
        ));

        assert!(matches!(pattern, Pattern::Node(_)));
        let vars = PatternUtils::find_variables(&pattern);
        assert_eq!(vars, vec!["n"]);
    }

    #[test]
    fn test_edge_pattern() {
        let pattern = Pattern::Edge(EdgePattern::new(
            Some("e".to_string()),
            vec!["KNOWS".to_string()],
            None,
            vec![],
            EdgeDirection::Out,
            None,
            Span::default(),
        ));

        assert!(matches!(pattern, Pattern::Edge(_)));
        let vars = PatternUtils::find_variables(&pattern);
        assert_eq!(vars, vec!["e"]);
    }

    #[test]
    fn test_path_pattern() {
        let elements = vec![
            PathElement::Node(NodePattern::new(
                Some("a".to_string()),
                vec![],
                None,
                vec![],
                Span::default(),
            )),
            PathElement::Edge(EdgePattern::new(
                Some("e".to_string()),
                vec![],
                None,
                vec![],
                EdgeDirection::Out,
                None,
                Span::default(),
            )),
            PathElement::Node(NodePattern::new(
                Some("b".to_string()),
                vec![],
                None,
                vec![],
                Span::default(),
            )),
        ];

        let pattern = Pattern::Path(PathPattern::new(elements, Span::default()));
        let vars = PatternUtils::find_variables(&pattern);
        assert_eq!(vars, vec!["a", "e", "b"]);
    }

    #[test]
    fn test_edge_range() {
        let range1 = EdgeRange::fixed(2);
        assert_eq!(range1.min, Some(2));
        assert_eq!(range1.max, Some(2));

        let range2 = EdgeRange::range(1, 3);
        assert_eq!(range2.min, Some(1));
        assert_eq!(range2.max, Some(3));

        let range3 = EdgeRange::at_least(1);
        assert_eq!(range3.min, Some(1));
        assert_eq!(range3.max, None);

        let range4 = EdgeRange::any();
        assert_eq!(range4.min, None);
        assert_eq!(range4.max, None);
    }
}

#[cfg(test)]
mod visitor_tests {
    use super::*;
    use crate::core::Value;

    #[test]
    fn test_default_visitor() {
        let mut visitor = DefaultVisitor;
        let expression = Expression::Constant(ConstantExpression::new(Value::Int(42), Span::default()));

        // 应该能够访问而不出错
        visitor.visit_expression(&expression);
    }

    #[test]
    fn test_type_checker() {
        let mut checker = TypeChecker::new();
        let left = Expression::Constant(ConstantExpression::new(Value::Int(5), Span::default()));
        let right = Expression::Constant(ConstantExpression::new(
            Value::String("hello".to_string()),
            Span::default(),
        ));
        let expression = Expression::Binary(BinaryExpression::new(left, BinaryOp::Add, right, Span::default()));

        checker.visit_expression(&expression);
        assert!(checker.has_warnings());
    }

    #[test]
    fn test_ast_formatter() {
        let mut formatter = AstFormatter::new();
        let expression = Expression::Constant(ConstantExpression::new(Value::Int(42), Span::default()));

        let result = formatter.format(&expression);
        assert!(result.contains("Constant: Int(42)"));
    }
}

#[cfg(test)]
mod utils_tests {
    use super::*;
    use crate::core::Value;

    #[test]
    fn test_expr_factory() {
        let span = Span::default();

        // 测试常量表达式
        let const_expression = ExprFactory::constant(Value::Int(42), span);
        assert!(matches!(const_expression, Expression::Constant(_)));

        // 测试变量表达式
        let var_expression = ExprFactory::variable("x".to_string(), span);
        assert!(matches!(var_expression, Expression::Variable(_)));

        // 测试二元表达式
        let left = ExprFactory::constant(Value::Int(5), span);
        let right = ExprFactory::constant(Value::Int(3), span);
        let binary_expression = ExprFactory::binary(left, BinaryOp::Add, right, span);
        assert!(matches!(binary_expression, Expression::Binary(_)));
    }

    #[test]
    fn test_constant_folding() {
        let span = Span::default();

        // 测试 5 + 3 -> 8
        let left = ExprFactory::constant(Value::Int(5), span);
        let right = ExprFactory::constant(Value::Int(3), span);
        let expression = ExprFactory::binary(left, BinaryOp::Add, right, span);

        let optimized = ExprOptimizer::constant_folding(expression);
        assert!(matches!(optimized, Expression::Constant(_)));
        if let Expression::Constant(e) = optimized {
            assert_eq!(e.value, Value::Int(8));
        }

        // 测试 -5 -> -5
        let operand = ExprFactory::constant(Value::Int(5), span);
        let expression = ExprFactory::unary(UnaryOp::Minus, operand, span);

        let optimized = ExprOptimizer::constant_folding(expression);
        assert!(matches!(optimized, Expression::Constant(_)));
        if let Expression::Constant(e) = optimized {
            assert_eq!(e.value, Value::Int(-5));
        }
    }

    #[test]
    fn test_expression_simplification() {
        let span = Span::default();

        // 测试 x + 0 -> x
        let x = ExprFactory::variable("x".to_string(), span);
        let zero = ExprFactory::constant(Value::Int(0), span);
        let expression = ExprFactory::binary(x.clone(), BinaryOp::Add, zero, span);

        let simplified = ExprOptimizer::simplify(expression);
        assert_eq!(simplified, x);

        // 测试 x * 1 -> x
        let x = ExprFactory::variable("x".to_string(), span);
        let one = ExprFactory::constant(Value::Int(1), span);
        let expression = ExprFactory::binary(x.clone(), BinaryOp::Multiply, one, span);

        let simplified = ExprOptimizer::simplify(expression);
        assert_eq!(simplified, x);

        // 测试 !!x -> x
        let x = ExprFactory::variable("x".to_string(), span);
        let not_expression = ExprFactory::unary(UnaryOp::Not, x.clone(), span);
        let expression = ExprFactory::unary(UnaryOp::Not, not_expression, span);

        let simplified = ExprOptimizer::simplify(expression);
        assert_eq!(simplified, x);
    }
}

#[cfg(test)]
mod error_tests {
    use super::*;
    use crate::query::parser::{ParseError, ParseErrors};
    use crate::query::parser::core::ParseErrorKind;

    #[test]
    fn test_parse_error_kind() {
        assert_eq!(ParseErrorKind::SyntaxError, ParseErrorKind::SyntaxError);
        assert_eq!(ParseErrorKind::UnexpectedToken, ParseErrorKind::UnexpectedToken);
        assert_ne!(ParseErrorKind::SyntaxError, ParseErrorKind::UnexpectedToken);
    }

    #[test]
    fn test_parse_error_new() {
        let position = Position::new(10, 5);
        let error = ParseError::new(
            ParseErrorKind::UnexpectedToken,
            "Unexpected token".to_string(),
            position,
        );

        assert_eq!(error.kind, ParseErrorKind::UnexpectedToken);
        assert_eq!(error.message, "Unexpected token");
        assert_eq!(error.position.line, 10);
        assert_eq!(error.position.column, 5);
        assert!(error.offset.is_none());
        assert!(error.expected_tokens.is_empty());
    }

    #[test]
    fn test_parse_error_with_context() {
        let position = Position::new(5, 10);
        let context_error = std::io::Error::new(std::io::ErrorKind::InvalidData, "In CREATE statement");
        let error = ParseError::new(
            ParseErrorKind::SyntaxError,
            "Invalid expression".to_string(),
            position,
        )
        .with_context(context_error)
        .with_offset(100)
        .with_expected_tokens(vec!["CREATE".to_string(), "MATCH".to_string()]);

        assert!(error.context.is_some());
        assert_eq!(error.offset, Some(100));
        assert_eq!(error.expected_tokens.len(), 2);
    }

    #[test]
    fn test_parse_error_unexpected_token() {
        let position = Position::new(1, 1);
        let error = ParseError::unexpected_token("IDENTIFIER", position);

        assert_eq!(error.kind, ParseErrorKind::UnexpectedToken);
        assert!(error.message.contains("Unexpected token"));
        assert!(error.message.contains("IDENTIFIER"));
    }

    #[test]
    fn test_parse_error_unterminated_string() {
        let position = Position::new(5, 10);
        let error = ParseError::unterminated_string(position);

        assert_eq!(error.kind, ParseErrorKind::UnterminatedString);
        assert!(error.message.contains("Unterminated string"));
    }

    #[test]
    fn test_parse_error_unterminated_comment() {
        let position = Position::new(5, 10);
        let error = ParseError::unterminated_comment(position);

        assert_eq!(error.kind, ParseErrorKind::UnterminatedComment);
        assert!(error.message.contains("Unterminated multi-line comment"));
    }

    #[test]
    fn test_parse_error_display() {
        let position = Position::new(10, 5);
        let error = ParseError::new(
            ParseErrorKind::UnexpectedToken,
            "Expected ')'".to_string(),
            position,
        )
        .with_unexpected_token("}'")
        .with_expected_tokens(vec![")".to_string(), "]".to_string()]);

        let display = format!("{}", error);
        assert!(display.contains("line 10, column 5"));
        assert!(display.contains("Expected ')'"));
        assert!(display.contains("Unexpected token: }'"));
        assert!(display.contains("Expected one of: ), ]"));
    }

    #[test]
    fn test_parse_errors_collection() {
        let mut errors = ParseErrors::new();
        assert!(errors.is_empty());
        assert_eq!(errors.len(), 0);

        let pos1 = Position::new(1, 1);
        errors.add(ParseError::new(
            ParseErrorKind::SyntaxError,
            "Error 1".to_string(),
            pos1,
        ));
        let pos2 = Position::new(2, 2);
        errors.add(ParseError::new(
            ParseErrorKind::UnexpectedToken,
            "Error 2".to_string(),
            pos2,
        ));

        assert!(!errors.is_empty());
        assert_eq!(errors.len(), 2);
    }

    #[test]
    fn test_parse_error_from_string() {
        let error: ParseError = "Simple error message".to_string().into();
        assert_eq!(error.kind, ParseErrorKind::SyntaxError);
        assert_eq!(error.message, "Simple error message");
        assert_eq!(error.position.line, 0);
        assert_eq!(error.position.column, 0);
    }
}
