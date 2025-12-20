use crate::core::ExpressionError;
use crate::core::Value;

/// 类型转换模块
/// 提供各种类型之间的转换功能

/// 将值转换为指定的数据类型
pub fn cast_value_to_datatype(
    value: Value,
    target_type: &crate::expression::expression::DataType,
) -> Result<Value, ExpressionError> {
    use crate::expression::expression::DataType;
    match target_type {
        DataType::Bool => cast_to_bool(value),
        DataType::Int => cast_to_int(value),
        DataType::Float => cast_to_float(value),
        DataType::String => cast_to_string(value),
        DataType::List => cast_to_list(value),
        DataType::Map => cast_to_map(value),
        DataType::Vertex => cast_to_vertex(value),
        DataType::Edge => cast_to_edge(value),
        DataType::Path => cast_to_path(value),
        DataType::DateTime => cast_to_datetime(value),
    }
}

/// 使用字符串类型名进行类型转换
pub fn cast_value(value: Value, target_type: &str) -> Result<Value, ExpressionError> {
    match target_type.to_lowercase().as_str() {
        "int" | "integer" => cast_to_int(value),
        "float" | "double" => cast_to_float(value),
        "string" => cast_to_string(value),
        "bool" | "boolean" => cast_to_bool(value),
        "list" => cast_to_list(value),
        "map" => cast_to_map(value),
        _ => Err(ExpressionError::TypeError(format!(
            "Unknown target type: {}",
            target_type
        ))),
    }
}

/// 转换为整数
pub fn cast_to_int(value: Value) -> Result<Value, ExpressionError> {
    match value {
        Value::Int(_) => Ok(value),
        Value::Float(f) => Ok(Value::Int(f as i64)),
        Value::String(s) => {
            if let Ok(i) = s.parse::<i64>() {
                Ok(Value::Int(i))
            } else if let Ok(f) = s.parse::<f64>() {
                Ok(Value::Int(f as i64))
            } else {
                Err(ExpressionError::TypeError(
                    "Cannot convert string to int".to_string(),
                ))
            }
        }
        Value::Bool(b) => Ok(Value::Int(if b { 1 } else { 0 })),
        Value::Null(_) => Ok(Value::Null(crate::core::NullType::Null)),
        _ => Err(ExpressionError::TypeError(
            "Cannot convert to int".to_string(),
        )),
    }
}

/// 转换为浮点数
pub fn cast_to_float(value: Value) -> Result<Value, ExpressionError> {
    match value {
        Value::Float(_) => Ok(value),
        Value::Int(i) => Ok(Value::Float(i as f64)),
        Value::String(s) => s.parse::<f64>().map(Value::Float).map_err(|_| {
            ExpressionError::TypeError("Cannot convert string to float".to_string())
        }),
        Value::Bool(b) => Ok(Value::Float(if b { 1.0 } else { 0.0 })),
        Value::Null(_) => Ok(Value::Null(crate::core::NullType::Null)),
        _ => Err(ExpressionError::TypeError(
            "Cannot convert to float".to_string(),
        )),
    }
}

/// 转换为字符串
pub fn cast_to_string(value: Value) -> Result<Value, ExpressionError> {
    match value {
        Value::String(_) => Ok(value),
        Value::Int(i) => Ok(Value::String(i.to_string())),
        Value::Float(f) => Ok(Value::String(f.to_string())),
        Value::Bool(b) => Ok(Value::String(b.to_string())),
        Value::Null(_) => Ok(Value::String("null".to_string())),
        Value::List(items) => {
            let strings: Result<Vec<String>, _> = items
                .into_iter()
                .map(|item| {
                    if let Ok(Value::String(s)) = cast_to_string(item) {
                        Ok(s)
                    } else {
                        Err(ExpressionError::TypeError(
                            "Cannot convert list item to string".to_string(),
                        ))
                    }
                })
                .collect();
            Ok(Value::String(format!("[{}]", strings?.join(", "))))
        }
        Value::Map(map) => {
            let pairs: Result<Vec<String>, _> = map
                .into_iter()
                .map(|(k, v)| {
                    if let Ok(Value::String(s)) = cast_to_string(v) {
                        Ok(format!("{}: {}", k, s))
                    } else {
                        Err(ExpressionError::TypeError(
                            "Cannot convert map value to string".to_string(),
                        ))
                    }
                })
                .collect();
            Ok(Value::String(format!("{{{}}}", pairs?.join(", "))))
        }
        _ => Err(ExpressionError::TypeError(
            "Cannot convert to string".to_string(),
        )),
    }
}

/// 转换为布尔值
pub fn cast_to_bool(value: Value) -> Result<Value, ExpressionError> {
    match value {
        Value::Bool(_) => Ok(value),
        Value::Int(i) => Ok(Value::Bool(i != 0)),
        Value::Float(f) => Ok(Value::Bool(f != 0.0 && !f.is_nan())),
        Value::String(s) => match s.to_lowercase().as_str() {
            "true" | "t" | "yes" | "y" | "1" => Ok(Value::Bool(true)),
            "false" | "f" | "no" | "n" | "0" | "" => Ok(Value::Bool(false)),
            _ => Err(ExpressionError::TypeError(
                "Cannot convert string to bool".to_string(),
            )),
        },
        Value::Null(_) => Ok(Value::Bool(false)),
        Value::List(items) => Ok(Value::Bool(!items.is_empty())),
        Value::Map(map) => Ok(Value::Bool(!map.is_empty())),
        _ => Ok(Value::Bool(true)),
    }
}

