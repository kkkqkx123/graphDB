//! 验证类访问者
//!
//! 这个模块提供了用于验证 Value 的访问者实现

use crate::core::visitor::core::{ValueVisitor, ValueAcceptor, utils};
use crate::core::value::{Value, NullType, DateValue, TimeValue, DateTimeValue, GeographyValue, DurationValue, DataSet};
use crate::core::vertex_edge_path::{Vertex, Edge, Path};
use std::collections::HashMap;

/// 验证错误类型
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("验证错误: {0}")]
    Validation(String),
    #[error("类型不匹配: 期望 {expected}, 实际 {actual}")]
    TypeMismatch { expected: String, actual: String },
    #[error("值超出范围: {value} 不在 [{min}, {max}] 范围内")]
    OutOfRange { value: String, min: String, max: String },
    #[error("字符串长度超出限制: {length} > {max_length}")]
    StringTooLong { length: usize, max_length: usize },
    #[error("集合大小超出限制: {size} > {max_size}")]
    CollectionTooLarge { size: usize, max_size: usize },
    #[error("递归深度超过限制: {depth} > {max_depth}")]
    MaxDepthExceeded { depth: usize, max_depth: usize },
    #[error("无效的日期: {year}-{month}-{day}")]
    InvalidDate { year: i32, month: u32, day: u32 },
    #[error("无效的时间: {hour}:{minute}:{second}")]
    InvalidTime { hour: u32, minute: u32, second: u32 },
    #[error("顶点 ID 无效: {id}")]
    InvalidVertexId { id: String },
    #[error("边类型无效: {edge_type}")]
    InvalidEdgeType { edge_type: String },
    #[error("路径结构无效")]
    InvalidPathStructure,
}

/// 验证规则
#[derive(Debug, Clone)]
pub struct ValidationRule {
    pub name: String,
    pub description: String,
    pub validator: Box<dyn Fn(&Value) -> Result<(), ValidationError>>,
}

impl ValidationRule {
    pub fn new<F>(name: &str, description: &str, validator: F) -> Self
    where
        F: Fn(&Value) -> Result<(), ValidationError> + 'static,
    {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            validator: Box::new(validator),
        }
    }
}

/// 验证配置
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    pub max_depth: usize,
    pub max_string_length: usize,
    pub max_collection_size: usize,
    pub strict_type_checking: bool,
    pub allow_null_values: bool,
    pub custom_rules: Vec<ValidationRule>,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            max_depth: 100,
            max_string_length: 10000,
            max_collection_size: 10000,
            strict_type_checking: false,
            allow_null_values: true,
            custom_rules: Vec::new(),
        }
    }
}

/// 基础验证访问者 - 验证 Value 的基本结构和类型
#[derive(Debug)]
pub struct BasicValidationVisitor {
    config: ValidationConfig,
    errors: Vec<ValidationError>,
    current_depth: usize,
}

impl BasicValidationVisitor {
    pub fn new(config: ValidationConfig) -> Self {
        Self {
            config,
            errors: Vec::new(),
            current_depth: 0,
        }
    }

    pub fn validate(value: &Value) -> Result<(), ValidationError> {
        let config = ValidationConfig::default();
        let mut visitor = Self::new(config);
        utils::visit_recursive(value, &mut visitor, 0, visitor.config.max_depth)?;
        
        if visitor.errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationError::Validation(format!(
                "发现 {} 个验证错误: {:?}",
                visitor.errors.len(),
                visitor.errors
            )))
        }
    }

    pub fn validate_with_config(value: &Value, config: ValidationConfig) -> Result<(), ValidationError> {
        let mut visitor = Self::new(config);
        utils::visit_recursive(value, &mut visitor, 0, visitor.config.max_depth)?;
        
        if visitor.errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationError::Validation(format!(
                "发现 {} 个验证错误: {:?}",
                visitor.errors.len(),
                visitor.errors
            )))
        }
    }

    pub fn errors(&self) -> &[ValidationError] {
        &self.errors
    }

    fn add_error(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    fn check_depth(&mut self) -> Result<(), ValidationError> {
        if self.current_depth > self.config.max_depth {
            self.add_error(ValidationError::MaxDepthExceeded {
                depth: self.current_depth,
                max_depth: self.config.max_depth,
            });
            return Err(ValidationError::MaxDepthExceeded {
                depth: self.current_depth,
                max_depth: self.config.max_depth,
            });
        }
        Ok(())
    }
}

