//! AST v2 使用示例
//!
//! 展示如何使用新的简化 AST 设计来构建和操作查询。

use super::*;
use crate::core::Value;

/// 示例：构建简单的 MATCH 查询
pub fn build_simple_match_query() -> Stmt {
    let span = Span::default();

    // 创建节点模式: (n:Person)
    let node_pattern =
        PatternFactory::simple_node(Some("n".to_string()), vec!["Person".to_string()], span);

    // 创建返回子句: RETURN n.name
    let return_item = ReturnItem::Expression {
        expr: Expr::PropertyAccess(PropertyAccessExpr::new(
            Expr::Variable(VariableExpr::new("n".to_string(), span)),
            "name".o_string(),
            span,
        )),
        alias: None,
    };

    let return_clause = ReturnClause {
        span,
        items: vec![return_item],
        distinct: false,
    };

    // 创建 MATCH 语句
    StmtFactory::match_stmt(
        vec![node_pattern],
        None, // 没有 WHERE 子句
        Some(return_clause),
        None, // 没有 ORDER BY
        None, // 没有 LIMIT
        None, // 没有 SKIP
        span,
    )
}

/// 示例：构建带条件的 MATCH 查询
pub fn build_conditional_match_query() -> Stmt {
    let span = Span::default();

    // 创建节点模式: (n:Person {age: 25})
    let properties = Expr::Map(MapExpr::new(
        vec![(
            "age".to_string(),
            Expr::Constant(ConstantExpr::new(Value::Int(25), span)),
        )],
        span,
    ));

    let node_pattern = PatternFactory::node(
        Some("n".to_string()),
        vec!["Person".to_string()],
        Some(properties),
        vec![],
        span,
    );

    // 创建 WHERE 子句: WHERE n.name = "John"
    let where_clause = Expr::Binary(BinaryExpr::new(
        Expr::PropertyAccess(PropertyAccessExpr::new(
            Expr::Variable(VariableExpr::new("n".to_string(), span)),
            "name".to_string(),
            span,
        )),
        BinaryOp::Equal,
        Expr::Constant(ConstantExpr::new(Value::String("John".to_string()), span)),
        span,
    ));

    // 创建返回子句: RETURN n
    let return_item = ReturnItem::Expression {
        expr: Expr::Variable(VariableExpr::new("n".to_string(), span)),
        alias: None,
    };

    let return_clause = ReturnClause {
        span,
        items: vec![return_item],
        distinct: false,
    };

    // 创建 MATCH 语句
    StmtFactory::match_stmt(
        vec![node_pattern],
        Some(where_clause),
        Some(return_clause),
        None,
        None,
        None,
        span,
    )
}

/// 示例：构建 CREATE 节点查询
pub fn build_create_node_query() -> Stmt {
    let span = Span::default();

    // 创建属性映射
    let properties = Expr::Map(MapExpr::new(
        vec![
            (
                "name".to_string(),
                Expr::Constant(ConstantExpr::new(Value::String("Alice".to_string()), span)),
            ),
            (
                "age".to_string(),
                Expr::Constant(ConstantExpr::new(Value::Int(30), span)),
            ),
        ],
        span,
    ));

    // 创建 CREATE 语句
    StmtFactory::create_node(
        Some("person".to_string()),
        vec!["Person".to_string()],
        Some(properties),
        span,
    )
}

/// 示例：构建 CREATE 边查询
pub fn build_create_edge_query() -> Stmt {
    let span = Span::default();

    // 创建源节点和目标节点表达式
    let src = Expr::Variable(VariableExpr::new("alice".to_string(), span));
    let dst = Expr::Variable(VariableExpr::new("bob".to_string(), span));

    // 创建属性映射
    let properties = Expr::Map(MapExpr::new(
        vec![(
            "since".to_string(),
            Expr::Constant(ConstantExpr::new(Value::Int(2020), span)),
        )],
        span,
    ));

    // 创建 CREATE 边语句
    StmtFactory::create_edge(
        Some("friendship".to_string()),
        "KNOWS".to_string(),
        src,
        dst,
        Some(properties),
        EdgeDirection::Outgoing,
        span,
    )
}

