//! Storage Module
//!
//! Core storage layer for the graph database, providing:
//! - Container: Memory-mapped storage containers
//! - CSR: Compressed Sparse Row graph structures
//! - Vertex/Edge: Vertex and edge storage
//! - Index: Primary and secondary indexes
//! - Cache: Record caching
//! - Engine: Storage engine core

pub mod cache;
pub mod compression;
pub mod container;
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

#[cfg(test)]
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

pub use extend::FulltextStorage;

pub use crate::core::StorageError;
pub use crate::core::StorageResult;

pub use compression::CompressionType;

pub use container::{
    open_container, open_container_from_file, ContainerConfig, ContainerError, ContainerResult,
    ContainerStats, FileHeader, IDataContainer, PersistentContainer, StorageBackend,
    VolatileContainer, DEFAULT_HUGE_PAGE_SIZE,
};

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

#[cfg(test)]
pub use test_mock::*;
