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
pub use schema::{SchemaInfo, SchemaManager, SchemaProvider, ValidationMode};
pub use types::{ColsDef, Column, SpaceInfo, Variable};
