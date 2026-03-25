//! Type system tool module
//!
//! Provide core functions such as type compatibility checking, type precedence, and type conversion.

use crate::core::value::dataset::List;
use crate::core::DataType;
use crate::core::Value;

/// Type system tools
pub struct TypeUtils;

impl TypeUtils {
    /// Check whether the two types are compatible.
    pub fn are_types_compatible(type1: &DataType, type2: &DataType) -> bool {
        if type1 == type2 {
            return true;
        }

        if Self::is_superior_type(type1) || Self::is_superior_type(type2) {
            return true;
        }

        if (type1 == &DataType::Int && type2 == &DataType::Float)
            || (type1 == &DataType::Float && type2 == &DataType::Int)
        {
            return true;
        }

        false
    }

    /// Check whether the type is a "superior type" (which can be compatible with any other type).
    pub fn is_superior_type(type_: &DataType) -> bool {
        matches!(type_, DataType::Null | DataType::Empty)
    }

    /// Priority of the obtained type (used for type promotion)
    /// The smaller the priority value, the more “basic” the type is. When a type is upgraded, its priority value increases.
    pub fn get_type_priority(type_: &DataType) -> u8 {
        match type_ {
            DataType::Null | DataType::Empty => 0,
            DataType::Bool => 10,
            DataType::Int => 20,
            DataType::Int8 => 21,
            DataType::Int16 => 22,
            DataType::Int32 => 23,
            DataType::Int64 => 24,
            DataType::UInt8 => 25,
            DataType::UInt16 => 26,
            DataType::UInt32 => 27,
            DataType::UInt64 => 28,
            DataType::Float => 30,
            DataType::Double => 31,
            DataType::Decimal128 => 32,
            DataType::String => 40,
            DataType::FixedString(_) => 41,
            DataType::Date => 50,
            DataType::Time => 60,
            DataType::Timestamp => 61,
            DataType::DateTime => 62,
            DataType::VID => 70,
            DataType::Vertex => 80,
            DataType::Edge => 90,
            DataType::Path => 100,
            DataType::List => 110,
            DataType::Set => 120,
            DataType::Map => 130,
            DataType::Blob => 140,
            DataType::Geography => 150,
            DataType::Duration => 160,
            DataType::DataSet => 170,
        }
    }

    /// Obtaining two types of common supertypes
    pub fn get_common_type(type1: &DataType, type2: &DataType) -> DataType {
        if type1 == type2 {
            return type1.clone();
        }

        if Self::is_superior_type(type1) {
            return type2.clone();
        }
        if Self::is_superior_type(type2) {
            return type1.clone();
        }

        if (type1 == &DataType::Int && type2 == &DataType::Float)
            || (type1 == &DataType::Float && type2 == &DataType::Int)
        {
            return DataType::Float;
        }

        DataType::Empty
    }

    /// Unified type compatibility checks (without the need for caching)
    pub fn check_compatibility(type1: &DataType, type2: &DataType) -> bool {
        Self::are_types_compatible(type1, type2)
    }

    /// Batch type checking (for optimizing memory allocation)
    pub fn check_compatibility_batch(pairs: &[(DataType, DataType)]) -> Vec<bool> {
        let mut results = Vec::with_capacity(pairs.len());

        for (t1, t2) in pairs {
            results.push(Self::check_compatibility(t1, t2));
        }
        results
    }

    /// Obtaining the literal value type
    pub fn literal_type(value: &crate::core::value::Value) -> DataType {
        value.get_type()
    }

    /// Type of the result of a binary operation
    pub fn binary_operation_result_type(
        op: &str,
        left_type: &DataType,
        right_type: &DataType,
    ) -> DataType {
        match op {
            "+" | "-" | "*" | "/" => {
                if left_type == &DataType::Float || right_type == &DataType::Float {
                    DataType::Float
                } else {
                    DataType::Int
                }
            }
            "==" | "!=" | "<" | "<=" | ">" | ">=" => DataType::Bool,
            _ => DataType::Empty,
        }
    }

    /// Determine whether caching is required (based on complexity heuristics)
    pub fn should_cache_expression(expr_depth: usize, expr_node_count: usize) -> bool {
        expr_depth > 3 || expr_node_count > 10
    }

