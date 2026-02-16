//! 字符串函数实现

use crate::core::error::ExpressionError;
use crate::core::value::dataset::List;
use crate::core::Value;
use crate::expression::functions::registry::FunctionRegistry;
use crate::expression::functions::signature::FunctionSignature;
use crate::expression::functions::signature::ValueType;

/// 注册所有字符串函数
pub fn register_all(registry: &mut FunctionRegistry) {
    register_length(registry);
    register_upper(registry);
    register_lower(registry);
    register_trim(registry);
    register_concat(registry);
    register_concat_ws(registry);
    register_replace(registry);
    register_substring(registry);
    register_contains(registry);
    register_starts_with(registry);
    register_ends_with(registry);
    register_left(registry);
    register_right(registry);
    register_lpad(registry);
    register_rpad(registry);
    register_reverse(registry);
    register_split(registry);
    register_strcasecmp(registry);
}

fn register_length(registry: &mut FunctionRegistry) {
    registry.register(
        "length",
        FunctionSignature::new(
            "length",
            vec![ValueType::String],
            ValueType::Int,
            1,
            1,
            true,
            "计算字符串长度",
        ),
        |args| {
            match &args[0] {
                Value::String(s) => Ok(Value::Int(s.len() as i64)),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("length函数需要字符串类型")),
            }
        },
    );
}

fn register_strcasecmp(registry: &mut FunctionRegistry) {
    registry.register(
        "strcasecmp",
        FunctionSignature::new(
            "strcasecmp",
            vec![ValueType::String, ValueType::String],
            ValueType::Int,
            2,
            2,
            true,
            "不区分大小写比较两个字符串",
        ),
        |args| {
            match (&args[0], &args[1]) {
                (Value::String(s1), Value::String(s2)) => {
                    let cmp_result = s1.to_lowercase().cmp(&s2.to_lowercase());
                    let result = match cmp_result {
                        std::cmp::Ordering::Less => -1,
                        std::cmp::Ordering::Equal => 0,
                        std::cmp::Ordering::Greater => 1,
                    };
                    Ok(Value::Int(result))
                }
                (Value::Null(_), _) | (_, Value::Null(_)) => {
                    Ok(Value::Null(crate::core::value::NullType::Null))
                }
                _ => Err(ExpressionError::type_error("strcasecmp函数需要字符串类型")),
            }
        },
    );
}

fn register_upper(registry: &mut FunctionRegistry) {
    // upper / toupper
    for name in ["upper", "toupper"] {
        registry.register(
            name,
            FunctionSignature::new(
                name,
                vec![ValueType::String],
                ValueType::String,
                1,
                1,
                true,
                "转换为大写",
            ),
            |args| {
                match &args[0] {
                    Value::String(s) => Ok(Value::String(s.to_uppercase())),
                    Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Err(ExpressionError::type_error("upper函数需要字符串类型")),
                }
            },
        );
    }
}

fn register_lower(registry: &mut FunctionRegistry) {
    // lower / tolower
    for name in ["lower", "tolower"] {
        registry.register(
            name,
            FunctionSignature::new(
                name,
                vec![ValueType::String],
                ValueType::String,
                1,
                1,
                true,
                "转换为小写",
            ),
            |args| {
                match &args[0] {
                    Value::String(s) => Ok(Value::String(s.to_lowercase())),
                    Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Err(ExpressionError::type_error("lower函数需要字符串类型")),
                }
            },
        );
    }
}

fn register_trim(registry: &mut FunctionRegistry) {
    // trim
    registry.register(
        "trim",
        FunctionSignature::new(
            "trim",
            vec![ValueType::String],
            ValueType::String,
            1,
            1,
            true,
            "去除首尾空白",
        ),
        |args| {
            match &args[0] {
                Value::String(s) => Ok(Value::String(s.trim().to_string())),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("trim函数需要字符串类型")),
            }
        },
    );

    // ltrim
    registry.register(
        "ltrim",
        FunctionSignature::new(
            "ltrim",
            vec![ValueType::String],
            ValueType::String,
            1,
            1,
            true,
            "去除左侧空白",
        ),
        |args| {
            match &args[0] {
                Value::String(s) => Ok(Value::String(s.trim_start().to_string())),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("ltrim函数需要字符串类型")),
            }
        },
    );

    // rtrim
    registry.register(
        "rtrim",
        FunctionSignature::new(
            "rtrim",
            vec![ValueType::String],
            ValueType::String,
            1,
            1,
            true,
            "去除右侧空白",
        ),
        |args| {
            match &args[0] {
                Value::String(s) => Ok(Value::String(s.trim_end().to_string())),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("rtrim函数需要字符串类型")),
            }
        },
    );
}

