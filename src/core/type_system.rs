//! 类型系统工具模块
//!
//! 提供类型兼容性检查、类型优先级和类型转换等核心功能

use crate::core::ValueTypeDef;

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
}
