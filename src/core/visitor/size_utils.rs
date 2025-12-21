//! 内存大小计算工具类
//!
//! 提供Value类型的内存大小计算功能，支持递归计算和配置选项

use crate::core::value::{
    DataSet, DateTimeValue, DateValue, DurationValue, GeographyValue, NullType, TimeValue, Value,
};
use crate::core::vertex_edge_path::{Edge, Path, Step, Tag, Vertex};
use std::collections::{HashMap, HashSet};

/// 大小计算错误类型
#[derive(Debug, thiserror::Error)]
pub enum SizeError {
    #[error("大小计算错误: {0}")]
    Calculation(String),
    #[error("递归深度超过限制")]
    MaxDepthExceeded,
}

/// 大小计算器配置
#[derive(Debug, Clone)]
pub struct SizeConfig {
    /// 最大递归深度
    pub max_depth: usize,
    /// 是否计算字符串内容的实际大小
    pub include_string_content: bool,
    /// 是否计算集合容器的开销
    pub include_container_overhead: bool,
}

impl Default for SizeConfig {
    fn default() -> Self {
        Self {
            max_depth: 100,
            include_string_content: true,
            include_container_overhead: true,
        }
    }
}

impl SizeConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = max_depth;
        self
    }

    pub fn with_string_content(mut self, include: bool) -> Self {
        self.include_string_content = include;
        self
    }

    pub fn with_container_overhead(mut self, include: bool) -> Self {
        self.include_container_overhead = include;
        self
    }
}

/// 内存大小计算器
#[derive(Debug)]
pub struct ValueSizeCalculator {
    config: SizeConfig,
    depth: usize,
}

impl ValueSizeCalculator {
    /// 创建新的大小计算器
    pub fn new() -> Self {
        Self::with_config(SizeConfig::default())
    }

    /// 使用配置创建大小计算器
    pub fn with_config(config: SizeConfig) -> Self {
        Self { config, depth: 0 }
    }

    /// 计算Value的内存大小
    pub fn calculate_size(value: &Value) -> Result<usize, SizeError> {
        Self::new().size_of(value)
    }

    /// 使用配置计算Value的内存大小
    pub fn calculate_size_with_config(
        value: &Value,
        config: SizeConfig,
    ) -> Result<usize, SizeError> {
        Self::with_config(config).size_of(value)
    }

    /// 计算大小
    pub fn size_of(&mut self, value: &Value) -> Result<usize, SizeError> {
        if self.depth > self.config.max_depth {
            return Err(SizeError::MaxDepthExceeded);
        }

        self.depth += 1;
        let result = match value {
            Value::Bool(v) => self.size_of_bool(*v),
            Value::Int(v) => self.size_of_int(*v),
            Value::Float(v) => self.size_of_float(*v),
            Value::String(v) => self.size_of_string(v),
            Value::Date(v) => self.size_of_date(v),
            Value::Time(v) => self.size_of_time(v),
            Value::DateTime(v) => self.size_of_datetime(v),
            Value::Vertex(v) => self.size_of_vertex(v),
            Value::Edge(v) => self.size_of_edge(v),
            Value::Path(v) => self.size_of_path(v),
            Value::List(v) => self.size_of_list(v),
            Value::Map(v) => self.size_of_map(v),
            Value::Set(v) => self.size_of_set(v),
            Value::Geography(v) => self.size_of_geography(v),
            Value::Duration(v) => self.size_of_duration(v),
            Value::DataSet(v) => self.size_of_dataset(v),
            Value::Null(v) => self.size_of_null(v),
            Value::Empty => self.size_of_empty(),
        };
        self.depth -= 1;
        result
    }
    
    /// 内部计算大小的方法，用于递归调用
    fn size_of_internal(&mut self, value: &Value) -> Result<usize, SizeError> {
        if self.depth > self.config.max_depth {
            return Err(SizeError::MaxDepthExceeded);
        }

        self.depth += 1;
        let result = match value {
            Value::Bool(v) => self.size_of_bool(*v),
            Value::Int(v) => self.size_of_int(*v),
            Value::Float(v) => self.size_of_float(*v),
            Value::String(v) => self.size_of_string(v),
            Value::Date(v) => self.size_of_date(v),
            Value::Time(v) => self.size_of_time(v),
            Value::DateTime(v) => self.size_of_datetime(v),
            Value::Vertex(v) => self.size_of_vertex(v),
            Value::Edge(v) => self.size_of_edge(v),
            Value::Path(v) => self.size_of_path(v),
            Value::List(v) => self.size_of_list(v),
            Value::Map(v) => self.size_of_map(v),
            Value::Set(v) => self.size_of_set(v),
            Value::Geography(v) => self.size_of_geography(v),
            Value::Duration(v) => self.size_of_duration(v),
            Value::DataSet(v) => self.size_of_dataset(v),
            Value::Null(v) => self.size_of_null(v),
            Value::Empty => self.size_of_empty(),
        };
        self.depth -= 1;
        result
    }

