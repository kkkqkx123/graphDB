# 批量操作分析文档

## 概述

本文档分析 GraphDB 项目中批量操作的当前实现状态，并提出可进一步改进的方向。

## 当前已实现的批量操作

### 1. 批量操作接口

**实现位置**: `src/storage/storage_client.rs`

**接口定义**:
```rust
pub trait StorageClient: Send + Sync + std::fmt::Debug {
    /// 批量插入顶点
    fn batch_insert_vertices(
        &mut self, 
        space: &str, 
        vertices: Vec<Vertex>
    ) -> Result<Vec<Value>, StorageError>;
    
    /// 批量插入边
    fn batch_insert_edges(
        &mut self, 
        space: &str, 
        edges: Vec<Edge>
    ) -> Result<(), StorageError>;
    
    // ... 其他批量操作
}
```

### 2. Redb 批量写入实现

**实现位置**: `src/storage/operations/redb_operations.rs`

**关键代码**:
```rust
impl RedbWriter {
    /// 批量写入操作
    pub fn batch_write(
        &mut self, 
        operations: Vec<WriteOperation>
    ) -> Result<(), StorageError> {
        let txn = self.db.begin_write()
            .map_err(|e| StorageError::DbError(format!("开始事务失败: {}", e)))?;
        
        {
            let mut nodes_table = txn.open_table(NODES_TABLE)
                .map_err(|e| StorageError::DbError(format!("打开表失败: {}", e)))?;
            let mut edges_table = txn.open_table(EDGES_TABLE)
                .map_err(|e| StorageError::DbError(format!("打开表失败: {}", e)))?;
            
            for op in operations {
                match op {
                    WriteOperation::InsertVertex(vertex) => {
                        let key = serialize_vertex_key(&vertex.id)?;
                        let value = serialize_vertex(&vertex)?;
                        nodes_table.insert(&key, &value)
                            .map_err(|e| StorageError::DbError(e.to_string()))?;
                    }
                    WriteOperation::InsertEdge(edge) => {
                        let key = serialize_edge_key(&edge.src, &edge.dst, &edge.edge_type)?;
                        let value = serialize_edge(&edge)?;
                        edges_table.insert(&key, &value)
                            .map_err(|e| StorageError::DbError(e.to_string()))?;
                    }
                    // ... 其他操作
                }
            }
        }
        
        txn.commit()
            .map_err(|e| StorageError::DbError(format!("提交事务失败: {}", e)))?;
        
        Ok(())
    }
}
```

**优化点**:
- 所有操作在同一事务中提交，减少事务开销
- 批量操作比单条操作性能提升显著

### 3. 嵌入式批量插入器

**实现位置**: `src/api/embedded/batch.rs`

**功能说明**:
- 提供高级批量插入 API
- 自动分批处理（避免单批次过大）
- 支持进度回调

**关键代码**:
```rust
pub struct BatchInserter<'a> {
    storage: &'a mut dyn StorageClient,
    space: String,
    vertex_buffer: Vec<Vertex>,
    edge_buffer: Vec<Edge>,
    batch_size: usize,
    stats: BatchStats,
}

#[derive(Debug, Clone, Default)]
pub struct BatchStats {
    pub vertices_inserted: usize,
    pub edges_inserted: usize,
    pub batches_committed: usize,
    pub errors: Vec<BatchError>,
}

impl<'a> BatchInserter<'a> {
    /// 创建新的批量插入器
    pub fn new(
        storage: &'a mut dyn StorageClient,
        space: &str,
        batch_size: usize,
    ) -> Self {
        Self {
            storage,
            space: space.to_string(),
            vertex_buffer: Vec::with_capacity(batch_size),
            edge_buffer: Vec::with_capacity(batch_size),
            batch_size,
            stats: BatchStats::default(),
        }
    }
    
    /// 添加顶点（自动分批）
    pub fn add_vertex(&mut self, vertex: Vertex) -> Result<(), BatchError> {
        self.vertex_buffer.push(vertex);
        
        if self.vertex_buffer.len() >= self.batch_size {
            self.flush_vertices()?;
        }
        
        Ok(())
    }
    
    /// 提交当前批次
    pub fn flush(&mut self) -> Result<(), BatchError> {
        self.flush_vertices()?;
        self.flush_edges()?;
        Ok(())
    }
    
    fn flush_vertices(&mut self) -> Result<(), BatchError> {
        if self.vertex_buffer.is_empty() {
            return Ok(());
        }
        
        let vertices = std::mem::take(&mut self.vertex_buffer);
        match self.storage.batch_insert_vertices(&self.space, vertices) {
            Ok(ids) => {
                self.stats.vertices_inserted += ids.len();
                self.stats.batches_committed += 1;
                Ok(())
            }
            Err(e) => {
                self.stats.errors.push(BatchError {
                    operation: "batch_insert_vertices".to_string(),
                    error: e.to_string(),
                });
                Err(BatchError {
                    operation: "batch_insert_vertices".to_string(),
                    error: e.to_string(),
                })
            }
        }
    }
}
```

