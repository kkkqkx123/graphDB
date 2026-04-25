//! Redis storage implementation (using bb8 connection pool optimized version)
//!
//! Storing BM25 word frequency statistics using Redis
//! Data structure:
//! - BM25:tf:{term} -> Hash { doc_id: tf_value }
//! - BM25:df:{term} -> String (df_value)
//!
//! Major improvements:
//! - Managing Multiple Redis Connections with bb8 Connection Pooling
//! - TF Atomic accumulation operation (HINCRBYFLOAT)
//! - Optimized memory usage calculation
//! - Enhanced error classification statistics
//! - Dynamic batch resizing

use crate::error::{Bm25Error, Result};
use crate::storage::common::r#trait::{Bm25Stats, StorageInterface};
use crate::storage::common::types::StorageInfo;
use bb8::Pool;
use redis::{aio::MultiplexedConnection, Client as RedisClient};
use std::collections::HashMap;
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

/// Redis Storage Configuration
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
            key_prefix: "bm25".to_string(),
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

/// Redis Storage Implementation
pub struct RedisStorage {
    pool: Pool<RedisConnectionManager>,
    key_prefix: String,
    memory_usage: Arc<AtomicUsize>,
    operation_count: Arc<AtomicU64>,
    total_latency: Arc<AtomicU64>,
    error_stats: Arc<ErrorStats>,
}

impl RedisStorage {
    pub async fn new(config: RedisStorageConfig) -> Result<Self> {
        let key_prefix = config.key_prefix.clone();
        let client = RedisClient::open(config.url.as_str())
            .map_err(|e| Bm25Error::StorageError(e.to_string()))?;

        let manager = RedisConnectionManager::new(client);

        let pool = Pool::builder()
            .max_size(config.pool_size)
            .min_idle(config.min_idle)
            .max_lifetime(config.max_lifetime)
            .connection_timeout(config.connection_timeout_bb8)
            .build(manager)
            .await
            .map_err(|e| Bm25Error::StorageError(e.to_string()))?;

        // Verifying Connection Pool Availability
        {
            let mut conn = pool
                .get()
                .await
                .map_err(|e| Bm25Error::StorageError(e.to_string()))?;
            let _: String = redis::cmd("PING")
                .query_async(&mut *conn)
                .await
                .map_err(|e| Bm25Error::StorageError(e.to_string()))?;
        }

        Ok(Self {
            pool,
            key_prefix,
            memory_usage: Arc::new(AtomicUsize::new(0)),
            operation_count: Arc::new(AtomicU64::new(0)),
            total_latency: Arc::new(AtomicU64::new(0)),
            error_stats: Arc::new(ErrorStats::default()),
        })
    }

    fn make_key(&self, key: &str) -> String {
        format!("{}:{}", self.key_prefix, key)
    }

    fn make_tf_key(&self, term: &str) -> String {
        self.make_key(&format!("tf:{}", term))
    }

    fn make_df_key(&self, term: &str) -> String {
        self.make_key(&format!("df:{}", term))
    }

