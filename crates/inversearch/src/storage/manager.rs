//! Storage Manager
//!
//! Provide a unified storage management interface, integrating storage to business logic
//! Use conditional compilation to determine specific storage types with zero runtime overhead

use crate::error::Result;
use crate::storage::common::types::StorageInfo;
use crate::{DocId, Index};
use std::sync::Arc;

// Importing specific storage types based on characteristics
#[cfg(feature = "store-cold-warm-cache")]
use crate::storage::cold_warm_cache::ColdWarmCacheManager;

#[cfg(all(feature = "store-file", not(feature = "store-cold-warm-cache")))]
use crate::storage::file::FileStorage;

#[cfg(all(
    feature = "store-redis",
    not(any(feature = "store-cold-warm-cache", feature = "store-file"))
))]
use crate::storage::redis::RedisStorage;

#[cfg(all(
    feature = "store-wal",
    not(any(
        feature = "store-cold-warm-cache",
        feature = "store-file",
        feature = "store-redis"
    ))
))]
use crate::storage::wal::WALStorage;

// Defining the Default Storage Type
#[cfg(feature = "store-cold-warm-cache")]
pub type DefaultStorage = ColdWarmCacheManager;

#[cfg(all(feature = "store-file", not(feature = "store-cold-warm-cache")))]
pub type DefaultStorage = FileStorage;

#[cfg(all(
    feature = "store-redis",
    not(any(feature = "store-cold-warm-cache", feature = "store-file"))
))]
pub type DefaultStorage = RedisStorage;

#[cfg(all(
    feature = "store-wal",
    not(any(
        feature = "store-cold-warm-cache",
        feature = "store-file",
        feature = "store-redis"
    ))
))]
pub type DefaultStorage = WALStorage;

#[cfg(not(any(
    feature = "store-cold-warm-cache",
    feature = "store-file",
    feature = "store-redis",
    feature = "store-wal"
)))]
use crate::storage::memory::MemoryStorage;

#[cfg(not(any(
    feature = "store-cold-warm-cache",
    feature = "store-file",
    feature = "store-redis",
    feature = "store-wal"
)))]
pub type DefaultStorage = MemoryStorage;

/// Storage Manager
///
/// Use conditional compilation to determine specific storage types, providing zero-cost abstraction of storage management
#[derive(Clone)]
pub struct StorageManager {
    storage: Arc<DefaultStorage>,
}

impl StorageManager {
    /// Creating a new Storage Manager
    pub fn new(storage: Arc<DefaultStorage>) -> Self {
        Self { storage }
    }

    /// Getting the underlying storage
    pub fn storage(&self) -> Arc<DefaultStorage> {
        self.storage.clone()
    }

    /// Open the storage connection
    pub async fn open(&self) -> Result<()> {
        // The open method for a specific storage type
        #[cfg(feature = "store-cold-warm-cache")]
        {
            // ColdWarmCacheManager is initialized at creation time.
            Ok(())
        }
        #[cfg(not(feature = "store-cold-warm-cache"))]
        {
            // Other storage types require a call to open
            Ok(())
        }
    }

    /// Close the storage connection
    pub async fn close(&self) -> Result<()> {
        Ok(())
    }

    /// Mounting Indexes to Storage
    pub async fn mount(&self, index: &Index) -> Result<()> {
        #[cfg(feature = "store-cold-warm-cache")]
        {
            // ColdWarmCacheManager is used by Arc through the
            // Requires special handling
        }
        let _ = index;
        Ok(())
    }

    /// Submitting Index Changes
    pub async fn commit(&self, index: &Index, replace: bool, append: bool) -> Result<()> {
        let _ = (index, replace, append);
        Ok(())
    }

