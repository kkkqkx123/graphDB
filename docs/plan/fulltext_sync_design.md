# 全文索引自动数据同步设计方案

**文档版本**: 1.0  
**创建日期**: 2026-04-09  
**状态**: 设计评审

---

## 执行摘要

本文档基于对 PostgreSQL、Elasticsearch、SQLite 等成熟数据库系统的全文索引实现研究，提出适合 GraphDB 项目的自动数据同步长期设计方案。

### 核心发现

| 数据库            | 同步机制                   | 优点                 | 缺点                           |
| ----------------- | -------------------------- | -------------------- | ------------------------------ |
| **PostgreSQL**    | Trigger + Generated Column | 实时性强、声明式语法 | 写入性能开销、trigger 维护复杂 |
| **Elasticsearch** | CDC + 异步队列             | 高吞吐、解耦、可扩展 | 最终一致性、架构复杂           |
| **SQLite FTS5**   | Virtual Table + Trigger    | 简单直接、零配置     | 单线程、功能有限               |

### 推荐方案

**分层事件驱动架构**：结合 Trigger 的实时性和 Event Bus 的解耦优势，实现高性能、可扩展的自动数据同步。

---

## 一、设计目标

### 1.1 核心需求

- ✅ **实时性**：数据变更后索引应尽快更新（准实时）
- ✅ **一致性**：保证数据与索引的最终一致性
- ✅ **性能**：写入性能开销 < 10%
- ✅ **可靠性**：不丢失更新、支持重试
- ✅ **可观测性**：同步状态可监控、可调试

### 1.2 非功能性需求

- **低耦合**：存储层与全文索引层解耦
- **可扩展**：支持多种索引类型（向量索引、空间索引等）
- **可配置**：支持同步策略配置（同步/异步、批量/单条）
- **容错性**：失败自动重试、死信队列

---

## 二、架构设计

### 2.1 整体架构图

```
┌─────────────────────────────────────────────────────────────┐
│                      Application Layer                       │
│  (Query Executors, API, etc.)                               │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Event Bus (EventHub)                      │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │VertexEvent  │  │ EdgeEvent   │  │ IndexEvent  │         │
│  └─────────────┘  └─────────────┘  └─────────────┘         │
└─────────────────────────────────────────────────────────────┘
                              │
              ┌───────────────┼───────────────┐
              │               │               │
              ▼               ▼               ▼
    ┌────────────────┐ ┌────────────┐ ┌──────────────┐
    │ Sync Handler   │ │Async Handler│ │ Audit Handler│
    │ (Immediate)    │ │ (Batch)     │ │ (Logging)    │
    └────────────────┘ └────────────┘ └──────────────┘
              │
              ▼
    ┌────────────────────────────────────────────────────┐
    │          FulltextCoordinator                        │
    │  ┌──────────────┐  ┌──────────────┐                │
    │  │ Index Manager│  │ Engine Pool  │                │
    │  └──────────────┘  └──────────────┘                │
    └────────────────────────────────────────────────────┘
                              │
                              ▼
    ┌────────────────────────────────────────────────────┐
    │          Search Engines (BM25/Inversearch)         │
    └────────────────────────────────────────────────────┘
```

### 2.2 核心组件

#### 2.2.1 事件系统（EventHub）

**设计灵感**：借鉴 Elasticsearch 的异步事件队列

```rust
/// 事件类型枚举
pub enum StorageEvent {
    /// 顶点插入事件
    VertexInserted {
        space_id: u64,
        vertex: Vertex,
        timestamp: u64,
    },
    /// 顶点更新事件
    VertexUpdated {
        space_id: u64,
        old_vertex: Vertex,
        new_vertex: Vertex,
        changed_fields: Vec<String>,
        timestamp: u64,
    },
    /// 顶点删除事件
    VertexDeleted {
        space_id: u64,
        vertex_id: Value,
        tag_name: String,
        timestamp: u64,
    },
    /// 边插入事件
    EdgeInserted {
        space_id: u64,
        edge: Edge,
        timestamp: u64,
    },
    /// 边删除事件
    EdgeDeleted {
        space_id: u64,
        src: Value,
        dst: Value,
        edge_type: String,
        rank: i64,
        timestamp: u64,
    },
}

/// 事件总线 trait
pub trait EventHub: Send + Sync {
    /// 发布事件
    fn publish(&self, event: StorageEvent) -> Result<(), EventError>;

    /// 批量发布事件
    fn publish_batch(&self, events: Vec<StorageEvent>) -> Result<(), EventError>;

    /// 订阅事件
    fn subscribe<F>(&self, event_type: EventType, handler: F) -> Result<SubscriptionId, EventError>
    where
        F: Fn(&StorageEvent) -> Result<(), EventHandlerError> + Send + Sync + 'static;

    /// 取消订阅
    fn unsubscribe(&self, subscription_id: SubscriptionId) -> Result<(), EventError>;
}
```

