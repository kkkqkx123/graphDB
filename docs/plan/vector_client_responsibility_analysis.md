# Vector-Client 包职责划分与架构设计

**创建日期**: 2026-04-11  
**状态**: 建议  
**相关文档**: [同步系统架构重构计划](./sync_architecture_refactoring_plan.md)

---

## 📋 概述

本文档分析 `crates/vector-client` 包的职责边界，明确哪些功能应该在 vector-client 中实现，哪些应该保留在 graphDB 主项目中。

### 核心原则

1. **vector-client** - 底层向量数据库客户端库
   - 专注于向量存储和检索
   - 提供通用的向量操作 API
   - 与图数据库逻辑解耦

2. **graphDB** - 图数据库系统
   - 专注于图数据管理
   - 处理事务和协调
   - 集成全文和向量索引

---

## 🎯 当前架构分析

### 现有结构

```
crates/vector-client/
├── src/
│   ├── lib.rs
│   ├── api/
│   ├── config/
│   ├── embedding/
│   ├── engine/
│   │   └── qdrant/
│   ├── manager/
│   │   └── mod.rs          # VectorManager
│   ├── types/
│   │   ├── config.rs
│   │   ├── filter.rs
│   │   ├── point.rs        # VectorPoint
│   │   └── search.rs       # SearchQuery, SearchResult
│   └── error.rs
```

### 当前职责

| 模块 | 职责 | 位置 | 合理性 |
|------|------|------|--------|
| `VectorManager` | 索引生命周期管理 | ✅ vector-client | ✅ 合理 |
| `VectorEngine` | 底层引擎抽象 | ✅ vector-client | ✅ 合理 |
| `VectorPoint` | 向量点数据结构 | ✅ vector-client | ✅ 合理 |
| `SearchQuery` | 查询结构 | ✅ vector-client | ✅ 合理 |
| `EmbeddingService` | 嵌入生成服务 | ✅ vector-client | ✅ 合理 |
| `VectorBatchManager` | 批量处理 + 事务 | ❌ graphDB | ⚠️ 待讨论 |

---

## 🏗️ 推荐的职责划分

### 应该在 vector-client 中实现的功能

#### 1. 底层批量处理（推荐 ✅）

**位置**: `crates/vector-client/src/batch/`

```rust
/// 向量批量处理器
pub struct VectorBatchProcessor {
    manager: Arc<VectorManager>,
    config: BatchConfig,
    buffer: VectorBuffer,
    background_task: Mutex<Option<JoinHandle<()>>>,
}

impl VectorBatchProcessor {
    /// 批量 upsert 向量
    pub async fn upsert_batch(&self, points: Vec<VectorPoint>) -> Result<()>;
    
    /// 批量删除向量
    pub async fn delete_batch(&self, ids: Vec<String>) -> Result<()>;
    
    /// 启动后台定时提交任务
    pub fn start_background_commit(&self);
    
    /// 手动提交所有缓冲
    pub async fn commit_all(&self) -> Result<()>;
}
```

**理由**:
- ✅ 批量处理是向量数据库的通用需求
- ✅ 可以复用给其他项目使用
- ✅ 与图数据库逻辑解耦
- ✅ 提高代码复用性

**实现细节**:

```
crates/vector-client/src/batch/
├── mod.rs
├── processor.rs          # VectorBatchProcessor
├── buffer.rs             # VectorBuffer (缓冲机制)
├── config.rs             # BatchConfig (可复用 graphDB 的)
└── error.rs              # BatchError
```

---

#### 2. 向量缓冲机制（推荐 ✅）

**位置**: `crates/vector-client/src/buffer.rs`

```rust
/// 向量操作缓冲器
pub struct VectorBuffer {
    /// Upsert 缓冲
    upsert_buffer: DashMap<String, Vec<VectorPoint>>,
    /// Delete 缓冲
    delete_buffer: DashMap<String, Vec<String>>,
    /// 最后提交时间
    last_commit: DashMap<String, Instant>,
}

impl VectorBuffer {
    /// 添加 upsert 操作
    pub fn add_upsert(&self, collection: &str, point: VectorPoint);
    
    /// 添加 delete 操作
    pub fn add_delete(&self, collection: &str, point_id: String);
    
    /// 获取并清空 upsert 缓冲
    pub fn drain_upserts(&self, collection: &str) -> Vec<VectorPoint>;
    
    /// 获取并清空 delete 缓冲
    pub fn drain_deletes(&self, collection: &str) -> Vec<String>;
}
```

**理由**:
- ✅ 通用缓冲机制
- ✅ 与具体业务逻辑解耦
- ✅ 可以被多个项目复用

