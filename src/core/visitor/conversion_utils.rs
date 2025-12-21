//! 类型转换工具类
//!
//! 提供Value类型之间的转换功能，支持自定义转换规则和配置

use crate::core::value::{
    DataSet, DateTimeValue, DateValue, DurationValue, GeographyValue, NullType, TimeValue, Value,
    ValueTypeDef,
};
use crate::core::vertex_edge_path::{Edge, Path, Vertex};
use std::collections::{HashMap, HashSet};

/// 类型转换错误类型
#[derive(Debug, thiserror::Error)]
pub enum ConversionError {
    #[error("转换错误: {0}")]
    Conversion(String),
    #[error("不支持的类型转换: {from} -> {to}")]
    UnsupportedConversion { from: String, to: String },
    #[error("转换失败: {reason}")]
    Failed { reason: String },
}

/// 类型转换器配置
#[derive(Debug, Clone)]
pub struct ConversionConfig {
    /// 是否允许宽松转换（如字符串到数字）
    pub lenient_conversion: bool,
    /// 浮点数转换精度
    pub float_precision: Option<u32>,
    /// 日期时间转换格式
    pub datetime_format: String,
    /// 是否在转换失败时返回默认值
    pub fallback_to_default: bool,
}

impl Default for ConversionConfig {
    fn default() -> Self {
        Self {
            lenient_conversion: true,
            float_precision: None,
            datetime_format: "%Y-%m-%d %H:%M:%S".to_string(),
            fallback_to_default: false,
        }
    }
}

impl ConversionConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_lenient(mut self, lenient: bool) -> Self {
        self.lenient_conversion = lenient;
        self
    }

    pub fn with_float_precision(mut self, precision: u32) -> Self {
        self.float_precision = Some(precision);
        self
    }

    pub fn with_datetime_format(mut self, format: String) -> Self {
        self.datetime_format = format;
        self
    }

    pub fn with_fallback(mut self, fallback: bool) -> Self {
        self.fallback_to_default = fallback;
        self
    }
}

/// 类型转换器
#[derive(Debug)]
pub struct ValueConverter {
    config: ConversionConfig,
}

impl ValueConverter {
    /// 创建新的转换器
    pub fn new() -> Self {
        Self::with_config(ConversionConfig::default())
    }

    /// 使用配置创建转换器
    pub fn with_config(config: ConversionConfig) -> Self {
        Self { config }
    }

    /// 转换Value到指定类型
    pub fn convert(&self, value: &Value, target_type: ValueTypeDef) -> Result<Value, ConversionError> {
        match (value, &target_type) {
            // 相同类型，直接返回
            (Value::Bool(_), ValueTypeDef::Bool) => Ok(value.clone()),
            (Value::Int(_), ValueTypeDef::Int) => Ok(value.clone()),
            (Value::Float(_), ValueTypeDef::Float) => Ok(value.clone()),
            (Value::String(_), ValueTypeDef::String) => Ok(value.clone()),
            (Value::Date(_), ValueTypeDef::Date) => Ok(value.clone()),
            (Value::Time(_), ValueTypeDef::Time) => Ok(value.clone()),
            (Value::DateTime(_), ValueTypeDef::DateTime) => Ok(value.clone()),
            (Value::Vertex(_), ValueTypeDef::Vertex) => Ok(value.clone()),
            (Value::Edge(_), ValueTypeDef::Edge) => Ok(value.clone()),
            (Value::Path(_), ValueTypeDef::Path) => Ok(value.clone()),
            (Value::List(_), ValueTypeDef::List) => Ok(value.clone()),
            (Value::Map(_), ValueTypeDef::Map) => Ok(value.clone()),
            (Value::Set(_), ValueTypeDef::Set) => Ok(value.clone()),
            (Value::Geography(_), ValueTypeDef::Geography) => Ok(value.clone()),
            (Value::Duration(_), ValueTypeDef::Duration) => Ok(value.clone()),
            (Value::DataSet(_), ValueTypeDef::DataSet) => Ok(value.clone()),
            (Value::Null(_), ValueTypeDef::Null) => Ok(value.clone()),
            (Value::Empty, ValueTypeDef::Empty) => Ok(value.clone()),

            // 布尔值转换
            (Value::Bool(v), target_type) => self.convert_bool(*v, target_type),
            
            // 整数转换
            (Value::Int(v), target_type) => self.convert_int(*v, target_type),
            
            // 浮点数转换
            (Value::Float(v), target_type) => self.convert_float(*v, target_type),
            
            // 字符串转换
            (Value::String(v), target_type) => self.convert_string(v, target_type),
            
            // 日期转换
            (Value::Date(v), target_type) => self.convert_date(v, target_type),
            
            // 时间转换
            (Value::Time(v), target_type) => self.convert_time(v, target_type),
            
            // 日期时间转换
            (Value::DateTime(v), target_type) => self.convert_datetime(v, target_type),
            
            // 复合类型转换
            (Value::List(v), target_type) => self.convert_list(v, target_type),
            (Value::Map(v), target_type) => self.convert_map(v, target_type),
            (Value::Set(v), target_type) => self.convert_set(v, target_type),
            
            // 其他类型到字符串的转换
            (value, ValueTypeDef::String) => self.to_string(value),
            
            // 不支持的转换
            (value, target_type) => Err(ConversionError::UnsupportedConversion {
                from: self.get_type_name(value),
                to: format!("{:?}", target_type),
            }),
        }
    }

