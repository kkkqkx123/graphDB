# Phase 4: 数据同步机制实现方案

## 阶段目标

实现图数据变更与全文索引的异步同步机制，确保数据一致性，同时不影响主事务性能。

**预计工期**: 4-6 天  
**前置依赖**: Phase 3 (查询引擎集成)

---

## 新增文件清单

### 1. 同步核心

| 文件路径 | 说明 |
|---------|------|
| `src/sync/mod.rs` | 同步模块入口 |
| `src/sync/manager.rs` | 同步管理器 |
| `src/sync/task.rs` | 同步任务定义 |
| `src/sync/queue.rs` | 异步任务队列 |

### 2. 批量处理

| 文件路径 | 说明 |
|---------|------|
| `src/sync/batch.rs` | 批量提交处理器 |
| `src/sync/scheduler.rs` | 调度器 |

### 3. 持久化

| 文件路径 | 说明 |
|---------|------|
| `src/sync/persistence.rs` | 同步状态持久化 |
| `src/sync/recovery.rs` | 故障恢复 |

---

## 详细实现方案

### 1. 同步任务定义

**文件**: `src/sync/task.rs`

```rust
use crate::core::{Value, Vertex};
use crate::coordinator::ChangeType;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 同步任务类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncTask {
    /// 顶点变更
    VertexChange {
        /// 任务ID
        task_id: String,
        /// 图空间ID
        space_id: u64,
        /// Tag 名称
        tag_name: String,
        /// 顶点ID
        vertex_id: Value,
        /// 变更的属性
        properties: Vec<(String, Value)>,
        /// 变更类型
        change_type: ChangeType,
        /// 创建时间
        created_at: DateTime<Utc>,
    },
    /// 批量索引
    BatchIndex {
        /// 任务ID
        task_id: String,
        /// 图空间ID
        space_id: u64,
        /// Tag 名称
        tag_name: String,
        /// 字段名称
        field_name: String,
        /// 文档列表
        documents: Vec<(String, String)>, // (doc_id, content)
        /// 创建时间
        created_at: DateTime<Utc>,
    },
    /// 提交索引
    CommitIndex {
        /// 任务ID
        task_id: String,
        /// 图空间ID
        space_id: u64,
        /// Tag 名称
        tag_name: String,
        /// 字段名称
        field_name: String,
        /// 创建时间
        created_at: DateTime<Utc>,
    },
    /// 重建索引
    RebuildIndex {
        /// 任务ID
        task_id: String,
        /// 图空间ID
        space_id: u64,
        /// Tag 名称
        tag_name: String,
        /// 字段名称
        field_name: String,
        /// 创建时间
        created_at: DateTime<Utc>,
    },
}

impl SyncTask {
    /// 创建顶点变更任务
    pub fn vertex_change(
        space_id: u64,
        tag_name: &str,
        vertex_id: &Value,
        properties: Vec<(String, Value)>,
        change_type: ChangeType,
    ) -> Self {
        Self::VertexChange {
            task_id: generate_task_id(),
            space_id,
            tag_name: tag_name.to_string(),
            vertex_id: vertex_id.clone(),
            properties,
            change_type,
            created_at: Utc::now(),
        }
    }
    
    /// 获取任务ID
    pub fn task_id(&self) -> &str {
        match self {
            Self::VertexChange { task_id, .. } => task_id,
            Self::BatchIndex { task_id, .. } => task_id,
            Self::CommitIndex { task_id, .. } => task_id,
            Self::RebuildIndex { task_id, .. } => task_id,
        }
    }
    
    /// 获取创建时间
    pub fn created_at(&self) -> DateTime<Utc> {
        match self {
            Self::VertexChange { created_at, .. } => *created_at,
            Self::BatchIndex { created_at, .. } => *created_at,
            Self::CommitIndex { created_at, .. } => *created_at,
            Self::RebuildIndex { created_at, .. } => *created_at,
        }
    }
}

fn generate_task_id() -> String {
    use uuid::Uuid;
    Uuid::new_v4().to_string()
}

/// 任务执行结果
#[derive(Debug, Clone)]
pub enum TaskResult {
    Success,
    Failed(String),
    Retryable(String),
}
```

### 2. 异步任务队列

**文件**: `src/sync/queue.rs`