fn register_concat(registry: &mut FunctionRegistry) {
    registry.register(
        "concat",
        FunctionSignature::new(
            "concat",
            vec![ValueType::String],
            ValueType::String,
            2,
            usize::MAX,
            true,
            "连接字符串",
        ),
        |args| {
            let mut result = String::new();
            for arg in args {
                match arg {
                    Value::String(s) => result.push_str(s),
                    Value::Null(_) => return Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => return Err(ExpressionError::type_error("concat函数需要字符串类型")),
                }
            }
            Ok(Value::String(result))
        },
    );
}

fn register_replace(registry: &mut FunctionRegistry) {
    registry.register(
        "replace",
        FunctionSignature::new(
            "replace",
            vec![ValueType::String, ValueType::String, ValueType::String],
            ValueType::String,
            3,
            3,
            true,
            "替换字符串",
        ),
        |args| {
            match (&args[0], &args[1], &args[2]) {
                (Value::String(s), Value::String(from), Value::String(to)) => {
                    Ok(Value::String(s.replace(from, to)))
                }
                (Value::String(_), Value::String(_), Value::Null(_))
                | (Value::String(_), Value::Null(_), _) => {
                    Ok(Value::Null(crate::core::value::NullType::Null))
                }
                (Value::Null(_), _, _) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("replace函数需要字符串类型")),
            }
        },
    );
}

fn register_substring(registry: &mut FunctionRegistry) {
    // substring / substr
    for name in ["substring", "substr"] {
        registry.register(
            name,
            FunctionSignature::new(
                name,
                vec![ValueType::String, ValueType::Int],
                ValueType::String,
                2,
                3,
                true,
                "获取子字符串",
            ),
            |args| {
                match (&args[0], &args[1]) {
                    (Value::String(s), Value::Int(start)) => {
                        let start = *start as usize;
                        if args.len() == 3 {
                            if let Value::Int(len) = &args[2] {
                                let len = *len as usize;
                                if start <= s.len() && len <= s.len() - start {
                                    Ok(Value::String(s[start..start + len].to_string()))
                                } else {
                                    Ok(Value::String(s[start..].to_string()))
                                }
                            } else {
                                Err(ExpressionError::type_error("substring函数需要整数类型参数"))
                            }
                        } else {
                            if start <= s.len() {
                                Ok(Value::String(s[start..].to_string()))
                            } else {
                                Ok(Value::String(String::new()))
                            }
                        }
                    }
                    (Value::Null(_), _) | (_, Value::Null(_)) => {
                        Ok(Value::Null(crate::core::value::NullType::Null))
                    }
                    _ => Err(ExpressionError::type_error("substring函数需要字符串和整数类型")),
                }
            },
        );
    }
}

fn register_contains(registry: &mut FunctionRegistry) {
    registry.register(
        "contains",
        FunctionSignature::new(
            "contains",
            vec![ValueType::String, ValueType::String],
            ValueType::Bool,
            2,
            2,
            true,
            "检查是否包含子字符串",
        ),
        |args| {
            match (&args[0], &args[1]) {
                (Value::String(s), Value::String(sub)) => {
                    Ok(Value::Bool(s.contains(sub)))
                }
                (Value::Null(_), _) | (_, Value::Null(_)) => {
                    Ok(Value::Null(crate::core::value::NullType::Null))
                }
                _ => Err(ExpressionError::type_error("contains函数需要字符串类型")),
            }
        },
    );
}

