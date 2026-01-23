//! 类型系统工具模块
//!
//! 提供类型兼容性检查、类型优先级和类型转换等核心功能

use crate::core::ValueTypeDef;
use crate::core::Value;
use std::collections::HashMap;

/// 类型转换映射表
/// 记录每种类型可以转换到哪些目标类型
static TYPE_CAST_MAP: std::sync::LazyLock<HashMap<ValueTypeDef, Vec<ValueTypeDef>>> =
    std::sync::LazyLock::new(|| {
        let mut map = HashMap::new();

        // 转换为 Int
        map.insert(
            ValueTypeDef::Int,
            vec![ValueTypeDef::Int, ValueTypeDef::Float, ValueTypeDef::String],
        );

        // 转换为 Float
        map.insert(
            ValueTypeDef::Float,
            vec![ValueTypeDef::Float, ValueTypeDef::Int, ValueTypeDef::String],
        );

        // 转换为 String
        map.insert(
            ValueTypeDef::String,
            vec![
                ValueTypeDef::String,
                ValueTypeDef::Int,
                ValueTypeDef::Float,
                ValueTypeDef::Bool,
                ValueTypeDef::Date,
                ValueTypeDef::DateTime,
            ],
        );

        // 转换为 Bool
        map.insert(
            ValueTypeDef::Bool,
            vec![
                ValueTypeDef::Bool,
                ValueTypeDef::Int,
                ValueTypeDef::Float,
                ValueTypeDef::String,
            ],
        );

        // 优越类型可以转换为任何类型
        map.insert(
            ValueTypeDef::Null,
            vec![
                ValueTypeDef::Null,
                ValueTypeDef::Int,
                ValueTypeDef::Float,
                ValueTypeDef::String,
                ValueTypeDef::Bool,
            ],
        );

        map.insert(
            ValueTypeDef::Empty,
            vec![
                ValueTypeDef::Empty,
                ValueTypeDef::Bool,
                ValueTypeDef::Int,
                ValueTypeDef::Float,
                ValueTypeDef::String,
            ],
        );

        map
    });

/// 类型系统工具
pub struct TypeUtils;

impl TypeUtils {
    /// 检查两种类型是否兼容
    pub fn are_types_compatible(type1: &ValueTypeDef, type2: &ValueTypeDef) -> bool {
        if type1 == type2 {
            return true;
        }

        if Self::is_superior_type(type1) || Self::is_superior_type(type2) {
            return true;
        }

        if (type1 == &ValueTypeDef::Int && type2 == &ValueTypeDef::Float)
            || (type1 == &ValueTypeDef::Float && type2 == &ValueTypeDef::Int)
        {
            return true;
        }

        false
    }

    /// 检查类型是否为"优越类型"（可以与任何类型兼容）
    pub fn is_superior_type(type_: &ValueTypeDef) -> bool {
        matches!(type_, ValueTypeDef::Null | ValueTypeDef::Empty)
    }

    /// 获取类型的优先级（用于类型提升）
    pub fn get_type_priority(type_: &ValueTypeDef) -> u8 {
        match type_ {
            ValueTypeDef::Null | ValueTypeDef::Empty => 0,
            ValueTypeDef::Bool => 1,
            ValueTypeDef::Int => 2,
            ValueTypeDef::Float => 3,
            ValueTypeDef::String => 4,
            ValueTypeDef::Date => 5,
            ValueTypeDef::Time => 6,
            ValueTypeDef::DateTime => 7,
            ValueTypeDef::Vertex => 8,
            ValueTypeDef::Edge => 9,
            ValueTypeDef::Path => 10,
            ValueTypeDef::List => 11,
            ValueTypeDef::Set => 12,
            ValueTypeDef::Map => 13,
            _ => 14,
        }
    }