    fn convert_bool(&self, value: bool, target_type: &ValueTypeDef) -> Result<Value, ConversionError> {
        match target_type {
            ValueTypeDef::String => Ok(Value::String(value.to_string())),
            ValueTypeDef::Float => Ok(Value::Float(if value { 1.0 } else { 0.0 })),
            ValueTypeDef::Int => Ok(Value::Int(value as i64)),
            ValueTypeDef::Bool => Ok(Value::Bool(value)),
            _ => Err(ConversionError::UnsupportedConversion {
                from: "Bool".to_string(),
                to: format!("{:?}", target_type),
            }),
        }
    }

    fn convert_int(&self, value: i64, target_type: &ValueTypeDef) -> Result<Value, ConversionError> {
        match target_type {
            ValueTypeDef::String => Ok(Value::String(value.to_string())),
            ValueTypeDef::Float => Ok(Value::Float(value as f64)),
            ValueTypeDef::Int => Ok(Value::Int(value)),
            ValueTypeDef::Bool => Ok(Value::Bool(value != 0)),
            _ => Err(ConversionError::UnsupportedConversion {
                from: "Int".to_string(),
                to: format!("{:?}", target_type),
            }),
        }
    }

    fn convert_float(&self, value: f64, target_type: &ValueTypeDef) -> Result<Value, ConversionError> {
        let converted_value = if let Some(precision) = self.config.float_precision {
            let factor = 10_f64.powi(precision as i32);
            (value * factor).round() / factor
        } else {
            value
        };

        match target_type {
            ValueTypeDef::String => Ok(Value::String(converted_value.to_string())),
            ValueTypeDef::Float => Ok(Value::Float(converted_value)),
            ValueTypeDef::Int => Ok(Value::Int(converted_value as i64)),
            ValueTypeDef::Bool => Ok(Value::Bool(converted_value != 0.0)),
            _ => Err(ConversionError::UnsupportedConversion {
                from: "Float".to_string(),
                to: format!("{:?}", target_type),
            }),
        }
    }

    fn convert_string(&self, value: &str, target_type: &ValueTypeDef) -> Result<Value, ConversionError> {
        match target_type {
            ValueTypeDef::Int => {
                value.parse::<i64>()
                    .map(Value::Int)
                    .map_err(|_| ConversionError::Conversion(format!("无法将字符串 '{}' 转换为整数", value)))
            }
            ValueTypeDef::Float => {
                value.parse::<f64>()
                    .map(Value::Float)
                    .map_err(|_| ConversionError::Conversion(format!("无法将字符串 '{}' 转换为浮点数", value)))
            }
            ValueTypeDef::Bool => {
                match value.to_lowercase().as_str() {
                    "true" | "1" | "yes" | "on" => Ok(Value::Bool(true)),
                    "false" | "0" | "no" | "off" => Ok(Value::Bool(false)),
                    _ if self.config.lenient_conversion => Ok(Value::Bool(!value.is_empty())),
                    _ => Err(ConversionError::Conversion(format!("无法将字符串 '{}' 转换为布尔值", value))),
                }
            }
            ValueTypeDef::String => Ok(Value::String(value.to_string())),
            _ => Err(ConversionError::UnsupportedConversion {
                from: "String".to_string(),
                to: format!("{:?}", target_type),
            }),
        }
    }

    fn convert_date(&self, value: &DateValue, target_type: &ValueTypeDef) -> Result<Value, ConversionError> {
        match target_type {
            ValueTypeDef::String => Ok(Value::String(format!(
                "{}-{}-{}",
                value.year, value.month, value.day
            ))),
            ValueTypeDef::DateTime => Ok(Value::DateTime(DateTimeValue {
                year: value.year,
                month: value.month,
                day: value.day,
                hour: 0,
                minute: 0,
                sec: 0,
                microsec: 0,
            })),
            ValueTypeDef::Date => Ok(Value::Date(value.clone())),
            _ => Err(ConversionError::UnsupportedConversion {
                from: "Date".to_string(),
                to: format!("{:?}", target_type),
            }),
        }
    }

