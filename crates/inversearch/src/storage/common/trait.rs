//! Storage Interface Definition
//!
//! Define the core trait and abstract interfaces of the storage module

use crate::error::Result;
use crate::r#type::{DocId, EnrichedSearchResults, SearchResults};
use crate::storage::common::types::StorageInfo;
use crate::Index;

/// StorageInterface - JavaScript-like version of StorageInterface
///
/// Note: all methods use &self, implementations need to use internal mutability (e.g. RwLock/Mutex)
#[async_trait::async_trait]
pub trait StorageInterface: Send + Sync {
    /// Mounting Indexes to Storage
    async fn mount(&self, index: &Index) -> Result<()>;

    /// Open the connection
    async fn open(&self) -> Result<()>;

    /// Close connection
    async fn close(&self) -> Result<()>;

    /// Destruction of databases
    async fn destroy(&self) -> Result<()>;

    /// Submitting Index Changes
    async fn commit(&self, index: &Index, replace: bool, append: bool) -> Result<()>;

    /// Get terminology results
    async fn get(
        &self,
        key: &str,
        ctx: Option<&str>,
        limit: usize,
        offset: usize,
        resolve: bool,
        enrich: bool,
    ) -> Result<SearchResults>;

    /// Enrichment results
    async fn enrich(&self, ids: &[DocId]) -> Result<EnrichedSearchResults>;

    /// Check if the ID exists
    async fn has(&self, id: DocId) -> Result<bool>;

    /// Delete ID
    async fn remove(&self, ids: &[DocId]) -> Result<()>;

    /// Empty data
    async fn clear(&self) -> Result<()>;

    /// Getting storage information
    async fn info(&self) -> Result<StorageInfo>;
}
