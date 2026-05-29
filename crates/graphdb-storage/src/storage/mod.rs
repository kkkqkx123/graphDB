//! Storage Module
//!
//! Core storage layer for the graph database, providing:
//! - Columnar storage for vertices and edges (CSR)
//! - Index: Primary and secondary indexes
//! - Cache: Record caching
//! - Engine: Storage engine core

pub mod cache;
pub mod compression;
pub mod edge;
pub mod engine;
pub mod extend;
pub mod index;

pub mod metadata;
pub mod metrics;
pub mod storage_client;
pub mod storage_types;
pub mod utils;
pub mod vertex;

pub mod test_mock;

pub use crate::core::types::{
    InsertEdgeInfo, InsertVertexInfo, UpdateInfo, UpdateOp, UpdateTarget,
};
pub use engine::graph_storage::GraphStorage;
pub use engine::sync_wrapper::SyncWrapper;
pub use storage_client::{
    StorageAdmin, StorageAuthOps, StorageClient, StorageReader, StorageSchemaOps, StorageStats,
    StorageWriter,
};
pub use storage_types::{
    ColumnDef, EdgeOffset, EncodingFormat, FieldDef, GeoShape, PropertyId, StoragePropertyDef,
};

pub use cache::{
    RecordCache, RecordCacheConfig, RecordCacheStats, SharedRecordCache,
};

pub use crate::core::StorageError;
pub use crate::core::StorageResult;

pub use compression::CompressionType;

pub use engine::config::FlushConfig;

pub use vertex::{
    Column, ColumnStore, IdIndexer, LabelId, Timestamp, VertexId, VertexRecord, VertexSchema,
    VertexTable, VertexTimestamp,
};

pub use edge::{
    Csr, EdgeDirection, EdgeId, EdgeRecord, EdgeSchema, EdgeStrategy, EdgeTable, ImmutableNbr,
    MutableCsr, Nbr, PropertyTable,
};

pub use crate::core::types::{
    EdgeDeletionContext, EdgeDeletionContextParams, EdgeIdentifier, EdgeKey, EdgeLocation,
    EdgeOperationContext, EdgePropertyUpdateContext, VertexIdentifier,
};

pub use engine::{
    batch_import_edges, batch_import_vertices, BatchImportStats, EdgeBatchReader, EdgeBatchWriter,
    PropertyGraph, PropertyGraphConfig, VertexBatchReader, VertexBatchWriter, DEFAULT_BATCH_SIZE,
};

pub use test_mock::MockStorage;
