//! Storage Manager
//!
//! Provide a unified storage management interface, Integrate storage into business the business logic
//! Use conditional compilation to determine the specific storage type, zero runtime overhead

use crate::error::Result;
use crate::storage::common::r#trait::StorageInterface;
use crate::storage::common::types::{Bm25Stats, StorageInfo};
use std::sync::Arc;
use tokio::sync::RwLock;

// Import specific storage types based on their-demand
#[cfg(feature = "storage-tantivy")]
use crate::storage::tantivy::TantivyStorage;

#[cfg(all(feature = "storage-redis", not(feature = "storage-tantivy")))]
use crate::storage::redis::RedisStorage;

// Define default storage type - When both features are enabled, prioritize using TantivyStorage
#[cfg(feature = "storage-tantivy")]
pub type DefaultStorage = TantivyStorage;

#[cfg(all(feature = "storage-redis", not(feature = "storage-tantivy")))]
pub type DefaultStorage = RedisStorage;

#[cfg(not(any(feature = "storage-tantivy", feature = "storage-redis")))]
compile_error!(
    "At least one storage backend must be enabled: 'storage-tantivy' or 'storage-redis'"
);

/// Storage Manager (Read-Only Operations)
///
/// Use conditional compilation to determine the specific storage type, providing zero-cost abstraction for storage management
#[derive(Clone)]
pub struct StorageManager {
    storage: Arc<DefaultStorage>,
}

impl StorageManager {
    /// Create a new storage manager
    pub fn new(storage: Arc<DefaultStorage>) -> Self {
        Self { storage }
    }

    /// Get underlying storage
    pub fn storage(&self) -> Arc<DefaultStorage> {
        self.storage.clone()
    }

    /// Get term statistics
    pub async fn get_stats(&self, term: &str) -> Result<Option<Bm25Stats>> {
        self.storage.get_stats(term).await
    }

    /// Get document frequency
    pub async fn get_df(&self, term: &str) -> Result<Option<u64>> {
        self.storage.get_df(term).await
    }

    /// Get term frequency
    pub async fn get_tf(&self, term: &str, doc_id: &str) -> Result<Option<f32>> {
        self.storage.get_tf(term, doc_id).await
    }

    /// Get storage information
    pub async fn info(&self) -> Result<StorageInfo> {
        self.storage.info().await
    }

    /// Health check
    pub async fn health_check(&self) -> Result<bool> {
        self.storage.health_check().await
    }
}

/// Variable storage manager, supports modification operations
///
/// Use conditional compilation to determine the specific storage type
pub struct MutableStorageManager {
    storage: Arc<RwLock<DefaultStorage>>,
}

impl Clone for MutableStorageManager {
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
        }
    }
}

impl MutableStorageManager {
    /// Create a new variable storage manager
    pub fn new(storage: DefaultStorage) -> Self {
        Self {
            storage: Arc::new(RwLock::new(storage)),
        }
    }

    /// Create a variable storage manager from Arc
    pub fn from_arc(storage: Arc<DefaultStorage>) -> Self {
        Self {
            storage: Arc::new(RwLock::new(Arc::try_unwrap(storage).unwrap_or_else(
                |_arc| {
                    // If Arc has multiple references, clone internal data
                    #[cfg(feature = "storage-tantivy")]
                    {
                        // TantivyStorage requires special handling,简化处理 here
                        panic!(
                            "Cannot create MutableStorageManager from shared Arc<TantivyStorage>"
                        )
                    }
                    #[cfg(all(feature = "storage-redis", not(feature = "storage-tantivy")))]
                    {
                        panic!("Cannot create MutableStorageManager from shared Arc<RedisStorage>")
                    }
                },
            ))),
        }
    }

    /// Get storage Arc (for sharing)
    pub fn storage_arc(&self) -> Arc<RwLock<DefaultStorage>> {
        self.storage.clone()
    }

    /// Initialize storage
    pub async fn init(&self) -> Result<()> {
        let mut storage = self.storage.write().await;
        storage.init().await
    }

    /// Submit term statistics
    pub async fn commit_stats(&self, term: &str, tf: f32, df: u64) -> Result<()> {
        let mut storage = self.storage.write().await;
        storage.commit_stats(term, tf, df).await
    }

