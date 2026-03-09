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
        Value::Null(_) => Ok(Value::Null(NullType::Null)),
        _ => {
            args[0]
                .to_string()
                .map(Value::String)
                .map_err(ExpressionError::type_error)
        }
    }
}

fn execute_to_int(args: &[Value]) -> Result<Value, ExpressionError> {
    match &args[0] {
        Value::Null(_) => Ok(Value::Null(NullType::Null)),
        _ => {
            let result = args[0].to_int();
            if let Value::Null(NullType::BadData) = result {
                Err(ExpressionError::type_error("to_int函数不支持该类型"))
            } else {
                Ok(result)
            }
        }
    }
}

fn execute_to_float(args: &[Value]) -> Result<Value, ExpressionError> {
    match &args[0] {
        Value::Null(_) => Ok(Value::Null(NullType::Null)),
        _ => {
            let result = args[0].to_float();
            if let Value::Null(NullType::BadData) = result {
                Err(ExpressionError::type_error("to_float函数不支持该类型"))
            } else {
                Ok(result)
            }
        }
    }
}

fn execute_to_bool(args: &[Value]) -> Result<Value, ExpressionError> {
    match &args[0] {
        Value::Null(_) => Ok(Value::Null(NullType::Null)),
        _ => {
            let result = args[0].to_bool();
            if let Value::Null(NullType::BadData) = result {
                Err(ExpressionError::type_error("to_bool函数不支持该类型"))
            } else {
                Ok(result)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_string() {
        let func = ConversionFunction::ToString;
        let result = func.execute(&[Value::Int(42)]).expect("执行不应失败");
        assert_eq!(result, Value::String("42".to_string()));
    }

    #[test]
    fn test_to_int() {
        let func = ConversionFunction::ToInt;
        let result = func.execute(&[Value::String("42".to_string())]).expect("执行不应失败");
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_to_float() {
        let func = ConversionFunction::ToFloat;
        let result = func.execute(&[Value::Int(42)]).expect("执行不应失败");
        assert_eq!(result, Value::Float(42.0));
    }

    #[test]
    fn test_to_bool() {
        let func = ConversionFunction::ToBool;
        let result = func.execute(&[Value::Int(1)]).expect("执行不应失败");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_null_handling() {
        let func = ConversionFunction::ToString;
        let result = func.execute(&[Value::Null(NullType::Null)]).expect("执行不应失败");
        assert_eq!(result, Value::Null(NullType::Null));
    }
}
