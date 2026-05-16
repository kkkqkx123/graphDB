pub mod api;
pub mod config;
pub mod error;
pub mod storage;
pub mod tokenizer;

// Re-export core API
pub use api::core;

// Re-export core types for backward compatibility
pub use api::core::{
    IndexManager, IndexManagerConfig, IndexSchema, LogMergePolicyConfig, MergePolicyType,
    ReloadPolicyConfig, SearchOptions, SearchResult as CoreSearchResult,
};

// Re-export embedded Bm25Index API
pub use api::embedded::{Bm25Index, SearchResult, SearchResultWithHighlights};

// Re-export error types
pub use error::{Bm25Error, Result};

// Re-export config types
pub use config::IndexManagerConfigBuilder;
pub use config::{Bm25Config, FieldWeights, SearchConfig};
pub use config::{StorageConfig, StorageConfigBuilder, StorageType, TantivyStorageConfig};

// Re-export storage types
pub use storage::{Bm25Stats, DefaultStorage, StorageEnum, StorageInfo, StorageInterface};
pub use storage::{MutableStorageManager, StorageManager, StorageManagerBuilder};
pub use storage::{StorageFactory, TantivyStorage};
