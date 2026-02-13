//! 阶段二：核心类型与表达式集成测试
//!
//! 测试范围:
//! - core::value - 值类型转换、比较、运算
//! - core::types - 类型系统兼容性检查
//! - expression::evaluator - 表达式求值、上下文访问
//! - expression::functions - 内置函数注册与调用
//! - expression::context - 上下文链、缓存管理

mod common;

use graphdb::core::value::{NullType, Value, DateValue, TimeValue, DateTimeValue, GeographyValue, DurationValue, DataSet};
use graphdb::core::types::DataType;
// Vertex, Edge, Path 在需要时通过 graphdb::core 使用
use graphdb::core::types::expression::Expression;
use graphdb::expression::{ExpressionEvaluator, ExpressionContext, BasicExpressionContext};
use graphdb::expression::functions::global_registry;

// ==================== Value 类型测试 ====================

#[test]
fn test_value_null_type_variants() {
    // 测试所有 NullType 变体
    let null_variants = vec![
        NullType::Null,
        NullType::NaN,
        NullType::BadData,
        NullType::BadType,
        NullType::ErrOverflow,
        NullType::UnknownProp,
        NullType::DivByZero,
        NullType::OutOfRange,
    ];

    for variant in &null_variants {
        let value = Value::Null(variant.clone());
        assert!(value.is_null());
        assert_eq!(value.get_type(), DataType::Null);
    }

    // 测试 is_bad 方法
    assert!(NullType::BadData.is_bad());
    assert!(NullType::BadType.is_bad());
    assert!(NullType::ErrOverflow.is_bad());
    assert!(NullType::OutOfRange.is_bad());
    assert!(!NullType::Null.is_bad());
    assert!(!NullType::NaN.is_bad());

    // 测试 is_computational_error 方法
    assert!(NullType::NaN.is_computational_error());
    assert!(NullType::DivByZero.is_computational_error());
    assert!(NullType::ErrOverflow.is_computational_error());
}

#[test]
fn test_value_type_checking() {
    // 基础类型检查
    assert_eq!(Value::Empty.get_type(), DataType::Empty);
    assert_eq!(Value::Null(NullType::Null).get_type(), DataType::Null);
    assert_eq!(Value::Bool(true).get_type(), DataType::Bool);
    assert_eq!(Value::Int(42).get_type(), DataType::Int);
    assert_eq!(Value::Float(3.14).get_type(), DataType::Float);
    assert_eq!(Value::String("test".to_string()).get_type(), DataType::String);

    // 数值类型检查
    assert!(Value::Int(42).is_numeric());
    assert!(Value::Float(3.14).is_numeric());
    assert!(!Value::String("42".to_string()).is_numeric());
    assert!(!Value::Bool(true).is_numeric());

    // BadNull 检查
    assert!(Value::Null(NullType::BadData).is_bad_null());
    assert!(Value::Null(NullType::BadType).is_bad_null());
    assert!(!Value::Null(NullType::Null).is_bad_null());
}

#[test]
fn test_value_boolean_conversion() {
    // 布尔值直接返回
    assert_eq!(Value::Bool(true).to_bool(), Value::Bool(true));
    assert_eq!(Value::Bool(false).to_bool(), Value::Bool(false));

    // 字符串转换
    assert_eq!(Value::String("true".to_string()).to_bool(), Value::Bool(true));
    assert_eq!(Value::String("TRUE".to_string()).to_bool(), Value::Bool(true));
    assert_eq!(Value::String("false".to_string()).to_bool(), Value::Bool(false));
    assert_eq!(Value::String("FALSE".to_string()).to_bool(), Value::Bool(false));
    assert_eq!(Value::String("invalid".to_string()).to_bool(), Value::Null(NullType::Null));

    // 空值和 Null
    assert_eq!(Value::Empty.to_bool(), Value::Null(NullType::Null));
    assert_eq!(Value::Null(NullType::Null).to_bool(), Value::Null(NullType::Null));

    // 其他类型返回 BadData
    assert_eq!(Value::Int(1).to_bool(), Value::Null(NullType::BadData));
}

