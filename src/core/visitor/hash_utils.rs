//! 哈希计算工具类
//!
//! 提供Value类型的哈希计算功能，支持一致性哈希和自定义哈希算法

use crate::core::value::{
    DataSet, DateTimeValue, DateValue, DurationValue, GeographyValue, NullType, TimeValue, Value,
};
use crate::core::vertex_edge_path::{Edge, Path, Step, Tag, Vertex};
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

/// 哈希计算错误类型
#[derive(Debug, thiserror::Error)]
pub enum HashError {
    #[error("哈希计算错误: {0}")]
    Calculation(String),
    #[error("递归深度超过限制")]
    MaxDepthExceeded,
}

/// 哈希计算器配置
#[derive(Debug, Clone)]
pub struct HashConfig {
    /// 最大递归深度
    pub max_depth: usize,
    /// 是否对集合元素进行排序以确保一致性
    pub sort_collections: bool,
    /// 浮点数哈希精度
    pub float_precision: Option<u32>,
}

impl Default for HashConfig {
    fn default() -> Self {
        Self {
            max_depth: 100,
            sort_collections: true,
            float_precision: None,
        }
    }
}

impl HashConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = max_depth;
        self
    }

    pub fn with_sort_collections(mut self, sort: bool) -> Self {
        self.sort_collections = sort;
        self
    }

    pub fn with_float_precision(mut self, precision: u32) -> Self {
        self.float_precision = Some(precision);
        self
    }
}

/// 哈希计算工具类
#[derive(Debug)]
pub struct ValueHasher {
    hasher: std::collections::hash_map::DefaultHasher,
    config: HashConfig,
    depth: usize,
}

impl ValueHasher {
    /// 创建新的哈希计算器
    pub fn new() -> Self {
        Self::with_config(HashConfig::default())
    }

    /// 使用配置创建哈希计算器
    pub fn with_config(config: HashConfig) -> Self {
        Self {
            hasher: std::collections::hash_map::DefaultHasher::new(),
            config,
            depth: 0,
        }
    }

    /// 计算Value的哈希值
    pub fn hash_value(value: &Value) -> Result<u64, HashError> {
        Self::new().hash(value)
    }

    /// 使用配置计算Value的哈希值
    pub fn hash_value_with_config(value: &Value, config: HashConfig) -> Result<u64, HashError> {
        Self::with_config(config).hash(value)
    }

    /// 计算哈希值
    pub fn hash(mut self, value: &Value) -> Result<u64, HashError> {
        self.visit_value(value)?;
        Ok(self.hasher.finish())
    }

    /// 访问Value并计算哈希
    fn visit_value(&mut self, value: &Value) -> Result<(), HashError> {
        if self.depth > self.config.max_depth {
            return Err(HashError::MaxDepthExceeded);
        }

        self.depth += 1;
        let result = match value {
            Value::Bool(v) => self.visit_bool(*v),
            Value::Int(v) => self.visit_int(*v),
            Value::Float(v) => self.visit_float(*v),
            Value::String(v) => self.visit_string(v),
            Value::Date(v) => self.visit_date(v),
            Value::Time(v) => self.visit_time(v),
            Value::DateTime(v) => self.visit_datetime(v),
            Value::Vertex(v) => self.visit_vertex(v),
            Value::Edge(v) => self.visit_edge(v),
            Value::Path(v) => self.visit_path(v),
            Value::List(v) => self.visit_list(v),
            Value::Map(v) => self.visit_map(v),
            Value::Set(v) => self.visit_set(v),
            Value::Geography(v) => self.visit_geography(v),
            Value::Duration(v) => self.visit_duration(v),
            Value::DataSet(v) => self.visit_dataset(v),
            Value::Null(v) => self.visit_null(v),
            Value::Empty => self.visit_empty(),
        };
        self.depth -= 1;
        result
    }

    fn visit_bool(&mut self, value: bool) -> Result<(), HashError> {
        value.hash(&mut self.hasher);
        Ok(())
    }

    fn visit_int(&mut self, value: i64) -> Result<(), HashError> {
        value.hash(&mut self.hasher);
        Ok(())
    }

    fn visit_float(&mut self, value: f64) -> Result<(), HashError> {
        // 特殊处理浮点数的哈希
        if let Some(precision) = self.config.float_precision {
            // 使用指定精度进行哈希
            let scaled = (value * 10_f64.powi(precision as i32)).round();
            scaled.to_bits().hash(&mut self.hasher);
        } else if value.is_nan() {
            (0x7ff80000u32 as u64).hash(&mut self.hasher);
        } else if value == 0.0 {
            0.0_f64.to_bits().hash(&mut self.hasher);
        } else {
            value.to_bits().hash(&mut self.hasher);
        }
        Ok(())
    }

    fn visit_string(&mut self, value: &str) -> Result<(), HashError> {
        value.hash(&mut self.hasher);
        Ok(())
    }

    fn visit_date(&mut self, value: &DateValue) -> Result<(), HashError> {
        value.hash(&mut self.hasher);
        Ok(())
    }

    fn visit_time(&mut self, value: &TimeValue) -> Result<(), HashError> {
        value.hash(&mut self.hasher);
        Ok(())
    }

    fn visit_datetime(&mut self, value: &DateTimeValue) -> Result<(), HashError> {
        value.hash(&mut self.hasher);
        Ok(())
    }

    fn visit_vertex(&mut self, value: &Vertex) -> Result<(), HashError> {
        value.hash(&mut self.hasher);
        Ok(())
    }

    fn visit_edge(&mut self, value: &Edge) -> Result<(), HashError> {
        value.hash(&mut self.hasher);
        Ok(())
    }

