# Inversearch Worker 架构设计文档

## 1. 设计目标

本设计旨在将 JavaScript Worker 架构移植到 Rust 中，提供高性能、类型安全的异步任务处理系统。

### 1.1 功能对比

| 功能 | JS Worker | Rust Worker | 实现状态 |
|------|-----------|-------------|----------|
| 消息路由 | ✅ handler.js | ✅ WorkerManager | 待实现 |
| 任务分发 | ✅ 原生消息 | ✅ TaskQueue | 待实现 |
| 索引操作 | ✅ Index.apply | ✅ IndexExecutor | 已存在 |
| 结果返回 | ✅ postMessage | ✅ ResultChannel | 待实现 |
| 异步处理 | ✅ Promise | ✅ tokio::spawn | 已存在 |
| 线程池 | ✅ Worker | ✅ WorkerPool | 待实现 |

## 2. 架构设计

### 2.1 整体架构

```
┌─────────────────────────────────────────────────────────────┐
│                    WorkerManager                           │
│  ┌───────────────────────────────────────────────────────┐   │
│  │                  TaskQueue                           │   │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐   │   │
│  │  │ TaskChannel │ │ TaskChannel │ │ TaskChannel │   │   │
│  │  │ (init)      │ │ (search)    │ │ (export)    │   │   │
│  │  └─────┬───────┘ └─────┬───────┘ └─────┬───────┘   │   │
│  │        │               │               │           │   │
│  │  ┌─────────────────────────────────────────────────┐   │   │
│  │  │              TaskDistributor                   │   │   │
│  │  └────────────────┬──────────────────────────────┘   │   │
│  │                   │                                  │   │
│  │  ┌─────────────────────────────────────────────────┐   │   │
│  │  │              WorkerPool                        │   │   │
│  │  │  ┌─────────┐ ┌─────────┐ ┌─────────┐         │   │   │
│  │  │  │Worker 1 │ │Worker 2 │ │Worker 3 │         │   │   │
│  │  │  │(tokio)  │ │(tokio)  │ │(tokio)  │         │   │   │
│  │  │  └─────────┘ └─────────┘ └─────────┘         │   │   │
│  │  └─────────────────────────────────────────────────┘   │   │
│  └───────────────────────────────────────────────────────┘   │
│                           │                                  │
│  ┌───────────────────────────────────────────────────────┐   │
│  │                  ResultChannel                         │   │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐     │   │
│  │  │ ResultRx    │ │ ResultRx    │ │ ResultRx    │     │   │
│  │  │ (tokio::mpsc)│ │ (tokio::mpsc)│ │ (tokio::mpsc)│     │   │
│  │  └─────┬───────┘ └─────┬───────┘ └─────┬───────┘     │   │
│  │        │               │               │             │   │
│  │  ┌─────────────────────────────────────────────────┐   │   │
│  │  │              ResultCollector                     │   │   │
│  │  └────────────────┬──────────────────────────────┘   │   │
│  │                   │                                  │   │
│  │  ┌─────────────────────────────────────────────────┐   │   │
│  │  │              ResponseSender                      │   │   │
│  │  │  (返回给客户端)                                 │   │   │
│  │  └─────────────────────────────────────────────────┘   │   │
│  └───────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 模块设计

#### 2.2.1 WorkerManager 模块

```rust
// src/worker/manager.rs
use tokio::sync::{mpsc, oneshot};
use std::sync::Arc;
use std::collections::HashMap;
use uuid::Uuid;

pub struct WorkerManager {
    task_queue: Arc<TaskQueue>,
    worker_pool: Arc<WorkerPool>,
    result_collector: Arc<ResultCollector>,
    config: WorkerConfig,
}

impl WorkerManager {
    pub fn new(config: WorkerConfig) -> Self {
        let task_queue = Arc::new(TaskQueue::new());
        let result_collector = Arc::new(ResultCollector::new());
        let worker_pool = Arc::new(WorkerPool::new(
            config.worker_count,
            task_queue.clone(),
            result_collector.clone(),
        ));
        
        Self {
            task_queue,
            worker_pool,
            result_collector,
            config,
        }
    }
    
