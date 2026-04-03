pub mod adapters;
pub mod config;
pub mod engine;
pub mod error;
pub mod factory;
pub mod manager;
pub mod metadata;
pub mod result;

pub use config::{FulltextConfig, SyncConfig, SyncMode, Bm25Config};
pub use engine::{EngineType, SearchEngine};
pub use error::{Result, SearchError};
pub use factory::SearchEngineFactory;
pub use manager::FulltextIndexManager;
pub use metadata::{IndexMetadata, IndexKey, IndexStatus};
pub use result::{IndexStats, SearchResult};
