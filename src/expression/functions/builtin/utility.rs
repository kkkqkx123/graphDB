//! 实用函数实现
//!
//! 提供实用工具函数，包括 coalesce, hash, json_extract

use crate::core::error::ExpressionError;
use crate::core::value::dataset::List;
use crate::core::value::NullType;
use crate::core::Value;
use crate::expression::functions::registry::FunctionRegistry;
use crate::expression::functions::signature::FunctionSignature;
use crate::expression::functions::signature::ValueType;
use serde_json::Value as JsonValue;

/// 注册所有实用函数
pub fn register_all(registry: &mut FunctionRegistry) {
    register_coalesce(registry);
    register_hash(registry);
    register_json_extract(registry);
}

fn register_coalesce(registry: &mut FunctionRegistry) {
    registry.register(
        "coalesce",
        FunctionSignature::new(
            "coalesce",
            vec![ValueType::Any],
            ValueType::Any,
            1,
            usize::MAX,
            true,
            "返回第一个非NULL值",
        ),
        |args| {
            for arg in args {
                match arg {
                    Value::Null(_) => continue,
                    other => return Ok(other.clone()),
                }
            }
            Ok(Value::Null(NullType::Null))
        },
    );
}

fn register_hash(registry: &mut FunctionRegistry) {
    registry.register(
        "hash",
        FunctionSignature::new(
            "hash",
            vec![ValueType::Null],
            ValueType::Null,
            1,
            1,
            true,
            "计算哈希值",
        ),
        |args| {
            match &args[0] {
                Value::Null(_) => Ok(Value::Null(NullType::Null)),
                _ => Ok(Value::Null(NullType::Null)),
            }
        },
    );

    registry.register(
        "hash",
        FunctionSignature::new(
            "hash",
            vec![ValueType::String],
            ValueType::Int,
            1,
            1,
            true,
            "计算字符串哈希值",
        ),
        |args| {
            match &args[0] {
                Value::String(s) => {
                    use std::collections::hash_map::DefaultHasher;
                    use std::hash::{Hash, Hasher};
                    let mut hasher = DefaultHasher::new();
                    s.hash(&mut hasher);
                    let hash_value = hasher.finish() as i64;
                    Ok(Value::Int(hash_value))
                }
                Value::Null(_) => Ok(Value::Null(NullType::Null)),
                _ => Err(ExpressionError::type_error("hash函数需要字符串类型")),
            }
        },
    );

    registry.register(
        "hash",
        FunctionSignature::new(
            "hash",
            vec![ValueType::Int],
            ValueType::Int,
            1,
            1,
            true,
            "计算整数哈希值",
        ),
        |args| {
            match &args[0] {
                Value::Int(i) => {
                    use std::collections::hash_map::DefaultHasher;
                    use std::hash::{Hash, Hasher};
                    let mut hasher = DefaultHasher::new();
                    i.hash(&mut hasher);
                    let hash_value = hasher.finish() as i64;
                    Ok(Value::Int(hash_value))
                }
                Value::Null(_) => Ok(Value::Null(NullType::Null)),
                _ => Err(ExpressionError::type_error("hash函数需要整数类型")),
            }
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_registry() -> FunctionRegistry {
        let mut registry = FunctionRegistry::new();
        register_all(&mut registry);
        registry
    }

    #[test]
    fn test_coalesce_first_non_null() {
        let registry = create_test_registry();
        let result = registry
            .execute(
                "coalesce",
                &[Value::Null(NullType::Null), Value::Int(1), Value::Int(2)],
            )
            .expect("coalesce函数执行应该成功");
        assert_eq!(result, Value::Int(1));
    }

    #[test]
    fn test_coalesce_all_null() {
        let registry = create_test_registry();
        let result = registry
            .execute(
                "coalesce",
                &[Value::Null(NullType::Null), Value::Null(NullType::Null)],
            )
            .expect("coalesce函数执行应该成功");
        assert_eq!(result, Value::Null(NullType::Null));
    }

    #[test]
    fn test_coalesce_single_value() {
        let registry = create_test_registry();
        let result = registry
            .execute("coalesce", &[Value::String("hello".to_string())])
            .expect("coalesce函数执行应该成功");
        assert_eq!(result, Value::String("hello".to_string()));
    }

    #[test]
    fn test_hash_string() {
        let registry = create_test_registry();
        let result = registry
            .execute("hash", &[Value::String("hello".to_string())])
            .expect("hash函数执行应该成功");
        if let Value::Int(hash) = result {
            assert_ne!(hash, 0);
        } else {
            panic!("hash函数应该返回整数");
        }
    }

    #[test]
    fn test_hash_int() {
        let registry = create_test_registry();
        let result = registry
            .execute("hash", &[Value::Int(42)])
            .expect("hash函数执行应该成功");
        if let Value::Int(hash) = result {
            assert_ne!(hash, 0);
        } else {
            panic!("hash函数应该返回整数");
        }
    }

    #[test]
    fn test_hash_consistency() {
        let registry = create_test_registry();
        let result1 = registry
            .execute("hash", &[Value::String("test".to_string())])
            .expect("hash函数执行应该成功");
        let result2 = registry
            .execute("hash", &[Value::String("test".to_string())])
            .expect("hash函数执行应该成功");
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_hash_null() {
        let registry = create_test_registry();
        let result = registry
            .execute("hash", &[Value::Null(NullType::Null)])
            .expect("hash函数应该处理NULL");
        assert_eq!(result, Value::Null(NullType::Null));
    }
}

fn register_json_extract(registry: &mut FunctionRegistry) {
    registry.register(
        "json_extract",
        FunctionSignature::new(
            "json_extract",
            vec![ValueType::String, ValueType::String],
            ValueType::Any,
            2,
            2,
            true,
            "从JSON字符串中提取指定路径的值",
        ),
        |args| {
            match (&args[0], &args[1]) {
                (Value::String(json_str), Value::String(path)) => {
                    let json_value: JsonValue = serde_json::from_str(json_str)
                        .map_err(|_| ExpressionError::type_error("无效的JSON字符串"))?;

                    let result = extract_json_value(&json_value, path);
                    Ok(json_to_value(result))
                }
                (Value::Null(_), _) | (_, Value::Null(_)) => {
                    Ok(Value::Null(NullType::Null))
                }
                _ => Err(ExpressionError::type_error("json_extract函数需要字符串参数")),
            }
        },
    );
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
