//! 类型转换函数实现

use crate::core::error::ExpressionError;
use crate::core::Value;
use crate::expression::functions::registry::FunctionRegistry;
use crate::expression::functions::signature::FunctionSignature;
use crate::expression::functions::signature::ValueType;

/// 注册所有类型转换函数
pub fn register_all(registry: &mut FunctionRegistry) {
    register_to_string(registry);
    register_to_int(registry);
    register_to_float(registry);
    register_to_bool(registry);
    register_toset(registry);
}

fn register_to_string(registry: &mut FunctionRegistry) {
    // to_string - STRING 版本
    registry.register(
        "to_string",
        FunctionSignature::new(
            "to_string",
            vec![ValueType::String],
            ValueType::String,
            1,
            1,
            true,
            "转换为字符串",
        ),
        |args| {
            match &args[0] {
                Value::String(s) => Ok(Value::String(s.clone())),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("to_string函数需要字符串类型")),
            }
        },
    );

    // to_string - INT 版本
    registry.register(
        "to_string",
        FunctionSignature::new(
            "to_string",
            vec![ValueType::Int],
            ValueType::String,
            1,
            1,
            true,
            "转换为字符串",
        ),
        |args| {
            match &args[0] {
                Value::Int(i) => Ok(Value::String(i.to_string())),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("to_string函数需要整数类型")),
            }
        },
    );

    // to_string - FLOAT 版本
    registry.register(
        "to_string",
        FunctionSignature::new(
            "to_string",
            vec![ValueType::Float],
            ValueType::String,
            1,
            1,
            true,
            "转换为字符串",
        ),
        |args| {
            match &args[0] {
                Value::Float(f) => Ok(Value::String(f.to_string())),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("to_string函数需要浮点数类型")),
            }
        },
    );

    // to_string - BOOL 版本
    registry.register(
        "to_string",
        FunctionSignature::new(
            "to_string",
            vec![ValueType::Bool],
            ValueType::String,
            1,
            1,
            true,
            "转换为字符串",
        ),
        |args| {
            match &args[0] {
                Value::Bool(b) => Ok(Value::String(b.to_string())),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("to_string函数需要布尔类型")),
            }
        },
    );
}

fn register_to_int(registry: &mut FunctionRegistry) {
    // to_int - INT 版本
    for name in ["to_int", "to_integer"] {
        registry.register(
            name,
            FunctionSignature::new(
                name,
                vec![ValueType::Int],
                ValueType::Int,
                1,
                1,
                true,
                "转换为整数",
            ),
            |args| {
                match &args[0] {
                    Value::Int(i) => Ok(Value::Int(*i)),
                    Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Err(ExpressionError::type_error("to_int函数需要整数类型")),
                }
            },
        );
    }

    // to_int - FLOAT 版本
    for name in ["to_int", "to_integer"] {
        registry.register(
            name,
            FunctionSignature::new(
                name,
                vec![ValueType::Float],
                ValueType::Int,
                1,
                1,
                true,
                "转换为整数",
            ),
            |args| {
                match &args[0] {
                    Value::Float(f) => Ok(Value::Int(*f as i64)),
                    Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Err(ExpressionError::type_error("to_int函数需要浮点数类型")),
                }
            },
        );
    }

    // to_int - STRING 版本
    for name in ["to_int", "to_integer"] {
        registry.register(
            name,
            FunctionSignature::new(
                name,
                vec![ValueType::String],
                ValueType::Int,
                1,
                1,
                true,
                "转换为整数",
            ),
            |args| {
                match &args[0] {
                    Value::String(s) => {
                        s.parse::<i64>()
                            .map(Value::Int)
                            .map_err(|_| ExpressionError::type_error("无法将字符串转换为整数"))
                    }
                    Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Err(ExpressionError::type_error("to_int函数需要字符串类型")),
                }
            },
        );
    }

    // to_int - BOOL 版本
    for name in ["to_int", "to_integer"] {
        registry.register(
            name,
            FunctionSignature::new(
                name,
                vec![ValueType::Bool],
                ValueType::Int,
                1,
                1,
                true,
                "转换为整数",
            ),
            |args| {
                match &args[0] {
                    Value::Bool(b) => Ok(Value::Int(if *b { 1 } else { 0 })),
                    Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Err(ExpressionError::type_error("to_int函数需要布尔类型")),
                }
            },
        );
    }
}