#[test]
fn test_value_integer_conversion() {
    // 整数直接返回
    assert_eq!(Value::Int(42).to_int(), Value::Int(42));
    assert_eq!(Value::Int(-100).to_int(), Value::Int(-100));

    // 浮点数转换（截断）
    assert_eq!(Value::Float(3.14).to_int(), Value::Int(3));
    assert_eq!(Value::Float(-2.9).to_int(), Value::Int(-2));

    // 边界值处理
    assert_eq!(Value::Float(f64::NAN).to_int(), Value::Null(NullType::Null));
    assert_eq!(Value::Float(f64::INFINITY).to_int(), Value::Null(NullType::Null));
    assert_eq!(Value::Float(f64::NEG_INFINITY).to_int(), Value::Null(NullType::Null));

    // 字符串解析
    assert_eq!(Value::String("42".to_string()).to_int(), Value::Int(42));
    assert_eq!(Value::String("-100".to_string()).to_int(), Value::Int(-100));
    assert_eq!(Value::String("invalid".to_string()).to_int(), Value::Null(NullType::Null));

    // 布尔值转换
    assert_eq!(Value::Bool(true).to_int(), Value::Int(1));
    assert_eq!(Value::Bool(false).to_int(), Value::Int(0));
}

#[test]
fn test_value_float_conversion() {
    // 浮点数直接返回
    assert_eq!(Value::Float(3.14).to_float(), Value::Float(3.14));

    // 整数转换
    assert_eq!(Value::Int(42).to_float(), Value::Float(42.0));

    // 字符串解析
    assert_eq!(Value::String("3.14".to_string()).to_float(), Value::Float(3.14));
    assert_eq!(Value::String("-2.5".to_string()).to_float(), Value::Float(-2.5));
    assert_eq!(Value::String("invalid".to_string()).to_float(), Value::Null(NullType::Null));

    // 布尔值转换
    assert_eq!(Value::Bool(true).to_float(), Value::Float(1.0));
    assert_eq!(Value::Bool(false).to_float(), Value::Float(0.0));
}

#[test]
fn test_value_arithmetic_operations() {
    // 加法
    assert_eq!(
        Value::Int(10).add(&Value::Int(5)).expect("整数加法应该成功"),
        Value::Int(15)
    );
    assert_eq!(
        Value::Float(3.5).add(&Value::Float(2.5)).expect("浮点数加法应该成功"),
        Value::Float(6.0)
    );
    assert_eq!(
        Value::Int(10).add(&Value::Float(2.5)).expect("整数与浮点数加法应该成功"),
        Value::Float(12.5)
    );
    assert_eq!(
        Value::String("Hello, ".to_string()).add(&Value::String("World".to_string())).expect("字符串连接应该成功"),
        Value::String("Hello, World".to_string())
    );

    // 减法
    assert_eq!(
        Value::Int(10).sub(&Value::Int(3)).expect("整数减法应该成功"),
        Value::Int(7)
    );
    assert_eq!(
        Value::Float(10.5).sub(&Value::Float(3.5)).expect("浮点数减法应该成功"),
        Value::Float(7.0)
    );

    // 乘法
    assert_eq!(
        Value::Int(6).mul(&Value::Int(7)).expect("整数乘法应该成功"),
        Value::Int(42)
    );
    assert_eq!(
        Value::Float(3.0).mul(&Value::Float(4.0)).expect("浮点数乘法应该成功"),
        Value::Float(12.0)
    );

    // 除法
    assert_eq!(
        Value::Int(10).div(&Value::Int(2)).expect("整数除法应该成功"),
        Value::Int(5)
    );
    assert_eq!(
        Value::Float(10.0).div(&Value::Float(4.0)).expect("浮点数除法应该成功"),
        Value::Float(2.5)
    );

    // 除零错误
    assert!(Value::Int(10).div(&Value::Int(0)).is_err());
    assert!(Value::Float(10.0).div(&Value::Float(0.0)).is_err());

    // 取模
    assert_eq!(
        Value::Int(10).rem(&Value::Int(3)).expect("整数取模应该成功"),
        Value::Int(1)
    );
    assert!(Value::Int(10).rem(&Value::Int(0)).is_err());
}