/// 示例：构建 DELETE 查询
pub fn build_delete_query() -> Stmt {
    let span = Span::default();

    // 创建要删除的节点表达式
    let vertices = vec![
        Expr::Variable(VariableExpr::new("n1".to_string(), span)),
        Expr::Variable(VariableExpr::new("n2".to_string(), span)),
    ];

    // 创建 DELETE 语句
    StmtFactory::delete(DeleteTarget::Vertices(vertices), None, span)
}

/// 示例：构建 UPDATE 查询
pub fn build_update_query() -> Stmt {
    let span = Span::default();

    // 创建要更新的节点表达式
    let vertex = Expr::Variable(VariableExpr::new("person".to_string(), span));

    // 创建赋值操作
    let assignments = vec![Assignment {
        property: "age".to_string(),
        value: Expr::Constant(ConstantExpr::new(Value::Int(31), span)),
    }];

    let set_clause = SetClause { span, assignments };

    // 创建 UPDATE 语句
    StmtFactory::update(UpdateTarget::Vertex(vertex), set_clause, None, span)
}

/// 示例：构建 GO 查询
pub fn build_go_query() -> Stmt {
    let span = Span::default();

    // 创建 FROM 子句
    let from = FromClause {
        span,
        vertices: vec![Expr::Variable(VariableExpr::new("start".to_string(), span))],
    };

    // 创建 OVER 子句
    let over = OverClause {
        span,
        edge_types: vec!["KNOWS".to_string()],
        direction: EdgeDirection::Outgoing,
    };

    // 创建 YIELD 子句
    let yield_item = YieldItem {
        expr: Expr::Variable(VariableExpr::new("$$.dst".to_string(), span)),
        alias: Some("dst".to_string()),
    };

    let yield_clause = YieldClause {
        span,
        items: vec![yield_item],
    };

    // 创建 GO 语句
    StmtFactory::go(
        Steps::Fixed(1),
        from,
        Some(over),
        None, // 没有 WHERE 子句
        Some(yield_clause),
        span,
    )
}

/// 示例：构建 LOOKUP 查询
pub fn build_lookup_query() -> Stmt {
    let span = Span::default();

    // 创建 WHERE 子句
    let where_clause = Expr::Binary(BinaryExpr::new(
        Expr::PropertyAccess(PropertyAccessExpr::new(
            Expr::Variable(VariableExpr::new("vertex".to_string(), span)),
            "name".to_string(),
            span,
        )),
        BinaryOp::Equal,
        Expr::Constant(ConstantExpr::new(Value::String("John".to_string()), span)),
        span,
    ));

    // 创建 YIELD 子句
    let yield_item = YieldItem {
        expr: Expr::Variable(VariableExpr::new("vertex".to_string(), span)),
        alias: None,
    };

    let yield_clause = YieldClause {
        span,
        items: vec![yield_item],
    };

    // 创建 LOOKUP 语句
    StmtFactory::lookup(
        LookupTarget::Tag("Person".to_string()),
        Some(where_clause),
        Some(yield_clause),
        span,
    )
}

/// 示例：构建 SUBGRAPH 查询
pub fn build_subgraph_query() -> Stmt {
    let span = Span::default();

    // 创建 FROM 子句
    let from = FromClause {
        span,
        vertices: vec![Expr::Variable(VariableExpr::new("start".to_string(), span))],
    };

    // 创建 OVER 子句
    let over = OverClause {
        span,
        edge_types: vec!["KNOWS".to_string(), "FOLLOWS".to_string()],
        direction: EdgeDirection::Both,
    };

    // 创建 YIELD 子句
    let yield_item = YieldItem {
        expr: Expr::Variable(VariableExpr::new("vertices".to_string(), span)),
        alias: None,
    };

    let yield_clause = YieldClause {
        span,
        items: vec![yield_item],
    };

    // 创建 SUBGRAPH 语句
    StmtFactory::subgraph(
        Steps::Range { min: 1, max: 3 },
        from,
        Some(over),
        None, // 没有 WHERE 子句
        Some(yield_clause),
        span,
    )
}

