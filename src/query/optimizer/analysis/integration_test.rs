//! 分析模块集成测试
//!
//! 测试分析模块各个组件之间的集成功能

use std::sync::Arc;

use crate::core::types::expr::{Expression, ExpressionMeta};
use crate::core::Value;
use crate::query::optimizer::analysis::{
    ExpressionAnalysis, ExpressionAnalyzer, FingerprintCalculator, PlanFingerprint,
    ReferenceCountAnalysis, ReferenceCountAnalyzer,
};
use crate::query::planning::plan::core::nodes::{
    FilterNode, GetVerticesNode, PlanNodeEnum, ProjectNode,
};
use crate::query::validator::context::ExpressionAnalysisContext;

/// 创建测试用的表达式上下文
fn create_test_context(expr: Expression) -> crate::core::types::ContextualExpression {
    let expr_ctx = Arc::new(ExpressionAnalysisContext::new());
    let expr_meta = ExpressionMeta::new(expr);
    let expr_id = expr_ctx.register_expression(expr_meta);
    crate::core::types::ContextualExpression::new(expr_id, expr_ctx)
}

#[test]
fn test_expression_analyzer_integration() {
    let analyzer = ExpressionAnalyzer::new();

    // 测试复杂表达式的分析
    let complex_expr = Expression::Binary {
        left: Box::new(Expression::Property {
            object: Box::new(Expression::Variable("n".to_string())),
            property: "age".to_string(),
        }),
        op: crate::core::types::BinaryOperator::GreaterThan,
        right: Box::new(Expression::Literal(Value::Int(18))),
    };

    let ctx_expr = create_test_context(complex_expr);
    let analysis = analyzer.analyze(&ctx_expr);

    assert!(analysis.is_deterministic);
    assert!(analysis.referenced_properties.contains(&"age".to_string()));
    assert!(analysis.referenced_variables.contains(&"n".to_string()));
    assert!(analysis.node_count > 0);
}

#[test]
fn test_nondeterministic_expression_analysis() {
    let analyzer = ExpressionAnalyzer::new();

    // 测试包含非确定性函数的表达式
    let nondeterministic_expr = Expression::Function {
        name: "rand".to_string(),
        args: vec![],
    };

    let ctx_expr = create_test_context(nondeterministic_expr);
    let analysis = analyzer.analyze(&ctx_expr);

    assert!(!analysis.is_deterministic);
    assert!(analysis.called_functions.contains(&"rand".to_string()));
}

#[test]
fn test_expression_analyzer_options() {
    // 测试只检查确定性的分析器
    let deterministic_analyzer = ExpressionAnalyzer::deterministic_only();
    let expr = Expression::Literal(Value::Int(42));
    let ctx_expr = create_test_context(expr);
    let analysis = deterministic_analyzer.analyze(&ctx_expr);

    assert!(analysis.is_deterministic);
    assert_eq!(analysis.referenced_properties.len(), 0);
    assert_eq!(analysis.referenced_variables.len(), 0);

    // 测试只提取属性的分析器
    let property_analyzer = ExpressionAnalyzer::property_extractor();
    let prop_expr = Expression::Property {
        object: Box::new(Expression::Variable("n".to_string())),
        property: "name".to_string(),
    };
    let ctx_expr = create_test_context(prop_expr);
    let prop_analysis = property_analyzer.analyze(&ctx_expr);

    assert!(prop_analysis
        .referenced_properties
        .contains(&"name".to_string()));
}

#[test]
fn test_fingerprint_calculator_integration() {
    let calculator = FingerprintCalculator::new();

    // 创建两个结构相同的计划节点
    let node1 = PlanNodeEnum::GetVertices(GetVerticesNode::new(1, "Person"));
    let node2 = PlanNodeEnum::GetVertices(GetVerticesNode::new(1, "Person"));

    let fp1 = calculator.calculate_fingerprint(&node1);
    let fp2 = calculator.calculate_fingerprint(&node2);

    // 相同结构的节点应该产生相同的指纹
    assert_eq!(fp1, fp2);

    // 创建不同结构的节点
    let node3 = PlanNodeEnum::Start(crate::query::planning::plan::core::nodes::StartNode::new());
    let fp3 = calculator.calculate_fingerprint(&node3);

    // 不同结构的节点应该产生不同的指纹
    assert_ne!(fp1, fp3);
}

