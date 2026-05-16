use crate::config::Config;
use crate::error::Result;
use crate::storage::common::StorageInterface;
use crate::storage::memory::MemoryStorage;
use std::sync::Arc;

pub struct StorageFactory;

impl StorageFactory {
    pub async fn from_config(_config: &Config) -> Result<Arc<dyn StorageInterface>> {
        let storage = MemoryStorage::new();
        Ok(Arc::new(storage))
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
}
