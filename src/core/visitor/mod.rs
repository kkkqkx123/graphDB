//! 访问者模式模块
//!
//! 这个模块提供了 GraphDB 中 Value 类型的访问者模式实现。
//! 访问者模式允许在不修改 Value 类型的情况下添加新的操作。

pub mod core;
pub mod analysis;
pub mod serialization;
pub mod transformation;
pub mod validation;

// 重新导出主要的类型和特征
pub use core::{ValueVisitor, ValueAcceptor};
pub use analysis::{TypeCheckerVisitor, ComplexityAnalyzerVisitor, TypeCategory, ComplexityLevel};
pub use serialization::{JsonSerializationVisitor, XmlSerializationVisitor, SerializationFormat, SerializationError};
pub use transformation::{DeepCloneVisitor, SizeCalculatorVisitor, HashCalculatorVisitor, TypeConversionVisitor, TransformationError};
pub use validation::{BasicValidationVisitor, TypeValidationVisitor, ValidationConfig, ValidationRule, ValidationError};

/// 便捷函数：检查值的类型
pub fn check_type(value: &crate::core::value::Value) -> TypeCategory {
    let mut visitor = TypeCheckerVisitor::new();
    value.accept(&mut visitor).unwrap_or(TypeCategory::Unknown)
}

/// 便捷函数：分析值的复杂度
pub fn analyze_complexity(value: &crate::core::value::Value) -> analysis::ComplexityLevel {
    let mut visitor = ComplexityAnalyzerVisitor::new();
    value.accept(&mut visitor).unwrap_or(ComplexityLevel::Simple)
}

/// 便捷函数：序列化为 JSON
pub fn to_json(value: &crate::core::value::Value) -> Result<String, SerializationError> {
    JsonSerializationVisitor::serialize(value)
}

/// 便捷函数：序列化为格式化的 JSON
pub fn to_json_pretty(value: &crate::core::value::Value) -> Result<String, SerializationError> {
    JsonSerializationVisitor::serialize_pretty(value)
}

/// 便捷函数：序列化为 XML
pub fn to_xml(value: &crate::core::value::Value) -> Result<String, SerializationError> {
    XmlSerializationVisitor::serialize(value)
}

/// 便捷函数：深度克隆值
pub fn deep_clone(value: &crate::core::value::Value) -> Result<crate::core::value::Value, TransformationError> {
    DeepCloneVisitor::clone_value(value)
}

/// 便捷函数：计算值的内存大小
pub fn calculate_size(value: &crate::core::value::Value) -> Result<usize, TransformationError> {
    SizeCalculatorVisitor::calculate_size(value)
}

/// 便捷函数：计算值的哈希
pub fn calculate_hash(value: &crate::core::value::Value) -> Result<u64, TransformationError> {
    HashCalculatorVisitor::calculate_hash(value)
}

/// 便捷函数：类型转换
pub fn convert_type(
    value: &crate::core::value::Value,
    target_type: crate::core::value::ValueTypeDef,
) -> Result<crate::core::value::Value, TransformationError> {
    TypeConversionVisitor::convert(value, target_type)
}

/// 便捷函数：基础验证
pub fn validate_basic(value: &crate::core::value::Value) -> Result<(), ValidationError> {
    BasicValidationVisitor::validate(value)
}

/// 便捷函数：带配置的验证
pub fn validate_with_config(
    value: &crate::core::value::Value,
    config: ValidationConfig,
) -> Result<(), ValidationError> {
    BasicValidationVisitor::validate_with_config(value, config)
}

/// 便捷函数：类型验证
pub fn validate_type(
    value: &crate::core::value::Value,
    expected_type: crate::core::value::ValueTypeDef,
) -> Result<(), ValidationError> {
    TypeValidationVisitor::validate_type(value, expected_type)
}

/// 便捷函数：严格类型验证
pub fn validate_type_strict(
    value: &crate::core::value::Value,
    expected_type: crate::core::value::ValueTypeDef,
) -> Result<(), ValidationError> {
    TypeValidationVisitor::validate_type_strict(value, expected_type)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::value::Value;
    use std::collections::HashMap;

    #[test]
    fn test_convenience_functions() {
        let value = Value::Int(42);
        
        // 测试类型检查
        let category = check_type(&value);
        assert_eq!(category, TypeCategory::Numeric);
        
        // 测试复杂度分析
        let complexity = analyze_complexity(&value);
        assert_eq!(complexity, ComplexityLevel::Simple);
        
        // 测试 JSON 序列化
        let json = to_json(&value).unwrap();
        assert_eq!(json, "42");
        
        // 测试深度克隆
        let cloned = deep_clone(&value).unwrap();
        assert_eq!(value, cloned);
        
        // 测试大小计算
        let size = calculate_size(&value).unwrap();
        assert!(size > 0);
        
        // 测试哈希计算
        let hash = calculate_hash(&value).unwrap();
        assert!(hash > 0);
        
        // 测试类型转换
        let string_value = convert_type(&value, crate::core::value::ValueTypeDef::String).unwrap();
        assert_eq!(string_value, Value::String("42".to_string()));
        
        // 测试基础验证
        assert!(validate_basic(&value).is_ok());
        
        // 测试类型验证
        assert!(validate_type(&value, crate::core::value::ValueTypeDef::Int).is_ok());
        assert!(validate_type(&value, crate::core::value::ValueTypeDef::String).is_err());
    }

    #[test]
    fn test_complex_value_operations() {
        let complex_value = Value::Map(HashMap::from([
            ("name".to_string(), Value::String("Alice".to_string())),
            ("age".to_string(), Value::Int(30)),
            ("tags".to_string(), Value::List(vec![
                Value::String("developer".to_string()),
                Value::String("rust".to_string()),
            ])),
        ]));
        
        // 测试复杂度分析
        let complexity = analyze_complexity(&complex_value);
        assert_eq!(complexity, ComplexityLevel::Complex);
        
        // 测试 JSON 序列化
        let json = to_json_pretty(&complex_value).unwrap();
        assert!(json.contains("\"name\": \"Alice\""));
        assert!(json.contains("\"age\": 30"));
        
        // 测试深度克隆
        let cloned = deep_clone(&complex_value).unwrap();
        assert_eq!(complex_value, cloned);
        
        // 测试大小计算
        let size = calculate_size(&complex_value).unwrap();
        assert!(size > calculate_size(&Value::Int(42)).unwrap());
    }

    #[test]
    fn test_validation_with_config() {
        let config = ValidationConfig {
            max_string_length: 5,
            allow_null_values: false,
            ..Default::default()
        };
        
        let short_string = Value::String("test".to_string());
        assert!(validate_with_config(&short_string, config.clone()).is_ok());
        
        let long_string = Value::String("this is too long".to_string());
        assert!(validate_with_config(&long_string, config.clone()).is_err());
        
        let null_value = Value::Null(crate::core::value::NullType::__NULL__);
        assert!(validate_with_config(&null_value, config).is_err());
    }
}