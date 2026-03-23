//! 存储层索引数据管理模块
//!
//! 提供索引数据管理功能，包括索引数据的更新、删除和查询
//! 注意：索引元数据管理由 metadata::IndexMetadataManager 负责

pub mod edge_index_manager;
pub mod index_data_manager;
pub mod index_key_codec;
pub mod index_updater;
pub mod vertex_index_manager;

pub use crate::core::types::{Index, IndexStatus, IndexType};
pub use edge_index_manager::*;
pub use index_data_manager::*;
pub use index_key_codec::*;
pub use index_updater::*;
pub use vertex_index_manager::*;
