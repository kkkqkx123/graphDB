//! 树形结构解析
//!
//! 解析嵌套字段路径，支持数组索引和属性访问
//!
//! # 支持的语法
//!
//! ```rust
//! use inversearch::parse_tree;
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
}

/// 解析树形路径
///
/// # 示例
///
/// ```
/// use inversearch::parse_tree;
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
                let index_part = &part[start+1..end-1];
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
                } else if index_part.starts_with('-') {
                    let idx: usize = index_part[1..].parse().unwrap_or(0);
                    result.push(TreePath::NegativeIndex(idx, base_field.to_string()));
                } else {
                    let idx: usize = index_part.parse().unwrap_or(0);
                    result.push(TreePath::Index(idx, base_field.to_string()));
                }
            } else {
                result.push(TreePath::Field(part.to_string()));
            }
        } else {
            let part = current;
            current = "";
            
            if let Some(start) = part.rfind('[') {
                let end = part.len();
                let index_part = &part[start+1..end-1];
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
                } else if index_part.starts_with('-') {
                    let idx: usize = index_part[1..].parse().unwrap_or(0);
                    result.push(TreePath::NegativeIndex(idx, base_field.to_string()));
                } else {
                    let idx: usize = index_part.parse().unwrap_or(0);
                    result.push(TreePath::Index(idx, base_field.to_string()));
                }
            } else {
                result.push(TreePath::Field(part.to_string()));
            }
        }
    }
    
    result
}

/// 从嵌套结构中提取字符串值
pub fn extract_value(document: &Value, path: &[TreePath]) -> Option<String> {
    let mut current = document;
    
    for segment in path {
        current = match segment {
            TreePath::Field(name) => {
                current.get(name)?
            }
            TreePath::Index(idx, field) => {
                let arr = current.get(field)?.as_array()?;
                arr.get(*idx)?
            }
            TreePath::NegativeIndex(idx, field) => {
                let arr = current.get(field)?.as_array()?;
                let pos = arr.len().saturating_sub(*idx);
                arr.get(pos)?
            }
            TreePath::Range(_, _, _) => {
                return None;
            }
        };
    }
    
    match current {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        Value::Null => Some(String::new()),
        _ => Some(current.to_string()),
    }
}

/// 检查路径是否存在
pub fn path_exists(document: &Value, path: &[TreePath]) -> bool {
    let mut current = document;
    
    for segment in path {
        current = match segment {
            TreePath::Field(name) => {
                match current.get(name) {
                    Some(v) => v,
                    None => return false,
                }
            }
            TreePath::Index(idx, field) => {
                match current.get(field) {
                    Some(v) => match v.as_array() {
                        Some(arr) => match arr.get(*idx) {
                            Some(v) => v,
                            None => return false,
                        },
                        None => return false,
                    },
                    None => return false,
                }
            }
            TreePath::NegativeIndex(idx, field) => {
                match current.get(field) {
                    Some(v) => match v.as_array() {
                        Some(arr) => {
                            let pos = arr.len().saturating_sub(*idx + 1);
                            match arr.get(pos) {
                                Some(v) => v,
                                None => return false,
                            }
                        }
                        None => return false,
                    },
                    None => return false,
                }
            }
            TreePath::Range(_, _, _) => {
                return true;
            }
        };
    }
    
    true
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

    #[test]
    fn test_path_exists() {
        let doc = json!({"user": {"name": "John"}});
        let mut marker = vec![];
        let path = parse_tree("user.name", &mut marker);
        assert!(path_exists(&doc, &path));
        
        let path = parse_tree("user.age", &mut marker);
        assert!(!path_exists(&doc, &path));
    }
}
