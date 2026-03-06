//! 预编译语句模块
//!
//! 提供高性能的预编译查询支持，包括查询计划缓存、参数绑定、批量执行等功能
//!
//! # 模块结构
//!
//! - `config` - 配置和数据结构
//! - `parameter_extractor` - 参数提取功能
//! - `statement` - 预编译语句核心功能
//! - `builder` - 构建器模式

pub mod builder;
pub mod config;
pub mod parameter_extractor;
pub mod statement;

#[cfg(test)]
mod statement_tests;

pub use builder::PreparedStatementBuilder;
pub use config::{ExecutionStats, ParameterInfo, StatementConfig};
pub use statement::PreparedStatement;