**实现策略**：

- **同步模式**：事件发布后立即阻塞执行 handler（默认，保证一致性）
- **异步模式**：事件发布到内存队列，后台线程批量处理（高性能）
- **混合模式**：关键数据同步，非关键数据异步

#### 2.2.2 存储层集成

**设计灵感**：借鉴 PostgreSQL 的 Trigger 机制

```rust
/// 存储层扩展 trait
pub trait StorageClientWithEvents: StorageClient {
    /// 设置事件总线
    fn set_event_hub(&mut self, event_hub: Arc<dyn EventHub>);

    /// 获取事件总线
    fn get_event_hub(&self) -> Option<Arc<dyn EventHub>>;

    /// 启用/禁用事件发布
    fn enable_events(&mut self, enabled: bool);
}

/// 存储操作包装器
pub struct EventEmittingStorage<S: StorageClient> {
    inner: S,
    event_hub: Arc<dyn EventHub>,
    enabled: bool,
}

impl<S: StorageClient> StorageClient for EventEmittingStorage<S> {
    fn insert_vertex(&mut self, space: &str, vertex: Vertex) -> Result<Value, StorageError> {
        // 1. 执行原始插入
        let result = self.inner.insert_vertex(space, vertex.clone())?;

        // 2. 发布事件
        if self.enabled {
            let event = StorageEvent::VertexInserted {
                space_id: self.get_space_id(space)?,
                vertex,
                timestamp: get_current_timestamp(),
            };
            self.event_hub.publish(event)?;
        }

        Ok(result)
    }

    fn update_vertex(&mut self, space: &str, vertex: Vertex) -> Result<(), StorageError> {
        // 1. 获取旧顶点
        let old_vertex = self.get_vertex(space, &vertex.vid)?
            .ok_or(StorageError::VertexNotFound(vertex.vid.clone()))?;

        // 2. 计算变更字段
        let changed_fields = compute_changed_fields(&old_vertex, &vertex);

        // 3. 执行更新
        self.inner.update_vertex(space, vertex.clone())?;

        // 4. 发布事件
        if self.enabled {
            let event = StorageEvent::VertexUpdated {
                space_id: self.get_space_id(space)?,
                old_vertex,
                new_vertex: vertex,
                changed_fields,
                timestamp: get_current_timestamp(),
            };
            self.event_hub.publish(event)?;
        }

        Ok(())
    }

    fn delete_vertex(&mut self, space: &str, id: &Value) -> Result<(), StorageError> {
        // 1. 获取顶点信息
        let vertex = self.get_vertex(space, id)?
            .ok_or(StorageError::VertexNotFound(id.clone()))?;

        // 2. 执行删除
        self.inner.delete_vertex(space, id)?;

        // 3. 发布事件
        if self.enabled {
            for tag in &vertex.tags {
                let event = StorageEvent::VertexDeleted {
                    space_id: self.get_space_id(space)?,
                    vertex_id: id.clone(),
                    tag_name: tag.name.clone(),
                    timestamp: get_current_timestamp(),
                };
                self.event_hub.publish(event)?;
            }
        }

        Ok(())
    }
}
```

#### 2.2.3 事件处理器

