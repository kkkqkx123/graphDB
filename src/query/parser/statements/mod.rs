//! 语句解析模块
//!
//! 按功能分类的语句解析器

mod create;
mod delete;
mod go;
mod match_stmt;
mod update;
mod query;
mod traverse;
mod mutate;
mod projection;
mod control_flow;
mod maintain;
mod admin;

mod create_impl;
mod delete_impl;
mod go_impl;

pub use create::*;
pub use delete::*;
pub use go::*;
pub use match_stmt::*;
pub use update::*;
pub use query::*;
pub use traverse::*;
pub use mutate::*;
pub use projection::*;
pub use control_flow::*;
pub use maintain::*;
pub use admin::*;