#[test]
fn test_fingerprint_stability() {
    let calculator = FingerprintCalculator::new();

    // 多次计算同一节点的指纹应该产生相同的结果
    let node = PlanNodeEnum::GetVertices(GetVerticesNode::new(1, "Test"));

    let fp1 = calculator.calculate_fingerprint(&node);
    let fp2 = calculator.calculate_fingerprint(&node);
    let fp3 = calculator.calculate_fingerprint(&node);

    assert_eq!(fp1, fp2);
    assert_eq!(fp2, fp3);
}

#[test]
fn test_reference_count_analyzer_integration() {
    let analyzer = ReferenceCountAnalyzer::new();

    // 创建一个简单的计划树
    let start_node =
        PlanNodeEnum::Start(crate::query::planning::plan::core::nodes::StartNode::new());

    let analysis = analyzer.analyze(&start_node);

    // 简单计划不应该有重复的子计划
    assert_eq!(analysis.repeated_count(), 0);
}

#[test]
fn test_expression_complexity_scoring() {
    let analyzer = ExpressionAnalyzer::new();

    // 简单表达式
    let simple_expr = Expression::Literal(Value::Int(1));
    let ctx_simple = create_test_context(simple_expr);
    let simple_analysis = analyzer.analyze(&ctx_simple);

    // 复杂表达式
    let complex_expr = Expression::Binary {
        left: Box::new(Expression::Function {
            name: "abs".to_string(),
            args: vec![Expression::Property {
                object: Box::new(Expression::Variable("x".to_string())),
                property: "value".to_string(),
            }],
        }),
        op: crate::core::types::BinaryOperator::Add,
        right: Box::new(Expression::Function {
            name: "coalesce".to_string(),
            args: vec![
                Expression::Property {
                    object: Box::new(Expression::Variable("y".to_string())),
                    property: "value".to_string(),
                },
                Expression::Literal(Value::Int(0)),
            ],
        }),
    };
    let ctx_complex = create_test_context(complex_expr);
    let complex_analysis = analyzer.analyze(&ctx_complex);

    // 复杂表达式的复杂度评分应该高于简单表达式
    assert!(complex_analysis.complexity_score > simple_analysis.complexity_score);
}

#[test]
fn test_expression_aggregate_detection() {
    let analyzer = ExpressionAnalyzer::new();

    // 包含聚合函数的表达式
    let aggregate_expr = Expression::Aggregate {
        func: crate::core::types::operators::AggregateFunction::Count(None),
        arg: Box::new(Expression::Variable("n".to_string())),
        distinct: false,
    };

    let ctx_expr = create_test_context(aggregate_expr);
    let analysis = analyzer.analyze(&ctx_expr);

    assert!(analysis.contains_aggregate);
}

#[test]
fn test_expression_depth_and_node_count() {
    let analyzer = ExpressionAnalyzer::new();

    // 嵌套表达式
    let nested_expr = Expression::Binary {
        left: Box::new(Expression::Binary {
            left: Box::new(Expression::Binary {
                left: Box::new(Expression::Literal(Value::Int(1))),
                op: crate::core::types::BinaryOperator::Add,
                right: Box::new(Expression::Literal(Value::Int(2))),
            }),
            op: crate::core::types::BinaryOperator::Add,
            right: Box::new(Expression::Literal(Value::Int(3))),
        }),
        op: crate::core::types::BinaryOperator::Add,
        right: Box::new(Expression::Literal(Value::Int(4))),
    };

    let ctx_expr = create_test_context(nested_expr);
    let analysis = analyzer.analyze(&ctx_expr);

    // 验证节点计数
    assert!(analysis.node_count > 0);
}

#[test]
fn test_multiple_function_calls() {
    let analyzer = ExpressionAnalyzer::new();

    // 包含多个函数调用的表达式
    let multi_func_expr = Expression::Function {
        name: "coalesce".to_string(),
        args: vec![
            Expression::Function {
                name: "abs".to_string(),
                args: vec![Expression::Literal(Value::Int(-5))],
            },
            Expression::Function {
                name: "sqrt".to_string(),
                args: vec![Expression::Literal(Value::Int(25))],
            },
            Expression::Literal(Value::Int(0)),
        ],
    };

    let ctx_expr = create_test_context(multi_func_expr);
    let analysis = analyzer.analyze(&ctx_expr);

    assert!(analysis.called_functions.contains(&"coalesce".to_string()));
    assert!(analysis.called_functions.contains(&"abs".to_string()));
    assert!(analysis.called_functions.contains(&"sqrt".to_string()));
}