    pub async fn submit_task(&self, task: WorkerTask) -> Result<TaskResult, WorkerError> {
        let task_id = Uuid::new_v4();
        let (tx, rx) = oneshot::channel();
        
        // 提交任务到队列
        self.task_queue.enqueue(task_id, task, tx).await?;
        
        // 等待结果
        match rx.await {
            Ok(result) => Ok(result),
            Err(_) => Err(WorkerError::TaskCancelled),
        }
    }
}
```

#### 2.2.2 TaskQueue 模块

```rust
// src/worker/task_queue.rs
use tokio::sync::{mpsc, RwLock};
use std::collections::HashMap;
use uuid::Uuid;

pub enum TaskType {
    Init { options: IndexOptions },
    Search { query: String, limit: usize, offset: usize },
    Export { field: Option<String> },
    Import { data: Vec<u8> },
}

pub struct WorkerTask {
    pub task_type: TaskType,
    pub priority: TaskPriority,
    pub timeout: Duration,
}

pub struct TaskQueue {
    queues: RwLock<HashMap<TaskType, mpsc::UnboundedSender<(Uuid, WorkerTask, oneshot::Sender<TaskResult>)>>>,
    distributor: TaskDistributor,
}

impl TaskQueue {
    pub fn new() -> Self {
        let mut queues = HashMap::new();
        
        // 为每种任务类型创建独立队列
        for task_type in [TaskType::Init, TaskType::Search, TaskType::Export, TaskType::Import] {
            let (tx, rx) = mpsc::unbounded_channel();
            queues.insert(task_type, tx);
        }
        
        Self {
            queues: RwLock::new(queues),
            distributor: TaskDistributor::new(),
        }
    }
    
    pub async fn enqueue(
        &self,
        task_id: Uuid,
        task: WorkerTask,
        result_tx: oneshot::Sender<TaskResult>,
    ) -> Result<(), WorkerError> {
        let queues = self.queues.read().await;
        if let Some(tx) = queues.get(&task.task_type) {
            tx.send((task_id, task, result_tx))
                .map_err(|_| WorkerError::QueueClosed)?;
            Ok(())
        } else {
            Err(WorkerError::InvalidTaskType)
        }
    }
}
```

#### 2.2.3 WorkerPool 模块

```rust
// src/worker/pool.rs
use tokio::task::JoinHandle;
use std::sync::Arc;

pub struct WorkerPool {
    workers: Vec<WorkerHandle>,
    task_queue: Arc<TaskQueue>,
    result_collector: Arc<ResultCollector>,
}

impl WorkerPool {
    pub fn new(
        worker_count: usize,
        task_queue: Arc<TaskQueue>,
        result_collector: Arc<ResultCollector>,
    ) -> Self {
        let mut workers = Vec::with_capacity(worker_count);
        
        for worker_id in 0..worker_count {
            let worker = Worker::new(
                worker_id,
                task_queue.clone(),
                result_collector.clone(),
            );
            let handle = tokio::spawn(worker.run());
            workers.push(WorkerHandle { handle });
        }
        
        Self {
            workers,
            task_queue,
            result_collector,
        }
    }
}

struct Worker {
    id: usize,
    task_queue: Arc<TaskQueue>,
    result_collector: Arc<ResultCollector>,
    index: Option<Index>,
}

impl Worker {
    pub async fn run(mut self) {
        loop {
            // 从队列获取任务
            if let Ok((task_id, task, result_tx)) = self.task_queue.dequeue().await {
                // 执行任务
                let result = self.execute_task(task).await;
                
                // 发送结果
                let _ = self.result_collector.collect(task_id, result, result_tx).await;
            }
        }
    }
    