#[test]
fn test_value_comparison() {
    // 整数比较
    assert!(Value::Int(10) > Value::Int(5));
    assert!(Value::Int(5) < Value::Int(10));
    assert_eq!(Value::Int(10), Value::Int(10));

    // 浮点数比较（包含 NaN 处理）
    assert!(Value::Float(3.14) > Value::Float(2.0));
    assert_eq!(Value::Float(f64::NAN), Value::Float(f64::NAN));

    // 字符串比较
    assert!(Value::String("b".to_string()) > Value::String("a".to_string()));
    assert_eq!(Value::String("test".to_string()), Value::String("test".to_string()));

    // 布尔值比较
    assert!(Value::Bool(true) > Value::Bool(false));

    // 不同类型比较（基于类型优先级）
    assert!(Value::Int(1) < Value::String("a".to_string()));
}

#[test]
fn test_value_unary_operations() {
    // 取反
    assert_eq!(Value::Int(42).negate().expect("整数取反应该成功"), Value::Int(-42));
    assert_eq!(Value::Float(3.14).negate().expect("浮点数取反应该成功"), Value::Float(-3.14));
    assert!(Value::String("test".to_string()).negate().is_err());

    // 绝对值
    assert_eq!(Value::Int(-42).abs().expect("整数绝对值应该成功"), Value::Int(42));
    assert_eq!(Value::Float(-3.14).abs().expect("浮点数绝对值应该成功"), Value::Float(3.14));
    assert!(Value::String("test".to_string()).abs().is_err());

    // 长度
    assert_eq!(Value::String("hello".to_string()).length().expect("字符串长度计算应该成功"), Value::Int(5));
    assert_eq!(Value::List(vec![Value::Int(1), Value::Int(2)]).length().expect("列表长度计算应该成功"), Value::Int(2));
    assert_eq!(Value::Map(std::collections::HashMap::new()).length().expect("映射长度计算应该成功"), Value::Int(0));
}

#[test]
fn test_value_complex_types() {
    // DateValue
    let date = DateValue { year: 2024, month: 6, day: 15 };
    assert_eq!(date.year, 2024);
    assert_eq!(date.month, 6);
    assert_eq!(date.day, 15);

    // TimeValue
    let time = TimeValue { hour: 14, minute: 30, sec: 45, microsec: 0 };
    assert_eq!(time.hour, 14);
    assert_eq!(time.minute, 30);

    // DateTimeValue
    let datetime = DateTimeValue { year: 2024, month: 6, day: 15, hour: 14, minute: 30, sec: 0, microsec: 0 };
    assert_eq!(datetime.year, 2024);

    // GeographyValue
    let geo = GeographyValue { latitude: 39.9042, longitude: 116.4074 };
    assert_eq!(geo.latitude, 39.9042);
    assert_eq!(geo.longitude, 116.4074);

    // DurationValue
    let duration = DurationValue { seconds: 3600, microseconds: 0, months: 0 };
    assert_eq!(duration.seconds, 3600);

    // DataSet
    let mut dataset = DataSet::new();
    dataset.col_names.push("name".to_string());
    dataset.col_names.push("age".to_string());
    dataset.rows.push(vec![Value::String("Alice".to_string()), Value::Int(30)]);
    assert_eq!(dataset.col_names.len(), 2);
    assert_eq!(dataset.rows.len(), 1);
}

#[test]
fn test_value_hash_and_equality() {
    use std::collections::HashSet;

    // 测试哈希一致性
    let value1 = Value::Int(42);
    let value2 = Value::Int(42);
    assert_eq!(value1.hash_value(), value2.hash_value());

    // 测试浮点数哈希（包括特殊值）
    let nan1 = Value::Float(f64::NAN);
    let nan2 = Value::Float(f64::NAN);
    assert_eq!(nan1.hash_value(), nan2.hash_value());

    let pos_zero = Value::Float(0.0);
    let neg_zero = Value::Float(-0.0);
    assert_eq!(pos_zero.hash_value(), neg_zero.hash_value());

    // 测试在 HashSet 中使用
    let mut set = HashSet::new();
    set.insert(Value::Int(1));
    set.insert(Value::Int(2));
    set.insert(Value::Int(1)); // 重复
    assert_eq!(set.len(), 2);
}

