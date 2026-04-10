# 向量搜索架构重构分析

## 目录

1. [当前架构分析](#当前架构分析)
2. [核心问题](#核心问题)
3. [Embedding 服务参考设计](#embedding 服务参考设计)
4. [协调逻辑归属分析](#协调逻辑归属分析)
5. [重构方案](#重构方案)
6. [实施步骤](#实施步骤)

---

## 当前架构分析

### 目录结构

```
src/vector/                      # 当前业务层向量模块
├── config.rs                    # 配置类型（与 vector-client 重复）
├── coordinator.rs               # 协调器（包装 VectorIndexManager）
├── embedding.rs                 # Embedding Service（应移至 crates）
├── manager.rs                   # 索引管理器（包装 VectorEngine）
└── mod.rs

crates/vector-client/            # 底层向量客户端
├── src/
│   ├── api/                     # 高层 API
│   ├── config/                  # 配置类型
│   ├── engine/                  # 引擎实现
│   ├── types/                   # 类型定义
│   └── lib.rs

src/sync/                        # 同步协调模块
├── batch.rs                     # 批量处理
├── manager.rs                   # 同步管理器（包含向量同步逻辑）
├── task.rs                      # 任务定义（包含向量任务）
└── ...

ref/embedding/                   # Embedding 参考实现
├── base.rs                      # EmbeddingProvider trait
├── config.rs                    # EmbedderConfig
├── error.rs                     # 错误类型
├── llama_cpp_provider.rs        # llama.cpp 实现
├── openai_compatible_provider.rs # HTTP API 实现
├── preprocessor.rs              # 文本预处理
└── response.rs                  # 响应解析
```

### 当前调用链

```
业务层 (Query Executor)
    ↓
src/vector/coordinator.rs (VectorCoordinator)
    ↓ (包装)
src/vector/manager.rs (VectorIndexManager)
    ↓ (包装)
crates/vector-client/src/engine/mod.rs (VectorEngine)
    ↓ (实现)
crates/vector-client/src/engine/qdrant/mod.rs (QdrantEngine)
```

### 当前数据流

```
1. 图数据变更 → src/sync/manager.rs
                ↓
         src/vector/coordinator.rs
                ↓
         src/vector/manager.rs
                ↓
         vector_client::VectorEngine
                ↓
         Qdrant 引擎

2. 文本查询 → src/query/executor/data_access/vector_search.rs
              ↓
       src/vector/coordinator.rs
              ↓
       src/vector/embedding.rs (EmbeddingService)
              ↓
       HTTP API / 本地库
```

---

## 核心问题

### 1. 冗余包装层

**问题描述**：

- `VectorCoordinator` 对 `VectorIndexManager` 的包装没有增加太多业务价值
- `VectorIndexManager` 对 `VectorEngine` 的包装只是简单的索引名称映射
- 形成 3 层包装，增加了代码复杂度和维护成本

**代码示例**：

```rust
// src/vector/coordinator.rs - 第 400-415 行
pub async fn search(
    &self,
    space_id: u64,
    tag_name: &str,
    field_name: &str,
    query_vector: Vec<f32>,
    limit: usize,
) -> VectorCoordinatorResult<Vec<SearchResult>> {
    let query = SearchQuery::new(query_vector, limit);

    // 简单转发给 manager
    let results = self
        .manager
        .search(space_id, tag_name, field_name, query)
        .await?;

    Ok(results)
}

// src/vector/manager.rs - 第 243-260 行
pub async fn search(
    &self,
    space_id: u64,
    tag_name: &str,
    field_name: &str,
    query: SearchQuery,
) -> VectorResult<Vec<SearchResult>> {
    let collection_name = self.get_collection_name(...)?;

    // 简单转发给 engine
    let results = self.engine.search(&collection_name, query).await?;
    Ok(results)
}
```

**影响**：

- 每次搜索调用需要跨越 3 层函数
- 错误需要在 3 层之间转换
- 难以定位问题所在层

### 2. 配置类型重复

**问题描述**：

- `src/vector/config.rs` 定义了与 `vector_client` 几乎相同的类型
- 存在大量简单的转换代码（`From` trait 实现）

**代码示例**：

```rust
// src/vector/config.rs - 第 97-108 行
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VectorDistance {
    Cosine,
    Euclid,
    Dot,
}

impl From<VectorDistance> for DistanceMetric {
    fn from(dist: VectorDistance) -> Self {
        match dist {
            DistanceMetric::Cosine => DistanceMetric::Cosine,
            DistanceMetric::Euclid => DistanceMetric::Euclid,
            DistanceMetric::Dot => DistanceMetric::Dot,
        }
    }
}

// crates/vector-client/src/types/mod.rs
pub enum DistanceMetric {
    Cosine,
    Euclid,
    Dot,
}
```

**影响**：

- 修改一个类型需要同步修改另一个
- 容易出现不一致
- 增加代码量但无实际价值

### 3. Embedding Service 设计不合理

**问题描述**：

- `src/vector/embedding.rs` 重新定义了 `EmbeddingService` trait
- 实现是占位符代码（mock 实现），没有实际功能
- `ref/embedding` 目录已有完整的 Embedding 实现，但未被使用

**当前实现**（src/vector/embedding.rs 第 120-145 行）：

```rust
#[async_trait::async_trait]
impl EmbeddingService for QdrantEmbeddingService {
    async fn embed(&self, text: &str) -> VectorResult<Vec<f32>> {
        log::debug!("Embedding text: {}", text);

        // 占位符：返回 mock 向量
        let hash = text.as_bytes().iter().map(|&b| b as u32).sum::<u32>();
        let mut vector = vec![0.0; self.dimension];
        for (i, val) in vector.iter_mut().enumerate().take(self.dimension) {
            *val = ((hash + i as u32) % 1000) as f32 / 1000.0;
        }
        Ok(vector)
    }
    // ...
}
```

**参考实现**（ref/embedding/openai_compatible_provider.rs）：

```rust
pub struct OpenAICompatibleProvider {
    client: reqwest::Client,
    config: EmbedderConfig,
}

#[async_trait]
impl EmbeddingProvider for OpenAICompatibleProvider {
    async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        // 真实的 HTTP API 调用
        let response = self.client
            .post(&self.config.base_url)
            .json(&self.build_request(texts))
            .send()
            .await?;

        self.parse_response(response).await
    }
}
```

**影响**：

- 真正的 Embedding 功能无法使用
- 代码重复（两个 Embedding trait）
- 参考实现被浪费

### 4. 职责边界不清晰

**问题描述**：

- `src/vector` 本应负责业务协调，但做了太多底层工作
- `crates/vector-client` 本应提供完整功能，但缺少 Embedding 和索引管理
- `src/sync` 包含了向量同步逻辑，但依赖 `src/vector` 的协调器

**职责混乱示例**：

```rust
// src/vector/manager.rs 负责索引生命周期管理
pub async fn create_index(...) { ... }
pub async fn drop_index(...) { ... }

// src/sync/manager.rs 又负责向量同步
pub async fn on_vector_change(...) { ... }

// src/vector/coordinator.rs 也负责协调
pub async fn on_vertex_inserted(...) { ... }
```

---

## Embedding 服务参考设计

### ref/embedding 目录分析

**核心组件**：

1. **EmbeddingProvider Trait** (`base.rs`)

   ```rust
   #[async_trait]
   pub trait EmbeddingProvider: Send + Sync {
       async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbeddingError>;
       fn dimension(&self) -> usize;
       fn model_name(&self) -> &str;
       fn provider_type(&self) -> ProviderType;
   }
   ```

2. **配置系统** (`config.rs`)

   ```rust
   pub struct EmbedderConfig {
       pub api_keys: Vec<String>,
       pub base_url: String,
       pub model: String,
       pub preprocessor: PreprocessorConfig,
       pub response_parser: ResponseParserConfig,
   }
   ```

3. **多 Provider 支持**：
   - `OpenAICompatibleProvider` - HTTP API（支持 OpenAI、Gemini、Ollama 等）
   - `LlamaCppProvider` - 本地 llama.cpp 库
   - 可扩展其他本地库（candle、ort 等）

4. **预处理系统** (`preprocessor.rs`)：
   - `NoopPreprocessor` - 无操作
   - `PrefixPreprocessor` - 添加前缀
   - `StellaPreprocessor` - Stella 模型专用
   - `NomicPreprocessor` - Nomic 模型专用
   - `ChainedPreprocessor` - 组合多个预处理器

5. **响应解析** (`response.rs`)：
   - 支持多种 API 格式
   - Token 使用统计
   - 错误处理

### 优势对比

| 特性             | src/vector/embedding.rs | ref/embedding/                  |
| ---------------- | ----------------------- | ------------------------------- |
| 多 Provider 支持 | ❌ 仅 Qdrant            | ✅ OpenAI、Gemini、llama.cpp 等 |
| 真实实现         | ❌ Mock 实现            | ✅ 完整 HTTP/本地实现           |
| 预处理           | ❌ 无                   | ✅ 多种预处理器                 |
| 错误处理         | ⚠️ 简单                 | ✅ 完善的错误类型               |
| 配置灵活性       | ⚠️ 固定                 | ✅ 可配置 URL、模型、密钥       |
| Token 统计       | ❌ 无                   | ✅ 支持                         |
| 可扩展性         | ❌ 困难                 | ✅ 易于添加新 Provider          |

---

## 协调逻辑归属分析

### 当前协调逻辑分布

**1. src/vector/coordinator.rs**：

```rust
// 图数据变更时的向量同步
pub async fn on_vertex_inserted(&self, space_id: u64, vertex: &Vertex) { ... }
pub async fn on_vertex_updated(&self, space_id: u64, vertex: &Vertex, changed_fields: &[String]) { ... }
pub async fn on_vertex_deleted(&self, space_id: u64, tag_name: &str, vertex_id: &Value) { ... }

// 向量变更协调
pub async fn on_vector_change(&self, ctx: VectorChangeContext) { ... }
```

**2. src/sync/manager.rs**：

```rust
// 同步模式管理（Sync/Async/Off）
pub async fn on_vertex_change(...) { ... }
pub async fn on_vector_change_with_context(...) { ... }

// 批量任务提交
pub async fn start(&self) { /* 定时提交任务 */ }
```

**3. src/sync/task.rs**：

```rust
pub enum SyncTask {
    VectorChange { ... },
    VectorBatchUpsert { ... },
    VectorBatchDelete { ... },
    VectorRebuildIndex { ... },
}
```

### 问题分析

**问题 1：协调逻辑分散**

- 图数据变更协调在 `src/vector/coordinator.rs`
- 同步模式管理在 `src/sync/manager.rs`
- 任务定义在 `src/sync/task.rs`
- 导致理解成本高，修改时需要同时改多个文件

**问题 2：职责重叠**

```rust
// src/sync/manager.rs 第 218-253 行
async fn execute_vector_vertex_change_sync(
    &self,
    space_id: u64,
    tag_name: &str,
    vertex_id: &Value,
    properties: &[(String, Value)],
    change_type: ChangeType,
    vector_coord: &Arc<VectorCoordinator>,
) {
    // 这里又调用了 VectorCoordinator
    for (field_name, value) in properties {
        if vector_coord.index_exists(space_id, tag_name, field_name) {
            let ctx = VectorChangeContext::new(...);
            vector_coord.on_vector_change(ctx).await?;
        }
    }
}
```

**问题 3：VectorCoordinator 职责过重**

- 既负责索引管理（调用 VectorIndexManager）
- 又负责 Embedding（调用 EmbeddingService）
- 还负责图数据协调（on*vertex*\* 方法）

### 正确的职责划分

**应该遵循的架构原则**：

```
┌─────────────────────────────────────────┐
│         业务层 (src/)                   │
│  - 图数据与向量搜索的业务协调            │
│  - 同步策略管理（Sync/Async/Off）        │
│  - 查询执行时的向量搜索                  │
└─────────────────────────────────────────┘
                  ↓ 使用
┌─────────────────────────────────────────┐
│      向量客户端层 (crates/vector-client)│
│  - 向量引擎抽象（VectorEngine）          │
│  - 索引生命周期管理（VectorManager）     │
│  - Embedding 服务（EmbeddingService）    │
│  - 配置、类型、错误定义                  │
└─────────────────────────────────────────┘
```

**具体职责划分**：

| 职责                    | 当前归属                  | 应该归属                |
| ----------------------- | ------------------------- | ----------------------- |
| 向量引擎抽象            | crates/vector-client      | ✅ crates/vector-client |
| 索引管理（create/drop） | src/vector/manager.rs     | crates/vector-client    |
| 向量搜索执行            | src/vector/manager.rs     | crates/vector-client    |
| Embedding 服务          | src/vector/embedding.rs   | crates/vector-client    |
| 图数据变更协调          | src/vector/coordinator.rs | src/sync/               |
| 同步模式管理            | src/sync/manager.rs       | ✅ src/sync/            |
| 批量任务处理            | src/sync/                 | ✅ src/sync/            |
| 查询时的向量搜索        | src/query/executor/       | src/query/executor/     |

---

## 重构方案

### 方案概述

**核心思想**：

1. 将底层实现（引擎、索引管理、Embedding）移至 `crates/vector-client`
2. `src/vector` 简化为协调层，仅保留与图数据相关的协调逻辑
3. 将协调逻辑进一步移至 `src/sync/`，统一管理同步任务

### 新架构设计

```
crates/vector-client/
├── src/
│   ├── lib.rs
│   ├── error.rs
│   ├── types/           # 类型定义
│   │   ├── mod.rs
│   │   ├── point.rs
│   │   ├── search.rs
│   │   └── filter.rs
│   ├── config/          # 配置
│   │   ├── mod.rs
│   │   ├── client.rs
│   │   └── collection.rs
│   ├── engine/          # 引擎抽象
│   │   ├── mod.rs       # VectorEngine trait
│   │   ├── qdrant/
│   │   └── mock.rs
│   ├── manager/         # 新增：索引管理器
│   │   ├── mod.rs       # VectorManager
│   │   └── index.rs     # 索引元数据管理
│   └── embedding/       # 新增：Embedding 服务
│       ├── mod.rs       # EmbeddingService trait
│       ├── service.rs   # 服务实现
│       ├── provider/    # Provider 实现
│       │   ├── openai.rs
│       │   ├── llama_cpp.rs
│       │   └── mod.rs
│       └── preprocessor/ # 预处理
│           └── mod.rs

src/vector/
├── mod.rs               # 简单导出
└── coordinator.rs       # 简化：仅协调图数据与向量

src/sync/
├── mod.rs
├── manager.rs           # 同步管理器（包含向量同步）
├── task.rs              # 任务定义
├── batch.rs             # 批量处理
└── vector_sync.rs       # 新增：向量同步协调器

src/query/executor/data_access/
├── vector_search.rs     # 直接使用 vector_client::VectorManager
└── vector_index.rs      # 直接使用 vector_client::VectorManager
```

### 详细设计

#### 1. crates/vector-client 增强

**新增 `manager/mod.rs`**：

```rust
pub struct VectorManager {
    engine: Arc<dyn VectorEngine>,
    indexes: DashMap<String, IndexMetadata>,
}

impl VectorManager {
    pub async fn new(config: VectorClientConfig) -> Result<Self>;

    // 索引管理
    pub async fn create_index(&self, name: &str, config: CollectionConfig) -> Result<()>;
    pub async fn drop_index(&self, name: &str) -> Result<()>;
    pub fn index_exists(&self, name: &str) -> bool;
    pub fn get_index_metadata(&self, name: &str) -> Option<IndexMetadata>;

    // 向量操作（直接暴露，不需要 space_id/tag_name/field_name 包装）
    pub async fn upsert(&self, collection: &str, point: VectorPoint) -> Result<UpsertResult>;
    pub async fn upsert_batch(&self, collection: &str, points: Vec<VectorPoint>) -> Result<UpsertResult>;
    pub async fn search(&self, collection: &str, query: SearchQuery) -> Result<Vec<SearchResult>>;
    pub async fn delete(&self, collection: &str, point_id: &str) -> Result<DeleteResult>;
}
```

**新增 `embedding/mod.rs`**：

```rust
pub trait EmbeddingService: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;
    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;
    fn dimension(&self) -> usize;
}

pub struct EmbeddingServiceImpl {
    provider: Box<dyn EmbeddingProvider>,
}

impl EmbeddingService for EmbeddingServiceImpl {
    // 委托给 Provider
}
```

**整合 ref/embedding**：

- 将 `ref/embedding/base.rs` → `crates/vector-client/src/embedding/provider.rs`
- 将 `ref/embedding/config.rs` → `crates/vector-client/src/embedding/config.rs`
- 将 `ref/embedding/openai_compatible_provider.rs` → `crates/vector-client/src/embedding/providers/openai.rs`
- 将 `ref/embedding/llama_cpp_provider.rs` → `crates/vector-client/src/embedding/providers/llama_cpp.rs`

#### 2. src/vector 简化

**删除文件**：

- ~~`src/vector/config.rs`~~ - 直接使用 `vector_client::config`
- ~~`src/vector/manager.rs`~~ - 使用 `vector_client::VectorManager`
- ~~`src/vector/embedding.rs`~~ - 使用 `vector_client::EmbeddingService`

**保留并简化 `src/vector/coordinator.rs`**：

```rust
pub struct VectorCoordinator {
    vector_manager: Arc<VectorManager>,  // 直接使用 VectorManager
    embedding_service: Option<Arc<dyn EmbeddingService>>,
}

impl VectorCoordinator {
    // 仅保留协调逻辑，删除索引管理、Embedding 实现等
    pub async fn on_vertex_inserted(&self, space_id: u64, vertex: &Vertex) -> Result<()>;
    pub async fn on_vertex_updated(&self, space_id: u64, vertex: &Vertex, changed_fields: &[String]) -> Result<()>;
    pub async fn on_vertex_deleted(&self, space_id: u64, tag_name: &str, vertex_id: &Value) -> Result<()>;

    // 简单的搜索方法，直接委托给 VectorManager
    pub async fn search(&self, collection: &str, query: SearchQuery) -> Result<Vec<SearchResult>> {
        self.vector_manager.search(collection, query).await
    }

    // Embedding 委托
    pub async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        if let Some(embedding) = &self.embedding_service {
            embedding.embed(text).await
        } else {
            Err(EmbeddingServiceNotAvailable)
        }
    }
}
```

**简化 `src/vector/mod.rs`**：

```rust
//! Vector Search Module - Simplified
//!
//! Provides coordination between graph data and vector search.

pub mod coordinator;

pub use coordinator::VectorCoordinator;

// 直接导出 vector-client 的类型
pub use vector_client::{
    VectorManager, EmbeddingService, VectorEngine,
    VectorPoint, SearchQuery, SearchResult, VectorFilter,
};
```

#### 3. src/sync 整合协调逻辑

**新增 `src/sync/vector_sync.rs`**：

```rust
//! Vector Synchronization Coordinator
//!
//! Coordinates vector index updates with graph data changes.

use crate::vector::VectorCoordinator;
use crate::core::{Vertex, Value};
use crate::coordinator::ChangeType;

pub struct VectorSyncCoordinator {
    vector_coordinator: Arc<VectorCoordinator>,
}

impl VectorSyncCoordinator {
    pub fn new(vector_coordinator: Arc<VectorCoordinator>) -> Self {
        Self { vector_coordinator }
    }

    /// 处理顶点插入时的向量同步
    pub async fn on_vertex_inserted(
        &self,
        space_id: u64,
        vertex: &Vertex,
    ) -> Result<(), SyncError> {
        self.vector_coordinator.on_vertex_inserted(space_id, vertex).await
    }

    /// 处理顶点更新时的向量同步
    pub async fn on_vertex_updated(
        &self,
        space_id: u64,
        vertex: &Vertex,
        changed_fields: &[String],
    ) -> Result<(), SyncError> {
        self.vector_coordinator.on_vertex_updated(space_id, vertex, changed_fields).await
    }

    /// 处理顶点删除时的向量同步
    pub async fn on_vertex_deleted(
        &self,
        space_id: u64,
        tag_name: &str,
        vertex_id: &Value,
    ) -> Result<(), SyncError> {
        self.vector_coordinator.on_vertex_deleted(space_id, tag_name, vertex_id).await
    }
}
```

**修改 `src/sync/manager.rs`**：

```rust
pub struct SyncManager {
    fulltext_coordinator: Arc<FulltextCoordinator>,
    vector_sync_coordinator: Option<Arc<VectorSyncCoordinator>>,  // 新增
    buffer: Arc<TaskBuffer>,
    mode: Arc<RwLock<SyncMode>>,
    // ...
}

impl SyncManager {
    pub fn with_vector_sync_coordinator(
        mut self,
        vector_sync_coordinator: Arc<VectorSyncCoordinator>,
    ) -> Self {
        self.vector_sync_coordinator = Some(vector_sync_coordinator);
        self
    }

    pub async fn on_vertex_change(...) {
        // 同步模式处理
        match mode {
            SyncMode::Sync => {
                // 全文索引同步
                self.fulltext_coordinator.on_vertex_change(...).await?;

                // 向量索引同步（委托给 VectorSyncCoordinator）
                if let Some(ref vector_sync) = self.vector_sync_coordinator {
                    vector_sync.on_vertex_inserted(space_id, vertex).await?;
                }
            }
            SyncMode::Async => {
                // 提交到任务缓冲区
                let task = SyncTask::vector_change(...);
                self.buffer.submit(task).await?;
            }
            SyncMode::Off => {}
        }
    }
}
```

**修改 `src/sync/task.rs`**：

- 保留 `SyncTask::VectorChange` 等向量任务
- 这些任务由 `VectorSyncCoordinator` 处理

### 调用链对比

**重构前**：

```
业务层
  ↓
src/vector/coordinator.rs (VectorCoordinator)
  ↓
src/vector/manager.rs (VectorIndexManager)
  ↓
crates/vector-client/src/engine/mod.rs (VectorEngine)
  ↓
QdrantEngine
```

**重构后**：

```
业务层（查询执行）
  ↓
crates/vector-client/src/manager/mod.rs (VectorManager)  [直接使用]
  ↓
VectorEngine → QdrantEngine

业务层（图数据协调）
  ↓
src/sync/manager.rs (SyncManager)
  ↓
src/sync/vector_sync.rs (VectorSyncCoordinator)
  ↓
src/vector/coordinator.rs (VectorCoordinator)
  ↓
crates/vector-client/src/manager/mod.rs (VectorManager)
```

### 优势

1. **减少包装层**：查询场景直接从 3 层减少到 1 层
2. **职责清晰**：
   - `crates/vector-client`：底层实现（引擎、索引管理、Embedding）
   - `src/vector`：业务协调（图数据与向量的桥梁）
   - `src/sync`：同步策略（Sync/Async/Off 管理）
3. **Embedding 可用**：整合 ref/embedding，提供真实的 Embedding 功能
4. **易于测试**：各层职责独立，可以单独测试
5. **易于扩展**：添加新的 VectorEngine 或 EmbeddingProvider 不影响业务层

---

## 实施步骤

### 阶段 1：增强 crates/vector-client

1. **创建 manager 模块**
   - [ ] 新建 `crates/vector-client/src/manager/mod.rs`
   - [ ] 实现 `VectorManager` 结构体
   - [ ] 实现索引管理方法
   - [ ] 实现向量操作方法

2. **创建 embedding 模块**
   - [ ] 新建 `crates/vector-client/src/embedding/mod.rs`
   - [ ] 定义 `EmbeddingService` trait
   - [ ] 迁移 `ref/embedding/base.rs` → `embedding/provider.rs`
   - [ ] 迁移 `ref/embedding/config.rs` → `embedding/config.rs`
   - [ ] 迁移 `ref/embedding/openai_compatible_provider.rs`
   - [ ] 迁移 `ref/embedding/llama_cpp_provider.rs`（可选）
   - [ ] 迁移 `ref/embedding/preprocessor.rs`
   - [ ] 迁移 `ref/embedding/response.rs`
   - [ ] 实现 `EmbeddingServiceImpl`

3. **更新 exports**
   - [ ] 修改 `crates/vector-client/src/lib.rs`
   - [ ] 导出 `VectorManager`
   - [ ] 导出 `EmbeddingService`
   - [ ] 更新文档

### 阶段 2：简化 src/vector

1. **删除冗余文件**
   - [ ] 删除 `src/vector/config.rs`
   - [ ] 删除 `src/vector/manager.rs`
   - [ ] 删除 `src/vector/embedding.rs`

2. **简化 coordinator.rs**
   - [ ] 修改 `VectorCoordinator` 使用 `VectorManager`
   - [ ] 删除索引管理方法
   - [ ] 删除 Embedding 实现
   - [ ] 保留协调方法（`on_vertex_*`）
   - [ ] 简化搜索方法（直接委托）

3. **简化 mod.rs**
   - [ ] 删除配置类型导出
   - [ ] 直接导出 `vector_client` 的类型
   - [ ] 更新文档注释

### 阶段 3：整合 src/sync

1. **创建 vector_sync.rs**
   - [ ] 新建 `src/sync/vector_sync.rs`
   - [ ] 实现 `VectorSyncCoordinator`
   - [ ] 实现图数据变更处理方法

2. **修改 manager.rs**
   - [ ] 添加 `vector_sync_coordinator` 字段
   - [ ] 修改 `on_vertex_change` 使用 `VectorSyncCoordinator`
   - [ ] 更新 `with_vector_coordinator` 方法

3. **修改 task.rs**
   - [ ] 确保向量任务类型完整
   - [ ] 更新任务处理方法

### 阶段 4：更新使用方

1. **更新查询执行器**
   - [ ] 修改 `src/query/executor/data_access/vector_search.rs`
   - [ ] 直接使用 `VectorManager` 而非 `VectorCoordinator`
   - [ ] 更新导入

2. **更新其他使用方**
   - [ ] 检查所有 `use crate::vector::` 的地方
   - [ ] 更新导入路径
   - [ ] 修复编译错误

3. **更新配置**
   - [ ] 检查 `src/config/mod.rs`
   - [ ] 确保配置类型正确导入

### 阶段 5：测试与验证

1. **单元测试**
   - [ ] 为 `VectorManager` 编写测试
   - [ ] 为 `EmbeddingService` 编写测试
   - [ ] 为 `VectorSyncCoordinator` 编写测试

2. **集成测试**
   - [ ] 测试图数据变更时的向量同步
   - [ ] 测试向量搜索功能
   - [ ] 测试 Embedding 功能

3. **性能测试**
   - [ ] 对比重构前后的性能
   - [ ] 确保没有性能退化

### 阶段 6：清理与文档

1. **清理旧代码**
   - [ ] 删除 `ref/embedding` 目录（已迁移）
   - [ ] 清理未使用的导入
   - [ ] 运行 `cargo clippy` 修复警告

2. **更新文档**
   - [ ] 更新架构文档
   - [ ] 更新 API 文档
   - [ ] 更新使用示例

3. **更新 AGENTS.md**
   - [ ] 记录新的架构设计
   - [ ] 记录职责划分原则

---

## 总结

### 核心问题

1. 冗余的 3 层包装（Coordinator → Manager → Engine）
2. 配置类型重复定义
3. Embedding 服务是 Mock 实现，不可用
4. 协调逻辑分散在多个位置

### 重构方案

1. **crates/vector-client**：整合底层实现（VectorManager、EmbeddingService）
2. **src/vector**：简化为协调层（仅保留图数据协调）
3. **src/sync**：统一管理同步逻辑（包含向量同步协调器）

### 预期收益

1. 代码量减少 ~30%（删除冗余包装和配置）
2. 查询性能提升（减少函数调用层）
3. Embedding 功能可用（整合 ref/embedding）
4. 职责清晰，易于维护和扩展

### 风险与缓解

1. **风险**：重构范围大，可能引入 bug
   - **缓解**：分阶段实施，每阶段都有测试覆盖

2. **风险**：影响现有功能
   - **缓解**：保持向后兼容，逐步迁移使用方

3. **风险**：团队学习成本
   - **缓解**：详细文档 + 代码示例

---

**文档创建时间**：2026-04-10  
**作者**：AI Assistant  
**参考目录**：`src/vector/`, `crates/vector-client/`, `src/sync/`, `ref/embedding/`