---

#### 3. 自动提交策略（推荐 ✅）

**位置**: `crates/vector-client/src/batch/processor.rs`

```rust
impl VectorBatchProcessor {
    /// 检查是否应该提交
    fn should_commit(&self, collection: &str) -> bool {
        // 基于大小
        if self.buffer.count(collection) >= self.config.batch_size {
            return true;
        }
        
        // 基于时间
        if self.buffer.is_timeout(collection, self.config.commit_interval) {
            return true;
        }
        
        false
    }
    
    /// 自动提交检查
    async fn auto_commit(&self) -> Result<()> {
        let collections = self.buffer.get_all_collections();
        for collection in collections {
            if self.should_commit(&collection) {
                self.commit_collection(&collection).await?;
            }
        }
        Ok(())
    }
}
```

**理由**:
- ✅ 自动提交是通用需求
- ✅ 策略可配置
- ✅ 与图数据库事务无关

---

### 应该保留在 graphDB 中的功能

#### 1. 事务两阶段提交（必须 ✅）

**位置**: `src/transaction/sync_handle.rs` 或 `src/batch/transaction.rs`

```rust
/// 事务向量操作缓冲器
pub struct TransactionVectorBuffer {
    batch_processor: Arc<VectorBatchProcessor>, // 委托给 vector-client
    pending_operations: DashMap<TransactionId, Vec<PendingVectorOperation>>,
}

impl TransactionVectorBuffer {
    /// 阶段 1：缓冲操作（在事务内）
    pub async fn prepare(
        &self,
        txn_id: TransactionId,
        operation: PendingVectorOperation,
    ) -> Result<(), TransactionError>;
    
    /// 阶段 2：提交（事务提交时）
    pub async fn commit(&self, txn_id: TransactionId) -> Result<(), TransactionError>;
    
    /// 回滚（事务回滚时）
    pub async fn rollback(&self, txn_id: TransactionId) -> Result<(), TransactionError>;
}
```

**理由**:
- ✅ 事务管理是图数据库的职责
- ✅ 与图事务集成
- ✅ 不应该污染 vector-client 库

**操作类型定义**:

```rust
/// 待处理的向量操作（图数据库特定）
#[derive(Debug, Clone)]
pub struct PendingVectorOperation {
    pub operation: VectorOperation,  // Upsert 或 Delete
    pub space_id: u64,               // 图空间 ID
    pub tag_name: String,            // 标签名
    pub field_name: String,          // 字段名
    pub change_type: VectorChangeType, // Insert 或 Delete
}

/// 向量操作（委托给 vector-client）
#[derive(Debug, Clone)]
pub enum VectorOperation {
    Upsert(VectorPoint),
    Delete(String),
}
```

---

#### 2. 与 SyncManager 集成（必须 ✅）

**位置**: `src/sync/manager.rs`

```rust
pub struct SyncManager {
    // ...
    vector_batch_processor: Arc<VectorBatchProcessor>, // 来自 vector-client
    transaction_buffer: Arc<TransactionVectorBuffer>,  // 图数据库特定
}
```

**理由**:
- ✅ 协调全文和向量索引
- ✅ 处理同步模式（Sync/Async/Off）
- ✅ 图数据库特有的业务逻辑

---

#### 3. 索引映射和命名（必须 ✅）

**位置**: `src/index/vector.rs`

```rust
/// 向量索引位置映射
pub struct VectorIndexMapper {
    // space_id + tag_name + field_name -> collection_name
    mappings: DashMap<IndexKey, String>,
}

impl VectorIndexMapper {
    /// 生成集合名称
    pub fn to_collection_name(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
    ) -> String {
        format!("space_{}_{}_{}", space_id, tag_name, field_name)
    }
    
    /// 解析集合名称
    pub fn from_collection_name(
        &self,
        collection_name: &str,
    ) -> Option<(u64, String, String)>;
}
```

**理由**:
- ✅ 图数据库特定的命名约定
- ✅ 与图 schema 集成
- ✅ 不应该硬编码在 vector-client 中

---

#### 4. Embedding 生成策略（必须 ✅）

**位置**: `src/index/vector.rs` 或 `src/embedding/strategy.rs`