```rust
/// 全文索引同步处理器
pub struct FulltextSyncHandler {
    coordinator: Arc<FulltextCoordinator>,
    config: SyncConfig,
    metrics: SyncMetrics,
}

impl FulltextSyncHandler {
    pub fn new(coordinator: Arc<FulltextCoordinator>, config: SyncConfig) -> Self {
        Self {
            coordinator,
            config,
            metrics: SyncMetrics::default(),
        }
    }

    /// 处理顶点插入事件
    pub fn on_vertex_inserted(&self, event: &StorageEvent) -> Result<(), SyncError> {
        if let StorageEvent::VertexInserted { space_id, vertex, .. } = event {
            // 检查是否有全文索引
            let indexes = self.coordinator.get_space_indexes(*space_id);

            for tag in &vertex.tags {
                for (field_name, value) in &tag.properties {
                    // 检查该字段是否有全文索引
                    if indexes.iter().any(|idx| idx.tag_name == tag.name && idx.field_name == field_name) {
                        if let Value::String(text) = value {
                            // 同步到索引
                            futures::executor::block_on(
                                self.coordinator.on_vertex_change(
                                    *space_id,
                                    &tag.name,
                                    &vertex.vid,
                                    &hashmap! { field_name.clone() => value.clone() },
                                    ChangeType::Insert,
                                )
                            )?;

                            self.metrics.increment_insert_success();
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// 处理顶点更新事件
    pub fn on_vertex_updated(&self, event: &StorageEvent) -> Result<(), SyncError> {
        if let StorageEvent::VertexUpdated { space_id, new_vertex, changed_fields, .. } = event {
            // 只处理变更的字段
            let mut properties = HashMap::new();

            for tag in &new_vertex.tags {
                for field_name in changed_fields {
                    if let Some(value) = tag.properties.get(field_name) {
                        properties.insert(field_name.clone(), value.clone());
                    }
                }
            }

            if !properties.is_empty() {
                futures::executor::block_on(
                    self.coordinator.on_vertex_change(
                        *space_id,
                        &new_vertex.tags[0].name,
                        &new_vertex.vid,
                        &properties,
                        ChangeType::Update,
                    )
                )?;

                self.metrics.increment_update_success();
            }
        }
        Ok(())
    }

    /// 处理顶点删除事件
    pub fn on_vertex_deleted(&self, event: &StorageEvent) -> Result<(), SyncError> {
        if let StorageEvent::VertexDeleted { space_id, tag_name, vertex_id, .. } = event {
            futures::executor::block_on(
                self.coordinator.on_vertex_deleted(*space_id, tag_name, vertex_id)
            )?;

            self.metrics.increment_delete_success();
        }
        Ok(())
    }
}
```

#### 2.2.4 异步批量处理

**设计灵感**：借鉴 Elasticsearch 的 Bulk API

```rust
/// 异步批量处理器
pub struct BatchSyncProcessor {
    event_queue: Arc<Mutex<VecDeque<StorageEvent>>>,
    coordinator: Arc<FulltextCoordinator>,
    batch_size: usize,
    flush_interval: Duration,
    shutdown: Arc<AtomicBool>,
}

impl BatchSyncProcessor {
    pub fn new(coordinator: Arc<FulltextCoordinator>, batch_size: usize, flush_interval: Duration) -> Self {
        Self {
            event_queue: Arc::new(Mutex::new(VecDeque::new())),
            coordinator,
            batch_size,
            flush_interval,
            shutdown: Arc::new(AtomicBool::new(false)),
        }
    }

    /// 启动后台处理线程
    pub fn start(&self) -> JoinHandle<()> {
        let queue = self.event_queue.clone();
        let coordinator = self.coordinator.clone();
        let batch_size = self.batch_size;
        let flush_interval = self.flush_interval;
        let shutdown = self.shutdown.clone();

        std::thread::spawn(move || {
            while !shutdown.load(Ordering::Relaxed) {
                // 等待事件或超时
                let events = {
                    let mut queue = queue.lock();
                    if queue.is_empty() {
                        std::thread::sleep(flush_interval);
                        continue;
                    }

                    // 取出一批事件
                    queue.drain(..batch_size.min(queue.len())).collect()
                };

                // 批量处理
                if !events.is_empty() {
                    let _ = Self::process_batch(&coordinator, events);
                }
            }
        })
    }

    fn process_batch(coordinator: &FulltextCoordinator, events: Vec<StorageEvent>) -> Result<(), SyncError> {
        // 按 space_id 和类型分组，减少索引切换
        let mut grouped: HashMap<(u64, String), Vec<(Value, HashMap<String, Value>)>> = HashMap::new();

        for event in events {
            match event {
                StorageEvent::VertexInserted { space_id, vertex, .. } |
                StorageEvent::VertexUpdated { space_id, new_vertex: vertex, changed_fields, .. } => {
                    for tag in &vertex.tags {
                        let key = (space_id, tag.name.clone());
                        let entry = grouped.entry(key).or_insert_with(Vec::new);

                        // 合并同一顶点的多个字段更新
                        if let Some((_, props)) = entry.iter_mut().find(|(vid, _)| *vid == vertex.vid) {
                            for field_name in &changed_fields {
                                if let Some(value) = tag.properties.get(field_name) {
                                    props.insert(field_name.clone(), value.clone());
                                }
                            }
                        } else {
                            let mut props = HashMap::new();
                            for field_name in &changed_fields {
                                if let Some(value) = tag.properties.get(field_name) {
                                    props.insert(field_name.clone(), value.clone());
                                }
                            }
                            entry.push((vertex.vid.clone(), props));
                        }
                    }
                }
                _ => {}
            }
        }

        // 批量写入索引
        for ((space_id, tag_name), updates) in grouped {
            for (vertex_id, properties) in updates {
                futures::executor::block_on(
                    coordinator.on_vertex_change(
                        space_id,
                        &tag_name,
                        &vertex_id,
                        &properties,
                        ChangeType::Update,
                    )
                )?;
            }
        }

        Ok(())
    }
}
```