    async fn get_connection(&self) -> Result<bb8::PooledConnection<'_, RedisConnectionManager>> {
        self.pool.get().await.map_err(|e| {
            self.error_stats
                .connection_errors
                .fetch_add(1, Ordering::Relaxed);
            Bm25Error::StorageError(e.to_string())
        })
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
                    Bm25Error::StorageError(e.to_string())
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
    #[allow(dead_code)]
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
                Bm25Error::StorageError(e.to_string())
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
    async fn init(&mut self) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let _: () = redis::cmd("PING")
            .query_async(&mut *conn)
            .await
            .map_err(|e| {
                self.record_error("connection");
                Bm25Error::StorageError(e.to_string())
            })?;
        Ok(())
    }

    async fn close(&mut self) -> Result<()> {
        Ok(())
    }

    async fn commit_stats(&mut self, term: &str, tf: f32, df: u64) -> Result<()> {
        let mut conn = self.get_connection().await?;

        let mut pipe = redis::pipe();

        // Use HINCRBYFLOAT for atomic accumulation of TFs, use "default" as default doc_id.
        pipe.cmd("HINCRBYFLOAT")
            .arg(self.make_tf_key(term))
            .arg("default")
            .arg(tf);

        // Setting the DF value
        pipe.cmd("SET").arg(self.make_df_key(term)).arg(df as usize);

        let _: () = pipe.query_async(&mut *conn).await.map_err(|e| {
            self.record_error("connection");
            Bm25Error::StorageError(e.to_string())
        })?;

        Ok(())
    }

    async fn commit_batch(&mut self, stats: &Bm25Stats) -> Result<()> {
        if stats.tf.is_empty() && stats.df.is_empty() {
            return Ok(());
        }

        let mut conn = self.get_connection().await?;
        let mut pipe = redis::pipe();

        // Batch TF Operations - Using HINCRBYFLOAT
        for (term, tf) in &stats.tf {
            pipe.cmd("HINCRBYFLOAT")
                .arg(self.make_tf_key(term))
                .arg("default")
                .arg(*tf);
        }

        // Batch DF operations - using SET
        for (term, df) in &stats.df {
            pipe.cmd("SET")
                .arg(self.make_df_key(term))
                .arg(*df as usize);
        }

        let _: () = pipe.query_async(&mut *conn).await.map_err(|e| {
            self.record_error("connection");
            Bm25Error::StorageError(e.to_string())
        })?;

        // Update memory usage
        self.update_memory_usage().await?;

        Ok(())
    }

    async fn get_stats(&self, term: &str) -> Result<Option<Bm25Stats>> {
        let mut conn = self.get_connection().await?;

        let tf: Option<f32> = redis::cmd("HGET")
            .arg(self.make_tf_key(term))
            .arg("default")
            .query_async(&mut *conn)
            .await
            .unwrap_or(None);

        let df: Option<u64> = redis::cmd("GET")
            .arg(self.make_df_key(term))
            .query_async(&mut *conn)
            .await
            .unwrap_or(None);

        if tf.is_none() && df.is_none() {
            return Ok(None);
        }

        let mut tf_map = HashMap::new();
        if let Some(tf_val) = tf {
            tf_map.insert(term.to_string(), tf_val);
        }

        let mut df_map = HashMap::new();
        if let Some(df_val) = df {
            df_map.insert(term.to_string(), df_val);
        }

        Ok(Some(Bm25Stats {
            tf: tf_map,
            df: df_map,
            total_docs: 0,
            avg_doc_length: 0.0,
        }))
    }

    async fn get_df(&self, term: &str) -> Result<Option<u64>> {
        let mut conn = self.get_connection().await?;

        let df: Option<u64> = redis::cmd("GET")
            .arg(self.make_df_key(term))
            .query_async(&mut *conn)
            .await
            .map_err(|e| {
                self.record_error("connection");
                Bm25Error::StorageError(e.to_string())
            })
            .unwrap_or(None);

        Ok(df)
    }

    async fn get_tf(&self, term: &str, doc_id: &str) -> Result<Option<f32>> {
        let mut conn = self.get_connection().await?;

        // Get the TF value for a specific doc_id from the hash
        let tf: Option<f32> = redis::cmd("HGET")
            .arg(self.make_tf_key(term))
            .arg(doc_id)
            .query_async(&mut *conn)
            .await
            .map_err(|e| {
                self.record_error("connection");
                Bm25Error::StorageError(e.to_string())
            })
            .unwrap_or(None);

        Ok(tf)
    }

    async fn clear(&mut self) -> Result<()> {
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
                    Bm25Error::StorageError(e.to_string())
                })?;
        }

        Ok(())
    }

    async fn info(&self) -> Result<StorageInfo> {
        let pattern = format!("{}:*", self.key_prefix);
        let _keys = self.scan_keys(&pattern).await?;

        let df_pattern = format!("{}:df:*", self.key_prefix);
        let df_keys = self.scan_keys(&df_pattern).await?;

        // Using Memory Usage
        let total_size = self.memory_usage.load(Ordering::Relaxed) as u64;

        Ok(StorageInfo {
            name: "RedisStorage".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            size: total_size,
            document_count: 0,
            term_count: df_keys.len(),
            is_connected: true,
        })
    }

    async fn health_check(&self) -> Result<bool> {
        match self.pool.get().await {
            Ok(mut conn) => {
                let result: std::result::Result<String, redis::RedisError> =
                    redis::cmd("PING").query_async(&mut *conn).await;
                Ok(result.is_ok())
            }
            Err(_) => Ok(false),
        }
    }

    async fn delete_doc_stats(&mut self, doc_id: &str) -> Result<()> {
        // Deleting a TF statistic for a specific document requires that the doc_id be removed from all lexical items
        // This is a time-consuming operation that requires scanning all TF keys
        let pattern = format!("{}:tf:*", self.key_prefix);
        let tf_keys = self.scan_keys(&pattern).await?;

        if !tf_keys.is_empty() {
            let mut conn = self.get_connection().await?;
            let mut pipe = redis::pipe();

            // Remove doc_id field from each TF Hash
            for tf_key in tf_keys {
                pipe.cmd("HDEL").arg(tf_key).arg(doc_id);
            }

            let _: () = pipe.query_async(&mut *conn).await.map_err(|e| {
                self.record_error("connection");
                Bm25Error::StorageError(e.to_string())
            })?;
        }

        Ok(())
    }
}