## 可进一步改进的方向

### 1. 流式批量导入（Streaming Batch Import）

**需求场景**:
- 超大数据集导入（GB/TB 级别）
- 内存有限，无法一次性加载所有数据
- 从外部数据源（CSV、JSON、数据库）导入

**建议实现**:

```rust
use tokio::sync::mpsc;
use futures::stream::Stream;

/// 流式批量导入器
pub struct StreamingBatchImporter {
    config: StreamingConfig,
    stats: Arc<RwLock<ImportStats>>,
}

pub struct StreamingConfig {
    /// 批次大小
    pub batch_size: usize,
    /// 并发写入数
    pub concurrency: usize,
    /// 缓冲区大小（批次数）
    pub buffer_size: usize,
    /// 是否启用索引更新
    pub update_indexes: bool,
    /// 错误处理策略
    pub error_policy: ErrorPolicy,
}

pub enum ErrorPolicy {
    /// 遇到错误立即停止
    FailFast,
    /// 跳过错误继续
    SkipError,
    /// 记录错误继续
    LogAndContinue,
}

/// 导入统计
#[derive(Debug, Default)]
pub struct ImportStats {
    pub total_read: AtomicU64,
    pub total_inserted: AtomicU64,
    pub total_failed: AtomicU64,
    pub bytes_processed: AtomicU64,
    pub start_time: Option<Instant>,
    pub end_time: Option<Instant>,
}

impl StreamingBatchImporter {
    /// 从流导入顶点
    pub async fn import_vertices_from_stream<S>(
        &self,
        storage: Arc<Mutex<dyn StorageClient>>,
        space: &str,
        stream: S,
    ) -> Result<ImportStats, ImportError>
    where
        S: Stream<Item = Result<Vertex, ParseError>> + Send + 'static,
    {
        let (tx, rx) = mpsc::channel::<Vec<Vertex>>(self.config.buffer_size);
        let stats = Arc::clone(&self.stats);
        
        // 启动写入工作线程
        let writer_handles: Vec<_> = (0..self.config.concurrency)
            .map(|i| {
                let storage = Arc::clone(&storage);
                let rx = rx.clone();
                let stats = Arc::clone(&stats);
                let space = space.to_string();
                
                tokio::spawn(async move {
                    Self::writer_worker(i, storage, rx, stats, space).await
                })
            })
            .collect();
        
        // 读取并分批
        let mut batch = Vec::with_capacity(self.config.batch_size);
        let mut stream = Box::pin(stream);
        
        while let Some(result) = stream.next().await {
            match result {
                Ok(vertex) => {
                    batch.push(vertex);
                    
                    if batch.len() >= self.config.batch_size {
                        let batch_to_send = std::mem::replace(
                            &mut batch, 
                            Vec::with_capacity(self.config.batch_size)
                        );
                        
                        if tx.send(batch_to_send).await.is_err() {
                            break; // 接收端已关闭
                        }
                    }
                }
                Err(e) => {
                    match self.config.error_policy {
                        ErrorPolicy::FailFast => return Err(ImportError::ParseError(e)),
                        ErrorPolicy::SkipError => continue,
                        ErrorPolicy::LogAndContinue => {
                            log::warn!("解析错误: {}", e);
                            continue;
                        }
                    }
                }
            }
        }
        
        // 发送剩余数据
        if !batch.is_empty() {
            let _ = tx.send(batch).await;
        }
        
        // 关闭发送端
        drop(tx);
        
        // 等待所有写入完成
        for handle in writer_handles {
            handle.await??;
        }
        
        let final_stats = self.stats.read().await.clone();
        Ok(final_stats)
    }
    
    /// 写入工作线程
    async fn writer_worker(
        id: usize,
        storage: Arc<Mutex<dyn StorageClient>>,
        mut rx: mpsc::Receiver<Vec<Vertex>>,
        stats: Arc<RwLock<ImportStats>>,
        space: String,
    ) -> Result<(), ImportError> {
        while let Some(batch) = rx.recv().await {
            let batch_size = batch.len();
            
            let mut storage = storage.lock().await;
            match storage.batch_insert_vertices(&space, batch) {
                Ok(_) => {
                    stats.write().await.total_inserted
                        .fetch_add(batch_size as u64, Ordering::Relaxed);
                }
                Err(e) => {
                    stats.write().await.total_failed
                        .fetch_add(batch_size as u64, Ordering::Relaxed);
                    log::error!("Worker {} 写入失败: {}", id, e);
                }
            }
        }
        
        Ok(())
    }
}
```