    fn convert_time(&self, value: &TimeValue, target_type: &ValueTypeDef) -> Result<Value, ConversionError> {
        match target_type {
            ValueTypeDef::String => Ok(Value::String(format!(
                "{}:{}:{}",
                value.hour, value.minute, value.sec
            ))),
            ValueTypeDef::DateTime => Ok(Value::DateTime(DateTimeValue {
                year: 1970,
                month: 1,
                day: 1,
                hour: value.hour,
                minute: value.minute,
                sec: value.sec,
                microsec: 0,
            })),
            ValueTypeDef::Time => Ok(Value::Time(value.clone())),
            _ => Err(ConversionError::UnsupportedConversion {
                from: "Time".to_string(),
                to: format!("{:?}", target_type),
            }),
        }
    }

    fn convert_datetime(&self, value: &DateTimeValue, target_type: &ValueTypeDef) -> Result<Value, ConversionError> {
        match target_type {
            ValueTypeDef::String => Ok(Value::String(format!(
                "{}-{}-{} {}:{}:{}",
                value.year, value.month, value.day, value.hour, value.minute, value.sec
            ))),
            ValueTypeDef::Date => Ok(Value::Date(DateValue {
                year: value.year,
                month: value.month,
                day: value.day,
            })),
            ValueTypeDef::Time => Ok(Value::Time(TimeValue {
                hour: value.hour,
                minute: value.minute,
                sec: value.sec,
            })),
            ValueTypeDef::DateTime => Ok(Value::DateTime(value.clone())),
            _ => Err(ConversionError::UnsupportedConversion {
                from: "DateTime".to_string(),
                to: format!("{:?}", target_type),
            }),
        }
    }

    fn convert_list(&self, value: &[Value], target_type: &ValueTypeDef) -> Result<Value, ConversionError> {
        match target_type {
            ValueTypeDef::String => {
                let items: Vec<String> = value
                    .iter()
                    .map(|v| match v {
                        Value::String(s) => s.clone(),
                        _ => format!("{:?}", v),
                    })
                    .collect();
                Ok(Value::String(format!("[{}]", items.join(", "))))
            }
            ValueTypeDef::List => {
                // 如果目标类型是List，需要知道元素类型，这里保持原样
                Ok(Value::List(value.to_vec()))
            }
            _ => Err(ConversionError::UnsupportedConversion {
                from: "List".to_string(),
                to: format!("{:?}", target_type),
            }),
        }
    }

    fn convert_map(&self, value: &HashMap<String, Value>, target_type: &ValueTypeDef) -> Result<Value, ConversionError> {
        match target_type {
            ValueTypeDef::String => {
                let pairs: Vec<String> = value
                    .iter()
                    .map(|(k, v)| {
                        let serialized_v = match v {
                            Value::String(s) => s.clone(),
                            _ => format!("{:?}", v),
                        };
                        format!("\"{}\": {}", k, serialized_v)
                    })
                    .collect();
                Ok(Value::String(format!("{{{}}}", pairs.join(", "))))
            }
            ValueTypeDef::Map => {
                // 如果目标类型是Map，需要知道值类型，这里保持原样
                Ok(Value::Map(value.clone()))
            }
            _ => Err(ConversionError::UnsupportedConversion {
                from: "Map".to_string(),
                to: format!("{:?}", target_type),
            }),
        }
    }

    fn convert_set(&self, value: &HashSet<Value>, target_type: &ValueTypeDef) -> Result<Value, ConversionError> {
        match target_type {
            ValueTypeDef::String => {
                let items: Vec<String> = value
                    .iter()
                    .map(|v| match v {
                        Value::String(s) => s.clone(),
                        _ => format!("{:?}", v),
                    })
                    .collect();
                Ok(Value::String(format!("[{}]", items.join(", "))))
            }
            ValueTypeDef::Set => {
                // 如果目标类型是Set，需要知道元素类型，这里保持原样
                Ok(Value::Set(value.clone()))
            }
            _ => Err(ConversionError::UnsupportedConversion {
                from: "Set".to_string(),
                to: format!("{:?}", target_type),
            }),
        }
    }

