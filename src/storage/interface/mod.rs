//! Storage Interface Module
//!
//! Defines the core storage interface (StorageClient trait) and storage-level types.
//! This module acts as the public API for storage operations.

pub mod storage_client;
pub mod types;

pub use storage_client::{StorageClient, StorageStats};
pub use types::{
    ColumnDef, EncodingFormat, FieldDef, GeoShape, InsertEdgeInfo, InsertVertexInfo,
    UpdateInfo, UpdateOp, UpdateTarget,
};