**使用示例**:
```rust
// 从 CSV 流导入
let file = File::open("vertices.csv").await?;
let reader = BufReader::new(file);
let csv_stream = CsvVertexStream::new(reader);

let importer = StreamingBatchImporter::new(StreamingConfig {
    batch_size: 10000,
    concurrency: 4,
    buffer_size: 10,
    update_indexes: true,
    error_policy: ErrorPolicy::LogAndContinue,
});

let stats = importer.import_vertices_from_stream(
    storage, 
    "my_space", 
    csv_stream
).await?;

println!("导入完成: {} 成功, {} 失败", 
    stats.total_inserted, 
    stats.total_failed
);
```

**实现复杂度**: 中
**预期收益**: 高（大数据导入场景）

### 2. 并发批量写入

**需求场景**:
- 利用多核 CPU 并行处理
- 提高写入吞吐量
- 单节点多线程写入

**建议实现**:

```rust
use crossbeam::channel::{bounded, Sender, Receiver};
use rayon::prelude::*;

/// 并发批量写入器
pub struct ConcurrentBatchWriter {
    config: ConcurrentConfig,
    workers: Vec<WorkerHandle>,
}

pub struct ConcurrentConfig {
    /// 工作线程数
    pub worker_count: usize,
    /// 每批次大小
    pub batch_size: usize,
    /// 通道缓冲区大小
    pub channel_capacity: usize,
}

struct WorkerHandle {
    thread: std::thread::JoinHandle<()>,
    sender: Sender<WriteTask>,
}

struct WriteTask {
    space: String,
    vertices: Vec<Vertex>,
    edges: Vec<Edge>,
    result_sender: oneshot::Sender<WriteResult>,
}

struct WriteResult {
    pub success_count: usize,
    pub failed_count: usize,
    pub errors: Vec<StorageError>,
}

impl ConcurrentBatchWriter {
    pub fn new(config: ConcurrentConfig) -> Self {
        let mut workers = Vec::with_capacity(config.worker_count);
        
        for id in 0..config.worker_count {
            let (tx, rx): (Sender<WriteTask>, Receiver<WriteTask>) = 
                bounded(config.channel_capacity);
            
            let thread = std::thread::spawn(move || {
                Self::worker_loop(id, rx);
            });
            
            workers.push(WorkerHandle { thread, sender: tx });
        }
        
        Self { config, workers }
    }
    
    /// 提交写入任务（自动路由到工作线程）
    pub async fn submit(
        &self,
        space: &str,
        vertices: Vec<Vertex>,
        edges: Vec<Edge>,
    ) -> Result<WriteResult, WriteError> {
        // 根据空间名称哈希选择工作线程
        // 同一空间的数据路由到同一线程，保证顺序性
        let worker_id = Self::hash_space(space) % self.workers.len();
        
        let (tx, rx) = oneshot::channel();
        let task = WriteTask {
            space: space.to_string(),
            vertices,
            edges,
            result_sender: tx,
        };
        
        self.workers[worker_id]
            .sender
            .send(task)
            .map_err(|_| WriteError::ChannelClosed)?;
        
        rx.await.map_err(|_| WriteError::RecvError)
    }
    
    /// 批量提交（并行处理）
    pub async fn submit_batch(
        &self,
        tasks: Vec<WriteBatch>,
    ) -> Vec<Result<WriteResult, WriteError>> {
        let futures: Vec<_> = tasks
            .into_iter()
            .map(|task| {
                self.submit(&task.space, task.vertices, task.edges)
            })
            .collect();
        
        futures::future::join_all(futures).await
    }
    
    fn worker_loop(id: usize, receiver: Receiver<WriteTask>) {
        // 每个工作线程有自己的存储连接
        let mut storage = Self::create_storage_connection();
        
        while let Ok(task) = receiver.recv() {
            let mut result = WriteResult {
                success_count: 0,
                failed_count: 0,
                errors: Vec::new(),
            };
            
            // 写入顶点
            if !task.vertices.is_empty() {
                match storage.batch_insert_vertices(&task.space, task.vertices) {
                    Ok(ids) => result.success_count += ids.len(),
                    Err(e) => {
                        result.failed_count += 1;
                        result.errors.push(e);
                    }
                }
            }
            
            // 写入边
            if !task.edges.is_empty() {
                match storage.batch_insert_edges(&task.space, task.edges) {
                    Ok(_) => result.success_count += task.edges.len(),
                    Err(e) => {
                        result.failed_count += task.edges.len();
                        result.errors.push(e);
                    }
                }
            }
            
            // 返回结果
            let _ = task.result_sender.send(result);
        }
        
        log::info!("Worker {} 退出", id);
    }
    
    fn hash_space(space: &str) -> usize {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        space.hash(&mut hasher);
        hasher.finish() as usize
    }
}
```

