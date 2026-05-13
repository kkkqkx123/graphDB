//! Cache Module
//!
//! Provides caching mechanisms for the storage engine.
//!
//! ## Cache Types
//!
//! ### Vertex Cache (Default)
//! - Caches vertex records for fast point lookups
//! - Caches external_id -> internal_id mappings
//!
//! ### Edge Property Cache (Optional)
//! - Optional caching for edge properties in high-load scenarios
//! - Disabled by default, enable when:
//!   - Edge property access frequency exceeds threshold
//!   - Property size is small (< 1KB)
//!   - Edge update frequency is low
//!
//! ## Design Note: Why Edge Cache is Optional
//!
//! Edge data caching is optional because:
//!
//! 1. **CSR is already read-optimized**: The CSR (Compressed Sparse Row) structure
//!    provides O(1) edge list access with contiguous memory layout, which is
//!    CPU cache-friendly.
//!
//! 2. **High memory cost**: Edge data volume is typically much larger than vertex
//!    data. Caching edges would consume significant memory.
//!
//! 3. **Frequent updates**: Edges are updated more frequently than vertices,
//!    making cache invalidation complex.
//!
//! 4. **Property access is O(1)**: Edge properties are stored in PropertyTable
//!    with direct offset access.

mod types;
mod stats;
mod config;
mod batch;
mod predictor;
mod record_cache;
mod edge_property_cache;

#[cfg(test)]
mod record_cache_test;

pub use types::*;
pub use stats::{CacheTypeStats, CacheTypeStatsSnapshot, RecordCacheStats};
pub use config::*;
pub use batch::*;
pub use predictor::*;
pub use record_cache::{RecordCache, SharedRecordCache};
pub use edge_property_cache::{
    CachedEdgeProperty, EdgePropertyCache, EdgePropertyCacheConfig, EdgePropertyCacheStats,
    EdgePropertyKey,
};
