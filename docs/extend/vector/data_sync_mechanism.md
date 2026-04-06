# 向量数据同步机制

> 分析日期: 2026-04-06
> 依赖: 现有 sync 模块架构

---

## 目录

- [1. 同步架构概述](#1-同步架构概述)
- [2. 同步任务扩展](#2-同步任务扩展)
- [3. 同步管理器扩展](#3-同步管理器扩展)
- [4. 向量协调器](#4-向量协调器)
- [5. 批量处理](#5-批量处理)
- [6. 故障恢复](#6-故障恢复)
- [7. 一致性保证](#7-一致性保证)

---

## 1. 同步架构概述

### 1.1 整体架构

```
┌─────────────────────────────────────────────────────────────┐
│                    Storage Layer                             │
│  RedbStorage - 图数据存储                                   │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                    Transaction Layer                         │
│  TransactionManager - 事务管理                              │
│  on_commit() → 触发同步                                     │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                    Sync Layer                                │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ SyncManager                                          │   │
│  │ ├── FulltextSync (现有)                             │   │
│  │ └── VectorSync (新增)                               │   │
│  └─────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ TaskBuffer                                           │   │
│  │ ├── FulltextTaskBuffer                              │   │
│  │ └── VectorTaskBuffer                                │   │
│  └─────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ RecoveryManager                                      │   │
│  │ ├── FailedTaskPersistence                           │   │
│  │ └── RetryScheduler                                  │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                    Coordinator Layer                         │
│  FulltextCoordinator + VectorCoordinator                    │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                    Engine Layer                              │
│  SearchEngine (BM25/Inversearch) + VectorEngine (Qdrant)   │
└─────────────────────────────────────────────────────────────┘
```

### 1.2 同步模式

| 模式 | 行为 | 适用场景 |
|------|------|---------|
| Sync | 阻塞等待向量索引完成 | 强一致性要求 |
| Async | 提交到队列立即返回 | 默认推荐，高性能 |
| Off | 不更新向量索引 | 维护模式 |

---

## 2. 同步任务扩展

### 2.1 任务类型定义

```rust
// src/sync/task.rs (扩展)

use crate::core::Value;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncTask {
    // 现有全文检索任务
    VertexChange {
        task_id: String,
        space_id: u64,
        tag_name: String,
        vertex_id: Value,
        properties: Vec<(String, Value)>,
        change_type: ChangeType,
        created_at: DateTime<Utc>,
    },
    BatchIndex {
        task_id: String,
        space_id: u64,
        tag_name: String,
        field_name: String,
        documents: Vec<(String, String)>,
        created_at: DateTime<Utc>,
    },
    CommitIndex {
        task_id: String,
        space_id: u64,
        tag_name: String,
        field_name: String,
        created_at: DateTime<Utc>,
    },
    RebuildIndex {
        task_id: String,
        space_id: u64,
        tag_name: String,
        field_name: String,
        created_at: DateTime<Utc>,
    },
    BatchDelete {
        task_id: String,
        space_id: u64,
        tag_name: String,
        field_name: String,
        doc_ids: Vec<String>,
        created_at: DateTime<Utc>,
    },
    
    // 新增向量同步任务
    VectorChange {
        task_id: String,
        space_id: u64,
        tag_name: String,
        field_name: String,
        vertex_id: Value,
        vector: Option<Vec<f32>>,
        payload: HashMap<String, Value>,
        change_type: VectorChangeType,
        created_at: DateTime<Utc>,
    },
    VectorBatchUpsert {
        task_id: String,
        space_id: u64,
        tag_name: String,
        field_name: String,
        points: Vec<VectorPoint>,
        created_at: DateTime<Utc>,
    },
    VectorBatchDelete {
        task_id: String,
        space_id: u64,
        tag_name: String,
        field_name: String,
        point_ids: Vec<String>,
        created_at: DateTime<Utc>,
    },
    VectorRebuildIndex {
        task_id: String,
        space_id: u64,
        tag_name: String,
        field_name: String,
        created_at: DateTime<Utc>,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum VectorChangeType {
    Insert,
    Update,
    Delete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorPoint {
    pub id: String,
    pub vector: Vec<f32>,
    pub payload: HashMap<String, Value>,
}

impl SyncTask {
    // 现有方法...
    
    // 新增向量任务创建方法
    pub fn vector_change(
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        vertex_id: &Value,
        vector: Option<Vec<f32>>,
        payload: HashMap<String, Value>,
        change_type: VectorChangeType,
    ) -> Self {
        Self::VectorChange {
            task_id: generate_task_id(),
            space_id,
            tag_name: tag_name.to_string(),
            field_name: field_name.to_string(),
            vertex_id: vertex_id.clone(),
            vector,
            payload,
            change_type,
            created_at: Utc::now(),
        }
    }
    
    pub fn vector_batch_upsert(
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        points: Vec<VectorPoint>,
    ) -> Self {
        Self::VectorBatchUpsert {
            task_id: generate_task_id(),
            space_id,
            tag_name: tag_name.to_string(),
            field_name: field_name.to_string(),
            points,
            created_at: Utc::now(),
        }
    }
    
    pub fn vector_batch_delete(
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        point_ids: Vec<String>,
    ) -> Self {
        Self::VectorBatchDelete {
            task_id: generate_task_id(),
            space_id,
            tag_name: tag_name.to_string(),
            field_name: field_name.to_string(),
            point_ids,
            created_at: Utc::now(),
        }
    }
    
    pub fn is_vector_task(&self) -> bool {
        matches!(
            self,
            Self::VectorChange { .. }
                | Self::VectorBatchUpsert { .. }
                | Self::VectorBatchDelete { .. }
                | Self::VectorRebuildIndex { .. }
        )
    }
}
```

### 2.2 任务优先级

```rust
impl SyncTask {
    pub fn priority(&self) -> u8 {
        match self {
            // 高优先级：删除操作需要尽快执行
            Self::BatchDelete { .. } => 10,
            Self::VectorBatchDelete { .. } => 10,
            
            // 中优先级：单点变更
            Self::VertexChange { .. } => 5,
            Self::VectorChange { .. } => 5,
            
            // 低优先级：批量操作
            Self::BatchIndex { .. } => 3,
            Self::VectorBatchUpsert { .. } => 3,
            
            // 最低优先级：重建索引
            Self::RebuildIndex { .. } => 1,
            Self::VectorRebuildIndex { .. } => 1,
            Self::CommitIndex { .. } => 1,
        }
    }
}
```

---

## 3. 同步管理器扩展

### 3.1 扩展SyncManager

```rust
// src/sync/manager.rs (扩展)

use crate::coordinator::{FulltextCoordinator, VectorCoordinator};
use crate::sync::batch::TaskBuffer;
use crate::sync::recovery::RecoveryManager;
use crate::sync::task::SyncTask;

pub struct SyncManager {
    fulltext_coordinator: Arc<FulltextCoordinator>,
    vector_coordinator: Option<Arc<VectorCoordinator>>,
    buffer: Arc<TaskBuffer>,
    mode: Arc<RwLock<SyncMode>>,
    running: Arc<AtomicBool>,
    recovery: Option<Arc<RecoveryManager>>,
    handle: Mutex<Option<JoinHandle<()>>>,
}

impl SyncManager {
    pub fn new(
        fulltext_coordinator: Arc<FulltextCoordinator>,
        config: BatchConfig,
    ) -> Self {
        let buffer = Arc::new(TaskBuffer::new(fulltext_coordinator.clone(), config));
        
        Self {
            fulltext_coordinator,
            vector_coordinator: None,
            buffer,
            mode: Arc::new(RwLock::new(SyncMode::Async)),
            running: Arc::new(AtomicBool::new(false)),
            recovery: None,
            handle: Mutex::new(None),
        }
    }
    
    pub fn with_vector_coordinator(
        mut self,
        vector_coordinator: Arc<VectorCoordinator>,
    ) -> Self {
        self.vector_coordinator = Some(vector_coordinator);
        self
    }
    
    pub fn with_recovery(mut self, data_dir: PathBuf) -> Self {
        self.recovery = Some(Arc::new(RecoveryManager::new(
            self.buffer.clone(),
            data_dir,
        )));
        self
    }
    
    /// 处理顶点变更（同时处理全文和向量）
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
                // 同步模式：直接执行
                self.execute_vertex_change_sync(
                    space_id, tag_name, vertex_id, properties, change_type
                ).await?;
            }
            SyncMode::Async => {
                // 异步模式：提交到队列
                let task = SyncTask::vertex_change(
                    space_id, tag_name, vertex_id, properties.to_vec(), change_type
                );
                self.buffer.submit(task).await?;
            }
            SyncMode::Off => {}
        }
        
        Ok(())
    }
    
    /// 处理向量变更
    pub async fn on_vector_change(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        vertex_id: &Value,
        vector: Option<Vec<f32>>,
        payload: HashMap<String, Value>,
        change_type: VectorChangeType,
    ) -> Result<(), SyncError> {
        let mode = *self.mode.read().await;
        
        if self.vector_coordinator.is_none() {
            return Ok(());
        }
        
        match mode {
            SyncMode::Sync => {
                self.execute_vector_change_sync(
                    space_id, tag_name, field_name, vertex_id, vector, payload, change_type
                ).await?;
            }
            SyncMode::Async => {
                let task = SyncTask::vector_change(
                    space_id, tag_name, field_name, vertex_id, vector, payload, change_type
                );
                self.buffer.submit(task).await?;
            }
            SyncMode::Off => {}
        }
        
        Ok(())
    }
    
    async fn execute_vertex_change_sync(
        &self,
        space_id: u64,
        tag_name: &str,
        vertex_id: &Value,
        properties: &[(String, Value)],
        change_type: ChangeType,
    ) -> Result<(), SyncError> {
        // 处理全文索引
        let props: HashMap<_, _> = properties.iter().cloned().collect();
        self.fulltext_coordinator
            .on_vertex_change(space_id, tag_name, vertex_id, &props, change_type)
            .await?;
        
        // 处理向量索引
        if let Some(ref vector_coord) = self.vector_coordinator {
            for (field_name, value) in properties {
                if let Some(vector) = value.as_vector() {
                    let mut payload = HashMap::new();
                    payload.insert("vertex_id".to_string(), vertex_id.clone());
                    
                    vector_coord.on_vector_change(
                        space_id,
                        tag_name,
                        field_name,
                        vertex_id,
                        Some(vector),
                        payload,
                        change_type.into(),
                    ).await?;
                }
            }
        }
        
        Ok(())
    }
}
```

### 3.2 任务执行

```rust
impl SyncManager {
    async fn execute_task(
        buffer: &TaskBuffer,
        task: &SyncTask,
        recovery: Option<&Arc<RecoveryManager>>,
    ) {
        let result: Result<(), SyncError> = match task {
            // 全文检索任务
            SyncTask::VertexChange { .. } => {
                Self::execute_fulltext_task(buffer, task).await
            }
            SyncTask::BatchIndex { .. } => {
                Self::execute_fulltext_task(buffer, task).await
            }
            SyncTask::BatchDelete { .. } => {
                Self::execute_fulltext_task(buffer, task).await
            }
            SyncTask::CommitIndex { .. } => {
                Self::execute_fulltext_task(buffer, task).await
            }
            SyncTask::RebuildIndex { .. } => {
                Self::execute_fulltext_task(buffer, task).await
            }
            
            // 向量检索任务
            SyncTask::VectorChange { .. } => {
                Self::execute_vector_task(buffer, task).await
            }
            SyncTask::VectorBatchUpsert { .. } => {
                Self::execute_vector_task(buffer, task).await
            }
            SyncTask::VectorBatchDelete { .. } => {
                Self::execute_vector_task(buffer, task).await
            }
            SyncTask::VectorRebuildIndex { .. } => {
                Self::execute_vector_task(buffer, task).await
            }
        };
        
        match result {
            Ok(_) => {
                log::debug!("Task executed successfully: {}", task.task_id());
            }
            Err(e) => {
                log::error!("Task execution failed [{}]: {}", task.task_id(), e);
                if let Some(recovery) = recovery {
                    if let Err(re) = recovery.record_failure(task.clone(), e.to_string()).await {
                        log::error!("Failed to record task failure: {}", re);
                    }
                }
            }
        }
    }
    
    async fn execute_vector_task(
        buffer: &TaskBuffer,
        task: &SyncTask,
    ) -> Result<(), SyncError> {
        let coordinator = buffer.vector_coordinator()
            .ok_or(SyncError::VectorCoordinatorNotAvailable)?;
        
        match task {
            SyncTask::VectorChange {
                space_id,
                tag_name,
                field_name,
                vertex_id,
                vector,
                payload,
                change_type,
                ..
            } => {
                coordinator.on_vector_change(
                    *space_id,
                    tag_name,
                    field_name,
                    vertex_id,
                    vector.clone(),
                    payload.clone(),
                    *change_type,
                ).await?;
            }
            SyncTask::VectorBatchUpsert {
                space_id,
                tag_name,
                field_name,
                points,
                ..
            } => {
                if let Some(engine) = coordinator.get_engine(*space_id, tag_name, field_name) {
                    engine.upsert_batch(points.clone()).await?;
                }
            }
            SyncTask::VectorBatchDelete {
                space_id,
                tag_name,
                field_name,
                point_ids,
                ..
            } => {
                if let Some(engine) = coordinator.get_engine(*space_id, tag_name, field_name) {
                    engine.delete_batch(point_ids.iter().map(|s| s.as_str()).collect()).await?;
                }
            }
            SyncTask::VectorRebuildIndex {
                space_id,
                tag_name,
                field_name,
                ..
            } => {
                coordinator.rebuild_index(*space_id, tag_name, field_name).await?;
            }
            _ => {}
        }
        
        Ok(())
    }
}
```

---

## 4. 向量协调器

### 4.1 协调器实现

```rust
// src/coordinator/vector.rs

use crate::core::error::{CoordinatorError, CoordinatorResult};
use crate::core::{Value, Vertex};
use crate::vector::{VectorEngine, VectorIndexManager, VectorSearchResult};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug)]
pub struct VectorCoordinator {
    manager: Arc<VectorIndexManager>,
    embedding_service: Option<Arc<dyn EmbeddingService>>,
}

impl VectorCoordinator {
    pub fn new(manager: Arc<VectorIndexManager>) -> Self {
        Self {
            manager,
            embedding_service: None,
        }
    }
    
    pub fn with_embedding(
        mut self,
        embedding_service: Arc<dyn EmbeddingService>,
    ) -> Self {
        self.embedding_service = Some(embedding_service);
        self
    }
    
    pub async fn create_vector_index(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        vector_size: usize,
        distance: DistanceMetric,
    ) -> CoordinatorResult<String> {
        self.manager
            .create_index(space_id, tag_name, field_name, vector_size, distance)
            .await
            .map_err(CoordinatorError::from)?;
        
        Ok(format!("{}_{}_{}", space_id, tag_name, field_name))
    }
    
    pub async fn on_vector_change(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        vertex_id: &Value,
        vector: Option<Vec<f32>>,
        payload: HashMap<String, Value>,
        change_type: VectorChangeType,
    ) -> CoordinatorResult<()> {
        let engine = self.manager.get_engine(space_id, tag_name, field_name);
        
        if engine.is_none() {
            return Ok(());
        }
        
        let engine = engine.unwrap();
        let point_id = vertex_id.to_string();
        
        match change_type {
            VectorChangeType::Insert | VectorChangeType::Update => {
                if let Some(vec) = vector {
                    engine.upsert(&point_id, vec, Some(payload)).await
                        .map_err(CoordinatorError::from)?;
                }
            }
            VectorChangeType::Delete => {
                engine.delete(&point_id).await
                    .map_err(CoordinatorError::from)?;
            }
        }
        
        Ok(())
    }
    
    pub async fn on_vertex_inserted(
        &self,
        space_id: u64,
        vertex: &Vertex,
    ) -> CoordinatorResult<()> {
        for tag in &vertex.tags {
            for (field_name, value) in &tag.properties {
                if let Some(vector) = value.as_vector() {
                    if let Some(engine) = self.manager.get_engine(
                        space_id, &tag.name, field_name
                    ) {
                        let point_id = vertex.vid.to_string();
                        let mut payload = HashMap::new();
                        payload.insert("vertex_id".to_string(), vertex.vid.clone());
                        
                        engine.upsert(&point_id, vector, Some(payload)).await
                            .map_err(CoordinatorError::from)?;
                    }
                }
            }
        }
        Ok(())
    }
    
    pub async fn on_vertex_deleted(
        &self,
        space_id: u64,
        tag_name: &str,
        vertex_id: &Value,
    ) -> CoordinatorResult<()> {
        let point_id = vertex_id.to_string();
        let indexes = self.manager.get_space_indexes(space_id);
        
        for metadata in indexes {
            if metadata.tag_name == tag_name {
                if let Some(engine) = self.manager.get_engine(
                    space_id, &metadata.tag_name, &metadata.field_name
                ) {
                    engine.delete(&point_id).await
                        .map_err(CoordinatorError::from)?;
                }
            }
        }
        Ok(())
    }
    
    pub async fn search(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        query_vector: Vec<f32>,
        limit: usize,
        filter: Option<VectorFilter>,
    ) -> CoordinatorResult<Vec<VectorSearchResult>> {
        self.manager
            .search(space_id, tag_name, field_name, query_vector, limit, filter)
            .await
            .map_err(CoordinatorError::from)
    }
    
    pub async fn embed_text(&self, text: &str) -> CoordinatorResult<Vec<f32>> {
        let embedding_service = self.embedding_service.as_ref()
            .ok_or(CoordinatorError::EmbeddingServiceNotAvailable)?;
        
        embedding_service.embed(text).await
            .map_err(|e| CoordinatorError::EmbeddingError(e.to_string()))
    }
    
    pub fn get_engine(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
    ) -> Option<Arc<dyn VectorEngine>> {
        self.manager.get_engine(space_id, tag_name, field_name)
    }
    
    pub async fn index_exists(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
    ) -> bool {
        self.manager.has_index(space_id, tag_name, field_name)
    }
    
    pub async fn rebuild_index(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
    ) -> CoordinatorResult<()> {
        // TODO: 实现从存储层重新加载数据并重建索引
        Ok(())
    }
}
```

---

## 5. 批量处理

### 5.1 向量批量缓冲区

```rust
// src/sync/batch.rs (扩展)

pub struct VectorTaskBuffer {
    coordinator: Arc<VectorCoordinator>,
    config: BatchConfig,
    point_buffers: Mutex<HashMap<IndexKey, Vec<VectorPoint>>>,
    delete_buffers: Mutex<HashMap<IndexKey, Vec<String>>>,
    last_commit: Mutex<HashMap<IndexKey, Instant>>,
}

impl VectorTaskBuffer {
    pub fn new(coordinator: Arc<VectorCoordinator>, config: BatchConfig) -> Self {
        Self {
            coordinator,
            config,
            point_buffers: Mutex::new(HashMap::new()),
            delete_buffers: Mutex::new(HashMap::new()),
            last_commit: Mutex::new(HashMap::new()),
        }
    }
    
    pub async fn add_point(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        point: VectorPoint,
    ) {
        let key = (space_id, tag_name.to_string(), field_name.to_string());
        
        let mut buffers = self.point_buffers.lock().await;
        buffers.entry(key.clone()).or_default().push(point);
        
        let mut last_commit = self.last_commit.lock().await;
        last_commit.entry(key).or_insert_with(Instant::now);
    }
    
    pub async fn add_deletion(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        point_id: String,
    ) {
        let key = (space_id, tag_name.to_string(), field_name.to_string());
        
        let mut buffers = self.delete_buffers.lock().await;
        buffers.entry(key.clone()).or_default().push(point_id);
        
        let mut last_commit = self.last_commit.lock().await;
        last_commit.entry(key).or_insert_with(Instant::now);
    }
    
    pub async fn should_commit(&self, key: &IndexKey) -> bool {
        // 检查批量大小
        {
            let buffers = self.point_buffers.lock().await;
            if let Some(buffer) = buffers.get(key) {
                if buffer.len() >= self.config.batch_size {
                    return true;
                }
            }
        }
        
        {
            let buffers = self.delete_buffers.lock().await;
            if let Some(buffer) = buffers.get(key) {
                if buffer.len() >= self.config.batch_size {
                    return true;
                }
            }
        }
        
        // 检查时间间隔
        let last_commit = self.last_commit.lock().await;
        if let Some(last) = last_commit.get(key) {
            if last.elapsed() >= self.config.commit_interval {
                return true;
            }
        }
        
        false
    }
    
    pub async fn commit_batch(&self, key: IndexKey) -> Result<(), BufferError> {
        let mut buffers = self.point_buffers.lock().await;
        
        if let Some(points) = buffers.remove(&key) {
            if points.is_empty() {
                return Ok(());
            }
            
            let (space_id, tag_name, field_name) = key.clone();
            
            if let Some(engine) = self.coordinator.get_engine(space_id, &tag_name, &field_name) {
                engine.upsert_batch(points).await
                    .map_err(|e| BufferError::IndexError(e.to_string()))?;
            }
            
            let mut last_commit = self.last_commit.lock().await;
            last_commit.insert(key, Instant::now());
        }
        
        Ok(())
    }
    
    pub async fn commit_deletions(&self, key: IndexKey) -> Result<(), BufferError> {
        let mut buffers = self.delete_buffers.lock().await;
        
        if let Some(point_ids) = buffers.remove(&key) {
            if point_ids.is_empty() {
                return Ok(());
            }
            
            let (space_id, tag_name, field_name) = key.clone();
            
            if let Some(engine) = self.coordinator.get_engine(space_id, &tag_name, &field_name) {
                let ids: Vec<&str> = point_ids.iter().map(|s| s.as_str()).collect();
                engine.delete_batch(ids).await
                    .map_err(|e| BufferError::IndexError(e.to_string()))?;
            }
            
            let mut last_commit = self.last_commit.lock().await;
            last_commit.insert(key, Instant::now());
        }
        
        Ok(())
    }
}
```

---

## 6. 故障恢复

### 6.1 向量任务恢复

```rust
// src/sync/recovery.rs (扩展)

impl RecoveryManager {
    pub async fn recover_vector_tasks(&self) -> Result<RecoveryResult, RecoveryError> {
        let failed_tasks = self.persistence.load_failed_tasks().await?;
        
        let mut result = RecoveryResult {
            total: 0,
            recovered: 0,
            skipped: 0,
            failed: 0,
        };
        
        for failed_task in failed_tasks {
            if !failed_task.task.is_vector_task() {
                continue;
            }
            
            result.total += 1;
            
            if failed_task.retry_count >= self.config.max_retry_count {
                result.skipped += 1;
                continue;
            }
            
            match self.buffer.submit(failed_task.task.clone()).await {
                Ok(_) => {
                    self.persistence
                        .remove_failed_task(failed_task.task.task_id())
                        .await?;
                    result.recovered += 1;
                }
                Err(e) => {
                    log::error!(
                        "Failed to recover vector task {}: {}",
                        failed_task.task.task_id(),
                        e
                    );
                    self.persistence
                        .increment_retry_count(failed_task.task.task_id())
                        .await?;
                    result.failed += 1;
                }
            }
        }
        
        Ok(result)
    }
}
```

### 6.2 持久化格式

```json
// failed_tasks.json
{
  "tasks": [
    {
      "task": {
        "type": "VectorChange",
        "task_id": "uuid-xxx",
        "space_id": 1,
        "tag_name": "Document",
        "field_name": "embedding",
        "vertex_id": "doc1",
        "vector": [0.1, 0.2, ...],
        "payload": {"title": "Test"},
        "change_type": "Insert",
        "created_at": "2026-04-06T10:00:00Z"
      },
      "error": "Connection refused",
      "retry_count": 2,
      "failed_at": "2026-04-06T10:01:00Z"
    }
  ]
}
```

---

## 7. 一致性保证

### 7.1 事务集成

```rust
// src/transaction/manager.rs (扩展)

impl TransactionManager {
    pub async fn commit(&self, txn_id: u64) -> Result<(), TransactionError> {
        let txn = self.get_transaction(txn_id)?;
        
        // 1. 提交图数据变更
        self.storage.commit_transaction(txn_id).await?;
        
        // 2. 触发同步（异步，不阻塞）
        if let Some(sync_manager) = &self.sync_manager {
            for change in txn.changes() {
                match change {
                    Change::VertexInsert { space_id, vertex } => {
                        // 全文同步
                        sync_manager.on_vertex_change(
                            *space_id,
                            &vertex.tags[0].name,
                            &vertex.vid,
                            &vertex.tags[0].properties.iter()
                                .map(|(k, v)| (k.clone(), v.clone()))
                                .collect::<Vec<_>>(),
                            ChangeType::Insert,
                        ).await?;
                        
                        // 向量同步
                        for (field_name, value) in &vertex.tags[0].properties {
                            if let Some(vector) = value.as_vector() {
                                let mut payload = HashMap::new();
                                payload.insert("vertex_id".to_string(), vertex.vid.clone());
                                
                                sync_manager.on_vector_change(
                                    *space_id,
                                    &vertex.tags[0].name,
                                    field_name,
                                    &vertex.vid,
                                    Some(vector),
                                    payload,
                                    VectorChangeType::Insert,
                                ).await?;
                            }
                        }
                    }
                    Change::VertexDelete { space_id, tag_name, vertex_id } => {
                        sync_manager.on_vertex_change(
                            *space_id,
                            tag_name,
                            vertex_id,
                            &[],
                            ChangeType::Delete,
                        ).await?;
                    }
                    Change::VertexUpdate { space_id, vertex, changed_fields } => {
                        // 类似处理...
                    }
                }
            }
        }
        
        Ok(())
    }
}
```

### 7.2 最终一致性保证

```
┌─────────────────────────────────────────────────────────────┐
│                    数据变更流程                              │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│  1. 事务提交图数据变更                                       │
│     - 原子性保证                                            │
│     - 持久化到磁盘                                          │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│  2. 创建同步任务                                            │
│     - 包含变更的完整信息                                    │
│     - 持久化到失败任务队列                                  │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│  3. 提交到同步队列                                          │
│     - Async 模式：非阻塞                                    │
│     - Sync 模式：阻塞等待                                   │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│  4. 后台处理                                                │
│     - 批量聚合                                              │
│     - 定时提交                                              │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│  5. 更新向量索引                                            │
│     - Qdrant upsert/delete                                 │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│  6. 清理失败任务记录                                        │
│     - 成功后删除持久化记录                                  │
└─────────────────────────────────────────────────────────────┘
```

### 7.3 一致性检查

```rust
impl VectorCoordinator {
    /// 检查向量索引与图数据的一致性
    pub async fn check_consistency(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
    ) -> Result<ConsistencyReport, CoordinatorError> {
        let engine = self.manager.get_engine(space_id, tag_name, field_name)
            .ok_or(CoordinatorError::IndexNotFound)?;
        
        // 获取向量索引中的所有点
        let indexed_count = engine.count().await?;
        
        // 获取图数据中的向量数量
        let graph_count = self.storage.count_vertices_with_field(
            space_id, tag_name, field_name
        ).await?;
        
        let mut report = ConsistencyReport {
            indexed_count,
            graph_count,
            missing_in_index: vec![],
            extra_in_index: vec![],
        };
        
        if indexed_count != graph_count {
            // 找出缺失和多余的点
            // TODO: 实现详细的一致性检查
        }
        
        Ok(report)
    }
    
    /// 修复不一致
    pub async fn repair_consistency(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
    ) -> Result<RepairReport, CoordinatorError> {
        let report = self.check_consistency(space_id, tag_name, field_name).await?;
        
        let mut repair = RepairReport {
            added: 0,
            removed: 0,
            failed: 0,
        };
        
        // 添加缺失的点
        for vertex_id in &report.missing_in_index {
            if let Some(vertex) = self.storage.get_vertex(space_id, vertex_id).await? {
                if let Some(vector) = vertex.get_property(field_name).and_then(|v| v.as_vector()) {
                    let engine = self.manager.get_engine(space_id, tag_name, field_name).unwrap();
                    if engine.upsert(&vertex_id.to_string(), vector, None).await.is_ok() {
                        repair.added += 1;
                    } else {
                        repair.failed += 1;
                    }
                }
            }
        }
        
        // 删除多余的点
        for point_id in &report.extra_in_index {
            let engine = self.manager.get_engine(space_id, tag_name, field_name).unwrap();
            if engine.delete(point_id).await.is_ok() {
                repair.removed += 1;
            } else {
                repair.failed += 1;
            }
        }
        
        Ok(repair)
    }
}

#[derive(Debug)]
pub struct ConsistencyReport {
    pub indexed_count: u64,
    pub graph_count: u64,
    pub missing_in_index: Vec<String>,
    pub extra_in_index: Vec<String>,
}

#[derive(Debug)]
pub struct RepairReport {
    pub added: usize,
    pub removed: usize,
    pub failed: usize,
}
```

---

## 附录: 配置示例

```toml
# config.toml

[vector]
enabled = true
default_engine = "qdrant"

[vector.qdrant]
url = "http://localhost:6334"
timeout_ms = 30000

[vector.sync]
mode = "async"
queue_size = 10000
batch_size = 100
commit_interval_ms = 1000
max_retry_count = 3
retry_delay_ms = 60000
```