    fn visit_path(&mut self, value: &Path) -> Result<(), HashError> {
        value.hash(&mut self.hasher);
        Ok(())
    }

    fn visit_list(&mut self, value: &[Value]) -> Result<(), HashError> {
        value.len().hash(&mut self.hasher);
        for item in value {
            self.visit_value(item)?;
        }
        Ok(())
    }

    fn visit_map(&mut self, value: &HashMap<String, Value>) -> Result<(), HashError> {
        value.len().hash(&mut self.hasher);
        
        if self.config.sort_collections {
            // 对键值对进行排序以确保一致的哈希
            let mut pairs: Vec<_> = value.iter().collect();
            pairs.sort_by_key(|&(k, _)| k);
            for (k, v) in pairs {
                k.hash(&mut self.hasher);
                self.visit_value(v)?;
            }
        } else {
            for (k, v) in value {
                k.hash(&mut self.hasher);
                self.visit_value(v)?;
            }
        }
        Ok(())
    }

    fn visit_set(&mut self, value: &HashSet<Value>) -> Result<(), HashError> {
        value.len().hash(&mut self.hasher);
        
        if self.config.sort_collections {
            // 对集合元素进行排序以确保一致的哈希
            let mut items: Vec<_> = value.iter().collect();
            items.sort_by(|a, b| {
                let hash_a = Self::hash_value_with_config(a, self.config.clone()).unwrap_or_else(|_| 0);
                let hash_b = Self::hash_value_with_config(b, self.config.clone()).unwrap_or_else(|_| 0);
                hash_a.cmp(&hash_b)
            });
            for item in items {
                self.visit_value(item)?;
            }
        } else {
            for item in value {
                self.visit_value(item)?;
            }
        }
        Ok(())
    }

    fn visit_geography(&mut self, value: &GeographyValue) -> Result<(), HashError> {
        value.hash(&mut self.hasher);
        Ok(())
    }

    fn visit_duration(&mut self, value: &DurationValue) -> Result<(), HashError> {
        value.hash(&mut self.hasher);
        Ok(())
    }

    fn visit_dataset(&mut self, value: &DataSet) -> Result<(), HashError> {
        value.hash(&mut self.hasher);
        Ok(())
    }

    fn visit_null(&mut self, null_type: &NullType) -> Result<(), HashError> {
        null_type.hash(&mut self.hasher);
        Ok(())
    }

    fn visit_empty(&mut self) -> Result<(), HashError> {
        0u8.hash(&mut self.hasher);
        Ok(())
    }
}

/// 便捷函数：计算Value的哈希值
pub fn calculate_hash(value: &Value) -> Result<u64, HashError> {
    ValueHasher::hash_value(value)
}

/// 便捷函数：使用配置计算Value的哈希值
pub fn calculate_hash_with_config(value: &Value, config: HashConfig) -> Result<u64, HashError> {
    ValueHasher::hash_value_with_config(value, config)
}

/// 便捷函数：计算多个Value的组合哈希值
pub fn calculate_combined_hash(values: &[Value]) -> Result<u64, HashError> {
    let mut hasher = ValueHasher::new();
    for value in values {
        hasher.visit_value(value)?;
    }
    Ok(hasher.hasher.finish())
}

/// 便捷函数：检查两个Value是否具有相同的哈希值
pub fn hash_equal(value1: &Value, value2: &Value) -> Result<bool, HashError> {
    let hash1 = calculate_hash(value1)?;
    let hash2 = calculate_hash(value2)?;
    Ok(hash1 == hash2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::value::Value;
    use std::collections::HashMap;

    #[test]
    fn test_basic_hashing() {
        let value1 = Value::Int(42);
        let value2 = Value::Int(42);
        let hash1 = calculate_hash(&value1).unwrap();
        let hash2 = calculate_hash(&value2).unwrap();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_float_hashing() {
        let value1 = Value::Float(3.14159);
        let value2 = Value::Float(3.14159);
        let hash1 = calculate_hash(&value1).unwrap();
        let hash2 = calculate_hash(&value2).unwrap();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_float_precision() {
        let value1 = Value::Float(3.14159265359);
        let value2 = Value::Float(3.14159265358);
        
        let config = HashConfig::new().with_float_precision(5);
        let hash1 = calculate_hash_with_config(&value1, config.clone()).unwrap();
        let hash2 = calculate_hash_with_config(&value2, config).unwrap();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_collection_ordering() {
        let mut map1 = HashMap::new();
        map1.insert("a".to_string(), Value::Int(1));
        map1.insert("b".to_string(), Value::Int(2));
        
        let mut map2 = HashMap::new();
        map2.insert("b".to_string(), Value::Int(2));
        map2.insert("a".to_string(), Value::Int(1));
        
        let config = HashConfig::new().with_sort_collections(true);
        let hash1 = calculate_hash_with_config(&Value::Map(map1), config.clone()).unwrap();
        let hash2 = calculate_hash_with_config(&Value::Map(map2), config).unwrap();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_combined_hash() {
        let values = vec![
            Value::Int(1),
            Value::String("test".to_string()),
            Value::Bool(true),
        ];
        let hash = calculate_combined_hash(&values).unwrap();
        assert!(hash > 0);
    }

    #[test]
    fn test_hash_equal() {
        let value1 = Value::String("hello".to_string());
        let value2 = Value::String("hello".to_string());
        let value3 = Value::String("world".to_string());
        
        assert!(hash_equal(&value1, &value2).unwrap());
        assert!(!hash_equal(&value1, &value3).unwrap());
    }
}