    fn to_string(&self, value: &Value) -> Result<Value, ConversionError> {
        let string_value = match value {
            Value::Bool(v) => v.to_string(),
            Value::Int(v) => v.to_string(),
            Value::Float(v) => v.to_string(),
            Value::String(v) => v.clone(),
            Value::Date(v) => format!("{}-{}-{}", v.year, v.month, v.day),
            Value::Time(v) => format!("{}:{}:{}", v.hour, v.minute, v.sec),
            Value::DateTime(v) => format!(
                "{}-{}-{} {}:{}:{}",
                v.year, v.month, v.day, v.hour, v.minute, v.sec
            ),
            Value::Vertex(v) => format!("Vertex({:?})", v.id()),
            Value::Edge(v) => format!(
                "Edge({:?} -> {:?}, type: {})",
                &*v.src, &*v.dst, v.edge_type
            ),
            Value::Path(v) => format!("Path(length: {})", v.len()),
            Value::List(_) => "[List]".to_string(),
            Value::Map(_) => "[Map]".to_string(),
            Value::Set(_) => "[Set]".to_string(),
            Value::Geography(_) => "[Geography]".to_string(),
            Value::Duration(v) => format!("{} seconds", v.seconds),
            Value::DataSet(v) => format!(
                "Dataset({} rows, {} columns)",
                v.rows.len(),
                v.col_names.len()
            ),
            Value::Null(_) => "null".to_string(),
            Value::Empty => "empty".to_string(),
        };
        Ok(Value::String(string_value))
    }

    fn get_type_name(&self, value: &Value) -> String {
        match value {
            Value::Bool(_) => "Bool".to_string(),
            Value::Int(_) => "Int".to_string(),
            Value::Float(_) => "Float".to_string(),
            Value::String(_) => "String".to_string(),
            Value::Date(_) => "Date".to_string(),
            Value::Time(_) => "Time".to_string(),
            Value::DateTime(_) => "DateTime".to_string(),
            Value::Vertex(_) => "Vertex".to_string(),
            Value::Edge(_) => "Edge".to_string(),
            Value::Path(_) => "Path".to_string(),
            Value::List(_) => "List".to_string(),
            Value::Map(_) => "Map".to_string(),
            Value::Set(_) => "Set".to_string(),
            Value::Geography(_) => "Geography".to_string(),
            Value::Duration(_) => "Duration".to_string(),
            Value::DataSet(_) => "DataSet".to_string(),
            Value::Null(_) => "Null".to_string(),
            Value::Empty => "Empty".to_string(),
        }
    }
}

/// 便捷函数：转换Value到指定类型
pub fn convert(value: &Value, target_type: ValueTypeDef) -> Result<Value, ConversionError> {
    ValueConverter::new().convert(value, target_type)
}

/// 便捷函数：使用配置转换Value到指定类型
pub fn convert_with_config(
    value: &Value,
    target_type: ValueTypeDef,
    config: ConversionConfig,
) -> Result<Value, ConversionError> {
    ValueConverter::with_config(config).convert(value, target_type)
}

/// 便捷函数：尝试转换，失败时返回原值
pub fn try_convert(value: &Value, target_type: ValueTypeDef) -> Value {
    convert(value, target_type).unwrap_or_else(|_| value.clone())
}

/// 便捷函数：批量转换
pub fn convert_batch(
    values: &[Value],
    target_type: ValueTypeDef,
) -> Result<Vec<Value>, ConversionError> {
    let converter = ValueConverter::new();
    values
        .iter()
        .map(|v| converter.convert(v, target_type.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::value::Value;

    #[test]
    fn test_basic_conversion() {
        let string_value = Value::String("123".to_string());
        let int_value = convert(&string_value, ValueTypeDef::Int).unwrap();
        assert_eq!(int_value, Value::Int(123));

        let bool_value = convert(&string_value, ValueTypeDef::Bool).unwrap();
        assert_eq!(bool_value, Value::Bool(true));
    }

    #[test]
    fn test_float_precision() {
        let float_value = Value::Float(3.14159265359);
        let config = ConversionConfig::new().with_float_precision(2);
        let result = convert_with_config(&float_value, ValueTypeDef::Float, config).unwrap();
        
        if let Value::Float(v) = result {
            assert_eq!(v, 3.14);
        } else {
            panic!("Expected Float value");
        }
    }

    #[test]
    fn test_lenient_conversion() {
        let config = ConversionConfig::new().with_lenient(true);
        
        // 测试宽松的字符串到布尔值转换
        let string_value = Value::String("non_empty".to_string());
        let bool_value = convert_with_config(&string_value, ValueTypeDef::Bool, config).unwrap();
        assert_eq!(bool_value, Value::Bool(true));
    }

    #[test]
    fn test_batch_conversion() {
        let values = vec![
            Value::String("1".to_string()),
            Value::String("2".to_string()),
            Value::String("3".to_string()),
        ];
        
        let results = convert_batch(&values, ValueTypeDef::Int).unwrap();
        assert_eq!(results, vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
        ]);
    }

    #[test]
    fn test_try_convert() {
        let string_value = Value::String("not_a_number".to_string());
        let result = try_convert(&string_value, ValueTypeDef::Int);
        assert_eq!(result, string_value); // 应该返回原值
    }
}