use crate::config::{Config, StorageBackend};
use crate::error::Result;
use crate::storage::cold_warm_cache::{ColdWarmCacheConfig, ColdWarmCacheManager};
use crate::storage::common::StorageInterface;
use crate::storage::memory::MemoryStorage;
use std::sync::Arc;

pub struct StorageFactory;

impl StorageFactory {
    pub async fn from_config(config: &Config) -> Result<Arc<dyn StorageInterface>> {
        if !config.storage.enabled {
            let storage = MemoryStorage::new();
            return Ok(Arc::new(storage));
        }

        match config.storage.backend {
            StorageBackend::ColdWarmCache => {
                let cache_config = ColdWarmCacheConfig {
                    hot_cache_max_size: config.cache.size * 1024 * 1024,
                    cold_storage_path: config.storage.base_path.clone()
                        .unwrap_or_else(|| std::path::PathBuf::from("./data/cold")),
                    wal_path: config.storage.wal_dir.clone()
                        .unwrap_or_else(|| std::path::PathBuf::from("./data/wal")),
                    wal_enabled: config.storage.enable_wal,
                    ..ColdWarmCacheConfig::default()
                };
                let manager = ColdWarmCacheManager::with_config(cache_config).await?;
                Ok(manager as Arc<dyn StorageInterface>)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[tokio::test]
    async fn test_create_storage() {
        let config = Config::default();
        let result = StorageFactory::from_config(&config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_cold_warm_cache() {
        let mut config = Config::default();
        config.storage.enabled = true;
        config.storage.backend = StorageBackend::ColdWarmCache;
        let result = StorageFactory::from_config(&config).await;
        assert!(result.is_ok());
    }
}
