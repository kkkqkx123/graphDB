//! 存储层索引管理模块
//!
//! 提供索引管理功能，包括索引的创建、删除、查询和维护
//! 参考 NebulaGraph 的索引架构设计

pub mod index_manager;
pub mod memory_index_manager;
pub mod redb_persistence;
pub mod redb_index_manager;

pub use index_manager::*;
pub use memory_index_manager::*;
pub use redb_persistence::*;
pub use redb_index_manager::*;

pub use crate::index::{Index, IndexStatus, IndexType, IndexInfo, IndexOptimization};
