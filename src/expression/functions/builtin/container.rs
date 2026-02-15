//! 容器操作函数实现
//!
//! 提供列表和映射的操作函数，包括 head, last, tail, size, range, keys

use crate::core::error::ExpressionError;
use crate::core::value::dataset::List;
use crate::core::value::NullType;
use crate::core::Value;
use crate::expression::functions::registry::FunctionRegistry;
use crate::expression::functions::signature::FunctionSignature;
use crate::expression::functions::signature::ValueType;
use std::collections::BTreeSet;

/// 注册所有容器操作函数
pub fn register_all(registry: &mut FunctionRegistry) {
    register_head(registry);
    register_last(registry);
    register_tail(registry);
    register_size(registry);
    register_range(registry);
    register_keys(registry);
}

fn register_head(registry: &mut FunctionRegistry) {
    registry.register(
        "head",
        FunctionSignature::new(
            "head",
            vec![ValueType::Null],
            ValueType::Null,
            1,
            1,
            true,
            "获取列表首元素",
        ),
        |args| {
            match &args[0] {
                Value::Null(_) => Ok(Value::Null(NullType::Null)),
                _ => Ok(Value::Null(NullType::Null)),
            }
        },
    );

    registry.register(
        "head",
        FunctionSignature::new(
            "head",
            vec![ValueType::List],
            ValueType::Any,
            1,
            1,
            true,
            "获取列表首元素",
        ),
        |args| {
            match &args[0] {
                Value::List(list) => Ok(list.values.first().cloned().unwrap_or(Value::Null(NullType::Null))),
                Value::Null(_) => Ok(Value::Null(NullType::Null)),
                _ => Err(ExpressionError::type_error("head函数需要列表类型")),
            }
        },
    );
}

fn register_last(registry: &mut FunctionRegistry) {
    registry.register(
        "last",
        FunctionSignature::new(
            "last",
            vec![ValueType::Null],
            ValueType::Null,
            1,
            1,
            true,
            "获取列表末元素",
        ),
        |args| {
            match &args[0] {
                Value::Null(_) => Ok(Value::Null(NullType::Null)),
                _ => Ok(Value::Null(NullType::Null)),
            }
        },
    );

    registry.register(
        "last",
        FunctionSignature::new(
            "last",
            vec![ValueType::List],
            ValueType::Any,
            1,
            1,
            true,
            "获取列表末元素",
        ),
        |args| {
            match &args[0] {
                Value::List(list) => Ok(list.values.last().cloned().unwrap_or(Value::Null(NullType::Null))),
                Value::Null(_) => Ok(Value::Null(NullType::Null)),
                _ => Err(ExpressionError::type_error("last函数需要列表类型")),
            }
        },
    );
}

fn register_tail(registry: &mut FunctionRegistry) {
    registry.register(
        "tail",
        FunctionSignature::new(
            "tail",
            vec![ValueType::Null],
            ValueType::Null,
            1,
            1,
            true,
            "获取列表尾部（除首元素外）",
        ),
        |args| {
            match &args[0] {
                Value::Null(_) => Ok(Value::Null(NullType::Null)),
                _ => Ok(Value::Null(NullType::Null)),
            }
        },
    );

    registry.register(
        "tail",
        FunctionSignature::new(
            "tail",
            vec![ValueType::List],
            ValueType::List,
            1,
            1,
            true,
            "获取列表尾部（除首元素外）",
        ),
        |args| {
            match &args[0] {
                Value::List(list) => {
                    if list.values.is_empty() {
                        Ok(Value::List(List { values: vec![] }))
                    } else {
                        Ok(Value::List(List { values: list.values[1..].to_vec() }))
                    }
                }
                Value::Null(_) => Ok(Value::Null(NullType::Null)),
                _ => Err(ExpressionError::type_error("tail函数需要列表类型")),
            }
        },
    );
}

fn register_size(registry: &mut FunctionRegistry) {
    registry.register(
        "size",
        FunctionSignature::new(
            "size",
            vec![ValueType::Null],
            ValueType::Null,
            1,
            1,
            true,
            "获取大小",
        ),
        |args| {
            match &args[0] {
                Value::Null(_) => Ok(Value::Null(NullType::Null)),
                _ => Ok(Value::Null(NullType::Null)),
            }
        },
    );

    registry.register(
        "size",
        FunctionSignature::new(
            "size",
            vec![ValueType::String],
            ValueType::Int,
            1,
            1,
            true,
            "获取字符串长度",
        ),
        |args| {
            match &args[0] {
                Value::String(s) => Ok(Value::Int(s.len() as i64)),
                Value::Null(_) => Ok(Value::Null(NullType::Null)),
                _ => Err(ExpressionError::type_error("size函数需要字符串类型")),
            }
        },
    );

    registry.register(
        "size",
        FunctionSignature::new(
            "size",
            vec![ValueType::List],
            ValueType::Int,
            1,
            1,
            true,
            "获取列表大小",
        ),
        |args| {
            match &args[0] {
                Value::List(list) => Ok(Value::Int(list.values.len() as i64)),
                Value::Null(_) => Ok(Value::Null(NullType::Null)),
                _ => Err(ExpressionError::type_error("size函数需要列表类型")),
            }
        },
    );

    registry.register(
        "size",
        FunctionSignature::new(
            "size",
            vec![ValueType::Map],
            ValueType::Int,
            1,
            1,
            true,
            "获取映射大小",
        ),
        |args| {
            match &args[0] {
                Value::Map(map) => Ok(Value::Int(map.len() as i64)),
                Value::Null(_) => Ok(Value::Null(NullType::Null)),
                _ => Err(ExpressionError::type_error("size函数需要映射类型")),
            }
        },
    );

    registry.register(
        "size",
        FunctionSignature::new(
            "size",
            vec![ValueType::Set],
            ValueType::Int,
            1,
            1,
            true,
            "获取集合大小",
        ),
        |args| {
            match &args[0] {
                Value::Set(set) => Ok(Value::Int(set.len() as i64)),
                Value::Null(_) => Ok(Value::Null(NullType::Null)),
                _ => Err(ExpressionError::type_error("size函数需要集合类型")),
            }
        },
    );
}

