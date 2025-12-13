//! 表达式系统重构测试
//!
//! 验证第三阶段重构：表达式系统重构的正确性

use graphdb::core::Value;
use graphdb::core::error::DBError;
use graphdb::graph::expression::{
    EvalContext, Expression as OldExpression, ExpressionEvaluator as OldEvaluator,
    ExpressionContext, ExpressionEvaluator, DefaultExpressionEvaluator,
    Expression as NewExpression, LiteralValue, BinaryOperator, UnaryOperator, AggregateFunction,
    ExpressionConverter, ContextAdapter, CompatibilityEvaluator
};
use std::collections::HashMap;

/// 创建测试上下文
fn create_test_context() -> EvalContext {
    let mut vars = HashMap::new();
    vars.insert("x".to_string(), Value::Int(10));
    vars.insert("y".to_string(), Value::Int(20));
    vars.insert("name".to_string(), Value::String("Alice".to_string()));
    vars.insert("flag".to_string(), Value::Bool(true));
    
    EvalContext {
        vertex: None,
        edge: None,
        vars,
    }
}

/// 创建新表达式测试上下文
struct TestExpressionContext {
    variables: HashMap<String, Value>,
}

impl TestExpressionContext {
    fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }
    
    fn with_variable(mut self, name: &str, value: Value) -> Self {
        self.variables.insert(name.to_string(), value);
        self
    }
}

impl ExpressionContext for TestExpressionContext {
    fn get_variable(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }

    fn get_property(&self, object: &Value, property: &str) -> DBResult<&Value> {
        match object {
            Value::Map(map) => {
                map.get(property)
                    .ok_or_else(|| DBError::Expression(format!("Property '{}' not found", property)))
            }
            _ => Err(DBError::Expression("Property access not supported".to_string())),
        }
    }

    fn get_function(&self, _name: &str) -> Option<&dyn graphdb::graph::expression::Function> {
        None
    }
}

#[test]
fn test_expression_conversion_old_to_new() {
    // 测试常量转换
    let old_expr = OldExpression::Constant(Value::Int(42));
    let new_expr = ExpressionConverter::convert_old_to_new(&old_expr);
    assert_eq!(new_expr, NewExpression::int(42));
    
    // 测试变量转换
    let old_expr = OldExpression::Variable("x".to_string());
    let new_expr = ExpressionConverter::convert_old_to_new(&old_expr);
    assert_eq!(new_expr, NewExpression::variable("x"));
    
    // 测试二元操作转换
    let old_expr = OldExpression::BinaryOp(
        Box::new(OldExpression::Variable("x".to_string())),
        graphdb::graph::expression::binary::BinaryOperator::Add,
        Box::new(OldExpression::Variable("y".to_string())),
    );
    let new_expr = ExpressionConverter::convert_old_to_new(&old_expr);
    
    if let NewExpression::Binary { left, op, right } = new_expr {
        assert_eq!(*left, NewExpression::variable("x"));
        assert_eq!(op, BinaryOperator::Add);
        assert_eq!(*right, NewExpression::variable("y"));
    } else {
        panic!("Expected binary expression");
    }
    
    // 测试函数调用转换
    let old_expr = OldExpression::Function(
        "count".to_string(),
        vec![OldExpression::Variable("x".to_string())]
    );
    let new_expr = ExpressionConverter::convert_old_to_new(&old_expr);
    
    if let NewExpression::Function { name, args } = new_expr {
        assert_eq!(name, "count");
        assert_eq!(args.len(), 1);
        assert_eq!(args[0], NewExpression::variable("x"));
    } else {
        panic!("Expected function expression");
    }
}

#[test]
fn test_expression_conversion_new_to_old() {
    // 测试常量转换
    let new_expr = NewExpression::int(42);
    let old_expr = ExpressionConverter::convert_new_to_old(&new_expr);
    assert_eq!(old_expr, OldExpression::Constant(Value::Int(42)));
    
    // 测试变量转换
    let new_expr = NewExpression::variable("x");
    let old_expr = ExpressionConverter::convert_new_to_old(&new_expr);
    assert_eq!(old_expr, OldExpression::Variable("x".to_string()));
    
    // 测试二元操作转换
    let new_expr = NewExpression::binary(
        NewExpression::variable("x"),
        BinaryOperator::Add,
        NewExpression::variable("y")
    );
    let old_expr = ExpressionConverter::convert_new_to_old(&new_expr);
    
    if let OldExpression::BinaryOp(left, op, right) = old_expr {
        assert_eq!(*left, OldExpression::Variable("x".to_string()));
        assert_eq!(op, graphdb::graph::expression::binary::BinaryOperator::Add);
        assert_eq!(*right, OldExpression::Variable("y".to_string()));
    } else {
        panic!("Expected binary expression");
    }
}

