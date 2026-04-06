//! 树形结构解析
//!
//! 解析嵌套字段路径，支持数组索引和属性访问
//!
//! # 支持的语法
//!
//! ```rust
//! use inversearch_service::parse_tree;
//!
//! // 嵌套属性
//! parse_tree("user.name", &mut vec![]);
//!
//! // 数组索引
//! parse_tree("items[0].title", &mut vec![]);
//!
//! // 负数索引
//! parse_tree("items[-1].name", &mut vec![]);
//!
//! // 范围索引
//! parse_tree("items[0-2].title", &mut vec![]);
//! ```

use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// 树形路径项
#[derive(Debug, Clone, PartialEq)]
pub enum TreePath {
    /// 普通字段
    Field(String),
    /// 数组索引
    Index(usize, String),
    /// 负数索引（倒数）
    NegativeIndex(usize, String),
    /// 范围索引 [start-end]
    Range(usize, usize, String),
    /// 通配符字段
    Wildcard(String),
}

/// 路径解析错误
#[derive(Debug, Clone, PartialEq)]
pub enum PathParseError {
    /// 语法错误
    SyntaxError(String),
    /// 类型错误
    TypeError(String),
    /// 越界错误
    OutOfBoundsError(String),
    /// 不存在错误
    NotFoundError(String),
}

/// 求值策略
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EvaluationStrategy {
    /// 严格求值：遇到错误立即返回
    #[default]
    Strict,
    /// 宽松求值：遇到错误时返回默认值并继续
    Lenient,
    /// 部分求值：返回已成功求值的部分结果
    Partial,
}

/// 路径解析缓存
pub struct PathCache {
    cache: Arc<RwLock<HashMap<String, Vec<TreePath>>>>,
}

