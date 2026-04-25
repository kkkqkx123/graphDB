//! Storage Interface Module
//!
//! Provide abstract interfaces and implementations of BM25 storage
//!
//! ## Module structure
//!
//! ```text
//! storage/
//! ├── common/
//! │ ├── types.rs # Shared type definitions
//! │ └── trait.rs # Store interface trait
//! ├── factory.rs # store factory
//! ├── redis.rs # Redis storage implementation (optional)
//! └── tantivy.rs # Tantivy local storage (default)
//! ```
//!
//! ## Conditional compilation features
//!
//! - `storage-redis`: Redis 存储
//! - `storage-tantivy`: Tantivy 本地文件存储（默认启用）

pub mod common;

#[cfg(feature = "storage-tantivy")]
pub mod tantivy;

#[cfg(feature = "storage-redis")]
pub mod redis;

pub mod factory;
pub mod manager;

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
