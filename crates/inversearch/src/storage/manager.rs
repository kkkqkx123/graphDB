use crate::error::Result;
use crate::storage::common::types::StorageInfo;
use crate::storage::memory::MemoryStorage;
use crate::{DocId, Index};
use std::sync::Arc;

pub type DefaultStorage = MemoryStorage;

#[derive(Clone)]
pub struct StorageManager {
    storage: Arc<DefaultStorage>,
}

impl StorageManager {
    pub fn new(storage: Arc<DefaultStorage>) -> Self {
        Self { storage }
    }

    pub fn storage(&self) -> Arc<DefaultStorage> {
        self.storage.clone()
    }

    pub async fn open(&self) -> Result<()> {
        Ok(())
    }

    pub async fn close(&self) -> Result<()> {
        Ok(())
    }

    pub async fn mount(&self, _index: &Index) -> Result<()> {
        Ok(())
    }

    pub async fn commit(&self, _index: &Index, _replace: bool, _append: bool) -> Result<()> {
        Ok(())
    }

    pub async fn get(
        &self,
        _key: &str,
        _ctx: Option<&str>,
        _limit: usize,
        _offset: usize,
        _resolve: bool,
        _enrich: bool,
    ) -> Result<crate::r#type::SearchResults> {
        Ok(Vec::new())
    }

    pub async fn enrich(&self, _ids: &[DocId]) -> Result<crate::r#type::EnrichedSearchResults> {
        Ok(Vec::new())
    }

    pub async fn has(&self, _id: DocId) -> Result<bool> {
        Ok(false)
    }

    pub async fn remove(&self, _ids: &[DocId]) -> Result<()> {
        Ok(())
    }

    pub async fn remove_documents(&self, ids: &[DocId]) -> Result<()> {
        self.remove(ids).await
    }

    pub async fn mount_index(&self, index: &Index) -> Result<()> {
        self.mount(index).await
    }

    pub async fn clear(&self) -> Result<()> {
        Ok(())
    }

    pub async fn destroy(&self) -> Result<()> {
        Ok(())
    }

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

pub struct StorageManagerBuilder;

impl StorageManagerBuilder {
    pub async fn build_default() -> Result<StorageManager> {
        let storage = Arc::new(MemoryStorage::new());
        Ok(StorageManager::new(storage))
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

        assert!(manager.has(1).await.is_ok());
        assert!(manager.remove(&[1, 2, 3]).await.is_ok());
        assert!(manager.clear().await.is_ok());

        let results = manager.get("test", None, 10, 0, true, false).await;
        assert!(results.is_ok());
    }
}
