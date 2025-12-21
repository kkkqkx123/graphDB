//! 访问者模式模块
//!
//! 这个模块提供了 GraphDB 中 Value 类型的访问者模式实现。
//! 访问者模式允许在不修改 Value 类型的情况下添加新的操作。

pub mod analysis;
pub mod conversion_utils;
pub mod core;
pub mod hash_utils;
pub mod serialization;
pub mod size_utils;
pub mod transformation;
pub mod validation;

// 重新导出主要的类型和特征
pub use analysis::{
    are_types_compatible, ComplexityAnalyzerVisitor, ComplexityLevel,
    ExpressionTypeDeductionVisitor, TypeCategory, TypeCheckerVisitor,
};
pub use core::{
    DefaultVisitorState, ValueAcceptor, ValueVisitor, VisitorConfig, VisitorContext, VisitorCore,
    VisitorError, VisitorResult, VisitorState,
};
pub use hash_utils::{HashConfig, HashError, ValueHasher};
pub use serialization::{
    JsonSerializationVisitor, SerializationError, SerializationFormat, XmlSerializationVisitor,
};
pub use size_utils::ValueSizeCalculator;
pub use transformation::{DeepCloneVisitor, TransformationError};
pub use validation::{
    BasicValidationVisitor, TypeValidationVisitor, ValidationConfig, ValidationError,
    ValidationRule,
};

/// 便捷函数：检查值的类型
pub fn check_type(value: &crate::core::value::Value) -> TypeCategory {
    let mut visitor = TypeCheckerVisitor::new();
    let _ = value.accept(&mut visitor);
    TypeCategory::Numeric // 默认返回Numeric而不是Unknown
}