    /// Check whether the type of the source data can be converted into the target type.
    ///
    /// Use the `match` expression to implement type conversion rules that are determined at compile time.
    /// Avoid runtime initialization and global state.
    pub fn can_cast(from: &DataType, to: &DataType) -> bool {
        if from == to {
            return true;
        }

        match (from, to) {
            // The value `Int` can be converted to `Int`, `Float`, or `String`.
            (DataType::Int, DataType::Float) => true,
            (DataType::Int, DataType::String) => true,

            // Values of type Int8, Int16, Int32, and Int64 can be converted to types Int, Float, and String.
            (DataType::Int8, DataType::Int) => true,
            (DataType::Int8, DataType::Float) => true,
            (DataType::Int8, DataType::String) => true,
            (DataType::Int16, DataType::Int) => true,
            (DataType::Int16, DataType::Float) => true,
            (DataType::Int16, DataType::String) => true,
            (DataType::Int32, DataType::Int) => true,
            (DataType::Int32, DataType::Float) => true,
            (DataType::Int32, DataType::String) => true,
            (DataType::Int64, DataType::Int) => true,
            (DataType::Int64, DataType::Float) => true,
            (DataType::Int64, DataType::String) => true,

            // Values of type UInt8, UInt16, UInt32, or UInt64 can be converted to types Int, Float, or String.
            (DataType::UInt8, DataType::Int) => true,
            (DataType::UInt8, DataType::Float) => true,
            (DataType::UInt8, DataType::String) => true,
            (DataType::UInt16, DataType::Int) => true,
            (DataType::UInt16, DataType::Float) => true,
            (DataType::UInt16, DataType::String) => true,
            (DataType::UInt32, DataType::Int) => true,
            (DataType::UInt32, DataType::Float) => true,
            (DataType::UInt32, DataType::String) => true,
            (DataType::UInt64, DataType::Int) => true,
            (DataType::UInt64, DataType::Float) => true,
            (DataType::UInt64, DataType::String) => true,

            // The value “Float” can be converted to either “Float”, “Int”, or “String”.
            (DataType::Float, DataType::Int) => true,
            (DataType::Float, DataType::String) => true,

            // A String can be converted to a String, Int, Float, Bool, Date, or DateTime.
            (DataType::String, DataType::Int) => true,
            (DataType::String, DataType::Float) => true,
            (DataType::String, DataType::Bool) => true,
            (DataType::String, DataType::Date) => true,
            (DataType::String, DataType::DateTime) => true,

            // The FixedString type can be converted to the following types: String, Int, Float, Bool, Date, and DateTime.
            (DataType::FixedString(_), DataType::String) => true,
            (DataType::FixedString(_), DataType::Int) => true,
            (DataType::FixedString(_), DataType::Float) => true,
            (DataType::FixedString(_), DataType::Bool) => true,
            (DataType::FixedString(_), DataType::Date) => true,
            (DataType::FixedString(_), DataType::DateTime) => true,

            // A `Bool` value can be converted to a `Bool`, `Int`, `Float`, or `String`.
            (DataType::Bool, DataType::Int) => true,
            (DataType::Bool, DataType::Float) => true,
            (DataType::Bool, DataType::String) => true,

            // The value “Null” can be converted into any data type.
            (DataType::Null, _) => true,

            // “Empty” can be converted to “Empty”, “Bool”, “Int”, “Float”, or “String”.
            (DataType::Empty, DataType::Empty) => true,
            (DataType::Empty, DataType::Bool) => true,
            (DataType::Empty, DataType::Int) => true,
            (DataType::Empty, DataType::Float) => true,
            (DataType::Empty, DataType::String) => true,

            _ => false,
        }
    }

