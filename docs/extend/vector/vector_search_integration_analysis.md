# 向量检索集成分析

> 分析日期: 2026-04-06
> 分析范围: 基于现有sync模块和全文检索模块，设计向量检索集成方案

---

## 目录

- [1. 概述](#1-概述)
- [2. 现有架构分析](#2-现有架构分析)
- [3. 向量检索架构设计](#3-向量检索架构设计)
- [4. Qdrant集成方案](#4-qdrant集成方案)
- [5. 实现计划](#5-实现计划)
- [6. 数据同步机制](#6-数据同步机制)
- [7. 查询集成](#7-查询集成)
- [8. 配置管理](#8-配置管理)
- [9. 测试策略](#9-测试策略)

---

## 1. 概述

### 1.1 目标

为GraphDB添加向量检索能力，支持语义搜索、相似度搜索等高级检索功能。通过复用Qdrant作为向量存储引擎，实现高效的向量相似度搜索。

### 1.2 核心价值

| 场景 | 描述 |
|------|------|
| 语义搜索 | 基于向量相似度的语义搜索，超越关键词匹配 |
| 推荐系统 | 基于向量相似度的内容推荐 |
| RAG应用 | 支持检索增强生成应用场景 |
| 多模态搜索 | 支持文本、图像等多模态向量检索 |

### 1.3 设计原则

1. **复用现有架构**: 遵循全文检索模块的设计模式
2. **松耦合设计**: 向量检索模块独立于图存储引擎
3. **可扩展性**: 支持多种向量引擎（当前聚焦Qdrant）
4. **一致性保证**: 通过同步机制保证图数据与向量索引的一致性

---

## 2. 现有架构分析

### 2.1 全文检索架构

```
┌─────────────────────────────────────────────────────────────┐
│                     Query Layer                              │
│  Parser → Validator → Planner → Executor                    │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│                   Coordinator Layer                          │
│  FulltextCoordinator - 索引管理和数据同步协调               │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│                   Search Engine Layer                        │
│  FulltextIndexManager + BM25/Inversearch 引擎适配器         │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│                     Sync Layer                               │
│  SyncManager + TaskBuffer + RecoveryManager                 │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 核心组件对照

| 全文检索组件 | 职责 | 向量检索对应组件 |
|-------------|------|-----------------|
| `SearchEngine` Trait | 搜索引擎抽象接口 | `VectorEngine` Trait |
| `FulltextIndexManager` | 索引生命周期管理 | `VectorIndexManager` |
| `FulltextCoordinator` | 数据变更协调 | `VectorCoordinator` |
| `SyncManager` | 异步同步管理 | 复用（扩展支持向量） |
| `TaskBuffer` | 批量任务缓冲 | 复用 |
| `RecoveryManager` | 失败任务恢复 | 复用 |

### 2.3 Sync模块关键特性

```rust
pub enum SyncMode {
    Sync,   // 同步模式：阻塞等待索引完成
    Async,  // 异步模式：提交到队列立即返回
    Off,    // 关闭模式：不更新索引
}

pub enum SyncTask {
    VertexChange { ... },
    BatchIndex { ... },
    CommitIndex { ... },
    RebuildIndex { ... },
    BatchDelete { ... },
}
```

**关键设计点**:
- 支持三种同步模式，满足不同一致性需求
- 批量处理缓冲区，提升吞吐量
- 失败任务持久化和自动重试

---

## 3. 向量检索架构设计

### 3.1 整体架构

```
┌─────────────────────────────────────────────────────────────┐
│                     Query Layer                              │
│  Vector Parser → Vector Validator → Vector Planner          │
│  Vector Search Executor                                      │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│                   Coordinator Layer                          │
│  VectorCoordinator - 向量索引管理和数据同步协调             │
│  + EmbeddingService - 向量化服务（可选）                    │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│                   Vector Engine Layer                        │
│  VectorIndexManager + QdrantAdapter                         │
│  (未来可扩展: MilvusAdapter, WeaviateAdapter)               │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│                     Sync Layer (复用)                        │
│  SyncManager + TaskBuffer + RecoveryManager                 │
│  扩展支持 VectorSyncTask                                    │
└─────────────────────────────────────────────────────────────┘
```

### 3.2 核心接口设计

#### 3.2.1 VectorEngine Trait

```rust
use async_trait::async_trait;

#[async_trait]
pub trait VectorEngine: Send + Sync + std::fmt::Debug {
    fn name(&self) -> &str;
    
    fn version(&self) -> &str;
    
    async fn upsert(
        &self,
        point_id: &str,
        vector: Vec<f32>,
        payload: Option<HashMap<String, Value>>,
    ) -> Result<(), VectorError>;
    
    async fn upsert_batch(
        &self,
        points: Vec<VectorPoint>,
    ) -> Result<(), VectorError>;
    
    async fn search(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
        filter: Option<VectorFilter>,
    ) -> Result<Vec<VectorSearchResult>, VectorError>;
    
    async fn delete(&self, point_id: &str) -> Result<(), VectorError>;
    
    async fn delete_batch(&self, point_ids: Vec<&str>) -> Result<(), VectorError>;
    
    async fn get(&self, point_id: &str) -> Result<Option<VectorPoint>, VectorError>;
    
    async fn count(&self) -> Result<u64, VectorError>;
    
    async fn create_collection(
        &self,
        collection_name: &str,
        config: CollectionConfig,
    ) -> Result<(), VectorError>;
    
    async fn delete_collection(&self, collection_name: &str) -> Result<(), VectorError>;
    
    async fn collection_exists(&self, collection_name: &str) -> Result<bool, VectorError>;
}

#[derive(Debug, Clone)]
pub struct VectorPoint {
    pub id: String,
    pub vector: Vec<f32>,
    pub payload: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
pub struct VectorSearchResult {
    pub id: String,
    pub score: f32,
    pub payload: HashMap<String, Value>,
    pub vector: Option<Vec<f32>>,
}

#[derive(Debug, Clone)]
pub struct CollectionConfig {
    pub vector_size: usize,
    pub distance: DistanceMetric,
    pub hnsw_config: Option<HnswConfig>,
}

#[derive(Debug, Clone, Copy)]
pub enum DistanceMetric {
    Cosine,
    Euclidean,
    Dot,
}
```

#### 3.2.2 VectorIndexManager

```rust
use dashmap::DashMap;
use std::sync::Arc;

#[derive(Debug)]
pub struct VectorIndexManager {
    engines: DashMap<IndexKey, Arc<dyn VectorEngine>>,
    metadata: DashMap<IndexKey, VectorIndexMetadata>,
    config: VectorConfig,
}

impl VectorIndexManager {
    pub fn new(config: VectorConfig) -> Result<Self, VectorError>;
    
    pub async fn create_index(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        vector_size: usize,
        distance: DistanceMetric,
    ) -> Result<String, VectorError>;
    
    pub fn get_engine(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
    ) -> Option<Arc<dyn VectorEngine>>;
    
    pub async fn drop_index(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
    ) -> Result<(), VectorError>;
    
    pub async fn search(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        query_vector: Vec<f32>,
        limit: usize,
        filter: Option<VectorFilter>,
    ) -> Result<Vec<VectorSearchResult>, VectorError>;
    
    pub fn list_indexes(&self) -> Vec<VectorIndexMetadata>;
}
```

#### 3.2.3 VectorCoordinator

```rust
#[derive(Debug)]
pub struct VectorCoordinator {
    manager: Arc<VectorIndexManager>,
    embedding_service: Option<Arc<dyn EmbeddingService>>,
}

impl VectorCoordinator {
    pub fn new(manager: Arc<VectorIndexManager>) -> Self;
    
    pub fn with_embedding(
        manager: Arc<VectorIndexManager>,
        embedding_service: Arc<dyn EmbeddingService>,
    ) -> Self;
    
    pub async fn create_vector_index(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        vector_size: usize,
        distance: DistanceMetric,
    ) -> CoordinatorResult<String>;
    
    pub async fn on_vertex_inserted(
        &self,
        space_id: u64,
        vertex: &Vertex,
    ) -> CoordinatorResult<()>;
    
    pub async fn on_vertex_updated(
        &self,
        space_id: u64,
        vertex: &Vertex,
        changed_fields: &[String],
    ) -> CoordinatorResult<()>;
    
    pub async fn on_vertex_deleted(
        &self,
        space_id: u64,
        tag_name: &str,
        vertex_id: &Value,
    ) -> CoordinatorResult<()>;
    
    pub async fn on_vector_change(
        &self,
        space_id: u64,
        tag_name: &str,
        vertex_id: &Value,
        field_name: &str,
        vector: Option<Vec<f32>>,
        change_type: VectorChangeType,
    ) -> CoordinatorResult<()>;
    
    pub async fn search(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        query: VectorQuery,
        limit: usize,
    ) -> CoordinatorResult<Vec<VectorSearchResult>>;
    
    pub async fn search_by_text(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        text: &str,
        limit: usize,
    ) -> CoordinatorResult<Vec<VectorSearchResult>>;
}

#[derive(Debug, Clone, Copy)]
pub enum VectorChangeType {
    Insert,
    Update,
    Delete,
}
```

---

## 4. Qdrant集成方案

### 4.1 Qdrant客户端配置

```rust
use qdrant_client::Qdrant;
use qdrant_client::config::CompressionEncoding;
use std::time::Duration;

pub struct QdrantConfig {
    pub url: String,
    pub api_key: Option<String>,
    pub timeout: Duration,
    pub connect_timeout: Duration,
    pub compression: Option<CompressionEncoding>,
}

impl Default for QdrantConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:6334".to_string(),
            api_key: None,
            timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
            compression: None,
        }
    }
}

impl QdrantConfig {
    pub fn build_client(&self) -> Result<Qdrant, VectorError> {
        let mut builder = Qdrant::from_url(&self.url)
            .timeout(self.timeout)
            .connect_timeout(self.connect_timeout);
        
        if let Some(ref api_key) = self.api_key {
            builder = builder.api_key(Some(api_key.clone()));
        }
        
        if let Some(compression) = self.compression {
            builder = builder.compression(Some(compression));
        }
        
        builder.build().map_err(|e| VectorError::Connection(e.to_string()))
    }
}
```

### 4.2 QdrantAdapter实现

```rust
use qdrant_client::Qdrant;
use qdrant_client::qdrant::{
    CreateCollectionBuilder,
    DeleteCollectionBuilder,
    Distance,
    Filter,
    PointStruct,
    SearchPointsBuilder,
    UpsertPointsBuilder,
    VectorParamsBuilder,
    ScalarQuantizationBuilder,
    HnswConfigDiffBuilder,
    Condition,
};
use qdrant_client::Payload;

pub struct QdrantAdapter {
    client: Qdrant,
    default_vector_size: usize,
    default_distance: Distance,
}

impl QdrantAdapter {
    pub fn new(config: QdrantConfig) -> Result<Self, VectorError> {
        let client = config.build_client()?;
        Ok(Self {
            client,
            default_vector_size: 768,
            default_distance: Distance::Cosine,
        })
    }
    
    pub fn with_dimensions(mut self, size: usize) -> Self {
        self.default_vector_size = size;
        self
    }
    
    pub fn with_distance(mut self, distance: DistanceMetric) -> Self {
        self.default_distance = match distance {
            DistanceMetric::Cosine => Distance::Cosine,
            DistanceMetric::Euclidean => Distance::Euclid,
            DistanceMetric::Dot => Distance::Dot,
        };
        self
    }
    
    fn collection_name(space_id: u64, tag_name: &str, field_name: &str) -> String {
        format!("space_{}_{}_{}", space_id, tag_name, field_name)
    }
}

#[async_trait]
impl VectorEngine for QdrantAdapter {
    fn name(&self) -> &str {
        "qdrant"
    }
    
    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }
    
    async fn upsert(
        &self,
        point_id: &str,
        vector: Vec<f32>,
        payload: Option<HashMap<String, Value>>,
    ) -> Result<(), VectorError> {
        let qdrant_payload = payload
            .map(|p| Payload::try_from(serde_json::to_value(p).unwrap()))
            .transpose()?
            .unwrap_or_default();
        
        let point = PointStruct::new(
            point_id,
            vector,
            qdrant_payload,
        );
        
        self.client
            .upsert_points(
                UpsertPointsBuilder::new(self.current_collection.as_str(), vec![point])
                    .wait(true)
            )
            .await?;
        
        Ok(())
    }
    
    async fn upsert_batch(
        &self,
        points: Vec<VectorPoint>,
    ) -> Result<(), VectorError> {
        let qdrant_points: Vec<PointStruct> = points
            .into_iter()
            .map(|p| {
                let payload = Payload::try_from(
                    serde_json::to_value(p.payload).unwrap()
                ).unwrap();
                PointStruct::new(p.id, p.vector, payload)
            })
            .collect();
        
        self.client
            .upsert_points_chunked(
                UpsertPointsBuilder::new(
                    self.current_collection.as_str(),
                    qdrant_points
                ).wait(true),
                100,
            )
            .await?;
        
        Ok(())
    }
    
    async fn search(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
        filter: Option<VectorFilter>,
    ) -> Result<Vec<VectorSearchResult>, VectorError> {
        let mut builder = SearchPointsBuilder::new(
            self.current_collection.as_str(),
            query_vector,
            limit as u64,
        )
        .with_payload(true);
        
        if let Some(f) = filter {
            builder = builder.filter(convert_filter(f));
        }
        
        let result = self.client.search_points(builder).await?;
        
        Ok(result.result.into_iter().map(|r| {
            VectorSearchResult {
                id: r.id.unwrap().to_string(),
                score: r.score,
                payload: r.payload.into_iter()
                    .map(|(k, v)| (k, convert_value(v)))
                    .collect(),
                vector: r.vectors.map(|v| v.into()),
            }
        }).collect())
    }
    
    async fn delete(&self, point_id: &str) -> Result<(), VectorError> {
        self.client
            .delete_points(
                DeletePointsBuilder::new(
                    self.current_collection.as_str(),
                    PointsIdsList {
                        ids: vec![PointId::from(point_id)],
                        ..Default::default()
                    },
                ).wait(true)
            )
            .await?;
        Ok(())
    }
    
    async fn create_collection(
        &self,
        collection_name: &str,
        config: CollectionConfig,
    ) -> Result<(), VectorError> {
        let distance = match config.distance {
            DistanceMetric::Cosine => Distance::Cosine,
            DistanceMetric::Euclidean => Distance::Euclid,
            DistanceMetric::Dot => Distance::Dot,
        };
        
        let mut builder = CreateCollectionBuilder::new(collection_name)
            .vectors_config(VectorParamsBuilder::new(
                config.vector_size as u64,
                distance,
            ));
        
        if let Some(hnsw) = config.hnsw_config {
            builder = builder.hnsw_config(
                HnswConfigDiffBuilder::default()
                    .m(hnsw.m as u64)
                    .ef_construct(hnsw.ef_construct as u64)
            );
        }
        
        self.client.create_collection(builder).await?;
        Ok(())
    }
    
    // ... 其他方法实现
}
```

### 4.3 集合命名规范

| 组件 | 命名格式 | 示例 |
|------|---------|------|
| Qdrant Collection | `space_{space_id}_{tag}_{field}` | `space_1_Post_content` |
| Index ID | `{space_id}_{tag}_{field}` | `1_Post_content` |

---

## 5. 实现计划

### 5.1 文件结构

```
src/
├── vector/
│   ├── mod.rs              # 模块入口
│   ├── engine.rs           # VectorEngine Trait
│   ├── manager.rs          # VectorIndexManager
│   ├── coordinator.rs      # VectorCoordinator
│   ├── config.rs           # 配置定义
│   ├── error.rs            # 错误类型
│   ├── result.rs           # 结果类型
│   ├── metadata.rs         # 元数据定义
│   ├── filter.rs           # 过滤器定义
│   └── adapters/
│       ├── mod.rs
│       └── qdrant_adapter.rs   # Qdrant适配器
├── coordinator/
│   ├── mod.rs
│   ├── fulltext.rs         # 现有全文协调器
│   ├── vector.rs           # 新增向量协调器
│   └── types.rs            # 共享类型
└── sync/
    ├── mod.rs
    ├── manager.rs          # 扩展支持向量同步
    ├── task.rs             # 扩展 VectorSyncTask
    └── ...
```

### 5.2 阶段规划

#### Phase 1: 核心接口和Qdrant适配器 (3-4天)

| 任务 | 预计时间 | 优先级 |
|------|---------|--------|
| 定义 VectorEngine Trait | 2h | P0 |
| 实现 VectorError, VectorResult | 1h | P0 |
| 实现 VectorPoint, VectorSearchResult | 1h | P0 |
| 实现 QdrantAdapter 核心方法 | 4h | P0 |
| 实现 QdrantAdapter 搜索方法 | 2h | P0 |
| 单元测试 | 2h | P0 |

#### Phase 2: 索引管理器 (2-3天)

| 任务 | 预计时间 | 优先级 |
|------|---------|--------|
| 实现 VectorIndexManager | 3h | P0 |
| 实现 VectorIndexMetadata | 1h | P0 |
| 实现索引生命周期管理 | 2h | P0 |
| 单元测试 | 2h | P0 |

#### Phase 3: 协调器和同步扩展 (2-3天)

| 任务 | 预计时间 | 优先级 |
|------|---------|--------|
| 实现 VectorCoordinator | 3h | P0 |
| 扩展 SyncTask 支持 VectorSyncTask | 2h | P0 |
| 扩展 SyncManager 支持向量同步 | 2h | P0 |
| 集成测试 | 2h | P0 |

#### Phase 4: 查询引擎集成 (3-4天)

| 任务 | 预计时间 | 优先级 |
|------|---------|--------|
| 扩展 AST 支持向量查询语法 | 3h | P0 |
| 实现 VectorSearchExecutor | 4h | P0 |
| 实现 VectorScanExecutor | 3h | P1 |
| 集成测试 | 2h | P0 |

#### Phase 5: 嵌入服务集成 (可选, 2-3天)

| 任务 | 预计时间 | 优先级 |
|------|---------|--------|
| 定义 EmbeddingService Trait | 1h | P2 |
| 实现 OpenAI Embedding 适配器 | 2h | P2 |
| 实现本地嵌入模型适配器 | 3h | P2 |
| 集成测试 | 2h | P2 |

---

## 6. 数据同步机制

### 6.1 同步任务扩展

```rust
pub enum SyncTask {
    // 现有任务类型
    VertexChange { ... },
    BatchIndex { ... },
    CommitIndex { ... },
    RebuildIndex { ... },
    BatchDelete { ... },
    
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
}
```

### 6.2 同步流程

```
┌─────────────────────────────────────────────────────────────┐
│                    数据变更触发                              │
│  INSERT/UPDATE/DELETE Vertex                               │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│                    判断向量字段                              │
│  检查字段是否有向量索引                                     │
│  检查字段值是否为向量类型                                   │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│                    创建同步任务                              │
│  VectorChange / VectorBatchUpsert / VectorBatchDelete      │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│                    提交到同步队列                            │
│  SyncMode::Sync → 直接执行                                  │
│  SyncMode::Async → 提交到 TaskBuffer                        │
│  SyncMode::Off → 跳过                                       │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│                    执行向量同步                              │
│  VectorCoordinator.on_vector_change()                       │
│  QdrantAdapter.upsert() / delete()                         │
└─────────────────────────────────────────────────────────────┘
```

### 6.3 向量字段类型

```rust
pub enum VectorValue {
    Float32(Vec<f32>),
    Float64(Vec<f64>),
    Base64(String),  // Base64编码的向量
}

impl Value {
    pub fn as_vector(&self) -> Option<Vec<f32>> {
        match self {
            Value::List(list) => {
                list.iter()
                    .map(|v| match v {
                        Value::Float(f) => Some(*f as f32),
                        Value::Double(d) => Some(*d as f32),
                        Value::Int(i) => Some(*i as f32),
                        _ => None,
                    })
                    .collect()
            }
            Value::String(s) => {
                // 尝试解析 Base64 编码的向量
                base64_decode_vector(s).ok()
            }
            _ => None,
        }
    }
}
```

---

## 7. 查询集成

### 7.1 SQL语法扩展

```sql
-- 创建向量索引
CREATE VECTOR INDEX idx_embedding 
ON Document(embedding) 
WITH (
    vector_size = 768,
    distance = 'cosine',
    engine = 'qdrant'
);

-- 向量相似度搜索
SEARCH VECTOR idx_embedding
WITH vector = [0.1, 0.2, ...]
LIMIT 10
RETURN id, content, score;

-- 文本到向量搜索（需要嵌入服务）
SEARCH VECTOR idx_embedding
WITH text = 'search query'
LIMIT 10
RETURN id, content, score;

-- 带过滤的向量搜索
SEARCH VECTOR idx_embedding
WITH vector = [0.1, 0.2, ...]
WHERE category = 'tech' AND year > 2020
LIMIT 10
RETURN id, content, score;

-- 与图查询结合
MATCH (d:Document)
WHERE d.embedding SIMILAR TO [0.1, 0.2, ...] WITH threshold = 0.8
RETURN d
LIMIT 10;
```

### 7.2 AST扩展

```rust
pub enum Statement {
    // 现有语句类型
    // ...
    
    // 向量语句
    CreateVectorIndex(CreateVectorIndexStatement),
    DropVectorIndex(DropVectorIndexStatement),
    SearchVector(SearchVectorStatement),
}

pub struct CreateVectorIndexStatement {
    pub index_name: String,
    pub tag_name: String,
    pub field_name: String,
    pub vector_size: usize,
    pub distance: DistanceMetric,
    pub engine: Option<String>,
    pub hnsw_config: Option<HnswConfig>,
}

pub struct SearchVectorStatement {
    pub index_name: String,
    pub query: VectorQuery,
    pub limit: usize,
    pub filter: Option<Expression>,
    pub yield_clause: Option<YieldClause>,
}

pub enum VectorQuery {
    Vector(Vec<f32>),
    Text(String),
    Parameter(String),
}
```

### 7.3 执行器实现

```rust
pub struct VectorSearchExecutor {
    coordinator: Arc<VectorCoordinator>,
    storage: Arc<dyn StorageClient>,
}

impl VectorSearchExecutor {
    pub async fn execute(&self, plan: &VectorSearchPlan) -> Result<ExecutionResult, ExecutorError> {
        // 1. 解析索引名称，获取 space_id, tag_name, field_name
        let (space_id, tag_name, field_name) = parse_index_name(&plan.index_name)?;
        
        // 2. 获取查询向量
        let query_vector = match &plan.query {
            VectorQuery::Vector(v) => v.clone(),
            VectorQuery::Text(t) => {
                self.coordinator
                    .embed_text(&t)
                    .await?
            }
            VectorQuery::Parameter(p) => {
                plan.context.get_parameter(p)?
            }
        };
        
        // 3. 构建过滤器
        let filter = plan.filter.as_ref()
            .map(|f| self.build_filter(f))
            .transpose()?;
        
        // 4. 执行向量搜索
        let results = self.coordinator
            .search(space_id, &tag_name, &field_name, query_vector, plan.limit, filter)
            .await?;
        
        // 5. 根据 point_ids 获取完整顶点数据
        let vertex_ids: Vec<Value> = results.iter()
            .map(|r| parse_vertex_id(&r.id))
            .collect::<Result<Vec<_>, _>>()?;
        
        let vertices = self.storage.get_vertices(space_id, &vertex_ids).await?;
        
        // 6. 构建结果
        let rows = self.build_result_rows(&results, &vertices, &plan.yield_clause)?;
        
        Ok(ExecutionResult::Rows(rows))
    }
}
```

---

## 8. 配置管理

### 8.1 配置结构

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorConfig {
    pub enabled: bool,
    pub default_engine: VectorEngineType,
    pub qdrant: QdrantConfig,
    pub sync: VectorSyncConfig,
    pub embedding: Option<EmbeddingConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantConfig {
    pub url: String,
    pub api_key: Option<String>,
    pub timeout_ms: u64,
    pub connect_timeout_ms: u64,
    pub default_vector_size: usize,
    pub default_distance: String,
    pub compression: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorSyncConfig {
    pub mode: SyncMode,
    pub queue_size: usize,
    pub batch_size: usize,
    pub commit_interval_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    pub provider: EmbeddingProvider,
    pub model: String,
    pub api_key: Option<String>,
    pub batch_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VectorEngineType {
    Qdrant,
    // 未来可扩展
    // Milvus,
    // Weaviate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmbeddingProvider {
    OpenAI,
    Local,
    Custom(String),
}
```

### 8.2 配置文件示例

```toml
# config.toml

[vector]
enabled = true
default_engine = "qdrant"

[vector.qdrant]
url = "http://localhost:6334"
api_key = "${QDRANT_API_KEY}"
timeout_ms = 30000
connect_timeout_ms = 10000
default_vector_size = 768
default_distance = "cosine"

[vector.sync]
mode = "async"
queue_size = 10000
batch_size = 100
commit_interval_ms = 1000

[vector.embedding]
provider = "openai"
model = "text-embedding-3-small"
api_key = "${OPENAI_API_KEY}"
batch_size = 100
```

---

## 9. 测试策略

### 9.1 单元测试

| 模块 | 测试重点 |
|------|---------|
| VectorEngine Trait | 接口契约验证 |
| QdrantAdapter | CRUD操作、搜索功能 |
| VectorIndexManager | 索引生命周期管理 |
| VectorCoordinator | 数据变更同步 |
| SyncTask | 向量任务序列化/反序列化 |

### 9.2 集成测试

```rust
#[tokio::test]
async fn test_vector_search_workflow() {
    // 1. 创建向量索引
    coordinator.create_vector_index(
        space_id, "Document", "embedding",
        768, DistanceMetric::Cosine
    ).await.unwrap();
    
    // 2. 插入带向量的顶点
    let vertex = Vertex {
        vid: Value::String("doc1".to_string()),
        tags: vec![Tag {
            name: "Document".to_string(),
            properties: vec![
                ("content".to_string(), Value::String("test content".to_string())),
                ("embedding".to_string(), Value::List(
                    (0..768).map(|i| Value::Float(i as f32 / 768.0)).collect()
                )),
            ].into_iter().collect(),
        }],
    };
    
    storage.insert_vertex(space_id, vertex.clone()).await.unwrap();
    
    // 3. 等待同步
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // 4. 执行向量搜索
    let query_vector = (0..768).map(|i| i as f32 / 768.0).collect();
    let results = coordinator.search(
        space_id, "Document", "embedding",
        query_vector, 10, None
    ).await.unwrap();
    
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "doc1");
    
    // 5. 删除顶点
    storage.delete_vertex(space_id, &vertex.vid).await.unwrap();
    
    // 6. 验证向量被删除
    tokio::time::sleep(Duration::from_millis(100)).await;
    let results = coordinator.search(
        space_id, "Document", "embedding",
        query_vector, 10, None
    ).await.unwrap();
    
    assert_eq!(results.len(), 0);
}
```

### 9.3 性能测试

| 测试场景 | 指标 |
|---------|------|
| 批量插入 | 向量/秒 |
| 向量搜索延迟 | P50, P99 延迟 |
| 并发搜索 | QPS |
| 内存使用 | 峰值内存 |

---

## 附录 A: 依赖项

```toml
# Cargo.toml

[dependencies]
qdrant-client = "1.7"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
async-trait = "0.1"
thiserror = "1"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4"] }
dashmap = "5"
```

## 附录 B: 与全文检索的对比

| 特性 | 全文检索 | 向量检索 |
|------|---------|---------|
| 搜索方式 | 关键词匹配 | 向量相似度 |
| 索引类型 | 倒排索引 | HNSW/IVF |
| 查询类型 | 文本查询 | 向量查询 |
| 距离度量 | BM25评分 | Cosine/Euclidean/Dot |
| 适用场景 | 精确匹配、关键词搜索 | 语义搜索、相似度搜索 |
| 存储引擎 | Tantivy/Inversearch | Qdrant |
| 同步机制 | 相同（复用SyncManager） | 相同（复用SyncManager） |

## 附录 C: 混合检索场景

向量检索和全文检索可以组合使用，实现混合检索：

```sql
-- 混合检索示例
MATCH (d:Document)
WHERE d.content MATCH 'graph database'  -- 全文检索
  AND d.embedding SIMILAR TO [0.1, ...] WITH threshold = 0.8  -- 向量检索
RETURN d
ORDER BY d.score_fulltext * 0.3 + d.score_vector * 0.7 DESC
LIMIT 10;
```

这种混合检索可以结合关键词匹配的精确性和向量检索的语义理解能力，提供更准确的搜索结果。