// ==================== DataType 测试 ====================

#[test]
fn test_datatype_variants() {
    // 测试所有 DataType 变体
    let types = vec![
        DataType::Empty,
        DataType::Null,
        DataType::Bool,
        DataType::Int,
        DataType::Int8,
        DataType::Int16,
        DataType::Int32,
        DataType::Int64,
        DataType::Float,
        DataType::Double,
        DataType::String,
        DataType::Date,
        DataType::Time,
        DataType::DateTime,
        DataType::Vertex,
        DataType::Edge,
        DataType::Path,
        DataType::List,
        DataType::Map,
        DataType::Set,
        DataType::Geography,
        DataType::Duration,
        DataType::DataSet,
        DataType::FixedString(100),
        DataType::VID,
        DataType::Blob,
        DataType::Timestamp,
    ];

    // 确保所有类型都可以被克隆和比较
    for dtype in &types {
        let cloned = dtype.clone();
        assert_eq!(*dtype, cloned);
    }
}

// ==================== Expression 测试 ====================

#[test]
fn test_expression_literal_creation() {
    // 整数字面量
    let expr = Expression::literal(42i64);
    match &expr {
        Expression::Literal(v) => assert_eq!(*v, Value::Int(42)),
        _ => panic!("期望 Literal 表达式"),
    }

    // 字符串字面量
    let expr = Expression::literal("hello".to_string());
    match &expr {
        Expression::Literal(v) => assert_eq!(*v, Value::String("hello".to_string())),
        _ => panic!("期望 Literal 表达式"),
    }

    // 布尔字面量
    let expr = Expression::literal(true);
    match &expr {
        Expression::Literal(v) => assert_eq!(*v, Value::Bool(true)),
        _ => panic!("期望 Literal 表达式"),
    }
}

#[test]
fn test_expression_variable_creation() {
    let expr = Expression::variable("x");
    match &expr {
        Expression::Variable(name) => assert_eq!(name, "x"),
        _ => panic!("期望 Variable 表达式"),
    }
}

#[test]
fn test_expression_binary_creation() {
    use graphdb::core::types::BinaryOperator;

    let left = Expression::literal(10i64);
    let right = Expression::literal(5i64);
    let expr = Expression::Binary {
        left: Box::new(left),
        op: BinaryOperator::Add,
        right: Box::new(right),
    };

    match &expr {
        Expression::Binary { op, .. } => assert!(matches!(op, BinaryOperator::Add)),
        _ => panic!("期望 Binary 表达式"),
    }
}

#[test]
fn test_expression_unary_creation() {
    use graphdb::core::types::UnaryOperator;

    let operand = Expression::literal(true);
    let expr = Expression::Unary {
        op: UnaryOperator::Not,
        operand: Box::new(operand),
    };

    match &expr {
        Expression::Unary { op, .. } => assert!(matches!(op, UnaryOperator::Not)),
        _ => panic!("期望 Unary 表达式"),
    }
}

// ==================== ExpressionEvaluator 测试 ====================

