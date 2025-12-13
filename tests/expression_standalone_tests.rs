//! 表达式系统独立测试
//!
//! 验证第三阶段重构：表达式系统重构的核心功能

use graphdb::core::Value;
use graphdb::core::error::DBError;
use graphdb::graph::expression::{
    ExpressionContext, ExpressionEvaluator, DefaultExpressionEvaluator,
    Expression as NewExpression, LiteralValue, BinaryOperator, UnaryOperator, AggregateFunction,
    DataType
};
use std::collections::HashMap;

/// 创建测试表达式上下文
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
fn test_literal_expressions() {
    let evaluator = DefaultExpressionEvaluator::new();
    let context = TestExpressionContext::new();
    
    // 测试整数字面量
    let expr = NewExpression::int(42);
    let result = evaluator.evaluate(&expr, &context).unwrap();
    assert_eq!(result, Value::Int(42));
    
    // 测试浮点数字面量
    let expr = NewExpression::float(3.14);
    let result = evaluator.evaluate(&expr, &context).unwrap();
    assert_eq!(result, Value::Float(3.14));
    
    // 测试布尔字面量
    let expr = NewExpression::bool(true);
    let result = evaluator.evaluate(&expr, &context).unwrap();
    assert_eq!(result, Value::Bool(true));
    
    // 测试字符串字面量
    let expr = NewExpression::string("hello");
    let result = evaluator.evaluate(&expr, &context).unwrap();
    assert_eq!(result, Value::String("hello".to_string()));
    
    // 测试空值
    let expr = NewExpression::null();
    let result = evaluator.evaluate(&expr, &context).unwrap();
    assert!(matches!(result, Value::Null(_)));
}

#[test]
fn test_variable_expressions() {
    let evaluator = DefaultExpressionEvaluator::new();
    let context = TestExpressionContext::new()
        .with_variable("x", Value::Int(10))
        .with_variable("name", Value::String("Alice".to_string()))
        .with_variable("flag", Value::Bool(true));
    
    // 测试变量访问
    let expr = NewExpression::variable("x");
    let result = evaluator.evaluate(&expr, &context).unwrap();
    assert_eq!(result, Value::Int(10));
    
    let expr = NewExpression::variable("name");
    let result = evaluator.evaluate(&expr, &context).unwrap();
    assert_eq!(result, Value::String("Alice".to_string()));
    
    let expr = NewExpression::variable("flag");
    let result = evaluator.evaluate(&expr, &context).unwrap();
    assert_eq!(result, Value::Bool(true));
    
    // 测试不存在的变量
    let expr = NewExpression::variable("unknown");
    let result = evaluator.evaluate(&expr, &context);
    assert!(result.is_err());
}

