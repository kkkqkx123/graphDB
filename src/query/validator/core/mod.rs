//! 验证器核心模块
//!
//! 提供验证器的基础 trait、类型和枚举定义

pub mod strategy;
pub mod types;
pub mod validator;

// 重新导出主要类型
pub use strategy::{
    semantic_error, syntax_error, type_error, type_mismatch, DefaultStrategySet, StrategyResult,
    StrategySet, ValidationStrategy, ValidationStrategyType,
};
pub use types::{
    ColumnDef, EdgeProperty, ExpressionProps, InputProperty, StatementType, TagProperty,
    VarProperty,
};
pub use validator::{StatementValidator, Validator, ValidatorBuilder};