**使用示例**:
```rust
let writer = ConcurrentBatchWriter::new(ConcurrentConfig {
    worker_count: 4,
    batch_size: 10000,
    channel_capacity: 100,
});

// 并行导入多个空间的数据
let tasks = vec![
    WriteBatch {
        space: "space1".to_string(),
        vertices: load_vertices("space1_data.csv"),
        edges: vec![],
    },
    WriteBatch {
        space: "space2".to_string(),
        vertices: load_vertices("space2_data.csv"),
        edges: vec![],
    },
];

let results = writer.submit_batch(tasks).await;
```

**实现复杂度**: 中
**预期收益**: 中（多核 CPU 利用）

### 3. 批量操作进度追踪

**需求场景**:
- 长时间导入任务需要进度反馈
- 错误记录和恢复
- 性能监控

**建议实现**:

```rust
use serde::{Serialize, Deserialize};

/// 批量操作进度追踪器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchProgress {
    /// 任务 ID
    pub task_id: String,
    /// 任务类型
    pub task_type: BatchTaskType,
    /// 状态
    pub status: BatchStatus,
    /// 总记录数（预估）
    pub total_records: Option<u64>,
    /// 已处理记录数
    pub processed_records: u64,
    /// 成功记录数
    pub success_records: u64,
    /// 失败记录数
    pub failed_records: u64,
    /// 开始时间
    pub start_time: SystemTime,
    /// 预计完成时间
    pub estimated_completion: Option<SystemTime>,
    /// 处理速率（记录/秒）
    pub processing_rate: f64,
    /// 错误详情
    pub errors: Vec<FailedRecord>,
    /// 阶段信息
    pub stages: Vec<StageInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BatchTaskType {
    ImportVertices,
    ImportEdges,
    RebuildIndex,
    DataMigration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BatchStatus {
    Pending,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedRecord {
    pub record_number: u64,
    pub record_data: String,
    pub error_message: String,
    pub timestamp: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageInfo {
    pub name: String,
    pub status: BatchStatus,
    pub start_time: Option<SystemTime>,
    pub end_time: Option<SystemTime>,
    pub progress_percent: f64,
}

/// 进度追踪器
pub struct ProgressTracker {
    progress: Arc<RwLock<BatchProgress>>,
    checkpoint_store: Arc<dyn CheckpointStore>,
}

impl ProgressTracker {
    /// 创建新的追踪任务
    pub async fn create_task(
        &self,
        task_type: BatchTaskType,
        total_records: Option<u64>,
    ) -> String {
        let task_id = Uuid::new_v4().to_string();
        let progress = BatchProgress {
            task_id: task_id.clone(),
            task_type,
            status: BatchStatus::Pending,
            total_records,
            processed_records: 0,
            success_records: 0,
            failed_records: 0,
            start_time: SystemTime::now(),
            estimated_completion: None,
            processing_rate: 0.0,
            errors: Vec::new(),
            stages: Vec::new(),
        };
        
        self.checkpoint_store.save(&task_id, &progress).await.ok();
        
        let mut guard = self.progress.write().await;
        *guard = progress;
        
        task_id
    }
    
    /// 更新进度
    pub async fn update_progress(
        &self,
        processed: u64,
        success: u64,
        failed: u64,
    ) {
        let mut progress = self.progress.write().await;
        
        progress.processed_records = processed;
        progress.success_records = success;
        progress.failed_records = failed;
        
        // 计算处理速率
        let elapsed = progress.start_time.elapsed().unwrap_or_default();
        if elapsed.as_secs() > 0 {
            progress.processing_rate = processed as f64 / elapsed.as_secs_f64();
        }
        
        // 估算完成时间
        if let Some(total) = progress.total_records {
            if progress.processing_rate > 0.0 {
                let remaining = (total - processed) as f64 / progress.processing_rate;
                progress.estimated_completion = Some(
                    SystemTime::now() + Duration::from_secs_f64(remaining)
                );
            }
        }
        
        // 定期保存检查点
        if processed % 10000 == 0 {
            self.checkpoint_store.save(&progress.task_id, &*progress).await.ok();
        }
    }
    
    /// 记录错误
    pub async fn record_error(&self, record: FailedRecord) {
        let mut progress = self.progress.write().await;
        progress.errors.push(record);
        
        // 限制错误记录数量，避免内存溢出
        if progress.errors.len() > 1000 {
            progress.errors.remove(0);
        }
    }
    
    /// 获取当前进度
    pub async fn get_progress(&self) -> BatchProgress {
        self.progress.read().await.clone()
    }
    
    /// 从检查点恢复
    pub async fn restore_from_checkpoint(&self, task_id: &str) -> Option<BatchProgress> {
        self.checkpoint_store.load(task_id).await.ok()
    }
}

/// 检查点存储 trait
#[async_trait]
pub trait CheckpointStore: Send + Sync {
    async fn save(&self, task_id: &str, progress: &BatchProgress) -> Result<(), Error>;
    async fn load(&self, task_id: &str) -> Result<BatchProgress, Error>;
}

/// 文件检查点存储
pub struct FileCheckpointStore {
    dir: PathBuf,
}

#[async_trait]
impl CheckpointStore for FileCheckpointStore {
    async fn save(&self, task_id: &str, progress: &BatchProgress) -> Result<(), Error> {
        let path = self.dir.join(format!("{}.json", task_id));
        let json = serde_json::to_string_pretty(progress)?;
        tokio::fs::write(&path, json).await?;
        Ok(())
    }
    
    async fn load(&self, task_id: &str) -> Result<BatchProgress, Error> {
        let path = self.dir.join(format!("{}.json", task_id));
        let json = tokio::fs::read_to_string(&path).await?;
        let progress = serde_json::from_str(&json)?;
        Ok(progress)
    }
}
```

