//! Storage Module
//!
//! Core storage layer for the graph database, providing:
//! - Columnar storage for vertices and edges (CSR)
//! - Index: Primary and secondary indexes
//! - Cache: Record caching
//! - Engine: Storage engine core

pub(crate) mod cache;
pub(crate) mod compression;
pub(crate) mod edge;
pub(crate) mod engine;
pub(crate) mod index;

pub(crate) mod metrics;
mod storage_client;
pub(crate) mod storage_types;
pub(crate) mod utils;
pub(crate) mod vertex;

mod test_mock;

pub use engine::graph_storage::GraphStorage;
pub use engine::persistence_coordinator::{CheckpointStats, PersistenceConfig};
pub use engine::sync_wrapper::SyncWrapper;
pub use index::secondary::IndexGcConfig;
pub use storage_client::{
    StorageAdmin, StorageAuthOps, StorageClient, StorageReader, StorageSchemaOps, StorageStats,
    StorageWriter,
};

pub use crate::core::metadata::SchemaManager;
pub use crate::core::StorageError;
pub use crate::core::StorageResult;

pub use test_mock::MockStorage;
