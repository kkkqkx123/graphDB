//! 数据变更执行器
//!
//! 提供数据的插入、删除、更新功能。

pub mod insert;
pub mod delete;
pub mod update;

pub use insert::{InsertVertexExecutor, InsertEdgeExecutor};
pub use delete::DeleteExecutor;
pub use update::UpdateExecutor;
