//! 数据修改执行器模块
//!
//! 包含所有与数据修改相关的执行器，这些执行器修改存储层的数据

pub mod delete;
pub mod index_ops;
pub mod insert;
pub mod remove;
pub mod tag_ops;
pub mod update;

pub use delete::DeleteExecutor;
pub use index_ops::{CreateIndexExecutor, DropIndexExecutor};
pub use insert::InsertExecutor;
pub use remove::{RemoveExecutor, RemoveItem, RemoveItemType, RemoveResult};
pub use tag_ops::DeleteTagExecutor;
pub use update::{UpdateExecutor, VertexUpdate, EdgeUpdate, UpdateResult};
