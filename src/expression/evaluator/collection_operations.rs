//! 集合操作求值器
//!
//! 提供集合类型的求值功能，包括下标访问、范围访问和属性访问

use crate::core::error::ExpressionError;
use crate::core::Value;
use crate::core::value::dataset::List;

/// 集合操作求值器
pub struct CollectionOperationEvaluator;

impl CollectionOperationEvaluator {
    /// 求值下标访问
    pub fn eval_subscript_access(
        &self,
        collection: &Value,
        index: &Value,
    ) -> Result<Value, ExpressionError> {
        if collection.is_null() || index.is_null() {
            return Ok(Value::Null(crate::core::value::NullType::Null));
        }

        match collection {
            Value::List(list) => {
                if let Value::Int(i) = index {
                    let adjusted_index = if *i < 0 { list.len() as i64 + i } else { *i };

                    if adjusted_index >= 0 && (adjusted_index as usize) < list.len() {
                        Ok(list[adjusted_index as usize].clone())
                    } else {
                        Err(ExpressionError::index_out_of_bounds(
                            adjusted_index as isize,
                            list.len(),
                        ))
                    }
                } else {
                    Err(ExpressionError::type_error("列表下标必须是整数"))
                }
            }
            Value::String(s) => {
                if let Value::Int(i) = index {
                    let chars: Vec<char> = s.chars().collect();
                    let adjusted_index = if *i < 0 { chars.len() as i64 + i } else { *i };

                    if adjusted_index >= 0 && (adjusted_index as usize) < chars.len() {
                        Ok(Value::String(chars[adjusted_index as usize].to_string()))
                    } else {
                        Err(ExpressionError::index_out_of_bounds(
                            adjusted_index as isize,
                            chars.len(),
                        ))
                    }
                } else {
                    Err(ExpressionError::type_error("字符串下标必须是整数"))
                }
            }
            Value::Map(map) => {
                if let Value::String(key) = index {
                    map.get(key).cloned().ok_or_else(|| {
                        ExpressionError::runtime_error(format!("映射键不存在: {}", key))
                    })
                } else {
                    Err(ExpressionError::type_error("映射键必须是字符串"))
                }
            }
            _ => Err(ExpressionError::type_error("不支持下标访问的类型")),
        }
    }

    /// 求值范围访问
    pub fn eval_range_access(
        &self,
        collection: &Value,
        start: Option<&Value>,
        end: Option<&Value>,
    ) -> Result<Value, ExpressionError> {
        if collection.is_null() {
            return Ok(Value::Null(crate::core::value::NullType::Null));
        }

        if start.map_or(false, |v| v.is_null()) || end.map_or(false, |v| v.is_null()) {
            return Ok(Value::Null(crate::core::value::NullType::Null));
        }

        match collection {
            Value::List(list) => {
                let start_idx = start
                    .map(|v| {
                        if let Value::Int(i) = v {
                            if *i < 0 {
                                (list.len() as i64 + i) as usize
                            } else {
                                *i as usize
                            }
                        } else {
                            0
                        }
                    })
                    .unwrap_or(0);

                let end_idx = end
                    .map(|v| {
                        if let Value::Int(i) = v {
                            if *i < 0 {
                                (list.len() as i64 + i) as usize
                            } else {
                                *i as usize
                            }
                        } else {
                            list.len()
                        }
                    })
                    .unwrap_or(list.len());

                if start_idx <= end_idx && end_idx <= list.len() {
                    Ok(Value::List(List::from(list[start_idx..end_idx].to_vec())))
                } else {
                    Err(ExpressionError::index_out_of_bounds(
                        start_idx as isize,
                        list.len(),
                    ))
                }
            }
            Value::String(s) => {
                let chars: Vec<char> = s.chars().collect();
                let start_idx = start
                    .map(|v| {
                        if let Value::Int(i) = v {
                            if *i < 0 {
                                (chars.len() as i64 + i) as usize
                            } else {
                                *i as usize
                            }
                        } else {
                            0
                        }
                    })
                    .unwrap_or(0);

                let end_idx = end
                    .map(|v| {
                        if let Value::Int(i) = v {
                            if *i < 0 {
                                (chars.len() as i64 + i) as usize
                            } else {
                                *i as usize
                            }
                        } else {
                            chars.len()
                        }
                    })
                    .unwrap_or(chars.len());

                if start_idx <= end_idx && end_idx <= chars.len() {
                    let result: String = chars[start_idx..end_idx].iter().collect();
                    Ok(Value::String(result))
                } else {
                    Err(ExpressionError::index_out_of_bounds(
                        start_idx as isize,
                        chars.len(),
                    ))
                }
            }
            _ => Err(ExpressionError::type_error("不支持范围访问的类型")),
        }
    }

    /// 求值属性访问
    pub fn eval_property_access(
        &self,
        object: &Value,
        property: &str,
    ) -> Result<Value, ExpressionError> {
        if object.is_null() {
            return Ok(Value::Null(crate::core::value::NullType::Null));
        }

        match object {
            Value::Vertex(vertex) => vertex.properties.get(property).cloned().ok_or_else(|| {
                ExpressionError::runtime_error(format!("顶点属性不存在: {}", property))
            }),
            Value::Edge(edge) => edge.properties().get(property).cloned().ok_or_else(|| {
                ExpressionError::runtime_error(format!("边属性不存在: {}", property))
            }),
            Value::Map(map) => map.get(property).cloned().ok_or_else(|| {
                ExpressionError::runtime_error(format!("映射键不存在: {}", property))
            }),
            Value::List(list) => {
                if let Ok(index) = property.parse::<isize>() {
                    let adjusted_index = if index < 0 {
                        list.len() as isize + index
                    } else {
                        index
                    };

                    if adjusted_index >= 0 && adjusted_index < list.len() as isize {
                        Ok(list[adjusted_index as usize].clone())
                    } else {
                        Err(ExpressionError::index_out_of_bounds(
                            adjusted_index,
                            list.len(),
                        ))
                    }
                } else {
                    Err(ExpressionError::type_error("列表索引必须是整数"))
                }
            }
            _ => Err(ExpressionError::type_error("不支持属性访问的类型")),
        }
    }
}