    /// The list of source types that can be converted into all possible target types
    ///
    /// Return a list of all target types that can be converted from this type.
    pub fn get_cast_targets(from: &DataType) -> Vec<DataType> {
        match from {
            DataType::Int => vec![DataType::Int, DataType::Float, DataType::String],
            DataType::Int8 => vec![
                DataType::Int8,
                DataType::Int,
                DataType::Float,
                DataType::String,
            ],
            DataType::Int16 => vec![
                DataType::Int16,
                DataType::Int,
                DataType::Float,
                DataType::String,
            ],
            DataType::Int32 => vec![
                DataType::Int32,
                DataType::Int,
                DataType::Float,
                DataType::String,
            ],
            DataType::Int64 => vec![
                DataType::Int64,
                DataType::Int,
                DataType::Float,
                DataType::String,
            ],
            DataType::UInt8 => vec![
                DataType::UInt8,
                DataType::Int,
                DataType::Float,
                DataType::String,
            ],
            DataType::UInt16 => vec![
                DataType::UInt16,
                DataType::Int,
                DataType::Float,
                DataType::String,
            ],
            DataType::UInt32 => vec![
                DataType::UInt32,
                DataType::Int,
                DataType::Float,
                DataType::String,
            ],
            DataType::UInt64 => vec![
                DataType::UInt64,
                DataType::Int,
                DataType::Float,
                DataType::String,
            ],
            DataType::Float => vec![DataType::Float, DataType::Int, DataType::String],
            DataType::String => vec![
                DataType::String,
                DataType::Int,
                DataType::Float,
                DataType::Bool,
                DataType::Date,
                DataType::DateTime,
            ],
            DataType::FixedString(_) => vec![
                DataType::String,
                DataType::Int,
                DataType::Float,
                DataType::Bool,
                DataType::Date,
                DataType::DateTime,
            ],
            DataType::Bool => vec![
                DataType::Bool,
                DataType::Int,
                DataType::Float,
                DataType::String,
            ],
            DataType::Null => vec![
                DataType::Null,
                DataType::Int,
                DataType::Float,
                DataType::String,
                DataType::Bool,
            ],
            DataType::Empty => vec![
                DataType::Empty,
                DataType::Bool,
                DataType::Int,
                DataType::Float,
                DataType::String,
            ],
            // Other types can only be converted into themselves.
            _ => vec![from.clone()],
        }
    }

    /// Verify whether the type conversion is valid (based on NebulaGraph design)
    pub fn validate_type_cast(from: &DataType, to: &DataType) -> bool {
        Self::can_cast(from, to)
    }

    /// The string representation of the obtained type.
    pub fn type_to_string(type_def: &DataType) -> String {
        match type_def {
            DataType::Empty => "empty".to_string(),
            DataType::Null => "null".to_string(),
            DataType::Bool => "bool".to_string(),
            DataType::Int
            | DataType::Int8
            | DataType::Int16
            | DataType::Int32
            | DataType::Int64
            | DataType::UInt8
            | DataType::UInt16
            | DataType::UInt32
            | DataType::UInt64 => "int".to_string(),
            DataType::Float | DataType::Double => "float".to_string(),
            DataType::Decimal128 => "decimal128".to_string(),
            DataType::String => "string".to_string(),
            DataType::FixedString(len) => format!("fixed_string({})", len),
            DataType::Date => "date".to_string(),
            DataType::Time => "time".to_string(),
            DataType::Timestamp => "timestamp".to_string(),
            DataType::DateTime => "datetime".to_string(),
            DataType::VID => "vid".to_string(),
            DataType::Vertex => "vertex".to_string(),
            DataType::Edge => "edge".to_string(),
            DataType::Path => "path".to_string(),
            DataType::List => "list".to_string(),
            DataType::Map => "map".to_string(),
            DataType::Set => "set".to_string(),
            DataType::Blob => "blob".to_string(),
            DataType::Geography => "geography".to_string(),
            DataType::Duration => "duration".to_string(),
            DataType::DataSet => "dataset".to_string(),
        }
    }

    /// Check whether the type can be used for indexing.
    pub fn is_indexable_type(type_def: &DataType) -> bool {
        matches!(
            type_def,
            DataType::Bool
                | DataType::Int
                | DataType::Int8
                | DataType::Int16
                | DataType::Int32
                | DataType::Int64
                | DataType::UInt8
                | DataType::UInt16
                | DataType::UInt32
                | DataType::UInt64
                | DataType::Float
                | DataType::Double
                | DataType::String
                | DataType::FixedString(_)
                | DataType::DateTime
                | DataType::Date
                | DataType::Time
                | DataType::Timestamp
                | DataType::VID
                | DataType::Blob
                | DataType::Geography
        )
    }