```rust
use tokio::sync::{mpsc, Mutex};
use std::collections::VecDeque;
use crate::sync::task::SyncTask;

/// 异步同步任务队列
pub struct SyncTaskQueue {
    /// 发送端
    sender: mpsc::Sender<SyncTask>,
    /// 接收端（共享）
    receiver: Mutex<mpsc::Receiver<SyncTask>>,
    /// 队列容量
    capacity: usize,
}

impl SyncTaskQueue {
    /// 创建新的任务队列
    pub fn new(capacity: usize) -> Self {
        let (sender, receiver) = mpsc::channel(capacity);
        Self {
            sender,
            receiver: Mutex::new(receiver),
            capacity,
        }
    }
    
    /// 提交任务
    /// 
    /// 如果队列已满，返回错误（不阻塞主流程）
    pub async fn submit(&self, task: SyncTask) -> Result<(), QueueError> {
        match self.sender.try_send(task) {
            Ok(_) => Ok(()),
            Err(mpsc::error::TrySendError::Full(_)) => {
                Err(QueueError::QueueFull)
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
                Err(QueueError::QueueClosed)
            }
        }
    }
    
    /// 阻塞提交任务
    pub async fn submit_blocking(&self, task: SyncTask) -> Result<(), QueueError> {
        self.sender.send(task).await
            .map_err(|_| QueueError::QueueClosed)
    }
    
    /// 获取任务（阻塞）
    pub async fn next(&self) -> Option<SyncTask> {
        let mut receiver = self.receiver.lock().await;
        receiver.recv().await
    }
    
    /// 尝试获取任务（非阻塞）
    pub async fn try_next(&self) -> Option<SyncTask> {
        let mut receiver = self.receiver.lock().await;
        match receiver.try_recv() {
            Ok(task) => Some(task),
            Err(_) => None,
        }
    }
    
    /// 获取队列容量
    pub fn capacity(&self) -> usize {
        self.capacity
    }
    
    /// 关闭队列
    pub fn close(&self) {
        self.sender.closed();
    }
}

#[derive(Debug, thiserror::Error)]
pub enum QueueError {
    #[error("队列已满")]
    QueueFull,
    #[error("队列已关闭")]
    QueueClosed,
}
```

### 3. 批量提交处理器

**文件**: `src/sync/batch.rs`

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time::interval;
use crate::sync::task::SyncTask;
use crate::coordinator::FulltextCoordinator;
use crate::search::engine::SearchEngine;
use std::sync::Arc;

/// 批量提交配置
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// 批量大小
    pub batch_size: usize,
    /// 提交间隔
    pub commit_interval: Duration,
    /// 最大等待时间
    pub max_wait_time: Duration,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            batch_size: 100,
            commit_interval: Duration::from_secs(1),
            max_wait_time: Duration::from_secs(5),
        }
    }
}

/// 批量提交处理器
pub struct BatchProcessor {
    coordinator: Arc<FulltextCoordinator>,
    config: BatchConfig,
    /// 缓冲区: (space_id, tag_name, field_name) -> Vec<(doc_id, content)>
    buffers: HashMap<(u64, String, String), Vec<(String, String)>>,
    /// 最后提交时间
    last_commit: HashMap<(u64, String, String), Instant>,
}

impl BatchProcessor {
    pub fn new(coordinator: Arc<FulltextCoordinator>, config: BatchConfig) -> Self {
        Self {
            coordinator,
            config,
            buffers: HashMap::new(),
            last_commit: HashMap::new(),
        }
    }
    
    /// 添加文档到缓冲区
    pub fn add_document(
        &mut self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        doc_id: String,
        content: String,
    ) {
        let key = (space_id, tag_name.to_string(), field_name.to_string());
        
        self.buffers
            .entry(key.clone())
            .or_default()
            .push((doc_id, content));
        
        self.last_commit.entry(key).or_insert_with(Instant::now);
    }
    
    /// 检查是否需要提交
    pub fn should_commit(&self, key: &(u64, String, String)) -> bool {
        if let Some(buffer) = self.buffers.get(key) {
            // 检查批量大小
            if buffer.len() >= self.config.batch_size {
                return true;
            }
        }
        
        // 检查时间间隔
        if let Some(last) = self.last_commit.get(key) {
            if last.elapsed() >= self.config.commit_interval {
                return true;
            }
        }
        
        false
    }
    
    /// 执行批量提交
    pub async fn commit_batch(
        &mut self,
        key: (u64, String, String),
    ) -> Result<(), BatchError> {
        if let Some(documents) = self.buffers.remove(&key) {
            if documents.is_empty() {
                return Ok(());
            }
            
            let (space_id, tag_name, field_name) = key.clone();
            
            // 获取引擎并批量索引
            if let Some(engine) = self.coordinator.get_engine(space_id, &tag_name, &field_name) {
                engine.index_batch(documents).await
                    .map_err(|e| BatchError::IndexError(e.to_string()))?;
                
                // 提交变更
                engine.commit().await
                    .map_err(|e| BatchError::CommitError(e.to_string()))?;
            }
            
            // 更新最后提交时间
            self.last_commit.insert(key, Instant::now());
        }
        
        Ok(())
    }
    