    /// Batch submission statistics
    pub async fn commit_batch(&self, stats: &Bm25Stats) -> Result<()> {
        let mut storage = self.storage.write().await;
        storage.commit_batch(stats).await
    }

    /// Get term statistics
    pub async fn get_stats(&self, term: &str) -> Result<Option<Bm25Stats>> {
        let storage = self.storage.read().await;
        storage.get_stats(term).await
    }

    /// Get document frequency
    pub async fn get_df(&self, term: &str) -> Result<Option<u64>> {
        let storage = self.storage.read().await;
        storage.get_df(term).await
    }

    /// Get term frequency
    pub async fn get_tf(&self, term: &str, doc_id: &str) -> Result<Option<f32>> {
        let storage = self.storage.read().await;
        storage.get_tf(term, doc_id).await
    }

    /// Clear all data
    pub async fn clear(&self) -> Result<()> {
        let mut storage = self.storage.write().await;
        storage.clear().await
    }

    /// Getting storage information
    pub async fn info(&self) -> Result<StorageInfo> {
        let storage = self.storage.read().await;
        storage.info().await
    }

    /// health checkup
    pub async fn health_check(&self) -> Result<bool> {
        let storage = self.storage.read().await;
        storage.health_check().await
    }

    /// Delete statistics for a specific document
    pub async fn delete_doc_stats(&self, doc_id: &str) -> Result<()> {
        let mut storage = self.storage.write().await;
        storage.delete_doc_stats(doc_id).await
    }

    /// Close Storage
    pub async fn close(&self) -> Result<()> {
        let mut storage = self.storage.write().await;
        storage.close().await
    }
}

/// Storage Manager Builder
///
/// Used to create a storage manager based on the configuration
pub struct StorageManagerBuilder;

impl StorageManagerBuilder {
    /// Creating a default storage manager (read-only)
    #[cfg(feature = "storage-tantivy")]
    pub fn build_tantivy(
        config: crate::storage::tantivy::TantivyStorageConfig,
    ) -> Result<StorageManager> {
        let storage = TantivyStorage::new(config);
        Ok(StorageManager::new(Arc::new(storage)))
    }

    /// Create Default Storage Manager (read-only) - available only when Redis is the default storage
    #[cfg(all(feature = "storage-redis", not(feature = "storage-tantivy")))]
    pub async fn build_redis(
        config: crate::storage::redis::RedisStorageConfig,
    ) -> Result<StorageManager> {
        let storage = RedisStorage::new(config).await?;
        Ok(StorageManager::new(Arc::new(storage)))
    }

    /// Creating a Variable Storage Manager
    #[cfg(feature = "storage-tantivy")]
    pub fn build_mutable_tantivy(
        config: crate::storage::tantivy::TantivyStorageConfig,
    ) -> Result<MutableStorageManager> {
        let storage = TantivyStorage::new(config);
        Ok(MutableStorageManager::new(storage))
    }

    /// Create Variable Storage Manager - available only when Redis is the default storage
    #[cfg(all(feature = "storage-redis", not(feature = "storage-tantivy")))]
    pub async fn build_mutable_redis(
        config: crate::storage::redis::RedisStorageConfig,
    ) -> Result<MutableStorageManager> {
        let storage = RedisStorage::new(config).await?;
        Ok(MutableStorageManager::new(storage))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_storage_manager_creation() {
        #[cfg(feature = "storage-tantivy")]
        {
            let dir = tempdir().expect("Failed to create temp dir");
            let config = crate::storage::tantivy::TantivyStorageConfig {
                index_path: dir.path().to_path_buf(),
                writer_memory_mb: 50,
            };
            let manager = StorageManagerBuilder::build_tantivy(config);
            assert!(manager.is_ok());
        }
    }

    #[tokio::test]
    async fn test_mutable_storage_manager() {
        #[cfg(feature = "storage-tantivy")]
        {
            let dir = tempdir().expect("Failed to create temp dir");
            let config = crate::storage::tantivy::TantivyStorageConfig {
                index_path: dir.path().to_path_buf(),
                writer_memory_mb: 50,
            };
            let manager = StorageManagerBuilder::build_mutable_tantivy(config).unwrap();

            assert!(manager.init().await.is_ok());
            assert!(manager.health_check().await.unwrap_or(false));
            assert!(manager.clear().await.is_ok());
        }
    }
}