impl ValueVisitor for BasicValidationVisitor {
    type Result = Result<(), ValidationError>;

    fn visit_bool(&mut self, _value: bool) -> Self::Result {
        self.check_depth()
    }

    fn visit_int(&mut self, _value: i64) -> Self::Result {
        self.check_depth()
    }

    fn visit_float(&mut self, value: f64) -> Self::Result {
        self.check_depth()?;
        
        // 检查 NaN 和无穷大
        if value.is_nan() {
            self.add_error(ValidationError::Validation(
                "浮点数不能为 NaN".to_string()
            ));
        }
        if value.is_infinite() {
            self.add_error(ValidationError::Validation(
                "浮点数不能为无穷大".to_string()
            ));
        }
        
        Ok(())
    }

    fn visit_string(&mut self, value: &str) -> Self::Result {
        self.check_depth()?;
        
        if value.len() > self.config.max_string_length {
            self.add_error(ValidationError::StringTooLong {
                length: value.len(),
                max_length: self.config.max_string_length,
            });
        }
        
        Ok(())
    }

    fn visit_date(&mut self, value: &DateValue) -> Self::Result {
        self.check_depth()?;
        
        // 验证日期的有效性
        if value.month < 1 || value.month > 12 {
            self.add_error(ValidationError::InvalidDate {
                year: value.year,
                month: value.month,
                day: value.day,
            });
        }
        
        if value.day < 1 || value.day > 31 {
            self.add_error(ValidationError::InvalidDate {
                year: value.year,
                month: value.month,
                day: value.day,
            });
        }
        
        // 简单的月份天数验证
        let max_day = match value.month {
            2 => {
                // 简单的闰年判断
                if (value.year % 4 == 0 && value.year % 100 != 0) || (value.year % 400 == 0) {
                    29
                } else {
                    28
                }
            }
            4 | 6 | 9 | 11 => 30,
            _ => 31,
        };
        
        if value.day > max_day {
            self.add_error(ValidationError::InvalidDate {
                year: value.year,
                month: value.month,
                day: value.day,
            });
        }
        
        Ok(())
    }

    fn visit_time(&mut self, value: &TimeValue) -> Self::Result {
        self.check_depth()?;
        
        if value.hour >= 24 {
            self.add_error(ValidationError::InvalidTime {
                hour: value.hour,
                minute: value.minute,
                second: value.sec,
            });
        }
        
        if value.minute >= 60 {
            self.add_error(ValidationError::InvalidTime {
                hour: value.hour,
                minute: value.minute,
                second: value.sec,
            });
        }
        
        if value.sec >= 60 {
            self.add_error(ValidationError::InvalidTime {
                hour: value.hour,
                minute: value.minute,
                second: value.sec,
            });
        }
        
        Ok(())
    }

    fn visit_datetime(&mut self, value: &DateTimeValue) -> Self::Result {
        self.check_depth()?;
        
        // 验证日期部分
        self.visit_date(&DateValue {
            year: value.year,
            month: value.month,
            day: value.day,
        })?;
        
        // 验证时间部分
        self.visit_time(&TimeValue {
            hour: value.hour,
            minute: value.minute,
            sec: value.sec,
        })?;
        
        Ok(())
    }

    fn visit_vertex(&mut self, value: &Vertex) -> Self::Result {
        self.check_depth()?;
        
        // 验证顶点 ID
        match value.id() {
            Value::Int(_) | Value::String(_) => {
                // 有效的 ID 类型
            }
            _ => {
                self.add_error(ValidationError::InvalidVertexId {
                    id: format!("{:?}", value.id()),
                });
            }
        }
        
        // 验证标签
        if value.tags().is_empty() {
            self.add_error(ValidationError::Validation(
                "顶点必须至少有一个标签".to_string()
            ));
        }
        
        Ok(())
    }

    fn visit_edge(&mut self, value: &Edge) -> Self::Result {
        self.check_depth()?;
        
        // 验证边类型
        if value.edge_type().is_empty() {
            self.add_error(ValidationError::InvalidEdgeType {
                edge_type: value.edge_type().to_string(),
            });
        }
        
        // 验证排名
        if value.ranking() < 0 {
            self.add_error(ValidationError::Validation(
                "边排名不能为负数".to_string()
            ));
        }
        
        Ok(())
    }

