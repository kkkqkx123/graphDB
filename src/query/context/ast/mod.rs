//! AST相关上下文模块
//!
//! 本模块包含与抽象语法树相关的上下文结构。
//! 核心是 AstContext，作为Parser和Planner之间的桥梁。

pub mod base;
pub mod common;
pub mod query_types;

// 重新导出所有公共类型
pub use base::{AstContext, QueryType, AstContextTrait, SpaceInfo};
pub use common::{Column, ColsDef, Variable, VariableInfo, *};
pub use query_types::*;