    /// 获取两个类型的公共超类型
    pub fn get_common_type(type1: &ValueTypeDef, type2: &ValueTypeDef) -> ValueTypeDef {
        if type1 == type2 {
            return type1.clone();
        }

        if Self::is_superior_type(type1) {
            return type2.clone();
        }
        if Self::is_superior_type(type2) {
            return type1.clone();
        }

        if (type1 == &ValueTypeDef::Int && type2 == &ValueTypeDef::Float)
            || (type1 == &ValueTypeDef::Float && type2 == &ValueTypeDef::Int)
        {
            return ValueTypeDef::Float;
        }

        ValueTypeDef::Empty
    }

    /// 统一的类型兼容性检查（无需缓存）
    pub fn check_compatibility(type1: &ValueTypeDef, type2: &ValueTypeDef) -> bool {
        Self::are_types_compatible(type1, type2)
    }

    /// 批量类型检查（优化内存分配）
    pub fn check_compatibility_batch(pairs: &[(ValueTypeDef, ValueTypeDef)]) -> Vec<bool> {
        let mut results = Vec::with_capacity(pairs.len());

        for (t1, t2) in pairs {
            results.push(Self::check_compatibility(t1, t2));
        }
        results
    }

    /// 获取字面值类型
    pub fn literal_type(value: &crate::core::value::Value) -> ValueTypeDef {
        value.get_type()
    }

    /// 二元操作结果类型
    pub fn binary_operation_result_type(
        op: &str,
        left_type: &ValueTypeDef,
        right_type: &ValueTypeDef,
    ) -> ValueTypeDef {
        match op {
            "+" | "-" | "*" | "/" => {
                if left_type == &ValueTypeDef::Float || right_type == &ValueTypeDef::Float {
                    ValueTypeDef::Float
                } else {
                    ValueTypeDef::Int
                }
            }
            "==" | "!=" | "<" | "<=" | ">" | ">=" => {
                ValueTypeDef::Bool
            }
            _ => ValueTypeDef::Empty,
        }
    }

    /// 判断是否需要缓存（基于复杂度启发式）
    pub fn should_cache_expression(expr_depth: usize, expr_node_count: usize) -> bool {
        expr_depth > 3 || expr_node_count > 10
    }

    /// 检查类型是否可以转换为目标类型
    pub fn can_cast(from: &ValueTypeDef, to: &ValueTypeDef) -> bool {
        if from == to {
            return true;
        }
        if let Some(targets) = TYPE_CAST_MAP.get(from) {
            return targets.contains(to);
        }
        false
    }

    /// 获取类型可以转换到的所有目标类型
    pub fn get_cast_targets(from: &ValueTypeDef) -> Vec<ValueTypeDef> {
        TYPE_CAST_MAP
            .get(from)
            .cloned()
            .unwrap_or_else(Vec::new)
    }

    /// 验证类型转换是否有效（基于 NebulaGraph 设计）
    pub fn validate_type_cast(from: &ValueTypeDef, to: &ValueTypeDef) -> bool {
        Self::can_cast(from, to)
    }

    /// 获取类型的字符串表示
    pub fn type_to_string(type_def: &ValueTypeDef) -> String {
        match type_def {
            ValueTypeDef::Empty => "empty".to_string(),
            ValueTypeDef::Null => "null".to_string(),
            ValueTypeDef::Bool => "bool".to_string(),
            ValueTypeDef::Int | ValueTypeDef::Int8 | ValueTypeDef::Int16 | ValueTypeDef::Int32 | ValueTypeDef::Int64 => "int".to_string(),
            ValueTypeDef::Float | ValueTypeDef::Double => "float".to_string(),
            ValueTypeDef::String => "string".to_string(),
            ValueTypeDef::Date => "date".to_string(),
            ValueTypeDef::Time => "time".to_string(),
            ValueTypeDef::DateTime => "datetime".to_string(),
            ValueTypeDef::Vertex => "vertex".to_string(),
            ValueTypeDef::Edge => "edge".to_string(),
            ValueTypeDef::Path => "path".to_string(),
            ValueTypeDef::List => "list".to_string(),
            ValueTypeDef::Map => "map".to_string(),
            ValueTypeDef::Set => "set".to_string(),
            ValueTypeDef::Geography => "geography".to_string(),
            ValueTypeDef::Duration => "duration".to_string(),
            ValueTypeDef::DataSet => "dataset".to_string(),
        }
    }

