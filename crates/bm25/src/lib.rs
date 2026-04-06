pub mod config;
pub mod error;
pub mod api;

#[cfg(any(feature = "storage-tantivy", feature = "storage-redis"))]
pub mod storage;

// Re-export core API (always available)
pub use api::core;

// Re-export core types for backward compatibility
pub use api::core::{
    IndexManager, IndexManagerConfig, IndexSchema, LogMergePolicyConfig, MergePolicyType,
    ReloadPolicyConfig, SearchOptions, SearchResult as CoreSearchResult,
};

// Re-export embedded API (with embedded feature)
#[cfg(feature = "embedded")]
pub use api::embedded;

#[cfg(feature = "embedded")]
pub use api::embedded::{Bm25Index, SearchResult};

// Re-export server API (with service feature)
#[cfg(feature = "service")]
pub use api::server;

#[cfg(feature = "service")]
pub use api::server::{
    Config as ServiceConfig, IndexConfig as ServiceIndexConfig, ServerConfig,
};

#[cfg(feature = "service")]
pub use api::server::init_logging;

#[cfg(feature = "service")]
pub use api::server::{run_server, BM25Service};

// Re-export error types
pub use error::{Bm25Error, Result};

// Re-export config types
pub use config::{Bm25Config, FieldWeights, SearchConfig};
pub use config::IndexManagerConfigBuilder;
pub use config::{StorageConfig, StorageType, StorageConfigBuilder, TantivyStorageConfig, RedisStorageConfig};

// Re-export storage types
#[cfg(any(feature = "storage-tantivy", feature = "storage-redis"))]
pub use storage::{StorageInterface, StorageInfo, Bm25Stats};

#[cfg(feature = "storage-tantivy")]
pub use storage::TantivyStorage;

#[cfg(feature = "storage-redis")]
pub use storage::RedisStorage;

#[cfg(any(feature = "storage-tantivy", feature = "storage-redis"))]
pub use storage::StorageFactory;

#[cfg(any(feature = "storage-tantivy", feature = "storage-redis"))]
pub use storage::{DefaultStorage, StorageManager, MutableStorageManager, StorageManagerBuilder};
