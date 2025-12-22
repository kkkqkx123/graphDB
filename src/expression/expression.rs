//! 表达式类型定义 - 已迁移到Core模块
//!
//! 所有表达式类型定义已迁移到 src/core/types/expression.rs
//! 请使用 crate::core::Expression 替代 crate::expression::Expression

// 重新导出Core模块中的表达式类型，以保持向后兼容性
pub use crate::core::types::expression::*;