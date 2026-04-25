//! Redis storage implementation (using bb8 connection pool optimized version)
//!
//! Major improvements:
//! - Managing Multiple Redis Connections with bb8 Connection Pooling
//! - Dynamically resize Pipeline batches
//! - Optimized memory usage calculation
//! - Enhanced error classification statistics

use crate::error::Result;
use crate::r#type::{DocId, EnrichedSearchResults, SearchResults};
use crate::storage::common::r#trait::StorageInterface;
use crate::storage::common::types::StorageInfo;
use crate::Index;
use bb8::Pool;
use redis::{aio::MultiplexedConnection, Client as RedisClient};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Redis Connection Pool Manager
pub struct RedisConnectionManager {
    client: RedisClient,
}

impl RedisConnectionManager {
    pub fn new(client: RedisClient) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl bb8::ManageConnection for RedisConnectionManager {
    type Connection = MultiplexedConnection;
    type Error = redis::RedisError;

    async fn connect(&self) -> std::result::Result<Self::Connection, Self::Error> {
        self.client.get_multiplexed_async_connection().await
    }

    async fn is_valid(&self, conn: &mut Self::Connection) -> std::result::Result<(), Self::Error> {
        redis::cmd("PING")
            .query_async(conn)
            .await
            .map(|_: String| ())
    }

    fn has_broken(&self, _: &mut Self::Connection) -> bool {
        false
    }
}

#[derive(Debug, Clone)]
pub struct RedisStorageConfig {
    pub url: String,
    pub pool_size: u32,
    pub connection_timeout: Duration,
    pub key_prefix: String,
    pub min_idle: Option<u32>,
    pub max_lifetime: Option<Duration>,
    pub connection_timeout_bb8: Duration,
}

impl Default for RedisStorageConfig {
    fn default() -> Self {
        Self {
            url: "redis://127.0.0.1:6379".to_string(),
            pool_size: 10,
            connection_timeout: Duration::from_secs(5),
            key_prefix: "inversearch".to_string(),
            min_idle: Some(2),
            max_lifetime: Some(Duration::from_secs(60)),
            connection_timeout_bb8: Duration::from_secs(10),
        }
    }
}

/// Error Type Statistics
#[derive(Debug, Default)]
pub struct ErrorStats {
    pub connection_errors: AtomicU64,
    pub serialization_errors: AtomicU64,
    pub deserialization_errors: AtomicU64,
    pub timeout_errors: AtomicU64,
    pub other_errors: AtomicU64,
}

pub struct RedisStorage {
    pool: Pool<RedisConnectionManager>,
    config: RedisStorageConfig,
    key_prefix: String,
    memory_usage: Arc<AtomicUsize>,
    operation_count: Arc<AtomicU64>,
    total_latency: Arc<AtomicU64>,
    error_stats: Arc<ErrorStats>,
    last_operation_time: Arc<std::sync::Mutex<Option<Instant>>>,
}

impl RedisStorage {
    pub async fn new(config: RedisStorageConfig) -> Result<Self> {
        let key_prefix = config.key_prefix.clone();
        let client = RedisClient::open(config.url.as_str())
            .map_err(|e| crate::error::StorageError::Connection(e.to_string()))?;

        let manager = RedisConnectionManager::new(client);

        let pool = Pool::builder()
            .max_size(config.pool_size)
            .min_idle(config.min_idle)
            .max_lifetime(config.max_lifetime)
            .connection_timeout(config.connection_timeout_bb8)
            .build(manager)
            .await
            .map_err(|e| crate::error::StorageError::Connection(e.to_string()))?;

        // Verifying Connection Pool Availability
        {
            let mut conn = pool
                .get()
                .await
                .map_err(|e| crate::error::StorageError::Connection(e.to_string()))?;
            let _: String = redis::cmd("PING")
                .query_async(&mut *conn)
                .await
                .map_err(|e| crate::error::StorageError::Connection(e.to_string()))?;
        }

        Ok(Self {
            pool,
            config,
            key_prefix,
            memory_usage: Arc::new(AtomicUsize::new(0)),
            operation_count: Arc::new(AtomicU64::new(0)),
            total_latency: Arc::new(AtomicU64::new(0)),
            error_stats: Arc::new(ErrorStats::default()),
            last_operation_time: Arc::new(std::sync::Mutex::new(None)),
        })
    }