---

## 三、实现方案

### 3.1 Phase 1: 基础事件系统（1-2 周）

**目标**：实现基本的事件发布订阅机制

**任务**：

1. 定义 `StorageEvent` 枚举
2. 实现 `EventHub` trait 和内存实现
3. 修改存储层集成事件发布
4. 实现 `FulltextSyncHandler`

**代码结构**：

```
src/
├── event/
│   ├── mod.rs          # EventHub trait 和类型定义
│   ├── hub.rs          # EventHub 实现
│   ├── types.rs        # StorageEvent 等类型
│   └── handler.rs      # 事件处理器 trait
├── storage/
│   └── event_storage.rs # 包装器实现
└── coordinator/
    └── fulltext.rs     # 添加事件处理方法
```

### 3.2 Phase 2: 同步模式实现（1 周）

**目标**：实现默认的同步数据同步

**配置**：

```rust
pub struct SyncConfig {
    /// 同步模式：Sync/Async
    pub mode: SyncMode,
    /// 是否启用事务内同步
    pub sync_in_transaction: bool,
    /// 失败重试次数
    pub max_retries: u32,
    /// 重试间隔
    pub retry_interval: Duration,
}

pub enum SyncMode {
    /// 同步模式：事件发布后立即执行 handler
    Synchronous,
    /// 异步模式：事件发布到队列，后台处理
    Asynchronous {
        batch_size: usize,
        flush_interval: Duration,
    },
}
```

**集成点**：

- 在 `StorageClient` 初始化时设置 EventHub
- 在执行器层调用存储操作前启用事件

### 3.3 Phase 3: 异步批量处理（2 周）

**目标**：实现高性能异步批量同步

**特性**：

- 后台批量处理线程
- 事件分组优化
- 失败重试机制
- 死信队列

### 3.4 Phase 4: 可观测性和监控（1 周）

**目标**：实现同步状态监控

**指标**：

```rust
pub struct SyncMetrics {
    /// 插入成功/失败计数
    pub insert_success: AtomicU64,
    pub insert_failed: AtomicU64,
    /// 更新成功/失败计数
    pub update_success: AtomicU64,
    pub update_failed: AtomicU64,
    /// 删除成功/失败计数
    pub delete_success: AtomicU64,
    pub delete_failed: AtomicU64,
    /// 同步延迟（P50, P95, P99）
    pub sync_latency_p50: AtomicU64,
    pub sync_latency_p95: AtomicU64,
    pub sync_latency_p99: AtomicU64,
    /// 队列长度
    pub queue_size: AtomicUsize,
}
```

**API**：

