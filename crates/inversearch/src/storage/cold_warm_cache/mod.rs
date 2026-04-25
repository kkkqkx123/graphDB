//! Hot and Cold Cache Storage Module
//!
//! Provides three-tier caching architecture + WAL for high performance persistent storage
//!
//! ## Architecture
//!
//! ```text
//! ColdWarmCache
//! Hot Cache (L1) - Memory, Hottest Data, LRU Elimination
//Warm Cache (L2) - memory-mapped file, frequently accessed. Warm Cache (L2) - memory mapped file, frequently accessed
//! └── Cold Storage - disk files, cold data, compressed storage
//! ```
//!
//! ## Module structure
//!
//! ```text
//! cold_warm_cache/
//! ├── mod.rs # Main Module
//! ├── config.rs # Configuration structure
//! ├── manager.rs # ColdWarmCacheManager core implementation
//! └── background.rs # Background tasks
//! ```

pub mod background;
pub mod config;
pub mod manager;

pub use config::{ColdWarmCacheConfig, WALConfig};
pub use manager::{CacheStats, ColdWarmCacheManager, IndexData, WALEntry, WALManager};