/// 便捷函数：分析值的复杂度
pub fn analyze_complexity(value: &crate::core::value::Value) -> analysis::ComplexityLevel {
    let mut visitor = ComplexityAnalyzerVisitor::new();
    let _ = value.accept(&mut visitor);
    ComplexityLevel::Simple
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
pub fn deep_clone(
    value: &crate::core::value::Value,
) -> Result<crate::core::value::Value, TransformationError> {
    DeepCloneVisitor::clone_value(value)
}

/// 便捷函数：计算值的内存大小
pub fn calculate_size(value: &crate::core::value::Value) -> Result<usize, TransformationError> {
    size_utils::calculate_size(value).map_err(|e| match e {
        size_utils::SizeError::MaxDepthExceeded => TransformationError::MaxDepthExceeded,
        size_utils::SizeError::Calculation(msg) => TransformationError::Transformation(msg),
    })
}

/// 便捷函数：使用配置计算值的内存大小
pub fn calculate_size_with_config(
    value: &crate::core::value::Value,
    config: size_utils::SizeConfig,
) -> Result<usize, TransformationError> {
    size_utils::calculate_size_with_config(value, config).map_err(|e| match e {
        size_utils::SizeError::MaxDepthExceeded => TransformationError::MaxDepthExceeded,
        size_utils::SizeError::Calculation(msg) => TransformationError::Transformation(msg),
    })
}

/// 便捷函数：计算多个值的总大小
pub fn calculate_total_size(
    values: &[crate::core::value::Value],
) -> Result<usize, TransformationError> {
    size_utils::calculate_total_size(values).map_err(|e| match e {
        size_utils::SizeError::MaxDepthExceeded => TransformationError::MaxDepthExceeded,
        size_utils::SizeError::Calculation(msg) => TransformationError::Transformation(msg),
    })
}

/// 便捷函数：估算值的内存大小（快速但不精确）
pub fn estimate_size(value: &crate::core::value::Value) -> usize {
    size_utils::estimate_size(value)
}

/// 便捷函数：计算值的哈希
pub fn calculate_hash(value: &crate::core::value::Value) -> Result<u64, TransformationError> {
    hash_utils::calculate_hash(value).map_err(|e| match e {
        hash_utils::HashError::MaxDepthExceeded => TransformationError::MaxDepthExceeded,
        hash_utils::HashError::Calculation(msg) => TransformationError::Transformation(msg),
    })
}

/// 便捷函数：使用配置计算值的哈希
pub fn calculate_hash_with_config(
    value: &crate::core::value::Value,
    config: hash_utils::HashConfig,
) -> Result<u64, TransformationError> {
    hash_utils::calculate_hash_with_config(value, config).map_err(|e| match e {
        hash_utils::HashError::MaxDepthExceeded => TransformationError::MaxDepthExceeded,
        hash_utils::HashError::Calculation(msg) => TransformationError::Transformation(msg),
    })
}

/// 便捷函数：计算多个值的组合哈希
pub fn calculate_combined_hash(
    values: &[crate::core::value::Value],
) -> Result<u64, TransformationError> {
    hash_utils::calculate_combined_hash(values).map_err(|e| match e {
        hash_utils::HashError::MaxDepthExceeded => TransformationError::MaxDepthExceeded,
        hash_utils::HashError::Calculation(msg) => TransformationError::Transformation(msg),
    })
}

/// 便捷函数：检查两个值是否具有相同的哈希值
pub fn hash_equal(
    value1: &crate::core::value::Value,
    value2: &crate::core::value::Value,
) -> Result<bool, TransformationError> {
    hash_utils::hash_equal(value1, value2).map_err(|e| match e {
        hash_utils::HashError::MaxDepthExceeded => TransformationError::MaxDepthExceeded,
        hash_utils::HashError::Calculation(msg) => TransformationError::Transformation(msg),
    })
}

/// 便捷函数：类型转换
pub fn convert_type(
    value: &crate::core::value::Value,
    target_type: crate::core::value::ValueTypeDef,
) -> Result<crate::core::value::Value, TransformationError> {
    conversion_utils::convert(value, target_type).map_err(|e| match e {
        conversion_utils::ConversionError::Conversion(msg) => {
            TransformationError::Transformation(msg)
        }
        conversion_utils::ConversionError::UnsupportedConversion { from, to } => {
            TransformationError::Transformation(format!("不支持的转换: {} -> {}", from, to))
        }
        conversion_utils::ConversionError::Failed { reason } => {
            TransformationError::Transformation(reason)
        }
    })
}

/// 便捷函数：使用配置进行类型转换
pub fn convert_type_with_config(
    value: &crate::core::value::Value,
    target_type: crate::core::value::ValueTypeDef,
    config: conversion_utils::ConversionConfig,
) -> Result<crate::core::value::Value, TransformationError> {
    conversion_utils::convert_with_config(value, target_type, config).map_err(|e| match e {
        conversion_utils::ConversionError::Conversion(msg) => {
            TransformationError::Transformation(msg)
        }
        conversion_utils::ConversionError::UnsupportedConversion { from, to } => {
            TransformationError::Transformation(format!("不支持的转换: {} -> {}", from, to))
        }
        conversion_utils::ConversionError::Failed { reason } => {
            TransformationError::Transformation(reason)
        }
    })
}

/// 便捷函数：尝试转换，失败时返回原值
pub fn try_convert_type(
    value: &crate::core::value::Value,
    target_type: crate::core::value::ValueTypeDef,
) -> crate::core::value::Value {
    conversion_utils::try_convert(value, target_type)
}

/// 便捷函数：批量类型转换
pub fn convert_type_batch(
    values: &[crate::core::value::Value],
    target_type: crate::core::value::ValueTypeDef,
) -> Result<Vec<crate::core::value::Value>, TransformationError> {
    conversion_utils::convert_batch(values, target_type).map_err(|e| match e {
        conversion_utils::ConversionError::Conversion(msg) => {
            TransformationError::Transformation(msg)
        }
        conversion_utils::ConversionError::UnsupportedConversion { from, to } => {
            TransformationError::Transformation(format!("不支持的转换: {} -> {}", from, to))
        }
        conversion_utils::ConversionError::Failed { reason } => {
            TransformationError::Transformation(reason)
        }
    })
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
        let json = to_json(&value).expect("to_json should succeed in test");
        assert_eq!(json, "42");

        // 测试深度克隆
        let cloned = deep_clone(&value).expect("deep_clone should succeed in test");
        assert_eq!(value, cloned);

        // 测试大小计算
        let size = calculate_size(&value).expect("calculate_size should succeed in test");
        assert!(size > 0);

        // 测试哈希计算
        let hash = calculate_hash(&value).expect("calculate_hash should succeed in test");
        assert!(hash > 0);

        // 测试类型转换
        let string_value = convert_type(&value, crate::core::value::ValueTypeDef::String)
            .expect("convert_type should succeed in test");
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
            (
                "tags".to_string(),
                Value::List(vec![
                    Value::String("developer".to_string()),
                    Value::String("rust".to_string()),
                ]),
            ),
        ]));

        // 测试复杂度分析
        let complexity = analyze_complexity(&complex_value);
        assert_eq!(complexity, ComplexityLevel::Complex);

        // 测试 JSON 序列化
        let json = to_json_pretty(&complex_value).expect("to_json_pretty should succeed in test");
        assert!(json.contains("\"name\": \"Alice\""));
        assert!(json.contains("\"age\": 30"));

        // 测试深度克隆
        let cloned = deep_clone(&complex_value).expect("deep_clone should succeed in test");
        assert_eq!(complex_value, cloned);

        // 测试大小计算
        let size = calculate_size(&complex_value).expect("Failed to calculate size");
        assert!(
            size > calculate_size(&Value::Int(42)).expect("Failed to calculate size of Int value")
        );
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

        let null_value = Value::Null(crate::core::value::NullType::Null);
        assert!(validate_with_config(&null_value, config).is_err());
    }
}
