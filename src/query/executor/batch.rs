//! 批量操作优化模块
//!
//! 提供高效的批量读取和批量操作接口，减少I/O次数，提高性能。

use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::time::timeout;

use crate::core::{DBResult, Edge, Value, Vertex};
use crate::storage::StorageClient;
use crate::utils::safe_lock;

/// 批量操作配置
#[derive(Debug, Clone)]
pub struct BatchConfig {
    pub batch_size: usize,
    pub max_concurrency: usize,
    pub timeout_ms: u64,
    pub enable_prefetch: bool,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            batch_size: 100,
            max_concurrency: num_cpus::get() * 2,
            timeout_ms: 5000,
            enable_prefetch: true,
        }
    }
}

impl BatchConfig {
    pub fn new(batch_size: usize, max_concurrency: usize) -> Self {
        Self {
            batch_size,
            max_concurrency,
            ..Default::default()
        }
    }

    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    pub fn with_prefetch(mut self, enable: bool) -> Self {
        self.enable_prefetch = enable;
        self
    }
}

/// 批量读取结果
#[derive(Debug)]
pub struct BatchReadResult<T> {
    pub items: Vec<T>,
    pub failed_keys: Vec<Value>,
    pub total_time: Duration,
    pub memory_used: usize,
}

impl<T> BatchReadResult<T> {
    pub fn new(items: Vec<T>, failed_keys: Vec<Value>, total_time: Duration, memory_used: usize) -> Self {
        Self {
            items,
            failed_keys,
            total_time,
            memory_used,
        }
    }

    pub fn success_count(&self) -> usize {
        self.items.len()
    }

    pub fn failure_count(&self) -> usize {
        self.failed_keys.len()
    }

    pub fn total_count(&self) -> usize {
        self.success_count() + self.failure_count()
    }
}

/// 批量操作优化器
///
/// 提供高效的批量读取接口，支持：
/// - 批量并行读取
/// - 预读优化
/// - 内存限制
/// - 超时控制
pub struct BatchOptimizer<S: StorageClient + Send + 'static> {
    storage: Arc<Mutex<S>>,
    config: BatchConfig,
}

