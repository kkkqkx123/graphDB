# Scheduler 模块问题清单与修改方案

## 问题清单

| 序号 | 问题描述 | 严重程度 | 问题类型 | 状态 |
|------|----------|----------|----------|------|
| 8.1 | 调度器与执行器耦合紧密 | 中 | 架构问题 | 待修复 |
| 8.2 | 缺乏查询队列管理 | 中 | 功能缺失 | 待修复 |
| 8.3 | 不支持查询优先级 | 低 | 功能缺失 | 待修复 |
| 8.4 | 资源监控功能不足 | 低 | 功能缺失 | 待修复 |
| 8.5 | 缺乏查询取消机制 | 低 | 功能缺失 | 待修复 |

---

## 详细问题分析

### 问题 8.1: 调度器与执行器耦合紧密

**涉及文件**: `src/query/scheduler/mod.rs`

**当前实现**:
```rust
impl<S: StorageEngine + 'static> Scheduler<S> {
    pub fn new(storage: Arc<Mutex<S>>) -> Self {
        let mut scheduler = Scheduler {
            storage,
            execution_config: ExecutionConfig::default(),
        };
        scheduler.execution_config.max_concurrent_queries = 100;
        scheduler
    }

    pub async fn schedule(&self, plan: ExecutionPlan) -> Result<ExecutionResult, QueryError> {
        let storage = self.storage.clone();
        let execution_config = self.execution_config;

        // 直接创建执行器并执行
        let mut executor = GraphQueryExecutor::new(storage, execution_config);
        let execution_result = executor.execute_plan(plan).await;

        execution_result
    }
}
```

**问题**:
- 调度器直接创建执行器
- 无法自定义调度策略
- 无法插入中间件或拦截器
- 无法实现分布式调度

---

### 问题 8.2: 缺乏查询队列管理

**当前实现**: 无查询队列

**问题**:
- 无法控制并发查询数
- 无法处理查询积压
- 无法实现公平调度

---

### 问题 8.3: 不支持查询优先级

**当前实现**: 无优先级概念

**问题**:
- 无法区分重要查询和普通查询
- 无法保证 SLA
- 无法实现资源预留

---

## 修改方案

### 修改方案 8.1-8.3: 解耦调度器与执行器

**预估工作量**: 3-4 人天

**修改目标**:
- 引入任务队列
- 支持优先级
- 解耦调度和执行

**修改步骤**:

**步骤 1**: 定义查询任务

```rust
// src/query/scheduler/task.rs

use std::sync::Arc;
use tokio::sync::oneshot;

/// 查询任务
#[derive(Debug)]
pub struct QueryTask {
    /// 任务 ID
    pub id: String,
    /// 查询文本
    pub query_text: String,
    /// 执行计划
    pub plan: Option<ExecutionPlan>,
    /// 优先级
    pub priority: QueryPriority,
    /// 创建时间
    pub created_at: std::time::Instant,
    /// 截止时间
    pub deadline: Option<std::time::Instant>,
    /// 用户信息
    pub user: Option<String>,
    /// 结果接收端
    pub result_tx: Option<oneshot::Sender<SchedulerResult>>,
    /// 元数据
    pub metadata: TaskMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum QueryPriority {
    Critical = 0,
    High = 1,
    Normal = 2,
    Low = 3,
    Background = 4,
}

#[derive(Debug, Clone, Default)]
pub struct TaskMetadata {
    pub client_ip: Option<String>,
    pub database: Option<String>,
    pub resource_hints: ResourceHints,
}

#[derive(Debug, Clone, Default)]
pub struct ResourceHints {
    pub estimated_memory: usize,
    pub estimated_duration: std::time::Duration,
    pub required_tags: Vec<String>,
}

impl QueryTask {
    pub fn new(id: String, query_text: String, priority: QueryPriority) -> Self {
        Self {
            id,
            query_text,
            plan: None,
            priority,
            created_at: std::time::Instant::now(),
            deadline: None,
            user: None,
            result_tx: None,
            metadata: TaskMetadata::default(),
        }
    }
    
    pub fn with_plan(mut self, plan: ExecutionPlan) -> Self {
        self.plan = Some(plan);
        self
    }
    
    pub fn with_deadline(mut self, deadline: std::time::Instant) -> Self {
        self.deadline = Some(deadline);
        self
    }
    
    pub fn with_user(mut self, user: String) -> Self {
        self.user = Some(user);
        self
    }
    
    pub fn is_expired(&self) -> bool {
        if let Some(deadline) = self.deadline {
            std::time::Instant::now() > deadline
        } else {
            false
        }
    }
}

/// 调度结果
pub type SchedulerResult = Result<ExecutionResult, QueryError>;
```