```rust
/// Embedding 生成策略（图数据库特定）
pub trait EmbeddingStrategy: Send + Sync {
    /// 判断是否需要生成 embedding
    fn should_generate(&self, value: &Value) -> bool;
    
    /// 生成 embedding
    async fn generate(&self, value: &Value) -> Result<Vec<f32>>;
}

/// 默认策略：向量类型直接返回
pub struct DefaultEmbeddingStrategy;

#[async_trait]
impl EmbeddingStrategy for DefaultEmbeddingStrategy {
    fn should_generate(&self, value: &Value) -> bool {
        value.is_vector()
    }
    
    async fn generate(&self, value: &Value) -> Result<Vec<f32>> {
        value.as_vector().ok_or("Not a vector")
    }
}

/// 文本 embedding 策略：为文本生成 embedding
pub struct TextEmbeddingStrategy {
    embedding_service: Arc<EmbeddingService>,
}

#[async_trait]
impl EmbeddingStrategy for TextEmbeddingStrategy {
    fn should_generate(&self, value: &Value) -> bool {
        value.is_string()
    }
    
    async fn generate(&self, value: &Value) -> Result<Vec<f32>> {
        let text = value.as_string().ok_or("Not a string")?;
        self.embedding_service.embed_query(&text).await
    }
}
```

**理由**:
- ✅ 图数据库特定的业务逻辑
- ✅ 支持多种 embedding 生成策略
- ✅ 不应该硬编码在 vector-client 中

---

## 📊 架构对比

### 方案 A：当前设计（不推荐 ❌）

```
graphDB/
├── src/sync/vector_batch.rs    # 包含所有批量处理逻辑
└── src/transaction/sync_handle.rs

vector-client/
└── src/manager/mod.rs          # 只有基础 CRUD
```

**问题**:
- ❌ 批量处理逻辑重复（如果其他项目也需要）
- ❌ vector-client 功能不完整
- ❌ 难以复用

---

### 方案 B：完全分离（推荐 ✅）

```
graphDB/
├── src/index/
│   ├── trait.rs                # IndexEngine trait
│   ├── fulltext.rs
│   └── vector.rs               # 图特定的向量索引逻辑
├── src/batch/
│   ├── trait.rs                # BatchProcessor trait
│   ├── config.rs               # 统一配置
│   └── transaction.rs          # 事务两阶段提交
└── src/sync/
    └── manager.rs              # 协调器

vector-client/
├── src/manager/mod.rs          # VectorManager (基础 CRUD)
├── src/batch/                  # 新增：通用批量处理
│   ├── processor.rs            # VectorBatchProcessor
│   ├── buffer.rs               # VectorBuffer
│   └── config.rs               # BatchConfig
└── src/types/
```

**优势**:
- ✅ 职责清晰
- ✅ 代码复用
- ✅ 易于维护
- ✅ vector-client 可独立使用

---

### 方案 C：过度分离（不推荐 ❌）

```
vector-client/
├── src/batch/
├── src/transaction/            # ❌ 不应该有事务逻辑
├── src/coordinator/            # ❌ 不应该有协调逻辑
└── src/mapper/                 # ❌ 不应该有图特定的映射
```

**问题**:
- ❌ 职责越界
- ❌ 耦合图数据库逻辑
- ❌ 降低 vector-client 的通用性

---

## 🎯 推荐架构

### 最终推荐：方案 B（适度分离）

```
┌─────────────────────────────────────────────────┐
│              graphDB (图数据库)                  │
│                                                 │
│  ┌─────────────────────────────────────────┐   │
│  │  Index Layer (src/index/)               │   │
│  │  - IndexEngine trait                    │   │
│  │  - FulltextEngine impl                  │   │
│  │  - VectorEngine impl (使用 vector-client)│   │
│  └─────────────────────────────────────────┘   │
│                                                 │
│  ┌─────────────────────────────────────────┐   │
│  │  Batch Layer (src/batch/)               │   │
│  │  - BatchProcessor trait                 │   │
│  │  - GenericBatchProcessor                │   │
│  │  - TransactionBuffer (两阶段提交)        │   │
│  └─────────────────────────────────────────┘   │
│                                                 │
│  ┌─────────────────────────────────────────┐   │
│  │  Sync Layer (src/sync/)                 │   │
│  │  - SyncCoordinator (协调全文 + 向量)     │   │
│  │  - SyncManager (高层 API)               │   │
│  └─────────────────────────────────────────┘   │
└─────────────────────────────────────────────────┘
              ⬇ 使用 (依赖) ⬇
┌─────────────────────────────────────────────────┐
│         vector-client (向量数据库客户端)          │
│                                                 │
│  ┌─────────────────────────────────────────┐   │
│  │  Manager Layer                          │   │
│  │  - VectorManager                        │   │
│  │  - VectorEngine trait                   │   │
│  │  - QdrantEngine impl                    │   │
│  └─────────────────────────────────────────┘   │
│                                                 │
│  ┌─────────────────────────────────────────┐   │
│  │  Batch Layer (新增)                     │   │
│  │  - VectorBatchProcessor                 │   │
│  │  - VectorBuffer                         │   │
│  │  - BatchConfig                          │   │
│  └─────────────────────────────────────────┘   │
│                                                 │
│  ┌─────────────────────────────────────────┐   │
│  │  Types Layer                            │   │
│  │  - VectorPoint                          │   │
│  │  - SearchQuery, SearchResult            │   │
│  │  - CollectionConfig                     │   │
│  └─────────────────────────────────────────┘   │
└─────────────────────────────────────────────────┘
```

