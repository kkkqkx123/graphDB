pub mod core;
pub mod embedded;

pub use embedded::{
    EmbeddedBatch, EmbeddedBatchOperation, EmbeddedBatchResult, EmbeddedIndex,
    EmbeddedIndexBuilder, EmbeddedIndexStats, EmbeddedSearchResult,
};