**步骤 2**: 实现查询队列

```rust
// src/query/scheduler/queue.rs

use std::collections::{BinaryHeap, HashMap};
use std::sync::{Arc, Mutex};
use std::cmp::Reverse;

/// 查询队列
pub struct QueryQueue {
    /// 优先级队列
    queues: HashMap<QueryPriority, Vec<QueryTask>>,
    /// 等待队列中的任务数
    waiting_count: usize,
}

impl QueryQueue {
    pub fn new() -> Self {
        let mut queues = HashMap::new();
        queues.insert(QueryPriority::Critical, Vec::new());
        queues.insert(QueryPriority::High, Vec::new());
        queues.insert(QueryPriority::Normal, Vec::new());
        queues.insert(QueryPriority::Low, Vec::new());
        queues.insert(QueryPriority::Background, Vec::new());
        
        Self {
            queues,
            waiting_count: 0,
        }
    }
    
    pub fn push(&mut self, task: QueryTask) {
        self.waiting_count += 1;
        if let Some(queue) = self.queues.get_mut(&task.priority) {
            queue.push(task);
        }
    }
    
    pub fn pop(&mut self) -> Option<QueryTask> {
        // 按优先级从高到低检查
        for priority in [
            QueryPriority::Critical,
            QueryPriority::High,
            QueryPriority::Normal,
            QueryPriority::Low,
            QueryPriority::Background,
        ] {
            if let Some(queue) = self.queues.get_mut(&priority) {
                if let Some(task) = queue.pop() {
                    self.waiting_count -= 1;
                    return Some(task);
                }
            }
        }
        None
    }
    
    pub fn len(&self) -> usize {
        self.waiting_count
    }
    
    pub fn is_empty(&self) -> bool {
        self.waiting_count == 0
    }
    
    pub fn peek(&self) -> Option<&QueryTask> {
        for priority in [
            QueryPriority::Critical,
            QueryPriority::High,
            QueryPriority::Normal,
            QueryPriority::Low,
            QueryPriority::Background,
        ] {
            if let Some(queue) = self.queues.get(&priority) {
                if let Some(task) = queue.first() {
                    return Some(task);
                }
            }
        }
        None
    }
}

/// 线程安全的队列包装
#[derive(Clone)]
pub struct SharedQueryQueue {
    inner: Arc<Mutex<QueryQueue>>,
}

impl SharedQueryQueue {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(QueryQueue::new())),
        }
    }
    
    pub fn push(&self, task: QueryTask) {
        let mut queue = self.inner.lock().unwrap();
        queue.push(task);
    }
    
    pub fn try_pop(&self) -> Option<QueryTask> {
        let mut queue = self.inner.lock().unwrap();
        queue.pop()
    }
    
    pub fn len(&self) -> usize {
        let queue = self.inner.lock().unwrap();
        queue.len()
    }
    
    pub fn is_empty(&self) -> bool {
        let queue = self.inner.lock().unwrap();
        queue.is_empty()
    }
}
```

**步骤 3**: 实现调度器