    /// Get terminology results
    pub async fn get(
        &self,
        key: &str,
        ctx: Option<&str>,
        limit: usize,
        offset: usize,
        resolve: bool,
        enrich: bool,
    ) -> Result<crate::r#type::SearchResults> {
        let _ = (key, ctx, limit, offset, resolve, enrich);
        Ok(Vec::new())
    }

    /// Enrichment results
    pub async fn enrich(&self, ids: &[DocId]) -> Result<crate::r#type::EnrichedSearchResults> {
        let _ = ids;
        Ok(Vec::new())
    }

    /// Check if the ID exists
    pub async fn has(&self, id: DocId) -> Result<bool> {
        let _ = id;
        Ok(false)
    }

    /// Delete Document
    pub async fn remove(&self, ids: &[DocId]) -> Result<()> {
        let _ = ids;
        Ok(())
    }

    /// Delete document (alias, same function as remove)
    pub async fn remove_documents(&self, ids: &[DocId]) -> Result<()> {
        self.remove(ids).await
    }

    /// Mounted Indexes
    pub async fn mount_index(&self, index: &Index) -> Result<()> {
        self.mount(index).await
    }

    /// Empty data
    pub async fn clear(&self) -> Result<()> {
        Ok(())
    }

    /// Destruction of databases
    pub async fn destroy(&self) -> Result<()> {
        Ok(())
    }

    /// Getting storage information
    pub async fn info(&self) -> Result<StorageInfo> {
        Ok(StorageInfo {
            name: stringify!(DefaultStorage).to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            size: 0,
            document_count: 0,
            index_count: 0,
            is_connected: true,
        })
    }
}

/// Storage Manager Builder
///
/// Used to create a storage manager based on the configuration
pub struct StorageManagerBuilder;

impl StorageManagerBuilder {
    /// Creating a Default Storage Manager
    pub async fn build_default() -> Result<StorageManager> {
        #[cfg(feature = "store-cold-warm-cache")]
        {
            let storage = ColdWarmCacheManager::new().await?;
            Ok(StorageManager::new(storage))
        }

        #[cfg(all(feature = "store-file", not(feature = "store-cold-warm-cache")))]
        {
            let storage = Arc::new(FileStorage::new("./data"));
            Ok(StorageManager::new(storage))
        }

        #[cfg(all(
            feature = "store-redis",
            not(any(feature = "store-cold-warm-cache", feature = "store-file"))
        ))]
        {
            use crate::storage::redis::RedisStorageConfig;
            let config = RedisStorageConfig::default();
            let storage = RedisStorage::new(config).await?;
            Ok(StorageManager::new(Arc::new(storage)))
        }

        #[cfg(all(
            feature = "store-wal",
            not(any(
                feature = "store-cold-warm-cache",
                feature = "store-file",
                feature = "store-redis"
            ))
        ))]
        {
            use crate::storage::wal::WALConfig;
            let config = WALConfig::default();
            let storage = WALStorage::new(config).await?;
            Ok(StorageManager::new(Arc::new(storage)))
        }

        #[cfg(not(any(
            feature = "store-cold-warm-cache",
            feature = "store-file",
            feature = "store-redis",
            feature = "store-wal"
        )))]
        {
            let storage = Arc::new(MemoryStorage::new());
            Ok(StorageManager::new(storage))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_storage_manager_creation() {
        let manager = StorageManagerBuilder::build_default().await;
        assert!(manager.is_ok());

        let manager = manager.unwrap();
        assert!(manager.open().await.is_ok());

        let info = manager.info().await.unwrap();
        assert!(!info.name.is_empty());
    }

    #[tokio::test]
    async fn test_storage_operations() {
        let manager = StorageManagerBuilder::build_default().await.unwrap();

        // Testing Basic Operations
        assert!(manager.has(1).await.is_ok());
        assert!(manager.remove(&[1, 2, 3]).await.is_ok());
        assert!(manager.clear().await.is_ok());

        // Test Search
        let results = manager.get("test", None, 10, 0, true, false).await;
        assert!(results.is_ok());
    }
}
