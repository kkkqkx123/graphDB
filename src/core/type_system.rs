//! 类型系统工具模块
//!
//! 提供类型兼容性检查、类型优先级和类型转换等核心功能

use crate::core::DataType;
use crate::core::Value;
use crate::core::value::dataset::List;

/// 类型系统工具
pub struct TypeUtils;

impl TypeUtils {
    /// 检查两种类型是否兼容
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

    /// 检查类型是否为"优越类型"（可以与任何类型兼容）
    pub fn is_superior_type(type_: &DataType) -> bool {
        matches!(type_, DataType::Null | DataType::Empty)
    }

    /// 获取类型的优先级（用于类型提升）
    /// 优先级数值越小表示类型越"基础"，类型提升时会向高优先级值提升
    pub fn get_type_priority(type_: &DataType) -> u8 {
        match type_ {
            DataType::Null | DataType::Empty => 0,
            DataType::Bool => 10,
            DataType::Int => 20,
            DataType::Int8 => 21,
            DataType::Int16 => 22,
            DataType::Int32 => 23,
            DataType::Int64 => 24,
            DataType::Float => 30,
            DataType::Double => 31,
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

    /// 获取两个类型的公共超类型
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

    /// 统一的类型兼容性检查（无需缓存）
    pub fn check_compatibility(type1: &DataType, type2: &DataType) -> bool {
        Self::are_types_compatible(type1, type2)
    }

    /// 批量类型检查（优化内存分配）
    pub fn check_compatibility_batch(pairs: &[(DataType, DataType)]) -> Vec<bool> {
        let mut results = Vec::with_capacity(pairs.len());

        for (t1, t2) in pairs {
            results.push(Self::check_compatibility(t1, t2));
        }
        results
    }

    /// 获取字面值类型
    pub fn literal_type(value: &crate::core::value::Value) -> DataType {
        value.get_type()
    }

    /// 二元操作结果类型
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
            "==" | "!=" | "<" | "<=" | ">" | ">=" => {
                DataType::Bool
            }
            _ => DataType::Empty,
        }
    }

    /// 判断是否需要缓存（基于复杂度启发式）
    pub fn should_cache_expression(expr_depth: usize, expr_node_count: usize) -> bool {
        expr_depth > 3 || expr_node_count > 10
    }

    /// 检查类型是否可以转换为目标类型
    /// 
    /// 使用 match 表达式实现编译期确定的类型转换规则，
    /// 避免运行时初始化和全局状态
    pub fn can_cast(from: &DataType, to: &DataType) -> bool {
        if from == to {
            return true;
        }

        match (from, to) {
            // Int 可以转换为 Int, Float, String
            (DataType::Int, DataType::Float) => true,
            (DataType::Int, DataType::String) => true,

            // Float 可以转换为 Float, Int, String
            (DataType::Float, DataType::Int) => true,
            (DataType::Float, DataType::String) => true,

            // String 可以转换为 String, Int, Float, Bool, Date, DateTime
            (DataType::String, DataType::Int) => true,
            (DataType::String, DataType::Float) => true,
            (DataType::String, DataType::Bool) => true,
            (DataType::String, DataType::Date) => true,
            (DataType::String, DataType::DateTime) => true,

            // Bool 可以转换为 Bool, Int, Float, String
            (DataType::Bool, DataType::Int) => true,
            (DataType::Bool, DataType::Float) => true,
            (DataType::Bool, DataType::String) => true,

            // Null 可以转换为任何类型
            (DataType::Null, _) => true,

            // Empty 可以转换为 Empty, Bool, Int, Float, String
            (DataType::Empty, DataType::Empty) => true,
            (DataType::Empty, DataType::Bool) => true,
            (DataType::Empty, DataType::Int) => true,
            (DataType::Empty, DataType::Float) => true,
            (DataType::Empty, DataType::String) => true,

            _ => false,
        }
    }