fn register_starts_with(registry: &mut FunctionRegistry) {
    registry.register(
        "starts_with",
        FunctionSignature::new(
            "starts_with",
            vec![ValueType::String, ValueType::String],
            ValueType::Bool,
            2,
            2,
            true,
            "检查是否以指定字符串开头",
        ),
        |args| {
            match (&args[0], &args[1]) {
                (Value::String(s), Value::String(prefix)) => {
                    Ok(Value::Bool(s.starts_with(prefix)))
                }
                (Value::Null(_), _) | (_, Value::Null(_)) => {
                    Ok(Value::Null(crate::core::value::NullType::Null))
                }
                _ => Err(ExpressionError::type_error("starts_with函数需要字符串类型")),
            }
        },
    );
}

fn register_ends_with(registry: &mut FunctionRegistry) {
    registry.register(
        "ends_with",
        FunctionSignature::new(
            "ends_with",
            vec![ValueType::String, ValueType::String],
            ValueType::Bool,
            2,
            2,
            true,
            "检查是否以指定字符串结尾",
        ),
        |args| {
            match (&args[0], &args[1]) {
                (Value::String(s), Value::String(suffix)) => {
                    Ok(Value::Bool(s.ends_with(suffix)))
                }
                (Value::Null(_), _) | (_, Value::Null(_)) => {
                    Ok(Value::Null(crate::core::value::NullType::Null))
                }
                _ => Err(ExpressionError::type_error("ends_with函数需要字符串类型")),
            }
        },
    );
}

fn register_left(registry: &mut FunctionRegistry) {
    registry.register(
        "left",
        FunctionSignature::new(
            "left",
            vec![ValueType::String, ValueType::Int],
            ValueType::String,
            2,
            2,
            true,
            "获取左侧子字符串",
        ),
        |args| {
            match (&args[0], &args[1]) {
                (Value::String(s), Value::Int(len)) => {
                    let len = *len as usize;
                    if len <= s.len() {
                        Ok(Value::String(s[..len].to_string()))
                    } else {
                        Ok(Value::String(s.clone()))
                    }
                }
                (Value::Null(_), _) | (_, Value::Null(_)) => {
                    Ok(Value::Null(crate::core::value::NullType::Null))
                }
                _ => Err(ExpressionError::type_error("left函数需要字符串和整数类型")),
            }
        },
    );
}

fn register_right(registry: &mut FunctionRegistry) {
    registry.register(
        "right",
        FunctionSignature::new(
            "right",
            vec![ValueType::String, ValueType::Int],
            ValueType::String,
            2,
            2,
            true,
            "获取右侧子字符串",
        ),
        |args| {
            match (&args[0], &args[1]) {
                (Value::String(s), Value::Int(len)) => {
                    let len = *len as usize;
                    if len <= s.len() {
                        Ok(Value::String(s[s.len() - len..].to_string()))
                    } else {
                        Ok(Value::String(s.clone()))
                    }
                }
                (Value::Null(_), _) | (_, Value::Null(_)) => {
                    Ok(Value::Null(crate::core::value::NullType::Null))
                }
                _ => Err(ExpressionError::type_error("right函数需要字符串和整数类型")),
            }
        },
    );
}

fn register_reverse(registry: &mut FunctionRegistry) {
    registry.register(
        "reverse",
        FunctionSignature::new(
            "reverse",
            vec![ValueType::String],
            ValueType::String,
            1,
            1,
            true,
            "反转字符串",
        ),
        |args| {
            match &args[0] {
                Value::String(s) => {
                    let reversed: String = s.chars().rev().collect();
                    Ok(Value::String(reversed))
                }
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("reverse函数需要字符串类型")),
            }
        },
    );
}

fn register_split(registry: &mut FunctionRegistry) {
    registry.register(
        "split",
        FunctionSignature::new(
            "split",
            vec![ValueType::String, ValueType::String],
            ValueType::List,
            2,
            2,
            true,
            "按分隔符分割字符串",
        ),
        |args| {
            match (&args[0], &args[1]) {
                (Value::String(s), Value::String(sep)) => {
                    let parts: Vec<Value> = s.split(sep).map(|part| Value::String(part.to_string())).collect();
                    Ok(Value::List(List { values: parts }))
                }
                (Value::Null(_), _) | (_, Value::Null(_)) => {
                    Ok(Value::Null(crate::core::value::NullType::Null))
                }
                _ => Err(ExpressionError::type_error("split函数需要字符串类型")),
            }
        },
    );
}

