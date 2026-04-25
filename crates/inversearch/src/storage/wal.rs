//! WAL (Write-Ahead Log) 模块
//!
//! 提供预写式日志功能，确保数据持久性和崩溃恢复能力

use crate::error::{InversearchError, Result};
use crate::r#type::DocId;
use crate::Index;
use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Utc};
use oxicode::config::standard;
use oxicode::serde::{decode_from_slice, encode_to_vec};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::fs as tokio_fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;

/// 索引变更类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndexChange {
    /// 添加文档
    Add { doc_id: DocId, content: String },
    /// 删除文档
    Remove { doc_id: DocId },
    /// 更新文档
    Update { doc_id: DocId, content: String },
}

/// WAL 配置
#[derive(Debug, Clone)]
pub struct WALConfig {
    /// 基础路径
    pub base_path: PathBuf,
    /// 最大 WAL 文件大小（字节）
    pub max_wal_size: usize,
    /// 是否启用压缩
    pub compression: bool,
    /// 压缩级别 (1-22)
    pub compression_level: i32,
    /// 最大 WAL 文件数量（用于轮转）
    pub max_wal_files: usize,
    /// 快照间隔（变更次数）
    pub snapshot_interval: usize,
    /// 是否启用自动清理
    pub auto_cleanup: bool,
    /// 清理间隔（秒）
    pub cleanup_interval: u64,
}

impl Default for WALConfig {
    fn default() -> Self {
        Self {
            base_path: PathBuf::from("./wal"),
            max_wal_size: 100 * 1024 * 1024, // 100MB
            compression: true,
            compression_level: 3,
            max_wal_files: 10,
            snapshot_interval: 1000,
            auto_cleanup: true,
            cleanup_interval: 3600, // 1 小时
        }
    }
}

/// WAL 管理器
pub struct WALManager {
    config: WALConfig,
    wal_path: PathBuf,
    snapshot_path: PathBuf,
    wal_size: Arc<AtomicUsize>,
    change_count: Arc<AtomicUsize>,
    last_cleanup_time: Arc<Mutex<Option<DateTime<Utc>>>>,
}

impl WALManager {
    /// 创建新的 WAL 管理器
    pub async fn new(config: WALConfig) -> Result<Self> {
        tokio_fs::create_dir_all(&config.base_path).await?;

        let wal_path = config.base_path.join("wal.log");
        let snapshot_path = config.base_path.join("snapshot.bin");

        // 获取当前 WAL 文件大小
        let wal_size = if wal_path.exists() {
            tokio_fs::metadata(&wal_path).await?.len() as usize
        } else {
            0
        };

        // 启动自动清理任务
        let manager = Self {
            config,
            wal_path,
            snapshot_path,
            wal_size: Arc::new(AtomicUsize::new(wal_size)),
            change_count: Arc::new(AtomicUsize::new(0)),
            last_cleanup_time: Arc::new(Mutex::new(None)),
        };

        if manager.config.auto_cleanup {
            manager.start_cleanup_task();
        }

        Ok(manager)
    }

