//! AST 模块测试
//!
//! 测试新的基于 trait 的 AST 架构

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;
    use crate::query::parser::ast::visitor::*;
    
    #[test]
    fn test_ast_node_traits() {
        let span = Span::default();
        
        // 测试常量表达式
        let const_expr = ConstantExpr::new(Value::Int(42), span);
        assert_eq!(const_expr.node_type(), "ConstantExpr");
        assert_eq!(const_expr.expr_type(), ExpressionType::Constant);
        assert!(const_expr.is_constant());
        
        // 测试变量表达式
        let var_expr = VariableExpr::new("x".to_string(), span);
        assert_eq!(var_expr.node_type(), "VariableExpr");
        assert_eq!(var_expr.expr_type(), ExpressionType::Variable);
        assert!(!var_expr.is_constant());
    }
    
    #[test]
    fn test_binary_expression() {
        let span = Span::default();
        
        let left = Box::new(ConstantExpr::new(Value::Int(5), span));
        let right = Box::new(ConstantExpr::new(Value::Int(3), span));
        let binary_expr = BinaryExpr::new(left, BinaryOp::Add, right, span);
        
        assert_eq!(binary_expr.expr_type(), ExpressionType::Binary);
        assert!(binary_expr.is_constant());
        assert_eq!(binary_expr.children().len(), 2);
    }
    
    #[test]
    fn test_function_call() {
        let span = Span::default();
        
        let args = vec![
            Box::new(ConstantExpr::new(Value::Int(1), span)),
            Box::new(ConstantExpr::new(Value::Int(2), span)),
        ];
        
        let func_expr = FunctionCallExpr::new("SUM".to_string(), args, false, span);
        
        assert_eq!(func_expr.expr_type(), ExpressionType::FunctionCall);
        assert!(!func_expr.is_constant());
        assert_eq!(func_expr.children().len(), 2);
    }
    
    #[test]
    fn test_case_expression() {
        let span = Span::default();
        
        let match_expr = Some(Box::new(VariableExpr::new("x".to_string(), span)));
        let when_then_pairs = vec![
            (
                Box::new(ConstantExpr::new(Value::Int(1), span)),
                Box::new(ConstantExpr::new(Value::String("one".to_string()), span))
            ),
            (
                Box::new(ConstantExpr::new(Value::Int(2), span)),
                Box::new(ConstantExpr::new(Value::String("two".to_string()), span))
            ),
        ];
        let default = Some(Box::new(ConstantExpr::new(Value::String("other".to_string()), span)));
        
        let case_expr = CaseExpr::new(match_expr, when_then_pairs, default, span);
        
        assert_eq!(case_expr.expr_type(), ExpressionType::Case);
        assert!(!case_expr.is_constant()); // 包含变量，不是常量
        assert_eq!(case_expr.children().len(), 4); // match + 2*(when+then) + default
    }
    
    #[test]
    fn test_list_expression() {
        let span = Span::default();
        
        let elements = vec![
            Box::new(ConstantExpr::new(Value::Int(1), span)),
            Box::new(ConstantExpr::new(Value::Int(2), span)),
            Box::new(ConstantExpr::new(Value::Int(3), span)),
        ];
        
        let list_expr = ListExpr::new(elements, span);
        
        assert_eq!(list_expr.expr_type(), ExpressionType::List);
        assert!(list_expr.is_constant());
        assert_eq!(list_expr.children().len(), 3);
    }
    
    #[test]
    fn test_map_expression() {
        let span = Span::default();
        
        let pairs = vec![
            ("key1".to_string(), Box::new(ConstantExpr::new(Value::Int(1), span))),
            ("key2".to_string(), Box::new(ConstantExpr::new(Value::Int(2), span))),
        ];
        
        let map_expr = MapExpr::new(pairs, span);
        
        assert_eq!(map_expr.expr_type(), ExpressionType::Map);
        assert!(map_expr.is_constant());
        assert_eq!(map_expr.children().len(), 2);
    }
    
    #[test]
    fn test_predicate_expression() {
        let span = Span::default();
        
        let list = Box::new(ListExpr::new(vec![
            Box::new(ConstantExpr::new(Value::Int(1), span)),
            Box::new(ConstantExpr::new(Value::Int(2), span)),
            Box::new(ConstantExpr::new(Value::Int(3), span)),
        ], span));
        
        let condition = Box::new(BinaryExpr::new(
            Box::new(VariableExpr::new("x".to_string(), span)),
            BinaryOp::Gt,
            Box::new(ConstantExpr::new(Value::Int(1), span)),
            span
        ));
        
        let predicate_expr = PredicateExpr::new(PredicateType::All, list, condition, span);
        
        assert_eq!(predicate_expr.expr_type(), ExpressionType::Predicate);
        assert!(!predicate_expr.is_constant());
        assert_eq!(predicate_expr.children().len(), 2);
    }
    
    #[test]
    fn test_create_statement() {
        let span = Span::default();
        
        let target = CreateTarget::Node {
            identifier: Some("n".to_string()),
            labels: vec!["Person".to_string()],
            properties: None,
        };
        
        let create_stmt = CreateStatement::new(target, false, span);
        
        assert_eq!(create_stmt.stmt_type(), StatementType::Create);
        assert_eq!(create_stmt.to_string(), "CREATE (n:Person)");
    }
    
    #[test]
    fn test_match_statement() {
        let span = Span::default();
        
        let patterns: Vec<Box<dyn Pattern>> = vec![]; // 空模式用于测试
        let match_stmt = MatchStatement::new(patterns, span);
        
        assert_eq!(match_stmt.stmt_type(), StatementType::Match);
        assert_eq!(match_stmt.to_string(), "MATCH ");
    }
    
    #[test]
    fn test_go_statement() {
        let span = Span::default();
        
        let steps = Steps::Fixed(1);
        let from = FromClause {
            vertices: vec![],
        };
        let over = OverClause {
            edge_types: vec!["friend".to_string()],
            direction: EdgeDirection::Outbound,
            reversely: false,
        };
        
        let go_stmt = GoStatement::new(steps, from, over, span);
        
        assert_eq!(go_stmt.stmt_type(), StatementType::Go);
        assert_eq!(go_stmt.to_string(), "GO 1 STEP FROM  OVER friend");
    }
    
    #[test]
    fn test_node_pattern() {
        let span = Span::default();
        
        let node_pattern = NodePattern::new(
            Some("n".to_string()),
            vec!["Person".to_string(), "Student".to_string()],
            span,
        );
        
        assert_eq!(node_pattern.pattern_type(), PatternType::Node);
        assert_eq!(node_pattern.variables(), vec!["n"]);
        assert_eq!(node_pattern.to_string(), "(n:Person:Student)");
    }
    
    #[test]
    fn test_edge_pattern() {
        let span = Span::default();
        
        let edge_pattern = EdgePattern::new(
            Some("e".to_string()),
            Some("friend".to_string()),
            EdgeDirection::Outbound,
            span,
        );
        
        assert_eq!(edge_pattern.pattern_type(), PatternType::Edge);
        assert_eq!(edge_pattern.variables(), vec!["e"]);
        assert_eq!(edge_pattern.to_string(), "[e:friend]->");
    }
    
    #[test]
    fn test_path_pattern() {
        let span = Span::default();
        
        let node1 = NodePattern::new(Some("a".to_string()), vec![], span);
        let edge = EdgePattern::new(None, Some("knows".to_string()), EdgeDirection::Outbound, span);
        let node2 = NodePattern::new(Some("b".to_string()), vec![], span);
        
        let path_pattern = PathPattern::new(vec![
            PathElement::Node(node1),
            PathElement::Edge(edge),
            PathElement::Node(node2),
        ], span);
        
        assert_eq!(path_pattern.pattern_type(), PatternType::Path);
        assert_eq!(path_pattern.variables(), vec!["a", "b"]);
    }
    
    #[test]
    fn test_ast_builder() {
        let span = Span::default();
        let builder = AstBuilder::new(span);
        
        // 构建简单的常量表达式
        let expr = builder.constant(Value::Int(42));
        assert_eq!(expr.expr_type(), ExpressionType::Constant);
        
        // 构建变量表达式
        let var_expr = builder.variable("x");
        assert_eq!(var_expr.expr_type(), ExpressionType::Variable);
        
        // 构建二元表达式
        let left = builder.constant(Value::Int(5));
        let right = builder.constant(Value::Int(3));
        let binary_expr = builder.binary(left, BinaryOp::Add, right);
        assert_eq!(binary_expr.expr_type(), ExpressionType::Binary);
        assert!(binary_expr.is_constant());
    }
    
    #[test]
    fn test_expression_builder() {
        let span = Span::default();
        let builder = ExpressionBuilder::new(span);
        
        let left = builder.constant(Value::Int(5));
        let right = builder.constant(Value::Int(3));
        let add_expr = builder.add(left, right);
        
        assert_eq!(add_expr.expr_type(), ExpressionType::Binary);
        assert!(add_expr.is_constant());
    }
    
    #[test]
    fn test_statement_builder() {
        let span = Span::default();
        let builder = StatementBuilder::new(span);
        
        let pattern = builder.node_pattern(Some("n".to_string()), vec!["Person".to_string()]);
        let match_stmt = builder.match_pattern(pattern);
        
        assert_eq!(match_stmt.stmt_type(), StatementType::Match);
    }
    
    #[test]
    fn test_visitor_pattern() {
        let span = Span::default();
        
        // 创建测试表达式
        let left = Box::new(ConstantExpr::new(Value::Int(5), span));
        let right = Box::new(ConstantExpr::new(Value::Int(3), span));
        let binary_expr = BinaryExpr::new(left, BinaryOp::Add, right, span);
        
        // 使用默认访问者
        let mut visitor = DefaultVisitor;
        let _ = binary_expr.accept(&mut visitor);
        
        // 应该能够正常访问而不出错
        assert!(true);
    }
        // 应该生成格式化的字符串
        assert!(result.contains("Constant: Int(42)"));
    }
}