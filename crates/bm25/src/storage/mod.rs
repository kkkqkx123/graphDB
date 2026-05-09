//! Storage Interface Module
//!
//! Provide abstract interfaces and implementations of BM25 storage
//!
//! ## Module structure
//!
//! ```text
//! storage/
//! ├── common/
//! │   ├── types.rs    # Shared type definitions
//! │   └── trait.rs    # Storage interface trait
//! ├── factory.rs      # Storage factory
//! ├── storage_enum.rs # Storage enum for static dispatch
//! ├── redis.rs        # Redis storage implementation (optional)
//! └── tantivy.rs      # Tantivy local storage (default)
//! ```
//!
//! ## Conditional compilation features
//!
//! - `storage-redis`: Redis storage
//! - `storage-tantivy`: Tantivy local file storage (default enabled)

pub mod common;

#[cfg(feature = "storage-tantivy")]
pub mod tantivy;

#[cfg(feature = "storage-redis")]
pub mod redis;

pub mod factory;
pub mod manager;

#[cfg(any(feature = "storage-tantivy", feature = "storage-redis"))]
pub mod storage_enum;

pub use common::{
    r#trait::StorageInterface,
    types::{Bm25Stats, StorageInfo},
};

#[cfg(feature = "storage-tantivy")]
pub use tantivy::TantivyStorage;

#[cfg(feature = "storage-redis")]
pub use redis::RedisStorage;

pub use factory::StorageFactory;
pub use manager::{DefaultStorage, MutableStorageManager, StorageManager, StorageManagerBuilder};

#[cfg(any(feature = "storage-tantivy", feature = "storage-redis"))]
pub use storage_enum::StorageEnum;
