//! 内存存储实现
//!
//! 提供基于内存的存储后端，数据不持久化

use crate::r#type::{SearchResults, EnrichedSearchResults, DocId};
use crate::error::Result;
use crate::Index;
use crate::storage::common::{StorageInterface, StorageInfo, StorageMetrics};
use crate::storage::base::StorageBase;

/// 内存存储
pub struct MemoryStorage {
    base: StorageBase,
    is_open: bool,
}

impl MemoryStorage {
    /// 创建新的内存存储
    pub fn new() -> Self {
        Self {
            base: StorageBase::new(),
            is_open: false,
        }
    }

    /// 获取内存使用情况
    pub fn get_memory_usage(&self) -> usize {
        self.base.get_memory_usage()
    }

    /// 获取操作统计
    pub fn get_operation_stats(&self) -> StorageMetrics {
        StorageMetrics {
            operation_count: self.base.get_operation_count(),
            average_latency: self.base.get_average_latency(),
            memory_usage: self.base.get_memory_usage(),
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
    async fn mount(&mut self, _index: &Index) -> Result<()> {
        Ok(())
    }

    async fn open(&mut self) -> Result<()> {
        self.is_open = true;
        Ok(())
    }

    async fn close(&mut self) -> Result<()> {
        self.is_open = false;
        Ok(())
    }

    async fn destroy(&mut self) -> Result<()> {
        self.base.clear();
        self.is_open = false;
        Ok(())
    }

    async fn commit(&mut self, index: &Index, _replace: bool, _append: bool) -> Result<()> {
        let start_time = self.base.record_operation_start();
        self.base.commit_from_index(index);
        self.base.record_operation_completion(start_time);
        Ok(())
    }

    async fn get(&self, key: &str, ctx: Option<&str>, limit: usize, offset: usize, _resolve: bool, _enrich: bool) -> Result<SearchResults> {
        let start_time = self.base.record_operation_start();
        let results = self.base.get(key, ctx, limit, offset);
        self.base.record_operation_completion(start_time);
        Ok(results)
    }

    async fn enrich(&self, ids: &[DocId]) -> Result<EnrichedSearchResults> {
        let start_time = self.base.record_operation_start();
        let results = self.base.enrich(ids);
        self.base.record_operation_completion(start_time);
        Ok(results)
    }

    async fn has(&self, id: DocId) -> Result<bool> {
        let start_time = self.base.record_operation_start();
        let result = self.base.has(id);
        self.base.record_operation_completion(start_time);
        Ok(result)
    }

    async fn remove(&mut self, ids: &[DocId]) -> Result<()> {
        let start_time = self.base.record_operation_start();
        self.base.remove(ids);
        self.base.update_memory_usage();
        self.base.record_operation_completion(start_time);
        Ok(())
    }

    async fn clear(&mut self) -> Result<()> {
        let start_time = self.base.record_operation_start();
        self.base.clear();
        self.base.update_memory_usage();
        self.base.record_operation_completion(start_time);
        Ok(())
    }

    async fn info(&self) -> Result<StorageInfo> {
        Ok(StorageInfo {
            name: "MemoryStorage".to_string(),
            version: "0.1.0".to_string(),
            size: (self.base.get_index_count() + self.base.get_document_count()) as u64,
            document_count: self.base.get_document_count(),
            index_count: self.base.get_index_count(),
            is_connected: self.is_open,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Index;

    #[tokio::test]
    async fn test_memory_storage() {
        let mut storage = MemoryStorage::new();
        storage.open().await.unwrap();

        let mut index = Index::default();
        index.add(1, "hello world", false).unwrap();
        index.add(2, "rust programming", false).unwrap();

        // 提交到存储
        storage.commit(&index, false, false).await.unwrap();

        // 测试获取
        let results = storage.get("hello", None, 10, 0, true, false).await.unwrap();
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
