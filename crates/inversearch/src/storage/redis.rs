use crate::error::Result;
use crate::r#type::{DocId, EnrichedSearchResults, SearchResults};
use crate::storage::common::r#trait::StorageInterface;
use crate::storage::common::types::StorageInfo;
use crate::Index;
use redis::{aio::MultiplexedConnection, Client as RedisClient};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct RedisStorageConfig {
    pub url: String,
    pub pool_size: usize,
    pub connection_timeout: Duration,
    pub key_prefix: String,
}

impl Default for RedisStorageConfig {
    fn default() -> Self {
        Self {
            url: "redis://127.0.0.1:6379".to_string(),
            pool_size: 10,
            connection_timeout: Duration::from_secs(5),
            key_prefix: "inversearch".to_string(),
        }
    }
}

pub struct RedisStorage {
    client: RedisClient,
    #[allow(dead_code)]
    config: RedisStorageConfig,
    key_prefix: String,
    memory_usage: Arc<AtomicUsize>,
    operation_count: Arc<AtomicUsize>,
    total_latency: Arc<AtomicUsize>,
    last_operation_time: Arc<std::sync::Mutex<Option<Instant>>>,
}

impl RedisStorage {
    pub async fn new(config: RedisStorageConfig) -> Result<Self> {
        let key_prefix = config.key_prefix.clone();
        let client = RedisClient::open(config.url.as_str())
            .map_err(|e| crate::error::StorageError::Connection(e.to_string()))?;

        let mut conn = client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| crate::error::StorageError::Connection(e.to_string()))?;

        let _: String = redis::cmd("PING")
            .query_async(&mut conn)
            .await
            .map_err(|e| crate::error::StorageError::Connection(e.to_string()))?;

