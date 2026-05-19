//! Cache Module
//!
//! Provides caching mechanisms for the storage engine.
//!
//! ## Cache Types
//!
//! ### Vertex Cache (Default)
//! - Caches vertex records for fast point lookups
//! - Caches external_id -> internal_id mappings

mod batch;
mod config;
mod record_cache;
mod stats;
mod types;

#[cfg(test)]
mod record_cache_test;

pub use batch::*;
pub use config::*;
pub use record_cache::{RecordCache, SharedRecordCache};
pub use stats::{CacheTypeStatsSnapshot, RecordCacheStats};
pub use types::*;