#[test]
fn test_new_expression_evaluator() {
    let evaluator = DefaultExpressionEvaluator::new();
    let context = TestExpressionContext::new()
        .with_variable("x", Value::Int(10))
        .with_variable("y", Value::Int(20));
    
    // 测试常量求值
    let expr = NewExpression::int(42);
    let result = evaluator.evaluate(&expr, &context).unwrap();
    assert_eq!(result, Value::Int(42));
    
    // 测试变量求值
    let expr = NewExpression::variable("x");
    let result = evaluator.evaluate(&expr, &context).unwrap();
    assert_eq!(result, Value::Int(10));
    
    // 测试二元操作
    let expr = NewExpression::add(
        NewExpression::variable("x"),
        NewExpression::variable("y")
    );
    let result = evaluator.evaluate(&expr, &context).unwrap();
    assert_eq!(result, Value::Int(30));
    
    // 测试比较操作
    let expr = NewExpression::lt(
        NewExpression::variable("x"),
        NewExpression::variable("y")
    );
    let result = evaluator.evaluate(&expr, &context).unwrap();
    assert_eq!(result, Value::Bool(true));
    
    // 测试逻辑操作
    let expr = NewExpression::and(
        NewExpression::bool(true),
        NewExpression::bool(false)
    );
    let result = evaluator.evaluate(&expr, &context).unwrap();
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_aggregate_functions() {
    let evaluator = DefaultExpressionEvaluator::new();
    let context = TestExpressionContext::new();
    
    // 测试计数
    let list_expr = NewExpression::list(vec![
        NewExpression::int(1),
        NewExpression::int(2),
        NewExpression::int(3),
    ]);
    
    let count_expr = NewExpression::aggregate(
        AggregateFunction::Count,
        list_expr,
        false
    );
    
    let result = evaluator.evaluate(&count_expr, &context).unwrap();
    assert_eq!(result, Value::Int(3));
    
    // 测试求和
    let list_expr = NewExpression::list(vec![
        NewExpression::int(1),
        NewExpression::int(2),
        NewExpression::int(3),
    ]);
    
    let sum_expr = NewExpression::aggregate(
        AggregateFunction::Sum,
        list_expr,
        false
    );
    
    let result = evaluator.evaluate(&sum_expr, &context).unwrap();
    assert_eq!(result, Value::Float(6.0));
    
    // 测试平均值
    let list_expr = NewExpression::list(vec![
        NewExpression::int(1),
        NewExpression::int(2),
        NewExpression::int(3),
    ]);
    
    let avg_expr = NewExpression::aggregate(
        AggregateFunction::Avg,
        list_expr,
        false
    );
    
    let result = evaluator.evaluate(&avg_expr, &context).unwrap();
    assert_eq!(result, Value::Float(2.0));
}

#[test]
fn test_type_casting() {
    let evaluator = DefaultExpressionEvaluator::new();
    let context = TestExpressionContext::new();
    
    // 测试整数到浮点数转换
    let expr = NewExpression::cast(
        NewExpression::int(42),
        graphdb::graph::expression::DataType::Float
    );
    
    let result = evaluator.evaluate(&expr, &context).unwrap();
    assert_eq!(result, Value::Float(42.0));
    
    // 测试数字到字符串转换
    let expr = NewExpression::cast(
        NewExpression::int(42),
        graphdb::graph::expression::DataType::String
    );
    
    let result = evaluator.evaluate(&expr, &context).unwrap();
    assert_eq!(result, Value::String("42".to_string()));
}

#[test]
fn test_case_expression() {
    let evaluator = DefaultExpressionEvaluator::new();
    let context = TestExpressionContext::new();
    
    let expr = NewExpression::case(
        vec![
            (
                NewExpression::bool(false),
                NewExpression::int(1)
            ),
            (
                NewExpression::bool(true),
                NewExpression::int(2)
            ),
        ],
        Some(NewExpression::int(3))
    );
    
    let result = evaluator.evaluate(&expr, &context).unwrap();
    assert_eq!(result, Value::Int(2));
}

#[test]
fn test_compatibility_evaluator() {
    let evaluator = CompatibilityEvaluator::new();
    let context = create_test_context();
    
    // 测试常量求值
    let old_expr = OldExpression::Constant(Value::Int(42));
    let result = evaluator.evaluate_old(&old_expr, &context).unwrap();
    assert_eq!(result, Value::Int(42));
    
    // 测试变量求值
    let old_expr = OldExpression::Variable("x".to_string());
    let result = evaluator.evaluate_old(&old_expr, &context).unwrap();
    assert_eq!(result, Value::Int(10));
    
    // 测试二元操作
    let old_expr = OldExpression::BinaryOp(
        Box::new(OldExpression::Variable("x".to_string())),
        graphdb::graph::expression::binary::BinaryOperator::Add,
        Box::new(OldExpression::Variable("y".to_string())),
    );
    let result = evaluator.evaluate_old(&old_expr, &context).unwrap();
    assert_eq!(result, Value::Int(30));
}

#[test]
fn test_expression_properties() {
    // 测试常量检查
    assert!(NewExpression::int(42).is_constant());
    assert!(NewExpression::bool(true).is_constant());
    assert!(!NewExpression::variable("x").is_constant());
    
    // 测试聚合函数检查
    let agg_expr = NewExpression::aggregate(
        AggregateFunction::Count,
        NewExpression::variable("x"),
        false
    );
    assert!(agg_expr.contains_aggregate());
    
    let simple_expr = NewExpression::add(
        NewExpression::int(1),
        NewExpression::int(2)
    );
    assert!(!simple_expr.contains_aggregate());
    
    // 测试变量提取
    let complex_expr = NewExpression::add(
        NewExpression::variable("x"),
        NewExpression::mul(
            NewExpression::variable("y"),
            NewExpression::int(2)
        )
    );
    let vars = complex_expr.get_variables();
    assert_eq!(vars, vec!["x", "y"]);
}

#[test]
fn test_expression_builder_methods() {
    // 测试便捷构建方法
    let expr = NewExpression::eq(
        NewExpression::variable("x"),
        NewExpression::int(42)
    );
    
    if let NewExpression::Binary { left, op, right } = expr {
        assert_eq!(*left, NewExpression::variable("x"));
        assert_eq!(op, BinaryOperator::Equal);
        assert_eq!(*right, NewExpression::int(42));
    } else {
        panic!("Expected binary expression");
    }
    
    // 测试逻辑操作
    let expr = NewExpression::and(
        NewExpression::bool(true),
        NewExpression::not(NewExpression::bool(false))
    );
    
    if let NewExpression::Binary { left, op, right } = expr {
        assert_eq!(*left, NewExpression::bool(true));
        assert_eq!(op, BinaryOperator::And);
        
        if let NewExpression::Unary { op: not_op, operand } = right.as_ref() {
            assert_eq!(*not_op, UnaryOperator::Not);
            assert_eq!(*operand, NewExpression::bool(false));
        } else {
            panic!("Expected unary expression");
        }
    } else {
        panic!("Expected binary expression");
    }
}

#[test]
fn test_container_expressions() {
    let evaluator = DefaultExpressionEvaluator::new();
    let context = TestExpressionContext::new();
    
    // 测试列表
    let list_expr = NewExpression::list(vec![
        NewExpression::int(1),
        NewExpression::int(2),
        NewExpression::int(3),
    ]);
    
    let result = evaluator.evaluate(&list_expr, &context).unwrap();
    if let Value::List(items) = result {
        assert_eq!(items.len(), 3);
        assert_eq!(items[0], Value::Int(1));
        assert_eq!(items[1], Value::Int(2));
        assert_eq!(items[2], Value::Int(3));
    } else {
        panic!("Expected list value");
    }
    
    // 测试映射
    let map_expr = NewExpression::map(vec![
        ("a", NewExpression::int(1)),
        ("b", NewExpression::string("hello")),
    ]);
    
    let result = evaluator.evaluate(&map_expr, &context).unwrap();
    if let Value::Map(map) = result {
        assert_eq!(map.len(), 2);
        assert_eq!(map.get("a"), Some(&Value::Int(1)));
        assert_eq!(map.get("b"), Some(&Value::String("hello".to_string())));
    } else {
        panic!("Expected map value");
    }
}

#[test]
fn test_performance_comparison() {
    use std::time::Instant;
    
    let old_evaluator = OldEvaluator;
    let new_evaluator = DefaultExpressionEvaluator::new();
    let compat_evaluator = CompatibilityEvaluator::new();
    let context = create_test_context();
    let new_context = TestExpressionContext::new()
        .with_variable("x", Value::Int(10))
        .with_variable("y", Value::Int(20));
    
    // 创建复杂的表达式
    let old_expr = OldExpression::BinaryOp(
        Box::new(OldExpression::BinaryOp(
            Box::new(OldExpression::Variable("x".to_string())),
            graphdb::graph::expression::binary::BinaryOperator::Add,
            Box::new(OldExpression::Variable("y".to_string())),
        )),
        graphdb::graph::expression::binary::BinaryOperator::Multiply,
        Box::new(OldExpression::Constant(Value::Int(2))),
    );
    
    let new_expr = NewExpression::mul(
        NewExpression::add(
            NewExpression::variable("x"),
            NewExpression::variable("y")
        ),
        NewExpression::int(2)
    );
    
    // 性能测试
    let iterations = 10000;
    
    // 测试旧求值器
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = old_evaluator.evaluate(&old_expr, &context);
    }
    let old_duration = start.elapsed();
    
    // 测试新求值器
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = new_evaluator.evaluate(&new_expr, &new_context);
    }
    let new_duration = start.elapsed();
    
    // 测试兼容性求值器
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = compat_evaluator.evaluate_old(&old_expr, &context);
    }
    let compat_duration = start.elapsed();
    
    println!("Performance comparison ({} iterations):", iterations);
    println!("Old evaluator: {:?}", old_duration);
    println!("New evaluator: {:?}", new_duration);
    println!("Compatibility evaluator: {:?}", compat_duration);
    
    // 验证结果一致性
    let old_result = old_evaluator.evaluate(&old_expr, &context).unwrap();
    let new_result = new_evaluator.evaluate(&new_expr, &new_context).unwrap();
    let compat_result = compat_evaluator.evaluate_old(&old_expr, &context).unwrap();
    
    assert_eq!(old_result, new_result);
    assert_eq!(old_result, compat_result);
    assert_eq!(new_result, Value::Int(60)); // (10 + 20) * 2 = 60
}