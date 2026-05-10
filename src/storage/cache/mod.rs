//! Cache Module
//!
//! Provides caching mechanisms for the storage engine.
//!
//! ## Design Note: Why No Edge Cache?
//!
//! Edge data is NOT cached separately because:
//!
//! 1. **CSR is already read-optimized**: The CSR (Compressed Sparse Row) structure
//!    provides O(1) edge list access with contiguous memory layout, which is
//!    CPU cache-friendly. Adding another cache layer would not improve performance.
//!
//! 2. **High memory cost**: Edge data volume is typically much larger than vertex
//!    data. Caching edges would consume significant memory with limited benefit.
//!
//! 3. **Frequent updates**: Edges are updated more frequently than vertices,
//!    making cache invalidation complex and potentially causing consistency issues.
//!
//! 4. **Property access is O(1)**: Edge properties are stored in PropertyTable
//!    with direct offset access, which is already optimal.
//!
//! The cache focuses on:
//! - **Vertex Cache**: Caches vertex records for fast point lookups
//! - **ID Index Cache**: Caches external_id -> internal_id mappings

mod types;
mod stats;
mod config;
mod batch;
mod predictor;
mod record_cache;

#[cfg(test)]
mod record_cache_test;

pub use types::*;
pub use stats::{CacheTypeStats, CacheTypeStatsSnapshot, RecordCacheStats};
pub use config::*;
pub use batch::*;
pub use predictor::*;
pub use record_cache::{RecordCache, SharedRecordCache};
