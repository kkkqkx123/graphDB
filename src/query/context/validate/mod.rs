//! 验证上下文模块
//!
//! 这个模块包含了查询验证阶段所需的所有上下文管理功能，
//! 按照功能进行了模块化拆分：
//!
//! - `types`: 基础数据类型定义
//! - `basic_context`: 基本验证上下文功能
//! - `schema`: Schema管理功能
//! - `generators`: 匿名变量和列生成器
//! - `context`: 增强验证上下文，集成所有功能

pub mod basic_context;
pub mod context;
pub mod generators;
pub mod schema;
pub mod types;

// 重新导出主要类型，方便外部使用
pub use basic_context::BasicValidationContext;
pub use context::ValidationContext;
pub use generators::{AnonColGenerator, AnonVarGenerator, GeneratorFactory};
pub use schema::{
    SchemaInfo, SchemaManager, SchemaProvider, SchemaValidationError, SchemaValidationResult,
    ValidationMode,
};
pub use types::{ColsDef, Column, SpaceInfo, Variable};

/// 验证上下文版本信息
pub const VALIDATE_CONTEXT_VERSION: &str = "1.0.0";

/// 验证上下文功能特性
pub struct ValidationContextFeatures;

impl ValidationContextFeatures {
    /// 检查是否支持Schema管理
    pub fn supports_schema_management() -> bool {
        true
    }

    /// 检查是否支持匿名生成器
    pub fn supports_anon_generators() -> bool {
        true
    }

    /// 检查是否支持符号表集成
    pub fn supports_symbol_table() -> bool {
        true
    }

    /// 检查是否支持错误收集
    pub fn supports_error_collection() -> bool {
        true
    }

    /// 获取所有支持的功能列表
    pub fn get_supported_features() -> Vec<&'static str> {
        vec![
            "schema_management",
            "anon_generators",
            "symbol_table",
            "error_collection",
            "space_management",
            "variable_management",
            "parameter_management",
            "alias_management",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(VALIDATE_CONTEXT_VERSION, "1.0.0");
    }

    #[test]
    fn test_features() {
        assert!(ValidationContextFeatures::supports_schema_management());
        assert!(ValidationContextFeatures::supports_anon_generators());
        assert!(ValidationContextFeatures::supports_symbol_table());
        assert!(ValidationContextFeatures::supports_error_collection());

        let features = ValidationContextFeatures::get_supported_features();
        assert!(features.contains(&"schema_management"));
        assert!(features.contains(&"anon_generators"));
        assert!(features.contains(&"symbol_table"));
        assert!(features.contains(&"error_collection"));
    }
}
