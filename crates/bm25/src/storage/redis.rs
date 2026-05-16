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
use crate::storage::common::metrics::{ErrorType, StorageMetrics, StorageMetricsCollector};
use crate::storage::common::r#trait::{Bm25Stats, StorageInterface};
use crate::storage::common::types::StorageInfo;
use bb8::Pool;
use redis::{aio::MultiplexedConnection, Client as RedisClient};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

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

use std::time::Duration;

/// Redis Storage Implementation
pub struct RedisStorage {
    pool: Pool<RedisConnectionManager>,
    key_prefix: String,
    memory_usage: Arc<AtomicUsize>,
    /// Unified metrics collector replacing individual atomic fields
    metrics: Arc<StorageMetricsCollector>,
}

impl std::fmt::Debug for RedisStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisStorage")
            .field("key_prefix", &self.key_prefix)
            .field("memory_usage", &self.memory_usage.load(Ordering::Relaxed))
            .field("operation_count", &self.metrics.get_operation_count())
            .finish()
    }
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
            metrics: Arc::new(StorageMetricsCollector::default()),
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
            self.metrics.record_error(ErrorType::Connection);
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
                    self.metrics.record_error(ErrorType::Connection);
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
        let start = Instant::now();
        
        let mut conn = self.get_connection().await?;

        // Use the Redis INFO memory command to get overall memory usage
        let info: String = redis::cmd("INFO")
            .arg("memory")
            .query_async(&mut *conn)
            .await
            .map_err(|e| {
                self.metrics.record_error(ErrorType::Connection);
                Bm25Error::StorageError(e.to_string())
            })?;

        // Parsing the used_memory field
        let memory = self.parse_redis_memory_info(&info);
        self.memory_usage.store(memory, Ordering::Relaxed);
        
        self.metrics.record_operation(start);
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

    /// Record an error of the specified type
    fn record_error(&self, error_type: ErrorType) {
        self.metrics.record_error(error_type);
    }

    /// Gets operation statistics and performance metrics
    ///
    /// Returns a snapshot of current storage metrics including
    /// operation counts, latencies, and error statistics.
    pub fn get_operation_stats(&self) -> StorageMetrics {
        self.metrics.get_metrics(self.memory_usage.load(Ordering::Relaxed) as u64)
    }
}

#[async_trait::async_trait]
impl StorageInterface for RedisStorage {
    async fn init(&mut self) -> Result<()> {
        let start = Instant::now();
        
        let mut conn = self.get_connection().await?;
        let _: String = redis::cmd("PING")
            .query_async(&mut *conn)
            .await
            .map_err(|e| {
                self.record_error(ErrorType::Connection);
                Bm25Error::StorageError(e.to_string())
            })?;
        
        self.metrics.record_operation(start);
        Ok(())
    }

    async fn close(&mut self) -> Result<()> {
        let start = Instant::now();
        self.metrics.record_operation(start);
        Ok(())
    }