fn register_range(registry: &mut FunctionRegistry) {
    registry.register(
        "range",
        FunctionSignature::new(
            "range",
            vec![ValueType::Int, ValueType::Int],
            ValueType::List,
            2,
            3,
            true,
            "生成范围列表",
        ),
        |args| {
            let start = match &args[0] {
                Value::Int(i) => *i,
                Value::Null(_) => return Ok(Value::Null(NullType::Null)),
                _ => return Err(ExpressionError::type_error("range函数需要整数参数")),
            };
            let end = match &args[1] {
                Value::Int(i) => *i,
                Value::Null(_) => return Ok(Value::Null(NullType::Null)),
                _ => return Err(ExpressionError::type_error("range函数需要整数参数")),
            };
            let step = if args.len() > 2 {
                match &args[2] {
                    Value::Int(i) => *i,
                    Value::Null(_) => return Ok(Value::Null(NullType::Null)),
                    _ => return Err(ExpressionError::type_error("range函数的step需要整数")),
                }
            } else {
                1
            };

            if step == 0 {
                return Err(ExpressionError::new(
                    crate::core::error::ExpressionErrorType::InvalidOperation,
                    "range函数的step不能为0".to_string(),
                ));
            }

            let mut result = Vec::new();
            if step > 0 {
                let mut i = start;
                while i <= end {
                    result.push(Value::Int(i));
                    i += step;
                }
            } else {
                let mut i = start;
                while i >= end {
                    result.push(Value::Int(i));
                    i += step;
                }
            }

            Ok(Value::List(List { values: result }))
        },
    );
}