    /// 启动自动清理任务
    fn start_cleanup_task(&self) {
        let config = self.config.clone();
        let last_cleanup_time = self.last_cleanup_time.clone();
        let base_path = self.config.base_path.clone();

        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(tokio::time::Duration::from_secs(config.cleanup_interval));

            loop {
                interval.tick().await;
                let mut last_cleanup = last_cleanup_time.lock().await;
                let now = Utc::now();
                if let Some(last) = *last_cleanup {
                    let duration = now.signed_duration_since(last);
                    if duration.num_seconds() >= config.cleanup_interval as i64 {
                        *last_cleanup = Some(now);

                        // 执行清理逻辑
                        if let Ok(mut entries) = tokio_fs::read_dir(&base_path).await {
                            let mut wal_files = Vec::new();

                            while let Some(entry) = entries.next_entry().await.ok().flatten() {
                                let path = entry.path();
                                if let Some(file_name) = path.file_name() {
                                    let file_name_str = file_name.to_string_lossy();
                                    if file_name_str.starts_with("wal_")
                                        && file_name_str.ends_with(".log")
                                    {
                                        if let Ok(metadata) = entry.metadata().await {
                                            wal_files.push((path, metadata.modified().ok()));
                                        }
                                    }
                                }
                            }

                            // 按修改时间排序
                            wal_files.sort_by(|a, b| match (&a.1, &b.1) {
                                (Some(time_a), Some(time_b)) => time_a.cmp(time_b),
                                (Some(_), None) => std::cmp::Ordering::Less,
                                (None, Some(_)) => std::cmp::Ordering::Greater,
                                (None, None) => std::cmp::Ordering::Equal,
                            });

                            // 删除超过最大数量的文件
                            let files_to_remove =
                                wal_files.len().saturating_sub(config.max_wal_files);
                            for (path, _) in wal_files.iter().take(files_to_remove) {
                                let _ = tokio_fs::remove_file(path).await;
                            }
                        }
                    }
                } else {
                    *last_cleanup = Some(now);
                }
            }
        });
    }

    /// 记录变更到 WAL
    pub async fn record_change(&self, change: IndexChange) -> Result<()> {
        let serialized = encode_to_vec(&change, standard())?;
        let encoded = general_purpose::STANDARD.encode(&serialized);
        let line = format!("{}\n", encoded);

        let mut file = tokio_fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.wal_path)
            .await?;

        file.write_all(line.as_bytes()).await?;
        file.flush().await?;

        self.wal_size.fetch_add(line.len(), Ordering::SeqCst);
        let count = self.change_count.fetch_add(1, Ordering::SeqCst) + 1;

        // 检查是否需要创建快照
        if count >= self.config.snapshot_interval {
            // 这里简化处理，实际应该异步创建快照
            self.change_count.store(0, Ordering::SeqCst);
        }

        Ok(())
    }

    /// 批量记录变更
    pub async fn record_changes(&self, changes: Vec<IndexChange>) -> Result<()> {
        if changes.is_empty() {
            return Ok(());
        }

        let mut lines = Vec::new();
        for change in changes {
            let serialized = encode_to_vec(&change, standard())?;
            let encoded = general_purpose::STANDARD.encode(&serialized);
            lines.push(format!("{}\n", encoded));
        }

        let mut file = tokio_fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.wal_path)
            .await?;

        let mut total_size = 0;
        for line in &lines {
            file.write_all(line.as_bytes()).await?;
            total_size += line.len();
        }
        self.wal_size.fetch_add(total_size, Ordering::SeqCst);
        file.sync_data().await?;

        // WAL 超过阈值时触发快照
        if self.wal_size.load(Ordering::Relaxed) > self.config.max_wal_size {
            self.trigger_snapshot().await?;
        }

        Ok(())
    }

    /// 触发快照（异步任务）
    async fn trigger_snapshot(&self) -> Result<()> {
        // 1. 轮转 WAL 文件
        self.rotate_wal_files().await?;

        // 2. 创建快照
        // 注意：快照创建需要传入 Index 实例，这里简化处理
        // 实际使用时应该在外部调用 create_snapshot 方法

        Ok(())
    }

    /// 轮转 WAL 文件
    async fn rotate_wal_files(&self) -> Result<()> {
        if !self.wal_path.exists() {
            return Ok(());
        }

        // 生成新的 WAL 文件名（带时间戳）
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
        let rotated_path = self.config.base_path.join(format!("wal_{}.log", timestamp));

        // 重命名当前 WAL 文件
        tokio_fs::rename(&self.wal_path, &rotated_path).await?;

        // 清理旧的 WAL 文件
        self.cleanup_old_wal_files().await?;

        // 重置 WAL 大小
        self.wal_size.store(0, Ordering::SeqCst);

        Ok(())
    }

    /// 清理旧的 WAL 文件
    async fn cleanup_old_wal_files(&self) -> Result<()> {
        let mut entries = tokio_fs::read_dir(&self.config.base_path).await?;
        let mut wal_files = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if let Some(file_name) = path.file_name() {
                let file_name_str = file_name.to_string_lossy();
                if file_name_str.starts_with("wal_") && file_name_str.ends_with(".log") {
                    if let Ok(metadata) = entry.metadata().await {
                        wal_files.push((path, metadata.modified().ok()));
                    }
                }
            }
        }

        // 按修改时间排序
        wal_files.sort_by(|a, b| match (&a.1, &b.1) {
            (Some(time_a), Some(time_b)) => time_a.cmp(time_b),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        });

        // 删除超过最大数量的文件
        let files_to_remove = wal_files.len().saturating_sub(self.config.max_wal_files);
        for (path, _) in wal_files.iter().take(files_to_remove) {
            let _ = tokio_fs::remove_file(path).await;
        }

        Ok(())
    }

    /// 手动触发清理
    pub async fn manual_cleanup(&self) -> Result<()> {
        self.cleanup_old_wal_files().await?;

        // 更新最后清理时间
        let mut last_cleanup = self.last_cleanup_time.lock().await;
        *last_cleanup = Some(Utc::now());

        Ok(())
    }

    /// 创建快照
    pub async fn create_snapshot(&self, index: &Index) -> Result<()> {
        let temp_snapshot = self.config.base_path.join("snapshot.tmp");

        // 1. 序列化索引
        let snapshot_data = self.serialize_index(index)?;

        // 2. 压缩（如果启用）
        let final_data = if self.config.compression {
            compress_data(&snapshot_data, self.config.compression_level)?
        } else {
            snapshot_data
        };

        // 3. 写入临时文件
        let mut file = tokio_fs::File::create(&temp_snapshot).await?;
        file.write_all(&final_data).await?;
        file.sync_all().await?;
        drop(file);

        // 4. 原子替换
        tokio_fs::rename(&temp_snapshot, &self.snapshot_path).await?;

        // 5. 清空 WAL
        let _ = tokio_fs::remove_file(&self.wal_path).await;

        Ok(())
    }

    /// 加载索引（从快照 + WAL）
    pub async fn load(&self, index: &mut Index) -> Result<()> {
        // 1. 加载快照
        if self.snapshot_path.exists() {
            let mut file = tokio_fs::File::open(&self.snapshot_path).await?;
            let mut data = Vec::new();
            file.read_to_end(&mut data).await?;

            // 解压缩（如果启用）
            let snapshot_data = if self.config.compression {
                decompress_data(&data)?
            } else {
                data
            };

            self.deserialize_index(index, &snapshot_data)?;
        }

        // 2. 重放 WAL
        if self.wal_path.exists() {
            let file = tokio_fs::File::open(&self.wal_path).await?;
            let reader = BufReader::new(file.into_std().await);

            for line in reader.lines().map_while(|r| r.ok()) {
                if let Ok(decoded) = general_purpose::STANDARD.decode(&line) {
                    if let Ok((change, _)) =
                        decode_from_slice::<IndexChange, _>(&decoded, standard())
                    {
                        self.apply_change(index, change)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// 应用变更到索引
    fn apply_change(&self, index: &mut Index, change: IndexChange) -> Result<()> {
        match change {
            IndexChange::Add { doc_id, content } => {
                index.add(doc_id, &content, false)?;
            }
            IndexChange::Remove { doc_id } => {
                index.remove(doc_id, false)?;
            }
            IndexChange::Update { doc_id, content } => {
                // 先移除再添加
                let _ = index.remove(doc_id, false);
                index.add(doc_id, &content, false)?;
            }
        }
        Ok(())
    }

    /// 序列化索引
    fn serialize_index(&self, index: &Index) -> Result<Vec<u8>> {
        use crate::serialize::SerializeConfig;
        let config = SerializeConfig::default();
        let export_data = index.export(&config)?;
        Ok(encode_to_vec(&export_data, standard())?)
    }

    /// 反序列化索引
    fn deserialize_index(&self, index: &mut Index, data: &[u8]) -> Result<()> {
        use crate::serialize::{IndexExportData, SerializeConfig};
        let (export_data, _): (IndexExportData, usize) = decode_from_slice(data, standard())?;
        let config = SerializeConfig::default();
        index.import(export_data, &config)?;
        Ok(())
    }

    /// 清空 WAL 和快照
    pub async fn clear(&self) -> Result<()> {
        let _ = tokio_fs::remove_file(&self.wal_path).await;
        let _ = tokio_fs::remove_file(&self.snapshot_path).await;
        Ok(())
    }

    /// 获取 WAL 大小
    pub fn wal_size(&self) -> usize {
        self.wal_size.load(Ordering::Relaxed)
    }

    /// 获取快照大小
    pub async fn snapshot_size(&self) -> Result<u64> {
        if self.snapshot_path.exists() {
            Ok(tokio_fs::metadata(&self.snapshot_path).await?.len())
        } else {
            Ok(0)
        }
    }
}

/// 压缩数据
fn compress_data(data: &[u8], level: i32) -> Result<Vec<u8>> {
    zstd::stream::encode_all(data, level)
        .map_err(|e| InversearchError::Serialization(format!("Compression error: {}", e)))
}

/// 解压缩数据
fn decompress_data(data: &[u8]) -> Result<Vec<u8>> {
    zstd::stream::decode_all(data)
        .map_err(|e| InversearchError::Serialization(format!("Decompression error: {}", e)))
}

// ============== WAL Storage 实现 ==============

use crate::r#type::{EnrichedSearchResults, SearchResults};
use crate::storage::common::{StorageInfo, StorageInterface};
use std::collections::HashMap;
use tokio::sync::RwLock;

/// WAL 存储
pub struct WALStorage {
    wal_manager: WALManager,
    documents: RwLock<HashMap<DocId, String>>,
    is_open: RwLock<bool>,
}

impl WALStorage {
    /// 创建新的 WAL 存储
    pub async fn new(config: WALConfig) -> Result<Self> {
        let wal_manager = WALManager::new(config).await?;

        Ok(Self {
            wal_manager,
            documents: RwLock::new(HashMap::new()),
            is_open: RwLock::new(false),
        })
    }

    /// 创建快照
    pub async fn create_snapshot(&self, index: &Index) -> Result<()> {
        self.wal_manager.create_snapshot(index).await
    }
}

#[async_trait::async_trait]
impl StorageInterface for WALStorage {
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
        self.documents.write().await.clear();
        self.wal_manager.clear().await?;
        *self.is_open.write().await = false;
        Ok(())
    }

    async fn commit(&self, index: &Index, _replace: bool, _append: bool) -> Result<()> {
        // 使用 WAL 创建快照
        self.wal_manager.create_snapshot(index).await
    }

    async fn get(
        &self,
        _key: &str,
        _ctx: Option<&str>,
        _limit: usize,
        _offset: usize,
        _resolve: bool,
        _enrich: bool,
    ) -> Result<SearchResults> {
        // WAL 存储需要通过加载索引来获取数据
        // 这里简化处理，返回空结果
        // 实际应用中应该维护一个内存索引
        Ok(Vec::new())
    }

    async fn enrich(&self, ids: &[DocId]) -> Result<EnrichedSearchResults> {
        let documents = self.documents.read().await;
        let mut results = Vec::new();

        for &id in ids {
            if let Some(content) = documents.get(&id) {
                results.push(crate::r#type::EnrichedSearchResult {
                    id,
                    doc: Some(serde_json::json!({
                        "content": content,
                        "id": id
                    })),
                    highlight: None,
                });
            }
        }

        Ok(results)
    }

    async fn has(&self, id: DocId) -> Result<bool> {
        Ok(self.documents.read().await.contains_key(&id))
    }

    async fn remove(&self, ids: &[DocId]) -> Result<()> {
        let mut documents = self.documents.write().await;
        for &id in ids {
            documents.remove(&id);
            self.wal_manager
                .record_change(IndexChange::Remove { doc_id: id })
                .await?;
        }
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        self.documents.write().await.clear();
        self.wal_manager.clear().await?;
        Ok(())
    }

    async fn info(&self) -> Result<StorageInfo> {
        let wal_size = self.wal_manager.wal_size() as u64;
        let snapshot_size = self.wal_manager.snapshot_size().await?;

        Ok(StorageInfo {
            name: "WALStorage".to_string(),
            version: "0.1.0".to_string(),
            size: wal_size + snapshot_size,
            document_count: self.documents.read().await.len(),
            index_count: 0,
            is_connected: *self.is_open.read().await,
        })
    }
}