#[test]
fn test_expression_analyzer_with_case_expression() {
    let analyzer = ExpressionAnalyzer::new();

    // CASE 表达式
    let case_expr = Expression::Case {
        test_expr: None,
        conditions: vec![
            (
                Expression::Binary {
                    left: Box::new(Expression::Variable("x".to_string())),
                    op: crate::core::types::BinaryOperator::GreaterThan,
                    right: Box::new(Expression::Literal(Value::Int(10))),
                },
                Expression::Literal(Value::String("large".to_string())),
            ),
            (
                Expression::Binary {
                    left: Box::new(Expression::Variable("x".to_string())),
                    op: crate::core::types::BinaryOperator::GreaterThan,
                    right: Box::new(Expression::Literal(Value::Int(5))),
                },
                Expression::Literal(Value::String("medium".to_string())),
            ),
        ],
        default: Some(Box::new(Expression::Literal(Value::String(
            "small".to_string(),
        )))),
    };

    let ctx_expr = create_test_context(case_expr);
    let analysis = analyzer.analyze(&ctx_expr);

    assert!(analysis.is_deterministic);
    assert!(analysis.node_count > 0);
}

#[test]
fn test_plan_fingerprint_with_filter() {
    let calculator = FingerprintCalculator::new();

    // 创建带过滤条件的计划节点
    let condition_expr = Expression::Binary {
        left: Box::new(Expression::Property {
            object: Box::new(Expression::Variable("n".to_string())),
            property: "age".to_string(),
        }),
        op: crate::core::types::BinaryOperator::GreaterThan,
        right: Box::new(Expression::Literal(Value::Int(18))),
    };

    let expr_ctx = Arc::new(ExpressionAnalysisContext::new());
    let expr_meta = ExpressionMeta::new(condition_expr);
    let expr_id = expr_ctx.register_expression(expr_meta);
    let ctx_expr = crate::core::types::ContextualExpression::new(expr_id, expr_ctx);

    let input_node = PlanNodeEnum::GetVertices(GetVerticesNode::new(1, "Person"));
    let filter_node = PlanNodeEnum::Filter(
        FilterNode::new(input_node, ctx_expr).expect("Failed to create FilterNode"),
    );

    let fp = calculator.calculate_fingerprint(&filter_node);

    // 验证指纹生成成功
    assert!(fp.value() != 0);
}

#[test]
fn test_plan_fingerprint_with_project() {
    let calculator = FingerprintCalculator::new();

    // 创建带投影的计划节点
    let input_node = PlanNodeEnum::GetVertices(GetVerticesNode::new(1, "Person"));

    // 创建 YieldColumn
    let expr_ctx = Arc::new(ExpressionAnalysisContext::new());
    let expr1 = Expression::Property {
        object: Box::new(Expression::Variable("n".to_string())),
        property: "name".to_string(),
    };
    let expr_meta1 = ExpressionMeta::new(expr1);
    let expr_id1 = expr_ctx.register_expression(expr_meta1);
    let ctx_expr1 = crate::core::types::ContextualExpression::new(expr_id1, expr_ctx.clone());

    let expr2 = Expression::Property {
        object: Box::new(Expression::Variable("n".to_string())),
        property: "age".to_string(),
    };
    let expr_meta2 = ExpressionMeta::new(expr2);
    let expr_id2 = expr_ctx.register_expression(expr_meta2);
    let ctx_expr2 = crate::core::types::ContextualExpression::new(expr_id2, expr_ctx);

    let columns = vec![
        crate::core::YieldColumn::new(ctx_expr1, "n.name".to_string()),
        crate::core::YieldColumn::new(ctx_expr2, "n.age".to_string()),
    ];

    let project_node = PlanNodeEnum::Project(
        ProjectNode::new(input_node, columns).expect("Failed to create ProjectNode"),
    );

    let fp = calculator.calculate_fingerprint(&project_node);

    // 验证指纹生成成功
    assert!(fp.value() != 0);
}

#[test]
fn test_expression_analyzer_with_list_expression() {
    let analyzer = ExpressionAnalyzer::new();

    // 列表表达式
    let list_expr = Expression::List(vec![
        Expression::Literal(Value::Int(1)),
        Expression::Literal(Value::Int(2)),
        Expression::Literal(Value::Int(3)),
    ]);

    let ctx_expr = create_test_context(list_expr);
    let analysis = analyzer.analyze(&ctx_expr);

    assert!(analysis.is_deterministic);
    assert!(analysis.node_count > 0);
}

#[test]
fn test_expression_analyzer_with_map_expression() {
    let analyzer = ExpressionAnalyzer::new();

    // 映射表达式
    let map_expr = Expression::Map(vec![
        ("key1".to_string(), Expression::Literal(Value::Int(1))),
        (
            "key2".to_string(),
            Expression::Literal(Value::String("value".to_string())),
        ),
    ]);

    let ctx_expr = create_test_context(map_expr);
    let analysis = analyzer.analyze(&ctx_expr);

    assert!(analysis.is_deterministic);
    assert!(analysis.node_count > 0);
}