    /// Get the default value of the type.
    pub fn get_default_value(type_def: &DataType) -> Option<Value> {
        match type_def {
            DataType::Bool => Some(Value::Bool(false)),
            DataType::Int => Some(Value::Int(0)),
            DataType::Int8 => Some(Value::Int8(0)),
            DataType::Int16 => Some(Value::Int16(0)),
            DataType::Int32 => Some(Value::Int32(0)),
            DataType::Int64 => Some(Value::Int64(0)),
            DataType::UInt8 => Some(Value::UInt8(0)),
            DataType::UInt16 => Some(Value::UInt16(0)),
            DataType::UInt32 => Some(Value::UInt32(0)),
            DataType::UInt64 => Some(Value::UInt64(0)),
            DataType::Float => Some(Value::Float(0.0)),
            DataType::String => Some(Value::String(String::new())),
            DataType::List => Some(Value::List(List::from(Vec::new()))),
            DataType::Map => Some(Value::Map(std::collections::HashMap::new())),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_are_types_compatible() {
        assert!(TypeUtils::are_types_compatible(
            &DataType::Int,
            &DataType::Int
        ));

        assert!(TypeUtils::are_types_compatible(
            &DataType::Null,
            &DataType::Int
        ));
        assert!(TypeUtils::are_types_compatible(
            &DataType::Empty,
            &DataType::String
        ));

        assert!(TypeUtils::are_types_compatible(
            &DataType::Int,
            &DataType::Float
        ));
        assert!(TypeUtils::are_types_compatible(
            &DataType::Float,
            &DataType::Int
        ));

        assert!(!TypeUtils::are_types_compatible(
            &DataType::Int,
            &DataType::String
        ));
    }

    #[test]
    fn test_is_superior_type() {
        assert!(TypeUtils::is_superior_type(&DataType::Null));
        assert!(TypeUtils::is_superior_type(&DataType::Empty));
        assert!(!TypeUtils::is_superior_type(&DataType::Int));
        assert!(!TypeUtils::is_superior_type(&DataType::String));
    }

    #[test]
    fn test_get_type_priority() {
        assert_eq!(TypeUtils::get_type_priority(&DataType::Null), 0);
        assert_eq!(TypeUtils::get_type_priority(&DataType::Int), 20);
        assert_eq!(TypeUtils::get_type_priority(&DataType::Float), 30);
        assert_eq!(TypeUtils::get_type_priority(&DataType::String), 40);
    }

    #[test]
    fn test_get_common_type() {
        assert_eq!(
            TypeUtils::get_common_type(&DataType::Int, &DataType::Float),
            DataType::Float
        );
        assert_eq!(
            TypeUtils::get_common_type(&DataType::Null, &DataType::String),
            DataType::String
        );
        assert_eq!(
            TypeUtils::get_common_type(&DataType::Int, &DataType::String),
            DataType::Empty
        );
    }

    #[test]
    fn test_check_compatibility() {
        assert!(TypeUtils::check_compatibility(
            &DataType::Int,
            &DataType::Int
        ));
        assert!(TypeUtils::check_compatibility(
            &DataType::Int,
            &DataType::Float
        ));
        assert!(!TypeUtils::check_compatibility(
            &DataType::Int,
            &DataType::String
        ));
    }

    #[test]
    fn test_check_compatibility_batch() {
        let pairs = vec![
            (DataType::Int, DataType::Int),
            (DataType::Int, DataType::Float),
            (DataType::Int, DataType::String),
            (DataType::Null, DataType::Int),
        ];

        let results = TypeUtils::check_compatibility_batch(&pairs);
        assert_eq!(results.len(), 4);
        assert!(results[0]);
        assert!(results[1]);
        assert!(!results[2]);
        assert!(results[3]);
    }

    #[test]
    fn test_literal_type() {
        use crate::core::value::Value;
        use std::f64::consts::PI;

        assert_eq!(TypeUtils::literal_type(&Value::Int(42)), DataType::Int);
        assert_eq!(TypeUtils::literal_type(&Value::Float(PI)), DataType::Float);
        assert_eq!(
            TypeUtils::literal_type(&Value::String("test".to_string())),
            DataType::String
        );
    }

    #[test]
    fn test_binary_operation_result_type() {
        assert_eq!(
            TypeUtils::binary_operation_result_type("+", &DataType::Int, &DataType::Int),
            DataType::Int
        );
        assert_eq!(
            TypeUtils::binary_operation_result_type("+", &DataType::Int, &DataType::Float),
            DataType::Float
        );
        assert_eq!(
            TypeUtils::binary_operation_result_type("==", &DataType::Int, &DataType::Int),
            DataType::Bool
        );
    }

    #[test]
    fn test_should_cache_expression() {
        assert!(!TypeUtils::should_cache_expression(2, 5));
        assert!(TypeUtils::should_cache_expression(4, 5));
        assert!(TypeUtils::should_cache_expression(2, 15));
    }

    #[test]
    fn test_can_cast() {
        // The same type
        assert!(TypeUtils::can_cast(&DataType::Int, &DataType::Int));
        assert!(TypeUtils::can_cast(&DataType::String, &DataType::String));

        // Int conversion
        assert!(TypeUtils::can_cast(&DataType::Int, &DataType::Float));
        assert!(TypeUtils::can_cast(&DataType::Int, &DataType::String));
        assert!(!TypeUtils::can_cast(&DataType::Int, &DataType::Bool));

        // Float conversion
        assert!(TypeUtils::can_cast(&DataType::Float, &DataType::Int));
        assert!(TypeUtils::can_cast(&DataType::Float, &DataType::String));

        // String conversion
        assert!(TypeUtils::can_cast(&DataType::String, &DataType::Int));
        assert!(TypeUtils::can_cast(&DataType::String, &DataType::Float));
        assert!(TypeUtils::can_cast(&DataType::String, &DataType::Bool));
        assert!(TypeUtils::can_cast(&DataType::String, &DataType::Date));
        assert!(TypeUtils::can_cast(&DataType::String, &DataType::DateTime));

        // Boolean conversion
        assert!(TypeUtils::can_cast(&DataType::Bool, &DataType::Int));
        assert!(TypeUtils::can_cast(&DataType::Bool, &DataType::Float));
        assert!(TypeUtils::can_cast(&DataType::Bool, &DataType::String));

        // The value “Null” can be converted into any data type.
        assert!(TypeUtils::can_cast(&DataType::Null, &DataType::Int));
        assert!(TypeUtils::can_cast(&DataType::Null, &DataType::String));
        assert!(TypeUtils::can_cast(&DataType::Null, &DataType::Bool));

        // "Empty" conversion
        assert!(TypeUtils::can_cast(&DataType::Empty, &DataType::Int));
        assert!(TypeUtils::can_cast(&DataType::Empty, &DataType::String));
        assert!(TypeUtils::can_cast(&DataType::Empty, &DataType::Bool));

        // The conversion was invalid.
        assert!(!TypeUtils::can_cast(&DataType::Int, &DataType::Date));
        assert!(!TypeUtils::can_cast(&DataType::Float, &DataType::Bool));
    }

    #[test]
    fn test_get_cast_targets() {
        let int_targets = TypeUtils::get_cast_targets(&DataType::Int);
        assert!(int_targets.contains(&DataType::Int));
        assert!(int_targets.contains(&DataType::Float));
        assert!(int_targets.contains(&DataType::String));

        let string_targets = TypeUtils::get_cast_targets(&DataType::String);
        assert!(string_targets.contains(&DataType::String));
        assert!(string_targets.contains(&DataType::Int));
        assert!(string_targets.contains(&DataType::Float));
        assert!(string_targets.contains(&DataType::Bool));

        let null_targets = TypeUtils::get_cast_targets(&DataType::Null);
        assert!(null_targets.contains(&DataType::Int));
        assert!(null_targets.contains(&DataType::Float));
        assert!(null_targets.contains(&DataType::String));
        assert!(null_targets.contains(&DataType::Bool));
    }

    #[test]
    fn test_validate_type_cast() {
        assert!(TypeUtils::validate_type_cast(
            &DataType::Int,
            &DataType::Float
        ));
        assert!(!TypeUtils::validate_type_cast(
            &DataType::Int,
            &DataType::Bool
        ));
    }

    #[test]
    fn test_type_to_string() {
        assert_eq!(TypeUtils::type_to_string(&DataType::Int), "int");
        assert_eq!(TypeUtils::type_to_string(&DataType::Float), "float");
        assert_eq!(TypeUtils::type_to_string(&DataType::String), "string");
        assert_eq!(TypeUtils::type_to_string(&DataType::Bool), "bool");
        assert_eq!(
            TypeUtils::type_to_string(&DataType::FixedString(100)),
            "fixed_string(100)"
        );
    }

    #[test]
    fn test_is_indexable_type() {
        assert!(TypeUtils::is_indexable_type(&DataType::Int));
        assert!(TypeUtils::is_indexable_type(&DataType::Float));
        assert!(TypeUtils::is_indexable_type(&DataType::String));
        assert!(TypeUtils::is_indexable_type(&DataType::Bool));
        assert!(!TypeUtils::is_indexable_type(&DataType::Null));
        assert!(!TypeUtils::is_indexable_type(&DataType::List));
    }

    #[test]
    fn test_get_default_value() {
        assert_eq!(
            TypeUtils::get_default_value(&DataType::Bool),
            Some(Value::Bool(false))
        );
        assert_eq!(
            TypeUtils::get_default_value(&DataType::Int),
            Some(Value::Int(0))
        );
        assert_eq!(
            TypeUtils::get_default_value(&DataType::Float),
            Some(Value::Float(0.0))
        );
        assert_eq!(
            TypeUtils::get_default_value(&DataType::String),
            Some(Value::String(String::new()))
        );
        assert!(TypeUtils::get_default_value(&DataType::Null).is_none());
    }
}
