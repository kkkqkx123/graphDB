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

// 工具函数
pub mod utils;
pub use utils::*;

// 测试模块
#[cfg(test)]
mod tests;