/// 示例：构建 FIND PATH 查询
pub fn build_find_path_query() -> Stmt {
    let span = Span::default();

    // 创建 FROM 子句
    let from = FromClause {
        span,
        vertices: vec![Expr::Variable(VariableExpr::new("start".to_string(), span))],
    };

    // 创建目标表达式
    let to = Expr::Variable(VariableExpr::new("end".to_string(), span));

    // 创建 OVER 子句
    let over = OverClause {
        span,
        edge_types: vec!["KNOWS".to_string()],
        direction: EdgeDirection::Outgoing,
    };

    // 创建 YIELD 子句
    let yield_item = YieldItem {
        expr: Expr::Variable(VariableExpr::new("path".to_string(), span)),
        alias: None,
    };

    let yield_clause = YieldClause {
        span,
        items: vec![yield_item],
    };

    // 创建 FIND PATH 语句
    StmtFactory::find_path(
        from,
        to,
        Some(over),
        None, // 没有 WHERE 子句
        true, // 最短路径
        Some(yield_clause),
        span,
    )
}

/// 示例：使用访问者模式遍历 AST
pub fn demonstrate_visitor_pattern() {
    let span = Span::default();

    // 创建一个复杂的表达式： (x + 5) * 2
    let expr = Expr::Binary(BinaryExpr::new(
        Expr::Binary(BinaryExpr::new(
            Expr::Variable(VariableExpr::new("x".to_string(), span)),
            BinaryOp::Add,
            Expr::Constant(ConstantExpr::new(Value::Int(5), span)),
            span,
        )),
        BinaryOp::Multiply,
        Expr::Constant(ConstantExpr::new(Value::Int(2), span)),
        span,
    ));

    // 使用默认访问者
    let mut visitor = DefaultVisitor;
    visitor.visit_expr(&expr);

    // 使用类型检查访问者
    let mut type_checker = TypeChecker::new();
    type_checker.visit_expr(&expr);

    println!("Type checker errors: {:?}", type_checker.errors);
    println!("Type checker warnings: {:?}", type_checker.warnings);

    // 使用 AST 格式化器
    let mut formatter = AstFormatter::new();
    let formatted = formatter.format(&expr);
    println!("Formatted AST:\n{}", formatted);
}

/// 示例：使用表达式优化器
pub fn demonstrate_expression_optimization() {
    let span = Span::default();

    // 创建一个可以优化的表达式： (5 + 3) * 1 + 0
    let expr = Expr::Binary(BinaryExpr::new(
        Expr::Binary(BinaryExpr::new(
            Expr::Binary(BinaryExpr::new(
                Expr::Constant(ConstantExpr::new(Value::Int(5), span)),
                BinaryOp::Add,
                Expr::Constant(ConstantExpr::new(Value::Int(3), span)),
                span,
            )),
            BinaryOp::Multiply,
            Expr::Constant(ConstantExpr::new(Value::Int(1), span)),
            span,
        )),
        BinaryOp::Add,
        Expr::Constant(ConstantExpr::new(Value::Int(0), span)),
        span,
    ));

    println!("Original expression: {}", expr.to_string());

    // 应用常量折叠
    let folded = ExprOptimizer::constant_folding(expr.clone());
    println!("After constant folding: {}", folded.to_string());

    // 应用完整优化
    let optimized = ExprOptimizer::simplify(expr);
    println!("After full optimization: {}", optimized.to_string());
}

