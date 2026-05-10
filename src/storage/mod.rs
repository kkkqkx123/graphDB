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
pub mod entity;
pub mod extend;
pub mod index;
pub mod interface;
pub mod iterator;
pub mod lob;
pub mod memory;
pub mod metadata;
pub mod operations;
pub mod page;
pub mod params;
pub mod persistence;
pub mod stats;
pub mod vertex;

#[cfg(test)]
pub mod test_mock;

pub use interface::{
    ColumnDef, EncodingFormat, FieldDef, GeoShape, InsertEdgeInfo, InsertVertexInfo,
    StorageClient, StorageStats, UpdateInfo, UpdateOp, UpdateTarget,
};
pub use engine::graph_storage::GraphStorage;

pub use cache::{RecordCache, RecordCacheConfig, RecordCacheStats, SharedRecordCache};

pub use entity::{EdgeStorage, SyncStorage, UserStorage, VertexStorage};

pub use extend::FulltextStorage;

pub use index::*;
pub use iterator::*;
pub use memory::{MemoryConfig, MemoryConfigBuilder, MemoryStats, MemoryTracker, NullBitmap};
pub use metadata::*;
pub use operations::*;

pub use crate::core::StorageError;
pub use crate::core::StorageResult;

pub use compression::{CompressionType, Compressor};

pub use container::{
    open_container, open_container_from_file, AnonMmap, ArenaAllocator, ArenaPool,
    ContainerConfig, ContainerError, ContainerResult, ContainerStats, FileHeader, FileMmap,
    HugePageMmap, IDataContainer, MemoryLevel, DEFAULT_HUGE_PAGE_SIZE,
};

pub use vertex::{
    Column, ColumnStore, IdIndexer, LabelId, PropertyDef as VertexPropertyDef, Timestamp, VertexId,
    VertexRecord, VertexSchema, VertexTable, VertexTimestamp,
};

pub use edge::{
    Csr, EdgeDirection, EdgeId, EdgeRecord, EdgeSchema, EdgeStrategy, EdgeTable, ImmutableNbr,
    MutableCsr, Nbr, PropertyDef as EdgePropertyDef, PropertyTable,
};

pub use params::{
    EdgeDeletionContext, EdgeIdentifier, EdgeKey, EdgeLocation, EdgeOperationContext,
    EdgePropertyUpdateContext, VertexIdentifier,
};

pub use engine::{
    batch_import_edges, batch_import_vertices, BatchImportStats, EdgeBatchReader, EdgeBatchWriter,
    VertexBatchReader, VertexBatchWriter, DEFAULT_BATCH_SIZE, PropertyGraph, PropertyGraphConfig,
};

pub use lob::{LargeObjectStore, LobId, LobStats, DEFAULT_LOB_THRESHOLD};

pub use stats::{ColumnStatistics, Histogram, HistogramBucket, StatsCollector};

#[cfg(test)]
pub use test_mock::*;
