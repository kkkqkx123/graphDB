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

        // NULL和EMPTY类型与任何类型兼容
        if Self::is_superior_type(type1) || Self::is_superior_type(type2) {
            return true;
        }

        // Int和Float可以相互兼容
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
            // 其他类型的默认优先级
            _ => 14,
        }
    }

    /// 获取两个类型的公共超类型
    pub fn get_common_type(type1: &ValueTypeDef, type2: &ValueTypeDef) -> ValueTypeDef {
        if type1 == type2 {
            return type1.clone();
        }

        // 如果其中一个是NULL或EMPTY，返回另一个
        if Self::is_superior_type(type1) {
            return type2.clone();
        }
        if Self::is_superior_type(type2) {
            return type1.clone();
        }

        // Int和Float的公共类型是Float
        if (type1 == &ValueTypeDef::Int && type2 == &ValueTypeDef::Float)
            || (type1 == &ValueTypeDef::Float && type2 == &ValueTypeDef::Int)
        {
            return ValueTypeDef::Float;
        }

        // 其他情况返回Empty
        ValueTypeDef::Empty
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_are_types_compatible() {
        // 相同类型兼容
        assert!(TypeUtils::are_types_compatible(
            &ValueTypeDef::Int,
            &ValueTypeDef::Int
        ));

        // 优越类型与任何类型兼容
        assert!(TypeUtils::are_types_compatible(
            &ValueTypeDef::Null,
            &ValueTypeDef::Int
        ));
        assert!(TypeUtils::are_types_compatible(
            &ValueTypeDef::Empty,
            &ValueTypeDef::String
        ));

        // Int和Float兼容
        assert!(TypeUtils::are_types_compatible(
            &ValueTypeDef::Int,
            &ValueTypeDef::Float
        ));
        assert!(TypeUtils::are_types_compatible(
            &ValueTypeDef::Float,
            &ValueTypeDef::Int
        ));

        // 不同类型不兼容
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
}