    /// 检查类型是否可以用于索引
    pub fn is_indexable_type(type_def: &ValueTypeDef) -> bool {
        match type_def {
            ValueTypeDef::Bool => true,
            ValueTypeDef::Int => true,
            ValueTypeDef::Float => true,
            ValueTypeDef::String => true,
            ValueTypeDef::DateTime => true,
            ValueTypeDef::Date => true,
            ValueTypeDef::Time => true,
            ValueTypeDef::Duration => true,
            ValueTypeDef::Geography => true,
            _ => false,
        }
    }

    /// 获取类型的默认值
    pub fn get_default_value(type_def: &ValueTypeDef) -> Option<Value> {
        match type_def {
            ValueTypeDef::Bool => Some(Value::Bool(false)),
            ValueTypeDef::Int => Some(Value::Int(0)),
            ValueTypeDef::Float => Some(Value::Float(0.0)),
            ValueTypeDef::String => Some(Value::String(String::new())),
            ValueTypeDef::List => Some(Value::List(Vec::new())),
            ValueTypeDef::Map => Some(Value::Map(std::collections::HashMap::new())),
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
            &ValueTypeDef::Int,
            &ValueTypeDef::Int
        ));

        assert!(TypeUtils::are_types_compatible(
            &ValueTypeDef::Null,
            &ValueTypeDef::Int
        ));
        assert!(TypeUtils::are_types_compatible(
            &ValueTypeDef::Empty,
            &ValueTypeDef::String
        ));

        assert!(TypeUtils::are_types_compatible(
            &ValueTypeDef::Int,
            &ValueTypeDef::Float
        ));
        assert!(TypeUtils::are_types_compatible(
            &ValueTypeDef::Float,
            &ValueTypeDef::Int
        ));

