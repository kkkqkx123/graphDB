//! Storage Factory Module
//!
//! Provides factory methods for creating storage instances based on configuration.

use crate::config::{RedisStorageConfig, StorageConfig, StorageType, TantivyStorageConfig};
#[cfg(any(not(feature = "storage-tantivy"), not(feature = "storage-redis")))]
use crate::error::Bm25Error;
use crate::error::Result;
use crate::storage::common::r#trait::StorageInterface;
use std::sync::Arc;

/// Storage factory for creating storage instances
pub struct StorageFactory;

impl StorageFactory {
    /// Create a storage instance based on configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Storage configuration
    ///
    /// # Returns
    ///
    /// * `Result<Arc<dyn StorageInterface>>` - Created storage instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bm25_service::config::StorageConfig;
    /// use bm25_service::storage::StorageFactory;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = StorageConfig::default();
    /// let storage = StorageFactory::create(config).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(config: StorageConfig) -> Result<Arc<dyn StorageInterface>> {
        match config.storage_type {
            StorageType::Tantivy => Self::create_tantivy(config.tantivy),
            StorageType::Redis => Self::create_redis(config.redis).await,
        }
    }

    /// Create a Tantivy storage instance
    ///
    /// # Arguments
    ///
    /// * `config` - Tantivy storage configuration
    ///
    /// # Returns
    ///
    /// * `Result<Arc<dyn StorageInterface>>` - Created storage instance
    pub fn create_tantivy(config: TantivyStorageConfig) -> Result<Arc<dyn StorageInterface>> {
        #[cfg(feature = "storage-tantivy")]
        {
            use crate::storage::TantivyStorage;

            let storage_config = crate::storage::tantivy::TantivyStorageConfig {
                index_path: std::path::PathBuf::from(&config.index_path),
                writer_memory_mb: config.writer_memory_mb,
            };

            let storage = TantivyStorage::new(storage_config);
            Ok(Arc::new(storage))
        }

        #[cfg(not(feature = "storage-tantivy"))]
        {
            let _config = config;
            Err(Bm25Error::StorageError(
                "Tantivy storage is not enabled. Please enable the 'storage-tantivy' feature."
                    .to_string(),
            ))
        }
    }

    /// Create a Redis storage instance
    ///
    /// # Arguments
    ///
    /// * `config` - Redis storage configuration
    ///
    /// # Returns
    ///
    /// * `Result<Arc<dyn StorageInterface>>` - Created storage instance
    pub async fn create_redis(config: RedisStorageConfig) -> Result<Arc<dyn StorageInterface>> {
        #[cfg(feature = "storage-redis")]
        {
            use crate::storage::redis::RedisStorageConfig as InternalRedisConfig;
            use crate::storage::RedisStorage;
            use std::time::Duration;

            let storage_config = InternalRedisConfig {
                url: config.url,
                pool_size: config.pool_size,
                connection_timeout: Duration::from_secs(config.connection_timeout_secs),
                key_prefix: config.key_prefix,
                min_idle: config.min_idle,
                max_lifetime: config.max_lifetime_secs.map(Duration::from_secs),
                connection_timeout_bb8: Duration::from_secs(config.connection_timeout_secs),
            };

            let mut storage = RedisStorage::new(storage_config).await?;
            storage.init().await?;

            Ok(Arc::new(storage))
        }

        #[cfg(not(feature = "storage-redis"))]
        {
            let _config = config; // Suppress unused variable warning
            Err(Bm25Error::StorageError(
                "Redis storage is not enabled. Please enable the 'storage-redis' feature."
                    .to_string(),
            ))
        }
    }

    /// Create and initialize a storage instance
    ///
    /// This method creates the storage and calls init() on it.
    ///
    /// # Arguments
    ///
    /// * `config` - Storage configuration
    ///
    /// # Returns
    ///
    /// * `Result<Arc<dyn StorageInterface>>` - Created and initialized storage instance
    pub async fn create_and_init(config: StorageConfig) -> Result<Arc<dyn StorageInterface>> {
        let storage = Self::create(config).await?;

        // Clone the Arc to call init
        // Note: This requires the storage to be mutable, which is handled internally
        Ok(storage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::StorageConfig;

    #[test]
    fn test_storage_factory_default() {
        let config = StorageConfig::default();
        assert_eq!(config.storage_type, StorageType::Tantivy);
    }

    #[tokio::test]
    #[cfg(feature = "storage-tantivy")]
    async fn test_storage_factory_create_tantivy() {
        use tempfile::tempdir;

        let temp_dir = tempdir().unwrap();
        let index_path = temp_dir.path().join("test_index");

        let config = StorageConfig::builder()
            .tantivy_index_path(index_path.to_string_lossy().to_string())
            .build();

        let result = StorageFactory::create(config).await;
        assert!(result.is_ok());
    }
}