    /// 提交所有缓冲区
    pub async fn commit_all(&mut self) -> Vec<((u64, String, String), Result<(), BatchError>)> {
        let keys: Vec<_> = self.buffers.keys().cloned().collect();
        let mut results = Vec::new();
        
        for key in keys {
            let result = self.commit_batch(key.clone()).await;
            results.push((key, result));
        }
        
        results
    }
    
    /// 启动后台提交任务
    pub fn start_background_commit(self: Arc<Mutex<Self>>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_millis(100));
            
            loop {
                ticker.tick().await;
                
                let mut processor = self.lock().await;
                let keys: Vec<_> = processor.buffers.keys().cloned().collect();
                
                for key in keys {
                    if processor.should_commit(&key) {
                        if let Err(e) = processor.commit_batch(key).await {
                            log::error!("批量提交失败: {:?}", e);
                        }
                    }
                }
            }
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BatchError {
    #[error("索引错误: {0}")]
    IndexError(String),
    #[error("提交错误: {0}")]
    CommitError(String),
}
```

### 4. 同步管理器

**文件**: `src/sync/manager.rs`

```rust
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;

use crate::sync::task::{SyncTask, TaskResult};
use crate::sync::queue::{SyncTaskQueue, QueueError};
use crate::sync::batch::{BatchProcessor, BatchConfig};
use crate::coordinator::{FulltextCoordinator, ChangeType};
use crate::core::Value;

/// 同步模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncMode {
    /// 同步模式（阻塞）
    Sync,
    /// 异步模式
    Async,
    /// 关闭同步
    Off,
}

/// 同步管理器
pub struct SyncManager {
    /// 协调器
    coordinator: Arc<FulltextCoordinator>,
    /// 同步模式
    mode: RwLock<SyncMode>,
    /// 任务队列
    queue: Option<Arc<SyncTaskQueue>>,
    /// 批量处理器
    batch_processor: Arc<Mutex<BatchProcessor>>,
    /// 后台任务句柄
    background_handle: Mutex<Option<JoinHandle<()>>>,
    /// 运行状态
    running: RwLock<bool>,
}

impl SyncManager {
    /// 创建新的同步管理器
    pub fn new(
        coordinator: Arc<FulltextCoordinator>,
        mode: SyncMode,
        queue_capacity: usize,
        batch_config: BatchConfig,
    ) -> Self {
        let queue = if mode == SyncMode::Async {
            Some(Arc::new(SyncTaskQueue::new(queue_capacity)))
        } else {
            None
        };
        
        let batch_processor = Arc::new(Mutex::new(BatchProcessor::new(
            coordinator.clone(),
            batch_config,
        )));
        
        Self {
            coordinator,
            mode: RwLock::new(mode),
            queue,
            batch_processor,
            background_handle: Mutex::new(None),
            running: RwLock::new(false),
        }
    }
    
    /// 启动同步管理器
    pub async fn start(&self) {
        let mut running = self.running.write().await;
        if *running {
            return;
        }
        
        *running = true;
        
        // 启动后台处理任务
        if let Some(queue) = &self.queue {
            let queue = queue.clone();
            let coordinator = self.coordinator.clone();
            let batch_processor = self.batch_processor.clone();
            
            let handle = tokio::spawn(async move {
                Self::process_loop(queue, coordinator, batch_processor).await;
            });
            
            let mut bg_handle = self.background_handle.lock().await;
            *bg_handle = Some(handle);
        }
        
        // 启动批量提交任务
        let batch_processor = self.batch_processor.clone();
        tokio::spawn(async move {
            BatchProcessor::start_background_commit(batch_processor).await;
        });
        
        log::info!("同步管理器已启动");
    }
    
    /// 停止同步管理器
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        if !*running {
            return;
        }
        
        *running = false;
        
        // 关闭队列
        if let Some(queue) = &self.queue {
            queue.close();
        }
        
        // 等待后台任务完成
        if let Some(handle) = self.background_handle.lock().await.take() {
            let _ = handle.await;
        }
        
        // 提交所有未完成的批量任务
        let mut processor = self.batch_processor.lock().await;
        let _ = processor.commit_all().await;
        