    fn visit_path(&mut self, value: &Path) -> Self::Result {
        self.check_depth()?;
        
        // 验证路径结构
        if value.steps.is_empty() {
            self.add_error(ValidationError::InvalidPathStructure);
        }
        
        Ok(())
    }

    fn visit_list(&mut self, value: &[Value]) -> Self::Result {
        self.check_depth()?;
        
        if value.len() > self.config.max_collection_size {
            self.add_error(ValidationError::CollectionTooLarge {
                size: value.len(),
                max_size: self.config.max_collection_size,
            });
        }
        
        Ok(())
    }

    fn visit_map(&mut self, value: &HashMap<String, Value>) -> Self::Result {
        self.check_depth()?;
        
        if value.len() > self.config.max_collection_size {
            self.add_error(ValidationError::CollectionTooLarge {
                size: value.len(),
                max_size: self.config.max_collection_size,
            });
        }
        
        Ok(())
    }

    fn visit_set(&mut self, value: &std::collections::HashSet<Value>) -> Self::Result {
        self.check_depth()?;
        
        if value.len() > self.config.max_collection_size {
            self.add_error(ValidationError::CollectionTooLarge {
                size: value.len(),
                max_size: self.config.max_collection_size,
            });
        }
        
        Ok(())
    }

    fn visit_geography(&mut self, _value: &GeographyValue) -> Self::Result {
        self.check_depth()
    }

    fn visit_duration(&mut self, value: &DurationValue) -> Self::Result {
        self.check_depth()?;
        
        if value.seconds < 0 {
            self.add_error(ValidationError::Validation(
                "持续时间不能为负数".to_string()
            ));
        }
        
        Ok(())
    }

    fn visit_dataset(&mut self, value: &DataSet) -> Self::Result {
        self.check_depth()?;
        
        // 验证数据集结构
        if value.col_names.is_empty() {
            self.add_error(ValidationError::Validation(
                "数据集必须至少有一列".to_string()
            ));
        }
        
        // 验证行数据一致性
        for (i, row) in value.rows.iter().enumerate() {
            if row.len() != value.col_names.len() {
                self.add_error(ValidationError::Validation(format!(
                    "第 {} 行的列数 ({}) 与列定义 ({}) 不匹配",
                    i, row.len(), value.col_names.len()
                )));
            }
        }
        
        Ok(())
    }

    fn visit_null(&mut self, _null_type: &NullType) -> Self::Result {
        self.check_depth()?;
        
        if !self.config.allow_null_values {
            self.add_error(ValidationError::Validation(
                "不允许 null 值".to_string()
            ));
        }
        
        Ok(())
    }

    fn visit_empty(&mut self) -> Self::Result {
        self.check_depth()
    }
}

/// 类型验证访问者 - 验证 Value 是否符合特定类型要求
#[derive(Debug)]
pub struct TypeValidationVisitor {
    expected_type: Option<crate::core::value::ValueTypeDef>,
    strict_mode: bool,
}

impl TypeValidationVisitor {
    pub fn new(expected_type: crate::core::value::ValueTypeDef) -> Self {
        Self {
            expected_type: Some(expected_type),
            strict_mode: false,
        }
    }

    pub fn strict(expected_type: crate::core::value::ValueTypeDef) -> Self {
        Self {
            expected_type: Some(expected_type),
            strict_mode: true,
        }
    }

    pub fn validate_type(value: &Value, expected_type: crate::core::value::ValueTypeDef) -> Result<(), ValidationError> {
        let mut visitor = Self::new(expected_type);
        value.accept(&mut visitor)
    }

    pub fn validate_type_strict(value: &Value, expected_type: crate::core::value::ValueTypeDef) -> Result<(), ValidationError> {
        let mut visitor = Self::strict(expected_type);
        value.accept(&mut visitor)
    }
}

impl ValueVisitor for TypeValidationVisitor {
    type Result = Result<(), ValidationError>;