impl PathCache {
    pub fn new() -> Self {
        PathCache {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn get(&self, path: &str) -> Option<Vec<TreePath>> {
        let cache = self.cache.read().ok()?;
        cache.get(path).cloned()
    }

    pub fn set(&self, path: &str, parsed: Vec<TreePath>) {
        if let Ok(mut cache) = self.cache.write() {
            cache.insert(path.to_string(), parsed);
        }
    }

    pub fn clear(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
        }
    }
}

impl Default for PathCache {
    fn default() -> Self {
        Self::new()
    }
}

/// 解析树形路径（带缓存）
pub fn parse_tree_cached(key: &str, marker: &mut Vec<bool>, cache: &PathCache) -> Vec<TreePath> {
    if let Some(cached) = cache.get(key) {
        return cached;
    }

    let result = parse_tree(key, marker);
    cache.set(key, result.clone());
    result
}

/// 解析树形路径
///
/// # 示例
///
/// ```rust
/// use inversearch_service::document::tree::parse_tree;
///
/// let mut marker = vec![];
/// let result = parse_tree("user.name", &mut marker);
/// assert_eq!(result.len(), 2);
/// ```
pub fn parse_tree(key: &str, marker: &mut Vec<bool>) -> Vec<TreePath> {
    let mut result = Vec::new();
    let mut current = key;

    while !current.is_empty() {
        if let Some(dot_pos) = current.find('.') {
            let part = &current[..dot_pos];
            current = &current[dot_pos + 1..];

            if let Some(start) = part.rfind('[') {
                let end = part.len();
                let index_part = &part[start + 1..end - 1];
                let base_field = &part[..start];

                if !base_field.is_empty() {
                    marker.push(true);
                }

                if index_part.contains('-') && !index_part.starts_with('-') {
                    let range_parts: Vec<&str> = index_part.split('-').collect();
                    if range_parts.len() == 2 {
                        let start_idx: usize = range_parts[0].parse().unwrap_or(0);
                        let end_idx: usize = range_parts[1].parse().unwrap_or(0);
                        result.push(TreePath::Range(start_idx, end_idx, base_field.to_string()));
                    } else {
                        result.push(TreePath::Field(part.to_string()));
                    }
                } else if let Some(idx_str) = index_part.strip_prefix('-') {
                    let idx: usize = idx_str.parse().unwrap_or(0);
                    result.push(TreePath::NegativeIndex(idx, base_field.to_string()));
                } else {
                    let idx: usize = index_part.parse().unwrap_or(0);
                    result.push(TreePath::Index(idx, base_field.to_string()));
                }
            } else if part == "*" {
                result.push(TreePath::Wildcard("*".to_string()));
            } else {
                result.push(TreePath::Field(part.to_string()));
            }
        } else {
            let part = current;
            current = "";

            if let Some(start) = part.rfind('[') {
                let end = part.len();
                let index_part = &part[start + 1..end - 1];
                let base_field = &part[..start];

                if !base_field.is_empty() {
                    marker.push(true);
                }

                if index_part.contains('-') && !index_part.starts_with('-') {
                    let range_parts: Vec<&str> = index_part.split('-').collect();
                    if range_parts.len() == 2 {
                        let start_idx: usize = range_parts[0].parse().unwrap_or(0);
                        let end_idx: usize = range_parts[1].parse().unwrap_or(0);
                        result.push(TreePath::Range(start_idx, end_idx, base_field.to_string()));
                    } else {
                        result.push(TreePath::Field(part.to_string()));
                    }
                } else if let Some(idx_str) = index_part.strip_prefix('-') {
                    let idx: usize = idx_str.parse().unwrap_or(0);
                    result.push(TreePath::NegativeIndex(idx, base_field.to_string()));
                } else {
                    let idx: usize = index_part.parse().unwrap_or(0);
                    result.push(TreePath::Index(idx, base_field.to_string()));
                }
            } else if part == "*" {
                result.push(TreePath::Wildcard("*".to_string()));
            } else {
                result.push(TreePath::Field(part.to_string()));
            }
        }
    }

    result
}

/// 从嵌套结构中提取字符串值（带策略）
pub fn extract_value_with_strategy(
    document: &Value,
    path: &[TreePath],
    strategy: EvaluationStrategy,
) -> Result<String, PathParseError> {
    let mut current = document;

    for segment in path {
        let next = match segment {
            TreePath::Field(name) => match current.get(name) {
                Some(v) => v,
                None => {
                    if strategy == EvaluationStrategy::Strict {
                        return Err(PathParseError::NotFoundError(format!(
                            "Field '{}' not found",
                            name
                        )));
                    }
                    return Ok(String::new());
                }
            },
            TreePath::Index(idx, field) => match current.get(field) {
                Some(v) => match v.as_array() {
                    Some(arr) => match arr.get(*idx) {
                        Some(v) => v,
                        None => {
                            if strategy == EvaluationStrategy::Strict {
                                return Err(PathParseError::OutOfBoundsError(format!(
                                    "Index {} out of bounds for field '{}'",
                                    idx, field
                                )));
                            }
                            return Ok(String::new());
                        }
                    },
                    None => {
                        if strategy == EvaluationStrategy::Strict {
                            return Err(PathParseError::TypeError(format!(
                                "Field '{}' is not an array",
                                field
                            )));
                        }
                        return Ok(String::new());
                    }
                },
                None => {
                    if strategy == EvaluationStrategy::Strict {
                        return Err(PathParseError::NotFoundError(format!(
                            "Field '{}' not found",
                            field
                        )));
                    }
                    return Ok(String::new());
                }
            },
            TreePath::NegativeIndex(idx, field) => match current.get(field) {
                Some(v) => match v.as_array() {
                    Some(arr) => {
                        let pos = arr.len().saturating_sub(*idx);
                        match arr.get(pos) {
                            Some(v) => v,
                            None => {
                                if strategy == EvaluationStrategy::Strict {
                                    return Err(PathParseError::OutOfBoundsError(format!(
                                        "Negative index -{} out of bounds for field '{}'",
                                        idx, field
                                    )));
                                }
                                return Ok(String::new());
                            }
                        }
                    }
                    None => {
                        if strategy == EvaluationStrategy::Strict {
                            return Err(PathParseError::TypeError(format!(
                                "Field '{}' is not an array",
                                field
                            )));
                        }
                        return Ok(String::new());
                    }
                },
                None => {
                    if strategy == EvaluationStrategy::Strict {
                        return Err(PathParseError::NotFoundError(format!(
                            "Field '{}' not found",
                            field
                        )));
                    }
                    return Ok(String::new());
                }
            },
            TreePath::Range(start, end, field) => match current.get(field) {
                Some(v) => match v.as_array() {
                    Some(arr) => {
                        if *start >= arr.len() || *end >= arr.len() || *start > *end {
                            if strategy == EvaluationStrategy::Strict {
                                return Err(PathParseError::OutOfBoundsError(format!(
                                    "Range [{}-{}] out of bounds for field '{}'",
                                    start, end, field
                                )));
                            }
                            return Ok(String::new());
                        }
                        let values: Vec<&Value> = arr[*start..=*end].iter().collect();
                        return Ok(values
                            .iter()
                            .filter_map(|v| v.as_str())
                            .collect::<Vec<_>>()
                            .join(", "));
                    }
                    None => {
                        if strategy == EvaluationStrategy::Strict {
                            return Err(PathParseError::TypeError(format!(
                                "Field '{}' is not an array",
                                field
                            )));
                        }
                        return Ok(String::new());
                    }
                },
                None => {
                    if strategy == EvaluationStrategy::Strict {
                        return Err(PathParseError::NotFoundError(format!(
                            "Field '{}' not found",
                            field
                        )));
                    }
                    return Ok(String::new());
                }
            },
            TreePath::Wildcard(pattern) => match current {
                Value::Object(obj) => {
                    let matched: Vec<&Value> = obj
                        .keys()
                        .filter(|k| k.contains(pattern))
                        .filter_map(|k| current.get(k))
                        .collect();
                    if matched.is_empty() {
                        if strategy == EvaluationStrategy::Strict {
                            return Err(PathParseError::NotFoundError(format!(
                                "No fields matching pattern '{}'",
                                pattern
                            )));
                        }
                        return Ok(String::new());
                    }
                    return Ok(matched
                        .iter()
                        .filter_map(|v| v.as_str())
                        .collect::<Vec<_>>()
                        .join(", "));
                }
                _ => {
                    if strategy == EvaluationStrategy::Strict {
                        return Err(PathParseError::TypeError(
                            "Cannot apply wildcard to non-object type".to_string(),
                        ));
                    }
                    return Ok(String::new());
                }
            },
        };
        current = next;
    }

    match current {
        Value::String(s) => Ok(s.clone()),
        Value::Number(n) => Ok(n.to_string()),
        Value::Bool(b) => Ok(b.to_string()),
        Value::Null => Ok(String::new()),
        _ => Ok(current.to_string()),
    }
}

/// 从嵌套结构中提取字符串值
pub fn extract_value(document: &Value, path: &[TreePath]) -> Option<String> {
    match extract_value_with_strategy(document, path, EvaluationStrategy::Lenient) {
        Ok(value) if !value.is_empty() => Some(value),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_simple_path() {
        let mut marker = vec![];
        let result = parse_tree("user.name", &mut marker);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], TreePath::Field("user".to_string()));
        assert_eq!(result[1], TreePath::Field("name".to_string()));
    }

