//! 存储层索引数据管理模块
//!
//! 提供索引数据管理功能，包括索引数据的更新、删除和查询
//! 注意：索引元数据管理由 metadata::IndexMetadataManager 负责

pub mod index_data_manager;

pub use index_data_manager::*;
pub use crate::index::{Index, IndexStatus, IndexType, IndexStats, IndexOptimization};
