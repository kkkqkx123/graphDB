//! 内存存储实现
//!
//! 提供基于内存的存储后端，数据不持久化

use crate::error::Result;
use crate::r#type::{DocId, EnrichedSearchResults, SearchResults};
use crate::storage::common::base::StorageBase;
use crate::storage::common::{StorageInfo, StorageInterface, StorageMetrics};
use crate::Index;
use tokio::sync::RwLock;

/// 内存存储
pub struct MemoryStorage {
    base: RwLock<StorageBase>,
    is_open: RwLock<bool>,
}

impl MemoryStorage {
    /// 创建新的内存存储
    pub fn new() -> Self {
        Self {
            base: RwLock::new(StorageBase::new()),
            is_open: RwLock::new(false),
        }
    }

    /// 获取内存使用情况
    pub fn get_memory_usage(&self) -> usize {
        self.base.blocking_read().get_memory_usage()
    }

    /// 获取操作统计
    pub async fn get_operation_stats(&self) -> StorageMetrics {
        let base = self.base.read().await;
        StorageMetrics {
            operation_count: base.get_operation_count(),
            average_latency: base.get_average_latency(),
            memory_usage: base.get_memory_usage(),
            error_count: 0,
        }
    }
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl StorageInterface for MemoryStorage {
    async fn mount(&self, _index: &Index) -> Result<()> {
        Ok(())
    }

    async fn open(&self) -> Result<()> {
        *self.is_open.write().await = true;
        Ok(())
    }

    async fn close(&self) -> Result<()> {
        *self.is_open.write().await = false;
        Ok(())
    }

    async fn destroy(&self) -> Result<()> {
        let mut base = self.base.write().await;
        base.clear();
        *self.is_open.write().await = false;
        Ok(())
    }

    async fn commit(&self, index: &Index, _replace: bool, _append: bool) -> Result<()> {
        let mut base = self.base.write().await;
        let start_time = base.record_operation_start();
        base.commit_from_index(index);
        base.record_operation_completion(start_time);
        Ok(())
    }

    async fn get(
        &self,
        key: &str,
        ctx: Option<&str>,
        limit: usize,
        offset: usize,
        _resolve: bool,
        _enrich: bool,
    ) -> Result<SearchResults> {
        let base = self.base.read().await;
        let start_time = base.record_operation_start();
        let results = base.get(key, ctx, limit, offset);
        base.record_operation_completion(start_time);
        Ok(results)
    }

    async fn enrich(&self, ids: &[DocId]) -> Result<EnrichedSearchResults> {
        let base = self.base.read().await;
        let start_time = base.record_operation_start();
        let results = base.enrich(ids);
        base.record_operation_completion(start_time);
        Ok(results)
    }

    async fn has(&self, id: DocId) -> Result<bool> {
        let base = self.base.read().await;
        let start_time = base.record_operation_start();
        let result = base.has(id);
        base.record_operation_completion(start_time);
        Ok(result)
    }

    async fn remove(&self, ids: &[DocId]) -> Result<()> {
        let mut base = self.base.write().await;
        let start_time = base.record_operation_start();
        base.remove(ids);
        base.update_memory_usage();
        base.record_operation_completion(start_time);
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        let mut base = self.base.write().await;
        let start_time = base.record_operation_start();
        base.clear();
        base.update_memory_usage();
        base.record_operation_completion(start_time);
        Ok(())
    }

    async fn info(&self) -> Result<StorageInfo> {
        let base = self.base.read().await;
        Ok(StorageInfo {
            name: "MemoryStorage".to_string(),
            version: "0.1.0".to_string(),
            size: (base.get_index_count() + base.get_document_count()) as u64,
            document_count: base.get_document_count(),
            index_count: base.get_index_count(),
            is_connected: *self.is_open.read().await,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Index;

    #[tokio::test]
    async fn test_memory_storage() {
        let storage = MemoryStorage::new();
        storage.open().await.unwrap();

        let mut index = Index::default();
        index.add(1, "hello world", false).unwrap();
        index.add(2, "rust programming", false).unwrap();

        // 提交到存储
        storage.commit(&index, false, false).await.unwrap();

        // 测试获取
        let results = storage
            .get("hello", None, 10, 0, true, false)
            .await
            .unwrap();
        println!("Get results: {:?}", results);
        assert_eq!(results.len(), 1);
        assert!(results.contains(&1));

        // 测试存在检查
        println!("Checking has(1)");
        let has_result = storage.has(1).await.unwrap();
        println!("has(1) result: {}", has_result);
        assert!(has_result);
        assert!(!storage.has(3).await.unwrap());

        // 测试删除
        storage.remove(&[1]).await.unwrap();
        assert!(!storage.has(1).await.unwrap());

        storage.close().await.unwrap();
    }
}