```rust
// src/query/scheduler/mod.rs

use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, Semaphore};
use tokio::task::JoinHandle;

/// 调度器配置
#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    /// 最大并发查询数
    pub max_concurrent_queries: usize,
    /// 默认超时时间
    pub default_timeout: std::time::Duration,
    /// 队列容量
    pub queue_capacity: usize,
    /// 是否启用优先级调度
    pub enable_priority_scheduling: bool,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            max_concurrent_queries: 100,
            default_timeout: std::time::Duration::from_secs(30),
            queue_capacity: 10000,
            enable_priority_scheduling: true,
        }
    }
}

/// 调度器
pub struct Scheduler<S: StorageEngine + 'static> {
    /// 配置
    config: SchedulerConfig,
    /// 查询队列
    queue: SharedQueryQueue,
    /// 信号量（控制并发）
    semaphore: Arc<Semaphore>,
    /// 存储引擎
    storage: Arc<Mutex<S>>,
    /// 运行状态
    state: SchedulerState,
    /// 工作线程句柄
    worker_handles: Vec<JoinHandle<()>>,
}

#[derive(Debug, Clone, Default)]
pub struct SchedulerState {
    pub is_running: bool,
    pub total_queued: usize,
    pub total_completed: usize,
    pub total_failed: usize,
    pub current_running: usize,
}

impl<S: StorageEngine + 'static> Scheduler<S> {
    pub fn new(storage: Arc<Mutex<S>>) -> Self {
        Self::with_config(storage, SchedulerConfig::default())
    }
    
    pub fn with_config(storage: Arc<Mutex<S>>, config: SchedulerConfig) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_queries));
        
        Self {
            config,
            queue: SharedQueryQueue::new(),
            semaphore,
            storage,
            state: SchedulerState::default(),
            worker_handles: Vec::new(),
        }
    }
    
    /// 启动调度器
    pub fn start(&mut self) {
        self.state.is_running = true;
        
        let worker_count = self.config.max_concurrent_queries;
        
        for i in 0..worker_count {
            let handle = self.spawn_worker(i);
            self.worker_handles.push(handle);
        }
    }
    
    fn spawn_worker(&self, worker_id: usize) -> JoinHandle<()> {
        let queue = self.queue.clone();
        let storage = self.storage.clone();
        let semaphore = self.semaphore.clone();
        let config = self.config.clone();
        
        tokio::spawn(async move {
            tracing::info!(worker_id, "Worker started");
            
            loop {
                // 从队列获取任务
                let task = queue.try_pop();
                if task.is_none() {
                    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                    continue;
                }
                
                let mut task = task.unwrap();
                
                // 获取执行许可
                let permit = semaphore.acquire().await.unwrap();
                
                // 执行任务
                let result = Self::execute_task(&storage, &config, &mut task).await;
                
                // 发送结果
                if let Some(tx) = task.result_tx.take() {
                    let _ = tx.send(result);
                }
                
                // 释放许可
                drop(permit);
            }
        })
    }
    
    async fn execute_task(
        storage: &Arc<Mutex<S>>,
        config: &SchedulerConfig,
        task: &mut QueryTask,
    ) -> SchedulerResult {
        // 检查是否过期
        if task.is_expired() {
            return Err(QueryError::Timeout("Query deadline exceeded".to_string()));
        }
        
        // 创建执行器
        let execution_config = ExecutionConfig {
            timeout: task.deadline
                .map(|d| d - std::time::Instant::now())
                .unwrap_or(config.default_timeout),
            ..Default::default()
        };
        
        let mut executor = GraphQueryExecutor::new(storage.clone(), execution_config);
        
        // 执行计划或解析查询
        if let Some(plan) = task.plan.take() {
            executor.execute_plan(plan).await
        } else {
            // 需要解析和规划
            let mut pipeline = QueryPipelineManager::new(storage.clone());
            let (result, _) = pipeline.execute_query_with_metrics(&task.query_text).await?;
            Ok(result)
        }
    }
    
    /// 提交查询任务
    pub async fn submit(&self, task: QueryTask) -> Result<QueryHandle, SchedulerError> {
        if self.state.current_running >= self.config.max_concurrent_queries {
            return Err(SchedulerError::QueueFull);
        }
        
        let (tx, rx) = oneshot::channel();
        let mut task = task;
        task.result_tx = Some(tx);
        
        self.queue.push(task);
        
        Ok(QueryHandle::new(rx))
    }
    
    /// 取消查询
    pub async fn cancel(&self, task_id: &str) -> Result<(), SchedulerError> {
        // 从队列中移除任务
        // 实现需要遍历队列
        unimplemented!("Cancel not implemented")
    }
    
    /// 获取队列状态
    pub fn get_queue_status(&self) -> QueueStatus {
        QueueStatus {
            queue_length: self.queue.len(),
            running_count: self.state.current_running,
            max_concurrent: self.config.max_concurrent_queries,
        }
    }
}

/// 查询句柄
pub struct QueryHandle {
    result_rx: oneshot::Receiver<SchedulerResult>,
}

impl QueryHandle {
    fn new(result_rx: oneshot::Receiver<SchedulerResult>) -> Self {
        Self { result_rx }
    }
    
    pub async fn get_result(self) -> SchedulerResult {
        self.result_rx.await.unwrap()
    }
}

/// 队列状态
#[derive(Debug, Clone)]
pub struct QueueStatus {
    pub queue_length: usize,
    pub running_count: usize,
    pub max_concurrent: usize,
}

/// 调度器错误
#[derive(Debug, thiserror::Error)]
pub enum SchedulerError {
    #[error("Queue is full")]
    QueueFull,
    
    #[error("Task not found: {0}")]
    TaskNotFound(String),
    
    #[error("Scheduler is not running")]
    NotRunning,
    
    #[error("Timeout: {0}")]
    Timeout(String),
}
```

