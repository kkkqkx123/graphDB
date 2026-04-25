//! Storage Factory for creating storage instances
//!
//! This module provides a factory pattern for creating storage backends
//! based on configuration.

use crate::config::{Config, StorageBackend};
use crate::error::Result;
use crate::storage::common::StorageInterface;
use std::sync::Arc;

/// Storage factory for creating storage instances
pub struct StorageFactory;

impl StorageFactory {
    /// Create a storage instance based on configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Service configuration
    ///
    /// # Returns
    ///
    /// * `Result<Arc<dyn StorageInterface>>` - Created storage instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use inversearch_service::config::Config;
    /// use inversearch_service::storage::factory::StorageFactory;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = Config::default();
    /// let storage = StorageFactory::from_config(&config).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn from_config(config: &Config) -> Result<Arc<dyn StorageInterface>> {
        if !config.storage.enabled {
            return Self::create_cold_warm_cache().await;
        }

        match &config.storage.backend {
            #[cfg(feature = "store-file")]
            StorageBackend::File => Self::create_file(&config.storage),

            #[cfg(feature = "store-redis")]
            StorageBackend::Redis => Self::create_redis(&config.storage).await,

            #[cfg(feature = "store-wal")]
            StorageBackend::Wal => Self::create_wal(&config.storage).await,

            StorageBackend::ColdWarmCache => Self::create_cold_warm_cache().await,
        }
    }

    /// Create a file storage instance
    ///
    /// # Arguments
    ///
    /// * `storage_config` - Storage configuration
    ///
    /// # Returns
    ///
    /// * `Result<Arc<dyn StorageInterface>>` - Created storage instance
    #[cfg(feature = "store-file")]
    pub fn create_file(
        storage_config: &crate::config::StorageConfig,
    ) -> Result<Arc<dyn StorageInterface>> {
        use crate::storage::file::FileStorage;

        let path = storage_config
            .file
            .as_ref()
            .map(|c| c.base_path.clone())
            .unwrap_or_else(|| "./data".to_string());

        let storage = FileStorage::new(path);
        Ok(Arc::new(storage))
    }

    /// Get file storage path from configuration
    pub fn get_file_path(storage_config: &crate::config::StorageConfig) -> String {
        #[cfg(feature = "store-file")]
        {
            storage_config
                .file
                .as_ref()
                .map(|c| c.base_path.clone())
                .unwrap_or_else(|| "./data".to_string())
        }
        #[cfg(not(feature = "store-file"))]
        {
            let _ = storage_config;
            "./data".to_string()
        }
    }

    /// Create a Redis storage instance
    ///
    /// # Arguments
    ///
    /// * `storage_config` - Storage configuration
    ///
    /// # Returns
    ///
    /// * `Result<Arc<dyn StorageInterface>>` - Created storage instance
    #[cfg(feature = "store-redis")]
    pub async fn create_redis(
        storage_config: &crate::config::StorageConfig,
    ) -> Result<Arc<dyn StorageInterface>> {
        use crate::storage::redis::{RedisStorage, RedisStorageConfig};

        let config = storage_config
            .redis
            .as_ref()
            .map(|c| RedisStorageConfig {
                url: c.url.clone(),
                pool_size: c.pool_size,
                ..Default::default()
            })
            .unwrap_or_default();

        let storage = RedisStorage::new(config).await?;
        Ok(Arc::new(storage))
    }

    /// Create a WAL storage instance
    ///
    /// # Arguments
    ///
    /// * `storage_config` - Storage configuration
    ///
    /// # Returns
    ///
    /// * `Result<Arc<dyn StorageInterface>>` - Created storage instance
    #[cfg(feature = "store-wal")]
    pub async fn create_wal(
        storage_config: &crate::config::StorageConfig,
    ) -> Result<Arc<dyn StorageInterface>> {
        use crate::storage::wal::WALConfig;
        use crate::storage::wal::WALStorage;

        let config = storage_config
            .wal
            .as_ref()
            .map(|c| WALConfig {
                base_path: std::path::PathBuf::from(&c.base_path),
                max_wal_size: c.max_wal_size,
                compression: c.compression,
                snapshot_interval: c.snapshot_interval,
                ..Default::default()
            })
            .unwrap_or_default();

        let storage = WALStorage::new(config).await?;
        Ok(Arc::new(storage))
    }

    /// Create a cold-warm cache storage instance
    ///
    /// # Returns
    ///
    /// * `Result<Arc<dyn StorageInterface>>` - Created storage instance
    async fn create_cold_warm_cache() -> Result<Arc<dyn StorageInterface>> {
        #[cfg(feature = "store-cold-warm-cache")]
        {
            use crate::storage::cold_warm_cache::ColdWarmCacheManager;
            let manager = ColdWarmCacheManager::new().await?;
            // ColdWarmCacheManager is already Arc<Self> and implements StorageInterface
            // We can directly cast it to Arc<dyn StorageInterface>
            Ok(manager as Arc<dyn StorageInterface>)
        }

        #[cfg(not(feature = "store-cold-warm-cache"))]
        {
            use crate::error::{InversearchError, StorageError};
            Err(InversearchError::Storage(StorageError::Generic(
                "Cold-warm cache storage is not enabled".to_string(),
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, StorageBackend};

    #[tokio::test]
    async fn test_create_cold_warm_cache() {
        // Use a temporary directory for the test to avoid conflicts with existing data
        let temp_dir =
            std::env::temp_dir().join(format!("inversearch_test_{}", std::process::id()));
        let wal_path = temp_dir.join("wal");
        let cold_path = temp_dir.join("cold");

        // Clean up any existing test data
        let _ = tokio::fs::remove_dir_all(&temp_dir).await;

        // Create directories
        let _ = tokio::fs::create_dir_all(&wal_path).await;
        let _ = tokio::fs::create_dir_all(&cold_path).await;

        let config = Config {
            storage: crate::config::StorageConfig {
                enabled: false,
                backend: StorageBackend::ColdWarmCache,
                ..Default::default()
            },
            ..Default::default()
        };

        let result = StorageFactory::from_config(&config).await;
        if let Err(ref e) = result {
            eprintln!("Failed to create cold-warm cache: {:?}", e);
        }
        assert!(result.is_ok(), "Should create cold-warm cache storage");

        // Clean up temp directory
        let _ = tokio::fs::remove_dir_all(&temp_dir).await;
    }
}
