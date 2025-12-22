use crate::core::{ExpressionError, Value};

/// 比较和字符串操作模块
/// 提供值比较和字符串操作功能

/// 比较两个值是否相等
pub fn values_equal(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::String(l), Value::String(r)) => l == r,
        (Value::Int(l), Value::Int(r)) => l == r,
        (Value::Float(l), Value::Float(r)) => (l - r).abs() < f64::EPSILON,
        (Value::Bool(l), Value::Bool(r)) => l == r,
        (Value::Null(_), Value::Null(_)) => true,
        (Value::List(l), Value::List(r)) => {
            l.len() == r.len() && l.iter().zip(r.iter()).all(|(a, b)| values_equal(a, b))
        }
        (Value::Map(l), Value::Map(r)) => {
            l.len() == r.len()
                && l.iter()
                    .all(|(k, v)| r.get(k).map_or(false, |rv| values_equal(v, rv)))
        }
        _ => false,
    }
}

/// 比较两个值的大小
pub fn compare_values(left: &Value, right: &Value) -> i32 {
    match (left, right) {
        (Value::String(l), Value::String(r)) => match l.cmp(r) {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
        },
        (Value::Int(l), Value::Int(r)) => match l.cmp(r) {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
        },
        (Value::Float(l), Value::Float(r)) => {
            match l.partial_cmp(r).unwrap_or(std::cmp::Ordering::Equal) {
                std::cmp::Ordering::Less => -1,
                std::cmp::Ordering::Equal => 0,
                std::cmp::Ordering::Greater => 1,
            }
        }
        (Value::Bool(l), Value::Bool(r)) => match l.cmp(r) {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
        },
        _ => 0, // 无法比较的类型返回相等
    }
}

/// 检查IN操作
pub fn check_in(left: &Value, right: &Value) -> Result<Value, ExpressionError> {
    match right {
        Value::List(list) => Ok(Value::Bool(
            list.iter().any(|item| values_equal(left, item)),
        )),
        _ => Ok(Value::Bool(false)),
    }
}

/// 检查STARTS WITH操作
pub fn check_starts_with(left: &Value, right: &Value) -> Result<Value, ExpressionError> {
    match (left, right) {
        (Value::String(l), Value::String(r)) => Ok(Value::Bool(l.starts_with(r))),
        _ => Ok(Value::Bool(false)),
    }
}

/// 检查ENDS WITH操作
pub fn check_ends_with(left: &Value, right: &Value) -> Result<Value, ExpressionError> {
    match (left, right) {
        (Value::String(l), Value::String(r)) => Ok(Value::Bool(l.ends_with(r))),
        _ => Ok(Value::Bool(false)),
    }
}

/// 检查CONTAINS操作
pub fn check_contains(left: &Value, right: &Value) -> Result<Value, ExpressionError> {
    match (left, right) {
        (Value::String(l), Value::String(r)) => Ok(Value::Bool(l.contains(r))),
        (Value::List(list), _) => Ok(Value::Bool(
            list.iter().any(|item| values_equal(item, right)),
        )),
        _ => Ok(Value::Bool(false)),
    }
}

/// 将值转换为布尔值
pub fn value_to_bool(value: &Value) -> bool {
    match value {
        Value::Bool(b) => *b,
        Value::Null(_) => false,
        Value::String(s) => !s.is_empty(),
        Value::Int(i) => *i != 0,
        Value::Float(f) => *f != 0.0,
        Value::List(l) => !l.is_empty(),
        Value::Map(m) => !m.is_empty(),
        _ => false,
    }
}
