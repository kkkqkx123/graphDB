//! 正则表达式函数实现

use crate::core::error::{ExpressionError, ExpressionErrorType};
use crate::core::Value;
use crate::expression::functions::registry::FunctionRegistry;
use crate::expression::functions::signature::FunctionSignature;
use crate::expression::functions::signature::ValueType;

/// 注册所有正则表达式函数
pub fn register_all(registry: &mut FunctionRegistry) {
    register_regex_match(registry);
    register_regex_replace(registry);
    register_regex_find(registry);
}

fn register_regex_match(registry: &mut FunctionRegistry) {
    registry.register(
        "regex_match",
        FunctionSignature::new(
            "regex_match",
            vec![ValueType::String, ValueType::String],
            ValueType::Bool,
            2,
            2,
            true,
            "正则表达式匹配",
        ),
        |args| {
            match (&args[0], &args[1]) {
                (Value::String(s), Value::String(pattern)) => {
                    let regex = regex::Regex::new(pattern)
                        .map_err(|e| ExpressionError::new(
                            ExpressionErrorType::InvalidOperation,
                            format!("无效的正则表达式: {}", e),
                        ))?;
                    Ok(Value::Bool(regex.is_match(s)))
                }
                (Value::Null(_), _) | (_, Value::Null(_)) => {
                    Ok(Value::Null(crate::core::value::NullType::Null))
                }
                _ => Err(ExpressionError::type_error("regex_match函数需要字符串类型")),
            }
        },
    );
}

fn register_regex_replace(registry: &mut FunctionRegistry) {
    registry.register(
        "regex_replace",
        FunctionSignature::new(
            "regex_replace",
            vec![ValueType::String, ValueType::String, ValueType::String],
            ValueType::String,
            3,
            3,
            true,
            "正则表达式替换",
        ),
        |args| {
            match (&args[0], &args[1], &args[2]) {
                (Value::String(s), Value::String(pattern), Value::String(replacement)) => {
                    let regex = regex::Regex::new(pattern)
                        .map_err(|e| ExpressionError::new(
                            ExpressionErrorType::InvalidOperation,
                            format!("无效的正则表达式: {}", e),
                        ))?;
                    Ok(Value::String(regex.replace_all(s, replacement.as_str()).to_string()))
                }
                (Value::Null(_), _, _) | (_, Value::Null(_), _) | (_, _, Value::Null(_)) => {
                    Ok(Value::Null(crate::core::value::NullType::Null))
                }
                _ => Err(ExpressionError::type_error("regex_replace函数需要字符串类型")),
            }
        },
    );
}

fn register_regex_find(registry: &mut FunctionRegistry) {
    registry.register(
        "regex_find",
        FunctionSignature::new(
            "regex_find",
            vec![ValueType::String, ValueType::String],
            ValueType::String,
            2,
            2,
            true,
            "正则表达式查找",
        ),
        |args| {
            match (&args[0], &args[1]) {
                (Value::String(s), Value::String(pattern)) => {
                    let regex = regex::Regex::new(pattern)
                        .map_err(|e| ExpressionError::new(
                            ExpressionErrorType::InvalidOperation,
                            format!("无效的正则表达式: {}", e),
                        ))?;
                    if let Some(matched) = regex.find(s) {
                        Ok(Value::String(matched.as_str().to_string()))
                    } else {
                        Ok(Value::Null(crate::core::value::NullType::Null))
                    }
                }
                (Value::Null(_), _) | (_, Value::Null(_)) => {
                    Ok(Value::Null(crate::core::value::NullType::Null))
                }
                _ => Err(ExpressionError::type_error("regex_find函数需要字符串类型")),
            }
        },
    );
}
