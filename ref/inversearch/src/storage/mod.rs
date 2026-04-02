//! 存储接口模块
//!
//! 提供持久化存储的抽象接口和实现

use crate::r#type::{SearchResults, EnrichedSearchResults, DocId};
use crate::error::Result;
use crate::Index;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use std::path::PathBuf;

pub mod redis;
pub mod wal;

/// 存储接口 - 类似JavaScript版本的StorageInterface
#[async_trait::async_trait]
pub trait StorageInterface: Send + Sync {
    /// 挂载索引到存储
    async fn mount(&mut self, index: &Index) -> Result<()>;
    
    /// 打开连接
    async fn open(&mut self) -> Result<()>;
    
    /// 关闭连接
    async fn close(&mut self) -> Result<()>;
    
    /// 销毁数据库
    async fn destroy(&mut self) -> Result<()>;
    
    /// 提交索引变更
    async fn commit(&mut self, index: &Index, replace: bool, append: bool) -> Result<()>;
    
    /// 获取术语结果
    async fn get(&self, key: &str, ctx: Option<&str>, limit: usize, offset: usize, resolve: bool, enrich: bool) -> Result<SearchResults>;
    
    /// 富化结果
    async fn enrich(&self, ids: &[DocId]) -> Result<EnrichedSearchResults>;
    
    /// 检查ID是否存在
    async fn has(&self, id: DocId) -> Result<bool>;
    
    /// 删除ID
    async fn remove(&mut self, ids: &[DocId]) -> Result<()>;
    
    /// 清空数据
    async fn clear(&mut self) -> Result<()>;
    
    /// 获取存储信息
    async fn info(&self) -> Result<StorageInfo>;
}

/// 存储信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageInfo {
    pub name: String,
    pub version: String,
    pub size: u64,
    pub document_count: usize,
    pub index_count: usize,
    pub is_connected: bool,
}

/// 内存存储实现 - 用于测试和开发
pub struct MemoryStorage {
    data: HashMap<String, Vec<DocId>>,
    context_data: HashMap<String, HashMap<String, Vec<DocId>>>,
    documents: HashMap<DocId, String>,
    is_open: bool,
    memory_usage: AtomicUsize,
    operation_count: AtomicUsize,
    total_latency: AtomicUsize,
}

impl MemoryStorage {
    /// 创建新的内存存储
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            context_data: HashMap::new(),
            documents: HashMap::new(),
            is_open: false,
            memory_usage: AtomicUsize::new(0),
            operation_count: AtomicUsize::new(0),
            total_latency: AtomicUsize::new(0),
        }
    }

    /// 获取内存使用情况
    pub fn get_memory_usage(&self) -> usize {
        self.memory_usage.load(Ordering::Relaxed)
    }

    /// 获取操作统计
    pub fn get_operation_stats(&self) -> MemoryStorageMetrics {
        let operation_count = self.operation_count.load(Ordering::Relaxed);
        let total_latency = self.total_latency.load(Ordering::Relaxed);
        let avg_latency = if operation_count > 0 {
            total_latency / operation_count
        } else {
            0
        };

        MemoryStorageMetrics {
            operation_count,
            average_latency: avg_latency,
            memory_usage: self.get_memory_usage(),
            error_count: 0,
        }
    }

    /// 更新内存使用量
    fn update_memory_usage(&self) {
        let mut total_size = 0;
        
        // 计算数据大小
        total_size += std::mem::size_of_val(&self.data);
        for (k, v) in &self.data {
            total_size += k.len() + v.len() * std::mem::size_of::<DocId>();
        }
        
        // 计算上下文数据大小
        total_size += std::mem::size_of_val(&self.context_data);
        for (ctx_key, ctx_map) in &self.context_data {
            total_size += ctx_key.len();
            total_size += std::mem::size_of_val(ctx_map);
            for (term, ids) in ctx_map {
                total_size += term.len() + ids.len() * std::mem::size_of::<DocId>();
            }
        }
        
        // 计算文档大小
        total_size += std::mem::size_of_val(&self.documents);
        for (id, content) in &self.documents {
            total_size += std::mem::size_of_val(id) + content.len();
        }
        
        self.memory_usage.store(total_size, Ordering::Relaxed);
    }

    /// 记录操作开始时间
    fn record_operation_start(&self) -> Instant {
        Instant::now()
    }

    /// 记录操作完成
    fn record_operation_completion(&self, start_time: Instant) {
        let latency = start_time.elapsed().as_micros() as usize;
        self.operation_count.fetch_add(1, Ordering::Relaxed);
        self.total_latency.fetch_add(latency, Ordering::Relaxed);
    }
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

