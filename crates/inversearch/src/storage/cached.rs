//! 缓存存储实现
//!
//! 提供内存缓存 + 持久化存储的组合实现
//! 作为默认存储后端，兼顾性能和数据安全

use crate::r#type::{SearchResults, EnrichedSearchResults, DocId};
use crate::error::Result;
use crate::Index;
use crate::storage::common::{StorageInterface, StorageInfo, FileStorageData};
use crate::storage::common::io::{load_from_file, atomic_write, remove_file_safe};
use crate::storage::base::StorageBase;
use std::path::PathBuf;
use std::time::Instant;

/// 缓存存储配置
#[derive(Debug, Clone)]
pub struct CachedStorageConfig {
    /// 基础路径
    pub base_path: PathBuf,
    /// 自动保存间隔（秒），0 表示不自动保存
    pub auto_save_interval: u64,
    /// 是否在 drop 时自动保存
    pub auto_save_on_drop: bool,
}

impl Default for CachedStorageConfig {
    fn default() -> Self {
        Self {
            base_path: PathBuf::from("./data"),
            auto_save_interval: 0,  // 默认不自动保存，由用户控制
            auto_save_on_drop: true,
        }
    }
}

/// 缓存存储
///
/// 结合内存存储的性能和文件存储的持久化能力
/// - 所有读写操作先在内存中进行
/// - 显式调用 `save()` 或 `close()` 时持久化到文件
/// - 打开时自动从文件加载数据
pub struct CachedStorage {
    config: CachedStorageConfig,
    base: StorageBase,
    is_open: bool,
    is_dirty: bool,  // 标记是否有未保存的变更
}

impl CachedStorage {
    /// 使用默认配置创建缓存存储
    pub fn new() -> Self {
        Self::with_config(CachedStorageConfig::default())
    }

    /// 使用指定路径创建缓存存储
    pub fn with_path(base_path: impl Into<PathBuf>) -> Self {
        let config = CachedStorageConfig {
            base_path: base_path.into(),
            ..Default::default()
        };
        Self::with_config(config)
    }

    /// 使用自定义配置创建缓存存储
    pub fn with_config(config: CachedStorageConfig) -> Self {
        Self {
            config,
            base: StorageBase::new(),
            is_open: false,
            is_dirty: false,
        }
    }

    /// 获取内存使用情况
    pub fn get_memory_usage(&self) -> usize {
        self.base.get_memory_usage()
    }

    /// 获取操作统计
    pub fn get_operation_count(&self) -> usize {
        self.base.get_operation_count()
    }

    /// 检查是否有未保存的变更
    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    /// 获取配置
    pub fn config(&self) -> &CachedStorageConfig {
        &self.config
    }

    /// 保存到文件（使用原子写入）
    pub async fn save(&mut self) -> Result<()> {
        let data = FileStorageData {
            version: "1.0.0".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            data: self.base.data.clone(),
            context_data: self.base.context_data.clone(),
            documents: self.base.documents.clone(),
        };

        // 使用 bincode 进行序列化
        let serialized = bincode::serialize(&data)
            .map_err(|e| crate::error::StorageError::Serialization(e.to_string()))?;

        // 使用原子写入
        let data_file = self.config.base_path.join("data.bin");
        atomic_write(&data_file, &serialized).await?;

        // 重置脏标记
        self.is_dirty = false;

        Ok(())
    }

    /// 从文件加载
    pub async fn load(&mut self) -> Result<()> {
        let data_file = self.config.base_path.join("data.bin");
        let data = load_from_file(&data_file).await?;

        self.base.data = data.data;
        self.base.context_data = data.context_data;
        self.base.documents = data.documents;
        self.is_dirty = false;

        self.base.update_memory_usage();

        Ok(())
    }

    /// 记录操作开始时间
    fn record_operation_start(&self) -> Instant {
        Instant::now()
    }

    /// 记录操作完成
    fn record_operation_completion(&self, start_time: Instant) {
        let latency = start_time.elapsed().as_micros() as usize;
        self.base.operation_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.base.total_latency.fetch_add(latency, std::sync::atomic::Ordering::Relaxed);
    }
}

impl Default for CachedStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl StorageInterface for CachedStorage {
    async fn mount(&mut self, _index: &Index) -> Result<()> {
        tokio::fs::create_dir_all(&self.config.base_path).await?;
        self.load().await
    }

