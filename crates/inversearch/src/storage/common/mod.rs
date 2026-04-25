//! Storage Module Public Components
//!
//! Provides types, utility functions, and traits shared by all storage implementations.

pub mod base;
pub mod compression;
pub mod config;
pub mod error;
pub mod io;
pub mod metrics;
pub mod r#trait;
pub mod types;
pub mod utils;

// Re-export common types
pub use base::StorageBase;
pub use compression::{compress_data, decompress_data};
pub use config::{StorageConfig, StorageType};
pub use error::{StorageError, StorageResult};
pub use io::{atomic_write, get_file_size, load_from_file, remove_file_safe, save_to_file};
pub use metrics::{MetricsCollector, OperationTimer, StorageMetrics};
pub use r#trait::StorageInterface;
pub use types::{FileStorageData, StorageInfo};
pub use utils::apply_limit_offset;
