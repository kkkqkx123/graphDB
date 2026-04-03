//! 文件存储实现
//!
//! 提供基于文件的持久化存储后端

use crate::r#type::{SearchResults, EnrichedSearchResults, DocId};
use crate::error::Result;
use crate::Index;
use crate::storage::common::{StorageInterface, StorageInfo, StorageMetrics, FileStorageData};
use crate::storage::common::io::{save_to_file, load_from_file};
use crate::storage::base::StorageBase;
use std::path::PathBuf;

/// 文件存储
pub struct FileStorage {
    base: StorageBase,
    base_path: PathBuf,
    is_open: bool,
}

impl FileStorage {
    /// 创建新的文件存储
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            base: StorageBase::new(),
            base_path: base_path.into(),
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

    /// 获取文件大小
    pub fn get_file_size(&self) -> u64 {
        let data_file = self.base_path.join("data.bin");
        crate::storage::common::io::get_file_size(&data_file)
    }

    /// 保存到文件
    pub async fn save_to_file(&self) -> Result<()> {
        let data = FileStorageData {
            version: "1.0.0".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            data: self.base.data.clone(),
            context_data: self.base.context_data.clone(),
            documents: self.base.documents.clone(),
        };

        let data_file = self.base_path.join("data.bin");
        save_to_file(&data_file, &data).await
    }

    /// 从文件加载
    pub async fn load_from_file(&mut self) -> Result<()> {
        let data_file = self.base_path.join("data.bin");
        let data = load_from_file(&data_file).await?;

        self.base.data = data.data;
        self.base.context_data = data.context_data;
        self.base.documents = data.documents;
        self.base.update_memory_usage();

        Ok(())
    }
}

#[async_trait::async_trait]
impl StorageInterface for FileStorage {
    async fn mount(&mut self, _index: &Index) -> Result<()> {
        tokio::fs::create_dir_all(&self.base_path).await?;

        if let Err(e) = self.load_from_file().await {
            eprintln!("Failed to load from file: {}", e);
        }
        Ok(())
    }

    async fn open(&mut self) -> Result<()> {
        self.is_open = true;
        self.load_from_file().await
    }

    async fn close(&mut self) -> Result<()> {
        self.save_to_file().await?;
        self.is_open = false;
        Ok(())
    }

    async fn destroy(&mut self) -> Result<()> {
        self.base.clear();

        let data_file = self.base_path.join("data.bin");
        crate::storage::common::io::remove_file_safe(&data_file).await?;

        self.base.update_memory_usage();
        self.is_open = false;
        Ok(())
    }

    async fn commit(&mut self, index: &Index, _replace: bool, _append: bool) -> Result<()> {
        let start_time = self.base.record_operation_start();

        self.base.commit_from_index(index);
        self.save_to_file().await?;
        self.base.update_memory_usage();
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
        self.save_to_file().await?;
        self.base.update_memory_usage();
        self.base.record_operation_completion(start_time);
        Ok(())
    }

    async fn clear(&mut self) -> Result<()> {
        let start_time = self.base.record_operation_start();

        self.base.clear();
        self.save_to_file().await?;
        self.base.update_memory_usage();
        self.base.record_operation_completion(start_time);
        Ok(())
    }

    async fn info(&self) -> Result<StorageInfo> {
        Ok(StorageInfo {
            name: "FileStorage".to_string(),
            version: "1.0.0".to_string(),
            size: self.get_file_size(),
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
    async fn test_file_storage() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        let mut storage = FileStorage::new(dir_path.to_str().unwrap().to_string());
        storage.open().await.unwrap();

        let mut index = Index::default();
        index.add(1, "test document", false).unwrap();
        index.add(2, "another test", false).unwrap();

        // 提交到存储
        storage.commit(&index, false, false).await.unwrap();

        // 测试获取
        let results = storage.get("test", None, 10, 0, true, false).await.unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.contains(&1));
        assert!(results.contains(&2));

        // 关闭存储（会保存到文件）
        storage.close().await.unwrap();

        // 重新打开并验证数据还在
        let mut storage2 = FileStorage::new(dir_path.to_str().unwrap().to_string());
        storage2.open().await.unwrap();

        let results2 = storage2.get("test", None, 10, 0, true, false).await.unwrap();
        assert_eq!(results2.len(), 2);

        storage2.destroy().await.unwrap();
    }
}
