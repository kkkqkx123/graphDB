//! 边类型管理执行器
//!
//! 提供边类型的创建、修改、描述、删除和列出功能。

pub mod create_edge;
pub mod alter_edge;
pub mod desc_edge;
pub mod drop_edge;
pub mod show_edges;

#[cfg(test)]
mod tests;

pub use create_edge::CreateEdgeExecutor;
pub use alter_edge::AlterEdgeExecutor;
pub use desc_edge::DescEdgeExecutor;
pub use drop_edge::DropEdgeExecutor;
pub use show_edges::ShowEdgesExecutor;