    #[test]
    fn test_parse_array_index() {
        let mut marker = vec![];
        let result = parse_tree("items[0].title", &mut marker);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], TreePath::Index(0, "items".to_string()));
        assert_eq!(result[1], TreePath::Field("title".to_string()));
    }

    #[test]
    fn test_parse_negative_index() {
        let mut marker = vec![];
        let result = parse_tree("items[-1].name", &mut marker);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], TreePath::NegativeIndex(1, "items".to_string()));
        assert_eq!(result[1], TreePath::Field("name".to_string()));
    }

    #[test]
    fn test_parse_range_index() {
        let mut marker = vec![];
        let result = parse_tree("items[0-2].title", &mut marker);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], TreePath::Range(0, 2, "items".to_string()));
        assert_eq!(result[1], TreePath::Field("title".to_string()));
    }

    #[test]
    fn test_extract_value_simple() {
        let doc = json!({"user": {"name": "John"}});
        let mut marker = vec![];
        let path = parse_tree("user.name", &mut marker);
        let result = extract_value(&doc, &path);
        assert_eq!(result, Some("John".to_string()));
    }

    #[test]
    fn test_extract_value_array() {
        let doc = json!({"items": [{"title": "A"}, {"title": "B"}]});
        let mut marker = vec![];
        let path = parse_tree("items[0].title", &mut marker);
        let result = extract_value(&doc, &path);
        assert_eq!(result, Some("A".to_string()));
    }

    #[test]
    fn test_extract_value_negative_index() {
        let doc = json!({"items": [{"title": "A"}, {"title": "B"}]});
        let mut marker = vec![];
        let path = parse_tree("items[-1].title", &mut marker);
        let result = extract_value(&doc, &path);
        assert_eq!(result, Some("B".to_string()));
    }
}
