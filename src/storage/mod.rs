pub mod api;
pub mod cache;
pub mod container;
pub mod edge;
pub mod entity;
pub mod extend;
pub mod graph_storage;
pub mod index;
pub mod iterator;
pub mod memory;
pub mod metadata;
pub mod monitoring;
pub mod operations;
pub mod page;
pub mod persistence;
pub mod property_graph;
pub mod shared_state;
pub mod vertex;

#[cfg(test)]
pub mod test_mock;

pub use api::{
    ColumnDef, EncodingFormat, FieldDef, FieldType, GeoShape, InsertEdgeInfo, InsertVertexInfo,
    StorageClient, StorageStats, UpdateInfo, UpdateOp, UpdateTarget,
};

pub use cache::{RecordCache, RecordCacheConfig, RecordCacheStats, SharedRecordCache};

pub use entity::{EdgeStorage, SyncStorage, UserStorage, VertexStorage};

pub use extend::FulltextStorage;

pub use index::*;
pub use iterator::*;
pub use memory::{
    AllocationResult, HugePageAllocator, HugePageBuffer, HugePageConfig, HugePageError,
    MemoryConfig, MemoryConfigBuilder, MemoryLevel, MemoryStats, MemoryTracker, NullBitmap,
    DEFAULT_HUGE_PAGE_SIZE,
};
pub use metadata::*;
pub use operations::*;
pub use shared_state::{StorageInner, StorageSharedState};

pub use crate::core::StorageError;
pub use crate::core::StorageResult;

pub use container::{
    AnonMmap, ArenaAllocator, ArenaPool, ContainerConfig, ContainerError, ContainerResult,
    ContainerStats, FileHeader, FileSharedMmap, IDataContainer, MmapContainer, ThreadLocalArena,
};

pub use vertex::{
    Column, ColumnStore, IdIndexer, LabelId, PropertyDef as VertexPropertyDef, Timestamp, VertexId,
    VertexRecord, VertexSchema, VertexTable, VertexTimestamp,
};

pub use edge::{
    Csr, EdgeDirection, EdgeId, EdgeRecord, EdgeSchema, EdgeStrategy, EdgeTable, ImmutableNbr,
    MutableCsr, Nbr, PropertyDef as EdgePropertyDef, PropertyTable,
};

pub use property_graph::{PropertyGraph, PropertyGraphConfig};

pub use persistence::{
    CompressionType, Compressor, DirtyPageTracker, FlushConfig, FlushManager, FlushTask, PageId,
};

pub use page::{
    FlatCsr, FlatCsrEdgeIterator, FlatCsrIterator, Page, PageHeader, PageManager, PageManagerStats,
    PageType, StoragePageId, DELETED_TIMESTAMP, EDGE_RECORD_SIZE, INVALID_TIMESTAMP,
    PAGE_DATA_SIZE, PAGE_HEADER_SIZE, PAGE_SIZE, VERTEX_RECORD_SIZE,
};

pub use graph_storage::GraphStorage;

#[cfg(test)]
pub use test_mock::*;
