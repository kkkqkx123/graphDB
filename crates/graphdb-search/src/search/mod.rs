pub mod config;
pub mod engine;
pub mod error;
#[cfg(feature = "fulltext-search")]
pub mod factory;
pub mod manager;
pub mod metadata;
pub mod metrics;
pub mod result;
#[cfg(feature = "fulltext-search")]
pub mod tantivy_index;
pub mod warmup;

#[cfg(test)]
mod isolation_test;

pub use crate::config::common::fulltext::TantivyConfig;
pub use config::{FulltextConfig, SyncConfig, SyncFailurePolicy};
pub use engine::{EngineType, SearchEngine};
pub use error::{Result, SearchError};
#[cfg(feature = "fulltext-search")]
pub use factory::SearchEngineFactory;
pub use manager::FulltextIndexManager;
pub use metadata::{IndexKey, IndexMetadata, IndexStatus};
pub use metrics::MetricsSearchEngine;
pub use result::{
    FulltextSearchEntry, FulltextSearchResult, HighlightResult, IndexStats, SearchResult,
    SearchStats,
};
#[cfg(feature = "fulltext-search")]
pub use tantivy_index::TantivySearchEngine;
pub use warmup::IndexWarmer;