---

### 修改方案 8.4-8.5: 资源监控和查询取消

**预估工作量**: 2 人天

**修改代码**:

```rust
// src/query/scheduler/monitor.rs

use std::sync::{Arc, Mutex};
use std::time::Duration;

/// 资源监控器
pub struct ResourceMonitor {
    /// 系统资源
    system_metrics: Arc<Mutex<SystemMetrics>>,
    /// 查询指标
    query_metrics: Arc<Mutex<QueryMetrics>>,
    /// 告警阈值
    alert_thresholds: AlertThresholds,
}

#[derive(Debug, Clone, Default)]
pub struct SystemMetrics {
    pub cpu_usage: f64,
    pub memory_usage: usize,
    pub memory_total: usize,
    pub disk_usage: usize,
    pub network_io: (usize, usize),
}

#[derive(Debug, Clone, Default)]
pub struct QueryMetrics {
    pub total_queries: u64,
    pub running_queries: u64,
    pub queued_queries: u64,
    pub completed_queries: u64,
    pub failed_queries: u64,
    pub average_latency: Duration,
    pub p99_latency: Duration,
}

#[derive(Debug, Clone)]
pub struct AlertThresholds {
    pub cpu_threshold: f64,
    pub memory_threshold: f64,
    pub queue_threshold: usize,
    pub latency_threshold: Duration,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            cpu_threshold: 0.9,
            memory_threshold: 0.9,
            queue_threshold: 1000,
            latency_threshold: Duration::from_secs(10),
        }
    }
}

impl ResourceMonitor {
    pub fn new() -> Self {
        Self {
            system_metrics: Arc::new(Mutex::new(SystemMetrics::default())),
            query_metrics: Arc::new(Mutex::new(QueryMetrics::default())),
            alert_thresholds: AlertThresholds::default(),
        }
    }
    
    pub fn record_query_start(&self) {
        let mut metrics = self.query_metrics.lock().unwrap();
        metrics.total_queries += 1;
        metrics.running_queries += 1;
    }
    
    pub fn record_query_complete(&self, latency: Duration) {
        let mut metrics = self.query_metrics.lock().unwrap();
        metrics.running_queries -= 1;
        metrics.completed_queries += 1;
        
        // 更新平均延迟
        metrics.average_latency = Self::update_average(
            metrics.average_latency,
            metrics.completed_queries,
            latency,
        );
    }
    
    pub fn record_query_failure(&self) {
        let mut metrics = self.query_metrics.lock().unwrap();
        metrics.running_queries -= 1;
        metrics.failed_queries += 1;
    }
    
    fn update_average(current: Duration, count: u64, new: Duration) -> Duration {
        let total = current.as_secs_f64() * (count - 1) as f64 + new.as_secs_f64();
        Duration::from_secs_f64(total / count as f64)
    }
    
    pub fn check_alerts(&self) -> Vec<Alert> {
        let mut alerts = Vec::new();
        let system = self.system_metrics.lock().unwrap();
        let query = self.query_metrics.lock().unwrap();
        
        if system.cpu_usage > self.alert_thresholds.cpu_threshold {
            alerts.push(Alert::new(
                AlertLevel::Warning,
                format!("High CPU usage: {:.1}%", system.cpu_usage * 100),
            ));
        }
        
        if system.memory_usage as f64 / system.memory_total as f64 
            > self.alert_thresholds.memory_threshold {
            alerts.push(Alert::new(
                AlertLevel::Critical,
                format!("High memory usage: {:.1}%", 
                    system.memory_usage as f64 / system.memory_total as f64 * 100),
            ));
        }
        
        if query.queued_queries > self.alert_thresholds.queue_threshold as u64 {
            alerts.push(Alert::new(
                AlertLevel::Warning,
                format!("Queue is full: {} queries", query.queued_queries),
            ));
        }
        
        alerts
    }
}

#[derive(Debug, Clone)]
pub struct Alert {
    pub level: AlertLevel,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AlertLevel {
    Info,
    Warning,
    Critical,
}

impl Alert {
    pub fn new(level: AlertLevel, message: String) -> Self {
        Self {
            level,
            message,
            timestamp: chrono::Utc::now(),
        }
    }
}
```