    async fn execute_task(&mut self, task: WorkerTask) -> TaskResult {
        match task.task_type {
            TaskType::Init { options } => {
                self.index = Some(Index::new(options));
                TaskResult::InitSuccess
            }
            TaskType::Search { query, limit, offset } => {
                if let Some(ref index) = self.index {
                    let search_options = SearchOptions {
                        query: Some(query),
                        limit: Some(limit),
                        offset: Some(offset),
                        ..Default::default()
                    };
                    
                    match index.search(&search_options) {
                        Ok(result) => TaskResult::SearchSuccess(result.results),
                        Err(e) => TaskResult::SearchError(e.to_string()),
                    }
                } else {
                    TaskResult::SearchError("Index not initialized".to_string())
                }
            }
            TaskType::Export { field } => {
                if let Some(ref index) = self.index {
                    match index.export(field.as_deref()) {
                        Ok(data) => TaskResult::ExportSuccess(data),
                        Err(e) => TaskResult::ExportError(e.to_string()),
                    }
                } else {
                    TaskResult::ExportError("Index not initialized".to_string())
                }
            }
            TaskType::Import { data } => {
                if let Some(ref mut index) = self.index {
                    match index.import(&data) {
                        Ok(_) => TaskResult::ImportSuccess,
                        Err(e) => TaskResult::ImportError(e.to_string()),
                    }
                } else {
                    TaskResult::ImportError("Index not initialized".to_string())
                }
            }
        }
    }
}
```

#### 2.2.4 ResultCollector 模块

```rust
// src/worker/result.rs
use tokio::sync::mpsc;
use std::collections::HashMap;
use uuid::Uuid;

pub struct ResultCollector {
    results: RwLock<HashMap<Uuid, TaskResult>>,
    metrics: Arc<Metrics>,
}

impl ResultCollector {
    pub fn new() -> Self {
        Self {
            results: RwLock::new(HashMap::new()),
            metrics: Arc::new(Metrics::new()),
        }
    }
    
    pub async fn collect(
        &self,
        task_id: Uuid,
        result: TaskResult,
        result_tx: oneshot::Sender<TaskResult>,
    ) -> Result<(), WorkerError> {
        // 记录指标
        self.metrics.record_task_completion(&result).await;
        
        // 存储结果
        {
            let mut results = self.results.write().await;
            results.insert(task_id, result.clone());
        }
        
        // 发送给等待的调用者
        result_tx.send(result)
            .map_err(|_| WorkerError::ResultChannelClosed)?;
        
        Ok(())
    }
    
    pub async fn get_result(&self, task_id: Uuid) -> Option<TaskResult> {
        let results = self.results.read().await;
        results.get(&task_id).cloned()
    }
}
```

## 3. 数据结构设计

### 3.1 任务类型枚举

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    /// 初始化索引
    Init {
        options: IndexOptions,
    },
    /// 搜索任务
    Search {
        query: String,
        limit: usize,
        offset: usize,
        enrich: bool,
        resolve: bool,
        context: bool,
    },
    /// 导出任务
    Export {
        field: Option<String>,
    },
    /// 导入任务
    Import {
        data: Vec<u8>,
        format: ImportFormat,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImportFormat {
    Json,
    Binary,
    Compressed,
}
```

### 3.2 任务结果枚举

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskResult {
    /// 初始化成功
    InitSuccess,
    /// 搜索成功
    SearchSuccess(Vec<u64>),
    /// 导出成功
    ExportSuccess(Vec<u8>),
    /// 导入成功
    ImportSuccess { count: usize },
    /// 搜索错误
    SearchError(String),
    /// 导出错误
    ExportError(String),
    /// 导入错误
    ImportError(String),
    /// 超时错误
    TimeoutError,
    /// 任务取消
    TaskCancelled,
}
```

### 3.3 配置结构

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerConfig {
    /// Worker 数量
    pub worker_count: usize,
    /// 任务队列大小
    pub queue_size: usize,
    /// 任务超时时间
    pub task_timeout: Duration,
    /// 结果缓存大小
    pub result_cache_size: usize,
    /// 是否启用指标收集
    pub enable_metrics: bool,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            worker_count: 4,
            queue_size: 1000,
            task_timeout: Duration::from_secs(30),
            result_cache_size: 10000,
            enable_metrics: true,
        }
    }
}
```