#[test]
fn test_expression_analyzer_with_type_cast() {
    let analyzer = ExpressionAnalyzer::new();

    // 类型转换表达式
    let cast_expr = Expression::TypeCast {
        expression: Box::new(Expression::Literal(Value::String("123".to_string()))),
        target_type: crate::core::types::DataType::Int,
    };

    let ctx_expr = create_test_context(cast_expr);
    let analysis = analyzer.analyze(&ctx_expr);

    assert!(analysis.is_deterministic);
    assert!(analysis.node_count > 0);
}

#[test]
fn test_expression_analyzer_with_subscript() {
    let analyzer = ExpressionAnalyzer::new();

    // 下标访问表达式
    let subscript_expr = Expression::Subscript {
        collection: Box::new(Expression::Variable("arr".to_string())),
        index: Box::new(Expression::Literal(Value::Int(0))),
    };

    let ctx_expr = create_test_context(subscript_expr);
    let analysis = analyzer.analyze(&ctx_expr);

    assert!(analysis.is_deterministic);
    assert!(analysis.referenced_variables.contains(&"arr".to_string()));
}

#[test]
fn test_expression_analyzer_with_unary_operator() {
    let analyzer = ExpressionAnalyzer::new();

    // 一元运算表达式
    let unary_expr = Expression::Unary {
        op: crate::core::types::UnaryOperator::Minus,
        operand: Box::new(Expression::Literal(Value::Int(5))),
    };

    let ctx_expr = create_test_context(unary_expr);
    let analysis = analyzer.analyze(&ctx_expr);

    assert!(analysis.is_deterministic);
    assert!(analysis.node_count > 0);
}

#[test]
fn test_expression_analyzer_with_range() {
    let analyzer = ExpressionAnalyzer::new();

    // 范围表达式
    let range_expr = Expression::Range {
        collection: Box::new(Expression::Variable("list".to_string())),
        start: Some(Box::new(Expression::Literal(Value::Int(0)))),
        end: Some(Box::new(Expression::Literal(Value::Int(10)))),
    };

    let ctx_expr = create_test_context(range_expr);
    let analysis = analyzer.analyze(&ctx_expr);

    assert!(analysis.is_deterministic);
    assert!(analysis.referenced_variables.contains(&"list".to_string()));
}

#[test]
fn test_expression_analyzer_with_label_tag_property() {
    let analyzer = ExpressionAnalyzer::new();

    // 标签属性动态访问
    let label_tag_prop_expr = Expression::LabelTagProperty {
        tag: Box::new(Expression::Variable("tagName".to_string())),
        property: "propertyName".to_string(),
    };

    let ctx_expr = create_test_context(label_tag_prop_expr);
    let analysis = analyzer.analyze(&ctx_expr);

    assert!(analysis.is_deterministic);
    assert!(analysis
        .referenced_variables
        .contains(&"tagName".to_string()));
}

#[test]
fn test_expression_analyzer_with_tag_property() {
    let analyzer = ExpressionAnalyzer::new();

    // 标签属性访问
    let tag_prop_expr = Expression::TagProperty {
        tag_name: "Person".to_string(),
        property: "name".to_string(),
    };

    let ctx_expr = create_test_context(tag_prop_expr);
    let analysis = analyzer.analyze(&ctx_expr);

    assert!(analysis.is_deterministic);
}

#[test]
fn test_expression_analyzer_with_edge_property() {
    let analyzer = ExpressionAnalyzer::new();

    // 边属性访问
    let edge_prop_expr = Expression::EdgeProperty {
        edge_name: "FRIEND".to_string(),
        property: "since".to_string(),
    };

    let ctx_expr = create_test_context(edge_prop_expr);
    let analysis = analyzer.analyze(&ctx_expr);

    assert!(analysis.is_deterministic);
}

#[test]
fn test_expression_analyzer_with_parameter() {
    let analyzer = ExpressionAnalyzer::new();

    // 查询参数表达式
    let param_expr = Expression::Parameter("userId".to_string());

    let ctx_expr = create_test_context(param_expr);
    let analysis = analyzer.analyze(&ctx_expr);

    assert!(analysis.is_deterministic);
    assert!(analysis.node_count > 0);
}