    fn visit_bool(&mut self, _value: bool) -> Self::Result {
        match &self.expected_type {
            Some(crate::core::value::ValueTypeDef::Bool) => Ok(()),
            Some(expected) => Err(ValidationError::TypeMismatch {
                expected: format!("{:?}", expected),
                actual: "Bool".to_string(),
            }),
            None => Ok(()),
        }
    }

    fn visit_int(&mut self, _value: i64) -> Self::Result {
        match &self.expected_type {
            Some(crate::core::value::ValueTypeDef::Int) => Ok(()),
            Some(expected) => Err(ValidationError::TypeMismatch {
                expected: format!("{:?}", expected),
                actual: "Int".to_string(),
            }),
            None => Ok(()),
        }
    }

    fn visit_float(&mut self, _value: f64) -> Self::Result {
        match &self.expected_type {
            Some(crate::core::value::ValueTypeDef::Float) => Ok(()),
            Some(expected) => Err(ValidationError::TypeMismatch {
                expected: format!("{:?}", expected),
                actual: "Float".to_string(),
            }),
            None => Ok(()),
        }
    }

    fn visit_string(&mut self, _value: &str) -> Self::Result {
        match &self.expected_type {
            Some(crate::core::value::ValueTypeDef::String) => Ok(()),
            Some(expected) => Err(ValidationError::TypeMismatch {
                expected: format!("{:?}", expected),
                actual: "String".to_string(),
            }),
            None => Ok(()),
        }
    }

    fn visit_date(&mut self, _value: &DateValue) -> Self::Result {
        match &self.expected_type {
            Some(crate::core::value::ValueTypeDef::Date) => Ok(()),
            Some(expected) => Err(ValidationError::TypeMismatch {
                expected: format!("{:?}", expected),
                actual: "Date".to_string(),
            }),
            None => Ok(()),
        }
    }

    fn visit_time(&mut self, _value: &TimeValue) -> Self::Result {
        match &self.expected_type {
            Some(crate::core::value::ValueTypeDef::Time) => Ok(()),
            Some(expected) => Err(ValidationError::TypeMismatch {
                expected: format!("{:?}", expected),
                actual: "Time".to_string(),
            }),
            None => Ok(()),
        }
    }

    fn visit_datetime(&mut self, _value: &DateTimeValue) -> Self::Result {
        match &self.expected_type {
            Some(crate::core::value::ValueTypeDef::DateTime) => Ok(()),
            Some(expected) => Err(ValidationError::TypeMismatch {
                expected: format!("{:?}", expected),
                actual: "DateTime".to_string(),
            }),
            None => Ok(()),
        }
    }

    fn visit_vertex(&mut self, _value: &Vertex) -> Self::Result {
        match &self.expected_type {
            Some(crate::core::value::ValueTypeDef::Vertex) => Ok(()),
            Some(expected) => Err(ValidationError::TypeMismatch {
                expected: format!("{:?}", expected),
                actual: "Vertex".to_string(),
            }),
            None => Ok(()),
        }
    }

    fn visit_edge(&mut self, _value: &Edge) -> Self::Result {
        match &self.expected_type {
            Some(crate::core::value::ValueTypeDef::Edge) => Ok(()),
            Some(expected) => Err(ValidationError::TypeMismatch {
                expected: format!("{:?}", expected),
                actual: "Edge".to_string(),
            }),
            None => Ok(()),
        }
    }

    fn visit_path(&mut self, _value: &Path) -> Self::Result {
        match &self.expected_type {
            Some(crate::core::value::ValueTypeDef::Path) => Ok(()),
            Some(expected) => Err(ValidationError::TypeMismatch {
                expected: format!("{:?}", expected),
                actual: "Path".to_string(),
            }),
            None => Ok(()),
        }
    }

    fn visit_list(&mut self, _value: &[Value]) -> Self::Result {
        match &self.expected_type {
            Some(crate::core::value::ValueTypeDef::List) => Ok(()),
            Some(expected) => Err(ValidationError::TypeMismatch {
                expected: format!("{:?}", expected),
                actual: "List".to_string(),
            }),
            None => Ok(()),
        }
    }

    fn visit_map(&mut self, _value: &HashMap<String, Value>) -> Self::Result {
        match &self.expected_type {
            Some(crate::core::value::ValueTypeDef::Map) => Ok(()),
            Some(expected) => Err(ValidationError::TypeMismatch {
                expected: format!("{:?}", expected),
                actual: "Map".to_string(),
            }),
            None => Ok(()),
        }
    }

