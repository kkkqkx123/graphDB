//! 类型转换函数实现

use crate::core::error::ExpressionError;
use crate::core::value::NullType;
use crate::core::Value;
use crate::define_function_enum;

define_function_enum! {
    /// 类型转换函数枚举
    pub enum ConversionFunction {
        ToString => {
            name: "to_string",
            arity: 1,
            variadic: false,
            description: "转换为字符串",
            handler: execute_to_string
        },
        ToInt => {
            name: "to_int",
            arity: 1,
            variadic: false,
            description: "转换为整数",
            handler: execute_to_int
        },
        ToFloat => {
            name: "to_float",
            arity: 1,
            variadic: false,
            description: "转换为浮点数",
            handler: execute_to_float
        },
        ToBool => {
            name: "to_bool",
            arity: 1,
            variadic: false,
            description: "转换为布尔值",
            handler: execute_to_bool
        },
    }
}

fn execute_to_string(args: &[Value]) -> Result<Value, ExpressionError> {
    match &args[0] {
        Value::String(s) => Ok(Value::String(s.clone())),
        Value::Int(i) => Ok(Value::String(i.to_string())),
        Value::Float(f) => Ok(Value::String(f.to_string())),
        Value::Bool(b) => Ok(Value::String(b.to_string())),
        Value::Null(_) => Ok(Value::Null(NullType::Null)),
        _ => Err(ExpressionError::type_error("to_string函数不支持该类型")),
    }
}

fn execute_to_int(args: &[Value]) -> Result<Value, ExpressionError> {
    match &args[0] {
        Value::Int(i) => Ok(Value::Int(*i)),
        Value::Float(f) => Ok(Value::Int(*f as i64)),
        Value::String(s) => s.parse::<i64>()
            .map(Value::Int)
            .map_err(|_| ExpressionError::type_error("无法将字符串转换为整数")),
        Value::Bool(b) => Ok(Value::Int(if *b { 1 } else { 0 })),
        Value::Null(_) => Ok(Value::Null(NullType::Null)),
        _ => Err(ExpressionError::type_error("to_int函数不支持该类型")),
    }
}

fn execute_to_float(args: &[Value]) -> Result<Value, ExpressionError> {
    match &args[0] {
        Value::Int(i) => Ok(Value::Float(*i as f64)),
        Value::Float(f) => Ok(Value::Float(*f)),
        Value::String(s) => s.parse::<f64>()
            .map(Value::Float)
            .map_err(|_| ExpressionError::type_error("无法将字符串转换为浮点数")),
        Value::Bool(b) => Ok(Value::Float(if *b { 1.0 } else { 0.0 })),
        Value::Null(_) => Ok(Value::Null(NullType::Null)),
        _ => Err(ExpressionError::type_error("to_float函数不支持该类型")),
    }
}

fn execute_to_bool(args: &[Value]) -> Result<Value, ExpressionError> {
    match &args[0] {
        Value::Bool(b) => Ok(Value::Bool(*b)),
        Value::Int(i) => Ok(Value::Bool(*i != 0)),
        Value::Float(f) => Ok(Value::Bool(*f != 0.0)),
        Value::String(s) => Ok(Value::Bool(!s.is_empty())),
        Value::Null(_) => Ok(Value::Null(NullType::Null)),
        _ => Err(ExpressionError::type_error("to_bool函数不支持该类型")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_string() {
        let func = ConversionFunction::ToString;
        let result = func.execute(&[Value::Int(42)]).unwrap();
        assert_eq!(result, Value::String("42".to_string()));
    }

    #[test]
    fn test_to_int() {
        let func = ConversionFunction::ToInt;
        let result = func.execute(&[Value::String("42".to_string())]).unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_to_float() {
        let func = ConversionFunction::ToFloat;
        let result = func.execute(&[Value::Int(42)]).unwrap();
        assert_eq!(result, Value::Float(42.0));
    }

    #[test]
    fn test_to_bool() {
        let func = ConversionFunction::ToBool;
        let result = func.execute(&[Value::Int(1)]).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_null_handling() {
        let func = ConversionFunction::ToString;
        let result = func.execute(&[Value::Null(NullType::Null)]).unwrap();
        assert_eq!(result, Value::Null(NullType::Null));
    }
}