fn register_to_float(registry: &mut FunctionRegistry) {
    // to_float - FLOAT 版本
    registry.register(
        "to_float",
        FunctionSignature::new(
            "to_float",
            vec![ValueType::Float],
            ValueType::Float,
            1,
            1,
            true,
            "转换为浮点数",
        ),
        |args| {
            match &args[0] {
                Value::Float(f) => Ok(Value::Float(*f)),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("to_float函数需要浮点数类型")),
            }
        },
    );

    // to_float - INT 版本
    registry.register(
        "to_float",
        FunctionSignature::new(
            "to_float",
            vec![ValueType::Int],
            ValueType::Float,
            1,
            1,
            true,
            "转换为浮点数",
        ),
        |args| {
            match &args[0] {
                Value::Int(i) => Ok(Value::Float(*i as f64)),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("to_float函数需要整数类型")),
            }
        },
    );

    // to_float - STRING 版本
    registry.register(
        "to_float",
        FunctionSignature::new(
            "to_float",
            vec![ValueType::String],
            ValueType::Float,
            1,
            1,
            true,
            "转换为浮点数",
        ),
        |args| {
            match &args[0] {
                Value::String(s) => {
                    s.parse::<f64>()
                        .map(Value::Float)
                        .map_err(|_| ExpressionError::type_error("无法将字符串转换为浮点数"))
                }
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("to_float函数需要字符串类型")),
            }
        },
    );

    // to_float - BOOL 版本
    registry.register(
        "to_float",
        FunctionSignature::new(
            "to_float",
            vec![ValueType::Bool],
            ValueType::Float,
            1,
            1,
            true,
            "转换为浮点数",
        ),
        |args| {
            match &args[0] {
                Value::Bool(b) => Ok(Value::Float(if *b { 1.0 } else { 0.0 })),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("to_float函数需要布尔类型")),
            }
        },
    );
}

fn register_to_bool(registry: &mut FunctionRegistry) {
    // to_bool - BOOL 版本
    registry.register(
        "to_bool",
        FunctionSignature::new(
            "to_bool",
            vec![ValueType::Bool],
            ValueType::Bool,
            1,
            1,
            true,
            "转换为布尔值",
        ),
        |args| {
            match &args[0] {
                Value::Bool(b) => Ok(Value::Bool(*b)),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("to_bool函数需要布尔类型")),
            }
        },
    );

    // to_bool - INT 版本
    registry.register(
        "to_bool",
        FunctionSignature::new(
            "to_bool",
            vec![ValueType::Int],
            ValueType::Bool,
            1,
            1,
            true,
            "转换为布尔值",
        ),
        |args| {
            match &args[0] {
                Value::Int(i) => Ok(Value::Bool(*i != 0)),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("to_bool函数需要整数类型")),
            }
        },
    );

    // to_bool - FLOAT 版本
    registry.register(
        "to_bool",
        FunctionSignature::new(
            "to_bool",
            vec![ValueType::Float],
            ValueType::Bool,
            1,
            1,
            true,
            "转换为布尔值",
        ),
        |args| {
            match &args[0] {
                Value::Float(f) => Ok(Value::Bool(*f != 0.0)),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("to_bool函数需要浮点数类型")),
            }
        },
    );

    // to_bool - STRING 版本
    registry.register(
        "to_bool",
        FunctionSignature::new(
            "to_bool",
            vec![ValueType::String],
            ValueType::Bool,
            1,
            1,
            true,
            "转换为布尔值",
        ),
        |args| {
            match &args[0] {
                Value::String(s) => {
                    let s_lower = s.to_lowercase();
                    Ok(Value::Bool(s_lower == "true" || s_lower == "1"))
                }
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("to_bool函数需要字符串类型")),
            }
        },
    );
}

fn register_toset(registry: &mut FunctionRegistry) {
    registry.register(
        "toset",
        FunctionSignature::new(
            "toset",
            vec![ValueType::List],
            ValueType::Set,
            1,
            1,
            true,
            "将列表转换为集合（去重）",
        ),
        |args| {
            match &args[0] {
                Value::List(list) => {
                    let set: std::collections::HashSet<Value> = list.values.iter().cloned().collect();
                    Ok(Value::Set(set))
                }
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("toset函数需要列表类型")),
            }
        },
    );
}