        Ok(Self {
            client,
            config,
            key_prefix,
            memory_usage: Arc::new(AtomicUsize::new(0)),
            operation_count: Arc::new(AtomicUsize::new(0)),
            total_latency: Arc::new(AtomicUsize::new(0)),
            last_operation_time: Arc::new(std::sync::Mutex::new(None)),
        })
    }

    fn make_key(&self, key: &str) -> String {
        format!("{}:{}", self.key_prefix, key)
    }

    fn make_index_key(&self, term: &str) -> String {
        self.make_key(&format!("index:{}", term))
    }

    fn make_context_key(&self, context: &str, term: &str) -> String {
        self.make_key(&format!("ctx:{}:{}", context, term))
    }

    fn make_doc_key(&self, doc_id: DocId) -> String {
        self.make_key(&format!("doc:{}", doc_id))
    }

    async fn get_connection(&self) -> Result<MultiplexedConnection> {
        self.client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| crate::error::StorageError::Connection(e.to_string()).into())
    }

    /// 批量提交（使用 MSET）
    pub async fn commit_batch(&mut self, index: &Index, batch_size: usize) -> Result<()> {
        let mut conn = self.get_connection().await?;

        // 收集所有索引项
        let mut index_items: Vec<(String, String)> = Vec::new();

        for doc_ids in index.map.index.values() {
            for (term_str, ids) in doc_ids {
                let key = self.make_index_key(term_str);
                let serialized = serde_json::to_string(ids)
                    .map_err(|e| crate::error::StorageError::Serialization(e.to_string()))?;
                index_items.push((key, serialized));
            }
        }

        for ctx_map in index.ctx.index.values() {
            for (ctx_term, doc_ids) in ctx_map {
                let key = self.make_context_key("default", ctx_term);
                let serialized = serde_json::to_string(doc_ids)
                    .map_err(|e| crate::error::StorageError::Serialization(e.to_string()))?;
                index_items.push((key, serialized));
            }
        }

        // 使用 MSET 批量设置
        for chunk in index_items.chunks(batch_size) {
            let mut mset_args: Vec<(&str, &str)> = Vec::new();
            for (key, value) in chunk {
                mset_args.push((key.as_str(), value.as_str()));
            }

            let _: () = redis::cmd("MSET")
                .arg(mset_args.as_slice())
                .query_async(&mut conn)
                .await
                .map_err(|e| crate::error::StorageError::Connection(e.to_string()))?;
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl StorageInterface for RedisStorage {
    async fn mount(&mut self, _index: &Index) -> Result<()> {
        Ok(())
    }

    async fn open(&mut self) -> Result<()> {
        Ok(())
    }

    async fn close(&mut self) -> Result<()> {
        Ok(())
    }

    async fn destroy(&mut self) -> Result<()> {
        let mut conn = self.get_connection().await?;

        let pattern = format!("{}:*", self.key_prefix);
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(&pattern)
            .query_async(&mut conn)
            .await
            .map_err(|e| crate::error::StorageError::Connection(e.to_string()))?;

        if !keys.is_empty() {
            let _: () = redis::cmd("DEL")
                .arg(keys.as_slice())
                .query_async(&mut conn)
                .await
                .map_err(|e| crate::error::StorageError::Connection(e.to_string()))?;
        }

        Ok(())
    }

    async fn commit(&mut self, index: &Index, _replace: bool, _append: bool) -> Result<()> {
        let start_time = self.record_operation_start();

        let mut conn = self.get_connection().await?;

        // 收集所有索引项
        let mut index_items: Vec<(String, String)> = Vec::new();

        for doc_ids in index.map.index.values() {
            for (term_str, ids) in doc_ids {
                let key = self.make_index_key(term_str);
                let serialized = serde_json::to_string(ids)
                    .map_err(|e| crate::error::StorageError::Serialization(e.to_string()))?;
                index_items.push((key, serialized));
            }
        }

        for ctx_map in index.ctx.index.values() {
            for (ctx_term, doc_ids) in ctx_map {
                let key = self.make_context_key("default", ctx_term);
                let serialized = serde_json::to_string(doc_ids)
                    .map_err(|e| crate::error::StorageError::Serialization(e.to_string()))?;
                index_items.push((key, serialized));
            }
        }

        // 使用 Pipeline 批量提交（每批 1000 个）
        const BATCH_SIZE: usize = 1000;

        for chunk in index_items.chunks(BATCH_SIZE) {
            let mut batch_pipe = redis::pipe();
            for (key, value) in chunk {
                batch_pipe.cmd("SET").arg(key).arg(value);
            }
            let _: () = batch_pipe
                .query_async(&mut conn)
                .await
                .map_err(|e| crate::error::StorageError::Connection(e.to_string()))?;
        }

        // 更新内存使用量
        self.update_memory_usage().await?;
        self.record_operation_completion(start_time);

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
        let mut conn = self.get_connection().await?;

        let redis_key = if let Some(ctx_key) = ctx {
            self.make_context_key(ctx_key, key)
        } else {
            self.make_index_key(key)
        };

        let serialized: String = redis::cmd("GET")
            .arg(&redis_key)
            .query_async(&mut conn)
            .await
            .map_err(|e| crate::error::StorageError::Connection(e.to_string()))?;

        if serialized.is_empty() {
            return Ok(Vec::new());
        }

        let doc_ids: Vec<DocId> = serde_json::from_str(&serialized)
            .map_err(|e| crate::error::StorageError::Deserialization(e.to_string()))?;

        let start = offset.min(doc_ids.len());
        let end = if limit > 0 {
            (start + limit).min(doc_ids.len())
        } else {
            doc_ids.len()
        };

        Ok(doc_ids[start..end].to_vec())
    }

    async fn enrich(&self, ids: &[DocId]) -> Result<EnrichedSearchResults> {
        let mut conn = self.get_connection().await?;
        let mut results = Vec::new();

        for &id in ids {
            let key = self.make_doc_key(id);
            let serialized: String = redis::cmd("GET")
                .arg(&key)
                .query_async(&mut conn)
                .await
                .map_err(|e| crate::error::StorageError::Connection(e.to_string()))?;

            if !serialized.is_empty() {
                results.push(crate::r#type::EnrichedSearchResult {
                    id,
                    doc: Some(
                        serde_json::from_str(&serialized).map_err(|e| {
                            crate::error::StorageError::Deserialization(e.to_string())
                        })?,
                    ),
                    highlight: None,
                });
            }
        }

        Ok(results)
    }

    async fn has(&self, id: DocId) -> Result<bool> {
        let mut conn = self.get_connection().await?;
        let key = self.make_doc_key(id);

        let exists: bool = redis::cmd("EXISTS")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .map_err(|e| crate::error::StorageError::Connection(e.to_string()))?;

        Ok(exists)
    }

    async fn remove(&mut self, ids: &[DocId]) -> Result<()> {
        let mut conn = self.get_connection().await?;

        for &id in ids {
            let key = self.make_doc_key(id);
            let _: () = redis::cmd("DEL")
                .arg(&key)
                .query_async(&mut conn)
                .await
                .map_err(|e| crate::error::StorageError::Connection(e.to_string()))?;
        }

        Ok(())
    }

    async fn clear(&mut self) -> Result<()> {
        self.destroy().await
    }

    async fn info(&self) -> Result<StorageInfo> {
        let mut conn = self.get_connection().await?;

        let pattern = format!("{}:*", self.key_prefix);
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(&pattern)
            .query_async(&mut conn)
            .await
            .map_err(|e| crate::error::StorageError::Connection(e.to_string()))?;

        // 计算文档数量
        let doc_pattern = format!("{}:doc:*", self.key_prefix);
        let doc_keys: Vec<String> = redis::cmd("KEYS")
            .arg(&doc_pattern)
            .query_async(&mut conn)
            .await
            .map_err(|e| crate::error::StorageError::Connection(e.to_string()))?;

        // 计算索引项数量
        let index_pattern = format!("{}:index:*", self.key_prefix);
        let index_keys: Vec<String> = redis::cmd("KEYS")
            .arg(&index_pattern)
            .query_async(&mut conn)
            .await
            .map_err(|e| crate::error::StorageError::Connection(e.to_string()))?;

        // 计算总大小
        let mut total_size = 0;
        for key in &keys {
            let size: usize = redis::cmd("STRLEN")
                .arg(key)
                .query_async(&mut conn)
                .await
                .map_err(|e| crate::error::StorageError::Connection(e.to_string()))?;
            total_size += size;
        }

        Ok(StorageInfo {
            name: "RedisStorage".to_string(),
            version: "0.1.0".to_string(),
            size: total_size as u64,
            document_count: doc_keys.len(),
            index_count: index_keys.len(),
            is_connected: true,
        })
    }
}

impl RedisStorage {
    /// 批量删除文档（优化版本）
    pub async fn remove_batch(&mut self, ids: &[DocId]) -> Result<()> {
        if ids.is_empty() {
            return Ok(());
        }

        let mut conn = self.get_connection().await?;
        let mut keys_to_delete = Vec::new();

        for &id in ids {
            keys_to_delete.push(self.make_doc_key(id));
        }

        // 使用 DEL 命令批量删除
        let _: () = redis::cmd("DEL")
            .arg(keys_to_delete.as_slice())
            .query_async(&mut conn)
            .await
            .map_err(|e| crate::error::StorageError::Connection(e.to_string()))?;

        Ok(())
    }

    /// 连接池管理
    pub async fn get_pooled_connection(&self) -> Result<MultiplexedConnection> {
        self.client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| crate::error::StorageError::Connection(e.to_string()).into())
    }

    /// 健康检查
    pub async fn health_check(&self) -> Result<bool> {
        match self.get_connection().await {
            Ok(mut conn) => {
                let result: String = redis::cmd("PING")
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| crate::error::StorageError::Connection(e.to_string()))?;
                Ok(result == "PONG")
            }
            Err(_) => Ok(false),
        }
    }

    /// 获取内存使用情况
    pub fn get_memory_usage(&self) -> usize {
        self.memory_usage.load(Ordering::Relaxed)
    }

    /// 获取操作统计
    pub fn get_operation_stats(&self) -> StorageMetrics {
        let operation_count = self.operation_count.load(Ordering::Relaxed);
        let total_latency = self.total_latency.load(Ordering::Relaxed);
        let avg_latency = if operation_count > 0 {
            total_latency / operation_count
        } else {
            0
        };

        StorageMetrics {
            operation_count,
            average_latency: avg_latency,
            memory_usage: self.get_memory_usage(),
            error_count: 0, // 需要额外的错误计数器
        }
    }

    /// 记录操作开始时间（内部使用）
    fn record_operation_start(&self) -> Instant {
        let start_time = Instant::now();
        if let Ok(mut last_op) = self.last_operation_time.lock() {
            *last_op = Some(start_time);
        }
        start_time
    }

    /// 记录操作完成（内部使用）
    fn record_operation_completion(&self, start_time: Instant) {
        let latency = start_time.elapsed().as_micros() as usize;
        self.operation_count.fetch_add(1, Ordering::Relaxed);
        self.total_latency.fetch_add(latency, Ordering::Relaxed);
    }

    /// 更新内存使用量估计
    async fn update_memory_usage(&self) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let pattern = format!("{}:*", self.key_prefix);
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(&pattern)
            .query_async(&mut conn)
            .await
            .map_err(|e| crate::error::StorageError::Connection(e.to_string()))?;

        let mut total_size = 0;
        for key in &keys {
            let size: usize = redis::cmd("STRLEN")
                .arg(key)
                .query_async(&mut conn)
                .await
                .map_err(|e| crate::error::StorageError::Connection(e.to_string()))?;
            total_size += size;
        }

        self.memory_usage.store(total_size, Ordering::Relaxed);
        Ok(())
    }
}

/// 存储性能指标
#[derive(Debug, Clone, Default)]
pub struct StorageMetrics {
    pub operation_count: usize,
    pub average_latency: usize, // 微秒
    pub memory_usage: usize,
    pub error_count: usize,
}

impl StorageMetrics {
    /// 创建空的指标
    pub fn new() -> Self {
        Self::default()
    }

    /// 重置所有指标
    pub fn reset(&mut self) {
        self.operation_count = 0;
        self.average_latency = 0;
        self.memory_usage = 0;
        self.error_count = 0;
    }
}
