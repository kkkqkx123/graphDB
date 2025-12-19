use super::error::ExpressionError;
use crate::core::Value;

/// 算术运算模块
/// 提供各种算术运算功能

/// 算术加法
pub fn arithmetic_add(left: &Value, right: &Value) -> Result<Value, ExpressionError> {
    match (left, right) {
        (Value::Int(l), Value::Int(r)) => Ok(Value::Int(l + r)),
        (Value::Float(l), Value::Float(r)) => Ok(Value::Float(l + r)),
        (Value::Int(l), Value::Float(r)) => Ok(Value::Float(*l as f64 + r)),
        (Value::Float(l), Value::Int(r)) => Ok(Value::Float(l + *r as f64)),
        (Value::String(l), Value::String(r)) => Ok(Value::String(l.clone() + r)),
        _ => Ok(Value::Null(crate::core::NullType::Null)),
    }
}

/// 算术减法
pub fn arithmetic_subtract(left: &Value, right: &Value) -> Result<Value, ExpressionError> {
    match (left, right) {
        (Value::Int(l), Value::Int(r)) => Ok(Value::Int(l - r)),
        (Value::Float(l), Value::Float(r)) => Ok(Value::Float(l - r)),
        (Value::Int(l), Value::Float(r)) => Ok(Value::Float(*l as f64 - r)),
        (Value::Float(l), Value::Int(r)) => Ok(Value::Float(l - *r as f64)),
        _ => Ok(Value::Null(crate::core::NullType::Null)),
    }
}

/// 算术乘法
pub fn arithmetic_multiply(left: &Value, right: &Value) -> Result<Value, ExpressionError> {
    match (left, right) {
        (Value::Int(l), Value::Int(r)) => Ok(Value::Int(l * r)),
        (Value::Float(l), Value::Float(r)) => Ok(Value::Float(l * r)),
        (Value::Int(l), Value::Float(r)) => Ok(Value::Float(*l as f64 * r)),
        (Value::Float(l), Value::Int(r)) => Ok(Value::Float(l * *r as f64)),
        _ => Ok(Value::Null(crate::core::NullType::Null)),
    }
}

/// 算术除法
pub fn arithmetic_divide(left: &Value, right: &Value) -> Result<Value, ExpressionError> {
    match (left, right) {
        (Value::Int(l), Value::Int(r)) => {
            if *r == 0 {
                Ok(Value::Null(crate::core::NullType::Null))
            } else {
                Ok(Value::Int(l / r))
            }
        }
        (Value::Float(l), Value::Float(r)) => {
            if *r == 0.0 {
                Ok(Value::Null(crate::core::NullType::Null))
            } else {
                Ok(Value::Float(l / r))
            }
        }
        (Value::Int(l), Value::Float(r)) => {
            if *r == 0.0 {
                Ok(Value::Null(crate::core::NullType::Null))
            } else {
                Ok(Value::Float(*l as f64 / r))
            }
        }
        (Value::Float(l), Value::Int(r)) => {
            if *r == 0 {
                Ok(Value::Null(crate::core::NullType::Null))
            } else {
                Ok(Value::Float(l / *r as f64))
            }
        }
        _ => Ok(Value::Null(crate::core::NullType::Null)),
    }
}

/// 算术取模
pub fn arithmetic_modulo(left: &Value, right: &Value) -> Result<Value, ExpressionError> {
    match (left, right) {
        (Value::Int(l), Value::Int(r)) => {
            if *r == 0 {
                Ok(Value::Null(crate::core::NullType::Null))
            } else {
                Ok(Value::Int(l % r))
            }
        }
        (Value::Float(l), Value::Float(r)) => {
            if *r == 0.0 {
                Ok(Value::Null(crate::core::NullType::Null))
            } else {
                Ok(Value::Float(l % r))
            }
        }
        _ => Ok(Value::Null(crate::core::NullType::Null)),
    }
}

/// 算术指数
pub fn arithmetic_exponent(left: &Value, right: &Value) -> Result<Value, ExpressionError> {
    match (left, right) {
        (Value::Int(l), Value::Int(r)) => Ok(Value::Float((*l as f64).powf(*r as f64))),
        (Value::Float(l), Value::Float(r)) => Ok(Value::Float(l.powf(*r))),
        (Value::Int(l), Value::Float(r)) => Ok(Value::Float((*l as f64).powf(*r))),
        (Value::Float(l), Value::Int(r)) => Ok(Value::Float(l.powf(*r as f64))),
        _ => Ok(Value::Null(crate::core::NullType::Null)),
    }
}