    fn visit_set(&mut self, _value: &std::collections::HashSet<Value>) -> Self::Result {
        match &self.expected_type {
            Some(crate::core::value::ValueTypeDef::Set) => Ok(()),
            Some(expected) => Err(ValidationError::TypeMismatch {
                expected: format!("{:?}", expected),
                actual: "Set".to_string(),
            }),
            None => Ok(()),
        }
    }

    fn visit_geography(&mut self, _value: &GeographyValue) -> Self::Result {
        match &self.expected_type {
            Some(crate::core::value::ValueTypeDef::Geography) => Ok(()),
            Some(expected) => Err(ValidationError::TypeMismatch {
                expected: format!("{:?}", expected),
                actual: "Geography".to_string(),
            }),
            None => Ok(()),
        }
    }

    fn visit_duration(&mut self, _value: &DurationValue) -> Self::Result {
        match &self.expected_type {
            Some(crate::core::value::ValueTypeDef::Duration) => Ok(()),
            Some(expected) => Err(ValidationError::TypeMismatch {
                expected: format!("{:?}", expected),
                actual: "Duration".to_string(),
            }),
            None => Ok(()),
        }
    }

    fn visit_dataset(&mut self, _value: &DataSet) -> Self::Result {
        match &self.expected_type {
            Some(crate::core::value::ValueTypeDef::DataSet) => Ok(()),
            Some(expected) => Err(ValidationError::TypeMismatch {
                expected: format!("{:?}", expected),
                actual: "DataSet".to_string(),
            }),
            None => Ok(()),
        }
    }

    fn visit_null(&mut self, _null_type: &NullType) -> Self::Result {
        match &self.expected_type {
            Some(crate::core::value::ValueTypeDef::Null) => Ok(()),
            Some(expected) => Err(ValidationError::TypeMismatch {
                expected: format!("{:?}", expected),
                actual: "Null".to_string(),
            }),
            None => Ok(()),
        }
    }

    fn visit_empty(&mut self) -> Self::Result {
        match &self.expected_type {
            Some(crate::core::value::ValueTypeDef::Empty) => Ok(()),
            Some(expected) => Err(ValidationError::TypeMismatch {
                expected: format!("{:?}", expected),
                actual: "Empty".to_string(),
            }),
            None => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::value::Value;

    #[test]
    fn test_basic_validation_visitor() {
        let valid_value = Value::Int(42);
        assert!(BasicValidationVisitor::validate(&valid_value).is_ok());
        
        let invalid_float = Value::Float(f64::NAN);
        assert!(BasicValidationVisitor::validate(&invalid_float).is_err());
    }

    #[test]
    fn test_validation_config() {
        let config = ValidationConfig {
            max_string_length: 5,
            ..Default::default()
        };
        
        let long_string = Value::String("this is a very long string".to_string());
        assert!(BasicValidationVisitor::validate_with_config(&long_string, config).is_err());
    }

    #[test]
    fn test_type_validation_visitor() {
        let int_value = Value::Int(42);
        assert!(TypeValidationVisitor::validate_type(&int_value, crate::core::value::ValueTypeDef::Int).is_ok());
        assert!(TypeValidationVisitor::validate_type(&int_value, crate::core::value::ValueTypeDef::String).is_err());
    }

    #[test]
    fn test_date_validation() {
        let valid_date = Value::Date(DateValue {
            year: 2023,
            month: 12,
            day: 25,
        });
        assert!(BasicValidationVisitor::validate(&valid_date).is_ok());
        
        let invalid_date = Value::Date(DateValue {
            year: 2023,
            month: 2,
            day: 30, // 2月没有30号
        });
        assert!(BasicValidationVisitor::validate(&invalid_date).is_err());
    }

    #[test]
    fn test_time_validation() {
        let valid_time = Value::Time(TimeValue {
            hour: 14,
            minute: 30,
            sec: 45,
        });
        assert!(BasicValidationVisitor::validate(&valid_time).is_ok());
        
        let invalid_time = Value::Time(TimeValue {
            hour: 25, // 无效小时
            minute: 30,
            sec: 45,
        });
        assert!(BasicValidationVisitor::validate(&invalid_time).is_err());
    }
}