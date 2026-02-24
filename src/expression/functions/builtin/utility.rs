//! 实用函数实现

use crate::core::error::ExpressionError;
use crate::core::value::dataset::List;
use crate::core::value::NullType;
use crate::core::Value;
use serde_json::Value as JsonValue;

/// 实用函数枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UtilityFunction {
    Coalesce,
    Hash,
    JsonExtract,
}

impl UtilityFunction {
    pub fn name(&self) -> &str {
        match self {
            UtilityFunction::Coalesce => "coalesce",
            UtilityFunction::Hash => "hash",
            UtilityFunction::JsonExtract => "json_extract",
        }
    }

    pub fn arity(&self) -> usize {
        match self {
            UtilityFunction::Coalesce => 1,
            UtilityFunction::Hash => 1,
            UtilityFunction::JsonExtract => 2,
        }
    }

    pub fn is_variadic(&self) -> bool {
        matches!(self, UtilityFunction::Coalesce)
    }

    pub fn description(&self) -> &str {
        match self {
            UtilityFunction::Coalesce => "返回第一个非NULL值",
            UtilityFunction::Hash => "计算哈希值",
            UtilityFunction::JsonExtract => "从JSON字符串中提取指定路径的值",
        }
    }

    pub fn execute(&self, args: &[Value]) -> Result<Value, ExpressionError> {
        match self {
            UtilityFunction::Coalesce => execute_coalesce(args),
            UtilityFunction::Hash => execute_hash(args),
            UtilityFunction::JsonExtract => execute_json_extract(args),
        }
    }
}

fn execute_coalesce(args: &[Value]) -> Result<Value, ExpressionError> {
    for arg in args {
        match arg {
            Value::Null(_) => continue,
            other => return Ok(other.clone()),
        }
    }
    Ok(Value::Null(NullType::Null))
}

fn execute_hash(args: &[Value]) -> Result<Value, ExpressionError> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    match &args[0] {
        Value::Null(_) => Ok(Value::Null(NullType::Null)),
        Value::String(s) => {
            let mut hasher = DefaultHasher::new();
            s.hash(&mut hasher);
            let hash_value = hasher.finish() as i64;
            Ok(Value::Int(hash_value))
        }
        Value::Int(i) => {
            let mut hasher = DefaultHasher::new();
            i.hash(&mut hasher);
            let hash_value = hasher.finish() as i64;
            Ok(Value::Int(hash_value))
        }
        _ => Err(ExpressionError::type_error("hash函数需要字符串或整数类型")),
    }
}

fn execute_json_extract(args: &[Value]) -> Result<Value, ExpressionError> {
    match (&args[0], &args[1]) {
        (Value::String(json_str), Value::String(path)) => {
            let json_value: JsonValue = serde_json::from_str(json_str)
                .map_err(|_| ExpressionError::type_error("无效的JSON字符串"))?;

            let result = extract_json_value(&json_value, path);
            Ok(json_to_value(result))
        }
        (Value::Null(_), _) | (_, Value::Null(_)) => Ok(Value::Null(NullType::Null)),
        _ => Err(ExpressionError::type_error("json_extract函数需要字符串参数")),
    }
}

fn extract_json_value<'a>(json: &'a JsonValue, path: &str) -> &'a JsonValue {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = json;

    for part in parts {
        if part.is_empty() {
            continue;
        }

        current = match current {
            JsonValue::Object(map) => map.get(part).unwrap_or(&JsonValue::Null),
            JsonValue::Array(arr) => {
                if let Ok(index) = part.parse::<usize>() {
                    arr.get(index).unwrap_or(&JsonValue::Null)
                } else {
                    &JsonValue::Null
                }
            }
            _ => &JsonValue::Null,
        };
    }

    current
}

fn json_to_value(json: &JsonValue) -> Value {
    match json {
        JsonValue::Null => Value::Null(NullType::Null),
        JsonValue::Bool(b) => Value::Bool(*b),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else if let Some(f) = n.as_f64() {
                Value::Float(f)
            } else {
                Value::Null(NullType::Null)
            }
        }
        JsonValue::String(s) => Value::String(s.clone()),
        JsonValue::Array(arr) => {
            let values: Vec<Value> = arr.iter().map(json_to_value).collect();
            Value::List(List { values })
        }
        JsonValue::Object(obj) => {
            let map: std::collections::HashMap<String, Value> = obj
                .iter()
                .map(|(k, v)| (k.clone(), json_to_value(v)))
                .collect();
            Value::Map(map)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coalesce() {
        let func = UtilityFunction::Coalesce;
        let result = func
            .execute(&[
                Value::Null(NullType::Null),
                Value::Int(42),
                Value::Int(100),
            ])
            .unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_hash() {
        let func = UtilityFunction::Hash;
        let result = func
            .execute(&[Value::String("test".to_string())])
            .unwrap();
        assert!(matches!(result, Value::Int(_)));
    }
}