/// 内存存储性能指标
#[derive(Debug, Clone)]
pub struct MemoryStorageMetrics {
    pub operation_count: usize,
    pub average_latency: usize,
    pub memory_usage: usize,
    pub error_count: usize,
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
        self.data.clear();
        self.context_data.clear();
        self.documents.clear();
        self.is_open = false;
        Ok(())
    }
    
    async fn commit(&mut self, index: &Index, _replace: bool, _append: bool) -> Result<()> {
        let start_time = self.record_operation_start();
        
        // 从索引导出数据到存储
        for doc_ids in index.map.index.values() {
            for (term_str, ids) in doc_ids {
                self.data.insert(term_str.clone(), ids.clone());
            }
        }
        
        // 导出上下文数据
        for ctx_map in index.ctx.index.values() {
            for (ctx_term, doc_ids) in ctx_map {
                self.context_data.entry("default".to_string())
                    .or_default()
                    .insert(ctx_term.clone(), doc_ids.clone());
            }
        }
        
        self.update_memory_usage();
        self.record_operation_completion(start_time);
        
        Ok(())
    }
    
    async fn get(&self, key: &str, ctx: Option<&str>, limit: usize, offset: usize, _resolve: bool, _enrich: bool) -> Result<SearchResults> {
        let results = if let Some(ctx_key) = ctx {
            // 上下文搜索
            if let Some(ctx_map) = self.context_data.get(ctx_key) {
                if let Some(doc_ids) = ctx_map.get(key) {
                    apply_limit_offset(doc_ids, limit, offset)
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        } else {
            // 普通搜索
            if let Some(doc_ids) = self.data.get(key) {
                apply_limit_offset(doc_ids, limit, offset)
            } else {
                Vec::new()
            }
        };
        
        Ok(results)
    }
    
    async fn enrich(&self, ids: &[DocId]) -> Result<EnrichedSearchResults> {
        let mut results = Vec::new();
        
        for &id in ids {
            if let Some(content) = self.documents.get(&id) {
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
        // 检查文档ID是否存在于索引数据中
        for doc_ids in self.data.values() {
            if doc_ids.contains(&id) {
                return Ok(true);
            }
        }

        // 检查上下文数据
        for ctx_map in self.context_data.values() {
            for doc_ids in ctx_map.values() {
                if doc_ids.contains(&id) {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }
    
    async fn remove(&mut self, ids: &[DocId]) -> Result<()> {
        for &id in ids {
            self.documents.remove(&id);
            
            // 从索引数据中移除
            for doc_ids in self.data.values_mut() {
                doc_ids.retain(|&doc_id| doc_id != id);
            }
            
            // 从上下文数据中移除
            for ctx_map in self.context_data.values_mut() {
                for doc_ids in ctx_map.values_mut() {
                    doc_ids.retain(|&doc_id| doc_id != id);
                }
            }
        }
        Ok(())
    }
    
    async fn clear(&mut self) -> Result<()> {
        self.data.clear();
        self.context_data.clear();
        self.documents.clear();
        Ok(())
    }
    
    async fn info(&self) -> Result<StorageInfo> {
        Ok(StorageInfo {
            name: "MemoryStorage".to_string(),
            version: "0.1.0".to_string(),
            size: (self.data.len() + self.context_data.len() + self.documents.len()) as u64,
            document_count: self.documents.len(),
            index_count: self.data.len(),
            is_connected: self.is_open,
        })
    }
}

/// 文件存储实现
pub struct FileStorage {
    base_path: PathBuf,
    data: HashMap<String, Vec<DocId>>,
    context_data: HashMap<String, HashMap<String, Vec<DocId>>>,
    documents: HashMap<DocId, String>,
    memory_usage: AtomicUsize,
    operation_count: AtomicUsize,
    total_latency: AtomicUsize,
    is_open: bool,
}

impl FileStorage {
    /// 创建新的文件存储
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
            data: HashMap::new(),
            context_data: HashMap::new(),
            documents: HashMap::new(),
            memory_usage: AtomicUsize::new(0),
            operation_count: AtomicUsize::new(0),
            total_latency: AtomicUsize::new(0),
            is_open: false,
        }
    }

    /// 获取内存使用情况
    pub fn get_memory_usage(&self) -> usize {
        self.memory_usage.load(Ordering::Relaxed)
    }

    /// 获取操作统计
    pub fn get_operation_stats(&self) -> FileStorageMetrics {
        let operation_count = self.operation_count.load(Ordering::Relaxed);
        let total_latency = self.total_latency.load(Ordering::Relaxed);
        let avg_latency = if operation_count > 0 {
            total_latency / operation_count
        } else {
            0
        };

        FileStorageMetrics {
            operation_count,
            average_latency: avg_latency,
            memory_usage: self.get_memory_usage(),
            file_size: self.get_file_size(),
            error_count: 0,
        }
    }

    /// 获取文件大小
    pub fn get_file_size(&self) -> u64 {
        let data_file = self.base_path.join("data.msgpack");
        if let Ok(metadata) = std::fs::metadata(data_file) {
            metadata.len()
        } else {
            0
        }
    }

    /// 保存到文件
    pub async fn save_to_file(&self) -> Result<()> {
        use tokio::fs::File;
        use tokio::io::AsyncWriteExt;

        let data = FileStorageData {
            version: "1.0.0".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            data: self.data.clone(),
            context_data: self.context_data.clone(),
            documents: self.documents.clone(),
        };

        // 使用 bincode 进行序列化（高效）
        let serialized = bincode::serialize(&data)
            .map_err(|e| crate::error::StorageError::Serialization(e.to_string()))?;

        let data_file = self.base_path.join("data.bin");
        let mut file = File::create(&data_file).await?;
        file.write_all(&serialized).await?;

        Ok(())
    }

    /// 从文件加载
    pub async fn load_from_file(&mut self) -> Result<()> {
        use tokio::fs::File;
        use tokio::io::AsyncReadExt;

        let data_file = self.base_path.join("data.bin");

        let mut file = match File::open(&data_file).await {
            Ok(f) => f,
            Err(_) => return Ok(()),
        };

        let mut contents = Vec::new();
        file.read_to_end(&mut contents).await?;

        if contents.is_empty() {
            return Ok(());
        }

        let data: FileStorageData = bincode::deserialize(&contents)
            .map_err(|e| crate::error::StorageError::Deserialization(e.to_string()))?;

        self.data = data.data;
        self.context_data = data.context_data;
        self.documents = data.documents;

        self.update_memory_usage();

        Ok(())
    }

    /// 更新内存使用量
    fn update_memory_usage(&self) {
        let mut total_size = 0;

        total_size += std::mem::size_of_val(&self.data);
        for (k, v) in &self.data {
            total_size += k.len() + v.len() * std::mem::size_of::<DocId>();
        }

        total_size += std::mem::size_of_val(&self.context_data);
        for (ctx_key, ctx_map) in &self.context_data {
            total_size += ctx_key.len();
            total_size += std::mem::size_of_val(ctx_map);
            for (term, ids) in ctx_map {
                total_size += term.len() + ids.len() * std::mem::size_of::<DocId>();
            }
        }

        total_size += std::mem::size_of_val(&self.documents);
        for (id, content) in &self.documents {
            total_size += std::mem::size_of_val(id) + content.len();
        }

        self.memory_usage.store(total_size, Ordering::Relaxed);
    }

    /// 记录操作开始时间
    fn record_operation_start(&self) -> Instant {
        Instant::now()
    }

    /// 记录操作完成
    fn record_operation_completion(&self, start_time: Instant) {
        let latency = start_time.elapsed().as_micros() as usize;
        self.operation_count.fetch_add(1, Ordering::Relaxed);
        self.total_latency.fetch_add(latency, Ordering::Relaxed);
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
        self.data.clear();
        self.context_data.clear();
        self.documents.clear();

        let data_file = self.base_path.join("data.bin");
        let _ = tokio::fs::remove_file(&data_file).await;

        self.update_memory_usage();
        self.is_open = false;
        Ok(())
    }

    async fn commit(&mut self, index: &Index, _replace: bool, _append: bool) -> Result<()> {
        let start_time = self.record_operation_start();

        for doc_ids in index.map.index.values() {
            for (term_str, ids) in doc_ids {
                self.data.insert(term_str.clone(), ids.clone());
            }
        }

        for ctx_map in index.ctx.index.values() {
            for (ctx_term, doc_ids) in ctx_map {
                self.context_data.entry("default".to_string())
                    .or_default()
                    .insert(ctx_term.clone(), doc_ids.clone());
            }
        }

        self.save_to_file().await?;
        self.update_memory_usage();
        self.record_operation_completion(start_time);
        Ok(())
    }

    async fn get(&self, key: &str, ctx: Option<&str>, limit: usize, offset: usize, _resolve: bool, _enrich: bool) -> Result<SearchResults> {
        let start_time = self.record_operation_start();

        let results = if let Some(ctx_key) = ctx {
            if let Some(ctx_map) = self.context_data.get(ctx_key) {
                if let Some(doc_ids) = ctx_map.get(key) {
                    apply_limit_offset(doc_ids, limit, offset)
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        } else {
            if let Some(doc_ids) = self.data.get(key) {
                apply_limit_offset(doc_ids, limit, offset)
            } else {
                Vec::new()
            }
        };

        self.record_operation_completion(start_time);
        Ok(results)
    }

    async fn enrich(&self, ids: &[DocId]) -> Result<EnrichedSearchResults> {
        let start_time = self.record_operation_start();
        let mut results = Vec::new();

        for &id in ids {
            if let Some(content) = self.documents.get(&id) {
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

        self.record_operation_completion(start_time);
        Ok(results)
    }

    async fn has(&self, id: DocId) -> Result<bool> {
        let start_time = self.record_operation_start();
        let result = Ok(self.documents.contains_key(&id));
        self.record_operation_completion(start_time);
        result
    }

    async fn remove(&mut self, ids: &[DocId]) -> Result<()> {
        let start_time = self.record_operation_start();

        for &id in ids {
            self.documents.remove(&id);

            for doc_ids in self.data.values_mut() {
                doc_ids.retain(|&doc_id| doc_id != id);
            }

            for ctx_map in self.context_data.values_mut() {
                for doc_ids in ctx_map.values_mut() {
                    doc_ids.retain(|&doc_id| doc_id != id);
                }
            }
        }

        self.save_to_file().await?;
        self.update_memory_usage();
        self.record_operation_completion(start_time);
        Ok(())
    }

    async fn clear(&mut self) -> Result<()> {
        let start_time = self.record_operation_start();

        self.data.clear();
        self.context_data.clear();
        self.documents.clear();

        self.save_to_file().await?;
        self.update_memory_usage();
        self.record_operation_completion(start_time);
        Ok(())
    }

    async fn info(&self) -> Result<StorageInfo> {
        Ok(StorageInfo {
            name: "FileStorage".to_string(),
            version: "1.0.0".to_string(),
            size: self.get_file_size(),
            document_count: self.documents.len(),
            index_count: self.data.len(),
            is_connected: self.is_open,
        })
    }
}

/// 应用限制和偏移的辅助函数
fn apply_limit_offset(results: &[DocId], limit: usize, offset: usize) -> SearchResults {
    if results.is_empty() {
        return Vec::new();
    }

    let start = offset.min(results.len());
    let end = if limit > 0 {
        (start + limit).min(results.len())
    } else {
        results.len()
    };

    results[start..end].to_vec()
}

/// 文件存储数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStorageData {
    pub version: String,
    pub timestamp: String,
    pub data: HashMap<String, Vec<DocId>>,
    pub context_data: HashMap<String, HashMap<String, Vec<DocId>>>,
    pub documents: HashMap<DocId, String>,
}

/// 文件存储性能指标
#[derive(Debug, Clone)]
pub struct FileStorageMetrics {
    pub operation_count: usize,
    pub average_latency: usize,
    pub memory_usage: usize,
    pub file_size: u64,
    pub error_count: usize,
}

/// 批量操作类型
#[derive(Debug, Clone)]
pub enum BatchOperation {
    Add { doc_id: DocId, content: String },
    Remove { doc_id: DocId },
    Update { doc_id: DocId, content: String },
    Query { key: String, ctx: Option<String> },
}

/// 批量操作结果
#[derive(Debug, Clone)]
pub struct BatchResult {
    pub success_count: usize,
    pub failure_count: usize,
    pub total_latency_us: usize,
    pub errors: Vec<String>,
}

/// 批量操作处理器
pub struct BatchOperationProcessor {
    batch_size: usize,
    max_concurrent: usize,
    queue: Vec<BatchOperation>,
}

/// 缓存条目
#[derive(Debug, Clone)]
pub struct CacheEntry<V> {
    pub value: V,
    pub timestamp: std::time::Instant,
    pub access_count: usize,
    pub size: usize,
}

impl<V> CacheEntry<V> {
    /// 检查是否过期
    pub fn is_expired(&self, ttl_seconds: u64) -> bool {
        if ttl_seconds == 0 {
            return false;
        }
        self.timestamp.elapsed().as_secs() > ttl_seconds
    }
}

/// 缓存配置
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub l1_max_size: usize,
    pub l2_max_size: usize,
    pub ttl_seconds: u64,
    pub enable_l2: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            l1_max_size: 1000,
            l2_max_size: 10000,
            ttl_seconds: 3600,
            enable_l2: true,
        }
    }
}

/// 多级缓存
pub struct MultiLevelCache<K, V>
where
    K: std::hash::Hash + Eq + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    l1_cache: lru::LruCache<K, CacheEntry<V>>,
    l2_cache: Option<lru::LruCache<K, CacheEntry<V>>>,
    config: CacheConfig,
    total_size: std::sync::atomic::AtomicUsize,
    hit_count: std::sync::atomic::AtomicUsize,
    miss_count: std::sync::atomic::AtomicUsize,
}

impl<K, V> MultiLevelCache<K, V>
where
    K: std::hash::Hash + Eq + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// 创建新的多级缓存
    pub fn new(config: CacheConfig) -> Self {
        let l1_cache = lru::LruCache::new(std::num::NonZeroUsize::new(config.l1_max_size).unwrap());
        let l2_cache = if config.enable_l2 {
            Some(lru::LruCache::new(std::num::NonZeroUsize::new(config.l2_max_size).unwrap()))
        } else {
            None
        };

        Self {
            l1_cache,
            l2_cache,
            config,
            total_size: std::sync::atomic::AtomicUsize::new(0),
            hit_count: std::sync::atomic::AtomicUsize::new(0),
            miss_count: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    /// 获取缓存值
    pub fn get(&mut self, key: &K) -> Option<V> {
        let ttl = self.config.ttl_seconds;
        
        // 检查 L1 缓存
        let l1_result = self.l1_cache.get(key);
        if let Some(entry) = l1_result {
            if !entry.is_expired(ttl) {
                self.hit_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                return Some(entry.value.clone());
            }
        }

        // 检查 L2 缓存
        if let Some(ref mut l2) = self.l2_cache {
            let l2_result = l2.get(key);
            if let Some(entry) = l2_result {
                if !entry.is_expired(ttl) {
                    self.hit_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    
                    // 提升到 L1 缓存
                    let new_entry = CacheEntry {
                        value: entry.value.clone(),
                        timestamp: std::time::Instant::now(),
                        access_count: entry.access_count + 1,
                        size: entry.size,
                    };
                    self.l1_cache.put(key.clone(), new_entry);
                    
                    return Some(entry.value.clone());
                }
            }
        }

        self.miss_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        None
    }

    /// 设置缓存值
    pub fn put(&mut self, key: K, value: V, size: usize) {
        let entry = CacheEntry {
            value: value.clone(),
            timestamp: std::time::Instant::now(),
            access_count: 0,
            size,
        };

        self.l1_cache.put(key.clone(), entry.clone());
        
        if let Some(ref mut l2) = self.l2_cache {
            l2.put(key, entry);
        }

        self.total_size.fetch_add(size, std::sync::atomic::Ordering::Relaxed);
    }

    /// 批量预热缓存
    pub async fn warmup<F, Fut>(&mut self, keys: Vec<K>, fetch_fn: F) -> Result<()>
    where
        K: std::fmt::Debug,
        F: Fn(K) -> Fut + Send + Sync,
        Fut: std::future::Future<Output = Result<(V, usize)>> + Send,
    {
        for key in keys {
            if self.get(&key).is_none() {
                match fetch_fn(key.clone()).await {
                    Ok((value, size)) => {
                        self.put(key, value, size);
                    }
                    Err(e) => {
                        eprintln!("Failed to warmup cache for key {:?}: {}", key, e);
                    }
                }
            }
        }
        Ok(())
    }

    /// 获取缓存统计信息
    pub fn get_stats(&self) -> CacheStats {
        let hit_count = self.hit_count.load(std::sync::atomic::Ordering::Relaxed);
        let miss_count = self.miss_count.load(std::sync::atomic::Ordering::Relaxed);
        let total = hit_count + miss_count;
        
        let hit_rate = if total > 0 {
            (hit_count as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        CacheStats {
            hit_count,
            miss_count,
            hit_rate,
            total_size: self.total_size.load(std::sync::atomic::Ordering::Relaxed),
            l1_size: self.l1_cache.len(),
            l2_size: self.l2_cache.as_ref().map(|c| c.len()).unwrap_or(0),
        }
    }

    /// 清空缓存
    pub fn clear(&mut self) {
        self.l1_cache.clear();
        if let Some(ref mut l2) = self.l2_cache {
            l2.clear();
        }
        self.total_size.store(0, std::sync::atomic::Ordering::Relaxed);
        self.hit_count.store(0, std::sync::atomic::Ordering::Relaxed);
        self.miss_count.store(0, std::sync::atomic::Ordering::Relaxed);
    }

    /// 获取配置
    pub fn config(&self) -> &CacheConfig {
        &self.config
    }
}

/// 缓存统计信息
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hit_count: usize,
    pub miss_count: usize,
    pub hit_rate: f64,
    pub total_size: usize,
    pub l1_size: usize,
    pub l2_size: usize,
}

impl BatchOperationProcessor {
    /// 创建新的批量操作处理器
    pub fn new(batch_size: usize, max_concurrent: usize) -> Self {
        Self {
            batch_size,
            max_concurrent,
            queue: Vec::new(),
        }
    }

    /// 添加操作到队列
    pub fn add_operation(&mut self, operation: BatchOperation) {
        self.queue.push(operation);
    }

    /// 执行批量操作（流水线处理）
    pub async fn execute_pipeline<F, Fut>(
        &mut self,
        operation: F,
    ) -> BatchResult
    where
        F: Fn(Vec<BatchOperation>) -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = Result<()>> + Send + 'static,
    {
        let start_time = std::time::Instant::now();
        let mut result = BatchResult {
            success_count: 0,
            failure_count: 0,
            total_latency_us: 0,
            errors: Vec::new(),
        };

        // 分批处理
        for batch in self.queue.chunks(self.batch_size) {
            let batch_vec: Vec<BatchOperation> = batch.to_vec();
            
            match operation(batch_vec).await {
                Ok(_) => {
                    result.success_count += batch.len();
                }
                Err(e) => {
                    result.failure_count += batch.len();
                    result.errors.push(e.to_string());
                }
            }
        }

        // 清空队列
        self.queue.clear();
        result.total_latency_us = start_time.elapsed().as_micros() as usize;

        result
    }

    /// 并行执行批量操作
    pub async fn execute_parallel<F, Fut>(
        &mut self,
        operation: F,
    ) -> BatchResult
    where
        F: Fn(BatchOperation) -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = Result<()>> + Send + 'static,
    {
        use tokio::sync::Semaphore;
        use std::sync::Arc;

        let start_time = std::time::Instant::now();
        let semaphore = Arc::new(Semaphore::new(self.max_concurrent));
        let mut result = BatchResult {
            success_count: 0,
            failure_count: 0,
            total_latency_us: 0,
            errors: Vec::new(),
        };

        let mut tasks = Vec::new();

        for op in self.queue.drain(..).collect::<Vec<_>>() {
            let semaphore = semaphore.clone();
            let op_func = operation.clone();
            
            let task = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                op_func(op).await
            });
            
            tasks.push(task);
        }

        // 等待所有任务完成
        for task in tasks {
            match task.await {
                Ok(Ok(_)) => {
                    result.success_count += 1;
                }
                Ok(Err(e)) => {
                    result.failure_count += 1;
                    result.errors.push(e.to_string());
                }
                Err(e) => {
                    result.failure_count += 1;
                    result.errors.push(format!("Task error: {}", e));
                }
            }
        }

        result.total_latency_us = start_time.elapsed().as_micros() as usize;

        result
    }

    /// 获取队列大小
    pub fn queue_size(&self) -> usize {
        self.queue.len()
    }

    /// 清空队列
    pub fn clear_queue(&mut self) {
        self.queue.clear();
    }
}

/// WAL 存储实现 - 支持增量持久化
pub struct WALStorage {
    wal_manager: wal::WALManager,
    documents: HashMap<DocId, String>,
    is_open: bool,
}

impl WALStorage {
    /// 创建新的 WAL 存储
    pub async fn new(config: wal::WALConfig) -> Result<Self> {
        let wal_manager = wal::WALManager::new(config).await?;

        Ok(Self {
            wal_manager,
            documents: HashMap::new(),
            is_open: false,
        })
    }

    /// 创建快照
    pub async fn create_snapshot(&self, index: &Index) -> Result<()> {
        self.wal_manager.create_snapshot(index).await
    }
}

#[async_trait::async_trait]
impl StorageInterface for WALStorage {
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
        self.documents.clear();
        self.wal_manager.clear().await?;
        self.is_open = false;
        Ok(())
    }

    async fn commit(&mut self, index: &Index, _replace: bool, _append: bool) -> Result<()> {
        // 使用 WAL 创建快照
        self.wal_manager.create_snapshot(index).await
    }

    async fn get(&self, key: &str, ctx: Option<&str>, limit: usize, offset: usize, _resolve: bool, _enrich: bool) -> Result<SearchResults> {
        // WAL 存储需要通过加载索引来获取数据
        // 这里简化处理，返回空结果
        // 实际应用中应该维护一个内存索引
        let _ = (key, ctx, limit, offset);
        Ok(Vec::new())
    }

    async fn enrich(&self, ids: &[DocId]) -> Result<EnrichedSearchResults> {
        let mut results = Vec::new();

        for &id in ids {
            if let Some(content) = self.documents.get(&id) {
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
        Ok(self.documents.contains_key(&id))
    }

    async fn remove(&mut self, ids: &[DocId]) -> Result<()> {
        for &id in ids {
            self.documents.remove(&id);
            self.wal_manager.record_change(wal::IndexChange::Remove { doc_id: id }).await?;
        }
        Ok(())
    }

    async fn clear(&mut self) -> Result<()> {
        self.documents.clear();
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
            document_count: self.documents.len(),
            index_count: 0,
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