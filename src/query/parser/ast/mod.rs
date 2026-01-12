//! 简化版 AST 模块 (v2)
//!
//! 本模块提供基于枚举的简化 AST 设计，减少样板代码和运行时开销。

// 基础类型定义
pub mod types;
pub use types::*;

// 表达式定义
pub mod expr;
pub use expr::*;

// 语句定义
pub mod stmt;
pub use stmt::*;

// 模式定义
pub mod pattern;
pub use pattern::*;

// 简化的访问者模式
pub mod visitor;
pub use visitor::*;

// 解析器实现
pub mod expr_parser;
pub use expr_parser::*;

pub mod stmt_parser;
pub use stmt_parser::*;

pub mod pattern_parser;
pub use pattern_parser::*;

// 工具函数
pub mod utils;
pub use utils::*;

// 测试模块
#[cfg(test)]
mod tests;