    async fn open(&mut self) -> Result<()> {
        self.is_open = true;
        self.load().await
    }

    async fn close(&mut self) -> Result<()> {
        if self.is_dirty {
            self.save().await?;
        }
        self.is_open = false;
        Ok(())
    }

    async fn destroy(&mut self) -> Result<()> {
        self.base.clear();
        self.is_dirty = false;

        let data_file = self.config.base_path.join("data.bin");
        remove_file_safe(&data_file).await?;

        self.base.update_memory_usage();
        self.is_open = false;
        Ok(())
    }

    async fn commit(&mut self, index: &Index, _replace: bool, _append: bool) -> Result<()> {
        let start_time = self.record_operation_start();

        self.base.commit_from_index(index);

        self.is_dirty = true;
        self.base.update_memory_usage();
        self.record_operation_completion(start_time);

        Ok(())
    }

    async fn get(&self, key: &str, ctx: Option<&str>, limit: usize, offset: usize, _resolve: bool, _enrich: bool) -> Result<SearchResults> {
        let results = self.base.get(key, ctx, limit, offset);
        Ok(results)
    }

    async fn enrich(&self, ids: &[DocId]) -> Result<EnrichedSearchResults> {
        let results = self.base.enrich(ids);
        Ok(results)
    }

    async fn has(&self, id: DocId) -> Result<bool> {
        let result = self.base.has(id);
        Ok(result)
    }

    async fn remove(&mut self, ids: &[DocId]) -> Result<()> {
        self.base.remove(ids);
        self.is_dirty = true;
        Ok(())
    }

    async fn clear(&mut self) -> Result<()> {
        self.base.clear();
        self.is_dirty = true;
        Ok(())
    }

    async fn info(&self) -> Result<StorageInfo> {
        let file_size = if self.config.base_path.exists() {
            let data_file = self.config.base_path.join("data.bin");
            crate::storage::common::io::get_file_size(&data_file)
        } else {
            0
        };

        Ok(StorageInfo {
            name: "CachedStorage".to_string(),
            version: "1.0.0".to_string(),
            size: file_size,
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
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_cached_storage_basic() {
        let temp_dir = TempDir::new().expect("TempDir::new should succeed");
        let mut storage = CachedStorage::with_path(temp_dir.path());

        storage.open().await.expect("storage.open should succeed");

        let mut index = Index::default();
        index.add(1, "hello world", false).expect("add should succeed");
        index.add(2, "rust programming", false).expect("add should succeed");

        // 提交到存储
        storage.commit(&index, false, false).await.expect("commit should succeed");
        assert!(storage.is_dirty());

        // 测试获取
        let results = storage.get("hello", None, 10, 0, true, false).await.expect("get should succeed");
        assert_eq!(results.len(), 1);
        assert!(results.contains(&1));

        // 关闭存储（会保存到文件）
        storage.close().await.expect("close should succeed");
        assert!(!storage.is_dirty());

        // 重新打开并验证数据还在
        let mut storage2 = CachedStorage::with_path(temp_dir.path());
        storage2.open().await.expect("storage2.open should succeed");

        let results2 = storage2.get("hello", None, 10, 0, true, false).await.expect("get should succeed");
        assert_eq!(results2.len(), 1);

        storage2.destroy().await.expect("destroy should succeed");
    }

    #[tokio::test]
    async fn test_cached_storage_persistence() {
        let temp_dir = TempDir::new().expect("TempDir::new should succeed");
        let path = temp_dir.path().to_path_buf();

        // 第一次创建并写入数据
        {
            let mut storage = CachedStorage::with_path(&path);
            storage.open().await.expect("storage.open should succeed");

            let mut index = Index::default();
            index.add(1, "persistent data", false).expect("add should succeed");
            storage.commit(&index, false, false).await.expect("commit should succeed");

            storage.close().await.expect("close should succeed");
        }

        // 第二次打开验证数据持久化
        {
            let mut storage = CachedStorage::with_path(&path);
            storage.open().await.expect("storage.open should succeed");

            let results = storage.get("persistent", None, 10, 0, true, false).await.expect("get should succeed");
            assert_eq!(results.len(), 1);
            assert!(results.contains(&1));

            storage.destroy().await.expect("destroy should succeed");
        }
    }
}