#[test]
fn test_binary_operations() {
    let evaluator = DefaultExpressionEvaluator::new();
    let context = TestExpressionContext::new()
        .with_variable("x", Value::Int(10))
        .with_variable("y", Value::Int(20))
        .with_variable("a", Value::Float(1.5))
        .with_variable("b", Value::Float(2.5));
    
    // 测试算术操作
    let expr = NewExpression::add(
        NewExpression::variable("x"),
        NewExpression::variable("y")
    );
    let result = evaluator.evaluate(&expr, &context).unwrap();
    assert_eq!(result, Value::Int(30));
    
    let expr = NewExpression::sub(
        NewExpression::variable("y"),
        NewExpression::variable("x")
    );
    let result = evaluator.evaluate(&expr, &context).unwrap();
    assert_eq!(result, Value::Int(10));
    
    let expr = NewExpression::mul(
        NewExpression::variable("x"),
        NewExpression::variable("y")
    );
    let result = evaluator.evaluate(&expr, &context).unwrap();
    assert_eq!(result, Value::Int(200));
    
    let expr = NewExpression::div(
        NewExpression::variable("y"),
        NewExpression::variable("x")
    );
    let result = evaluator.evaluate(&expr, &context).unwrap();
    assert_eq!(result, Value::Int(2));
    
    // 测试浮点数运算
    let expr = NewExpression::add(
        NewExpression::variable("a"),
        NewExpression::variable("b")
    );
    let result = evaluator.evaluate(&expr, &context).unwrap();
    assert_eq!(result, Value::Float(4.0));
    
    // 测试比较操作
    let expr = NewExpression::eq(
        NewExpression::variable("x"),
        NewExpression::int(10)
    );
    let result = evaluator.evaluate(&expr, &context).unwrap();
    assert_eq!(result, Value::Bool(true));
    
    let expr = NewExpression::lt(
        NewExpression::variable("x"),
        NewExpression::variable("y")
    );
    let result = evaluator.evaluate(&expr, &context).unwrap();
    assert_eq!(result, Value::Bool(true));
    
    let expr = NewExpression::gt(
        NewExpression::variable("x"),
        NewExpression::variable("y")
    );
    let result = evaluator.evaluate(&expr, &context).unwrap();
    assert_eq!(result, Value::Bool(false));
    
    // 测试逻辑操作
    let expr = NewExpression::and(
        NewExpression::bool(true),
        NewExpression::bool(false)
    );
    let result = evaluator.evaluate(&expr, &context).unwrap();
    assert_eq!(result, Value::Bool(false));
    
    let expr = NewExpression::or(
        NewExpression::bool(true),
        NewExpression::bool(false)
    );
    let result = evaluator.evaluate(&expr, &context).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_unary_operations() {
    let evaluator = DefaultExpressionEvaluator::new();
    let context = TestExpressionContext::new()
        .with_variable("x", Value::Int(10))
        .with_variable("flag", Value::Bool(true));
    
    // 测试一元加
    let expr = NewExpression::unary(
        UnaryOperator::Plus,
        NewExpression::variable("x")
    );
    let result = evaluator.evaluate(&expr, &context).unwrap();
    assert_eq!(result, Value::Int(10));
    
    // 测试一元减
    let expr = NewExpression::unary(
        UnaryOperator::Minus,
        NewExpression::variable("x")
    );
    let result = evaluator.evaluate(&expr, &context).unwrap();
    assert_eq!(result, Value::Int(-10));
    
    // 测试逻辑非
    let expr = NewExpression::unary(
        UnaryOperator::Not,
        NewExpression::variable("flag")
    );
    let result = evaluator.evaluate(&expr, &context).unwrap();
    assert_eq!(result, Value::Bool(false));
    
    // 测试空值检查
    let expr = NewExpression::unary(
        UnaryOperator::IsNull,
        NewExpression::null()
    );
    let result = evaluator.evaluate(&expr, &context).unwrap();
    assert_eq!(result, Value::Bool(true));
    
    let expr = NewExpression::unary(
        UnaryOperator::IsNotNull,
        NewExpression::variable("x")
    );
    let result = evaluator.evaluate(&expr, &context).unwrap();
    assert_eq!(result, Value::Bool(true));
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
        ("c", NewExpression::bool(true)),
    ]);
    
    let result = evaluator.evaluate(&map_expr, &context).unwrap();
    if let Value::Map(map) = result {
        assert_eq!(map.len(), 3);
        assert_eq!(map.get("a"), Some(&Value::Int(1)));
        assert_eq!(map.get("b"), Some(&Value::String("hello".to_string())));
        assert_eq!(map.get("c"), Some(&Value::Bool(true)));
    } else {
        panic!("Expected map value");
    }
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
    
    // 测试最小值
    let list_expr = NewExpression::list(vec![
        NewExpression::int(3),
        NewExpression::int(1),
        NewExpression::int(2),
    ]);
    
    let min_expr = NewExpression::aggregate(
        AggregateFunction::Min,
        list_expr,
        false
    );
    
    let result = evaluator.evaluate(&min_expr, &context).unwrap();
    assert_eq!(result, Value::Int(1));
    
    // 测试最大值
    let list_expr = NewExpression::list(vec![
        NewExpression::int(1),
        NewExpression::int(3),
        NewExpression::int(2),
    ]);
    
    let max_expr = NewExpression::aggregate(
        AggregateFunction::Max,
        list_expr,
        false
    );
    
    let result = evaluator.evaluate(&max_expr, &context).unwrap();
    assert_eq!(result, Value::Int(3));
}