/// 转换为列表
pub fn cast_to_list(value: Value) -> Result<Value, ExpressionError> {
    match value {
        Value::List(_) => Ok(value),
        Value::String(s) => {
            let chars: Vec<Value> = s.chars().map(|c| Value::String(c.to_string())).collect();
            Ok(Value::List(chars))
        }
        Value::Null(_) => Ok(Value::List(vec![])),
        _ => Ok(Value::List(vec![value])),
    }
}

/// 转换为映射
pub fn cast_to_map(value: Value) -> Result<Value, ExpressionError> {
    match value {
        Value::Map(_) => Ok(value),
        Value::Null(_) => Ok(Value::Map(std::collections::HashMap::new())),
        Value::List(items) => {
            let mut map = std::collections::HashMap::new();
            for (i, item) in items.into_iter().enumerate() {
                map.insert(i.to_string(), item);
            }
            Ok(Value::Map(map))
        }
        _ => Err(ExpressionError::TypeError(
            "Cannot convert to map".to_string(),
        )),
    }
}

/// 转换为顶点
pub fn cast_to_vertex(value: Value) -> Result<Value, ExpressionError> {
    match value {
        Value::Vertex(_) => Ok(value),
        Value::Map(map) => {
            // 从Map创建Vertex，需要vid和tags
            let vid = map
                .get("vid")
                .or_else(|| map.get("id"))
                .cloned()
                .unwrap_or(Value::Null(crate::core::NullType::Null));
            let tags = if let Some(tags_value) = map.get("tags") {
                match tags_value {
                    Value::List(tag_list) => {
                        // 简化实现：将每个tag转换为Tag
                        tag_list
                            .iter()
                            .map(|tag| match tag {
                                Value::Map(tag_map) => {
                                    let name = tag_map
                                        .get("name")
                                        .and_then(|v| match v {
                                            Value::String(s) => Some(s.clone()),
                                            _ => None,
                                        })
                                        .unwrap_or("default".to_string());
                                    let properties = tag_map
                                        .iter()
                                        .filter(|(k, _)| k.as_str() != "name")
                                        .map(|(k, v)| (k.clone(), v.clone()))
                                        .collect();
                                    crate::core::Tag { name, properties }
                                }
                                _ => crate::core::Tag {
                                    name: "default".to_string(),
                                    properties: std::collections::HashMap::new(),
                                },
                            })
                            .collect()
                    }
                    _ => vec![],
                }
            } else {
                vec![]
            };

            // 提取顶点级别的属性
            let properties = map
                .iter()
                .filter(|(k, _)| !["vid", "id", "tags"].contains(&k.as_str()))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();

            Ok(Value::Vertex(Box::new(crate::core::Vertex {
                vid: Box::new(vid),
                tags,
                properties,
            })))
        }
        _ => Err(ExpressionError::TypeError(
            "Cannot convert to vertex".to_string(),
        )),
    }
}

/// 转换为边
pub fn cast_to_edge(value: Value) -> Result<Value, ExpressionError> {
    match value {
        Value::Edge(_) => Ok(value),
        Value::Map(map) => {
            // 从Map创建Edge
            let edge_type = map
                .get("type")
                .or_else(|| map.get("edge_type"))
                .and_then(|v| match v {
                    Value::String(s) => Some(s.clone()),
                    _ => None,
                })
                .unwrap_or("default".to_string());
            let src = map
                .get("src")
                .cloned()
                .unwrap_or(Value::Null(crate::core::NullType::Null));
            let dst = map
                .get("dst")
                .cloned()
                .unwrap_or(Value::Null(crate::core::NullType::Null));
            let ranking = map
                .get("ranking")
                .and_then(|v| match v {
                    Value::Int(i) => Some(*i),
                    _ => None,
                })
                .unwrap_or(0);
            let props = map
                .iter()
                .filter(|(k, _)| {
                    !["type", "edge_type", "src", "dst", "ranking"].contains(&k.as_str())
                })
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();

            Ok(Value::Edge(crate::core::Edge {
                src: Box::new(src),
                dst: Box::new(dst),
                edge_type,
                ranking,
                props,
            }))
        }
        _ => Err(ExpressionError::TypeError(
            "Cannot convert to edge".to_string(),
        )),
    }
}

/// 转换为路径
pub fn cast_to_path(value: Value) -> Result<Value, ExpressionError> {
    match value {
        Value::Path(_) => Ok(value),
        Value::List(items) => {
            // 从List创建Path - 简化实现
            // 假设第一个元素是源顶点，其余是步骤
            if items.is_empty() {
                return Ok(Value::Path(crate::core::Path::default()));
            }

            // 尝试将第一个元素作为源顶点
            let src_vertex = match &items[0] {
                Value::Vertex(v) => (**v).clone(),
                _ => {
                    return Err(ExpressionError::TypeError(
                        "Path list must start with a vertex".to_string(),
                    ))
                }
            };

            // 简化实现：创建空路径
            Ok(Value::Path(crate::core::Path {
                src: Box::new(src_vertex),
                steps: vec![],
            }))
        }
        _ => Err(ExpressionError::TypeError(
            "Cannot convert to path".to_string(),
        )),
    }
}

/// 转换为日期时间
pub fn cast_to_datetime(value: Value) -> Result<Value, ExpressionError> {
    match value {
        Value::String(s) => {
            // 简化实现：将字符串解析为时间戳
            s.parse::<i64>().map(|ts| Value::Int(ts)).map_err(|_| {
                ExpressionError::TypeError("Cannot convert string to datetime".to_string())
            })
        }
        Value::Int(i) => Ok(Value::Int(i)),
        Value::Float(f) => Ok(Value::Int(f as i64)),
        _ => Err(ExpressionError::TypeError(
            "Cannot convert to datetime".to_string(),
        )),
    }
}