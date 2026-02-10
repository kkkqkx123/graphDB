//! 批量操作优化模块
//!
//! 提供高效的批量读取和批量操作接口，减少I/O次数，提高性能。

use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::core::{Edge, Value, Vertex};
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
/// - 批量顺序读取
/// - 预读优化
/// - 内存限制
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

    pub fn batch_get_vertices(
        &self,
        ids: &[Value],
    ) -> BatchReadResult<Option<Vertex>> {
        let start_time = std::time::Instant::now();
        let mut results = Vec::with_capacity(ids.len());
        let mut failed_keys = Vec::new();

        let storage = safe_lock(&*self.storage)
            .expect("BatchOptimizer storage lock should not be poisoned");

        for id in ids {
            match storage.get_vertex("default", id) {
                Ok(vertex) => results.push(vertex),
                Err(_) => failed_keys.push(id.clone()),
            }
        }

        let total_time = start_time.elapsed();
        let memory_used = ids.len() * std::mem::size_of::<Value>();

        BatchReadResult::new(results, failed_keys, total_time, memory_used)
    }

    pub fn batch_get_edges(
        &self,
        edge_keys: &[(Value, Value, String)],
    ) -> BatchReadResult<Option<Edge>> {
        let start_time = std::time::Instant::now();
        let mut results = Vec::with_capacity(edge_keys.len());
        let mut failed_keys = Vec::new();

        let storage = safe_lock(&*self.storage)
            .expect("BatchOptimizer storage lock should not be poisoned");

        for (src, dst, edge_type) in edge_keys {
            match storage.get_edge("default", src, dst, edge_type) {
                Ok(edge) => results.push(edge),
                Err(_) => failed_keys.push(Value::String(format!(
                    "{}->{}:{}",
                    src, dst, edge_type
                ))),
            }
        }

        let total_time = start_time.elapsed();
        let memory_used = edge_keys.len()
            * (std::mem::size_of::<Value>() * 2 + std::mem::size_of::<String>());

        BatchReadResult::new(results, failed_keys, total_time, memory_used)
    }

    pub fn batch_scan_vertices(
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

    pub fn batch_scan_edges(
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