    async fn commit_stats(&mut self, term: &str, tf: f32, df: u64) -> Result<()> {
        let start = Instant::now();
        
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
            self.record_error(ErrorType::Connection);
            Bm25Error::StorageError(e.to_string())
        })?;

        self.metrics.record_operation(start);
        Ok(())
    }

    async fn commit_batch(&mut self, stats: &Bm25Stats) -> Result<()> {
        let start = Instant::now();
        
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
            self.record_error(ErrorType::Connection);
            Bm25Error::StorageError(e.to_string())
        })?;

        // Update memory usage
        self.update_memory_usage().await?;

        self.metrics.record_operation(start);
        Ok(())
    }

    async fn get_stats(&self, term: &str) -> Result<Option<Bm25Stats>> {
        let start = Instant::now();
        
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

        self.metrics.record_operation(start);
        
        Ok(Some(Bm25Stats {
            tf: tf_map,
            df: df_map,
            total_docs: 0,
            avg_doc_length: 0.0,
        }))
    }

    async fn get_df(&self, term: &str) -> Result<Option<u64>> {
        let start = Instant::now();
        
        let mut conn = self.get_connection().await?;

        let df: Option<u64> = redis::cmd("GET")
            .arg(self.make_df_key(term))
            .query_async(&mut *conn)
            .await
            .map_err(|e| {
                self.record_error(ErrorType::Connection);
                Bm25Error::StorageError(e.to_string())
            })
            .unwrap_or(None);

        self.metrics.record_operation(start);
        
        Ok(df)
    }

    async fn get_tf(&self, term: &str, doc_id: &str) -> Result<Option<f32>> {
        let start = Instant::now();
        
        let mut conn = self.get_connection().await?;

        // Get the TF value for a specific doc_id from the hash
        let tf: Option<f32> = redis::cmd("HGET")
            .arg(self.make_tf_key(term))
            .arg(doc_id)
            .query_async(&mut *conn)
            .await
            .map_err(|e| {
                self.record_error(ErrorType::Connection);
                Bm25Error::StorageError(e.to_string())
            })
            .unwrap_or(None);

        self.metrics.record_operation(start);
        
        Ok(tf)
    }

    async fn clear(&mut self) -> Result<()> {
        let start = Instant::now();
        
        let pattern = format!("{}:*", self.key_prefix);
        let keys = self.scan_keys(&pattern).await?;

        if !keys.is_empty() {
            let mut conn = self.get_connection().await?;
            let _: () = redis::cmd("DEL")
                .arg(keys.as_slice())
                .query_async(&mut *conn)
                .await
                .map_err(|e| {
                    self.record_error(ErrorType::Connection);
                    Bm25Error::StorageError(e.to_string())
                })?;
        }

        self.metrics.record_operation(start);
        Ok(())
    }

    async fn info(&self) -> Result<StorageInfo> {
        let start = Instant::now();
        
        let pattern = format!("{}:*", self.key_prefix);
        let _keys = self.scan_keys(&pattern).await?;

        let df_pattern = format!("{}:df:*", self.key_prefix);
        let df_keys = self.scan_keys(&df_pattern).await?;

        // Using Memory Usage
        let total_size = self.memory_usage.load(Ordering::Relaxed) as u64;

        self.metrics.record_operation(start);
        
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
        let start = Instant::now();
        
        let healthy = match self.pool.get().await {
            Ok(mut conn) => {
                let result: std::result::Result<String, redis::RedisError> =
                    redis::cmd("PING").query_async(&mut *conn).await;
                result.is_ok()
            }
            Err(_) => false,
        };

        self.metrics.record_operation(start);
        
        Ok(healthy)
    }

    async fn delete_doc_stats(&mut self, doc_id: &str) -> Result<()> {
        let start = Instant::now();
        
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
                self.record_error(ErrorType::Connection);
                Bm25Error::StorageError(e.to_string())
            })?;
        }

        self.metrics.record_operation(start);
        Ok(())
    }
}

impl RedisStorage {
    /// Health checkup
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

    /// Submit document-specific TF statistics (Redis-specific method)
    pub async fn commit_doc_tf(&mut self, term: &str, doc_id: &str, tf: f32) -> Result<()> {
        let start = Instant::now();
        
        let mut conn = self.get_connection().await?;

        let _: () = redis::cmd("HINCRBYFLOAT")
            .arg(self.make_tf_key(term))
            .arg(doc_id)
            .arg(tf)
            .query_async(&mut *conn)
            .await
            .map_err(|e| {
                self.record_error(ErrorType::Connection);
                Bm25Error::StorageError(e.to_string())
            })?;

        self.metrics.record_operation(start);
        Ok(())
    }

    /// TF statistics for batch submission of multiple documents (Redis-specific approach)
    pub async fn commit_batch_doc_tf(
        &mut self,
        term: &str,
        doc_tfs: &[(String, f32)],
    ) -> Result<()> {
        let start = Instant::now();
        
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
            self.record_error(ErrorType::Connection);
            Bm25Error::StorageError(e.to_string())
        })?;

        self.metrics.record_operation(start);
        Ok(())
    }

    /// Get the TF of all documents under the term (Redis-specific method)
    pub async fn get_all_doc_tf(&self, term: &str) -> Result<HashMap<String, f32>> {
        let start = Instant::now();
        
        let mut conn = self.get_connection().await?;

        let tf_map: HashMap<String, f32> = redis::cmd("HGETALL")
            .arg(self.make_tf_key(term))
            .query_async(&mut *conn)
            .await
            .map_err(|e| {
                self.record_error(ErrorType::Connection);
                Bm25Error::StorageError(e.to_string())
            })
            .unwrap_or_default();

        self.metrics.record_operation(start);
        
        Ok(tf_map)
    }
}
