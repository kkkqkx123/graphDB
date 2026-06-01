//! Storage Tier Indexing Data Management Module
//!
//! Provide index data management functions, including index data update, delete and query
//! Note: Index metadata management is the responsibility of the metadata::IndexMetadataManager.
//!
//! ## Index Classification
//!
//! ### Property Indexes
//!
//! Secondary indexes support complex property-based queries:
//! - `vertex_index_manager`: Index on vertex properties
//! - `edge_index_manager`: Index on edge properties
//!
//! Characteristics:
//! - Support MVCC for snapshot isolation
//! - BTreeMap-based for range queries
//! - Support tombstone GC for deleted entries
//! - Optional key compression for memory efficiency
//!
//! ## Module Structure
//!
//! - `index_types`: Index classification traits and types
//! - `secondary`: Secondary indexes (Property-based)
//!   - `vertex_index_manager`: BTreeMap-based vertex index management
//!   - `edge_index_manager`: BTreeMap-based edge index management
//!   - `index_data_manager`: `IndexDataManager` trait and `IndexDataManagerImpl` implementation
//!   - `index_updater`: Automatic index maintenance during DML operations
//!   - `key_codec`: Index key encoding/decoding and compression utilities
//!   - `index_gc_manager`: Background GC for tombstone cleanup

pub mod index_types;
pub mod secondary;