## 4. 错误处理设计

```rust
#[derive(Debug, thiserror::Error)]
pub enum WorkerError {
    #[error("Queue closed")]
    QueueClosed,
    #[error("Invalid task type")]
    InvalidTaskType,
    #[error("Task cancelled")]
    TaskCancelled,
    #[error("Result channel closed")]
    ResultChannelClosed,
    #[error("Worker pool error: {0}")]
    PoolError(String),
    #[error("Task timeout")]
    TaskTimeout,
    #[error("Index error: {0}")]
    IndexError(#[from] IndexError),
}
```

## 5. 性能优化策略

### 5.1 任务优先级队列

```rust
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskPriority {
    Critical = 0,
    High = 1,
    Normal = 2,
    Low = 3,
}

impl PartialOrd for TaskPriority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TaskPriority {
    fn cmp(&self, other: &Self) -> Ordering {
        // 优先级数值越小，优先级越高
        (*self as u8).cmp(&(*other as u8))
    }
}
```

### 5.2 批量处理优化

```rust
pub struct BatchProcessor {
    batch_size: usize,
    timeout: Duration,
}

impl BatchProcessor {
    pub async fn process_batch(&self, tasks: Vec<WorkerTask>) -> Vec<TaskResult> {
        let mut results = Vec::with_capacity(tasks.len());
        
        // 按任务类型分组
        let mut grouped: HashMap<TaskType, Vec<WorkerTask>> = HashMap::new();
        for task in tasks {
            grouped.entry(task.task_type.clone()).or_default().push(task);
        }
        
        // 并行处理各组任务
        let mut handles = Vec::new();
        for (task_type, task_group) in grouped {
            let handle = tokio::spawn(self.process_group(task_type, task_group));
            handles.push(handle);
        }
        
        // 收集结果
        for handle in handles {
            if let Ok(group_results) = handle.await {
                results.extend(group_results);
            }
        }
        
        results
    }
}
```

### 5.3 连接池优化

```rust
pub struct IndexPool {
    pool: deadpool::Pool<Index>,
}

impl IndexPool {
    pub fn new(config: &WorkerConfig) -> Self {
        let manager = IndexManager::new(config);
        let pool = deadpool::Pool::builder(manager)
            .max_size(config.worker_count)
            .build()
            .expect("Failed to create index pool");
        
        Self { pool }
    }
    
    pub async fn get(&self) -> Result<Index, WorkerError> {
        self.pool.get().await.map_err(|e| WorkerError::PoolError(e.to_string()))
    }
}
```

## 6. 监控和指标

```rust
pub struct Metrics {
    tasks_total: Counter,
    tasks_success: Counter,
    tasks_failed: Counter,
    task_duration: Histogram,
    queue_size: Gauge,
    worker_utilization: Gauge,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            tasks_total: Counter::new("worker_tasks_total", "Total number of tasks"),
            tasks_success: Counter::new("worker_tasks_success", "Successful tasks"),
            tasks_failed: Counter::new("worker_tasks_failed", "Failed tasks"),
            task_duration: Histogram::new("worker_task_duration_seconds", "Task execution time"),
            queue_size: Gauge::new("worker_queue_size", "Current queue size"),
            worker_utilization: Gauge::new("worker_utilization", "Worker utilization percentage"),
        }
    }
    
    pub async fn record_task_completion(&self, result: &TaskResult) {
        self.tasks_total.increment();
        
        match result {
            TaskResult::SearchSuccess(_) | TaskResult::ExportSuccess(_) | TaskResult::ImportSuccess { .. } => {
                self.tasks_success.increment();
            }
            TaskResult::SearchError(_) | TaskResult::ExportError(_) | TaskResult::ImportError(_) => {
                self.tasks_failed.increment();
            }
            _ => {}
        }
    }
}
```