fn register_keys(registry: &mut FunctionRegistry) {
    registry.register(
        "keys",
        FunctionSignature::new(
            "keys",
            vec![ValueType::Vertex],
            ValueType::List,
            1,
            1,
            true,
            "获取顶点属性键列表",
        ),
        |args| {
            let mut keys: BTreeSet<String> = BTreeSet::new();

            match &args[0] {
                Value::Vertex(v) => {
                    for tag in &v.tags {
                        for key in tag.properties.keys() {
                            keys.insert(key.clone());
                        }
                    }
                    for key in v.properties.keys() {
                        keys.insert(key.clone());
                    }
                }
                Value::Null(_) => return Ok(Value::Null(NullType::Null)),
                _ => return Err(ExpressionError::type_error("keys函数需要顶点类型")),
            }

            let result: Vec<Value> = keys.into_iter().map(Value::String).collect();
            Ok(Value::List(List { values: result }))
        },
    );

    registry.register(
        "keys",
        FunctionSignature::new(
            "keys",
            vec![ValueType::Edge],
            ValueType::List,
            1,
            1,
            true,
            "获取边属性键列表",
        ),
        |args| {
            let mut keys: BTreeSet<String> = BTreeSet::new();

            match &args[0] {
                Value::Edge(e) => {
                    for key in e.props.keys() {
                        keys.insert(key.clone());
                    }
                }
                Value::Null(_) => return Ok(Value::Null(NullType::Null)),
                _ => return Err(ExpressionError::type_error("keys函数需要边类型")),
            }

            let result: Vec<Value> = keys.into_iter().map(Value::String).collect();
            Ok(Value::List(List { values: result }))
        },
    );

    registry.register(
        "keys",
        FunctionSignature::new(
            "keys",
            vec![ValueType::Map],
            ValueType::List,
            1,
            1,
            true,
            "获取映射键列表",
        ),
        |args| {
            let mut keys: BTreeSet<String> = BTreeSet::new();

            match &args[0] {
                Value::Map(m) => {
                    for key in m.keys() {
                        keys.insert(key.clone());
                    }
                }
                Value::Null(_) => return Ok(Value::Null(NullType::Null)),
                _ => return Err(ExpressionError::type_error("keys函数需要映射类型")),
            }

            let result: Vec<Value> = keys.into_iter().map(Value::String).collect();
            Ok(Value::List(List { values: result }))
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_registry() -> FunctionRegistry {
        let mut registry = FunctionRegistry::new();
        register_all(&mut registry);
        registry
    }

    #[test]
    fn test_head_function() {
        let registry = create_test_registry();
        let list = Value::List(List { values: vec![Value::Int(1), Value::Int(2), Value::Int(3)] });
        let result = registry.execute("head", &[list]).expect("head函数执行应该成功");
        assert_eq!(result, Value::Int(1));
    }

    #[test]
    fn test_head_empty_list() {
        let registry = create_test_registry();
        let list = Value::List(List { values: vec![] });
        let result = registry.execute("head", &[list]).expect("head函数执行应该成功");
        assert_eq!(result, Value::Null(NullType::Null));
    }

    #[test]
    fn test_last_function() {
        let registry = create_test_registry();
        let list = Value::List(List { values: vec![Value::Int(1), Value::Int(2), Value::Int(3)] });
        let result = registry.execute("last", &[list]).expect("last函数执行应该成功");
        assert_eq!(result, Value::Int(3));
    }

    #[test]
    fn test_tail_function() {
        let registry = create_test_registry();
        let list = Value::List(List { values: vec![Value::Int(1), Value::Int(2), Value::Int(3)] });
        let result = registry.execute("tail", &[list]).expect("tail函数执行应该成功");
        assert_eq!(
            result,
            Value::List(List { values: vec![Value::Int(2), Value::Int(3)] })
        );
    }

    #[test]
    fn test_size_string() {
        let registry = create_test_registry();
        let result = registry
            .execute("size", &[Value::String("hello".to_string())])
            .expect("size函数执行应该成功");
        assert_eq!(result, Value::Int(5));
    }

    #[test]
    fn test_size_list() {
        let registry = create_test_registry();
        let list = Value::List(List { values: vec![Value::Int(1), Value::Int(2), Value::Int(3)] });
        let result = registry.execute("size", &[list]).expect("size函数执行应该成功");
        assert_eq!(result, Value::Int(3));
    }

    #[test]
    fn test_size_map() {
        let registry = create_test_registry();
        let map = Value::Map(HashMap::from([
            ("a".to_string(), Value::Int(1)),
            ("b".to_string(), Value::Int(2)),
        ]));
        let result = registry.execute("size", &[map]).expect("size函数执行应该成功");
        assert_eq!(result, Value::Int(2));
    }

    #[test]
    fn test_range_basic() {
        let registry = create_test_registry();
        let result = registry
            .execute("range", &[Value::Int(1), Value::Int(5)])
            .expect("range函数执行应该成功");
        assert_eq!(
            result,
            Value::List(List { values: vec![
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
                Value::Int(4),
                Value::Int(5)
            ]})
        );
    }

    #[test]
    fn test_range_with_step() {
        let registry = create_test_registry();
        let result = registry
            .execute("range", &[Value::Int(1), Value::Int(10), Value::Int(2)])
            .expect("range函数执行应该成功");
        assert_eq!(
            result,
            Value::List(List { values: vec![Value::Int(1), Value::Int(3), Value::Int(5), Value::Int(7), Value::Int(9)] })
        );
    }

    #[test]
    fn test_range_negative_step() {
        let registry = create_test_registry();
        let result = registry
            .execute("range", &[Value::Int(5), Value::Int(1), Value::Int(-1)])
            .expect("range函数执行应该成功");
        assert_eq!(
            result,
            Value::List(List { values: vec![
                Value::Int(5),
                Value::Int(4),
                Value::Int(3),
                Value::Int(2),
                Value::Int(1)
            ]})
        );
    }

    #[test]
    fn test_keys_map() {
        let registry = create_test_registry();
        let map = Value::Map(HashMap::from([
            ("c".to_string(), Value::Int(3)),
            ("a".to_string(), Value::Int(1)),
            ("b".to_string(), Value::Int(2)),
        ]));
        let result = registry.execute("keys", &[map]).expect("keys函数执行应该成功");
        if let Value::List(keys) = result {
            assert_eq!(keys.values.len(), 3);
            assert!(keys.values.contains(&Value::String("a".to_string())));
            assert!(keys.values.contains(&Value::String("b".to_string())));
            assert!(keys.values.contains(&Value::String("c".to_string())));
        } else {
            panic!("keys函数应该返回列表");
        }
    }

    #[test]
    fn test_null_handling() {
        let registry = create_test_registry();
        let null_value = Value::Null(NullType::Null);

        assert_eq!(
            registry.execute("head", &[null_value.clone()]).expect("head函数应该处理NULL"),
            Value::Null(NullType::Null)
        );
        assert_eq!(
            registry.execute("last", &[null_value.clone()]).expect("last函数应该处理NULL"),
            Value::Null(NullType::Null)
        );
        assert_eq!(
            registry.execute("tail", &[null_value.clone()]).expect("tail函数应该处理NULL"),
            Value::Null(NullType::Null)
        );
    }
}
