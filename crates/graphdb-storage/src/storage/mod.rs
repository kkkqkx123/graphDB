//! Storage Module
//!
//! Core storage layer for the graph database, providing:
//! - Columnar storage for vertices and edges (CSR)
//! - Index: Primary and secondary indexes
//! - Cache: Record caching
//! - Engine: Storage engine core

pub(crate) mod cache;
pub(crate) mod client;
pub(crate) mod compression;
pub(crate) mod edge;
pub(crate) mod engine;
pub(crate) mod index;

mod metrics;
pub(crate) mod types;
pub(crate) mod utils;
pub(crate) mod vertex;

#[cfg(any(test, feature = "test-support"))]
mod test_mock;

pub use client::{
    StorageAdmin, StorageAuthOps, StorageClient, StorageGcOps, StoragePersistenceOps,
    StorageReader, StorageRecoveryOps, StorageSchemaContextOps, StorageSchemaOps, StorageStats,
    StorageSyncContextOps, StorageTransactionContextOps, StorageWriter,
};
pub use engine::graph_storage::GraphStorage;
pub use engine::persistence_coordinator::{CheckpointStats, SnapshotStats};
pub use engine::sync_wrapper::SyncWrapper;
pub use metrics::MetricsStorage;

pub use crate::core::StorageError;

#[cfg(any(test, feature = "test-support"))]
pub use test_mock::MockStorage;