```rust
// 获取同步状态
GET /api/v1/fulltext/sync/status

// 获取同步指标
GET /api/v1/fulltext/sync/metrics

// 手动触发同步
POST /api/v1/fulltext/sync/trigger
```

---

## 四、配置示例

### 4.1 基础配置

```toml
# config.toml

[fulltext]
enabled = true
default_engine = "Bm25"

[fulltext.sync]
# 同步模式：sync | async
mode = "sync"

# 同步策略
[fulltext.sync.policy]
# 是否在事务内同步（true 保证强一致性，false 性能更好）
sync_in_transaction = true
# 失败重试次数
max_retries = 3
# 重试间隔（毫秒）
retry_interval_ms = 1000

# 异步模式配置（仅当 mode = "async" 时有效）
[fulltext.async]
# 批量大小
batch_size = 100
# 刷新间隔（毫秒）
flush_interval_ms = 1000
# 队列容量
queue_capacity = 10000
```

### 4.2 使用示例

```rust
// 初始化存储层和事件系统
let storage = RedbStorage::new(config)?;
let coordinator = Arc::new(FulltextCoordinator::new()?);

// 创建事件总线
let event_hub = Arc::new(MemoryEventHub::new());

// 包装存储层，启用事件
let mut storage_with_events = EventEmittingStorage::new(storage, event_hub.clone());
storage_with_events.enable_events(true);

// 订阅全文索引同步事件
let sync_handler = FulltextSyncHandler::new(coordinator);
event_hub.subscribe(EventType::VertexEvent, move |event| {
    sync_handler.handle(event)
})?;

// 启动异步处理器（可选）
if config.sync_mode == SyncMode::Asynchronous {
    let batch_processor = BatchSyncProcessor::new(coordinator, 100, Duration::from_millis(1000));
    batch_processor.start();
}
```

---

## 五、性能评估

### 5.1 预期性能指标

| 指标         | 同步模式 | 异步模式 | 目标 |
| ------------ | -------- | -------- | ---- |
| 写入延迟增加 | < 10%    | < 5%     | ✅   |
| 同步延迟     | < 10ms   | < 1s     | ✅   |
| 吞吐量影响   | < 10%    | +20%     | ✅   |
| 内存开销     | < 50MB   | < 200MB  | ✅   |

### 5.2 基准测试

```rust
#[bench]
fn benchmark_vertex_insert_with_sync(b: &mut Bencher) {
    let mut storage = create_storage_with_sync();

    b.iter(|| {
        storage.insert_vertex("test_space", create_vertex())
    });
}

#[bench]
fn benchmark_vertex_insert_async(b: &mut Bencher) {
    let mut storage = create_storage_with_async();

    b.iter(|| {
        storage.insert_vertex("test_space", create_vertex())
    });
}
```

---

## 六、容错和恢复

### 6.1 失败重试

```rust
pub async fn retry_with_backoff<F, T>(
    max_retries: u32,
    base_delay: Duration,
    operation: F,
) -> Result<T, SyncError>
where
    F: Fn() -> Result<T, SyncError>,
{
    let mut delay = base_delay;

    for attempt in 1..=max_retries {
        match operation() {
            Ok(result) => return Ok(result),
            Err(e) if attempt == max_retries => return Err(e),
            Err(_) => {
                tokio::time::sleep(delay).await;
                delay *= 2; // 指数退避
            }
        }
    }

    unreachable!()
}
```

### 6.2 死信队列

```rust
pub struct DeadLetterQueue {
    queue: Arc<Mutex<VecDeque<(StorageEvent, SyncError, u32)>>>,
    max_size: usize,
}

impl DeadLetterQueue {
    pub fn push(&self, event: StorageEvent, error: SyncError, retry_count: u32) {
        let mut queue = self.queue.lock();

        if queue.len() >= self.max_size {
            // 丢弃最旧的事件
            queue.pop_front();
        }

        queue.push_back((event, error, retry_count));
    }

    pub fn get_failed_events(&self) -> Vec<(StorageEvent, SyncError)> {
        self.queue.lock().iter().map(|(e, err, _)| (e.clone(), err.clone())).collect()
    }
}
```

### 6.3 数据一致性检查