        assert!(!TypeUtils::are_types_compatible(
            &ValueTypeDef::Int,
            &ValueTypeDef::String
        ));
    }

    #[test]
    fn test_is_superior_type() {
        assert!(TypeUtils::is_superior_type(&ValueTypeDef::Null));
        assert!(TypeUtils::is_superior_type(&ValueTypeDef::Empty));
        assert!(!TypeUtils::is_superior_type(&ValueTypeDef::Int));
        assert!(!TypeUtils::is_superior_type(&ValueTypeDef::String));
    }

    #[test]
    fn test_get_type_priority() {
        assert_eq!(TypeUtils::get_type_priority(&ValueTypeDef::Null), 0);
        assert_eq!(TypeUtils::get_type_priority(&ValueTypeDef::Int), 2);
        assert_eq!(TypeUtils::get_type_priority(&ValueTypeDef::Float), 3);
        assert_eq!(TypeUtils::get_type_priority(&ValueTypeDef::String), 4);
    }

    #[test]
    fn test_get_common_type() {
        assert_eq!(
            TypeUtils::get_common_type(&ValueTypeDef::Int, &ValueTypeDef::Float),
            ValueTypeDef::Float
        );
        assert_eq!(
            TypeUtils::get_common_type(&ValueTypeDef::Null, &ValueTypeDef::String),
            ValueTypeDef::String
        );
        assert_eq!(
            TypeUtils::get_common_type(&ValueTypeDef::Int, &ValueTypeDef::String),
            ValueTypeDef::Empty
        );
    }

    #[test]
    fn test_check_compatibility() {
        assert!(TypeUtils::check_compatibility(
            &ValueTypeDef::Int,
            &ValueTypeDef::Int
        ));
        assert!(TypeUtils::check_compatibility(
            &ValueTypeDef::Int,
            &ValueTypeDef::Float
        ));
        assert!(!TypeUtils::check_compatibility(
            &ValueTypeDef::Int,
            &ValueTypeDef::String
        ));
    }

    #[test]
    fn test_check_compatibility_batch() {
        let pairs = vec![
            (ValueTypeDef::Int, ValueTypeDef::Int),
            (ValueTypeDef::Int, ValueTypeDef::Float),
            (ValueTypeDef::Int, ValueTypeDef::String),
            (ValueTypeDef::Null, ValueTypeDef::Int),
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

        assert_eq!(TypeUtils::literal_type(&Value::Int(42)), ValueTypeDef::Int);
        assert_eq!(
            TypeUtils::literal_type(&Value::String("test".to_string())),
            ValueTypeDef::String
        );
        assert_eq!(
            TypeUtils::literal_type(&Value::Bool(true)),
            ValueTypeDef::Bool
        );
    }

    #[test]
    fn test_binary_operation_result_type() {
        assert_eq!(
            TypeUtils::binary_operation_result_type("+", &ValueTypeDef::Int, &ValueTypeDef::Int),
            ValueTypeDef::Int
        );
        assert_eq!(
            TypeUtils::binary_operation_result_type("+", &ValueTypeDef::Int, &ValueTypeDef::Float),
            ValueTypeDef::Float
        );

        assert_eq!(
            TypeUtils::binary_operation_result_type("==", &ValueTypeDef::Int, &ValueTypeDef::Int),
            ValueTypeDef::Bool
        );
    }

    #[test]
    fn test_should_cache_expression() {
        assert!(!TypeUtils::should_cache_expression(2, 5));
        assert!(TypeUtils::should_cache_expression(4, 5));
        assert!(TypeUtils::should_cache_expression(2, 15));
        assert!(TypeUtils::should_cache_expression(5, 20));
    }

    #[test]
    fn test_can_cast() {
        assert!(TypeUtils::can_cast(&ValueTypeDef::Int, &ValueTypeDef::Int));
        assert!(TypeUtils::can_cast(&ValueTypeDef::Int, &ValueTypeDef::Float));
        assert!(TypeUtils::can_cast(&ValueTypeDef::Int, &ValueTypeDef::String));
        assert!(!TypeUtils::can_cast(&ValueTypeDef::Int, &ValueTypeDef::Bool));

        assert!(TypeUtils::can_cast(&ValueTypeDef::Float, &ValueTypeDef::Float));
        assert!(TypeUtils::can_cast(&ValueTypeDef::Float, &ValueTypeDef::Int));
        assert!(TypeUtils::can_cast(&ValueTypeDef::Float, &ValueTypeDef::String));

        assert!(TypeUtils::can_cast(&ValueTypeDef::Null, &ValueTypeDef::Int));
        assert!(TypeUtils::can_cast(&ValueTypeDef::Null, &ValueTypeDef::String));
    }

    #[test]
    fn test_get_cast_targets() {
        let int_targets = TypeUtils::get_cast_targets(&ValueTypeDef::Int);
        assert!(int_targets.contains(&ValueTypeDef::Int));
        assert!(int_targets.contains(&ValueTypeDef::Float));
        assert!(int_targets.contains(&ValueTypeDef::String));

        let float_targets = TypeUtils::get_cast_targets(&ValueTypeDef::Float);
        assert!(float_targets.contains(&ValueTypeDef::Float));
        assert!(float_targets.contains(&ValueTypeDef::Int));
        assert!(float_targets.contains(&ValueTypeDef::String));

        // Bool 有定义的转换规则
        let bool_targets = TypeUtils::get_cast_targets(&ValueTypeDef::Bool);
        assert!(bool_targets.contains(&ValueTypeDef::Bool));
        assert!(bool_targets.contains(&ValueTypeDef::Int));
    }

    #[test]
    fn test_validate_type_cast() {
        assert!(TypeUtils::validate_type_cast(&ValueTypeDef::Int, &ValueTypeDef::Float));
        assert!(TypeUtils::validate_type_cast(&ValueTypeDef::Float, &ValueTypeDef::String));
        // String 可以转换为 Int（根据 NebulaGraph 规范）
        assert!(TypeUtils::validate_type_cast(&ValueTypeDef::String, &ValueTypeDef::Int));
        // Date 不能转换为 Int
        assert!(!TypeUtils::validate_type_cast(&ValueTypeDef::Date, &ValueTypeDef::Int));
    }

    #[test]
    fn test_type_to_string() {
        assert_eq!(TypeUtils::type_to_string(&ValueTypeDef::Int), "int");
        assert_eq!(TypeUtils::type_to_string(&ValueTypeDef::Float), "float");
        assert_eq!(TypeUtils::type_to_string(&ValueTypeDef::String), "string");
        assert_eq!(TypeUtils::type_to_string(&ValueTypeDef::Bool), "bool");
        assert_eq!(TypeUtils::type_to_string(&ValueTypeDef::DateTime), "datetime");
        assert_eq!(TypeUtils::type_to_string(&ValueTypeDef::Vertex), "vertex");
        assert_eq!(TypeUtils::type_to_string(&ValueTypeDef::Edge), "edge");
        assert_eq!(TypeUtils::type_to_string(&ValueTypeDef::Path), "path");
        assert_eq!(TypeUtils::type_to_string(&ValueTypeDef::List), "list");
        assert_eq!(TypeUtils::type_to_string(&ValueTypeDef::Map), "map");
        assert_eq!(TypeUtils::type_to_string(&ValueTypeDef::Set), "set");
    }

    #[test]
    fn test_is_indexable_type() {
        assert!(TypeUtils::is_indexable_type(&ValueTypeDef::Bool));
        assert!(TypeUtils::is_indexable_type(&ValueTypeDef::Int));
        assert!(TypeUtils::is_indexable_type(&ValueTypeDef::Float));
        assert!(TypeUtils::is_indexable_type(&ValueTypeDef::String));
        assert!(TypeUtils::is_indexable_type(&ValueTypeDef::DateTime));
        assert!(TypeUtils::is_indexable_type(&ValueTypeDef::Date));
        assert!(TypeUtils::is_indexable_type(&ValueTypeDef::Time));
        assert!(TypeUtils::is_indexable_type(&ValueTypeDef::Duration));
        assert!(TypeUtils::is_indexable_type(&ValueTypeDef::Geography));
        
        assert!(!TypeUtils::is_indexable_type(&ValueTypeDef::Vertex));
        assert!(!TypeUtils::is_indexable_type(&ValueTypeDef::Edge));
        assert!(!TypeUtils::is_indexable_type(&ValueTypeDef::Path));
        assert!(!TypeUtils::is_indexable_type(&ValueTypeDef::List));
        assert!(!TypeUtils::is_indexable_type(&ValueTypeDef::Map));
        assert!(!TypeUtils::is_indexable_type(&ValueTypeDef::Set));
    }

    #[test]
    fn test_get_default_value() {
        assert_eq!(TypeUtils::get_default_value(&ValueTypeDef::Bool), Some(Value::Bool(false)));
        assert_eq!(TypeUtils::get_default_value(&ValueTypeDef::Int), Some(Value::Int(0)));
        assert_eq!(TypeUtils::get_default_value(&ValueTypeDef::Float), Some(Value::Float(0.0)));
        assert_eq!(TypeUtils::get_default_value(&ValueTypeDef::String), Some(Value::String(String::new())));
        assert_eq!(TypeUtils::get_default_value(&ValueTypeDef::List), Some(Value::List(Vec::new())));
        assert_eq!(TypeUtils::get_default_value(&ValueTypeDef::Map), Some(Value::Map(std::collections::HashMap::new())));
        
        assert_eq!(TypeUtils::get_default_value(&ValueTypeDef::Vertex), None);
        assert_eq!(TypeUtils::get_default_value(&ValueTypeDef::Edge), None);
        assert_eq!(TypeUtils::get_default_value(&ValueTypeDef::Path), None);
    }
}