    /// Get the Redis storage configuration
    pub fn config(&self) -> &RedisStorageConfig {
        &self.config
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

    async fn get_connection(&self) -> Result<bb8::PooledConnection<'_, RedisConnectionManager>> {
        self.pool
            .get()
            .await
            .map_err(|e| {
                self.error_stats
                    .connection_errors
                    .fetch_add(1, Ordering::Relaxed);
                crate::error::StorageError::Connection(e.to_string())
            })
            .map_err(Into::into)
    }

    async fn scan_keys(&self, pattern: &str) -> Result<Vec<String>> {
        let mut conn = self.get_connection().await?;
        let mut cursor = 0u64;
        let mut all_keys = Vec::new();

        // Dynamically adjusting the COUNT value
        let count = if pattern.contains("*") { 1000 } else { 100 };

        loop {
            let (next_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(pattern)
                .arg("COUNT")
                .arg(count)
                .query_async(&mut *conn)
                .await
                .map_err(|e| {
                    self.error_stats
                        .connection_errors
                        .fetch_add(1, Ordering::Relaxed);
                    crate::error::StorageError::Connection(e.to_string())
                })?;

            all_keys.extend(keys);
            cursor = next_cursor;

            if cursor == 0 {
                break;
            }
        }

        Ok(all_keys)
    }

    /// Calculate the optimal batch size
    fn calculate_batch_size(&self, total_items: usize) -> usize {
        if total_items > 10000 {
            2000
        } else if total_items > 1000 {
            1000
        } else {
            500
        }
    }

    /// Optimized memory usage calculation
    async fn update_memory_usage(&self) -> Result<()> {
        let mut conn = self.get_connection().await?;

        // Use the Redis INFO memory command to get overall memory usage
        let info: String = redis::cmd("INFO")
            .arg("memory")
            .query_async(&mut *conn)
            .await
            .map_err(|e| {
                self.error_stats
                    .connection_errors
                    .fetch_add(1, Ordering::Relaxed);
                crate::error::StorageError::Connection(e.to_string())
            })?;

        // Parsing the used_memory field
        let memory = self.parse_redis_memory_info(&info);
        self.memory_usage.store(memory, Ordering::Relaxed);
        Ok(())
    }

    fn parse_redis_memory_info(&self, info: &str) -> usize {
        for line in info.lines() {
            if line.starts_with("used_memory:") {
                if let Some(value) = line.strip_prefix("used_memory:") {
                    return value.trim().parse().unwrap_or(0);
                }
            }
        }
        0
    }

    fn record_error(&self, error_type: &str) {
        match error_type {
            "connection" => {
                self.error_stats
                    .connection_errors
                    .fetch_add(1, Ordering::Relaxed);
            }
            "serialization" => {
                self.error_stats
                    .serialization_errors
                    .fetch_add(1, Ordering::Relaxed);
            }
            "deserialization" => {
                self.error_stats
                    .deserialization_errors
                    .fetch_add(1, Ordering::Relaxed);
            }
            "timeout" => {
                self.error_stats
                    .timeout_errors
                    .fetch_add(1, Ordering::Relaxed);
            }
            _ => {
                self.error_stats
                    .other_errors
                    .fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

#[async_trait::async_trait]
impl StorageInterface for RedisStorage {
    async fn mount(&self, _index: &Index) -> Result<()> {
        Ok(())
    }

    async fn open(&self) -> Result<()> {
        Ok(())
    }

    async fn close(&self) -> Result<()> {
        Ok(())
    }

    async fn destroy(&self) -> Result<()> {
        let pattern = format!("{}:*", self.key_prefix);
        let keys = self.scan_keys(&pattern).await?;

        if !keys.is_empty() {
            let mut conn = self.get_connection().await?;
            let _: () = redis::cmd("DEL")
                .arg(keys.as_slice())
                .query_async(&mut *conn)
                .await
                .map_err(|e| {
                    self.record_error("connection");
                    crate::error::StorageError::Connection(e.to_string())
                })?;
        }

        Ok(())
    }

    async fn commit(&self, index: &Index, _replace: bool, _append: bool) -> Result<()> {
        let start_time = self.record_operation_start();

        let mut conn = self.get_connection().await?;

        // Collect all index entries
        let mut index_items: Vec<(String, String)> = Vec::new();

        for doc_ids in index.map.index.values() {
            for (term_str, ids) in doc_ids {
                let key = self.make_index_key(term_str);
                let serialized = serde_json::to_string(ids).map_err(|e| {
                    self.record_error("serialization");
                    crate::error::StorageError::Serialization(e.to_string())
                })?;
                index_items.push((key, serialized));
            }
        }

        for ctx_map in index.ctx.index.values() {
            for (ctx_term, doc_ids) in ctx_map {
                let key = self.make_context_key("default", ctx_term);
                let serialized = serde_json::to_string(doc_ids).map_err(|e| {
                    self.record_error("serialization");
                    crate::error::StorageError::Serialization(e.to_string())
                })?;
                index_items.push((key, serialized));
            }
        }

        // Pipeline with Dynamic Batch Sizing
        let batch_size = self.calculate_batch_size(index_items.len());

        for chunk in index_items.chunks(batch_size) {
            let mut batch_pipe = redis::pipe();
            for (key, value) in chunk {
                batch_pipe.cmd("SET").arg(key).arg(value);
            }
            let _: () = batch_pipe.query_async(&mut *conn).await.map_err(|e| {
                self.record_error("connection");
                crate::error::StorageError::Connection(e.to_string())
            })?;
        }

        // Update memory usage
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
            .query_async(&mut *conn)
            .await
            .map_err(|e| {
                self.record_error("connection");
                crate::error::StorageError::Connection(e.to_string())
            })?;

        if serialized.is_empty() {
            return Ok(Vec::new());
        }

        let doc_ids: Vec<DocId> = serde_json::from_str(&serialized).map_err(|e| {
            self.record_error("deserialization");
            crate::error::StorageError::Deserialization(e.to_string())
        })?;

        let start = offset.min(doc_ids.len());
        let end = if limit > 0 {
            (start + limit).min(doc_ids.len())
        } else {
            doc_ids.len()
        };

        Ok(doc_ids[start..end].to_vec())
    }

    async fn enrich(&self, ids: &[DocId]) -> Result<EnrichedSearchResults> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let keys: Vec<String> = ids.iter().map(|&id| self.make_doc_key(id)).collect();
        let mut conn = self.get_connection().await?;

        let serialized_list: Vec<String> = redis::cmd("MGET")
            .arg(&keys)
            .query_async(&mut *conn)
            .await
            .map_err(|e| {
                self.record_error("connection");
                crate::error::StorageError::Connection(e.to_string())
            })?;

        let mut results = Vec::new();
        for (i, serialized) in serialized_list.into_iter().enumerate() {
            if !serialized.is_empty() {
                let doc: serde_json::Value = serde_json::from_str(&serialized).map_err(|e| {
                    self.record_error("deserialization");
                    crate::error::StorageError::Deserialization(e.to_string())
                })?;
                results.push(crate::r#type::EnrichedSearchResult {
                    id: ids[i],
                    doc: Some(doc),
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
            .query_async(&mut *conn)
            .await
            .map_err(|e| {
                self.record_error("connection");
                crate::error::StorageError::Connection(e.to_string())
            })?;

        Ok(exists)
    }

    async fn remove(&self, ids: &[DocId]) -> Result<()> {
        if ids.is_empty() {
            return Ok(());
        }

        let keys: Vec<String> = ids.iter().map(|&id| self.make_doc_key(id)).collect();
        let mut conn = self.get_connection().await?;

        let _: () = redis::cmd("DEL")
            .arg(&keys)
            .query_async(&mut *conn)
            .await
            .map_err(|e| {
                self.record_error("connection");
                crate::error::StorageError::Connection(e.to_string())
            })?;

        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        self.destroy().await
    }

    async fn info(&self) -> Result<StorageInfo> {
        let pattern = format!("{}:*", self.key_prefix);
        let _keys = self.scan_keys(&pattern).await?;

        let doc_pattern = format!("{}:doc:*", self.key_prefix);
        let doc_keys = self.scan_keys(&doc_pattern).await?;

        let index_pattern = format!("{}:index:*", self.key_prefix);
        let index_keys = self.scan_keys(&index_pattern).await?;

        // Using Memory Usage
        let total_size = self.memory_usage.load(Ordering::Relaxed) as u64;

        Ok(StorageInfo {
            name: "RedisStorage".to_string(),
            version: "0.2.0".to_string(),
            size: total_size,
            document_count: doc_keys.len(),
            index_count: index_keys.len(),
            is_connected: true,
        })
    }
}

impl RedisStorage {
    /// Batch delete documents (optimized version, using pipeline)
    pub async fn remove_batch(&self, ids: &[DocId]) -> Result<()> {
        if ids.is_empty() {
            return Ok(());
        }

        let keys: Vec<String> = ids.iter().map(|&id| self.make_doc_key(id)).collect();
        let mut conn = self.get_connection().await?;

        let _: () = redis::cmd("DEL")
            .arg(&keys)
            .query_async(&mut *conn)
            .await
            .map_err(|e| {
                self.record_error("connection");
                crate::error::StorageError::Connection(e.to_string())
            })?;

        Ok(())
    }

    /// health checkup
    pub async fn health_check(&self) -> Result<bool> {
        match self.pool.get().await {
            Ok(mut conn) => {
                let result: std::result::Result<String, redis::RedisError> =
                    redis::cmd("PING").query_async(&mut *conn).await;
                Ok(result.is_ok())
            }
            Err(_) => Ok(false),
        }
    }

    /// Getting Memory Usage
    pub fn get_memory_usage(&self) -> usize {
        self.memory_usage.load(Ordering::Relaxed)
    }

    /// Get Operation Statistics
    pub fn get_operation_stats(&self) -> StorageMetrics {
        let operation_count = self.operation_count.load(Ordering::Relaxed) as usize;
        let total_latency = self.total_latency.load(Ordering::Relaxed) as usize;
        let avg_latency = if operation_count > 0 {
            total_latency / operation_count
        } else {
            0
        };

        StorageMetrics {
            operation_count,
            average_latency: avg_latency,
            memory_usage: self.get_memory_usage(),
            error_count: self.get_total_errors(),
            connection_errors: self.error_stats.connection_errors.load(Ordering::Relaxed) as usize,
            serialization_errors: self
                .error_stats
                .serialization_errors
                .load(Ordering::Relaxed) as usize,
            deserialization_errors: self
                .error_stats
                .deserialization_errors
                .load(Ordering::Relaxed) as usize,
        }
    }

    fn get_total_errors(&self) -> usize {
        (self.error_stats.connection_errors.load(Ordering::Relaxed)
            + self
                .error_stats
                .serialization_errors
                .load(Ordering::Relaxed)
            + self
                .error_stats
                .deserialization_errors
                .load(Ordering::Relaxed)
            + self.error_stats.timeout_errors.load(Ordering::Relaxed)
            + self.error_stats.other_errors.load(Ordering::Relaxed)) as usize
    }

    /// Record operation start time (internal use)
    fn record_operation_start(&self) -> Instant {
        let start_time = Instant::now();
        if let Ok(mut last_op) = self.last_operation_time.lock() {
            *last_op = Some(start_time);
        }
        start_time
    }

    /// Record operation completion (internal use)
    fn record_operation_completion(&self, start_time: Instant) {
        let latency = start_time.elapsed().as_micros() as u64;
        self.operation_count.fetch_add(1, Ordering::Relaxed);
        self.total_latency.fetch_add(latency, Ordering::Relaxed);
    }
}

/// Storage Performance Metrics
#[derive(Debug, Clone, Default)]
pub struct StorageMetrics {
    pub operation_count: usize,
    pub average_latency: usize, // microsecond
    pub memory_usage: usize,
    pub error_count: usize,
    pub connection_errors: usize,
    pub serialization_errors: usize,
    pub deserialization_errors: usize,
}

impl StorageMetrics {
    /// Creating empty indicators
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset all indicators
    pub fn reset(&mut self) {
        self.operation_count = 0;
        self.average_latency = 0;
        self.memory_usage = 0;
        self.error_count = 0;
        self.connection_errors = 0;
        self.serialization_errors = 0;
        self.deserialization_errors = 0;
    }
}
