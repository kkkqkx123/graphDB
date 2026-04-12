//! Storage API Module
//!
//! Defines the core storage interface (StorageClient trait) and storage-level types.

pub mod storage_client;
pub mod types;

pub use storage_client::{StorageClient, StorageStats};
pub use types::{
    ColumnDef, EncodingFormat, FieldDef, FieldType, GeoShape, InsertEdgeInfo, InsertVertexInfo,
    UpdateInfo, UpdateOp, UpdateTarget,
};