#[test]
fn test_expression_analyzer_with_path() {
    let analyzer = ExpressionAnalyzer::new();

    // 路径表达式
    let path_expr = Expression::Path(vec![
        Expression::Variable("v1".to_string()),
        Expression::Variable("e1".to_string()),
        Expression::Variable("v2".to_string()),
    ]);

    let ctx_expr = create_test_context(path_expr);
    let analysis = analyzer.analyze(&ctx_expr);

    assert!(analysis.is_deterministic);
    assert!(analysis.referenced_variables.contains(&"v1".to_string()));
    assert!(analysis.referenced_variables.contains(&"e1".to_string()));
    assert!(analysis.referenced_variables.contains(&"v2".to_string()));
}

#[test]
fn test_expression_analyzer_with_label() {
    let analyzer = ExpressionAnalyzer::new();

    // 标签表达式
    let label_expr = Expression::Label("Person".to_string());

    let ctx_expr = create_test_context(label_expr);
    let analysis = analyzer.analyze(&ctx_expr);

    assert!(analysis.is_deterministic);
    assert!(analysis.node_count > 0);
}

#[test]
fn test_expression_analysis_default() {
    let analysis = ExpressionAnalysis::new();

    assert!(analysis.is_deterministic);
    assert_eq!(analysis.complexity_score, 0);
    assert_eq!(analysis.referenced_properties.len(), 0);
    assert_eq!(analysis.referenced_variables.len(), 0);
    assert_eq!(analysis.called_functions.len(), 0);
    assert!(!analysis.contains_aggregate);
    assert!(!analysis.contains_subquery);
    assert_eq!(analysis.node_count, 0);
}

#[test]
fn test_plan_fingerprint_default() {
    let fp = PlanFingerprint::new(12345);

    assert_eq!(fp.value(), 12345);
}

#[test]
fn test_reference_count_analysis_default() {
    let analysis = ReferenceCountAnalysis::new();

    assert_eq!(analysis.repeated_count(), 0);
    assert!(!analysis.is_repeated(1));
    assert!(analysis.get_node_info(1).is_none());
}

#[test]
fn test_expression_analyzer_clone() {
    let analyzer = ExpressionAnalyzer::new();
    let _analyzer_clone = analyzer.clone();

    // 验证克隆成功
}

#[test]
fn test_fingerprint_calculator_clone() {
    let calculator = FingerprintCalculator::new();
    let _calculator_clone = calculator.clone();

    // 验证克隆成功
}

#[test]
fn test_reference_count_analyzer_clone() {
    let analyzer = ReferenceCountAnalyzer::new();
    let _analyzer_clone = analyzer.clone();

    // 验证克隆成功
}

#[test]
fn test_expression_analysis_clone() {
    let mut analysis = ExpressionAnalysis::new();
    analysis.is_deterministic = false;
    analysis.complexity_score = 50;

    let analysis_clone = analysis.clone();

    assert!(!analysis_clone.is_deterministic);
    assert_eq!(analysis_clone.complexity_score, 50);
}

#[test]
fn test_plan_fingerprint_hash() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let fp1 = PlanFingerprint::new(12345);
    let fp2 = PlanFingerprint::new(12345);
    let fp3 = PlanFingerprint::new(67890);

    let mut hasher1 = DefaultHasher::new();
    fp1.hash(&mut hasher1);
    let hash1 = hasher1.finish();

    let mut hasher2 = DefaultHasher::new();
    fp2.hash(&mut hasher2);
    let hash2 = hasher2.finish();

    let mut hasher3 = DefaultHasher::new();
    fp3.hash(&mut hasher3);
    let hash3 = hasher3.finish();

    assert_eq!(hash1, hash2);
    assert_ne!(hash1, hash3);
}

#[test]
fn test_expression_analyzer_default() {
    let analyzer = ExpressionAnalyzer::default();

    let expr = Expression::Literal(Value::Int(42));
    let ctx_expr = create_test_context(expr);
    let analysis = analyzer.analyze(&ctx_expr);

    assert!(analysis.is_deterministic);
}

#[test]
fn test_fingerprint_calculator_default() {
    let calculator = FingerprintCalculator::new();

    let node = PlanNodeEnum::Start(crate::query::planning::plan::core::nodes::StartNode::new());
    let fp = calculator.calculate_fingerprint(&node);

    assert!(fp.value() != 0);
}

#[test]
fn test_reference_count_analyzer_default() {
    let analyzer = ReferenceCountAnalyzer::default();

    let node = PlanNodeEnum::Start(crate::query::planning::plan::core::nodes::StartNode::new());
    let analysis = analyzer.analyze(&node);

    assert_eq!(analysis.repeated_count(), 0);
}