/// 示例：构建复杂的图模式
pub fn build_complex_pattern() -> Pattern {
    let span = Span::default();

    // 创建节点模式: (a:Person)
    let node_a =
        PatternFactory::simple_node(Some("a".to_string()), vec!["Person".to_string()], span);

    // 创建边模式: -[e:KNOWS]->
    let edge_e = PatternFactory::simple_edge(
        Some("e".to_string()),
        vec!["KNOWS".to_string()],
        EdgeDirection::Outgoing,
        span,
    );

    // 创建节点模式: (b:Person {age: 25})
    let properties_b = Expr::Map(MapExpr::new(
        vec![(
            "age".to_string(),
            Expr::Constant(ConstantExpr::new(Value::Int(25), span)),
        )],
        span,
    ));

    let node_b = PatternFactory::node(
        Some("b".to_string()),
        vec!["Person".to_string()],
        Some(properties_b),
        vec![],
        span,
    );

    // 创建路径模式: (a)-[e]->(b)
    let elements = vec![
        PathElement::Node(node_a),
        PathElement::Edge(edge_e),
        PathElement::Node(node_b),
    ];

    PatternFactory::path(elements, span)
}

/// 示例：使用 AST 构建器
pub fn demonstrate_ast_builder() {
    let span = Span::default();
    let builder = AstBuilder::new(span);

    // 构建简单的 MATCH 查询
    let pattern =
        PatternFactory::simple_node(Some("n".to_string()), vec!["Person".to_string()], span);

    let return_expr = Expr::PropertyAccess(PropertyAccessExpr::new(
        Expr::Variable(VariableExpr::new("n".to_string(), span)),
        "name".to_string(),
        span,
    ));

    let stmt = builder.build_simple_match(pattern, return_expr);
    println!("Built MATCH query: {:?}", stmt);

    // 构建 CREATE 节点查询
    let create_stmt =
        builder.build_create_node(Some("person".to_string()), vec!["Person".to_string()]);
    println!("Built CREATE query: {:?}", create_stmt);
}

#[cfg(test)]
mod examples_tests {
    use super::*;

    #[test]
    fn test_build_simple_match_query() {
        let stmt = build_simple_match_query();
        assert!(matches!(stmt, Stmt::Match(_)));
    }

    #[test]
    fn test_build_conditional_match_query() {
        let stmt = build_conditional_match_query();
        assert!(matches!(stmt, Stmt::Match(_)));
    }

    #[test]
    fn test_build_create_node_query() {
        let stmt = build_create_node_query();
        assert!(matches!(stmt, Stmt::Create(_)));
    }

    #[test]
    fn test_build_create_edge_query() {
        let stmt = build_create_edge_query();
        assert!(matches!(stmt, Stmt::Create(_)));
    }

    #[test]
    fn test_build_delete_query() {
        let stmt = build_delete_query();
        assert!(matches!(stmt, Stmt::Delete(_)));
    }

    #[test]
    fn test_build_update_query() {
        let stmt = build_update_query();
        assert!(matches!(stmt, Stmt::Update(_)));
    }

    #[test]
    fn test_build_go_query() {
        let stmt = build_go_query();
        assert!(matches!(stmt, Stmt::Go(_)));
    }

    #[test]
    fn test_build_lookup_query() {
        let stmt = build_lookup_query();
        assert!(matches!(stmt, Stmt::Lookup(_)));
    }

    #[test]
    fn test_build_subgraph_query() {
        let stmt = build_subgraph_query();
        assert!(matches!(stmt, Stmt::Subgraph(_)));
    }

    #[test]
    fn test_build_find_path_query() {
        let stmt = build_find_path_query();
        assert!(matches!(stmt, Stmt::FindPath(_)));
    }

    #[test]
    fn test_build_complex_pattern() {
        let pattern = build_complex_pattern();
        assert!(matches!(pattern, Pattern::Path(_)));

        let vars = PatternUtils::find_variables(&pattern);
        assert!(vars.contains(&"a".to_string()));
        assert!(vars.contains(&"e".to_string()));
        assert!(vars.contains(&"b".to_string()));
    }

    #[test]
    fn test_demonstrate_ast_builder() {
        // 这个测试主要是确保代码可以编译和运行
        demonstrate_ast_builder();
    }

    #[test]
    fn test_demonstrate_visitor_pattern() {
        // 这个测试主要是确保代码可以编译和运行
        demonstrate_visitor_pattern();
    }

    #[test]
    fn test_demonstrate_expression_optimization() {
        // 这个测试主要是确保代码可以编译和运行
        demonstrate_expression_optimization();
    }
}