        log::info!("同步管理器已停止");
    }
    
    /// 处理顶点变更
    /// 
    /// 根据同步模式决定是同步还是异步处理
    pub async fn on_vertex_change(
        &self,
        space_id: u64,
        tag_name: &str,
        vertex_id: &Value,
        properties: &[(String, Value)],
        change_type: ChangeType,
    ) -> Result<(), SyncError> {
        let mode = *self.mode.read().await;
        
        match mode {
            SyncMode::Sync => {
                // 同步模式：直接调用协调器
                let props: std::collections::HashMap<_, _> = properties.iter()
                    .cloned()
                    .collect();
                self.coordinator
                    .on_vertex_change(space_id, tag_name, vertex_id, &props, change_type)
                    .await
                    .map_err(|e| SyncError::CoordinatorError(e.to_string()))?;
            }
            SyncMode::Async => {
                // 异步模式：提交到队列
                let task = SyncTask::vertex_change(
                    space_id,
                    tag_name,
                    vertex_id,
                    properties.to_vec(),
                    change_type,
                );
                
                if let Some(queue) = &self.queue {
                    queue.submit(task).await?;
                }
            }
            SyncMode::Off => {
                // 同步关闭，不处理
            }
        }
        
        Ok(())
    }
    
    /// 后台处理循环
    async fn process_loop(
        queue: Arc<SyncTaskQueue>,
        coordinator: Arc<FulltextCoordinator>,
        batch_processor: Arc<Mutex<BatchProcessor>>,
    ) {
        while let Some(task) = queue.next().await {
            let result = Self::execute_task(&task, &coordinator, &batch_processor).await;
            
            match result {
                TaskResult::Success => {
                    log::debug!("任务执行成功: {}", task.task_id());
                }
                TaskResult::Failed(msg) => {
                    log::error!("任务执行失败 [{}]: {}", task.task_id(), msg);
                    // TODO: 记录到失败队列，支持重试
                }
                TaskResult::Retryable(msg) => {
                    log::warn!("任务可重试 [{}]: {}", task.task_id(), msg);
                    // TODO: 重试逻辑
                }
            }
        }
    }
    
    /// 执行单个任务
    async fn execute_task(
        task: &SyncTask,
        coordinator: &FulltextCoordinator,
        batch_processor: &Arc<Mutex<BatchProcessor>>,
    ) -> TaskResult {
        match task {
            SyncTask::VertexChange { space_id, tag_name, vertex_id, properties, change_type, .. } => {
                // 将属性按字段分组，添加到批量处理器
                let mut processor = batch_processor.lock().await;
                
                for (field_name, value) in properties {
                    if let Value::String(content) = value {
                        let doc_id = vertex_id.to_string();
                        processor.add_document(
                            *space_id,
                            tag_name,
                            field_name,
                            doc_id,
                            content.clone(),
                        );
                    }
                }
                
                TaskResult::Success
            }
            SyncTask::BatchIndex { space_id, tag_name, field_name, documents, .. } => {
                if let Some(engine) = coordinator.get_engine(*space_id, tag_name, field_name) {
                    match engine.index_batch(documents.clone()).await {
                        Ok(_) => TaskResult::Success,
                        Err(e) => TaskResult::Retryable(e.to_string()),
                    }
                } else {
                    TaskResult::Failed(format!("索引不存在: {}.{}.{}"", space_id, tag_name, field_name))
                }
            }
            SyncTask::CommitIndex { space_id, tag_name, field_name, .. } => {
                if let Some(engine) = coordinator.get_engine(*space_id, tag_name, field_name) {
                    match engine.commit().await {
                        Ok(_) => TaskResult::Success,
                        Err(e) => TaskResult::Retryable(e.to_string()),
                    }
                } else {
                    TaskResult::Success // 索引可能已被删除
                }
            }
            SyncTask::RebuildIndex { space_id, tag_name, field_name, .. } => {
                match coordinator.rebuild_index(*space_id, tag_name, field_name).await {
                    Ok(_) => TaskResult::Success,
                    Err(e) => TaskResult::Failed(e.to_string()),
                }
            }
        }
    }
    
    /// 获取同步模式
    pub async fn get_mode(&self) -> SyncMode {
        *self.mode.read().await
    }
    
    /// 设置同步模式
    pub async fn set_mode(&self, mode: SyncMode) {
        let mut current = self.mode.write().await;
        *current = mode;
    }
    
    /// 强制提交所有待处理的数据
    pub async fn force_commit(&self) -> Result<(), SyncError> {
        let mut processor = self.batch_processor.lock().await;
        let results = processor.commit_all().await;
        
        for (key, result) in results {
            if let Err(e) = result {
                log::error!("提交失败 {:?}: {:?}", key, e);
                return Err(SyncError::CommitError(e.to_string()));
            }
        }
        
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("队列错误: {0}")]
    QueueError(#[from] QueueError),
    #[error("协调器错误: {0}")]
    CoordinatorError(String),
    #[error("提交错误: {0}")]
    CommitError(String),
    #[error("内部错误: {0}")]
    Internal(String),
}
```

### 5. 模块入口文件

**文件**: `src/sync/mod.rs`

```rust
//! 数据同步模块
//! 
//! 本模块负责图数据变更与全文索引的异步同步。

pub mod manager;
pub mod task;
pub mod queue;
pub mod batch;
pub mod persistence;
pub mod recovery;

pub use manager::{SyncManager, SyncMode, SyncError};
pub use task::{SyncTask, TaskResult};
pub use queue::{SyncTaskQueue, QueueError};
pub use batch::{BatchProcessor, BatchConfig, BatchError};
```

---

## 数据流设计

### 同步模式对比

| 模式 | 行为 | 适用场景 |
|------|------|----------|
| **Sync** | 阻塞等待索引完成 | 强一致性要求 |
| **Async** | 提交到队列立即返回 | 默认推荐 |
| **Off** | 不更新全文索引 | 维护模式 |

### 异步同步流程

```
存储层事务提交成功
    │
    ▼
SyncManager::on_vertex_change()
    │ 检查同步模式
    ▼
Async 模式
    │ 创建 SyncTask
    ▼
SyncTaskQueue::submit()
    │ 非阻塞提交
    ▼
立即返回（不阻塞主流程）
    │
    ▼
后台处理循环
    │ 从队列获取任务
    ▼
BatchProcessor::add_document()
    │ 添加到缓冲区
    ▼
定时或满批量触发
    │
    ▼
SearchEngine::index_batch()
    │ 批量索引
    ▼
SearchEngine::commit()
    │ 提交到磁盘
    ▼
完成
```

---

## 测试方案

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};
    
    #[tokio::test]
    async fn test_async_sync() {
        let (coordinator, _temp) = setup_test_coordinator().await;
        let manager = SyncManager::new(
            Arc::new(coordinator),
            SyncMode::Async,
            1000,
            BatchConfig::default(),
        );
        
        manager.start().await;
        
        // 提交变更
        let vertex_id = Value::from(1i64);
        let properties = vec![("content".to_string(), Value::from("Hello world"))];
        
        manager.on_vertex_change(1, "Post", &vertex_id, &properties, ChangeType::Insert)
            .await
            .unwrap();
        
        // 等待后台处理
        sleep(Duration::from_millis(200)).await;
        
        // 强制提交
        manager.force_commit().await.unwrap();
        
        // 验证搜索结果
        let results = coordinator.search(1, "Post", "content", "Hello", 10).await.unwrap();
        assert_eq!(results.len(), 1);
        
        manager.stop().await;
    }
    
    #[tokio::test]
    async fn test_batch_processing() {
        let (coordinator, _temp) = setup_test_coordinator().await;
        let mut processor = BatchProcessor::new(
            Arc::new(coordinator),
            BatchConfig {
                batch_size: 5,
                commit_interval: Duration::from_secs(10), // 长间隔，测试批量触发
                max_wait_time: Duration::from_secs(30),
            },
        );
        
        // 添加 5 个文档（达到批量大小）
        for i in 0..5 {
            processor.add_document(
                1,
                "Post",
                "content",
                i.to_string(),
                format!("Content {}", i),
            );
        }
        
        // 应该触发提交
        assert!(processor.should_commit(&(1, "Post".to_string(), "content".to_string())));
    }
}
```

---

## 验收标准

- [ ] `SyncManager` 支持 Sync/Async/Off 三种模式
- [ ] 异步模式下不阻塞主事务
- [ ] 批量提交功能正常工作
- [ ] 后台处理循环稳定运行
- [ ] 支持强制提交所有待处理数据
- [ ] 所有单元测试通过

---

## 风险与缓解措施

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 队列溢出 | 高 | 监控队列大小，超限告警 |
| 数据丢失 | 高 | 后续添加持久化队列（Phase 5） |
| 提交延迟 | 中 | 可配置的批量大小和间隔 |
| 重复索引 | 中 | 使用文档ID去重 |

---

## 下一阶段依赖

本阶段完成后，系统具备：

- 完整的异步同步机制
- 可配置的同步策略
- 批量提交优化
- 后台任务管理