#[test]
fn test_type_casting() {
    let evaluator = DefaultExpressionEvaluator::new();
    let context = TestExpressionContext::new();
    
    // 测试整数到浮点数转换
    let expr = NewExpression::cast(
        NewExpression::int(42),
        DataType::Float
    );
    
    let result = evaluator.evaluate(&expr, &context).unwrap();
    assert_eq!(result, Value::Float(42.0));
    
    // 测试浮点数到整数转换
    let expr = NewExpression::cast(
        NewExpression::float(3.14),
        DataType::Int
    );
    
    let result = evaluator.evaluate(&expr, &context).unwrap();
    assert_eq!(result, Value::Int(3));
    
    // 测试数字到字符串转换
    let expr = NewExpression::cast(
        NewExpression::int(42),
        DataType::String
    );
    
    let result = evaluator.evaluate(&expr, &context).unwrap();
    assert_eq!(result, Value::String("42".to_string()));
    
    // 测试布尔值转换
    let expr = NewExpression::cast(
        NewExpression::bool(true),
        DataType::String
    );
    
    let result = evaluator.evaluate(&expr, &context).unwrap();
    assert_eq!(result, Value::String("true".to_string()));
}

#[test]
fn test_case_expression() {
    let evaluator = DefaultExpressionEvaluator::new();
    let context = TestExpressionContext::new();
    
    // 测试第一个条件为真
    let expr = NewExpression::case(
        vec![
            (
                NewExpression::bool(true),
                NewExpression::int(1)
            ),
            (
                NewExpression::bool(false),
                NewExpression::int(2)
            ),
        ],
        Some(NewExpression::int(3))
    );
    
    let result = evaluator.evaluate(&expr, &context).unwrap();
    assert_eq!(result, Value::Int(1));
    
    // 测试第二个条件为真
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
    
    // 测试默认值
    let expr = NewExpression::case(
        vec![
            (
                NewExpression::bool(false),
                NewExpression::int(1)
            ),
            (
                NewExpression::bool(false),
                NewExpression::int(2)
            ),
        ],
        Some(NewExpression::int(3))
    );
    
    let result = evaluator.evaluate(&expr, &context).unwrap();
    assert_eq!(result, Value::Int(3));
    
    // 测试无默认值且所有条件为假
    let expr = NewExpression::case(
        vec![
            (
                NewExpression::bool(false),
                NewExpression::int(1)
            ),
            (
                NewExpression::bool(false),
                NewExpression::int(2)
            ),
        ],
        None
    );
    
    let result = evaluator.evaluate(&expr, &context).unwrap();
    assert!(matches!(result, Value::Null(_)));
}

#[test]
fn test_complex_expressions() {
    let evaluator = DefaultExpressionEvaluator::new();
    let context = TestExpressionContext::new()
        .with_variable("x", Value::Int(10))
        .with_variable("y", Value::Int(20))
        .with_variable("z", Value::Int(30));
    
    // 测试嵌套的二元操作
    let expr = NewExpression::add(
        NewExpression::mul(
            NewExpression::variable("x"),
            NewExpression::variable("y")
        ),
        NewExpression::variable("z")
    );
    
    let result = evaluator.evaluate(&expr, &context).unwrap();
    assert_eq!(result, Value::Int(230)); // 10 * 20 + 30 = 230
    
    // 测试复杂的逻辑表达式
    let expr = NewExpression::and(
        NewExpression::gt(
            NewExpression::variable("x"),
            NewExpression::int(5)
        ),
        NewExpression::or(
            NewExpression::eq(
                NewExpression::variable("y"),
                NewExpression::int(20)
            ),
            NewExpression::eq(
                NewExpression::variable("z"),
                NewExpression::int(25)
            )
        )
    );
    
    let result = evaluator.evaluate(&expr, &context).unwrap();
    assert_eq!(result, Value::Bool(true)); // (10 > 5) && (20 == 20 || 30 == 25) = true && (true || false) = true
}