    fn size_of_bool(&self, _value: bool) -> Result<usize, SizeError> {
        Ok(std::mem::size_of::<bool>())
    }

    fn size_of_int(&self, _value: i64) -> Result<usize, SizeError> {
        Ok(std::mem::size_of::<i64>())
    }

    fn size_of_float(&self, _value: f64) -> Result<usize, SizeError> {
        Ok(std::mem::size_of::<f64>())
    }

    fn size_of_string(&self, value: &str) -> Result<usize, SizeError> {
        let mut size = std::mem::size_of::<String>();
        if self.config.include_string_content {
            size += value.len();
        }
        Ok(size)
    }

    fn size_of_date(&self, _value: &DateValue) -> Result<usize, SizeError> {
        Ok(std::mem::size_of::<DateValue>())
    }

    fn size_of_time(&self, _value: &TimeValue) -> Result<usize, SizeError> {
        Ok(std::mem::size_of::<TimeValue>())
    }

    fn size_of_datetime(&self, _value: &DateTimeValue) -> Result<usize, SizeError> {
        Ok(std::mem::size_of::<DateTimeValue>())
    }

    fn size_of_vertex(&mut self, value: &Vertex) -> Result<usize, SizeError> {
        let mut size = std::mem::size_of::<Vertex>();
        
        // 计算ID的大小
        size += std::mem::size_of_val(value.id());
        
        // 计算标签的大小
        for tag in value.tags() {
            size += std::mem::size_of::<Tag>();
            size += tag.name.len();
            for (prop_name, prop_value) in &tag.properties {
                size += prop_name.len();
                size += self.size_of_internal(prop_value)?;
            }
        }
        
        // 计算顶点属性的大小
        for (prop_name, prop_value) in value.vertex_properties() {
            size += prop_name.len();
            size += self.size_of_internal(prop_value)?;
        }
        
        Ok(size)
    }

    fn size_of_edge(&mut self, value: &Edge) -> Result<usize, SizeError> {
        let mut size = std::mem::size_of::<Edge>();
        size += std::mem::size_of_val(&value.src);
        size += std::mem::size_of_val(&value.dst);
        size += value.edge_type.len();
        
        for (prop_name, prop_value) in value.get_all_properties() {
            size += prop_name.len();
            size += self.size_of_internal(prop_value)?;
        }
        
        Ok(size)
    }

    fn size_of_path(&mut self, value: &Path) -> Result<usize, SizeError> {
        let mut size = std::mem::size_of::<Path>();
        size += self.size_of_internal(&Value::Vertex(Box::new(value.src.as_ref().clone())))?;
        
        for step in &value.steps {
            size += std::mem::size_of::<Step>();
            size += self.size_of_internal(&Value::Vertex(Box::new(step.dst.as_ref().clone())))?;
            size += self.size_of_internal(&Value::Edge(step.edge.as_ref().clone()))?;
        }
        
        Ok(size)
    }

    fn size_of_list(&mut self, value: &[Value]) -> Result<usize, SizeError> {
        let mut size = if self.config.include_container_overhead {
            std::mem::size_of::<Vec<Value>>()
        } else {
            0
        };
        
        for item in value {
            size += self.size_of_internal(item)?;
        }
        
        Ok(size)
    }

    fn size_of_map(&mut self, value: &HashMap<String, Value>) -> Result<usize, SizeError> {
        let mut size = if self.config.include_container_overhead {
            std::mem::size_of::<HashMap<String, Value>>()
        } else {
            0
        };
        
        for (key, val) in value {
            size += key.len();
            size += self.size_of_internal(val)?;
        }
        
        Ok(size)
    }