impl RedisStorage {
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
    #[allow(dead_code)]
    fn record_operation_start(&self) -> Instant {
        Instant::now()
    }

    /// Record operation completion (internal use)
    #[allow(dead_code)]
    fn record_operation_completion(&self, start_time: Instant) {
        let latency = start_time.elapsed().as_micros() as u64;
        self.operation_count.fetch_add(1, Ordering::Relaxed);
        self.total_latency.fetch_add(latency, Ordering::Relaxed);
    }

    /// Submit document-specific TF statistics (Redis-specific method)
    pub async fn commit_doc_tf(&mut self, term: &str, doc_id: &str, tf: f32) -> Result<()> {
        let mut conn = self.get_connection().await?;

        let _: () = redis::cmd("HINCRBYFLOAT")
            .arg(self.make_tf_key(term))
            .arg(doc_id)
            .arg(tf)
            .query_async(&mut *conn)
            .await
            .map_err(|e| {
                self.record_error("connection");
                Bm25Error::StorageError(e.to_string())
            })?;

        Ok(())
    }

    /// TF statistics for batch submission of multiple documents (Redis-specific approach)
    pub async fn commit_batch_doc_tf(
        &mut self,
        term: &str,
        doc_tfs: &[(String, f32)],
    ) -> Result<()> {
        if doc_tfs.is_empty() {
            return Ok(());
        }

        let mut conn = self.get_connection().await?;
        let mut pipe = redis::pipe();

        for (doc_id, tf) in doc_tfs {
            pipe.cmd("HINCRBYFLOAT")
                .arg(self.make_tf_key(term))
                .arg(doc_id)
                .arg(*tf);
        }

        let _: () = pipe.query_async(&mut *conn).await.map_err(|e| {
            self.record_error("connection");
            Bm25Error::StorageError(e.to_string())
        })?;

        Ok(())
    }

    /// Get the TF of all documents under the term (Redis-specific method)
    pub async fn get_all_doc_tf(&self, term: &str) -> Result<HashMap<String, f32>> {
        let mut conn = self.get_connection().await?;

        let tf_map: HashMap<String, f32> = redis::cmd("HGETALL")
            .arg(self.make_tf_key(term))
            .query_async(&mut *conn)
            .await
            .map_err(|e| {
                self.record_error("connection");
                Bm25Error::StorageError(e.to_string())
            })
            .unwrap_or_default();

        Ok(tf_map)
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
