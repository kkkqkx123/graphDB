//! Storage Tier Indexing Data Management Module
//!
//! Provide index data management functions, including index data update, delete and query
//! Note: Index metadata management is the responsibility of the metadata::IndexMetadataManager.
//!
//! ## Index Classification
//!
//! ### Primary Indexes (CSR-Aware)
//!
//! Primary indexes are tightly coupled with CSR storage structure:
//! - `edge_id_index`: Maps edge_id -> (src, dst, prop_offset)
//! - `degree_index`: Maps vertex_id -> (out_degree, in_degree)
//!
//! Characteristics:
//! - Native ID types (u64) for maximum performance
//! - No MVCC overhead (always consistent with CSR)
//! - Automatically maintained during DML operations
//!
//! ### Secondary Indexes (Property Indexes)
//!
//! Secondary indexes support complex property-based queries:
//! - `vertex_index_manager`: Index on vertex properties
//! - `edge_index_manager`: Index on edge properties
//!
//! Characteristics:
//! - Support MVCC for snapshot isolation
//! - BTreeMap-based for range queries
//! - Support tombstone GC for deleted entries
//!
//! ## Module Structure
//!
//! - `index_types`: Index classification traits and types
//! - `vertex_index_manager`: BTreeMap-based vertex index management
//! - `edge_index_manager`: BTreeMap-based edge index management
//! - `index_data_manager`: `IndexDataManager` trait and `InMemoryIndexDataManager` implementation
//! - `index_updater`: Automatic index maintenance during DML operations
//! - `index_key_codec`: Index key encoding/decoding utilities
//! - `index_compression`: Index compression algorithms
//! - `index_gc_manager`: Background GC for tombstone cleanup
//! - `edge_id_index`: CSR-aware edge ID index for fast edge lookup
//! - `degree_index`: CSR-aware degree index for fast degree queries

pub mod degree_index;
pub mod edge_id_index;
pub mod edge_index_manager;
pub mod index_compression;
pub mod index_data_manager;
pub mod index_gc_manager;
pub mod index_key_codec;
pub mod index_types;
pub mod index_updater;
pub mod vertex_index_manager;

pub use crate::core::types::{Index, IndexStatus, IndexType};
pub use degree_index::*;
pub use edge_id_index::*;
pub use edge_index_manager::*;
pub use index_compression::*;
pub use index_data_manager::*;
pub use index_gc_manager::*;
pub use index_key_codec::*;
pub use index_types::*;
pub use index_updater::*;
pub use vertex_index_manager::*;
