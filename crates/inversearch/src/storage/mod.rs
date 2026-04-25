//! Storage Interface Module
//!
//! Provide abstract interfaces and implementations for persistent storage
//!
//! ## Module structure
//!
//! ```text
//! storage/
//! ├── common/ # public components (types, traits, utility functions)
//! │   ├── mod.rs
//! │ ├── base.rs # Store the base class
//! │ ├── config.rs # store configuration
//! │ ├── compression.rs # 压缩/解压缩
//! │ ├── error.rs # Store the type of error
//! │ ├── io.rs # File I/O operations
//! │ ├── metrics.rs # Performance metrics
//! │ ├── trait.rs # Storage interface trait
//! │ ├── types.rs # Shared type definitions
//! │ └── utils.rs # utility functions
//! ├── file.rs # File storage implementation
//! ├── redis.rs # Redis storage implementation
//! ├── wal.rs # WAL prewriting logs
//! ├── memory.rs # Memory storage implementation (for testing)
//! ├── factory.rs # store factory
//! └── cold_warm_cache/ # Hot and cold cache storage implementation (default)
//!     ├── mod.rs
//!     ├── config.rs
//!     ├── manager.rs
//!     ├── policy.rs
//!     ├── stats.rs
//!     └── background.rs
//! ```
//!
//! ## Conditional compilation features
//!
//! - `store-cold-warm-cache`: 冷热缓存存储（默认启用）
//! - `store-file`: 文件存储
//! - `store-redis`: Redis 存储
//! - `store-wal`: WAL 预写日志存储

// Public Component - all storage is shared
pub mod common;

// Stored Implementations of Conditional Compilation
#[cfg(feature = "store-file")]
pub mod file;

#[cfg(feature = "store-redis")]
pub mod redis;

#[cfg(feature = "store-wal")]
pub mod wal;

// Hot and cold cache storage implementation (default)
pub mod cold_warm_cache;

// Memory storage for testing (for testing only)
pub mod memory;

// storage plant
pub mod factory;

// Storage Manager
pub mod manager;

// Persistence Manager
pub mod persistence;

// Re-export commonly used types and traits for ease of use
pub use common::{
    compression::{compress_data, decompress_data},
    config::{StorageConfig, StorageType},
    error::{StorageError, StorageResult},
    io::{atomic_write, get_file_size, load_from_file, remove_file_safe, save_to_file},
    metrics::{MetricsCollector, OperationTimer},
    FileStorageData, StorageInfo, StorageInterface, StorageMetrics,
};

// Re-export factory
pub use factory::StorageFactory;

// Re-export Storage Manager
pub use manager::{DefaultStorage, StorageManager, StorageManagerBuilder};

// Re-export Persistence Manager
pub use persistence::{BackupInfo, IndexMetadata, IndexSnapshot, PersistenceManager};
