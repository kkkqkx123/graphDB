use crate::encoder::Encoder;
use crate::error::Result;
use serde_json::Value;

pub fn parse_simple(document: &Value, path: &str) -> Result<String> {
    let mut current = document;
    let parts: Vec<&str> = path.split('.').collect();
    
    for (i, part) in parts.iter().enumerate() {
        if let Some(value) = current.get(part) {
            current = value;
        } else {
            return Ok(String::new());
        }
        
        // Handle array indices
        if i < parts.len() - 1 {
            if let Some(index_str) = parts.get(i + 1) {
                if let Ok(index) = index_str.parse::<usize>() {
                    if let Some(array) = current.as_array() {
                        if index < array.len() {
                            current = &array[index];
                        } else {
                            return Ok(String::new());
                        }
                    }
                }
            }
        }
    }
    
    match current {
        Value::String(s) => Ok(s.clone()),
        Value::Number(n) => Ok(n.to_string()),
        Value::Bool(b) => Ok(b.to_string()),
        _ => Ok(String::new()),
    }
}