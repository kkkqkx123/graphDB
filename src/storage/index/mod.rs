//! Storage Tier Indexing Data Management Module
//!
//! Provide index data management functions, including index data update, delete and query
//! Note: Index metadata management is the responsibility of the metadata::IndexMetadataManager.

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