    fn size_of_set(&mut self, value: &HashSet<Value>) -> Result<usize, SizeError> {
        let mut size = if self.config.include_container_overhead {
            std::mem::size_of::<HashSet<Value>>()
        } else {
            0
        };
        
        for item in value {
            size += self.size_of_internal(item)?;
        }
        
        Ok(size)
    }

    fn size_of_geography(&self, value: &GeographyValue) -> Result<usize, SizeError> {
        let mut size = std::mem::size_of::<GeographyValue>();

        // 计算地理数据的大小
        if let Some(_) = value.point {
            size += std::mem::size_of::<(f64, f64)>();
        }
        if let Some(ref line) = value.linestring {
            size += std::mem::size_of::<Vec<(f64, f64)>>()
                + line.len() * std::mem::size_of::<(f64, f64)>();
        }
        if let Some(ref poly) = value.polygon {
            size += std::mem::size_of::<Vec<Vec<(f64, f64)>>>();
            for ring in poly {
                size += std::mem::size_of::<Vec<(f64, f64)>>();
                size += ring.len() * std::mem::size_of::<(f64, f64)>();
            }
        }

        Ok(size)
    }

    fn size_of_duration(&self, _value: &DurationValue) -> Result<usize, SizeError> {
        Ok(std::mem::size_of::<DurationValue>())
    }

    fn size_of_dataset(&mut self, value: &DataSet) -> Result<usize, SizeError> {
        let mut size = std::mem::size_of::<DataSet>();
        
        if self.config.include_container_overhead {
            size += value.col_names.len() * std::mem::size_of::<String>();
        }
        
        for row in &value.rows {
            if self.config.include_container_overhead {
                size += std::mem::size_of::<Vec<Value>>();
            }
            for cell in row {
                size += self.size_of_internal(cell)?;
            }
        }
        
        Ok(size)
    }

    fn size_of_null(&self, _null_type: &NullType) -> Result<usize, SizeError> {
        Ok(std::mem::size_of::<NullType>())
    }

    fn size_of_empty(&self) -> Result<usize, SizeError> {
        Ok(std::mem::size_of::<Value>())
    }
}

/// 便捷函数：计算Value的内存大小
pub fn calculate_size(value: &Value) -> Result<usize, SizeError> {
    let mut calculator = ValueSizeCalculator::new();
    calculator.size_of(value)
}

/// 便捷函数：使用配置计算Value的内存大小
pub fn calculate_size_with_config(value: &Value, config: SizeConfig) -> Result<usize, SizeError> {
    let mut calculator = ValueSizeCalculator::with_config(config);
    calculator.size_of(value)
}

/// 便捷函数：计算多个Value的总大小
pub fn calculate_total_size(values: &[Value]) -> Result<usize, SizeError> {
    let mut total = 0;
    for value in values {
        total += calculate_size(value)?;
    }
    Ok(total)
}

/// 便捷函数：估算Value的内存大小（快速但不精确）
pub fn estimate_size(value: &Value) -> usize {
    // 使用默认配置但限制深度以提高性能
    let config = SizeConfig::new()
        .with_max_depth(10)
        .with_container_overhead(false);
    calculate_size_with_config(value, config).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::value::Value;

    #[test]
    fn test_basic_size_calculation() {
        let value = Value::Int(42);
        let size = calculate_size(&value).unwrap();
        assert_eq!(size, std::mem::size_of::<i64>());
    }

    #[test]
    fn test_string_size_calculation() {
        let value = Value::String("hello".to_string());
        let size = calculate_size(&value).unwrap();
        assert!(size > std::mem::size_of::<String>());
    }

    #[test]
    fn test_list_size_calculation() {
        let value = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        let size = calculate_size(&value).unwrap();
        assert!(size > std::mem::size_of::<i64>() * 3);
    }

    #[test]
    fn test_config_options() {
        let value = Value::String("test".to_string());

        let config_with_content = SizeConfig::new().with_string_content(true);
        let size_with_content = calculate_size_with_config(&value, config_with_content).unwrap();

        let config_without_content = SizeConfig::new().with_string_content(false);
        let size_without_content =
            calculate_size_with_config(&value, config_without_content).unwrap();

        assert!(size_with_content > size_without_content);
    }

    #[test]
    fn test_estimate_size() {
        let value = Value::Int(42);
        let estimated = estimate_size(&value);
        assert_eq!(estimated, std::mem::size_of::<i64>());
    }
}