#[test]
fn test_evaluator_literal() {
    let mut ctx = BasicExpressionContext::new();
    
    // 整数
    let expr = Expression::literal(42i64);
    let result = ExpressionEvaluator::evaluate(&expr, &mut ctx).expect("整数字面量求值应该成功");
    assert_eq!(result, Value::Int(42));

    // 字符串
    let expr = Expression::literal("test".to_string());
    let result = ExpressionEvaluator::evaluate(&expr, &mut ctx).expect("字符串字面量求值应该成功");
    assert_eq!(result, Value::String("test".to_string()));

    // 布尔值
    let expr = Expression::literal(true);
    let result = ExpressionEvaluator::evaluate(&expr, &mut ctx).expect("布尔字面量求值应该成功");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_evaluator_variable() {
    let mut ctx = BasicExpressionContext::new();
    ctx.set_variable("x", Value::Int(100));
    ctx.set_variable("name", Value::String("Alice".to_string()));

    // 读取已设置变量
    let expr = Expression::variable("x");
    let result = ExpressionEvaluator::evaluate(&expr, &mut ctx).expect("变量求值应该成功");
    assert_eq!(result, Value::Int(100));

    let expr = Expression::variable("name");
    let result = ExpressionEvaluator::evaluate(&expr, &mut ctx).expect("变量求值应该成功");
    assert_eq!(result, Value::String("Alice".to_string()));
}

#[test]
fn test_evaluator_binary_arithmetic() {
    use graphdb::core::types::BinaryOperator;
    let mut ctx = BasicExpressionContext::new();

    // 加法: 10 + 5
    let expr = Expression::Binary {
        left: Box::new(Expression::literal(10i64)),
        op: BinaryOperator::Add,
        right: Box::new(Expression::literal(5i64)),
    };
    let result = ExpressionEvaluator::evaluate(&expr, &mut ctx).expect("二元加法求值应该成功");
    assert_eq!(result, Value::Int(15));

    // 减法: 20 - 8
    let expr = Expression::Binary {
        left: Box::new(Expression::literal(20i64)),
        op: BinaryOperator::Subtract,
        right: Box::new(Expression::literal(8i64)),
    };
    let result = ExpressionEvaluator::evaluate(&expr, &mut ctx).expect("二元减法求值应该成功");
    assert_eq!(result, Value::Int(12));

    // 乘法: 6 * 7
    let expr = Expression::Binary {
        left: Box::new(Expression::literal(6i64)),
        op: BinaryOperator::Multiply,
        right: Box::new(Expression::literal(7i64)),
    };
    let result = ExpressionEvaluator::evaluate(&expr, &mut ctx).expect("二元乘法求值应该成功");
    assert_eq!(result, Value::Int(42));

    // 除法: 20 / 4
    let expr = Expression::Binary {
        left: Box::new(Expression::literal(20i64)),
        op: BinaryOperator::Divide,
        right: Box::new(Expression::literal(4i64)),
    };
    let result = ExpressionEvaluator::evaluate(&expr, &mut ctx).expect("二元除法求值应该成功");
    assert_eq!(result, Value::Int(5));
}

#[test]
fn test_evaluator_binary_comparison() {
    use graphdb::core::types::BinaryOperator;
    let mut ctx = BasicExpressionContext::new();

    // 等于: 5 == 5
    let expr = Expression::Binary {
        left: Box::new(Expression::literal(5i64)),
        op: BinaryOperator::Equal,
        right: Box::new(Expression::literal(5i64)),
    };
    let result = ExpressionEvaluator::evaluate(&expr, &mut ctx).expect("二元相等比较求值应该成功");
    assert_eq!(result, Value::Bool(true));

    // 不等于: 5 != 3
    let expr = Expression::Binary {
        left: Box::new(Expression::literal(5i64)),
        op: BinaryOperator::NotEqual,
        right: Box::new(Expression::literal(3i64)),
    };
    let result = ExpressionEvaluator::evaluate(&expr, &mut ctx).expect("二元不等比较求值应该成功");
    assert_eq!(result, Value::Bool(true));

    // 大于: 10 > 5
    let expr = Expression::Binary {
        left: Box::new(Expression::literal(10i64)),
        op: BinaryOperator::GreaterThan,
        right: Box::new(Expression::literal(5i64)),
    };
    let result = ExpressionEvaluator::evaluate(&expr, &mut ctx).expect("二元大于比较求值应该成功");
    assert_eq!(result, Value::Bool(true));

    // 小于: 3 < 7
    let expr = Expression::Binary {
        left: Box::new(Expression::literal(3i64)),
        op: BinaryOperator::LessThan,
        right: Box::new(Expression::literal(7i64)),
    };
    let result = ExpressionEvaluator::evaluate(&expr, &mut ctx).expect("二元小于比较求值应该成功");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_evaluator_binary_logical() {
    use graphdb::core::types::BinaryOperator;
    let mut ctx = BasicExpressionContext::new();

    // AND: true && true
    let expr = Expression::Binary {
        left: Box::new(Expression::literal(true)),
        op: BinaryOperator::And,
        right: Box::new(Expression::literal(true)),
    };
    let result = ExpressionEvaluator::evaluate(&expr, &mut ctx).expect("二元AND逻辑求值应该成功");
    assert_eq!(result, Value::Bool(true));

    // AND: true && false
    let expr = Expression::Binary {
        left: Box::new(Expression::literal(true)),
        op: BinaryOperator::And,
        right: Box::new(Expression::literal(false)),
    };
    let result = ExpressionEvaluator::evaluate(&expr, &mut ctx).expect("二元AND逻辑求值应该成功");
    assert_eq!(result, Value::Bool(false));

    // OR: false || true
    let expr = Expression::Binary {
        left: Box::new(Expression::literal(false)),
        op: BinaryOperator::Or,
        right: Box::new(Expression::literal(true)),
    };
    let result = ExpressionEvaluator::evaluate(&expr, &mut ctx).expect("二元OR逻辑求值应该成功");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_evaluator_unary() {
    use graphdb::core::types::UnaryOperator;
    let mut ctx = BasicExpressionContext::new();

    // NOT: !true
    let expr = Expression::Unary {
        op: UnaryOperator::Not,
        operand: Box::new(Expression::literal(true)),
    };
    let result = ExpressionEvaluator::evaluate(&expr, &mut ctx).expect("一元NOT求值应该成功");
    assert_eq!(result, Value::Bool(false));

    // NOT: !false
    let expr = Expression::Unary {
        op: UnaryOperator::Not,
        operand: Box::new(Expression::literal(false)),
    };
    let result = ExpressionEvaluator::evaluate(&expr, &mut ctx).expect("一元NOT求值应该成功");
    assert_eq!(result, Value::Bool(true));

    // 负数: -42
    let expr = Expression::Unary {
        op: UnaryOperator::Minus,
        operand: Box::new(Expression::literal(42i64)),
    };
    let result = ExpressionEvaluator::evaluate(&expr, &mut ctx).expect("一元负号求值应该成功");
    assert_eq!(result, Value::Int(-42));
}

#[test]
fn test_evaluator_nested_expression() {
    use graphdb::core::types::BinaryOperator;
    let mut ctx = BasicExpressionContext::new();

    // (10 + 5) * 2 = 30
    let expr = Expression::Binary {
        left: Box::new(Expression::Binary {
            left: Box::new(Expression::literal(10i64)),
            op: BinaryOperator::Add,
            right: Box::new(Expression::literal(5i64)),
        }),
        op: BinaryOperator::Multiply,
        right: Box::new(Expression::literal(2i64)),
    };
    let result = ExpressionEvaluator::evaluate(&expr, &mut ctx).expect("嵌套表达式求值应该成功");
    assert_eq!(result, Value::Int(30));

    // 10 + (5 * 2) = 20
    let expr = Expression::Binary {
        left: Box::new(Expression::literal(10i64)),
        op: BinaryOperator::Add,
        right: Box::new(Expression::Binary {
            left: Box::new(Expression::literal(5i64)),
            op: BinaryOperator::Multiply,
            right: Box::new(Expression::literal(2i64)),
        }),
    };
    let result = ExpressionEvaluator::evaluate(&expr, &mut ctx).expect("嵌套表达式求值应该成功");
    assert_eq!(result, Value::Int(20));
}

#[test]
fn test_evaluator_batch_evaluation() {
    let mut ctx = BasicExpressionContext::new();

    let expressions = vec![
        Expression::literal(1i64),
        Expression::literal(2i64),
        Expression::literal(3i64),
    ];

    let results = ExpressionEvaluator::evaluate_batch(&expressions, &mut ctx).expect("批量表达式求值应该成功");
    assert_eq!(results.len(), 3);
    assert_eq!(results[0], Value::Int(1));
    assert_eq!(results[1], Value::Int(2));
    assert_eq!(results[2], Value::Int(3));
}

#[test]
fn test_evaluator_can_evaluate() {
    // 纯常量表达式可以求值
    let const_expr = Expression::Binary {
        left: Box::new(Expression::literal(10i64)),
        op: graphdb::core::types::BinaryOperator::Add,
        right: Box::new(Expression::literal(5i64)),
    };
    assert!(ExpressionEvaluator::can_evaluate(&const_expr));

    // 包含变量的表达式需要上下文
    let var_expr = Expression::variable("x");
    assert!(!ExpressionEvaluator::can_evaluate(&var_expr));
}

// ==================== Function Registry 测试 ====================

#[test]
fn test_function_registry_builtins() {
    let registry = global_registry();

    // 测试数学函数
    let result = registry.execute("abs", &[Value::Int(-42)]).expect("abs函数执行应该成功");
    assert_eq!(result, Value::Int(42));

    let result = registry.execute("abs", &[Value::Float(-3.14)]).expect("abs函数执行应该成功");
    assert_eq!(result, Value::Float(3.14));

    // 测试字符串函数
    let result = registry.execute("length", &[Value::String("hello".to_string())]).expect("length函数执行应该成功");
    assert_eq!(result, Value::Int(5));

    // 测试类型转换函数
    let result = registry.execute("to_int", &[Value::String("42".to_string())]).expect("to_int函数执行应该成功");
    assert_eq!(result, Value::Int(42));
}

#[test]
fn test_function_registry_errors() {
    let registry = global_registry();

    // 未定义的函数
    let result = registry.execute("undefined_func", &[Value::Int(1)]);
    assert!(result.is_err());

    // 参数数量不匹配
    let result = registry.execute("abs", &[]);
    assert!(result.is_err());
}

// ==================== ExpressionContext 测试 ====================

#[test]
fn test_basic_context_variables() {
    let mut ctx = BasicExpressionContext::new();

    // 设置变量
    ctx.set_variable("x", Value::Int(100));
    ctx.set_variable("y", Value::String("test".to_string()));

    // 获取变量
    assert_eq!(ctx.get_variable("x"), Some(Value::Int(100)));
    assert_eq!(ctx.get_variable("y"), Some(Value::String("test".to_string())));
    assert_eq!(ctx.get_variable("z"), None);
}

#[test]
fn test_basic_context_functions() {
    let ctx = BasicExpressionContext::new();

    // 通过全局注册表测试函数存在性和执行
    let registry = global_registry();
    
    // 测试 abs 函数
    let result = registry.execute("abs", &[Value::Int(-42)]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Int(42));

    // 测试 length 函数
    let result = registry.execute("length", &[Value::String("hello".to_string())]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Int(5));

    // 未定义的函数应该返回错误
    let result = registry.execute("undefined_func", &[Value::Int(1)]);
    assert!(result.is_err());
}

#[test]
fn test_basic_context_cache() {
    let mut ctx = BasicExpressionContext::new();

    // 设置和获取缓存值（通过变量模拟）
    ctx.set_variable("cached_key1", Value::Int(42));
    assert_eq!(ctx.get_variable("cached_key1"), Some(Value::Int(42)));
    assert_eq!(ctx.get_variable("nonexistent"), None);
}

#[test]
fn test_context_parent_child() {
    let mut parent = BasicExpressionContext::new();
    parent.set_variable("parent_var", Value::Int(100));

    let mut child = BasicExpressionContext::with_parent(parent);
    child.set_variable("child_var", Value::Int(200));

    // 子上下文应该能访问自己的变量
    assert_eq!(child.get_variable("child_var"), Some(Value::Int(200)));
}

// ==================== 复杂场景测试 ====================

#[test]
fn test_complex_arithmetic_expression() {
    use graphdb::core::types::BinaryOperator;
    let mut ctx = BasicExpressionContext::new();

    // 复杂表达式: (100 - 50) * 2 + 10 / 5 = 102
    let expr = Expression::Binary {
        left: Box::new(Expression::Binary {
            left: Box::new(Expression::Binary {
                left: Box::new(Expression::literal(100i64)),
                op: BinaryOperator::Subtract,
                right: Box::new(Expression::literal(50i64)),
            }),
            op: BinaryOperator::Multiply,
            right: Box::new(Expression::literal(2i64)),
        }),
        op: BinaryOperator::Add,
        right: Box::new(Expression::Binary {
            left: Box::new(Expression::literal(10i64)),
            op: BinaryOperator::Divide,
            right: Box::new(Expression::literal(5i64)),
        }),
    };

    let result = ExpressionEvaluator::evaluate(&expr, &mut ctx).expect("复杂表达式求值应该成功");
    assert_eq!(result, Value::Int(102));
}

#[test]
fn test_mixed_type_operations() {
    use graphdb::core::types::BinaryOperator;
    let mut ctx = BasicExpressionContext::new();

    // 整数和浮点数混合运算
    let expr = Expression::Binary {
        left: Box::new(Expression::literal(10i64)),
        op: BinaryOperator::Add,
        right: Box::new(Expression::literal(5.5f64)),
    };
    let result = ExpressionEvaluator::evaluate(&expr, &mut ctx).expect("混合类型操作求值应该成功");
    assert_eq!(result, Value::Float(15.5));
}

#[test]
fn test_string_concatenation() {
    use graphdb::core::types::BinaryOperator;
    let mut ctx = BasicExpressionContext::new();

    // 字符串连接
    let expr = Expression::Binary {
        left: Box::new(Expression::literal("Hello, ".to_string())),
        op: BinaryOperator::Add,
        right: Box::new(Expression::literal("World!".to_string())),
    };
    let result = ExpressionEvaluator::evaluate(&expr, &mut ctx).expect("字符串连接求值应该成功");
    assert_eq!(result, Value::String("Hello, World!".to_string()));
}

#[test]
fn test_list_operations() {
    // 创建列表值
    let list = Value::List(vec![
        Value::Int(1),
        Value::Int(2),
        Value::Int(3),
    ]);

    assert_eq!(list.length().expect("列表长度计算应该成功"), Value::Int(3));
    assert_eq!(list.get_type(), DataType::List);

    // 空列表
    let empty_list = Value::List(vec![]);
    assert_eq!(empty_list.length().expect("空列表长度计算应该成功"), Value::Int(0));
}

#[test]
fn test_map_operations() {
    use std::collections::HashMap;

    // 创建 Map 值
    let mut map = HashMap::new();
    map.insert("name".to_string(), Value::String("Alice".to_string()));
    map.insert("age".to_string(), Value::Int(30));
    let map_value = Value::Map(map);

    assert_eq!(map_value.length().expect("映射长度计算应该成功"), Value::Int(2));
    assert_eq!(map_value.get_type(), DataType::Map);
}

#[test]
fn test_value_memory_estimation() {
    // 测试内存估算功能
    let int_val = Value::Int(42);
    assert!(int_val.estimated_size() > 0);

    let string_val = Value::String("hello world".to_string());
    assert!(string_val.estimated_size() >= std::mem::size_of::<Value>() + "hello world".len());

    let list_val = Value::List(vec![Value::Int(1), Value::Int(2)]);
    assert!(list_val.estimated_size() > int_val.estimated_size());
}

#[test]
fn test_null_type_display() {
    assert_eq!(NullType::Null.to_string(), "NULL");
    assert_eq!(NullType::NaN.to_string(), "NaN");
    assert_eq!(NullType::BadData.to_string(), "BAD_DATA");
    assert_eq!(NullType::BadType.to_string(), "BAD_TYPE");
    assert_eq!(NullType::ErrOverflow.to_string(), "ERR_OVERFLOW");
    assert_eq!(NullType::UnknownProp.to_string(), "UNKNOWN_PROP");
    assert_eq!(NullType::DivByZero.to_string(), "DIV_BY_ZERO");
    assert_eq!(NullType::OutOfRange.to_string(), "OUT_OF_RANGE");
}

#[test]
fn test_default_values() {
    // 测试默认值
    let default_null: NullType = Default::default();
    assert_eq!(default_null, NullType::Null);

    let default_date: DateValue = Default::default();
    assert_eq!(default_date.year, 1970);
    assert_eq!(default_date.month, 1);
    assert_eq!(default_date.day, 1);

    let default_time: TimeValue = Default::default();
    assert_eq!(default_time.hour, 0);
    assert_eq!(default_time.minute, 0);

    let default_geo: GeographyValue = Default::default();
    assert_eq!(default_geo.latitude, 0.0);
    assert_eq!(default_geo.longitude, 0.0);
}