## 7. 使用示例

### 7.1 基本使用

```rust
use inversearch::worker::{WorkerManager, WorkerConfig, TaskType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建 WorkerManager
    let config = WorkerConfig::default();
    let manager = WorkerManager::new(config);
    
    // 初始化索引
    let init_task = TaskType::Init {
        options: IndexOptions::default(),
    };
    manager.submit_task(init_task).await?;
    
    // 执行搜索
    let search_task = TaskType::Search {
        query: "rust programming".to_string(),
        limit: 10,
        offset: 0,
    };
    
    match manager.submit_task(search_task).await? {
        TaskResult::SearchSuccess(results) => {
            println!("Found {} results", results.len());
        }
        TaskResult::SearchError(e) => {
            eprintln!("Search failed: {}", e);
        }
        _ => {}
    }
    
    Ok(())
}
```

### 7.2 批量处理

```rust
let tasks = vec![
    TaskType::Search { query: "rust".to_string(), limit: 5, offset: 0 },
    TaskType::Search { query: "programming".to_string(), limit: 5, offset: 0 },
    TaskType::Search { query: "tutorial".to_string(), limit: 5, offset: 0 },
];

let mut handles = Vec::new();
for task in tasks {
    let manager = manager.clone();
    let handle = tokio::spawn(async move {
        manager.submit_task(task).await
    });
    handles.push(handle);
}

for handle in handles {
    if let Ok(result) = handle.await {
        println!("Task result: {:?}", result);
    }
}
```

## 8. 性能基准

### 8.1 预期性能指标

| 指标 | JS Worker | Rust Worker | 提升倍数 |
|------|-----------|-------------|----------|
| 任务延迟 | ~50ms | ~5ms | 10x |
| 并发任务数 | 100 | 1000+ | 10x |
| 内存使用 | 高 | 低 | 3x |
| CPU 利用率 | 中等 | 高 | 2x |

### 8.2 基准测试计划

```rust
#[cfg(test)]
mod benchmarks {
    use super::*;
    use criterion::{black_box, criterion_group, criterion_main, Criterion};
    
    fn benchmark_task_submission(c: &mut Criterion) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let manager = WorkerManager::new(WorkerConfig::default());
        
        c.bench_function("task_submission", |b| {
            b.iter(|| {
                runtime.block_on(async {
                    let task = TaskType::Search {
                        query: "benchmark".to_string(),
                        limit: 10,
                        offset: 0,
                    };
                    manager.submit_task(black_box(task)).await.unwrap()
                })
            });
        });
    }
    
    criterion_group!(benches, benchmark_task_submission);
    criterion_main!(benches);
}
```

## 9. 部署和运维

### 9.1 配置示例

```toml
# config.toml
[worker]
worker_count = 8
queue_size = 2000
task_timeout = 30
result_cache_size = 50000
enable_metrics = true

[worker.metrics]
endpoint = "http://localhost:9090"
interval = 15
labels = { service = "inversearch", component = "worker" }
```

### 9.2 监控面板

```json
{
  "dashboard": {
    "title": "Inversearch Worker Metrics",
    "panels": [
      {
        "title": "Task Throughput",
        "targets": [
          {
            "expr": "rate(worker_tasks_total[5m])",
            "legendFormat": "Tasks/sec"
          }
        ]
      },
      {
        "title": "Task Success Rate",
        "targets": [
          {
            "expr": "worker_tasks_success / worker_tasks_total",
            "legendFormat": "Success Rate"
          }
        ]
      }
    ]
  }
}
```

## 10. 后续优化方向

1. **分布式 Worker**：支持跨机器 Worker 集群
2. **任务持久化**：支持任务队列持久化到 Redis
3. **动态扩缩容**：根据负载自动调整 Worker 数量
4. **智能调度**：基于 Worker 负载和历史性能进行任务调度
5. **流式处理**：支持大文件和流式数据处理

---

*文档版本：v1.0*  
*最后更新：2024年*  
*维护团队：Inversearch 开发团队*