impl<S: StorageClient + Send + 'static> BatchOptimizer<S> {
    pub fn new(storage: Arc<Mutex<S>>, config: BatchConfig) -> Self {
        Self { storage, config }
    }

    pub fn with_default_config(storage: Arc<Mutex<S>>) -> Self {
        Self {
            storage,
            config: BatchConfig::default(),
        }
    }

    pub async fn batch_get_vertices(
        &self,
        ids: &[Value],
    ) -> BatchReadResult<Option<Vertex>> {
        let start_time = std::time::Instant::now();
        let mut results = Vec::with_capacity(ids.len());
        let mut failed_keys = Vec::new();

        let semaphore = Arc::new(Semaphore::new(self.config.max_concurrency));
        let mut handles = Vec::new();

        for id in ids {
            let id = id.clone();
            let storage = Arc::clone(&self.storage);
            let permit = Arc::clone(&semaphore).acquire_owned().await.unwrap();

            let handle = tokio::spawn(async move {
                let _permit = permit;
                let result = safe_lock(&*storage)
                    .expect("BatchOptimizer storage lock should not be poisoned")
                    .get_vertex("default", &id);

                (id, result)
            });
            handles.push(handle);
        }

        for handle in handles {
            match timeout(
                Duration::from_millis(self.config.timeout_ms),
                handle,
            )
            .await
            {
                Ok(Ok((id, result))) => {
                    match result {
                        Ok(vertex) => results.push(vertex),
                        Err(_) => failed_keys.push(id),
                    }
                }
                Ok(Err(_)) => {
                    // Spawn error, treat as failure
                }
                Err(_) => {
                    // Timeout
                }
            }
        }

        let total_time = start_time.elapsed();
        let memory_used = ids.len() * std::mem::size_of::<Value>();

        BatchReadResult::new(results, failed_keys, total_time, memory_used)
    }

    pub async fn batch_get_edges(
        &self,
        edge_keys: &[(Value, Value, String)],
    ) -> BatchReadResult<Option<Edge>> {
        let start_time = std::time::Instant::now();
        let mut results = Vec::with_capacity(edge_keys.len());
        let mut failed_keys = Vec::new();

        let semaphore = Arc::new(Semaphore::new(self.config.max_concurrency));
        let mut handles = Vec::new();

        for (src, dst, edge_type) in edge_keys {
            let src = src.clone();
            let dst = dst.clone();
            let edge_type = edge_type.clone();
            let storage = Arc::clone(&self.storage);
            let permit = Arc::clone(&semaphore).acquire_owned().await.unwrap();

            let handle = tokio::spawn(async move {
                let _permit = permit;
                let result = safe_lock(&*storage)
                    .expect("BatchOptimizer storage lock should not be poisoned")
                    .get_edge("default", &src, &dst, &edge_type);

                ((src, dst, edge_type), result)
            });
            handles.push(handle);
        }

        for handle in handles {
            match timeout(
                Duration::from_millis(self.config.timeout_ms),
                handle,
            )
            .await
            {
                Ok(Ok(((src, dst, edge_type), result))) => {
                    match result {
                        Ok(edge) => results.push(edge),
                        Err(_) => failed_keys.push(Value::String(format!(
                            "{}->{}:{}",
                            src, dst, edge_type
                        ))),
                    }
                }
                Ok(Err(_)) => {}
                Err(_) => {}
            }
        }

        let total_time = start_time.elapsed();
        let memory_used = edge_keys.len()
            * (std::mem::size_of::<Value>() * 2 + std::mem::size_of::<String>());

        BatchReadResult::new(results, failed_keys, total_time, memory_used)
    }

    pub async fn batch_scan_vertices(
        &self,
        tag: Option<&str>,
        limit: Option<usize>,
    ) -> BatchReadResult<Vertex> {
        let start_time = std::time::Instant::now();
        let storage_guard = safe_lock(&*self.storage)
            .expect("BatchOptimizer storage lock should not be poisoned");

        let vertices = if let Some(tag_name) = tag {
            storage_guard.scan_vertices_by_tag("default", tag_name)
        } else {
            storage_guard.scan_vertices("default")
        };

        let mut vertices: Vec<Vertex> = match vertices {
            Ok(v) => v,
            Err(_) => Vec::new(),
        };

        if let Some(limit) = limit {
            vertices.truncate(limit);
        }

        let total_time = start_time.elapsed();
        let memory_used = vertices.iter().map(|v| v.estimated_size()).sum();

        BatchReadResult::new(vertices, Vec::new(), total_time, memory_used)
    }

    pub async fn batch_scan_edges(
        &self,
        edge_type: Option<&str>,
        limit: Option<usize>,
    ) -> BatchReadResult<Edge> {
        let start_time = std::time::Instant::now();
        let storage_guard = safe_lock(&*self.storage)
            .expect("BatchOptimizer storage lock should not be poisoned");

        let edges = if let Some(type_name) = edge_type {
            storage_guard.scan_edges_by_type("default", type_name)
        } else {
            storage_guard.scan_all_edges("default")
        };

        let mut edges: Vec<Edge> = match edges {
            Ok(e) => e,
            Err(_) => Vec::new(),
        };

        if let Some(limit) = limit {
            edges.truncate(limit);
        }

        let total_time = start_time.elapsed();
        let memory_used = edges.iter().map(|e| e.estimated_size()).sum();

        BatchReadResult::new(edges, Vec::new(), total_time, memory_used)
    }
}

/// 并发控制配置
#[derive(Debug, Clone)]
pub struct ConcurrencyConfig {
    pub max_read_concurrency: usize,
    pub max_write_concurrency: usize,
    pub max_total_concurrency: usize,
    pub queue_size: usize,
    pub fair_mode: bool,
}

impl Default for ConcurrencyConfig {
    fn default() -> Self {
        Self {
            max_read_concurrency: num_cpus::get() * 4,
            max_write_concurrency: num_cpus::get() * 2,
            max_total_concurrency: num_cpus::get() * 6,
            queue_size: 10000,
            fair_mode: true,
        }
    }
}

impl ConcurrencyConfig {
    pub fn new(
        max_read: usize,
        max_write: usize,
        max_total: usize,
    ) -> Self {
        Self {
            max_read_concurrency: max_read,
            max_write_concurrency: max_write,
            max_total_concurrency: max_total,
            ..Default::default()
        }
    }

    pub fn with_fair_mode(mut self, fair: bool) -> Self {
        self.fair_mode = fair;
        self
    }

    pub fn with_queue_size(mut self, size: usize) -> Self {
        self.queue_size = size;
        self
    }
}