```rust
pub async fn verify_consistency(
    storage: &dyn StorageClient,
    coordinator: &FulltextCoordinator,
) -> Result<ConsistencyReport, ConsistencyError> {
    let mut report = ConsistencyReport::default();

    // 扫描所有顶点
    let vertices = storage.scan_vertices("test_space")?;

    for vertex in vertices {
        // 检查每个全文索引字段
        for tag in &vertex.tags {
            for (field_name, value) in &tag.properties {
                if let Value::String(text) = value {
                    // 在索引中搜索
                    let results = coordinator.search(field_name, &vertex.vid.to_string()).await?;

                    if results.is_empty() && !text.is_empty() {
                        report.missing_indexes.push(vertex.vid.clone());
                    }
                }
            }
        }
    }

    Ok(report)
}
```

---

## 七、与其他数据库对比

### 7.1 PostgreSQL vs GraphDB

| 特性     | PostgreSQL       | GraphDB（推荐方案） |
| -------- | ---------------- | ------------------- |
| 同步机制 | Trigger          | EventHub + Handler  |
| 一致性   | 强一致（事务内） | 可配置（同步/异步） |
| 性能开销 | 固定（~15%）     | 可优化（异步<5%）   |
| 扩展性   | 有限             | 高（支持多种索引）  |
| 可观测性 | 系统表           | 专用 API + 指标     |

### 7.2 Elasticsearch vs GraphDB

| 特性       | Elasticsearch  | GraphDB（推荐方案） |
| ---------- | -------------- | ------------------- |
| 同步机制   | CDC + Logstash | EventHub            |
| 延迟       | 秒级（近实时） | 毫秒级（可配置）    |
| 架构复杂度 | 高             | 中等                |
| 适用场景   | 大规模分布式   | 单节点高性能        |

### 7.3 SQLite FTS5 vs GraphDB

| 特性       | SQLite FTS5             | GraphDB（推荐方案） |
| ---------- | ----------------------- | ------------------- |
| 同步机制   | Virtual Table + Trigger | EventHub + Handler  |
| 配置复杂度 | 零配置                  | 可配置              |
| 性能       | 单线程                  | 多线程异步          |
| 扩展性     | 低                      | 高                  |

---

## 八、风险评估

| 风险     | 影响 | 概率 | 缓解措施                 |
| -------- | ---- | ---- | ------------------------ |
| 事件丢失 | 高   | 低   | 持久化事件队列、WAL 日志 |
| 同步延迟 | 中   | 中   | 监控告警、自动扩容       |
| 内存溢出 | 中   | 低   | 队列限流、背压机制       |
| 死锁     | 低   | 低   | 避免循环依赖、超时机制   |

---

## 九、验收标准

### 9.1 功能验收

- [ ] 插入顶点自动同步到全文索引
- [ ] 更新顶点自动更新全文索引
- [ ] 删除顶点自动删除全文索引
- [ ] 支持同步和异步两种模式
- [ ] 支持失败重试和死信队列

### 9.2 性能验收

- [ ] 同步模式写入延迟增加 < 10%
- [ ] 异步模式写入延迟增加 < 5%
- [ ] 同步延迟 P99 < 100ms（同步模式）
- [ ] 同步延迟 P99 < 1s（异步模式）

### 9.3 可靠性验收

- [ ] 10000 次写入无事件丢失
- [ ] 失败自动重试成功率 > 99%
- [ ] 系统重启后能恢复同步状态

---

## 十、结论

本方案借鉴了 PostgreSQL 的 Trigger 机制、Elasticsearch 的异步队列和 SQLite FTS5 的简单直接，提出了适合 GraphDB 的分层事件驱动架构。

**核心优势**：

1. **解耦**：存储层与索引层通过事件总线解耦
2. **灵活**：支持同步/异步、单条/批量多种模式
3. **可扩展**：易于添加新的索引类型和处理器
4. **可观测**：完善的监控和指标系统

**推荐实施路径**：

1. Phase 1: 基础事件系统（2 周）
2. Phase 2: 同步模式（1 周）
3. Phase 3: 异步批量（2 周）
4. Phase 4: 监控告警（1 周）

总工期：6 周

---

**文档结束**