**使用示例**:
```rust
let tracker = ProgressTracker::new(
    Arc::new(FileCheckpointStore::new("./checkpoints"))
);

let task_id = tracker.create_task(BatchTaskType::ImportVertices, Some(1000000)).await;

// 在导入过程中更新进度
for batch in batches {
    match import_batch(batch).await {
        Ok(count) => {
            tracker.update_progress(
                processed_count, 
                success_count + count, 
                failed_count
            ).await;
        }
        Err(e) => {
            tracker.record_error(FailedRecord {
                record_number: current_record,
                record_data: format!("{:?}", batch),
                error_message: e.to_string(),
                timestamp: SystemTime::now(),
            }).await;
        }
    }
}

// 查询进度
let progress = tracker.get_progress().await;
println!("进度: {}/{}", progress.processed_records, progress.total_records.unwrap_or(0));
println!("速率: {:.2} 记录/秒", progress.processing_rate);
println!("预计完成: {:?}", progress.estimated_completion);
```

**实现复杂度**: 低
**预期收益**: 中（用户体验提升）

### 4. 批量导入格式支持

**需求场景**:
- 支持多种数据格式（CSV、JSON、Parquet）
- 支持压缩文件
- 支持网络数据源

**建议实现**:

```rust
/// 批量导入器
pub struct BulkImporter {
    parsers: HashMap<String, Box<dyn DataParser>>,
}

/// 数据解析器 trait
#[async_trait]
pub trait DataParser: Send + Sync {
    async fn parse_vertices<R: AsyncRead + Unpin>(
        &self,
        reader: R,
        config: ParseConfig,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<Vertex, ParseError>> + Send>>, ParseError>;
    
    async fn parse_edges<R: AsyncRead + Unpin>(
        &self,
        reader: R,
        config: ParseConfig,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<Edge, ParseError>> + Send>>, ParseError>;
}

/// CSV 解析器
pub struct CsvParser;

#[async_trait]
impl DataParser for CsvParser {
    async fn parse_vertices<R: AsyncRead + Unpin>(
        &self,
        reader: R,
        config: ParseConfig,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<Vertex, ParseError>> + Send>>, ParseError> {
        let mut csv_reader = csv_async::AsyncReader::from_reader(reader);
        
        let stream = try_stream! {
            let headers = csv_reader.headers().await?;
            
            let mut records = csv_reader.records();
            while let Some(record) = records.next().await {
                let record = record?;
                yield Self::record_to_vertex(&headers, &record, &config)?;
            }
        };
        
        Ok(Box::pin(stream))
    }
}

/// JSON 解析器
pub struct JsonParser;

#[async_trait]
impl DataParser for JsonParser {
    async fn parse_vertices<R: AsyncRead + Unpin>(
        &self,
        reader: R,
        config: ParseConfig,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<Vertex, ParseError>> + Send>>, ParseError> {
        let stream = try_stream! {
            let mut deserializer = serde_json::Deserializer::from_reader(reader);
            
            if let Ok(array) = serde_json::Value::deserialize(&mut deserializer) {
                if let Some(arr) = array.as_array() {
                    for value in arr {
                        yield Self::json_to_vertex(value, &config)?;
                    }
                }
            }
        };
        
        Ok(Box::pin(stream))
    }
}

/// 压缩支持
pub struct CompressedReader<R: AsyncRead + Unpin> {
    inner: Box<dyn AsyncRead + Send + Unpin>,
}

impl<R: AsyncRead + Unpin> CompressedReader<R> {
    pub async fn new(reader: R, compression: CompressionType) -> Result<Self, Error> {
        let inner: Box<dyn AsyncRead + Send + Unpin> = match compression {
            CompressionType::Gzip => {
                Box::new(async_compression::tokio::bufread::GzipDecoder::new(BufReader::new(reader)))
            }
            CompressionType::Zstd => {
                Box::new(async_compression::tokio::bufread::ZstdDecoder::new(BufReader::new(reader)))
            }
            CompressionType::None => Box::new(reader),
        };
        
        Ok(Self { inner })
    }
}

/// 使用示例
impl BulkImporter {
    pub async fn import_from_file(
        &self,
        path: &Path,
        format: DataFormat,
        compression: CompressionType,
        storage: Arc<Mutex<dyn StorageClient>>,
        space: &str,
    ) -> Result<ImportStats, ImportError> {
        let file = File::open(path).await?;
        let reader = CompressedReader::new(file, compression).await?;
        
        let parser = self.parsers.get(&format.to_string())
            .ok_or(ImportError::UnsupportedFormat)?;
        
        let vertex_stream = parser.parse_vertices(reader, ParseConfig::default()).await?;
        
        let importer = StreamingBatchImporter::new(StreamingConfig::default());
        let stats = importer.import_vertices_from_stream(storage, space, vertex_stream).await?;
        
        Ok(stats)
    }
}
```

**实现复杂度**: 中
**预期收益**: 中（易用性提升）

## 总结

### 已实现的优势

1. **基础批量接口** - `batch_insert_vertices`/`batch_insert_edges`
2. **事务批量提交** - 单事务多操作，减少开销
3. **自动分批** - `BatchInserter` 自动管理批次大小
4. **错误处理** - 批量操作中的错误记录

### 建议优先级

| 优先级 | 优化项 | 预期收益 | 实现复杂度 |
|-------|--------|---------|-----------|
| P0 | 流式批量导入 | 高 | 中 |
| P1 | 批量操作进度追踪 | 中 | 低 |
| P2 | 并发批量写入 | 中 | 中 |
| P3 | 批量导入格式支持 | 中 | 中 |

**建议**: 优先实现流式批量导入，这是大数据量导入的基础能力，可显著提升导入性能和内存效率。