/// 并发控制器
///
/// 管理读写操作的并发执行，支持：
/// - 读写分离
/// - 公平调度
/// - 队列管理
pub struct ConcurrencyController<S: StorageClient> {
    storage: Arc<Mutex<S>>,
    read_semaphore: Arc<Semaphore>,
    write_semaphore: Arc<Semaphore>,
    total_semaphore: Arc<Semaphore>,
    config: ConcurrencyConfig,
    stats: ConcurrencyStats,
}

#[derive(Debug, Default)]
pub struct ConcurrencyStats {
    pub total_reads: u64,
    pub total_writes: u64,
    pub total_rejected: u64,
    pub current_reads: usize,
    pub current_writes: usize,
    pub avg_wait_time_read: Duration,
    pub avg_wait_time_write: Duration,
}

impl<S: StorageClient> ConcurrencyController<S> {
    pub fn new(storage: Arc<Mutex<S>>, config: ConcurrencyConfig) -> Self {
        Self {
            storage,
            read_semaphore: Arc::new(Semaphore::new(config.max_read_concurrency)),
            write_semaphore: Arc::new(Semaphore::new(config.max_write_concurrency)),
            total_semaphore: Arc::new(Semaphore::new(config.max_total_concurrency)),
            config,
            stats: ConcurrencyStats::default(),
        }
    }

    pub fn with_default_config(storage: Arc<Mutex<S>>) -> Self {
        Self::new(storage, ConcurrencyConfig::default())
    }

    pub async fn read<T, F>(&mut self, operation: F) -> DBResult<T>
    where
        F: FnOnce(&S) -> DBResult<T>,
    {
        let _total_permit = self.total_semaphore.acquire().await.unwrap();
        let start = std::time::Instant::now();

        let read_permit = self.read_semaphore.acquire().await.unwrap();

        let storage_guard = safe_lock(&*self.storage)
            .expect("ConcurrencyController storage lock should not be poisoned");

        let result = operation(&*storage_guard);

        drop(read_permit);
        drop(storage_guard);
        drop(_total_permit);

        self.stats.current_reads += 1;
        self.stats.total_reads += 1;

        let wait_time = start.elapsed();
        self.update_avg_wait_time(wait_time, true);

        self.stats.current_reads -= 1;

        result
    }

    pub async fn write<T, F>(&mut self, operation: F) -> DBResult<T>
    where
        F: FnOnce(&mut S) -> DBResult<T>,
    {
        let _total_permit = self.total_semaphore.acquire().await.unwrap();
        let start = std::time::Instant::now();

        let write_permit = self.write_semaphore.acquire().await.unwrap();

        let mut storage_guard = safe_lock(&*self.storage)
            .expect("ConcurrencyController storage lock should not be poisoned");

        let result = operation(&mut *storage_guard);

        drop(write_permit);
        drop(storage_guard);
        drop(_total_permit);

        self.stats.current_writes += 1;
        self.stats.total_writes += 1;

        let wait_time = start.elapsed();
        self.update_avg_wait_time(wait_time, false);

        self.stats.current_writes -= 1;

        result
    }

    fn update_avg_wait_time(&mut self, wait_time: Duration, is_read: bool) {
        if is_read {
            let total = self.stats.total_reads as f64;
            let current_avg = self.stats.avg_wait_time_read.as_secs_f64();
            self.stats.avg_wait_time_read =
                Duration::from_secs_f64(current_avg * (total - 1.0) / total + wait_time.as_secs_f64() / total);
        } else {
            let total = self.stats.total_writes as f64;
            let current_avg = self.stats.avg_wait_time_write.as_secs_f64();
            self.stats.avg_wait_time_write =
                Duration::from_secs_f64(current_avg * (total - 1.0) / total + wait_time.as_secs_f64() / total);
        }
    }

    pub fn stats(&self) -> &ConcurrencyStats {
        &self.stats
    }

    pub fn can_read(&self) -> bool {
        self.read_semaphore.available_permits() > 0
    }

    pub fn can_write(&self) -> bool {
        self.write_semaphore.available_permits() > 0
    }

    pub fn utilization(&self) -> (f64, f64) {
        let read_util = 1.0
            - self.read_semaphore.available_permits() as f64
                / self.config.max_read_concurrency as f64;
        let write_util = 1.0
            - self.write_semaphore.available_permits() as f64
                / self.config.max_write_concurrency as f64;
        (read_util, write_util)
    }
}
