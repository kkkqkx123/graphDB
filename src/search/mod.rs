pub mod adapters;
pub mod config;
pub mod engine;
pub mod error;
pub mod factory;
pub mod index_cache;
pub mod manager;
pub mod metadata;
pub mod metrics;
pub mod result;
pub mod warmup;

#[cfg(test)]
mod isolation_test;

pub use config::{FulltextConfig, SyncConfig, SyncFailurePolicy};
pub use engine::{EngineType, SearchEngine};
pub use error::{Result, SearchError};
pub use factory::SearchEngineFactory;
pub use index_cache::IndexCache;
pub use manager::FulltextIndexManager;
pub use metadata::{IndexKey, IndexMetadata, IndexStatus};
pub use metrics::FulltextMetrics;
pub use result::{
    FulltextSearchEntry, FulltextSearchResult, HighlightResult, IndexStats, SearchResult,
    SearchStats,
};
pub use warmup::IndexWarmer;