    /// 获取类型可以转换到的所有目标类型
    /// 
    /// 返回该类型可以转换到的所有目标类型列表
    pub fn get_cast_targets(from: &DataType) -> Vec<DataType> {
        match from {
            DataType::Int => vec![
                DataType::Int,
                DataType::Float,
                DataType::String,
            ],
            DataType::Float => vec![
                DataType::Float,
                DataType::Int,
                DataType::String,
            ],
            DataType::String => vec![
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
            // 其他类型只能转换为自身
            _ => vec![from.clone()],
        }
    }

    /// 验证类型转换是否有效（基于 NebulaGraph 设计）
    pub fn validate_type_cast(from: &DataType, to: &DataType) -> bool {
        Self::can_cast(from, to)
    }

    /// 获取类型的字符串表示
    pub fn type_to_string(type_def: &DataType) -> String {
        match type_def {
            DataType::Empty => "empty".to_string(),
            DataType::Null => "null".to_string(),
            DataType::Bool => "bool".to_string(),
            DataType::Int | DataType::Int8 | DataType::Int16 | DataType::Int32 | DataType::Int64 => "int".to_string(),
            DataType::Float | DataType::Double => "float".to_string(),
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

    /// 检查类型是否可以用于索引
    pub fn is_indexable_type(type_def: &DataType) -> bool {
        match type_def {
            DataType::Bool => true,
            DataType::Int => true,
            DataType::Int8 => true,
            DataType::Int16 => true,
            DataType::Int32 => true,
            DataType::Int64 => true,
            DataType::Float => true,
            DataType::Double => true,
            DataType::String => true,
            DataType::FixedString(_) => true,
            DataType::DateTime => true,
            DataType::Date => true,
            DataType::Time => true,
            DataType::Timestamp => true,
            DataType::VID => true,
            DataType::Blob => true,
            DataType::Geography => true,
            _ => false,
        }
    }

    /// 获取类型的默认值
    pub fn get_default_value(type_def: &DataType) -> Option<Value> {
        match type_def {
            DataType::Bool => Some(Value::Bool(false)),
            DataType::Int => Some(Value::Int(0)),
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

        assert_eq!(TypeUtils::literal_type(&Value::Int(42)), DataType::Int);
        assert_eq!(TypeUtils::literal_type(&Value::Float(3.14)), DataType::Float);
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
        // 相同类型
        assert!(TypeUtils::can_cast(&DataType::Int, &DataType::Int));
        assert!(TypeUtils::can_cast(&DataType::String, &DataType::String));

        // Int 转换
        assert!(TypeUtils::can_cast(&DataType::Int, &DataType::Float));
        assert!(TypeUtils::can_cast(&DataType::Int, &DataType::String));
        assert!(!TypeUtils::can_cast(&DataType::Int, &DataType::Bool));

        // Float 转换
        assert!(TypeUtils::can_cast(&DataType::Float, &DataType::Int));
        assert!(TypeUtils::can_cast(&DataType::Float, &DataType::String));

        // String 转换
        assert!(TypeUtils::can_cast(&DataType::String, &DataType::Int));
        assert!(TypeUtils::can_cast(&DataType::String, &DataType::Float));
        assert!(TypeUtils::can_cast(&DataType::String, &DataType::Bool));
        assert!(TypeUtils::can_cast(&DataType::String, &DataType::Date));
        assert!(TypeUtils::can_cast(&DataType::String, &DataType::DateTime));

        // Bool 转换
        assert!(TypeUtils::can_cast(&DataType::Bool, &DataType::Int));
        assert!(TypeUtils::can_cast(&DataType::Bool, &DataType::Float));
        assert!(TypeUtils::can_cast(&DataType::Bool, &DataType::String));

        // Null 可以转换为任何类型
        assert!(TypeUtils::can_cast(&DataType::Null, &DataType::Int));
        assert!(TypeUtils::can_cast(&DataType::Null, &DataType::String));
        assert!(TypeUtils::can_cast(&DataType::Null, &DataType::Bool));

        // Empty 转换
        assert!(TypeUtils::can_cast(&DataType::Empty, &DataType::Int));
        assert!(TypeUtils::can_cast(&DataType::Empty, &DataType::String));
        assert!(TypeUtils::can_cast(&DataType::Empty, &DataType::Bool));

        // 无效转换
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
        assert!(TypeUtils::validate_type_cast(&DataType::Int, &DataType::Float));
        assert!(!TypeUtils::validate_type_cast(&DataType::Int, &DataType::Bool));
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