---

## 📝 实施步骤

### 步骤 1：在 vector-client 中添加批量处理（1-2 天）

**任务**:

1. 创建 `crates/vector-client/src/batch/mod.rs`
2. 实现 `VectorBatchProcessor`
3. 实现 `VectorBuffer`
4. 添加 `BatchConfig`（或复用 graphDB 的）
5. 编写单元测试

**代码结构**:

```rust
// crates/vector-client/src/batch/processor.rs
pub struct VectorBatchProcessor {
    manager: Arc<VectorManager>,
    config: BatchConfig,
    buffer: VectorBuffer,
    background_task: Mutex<Option<JoinHandle<()>>>,
}

impl VectorBatchProcessor {
    pub fn new(manager: Arc<VectorManager>, config: BatchConfig) -> Self;
    
    pub async fn upsert(&self, collection: &str, point: VectorPoint) -> Result<()>;
    pub async fn upsert_batch(&self, collection: &str, points: Vec<VectorPoint>) -> Result<()>;
    pub async fn delete(&self, collection: &str, point_id: &str) -> Result<()>;
    pub async fn delete_batch(&self, collection: &str, point_ids: Vec<String>) -> Result<()>;
    
    pub fn start_background_commit(&self);
    pub async fn commit_all(&self) -> Result<()>;
}
```

---

### 步骤 2：简化 graphDB 的 vector_batch.rs（1 天）

**任务**:

1. 保留事务相关的两阶段提交逻辑
2. 委托批量处理给 `VectorBatchProcessor`
3. 简化代码

**重构后**:

```rust
// src/sync/vector_batch.rs (简化版)
pub struct VectorBatchManager {
    vector_processor: Arc<VectorBatchProcessor>, // 委托给 vector-client
    pending_buffers: DashMap<TransactionId, Vec<PendingVectorOperation>>,
}

impl VectorBatchManager {
    pub fn new(vector_processor: Arc<VectorBatchProcessor>) -> Self;
    
    // 只保留事务相关的两阶段提交方法
    pub async fn buffer_operation(...) -> Result<()>;
    pub async fn commit_transaction(...) -> Result<()>;
    pub async fn rollback_transaction(...) -> Result<()>;
}
```

---

### 步骤 3：统一配置（0.5 天）

**任务**:

1. 在 `src/batch/config.rs` 定义统一的 `BatchConfig`
2. vector-client 和 graphDB 共享同一配置
3. 移除 `VectorBatchConfig`

---

### 步骤 4：更新依赖和文档（0.5 天）

**任务**:

1. 更新 `Cargo.toml`（如有需要）
2. 更新 API 文档
3. 更新架构文档

---

## ✅ 验收标准

### vector-client 包

- [ ] `VectorBatchProcessor` 实现完整
- [ ] `VectorBuffer` 实现完整
- [ ] 单元测试覆盖率 > 80%
- [ ] 文档完整
- [ ] 可独立使用（不依赖 graphDB）

### graphDB 项目

- [ ] `VectorBatchManager` 简化完成
- [ ] 事务两阶段提交功能完整
- [ ] 所有现有测试通过
- [ ] 性能无退化
- [ ] 代码重复率 < 10%

---

## 📚 参考

### 相关文档

- [同步系统架构重构计划](./sync_architecture_refactoring_plan.md)
- [Vector Batch Improvements](./sync/vector_batch_improvements.md)

### 设计模式

- **Repository Pattern** - `VectorManager` 作为仓库
- **Strategy Pattern** - 批量处理策略
- **Decorator Pattern** - `VectorBatchProcessor` 装饰 `VectorManager`

### Rust 最佳实践

- 使用 `Arc` 共享状态
- 使用 `DashMap` 处理并发
- 使用 `tokio::spawn` 后台任务
- 使用 trait 抽象提高可测试性

---

**文档维护者**: AI Assistant  
**最后更新**: 2026-04-11