fn register_concat_ws(registry: &mut FunctionRegistry) {
    registry.register(
        "concat_ws",
        FunctionSignature::new(
            "concat_ws",
            vec![ValueType::String],
            ValueType::String,
            2,
            usize::MAX,
            true,
            "使用分隔符连接字符串",
        ),
        |args| {
            if args.is_empty() {
                return Ok(Value::String(String::new()));
            }

            let separator = match &args[0] {
                Value::String(s) => s.clone(),
                Value::Null(_) => return Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => return Err(ExpressionError::type_error("concat_ws函数第一个参数需要字符串类型")),
            };

            let mut result = String::new();
            let mut first = true;

            for arg in &args[1..] {
                match arg {
                    Value::String(s) => {
                        if !first {
                            result.push_str(&separator);
                        }
                        result.push_str(s);
                        first = false;
                    }
                    Value::Null(_) => continue,
                    _ => return Err(ExpressionError::type_error("concat_ws函数参数需要字符串类型")),
                }
            }

            Ok(Value::String(result))
        },
    );
}

fn register_lpad(registry: &mut FunctionRegistry) {
    registry.register(
        "lpad",
        FunctionSignature::new(
            "lpad",
            vec![ValueType::String, ValueType::Int],
            ValueType::String,
            2,
            3,
            true,
            "左侧填充字符串至指定长度",
        ),
        |args| {
            match (&args[0], &args[1]) {
                (Value::String(s), Value::Int(len)) => {
                    let len = *len as usize;
                    let pad_str = if args.len() == 3 {
                        match &args[2] {
                            Value::String(pad) => pad.clone(),
                            Value::Null(_) => return Ok(Value::Null(crate::core::value::NullType::Null)),
                            _ => return Err(ExpressionError::type_error("lpad函数第三个参数需要字符串类型")),
                        }
                    } else {
                        String::from(" ")
                    };

                    if s.len() >= len {
                        Ok(Value::String(s[..len].to_string()))
                    } else {
                        let pad_len = len - s.len();
                        let mut result = String::new();
                        while result.len() < pad_len {
                            result.push_str(&pad_str);
                        }
                        result.truncate(pad_len);
                        result.push_str(s);
                        Ok(Value::String(result))
                    }
                }
                (Value::Null(_), _) | (_, Value::Null(_)) => {
                    Ok(Value::Null(crate::core::value::NullType::Null))
                }
                _ => Err(ExpressionError::type_error("lpad函数需要字符串和整数类型")),
            }
        },
    );
}

fn register_rpad(registry: &mut FunctionRegistry) {
    registry.register(
        "rpad",
        FunctionSignature::new(
            "rpad",
            vec![ValueType::String, ValueType::Int],
            ValueType::String,
            2,
            3,
            true,
            "右侧填充字符串至指定长度",
        ),
        |args| {
            match (&args[0], &args[1]) {
                (Value::String(s), Value::Int(len)) => {
                    let len = *len as usize;
                    let pad_str = if args.len() == 3 {
                        match &args[2] {
                            Value::String(pad) => pad.clone(),
                            Value::Null(_) => return Ok(Value::Null(crate::core::value::NullType::Null)),
                            _ => return Err(ExpressionError::type_error("rpad函数第三个参数需要字符串类型")),
                        }
                    } else {
                        String::from(" ")
                    };

                    if s.len() >= len {
                        Ok(Value::String(s[..len].to_string()))
                    } else {
                        let pad_len = len - s.len();
                        let mut result = s.clone();
                        let mut pad_count = 0;
                        while pad_count < pad_len {
                            result.push_str(&pad_str);
                            pad_count += pad_str.len();
                        }
                        result.truncate(len);
                        Ok(Value::String(result))
                    }
                }
                (Value::Null(_), _) | (_, Value::Null(_)) => {
                    Ok(Value::Null(crate::core::value::NullType::Null))
                }
                _ => Err(ExpressionError::type_error("rpad函数需要字符串和整数类型")),
            }
        },
    );
}