#[test]
fn test_expression_properties() {
    // 测试常量检查
    assert!(NewExpression::int(42).is_constant());
    assert!(NewExpression::bool(true).is_constant());
    assert!(NewExpression::string("hello").is_constant());
    assert!(!NewExpression::variable("x").is_constant());
    
    // 测试列表常量检查
    let const_list = NewExpression::list(vec![
        NewExpression::int(1),
        NewExpression::int(2),
        NewExpression::int(3),
    ]);
    assert!(const_list.is_constant());
    
    let non_const_list = NewExpression::list(vec![
        NewExpression::int(1),
        NewExpression::variable("x"),
        NewExpression::int(3),
    ]);
    assert!(!non_const_list.is_constant());
    
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
    
    // 测试嵌套表达式中的变量提取
    let nested_expr = NewExpression::case(
        vec![
            (
                NewExpression::gt(NewExpression::variable("x"), NewExpression::int(0)),
                NewExpression::variable("y")
            ),
        ],
        Some(NewExpression::variable("z"))
    );
    let vars = nested_expr.get_variables();
    assert_eq!(vars, vec!["x", "y", "z"]);
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
    
    // 测试链式比较
    let expr = NewExpression::and(
        NewExpression::ge(NewExpression::variable("x"), NewExpression::int(0)),
        NewExpression::le(NewExpression::variable("x"), NewExpression::int(100))
    );
    
    if let NewExpression::Binary { left, op, right } = expr {
        assert_eq!(op, BinaryOperator::And);
        
        if let NewExpression::Binary { left: left_left, op: left_op, right: left_right } = left.as_ref() {
            assert_eq!(*left_op, BinaryOperator::GreaterThanOrEqual);
            assert_eq!(*left_left, NewExpression::variable("x"));
            assert_eq!(*left_right, NewExpression::int(0));
        } else {
            panic!("Expected binary expression");
        }
        
        if let NewExpression::Binary { left: right_left, op: right_op, right: right_right } = right.as_ref() {
            assert_eq!(*right_op, BinaryOperator::LessThanOrEqual);
            assert_eq!(*right_left, NewExpression::variable("x"));
            assert_eq!(*right_right, NewExpression::int(100));
        } else {
            panic!("Expected binary expression");
        }
    } else {
        panic!("Expected binary expression");
    }
}

#[test]
fn test_error_handling() {
    let evaluator = DefaultExpressionEvaluator::new();
    let context = TestExpressionContext::new();
    
    // 测试除零错误
    let expr = NewExpression::div(
        NewExpression::int(10),
        NewExpression::int(0)
    );
    let result = evaluator.evaluate(&expr, &context);
    assert!(result.is_err());
    
    // 测试模零错误
    let expr = NewExpression::binary(
        NewExpression::int(10),
        BinaryOperator::Modulo,
        NewExpression::int(0)
    );
    let result = evaluator.evaluate(&expr, &context);
    assert!(result.is_err());
    
    // 测试类型错误
    let expr = NewExpression::add(
        NewExpression::int(10),
        NewExpression::bool(true)
    );
    let result = evaluator.evaluate(&expr, &context);
    assert!(result.is_err());
    
    // 测试一元操作类型错误
    let expr = NewExpression::unary(
        UnaryOperator::Minus,
        NewExpression::bool(true)
    );
    let result = evaluator.evaluate(&expr, &context);
    assert!(result.is_err());
}