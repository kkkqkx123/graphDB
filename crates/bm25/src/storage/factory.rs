use crate::config::{StorageConfig, StorageType, TantivyStorageConfig};
use crate::error::Result;
use crate::storage::common::r#trait::StorageInterface;
use crate::storage::storage_enum::StorageEnum;
use crate::storage::TantivyStorage;

pub struct StorageFactory;

impl StorageFactory {
    pub async fn create(config: StorageConfig) -> Result<StorageEnum> {
        match config.storage_type {
            StorageType::Tantivy => Self::create_tantivy(config.tantivy),
        }
    }

    pub fn create_tantivy(config: TantivyStorageConfig) -> Result<StorageEnum> {
        let storage_config = crate::storage::tantivy::TantivyStorageConfig {
            index_path: std::path::PathBuf::from(&config.index_path),
            writer_memory_mb: config.writer_memory_mb,
        };

        let storage = TantivyStorage::new(storage_config);
        Ok(StorageEnum::Tantivy(storage))
    }

    pub async fn create_and_init(config: StorageConfig) -> Result<StorageEnum> {
        let mut storage = Self::create(config).await?;
        storage.init().await?;
        Ok(storage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_factory_default() {
        let config = StorageConfig::default();
        assert_eq!(config.storage_type, StorageType::Tantivy);
    }

    #[tokio::test]
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