---

## 修改优先级

| 序号 | 修改方案 | 优先级 | 预估工作量 | 依赖 |
|------|----------|--------|------------|------|
| 8.1-8.3 | 解耦调度器与执行器 | 中 | 3-4 人天 | 无 |
| 8.4-8.5 | 资源监控和查询取消 | 低 | 2 人天 | 8.1-8.3 |

---

## 测试建议

### 测试用例 1: 查询队列

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_priority_queue_order() {
        let mut queue = QueryQueue::new();
        
        queue.push(QueryTask::new(
            "1".to_string(),
            "LOW priority query".to_string(),
            QueryPriority::Low,
        ));
        
        queue.push(QueryTask::new(
            "2".to_string(),
            "HIGH priority query".to_string(),
            QueryPriority::High,
        ));
        
        queue.push(QueryTask::new(
            "3".to_string(),
            "NORMAL priority query".to_string(),
            QueryPriority::Normal,
        ));
        
        // 应该先弹出 HIGH
        let task = queue.pop().unwrap();
        assert_eq!(task.priority, QueryPriority::High);
        assert_eq!(task.query_text, "HIGH priority query");
        
        // 然后是 NORMAL
        let task = queue.pop().unwrap();
        assert_eq!(task.priority, QueryPriority::Normal);
        
        // 最后是 LOW
        let task = queue.pop().unwrap();
        assert_eq!(task.priority, QueryPriority::Low);
    }
}
```

---

## 风险与注意事项

### 风险 1: 调度器性能

- **风险**: 调度器本身可能成为瓶颈
- **缓解措施**: 使用高效的数据结构
- **实现**: 使用 lock-free 队列

### 风险 2: 资源竞争

- **风险**: 多个工作线程竞争存储引擎
- **缓解措施**: 使用合理的锁策略
- **实现**: 评估存储引擎锁的粒度
