# Search Adapter Architecture Analysis

## 概述

本文档分析 `src/search/adapters` 目录的设计架构，评估其是否应该使用 `crates/bm25` 和 `crates/inversearch` 包的 API 层，而不是直接使用内部组件。

**分析日期**: 2026-04-06  
**分析范围**: 
- `src/search/adapters/bm25_adapter.rs`
- `src/search/adapters/inversearch_adapter.rs`
- `crates/bm25/src/api/`
- `crates/inversearch/src/api/`

---

## 1. 当前架构分析

### 1.1 src/search/adapters 目录结构

```
src/search/adapters/
├── mod.rs                    # 模块导出
├── bm25_adapter.rs           # BM25 搜索引擎适配器 (368 行)
├── bm25_adapter_test.rs      # BM25 测试
└── inversearch_adapter.rs    # Inversearch 搜索引擎适配器 (192 行)
```

**职责**: 实现 `SearchEngine` trait，为 GraphDB 提供统一的全文搜索接口。

### 1.2 SearchEngine Trait 定义

[engine.rs](file://d:\项目\database\graphDB\src\search\engine.rs#L6-L28):

```rust
#[async_trait]
pub trait SearchEngine: Send + Sync + std::fmt::Debug {
    fn name(&self) -> &str;
    fn version(&self) -> &str;

    async fn index(&self, doc_id: &str, content: &str) -> Result<(), SearchError>;
    async fn index_batch(&self, docs: Vec<(String, String)>) -> Result<(), SearchError>;
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, SearchError>;
    async fn delete(&self, doc_id: &str) -> Result<(), SearchError>;
    async fn delete_batch(&self, doc_ids: Vec<&str>) -> Result<(), SearchError>;
    
    async fn commit(&self) -> Result<(), SearchError>;
    async fn rollback(&self) -> Result<(), SearchError>;
    async fn stats(&self) -> Result<IndexStats, SearchError>;
    async fn close(&self) -> Result<(), SearchError>;
}
```

### 1.3 当前实现方式

#### BM25 Adapter (问题较多)

**当前实现**: [bm25_adapter.rs](file://d:\项目\database\graphDB\src\search\adapters\bm25_adapter.rs#L1-L25)

```rust
use bm25_service::index::delete::delete_document_with_writer;
use bm25_service::index::document::add_document_with_writer;
use bm25_service::index::search::{search, SearchOptions};
use bm25_service::index::stats::get_stats;
use bm25_service::index::{IndexManager, IndexSchema};
use tantivy::IndexWriter;  // ❌ 直接依赖 tantivy
use tokio::sync::Mutex;

pub struct Bm25SearchEngine {
    manager: Arc<IndexManager>,
    schema: IndexSchema,
    index_path: std::path::PathBuf,
    writer: Arc<Mutex<Option<IndexWriter>>>,  // ❌ 直接持有 tantivy::IndexWriter
    operation_count: Arc<AtomicUsize>,
    batch_size: usize,
}
```

**关键问题**:

1. **直接使用内部组件**:
   - `bm25_service::index::IndexManager` (内部组件)
   - `bm25_service::index::IndexSchema` (内部组件)
   - `tantivy::IndexWriter` (第三方依赖)
   - `bm25_service::index::delete::delete_document_with_writer` (底层函数)

2. **复杂的 Writer 管理**:
   ```rust
   // 第 71-81 行：复杂的 writer 获取逻辑
   async fn get_or_create_writer(&self) -> Result<IndexWriter, SearchError> {
       let mut writer_guard: tokio::sync::MutexGuard<'_, Option<IndexWriter>> =
           self.writer.lock().await;
       if writer_guard.is_none() {
           let writer = self
               .manager
               .writer()
               .map_err(|e| SearchError::Bm25Error(format!("Failed to create writer: {}", e)))?;
           *writer_guard = Some(writer);
       }
       Ok(writer_guard.take().unwrap())  // ❌ 使用 unwrap
   }
   ```

3. **spawn_blocking 中的复杂逻辑**:
   ```rust
   // 第 120-143 行
   tokio::task::spawn_blocking(move || {
       let mut writer_guard = futures::executor::block_on(writer.lock());
       if writer_guard.is_none() {
           return Err(SearchError::Internal("Writer not initialized".to_string()));
       }
       
       let writer_ref = writer_guard.as_mut().unwrap();
       add_document_with_writer(writer_ref, &schema, &doc_id, &fields)
           .map_err(|e| SearchError::Bm25Error(e.to_string()))?;
       
       if should_commit {
           writer_ref
               .commit()
               .map_err(|e| SearchError::Bm25Error(format!("Commit failed: {}", e)))?;
       }
       
       Ok(())
   })
   ```

4. **commit()/rollback()/close() 实现正确但复杂**:
   ```rust
   // 第 267-283 行
   async fn commit(&self) -> Result<(), SearchError> {
       let mut writer_guard: tokio::sync::MutexGuard<'_, Option<IndexWriter>> =
           self.writer.lock().await;
       if let Some(mut writer) = writer_guard.take() {
           writer
               .commit()
               .map_err(|e| SearchError::Bm25Error(format!("Commit failed: {}", e)))?;
           *writer_guard = Some(writer);
       }
       Ok(())
   }
   ```

#### Inversearch Adapter (相对简洁)

**当前实现**: [inversearch_adapter.rs](file://d:\项目\database\graphDB\src\search\adapters\inversearch_adapter.rs#L1-L50)

```rust
use inversearch_service::index::IndexOptions;
use inversearch_service::Index;  // ❌ 直接使用核心 Index
use parking_lot::Mutex;

pub struct InversearchEngine {
    index: Mutex<Index>,  // ❌ 直接持有 inversearch::Index
    config: InversearchConfig,
}
```

**问题**:

1. **直接使用核心 Index**:
   - `inversearch_service::Index` (核心组件，非 API 层)
   - `inversearch_service::index::IndexOptions` (内部配置)

2. **简单的包装**:
   ```rust
   // 第 89-101 行
   async fn index(&self, doc_id: &str, content: &str) -> Result<(), SearchError> {
       let mut index = self.index.lock();
       let doc_id_u64 = doc_id
           .parse::<u64>()
           .map_err(|_| SearchError::InvalidDocId(doc_id.to_string()))?;
       index
           .add(doc_id_u64, content, false)
           .map_err(|e| SearchError::InversearchError(e.to_string()))?;
       Ok(())
   }
   ```

3. **commit()/rollback() 空实现**:
   ```rust
   async fn commit(&self) -> Result<(), SearchError> {
       Ok(())  // 空实现
   }
   
   async fn rollback(&self) -> Result<(), SearchError> {
       Ok(())  // 空实现
   }
   ```

---

## 2. Crates API 层分析

### 2.1 BM25 Crate API 架构

**文件结构**:
```
crates/bm25/src/api/
├── mod.rs                    # 模块导出
├── core/                     # 核心 API (始终可用)
│   ├── mod.rs               # 导出所有核心功能
│   ├── index.rs             # IndexManager
│   ├── search.rs            # search 函数
│   ├── document.rs          # add_document 等
│   ├── delete.rs            # delete_document 等
│   ├── batch.rs             # 批量操作
│   ├── schema.rs            # IndexSchema
│   ├── stats.rs             # 统计信息
│   └── persistence.rs       # 持久化
├── embedded/                 # 嵌入式 API (embedded 特性)
│   ├── mod.rs
│   └── index.rs             # Bm25Index 高级封装
└── server/                   # 服务端 API (service 特性)
    ├── mod.rs
    ├── grpc.rs              # gRPC 服务
    └── config.rs            # 服务配置
```

**核心 API (api/core/mod.rs)**:

```rust
pub use index::{IndexManager, IndexManagerConfig, LogMergePolicyConfig, MergePolicyType, ReloadPolicyConfig};
pub use search::{search, SearchOptions, SearchResult};
pub use document::{add_document, add_document_with_writer, update_document_with_writer, get_document};
pub use delete::{delete_document, delete_document_with_writer, batch_delete_documents};
pub use batch::{batch_add_documents, batch_add_documents_with_writer};
pub use schema::IndexSchema;
pub use stats::{get_stats, IndexStats};
pub use persistence::{PersistenceManager, IndexMetadata, BackupInfo};
```

**嵌入式 API (api/embedded/index.rs)** - **推荐用于 GraphDB**:

```rust
pub struct Bm25Index {
    manager: IndexManager,
    schema: IndexSchema,
}

impl Bm25Index {
    pub fn create<P: AsRef<std::path::Path>>(path: P) -> Result<Self> { }
    pub fn open<P: AsRef<std::path::Path>>(path: P) -> Result<Self> { }
    
    pub fn add_document(&self, document_id: &str, title: &str, content: &str) -> Result<()> { }
    pub fn update_document(&self, document_id: &str, title: &str, content: &str) -> Result<()> { }
    pub fn delete_document(&self, document_id: &str) -> Result<()> { }
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> { }
    pub fn count(&self) -> Result<u64> { }
    pub fn commit(&self) -> Result<()> { }
    
    pub fn manager(&self) -> &IndexManager { }
    pub fn schema(&self) -> &IndexSchema { }
}
```

**关键优势**:
1. ✅ **高级抽象**: 隐藏 IndexManager 和 IndexWriter 的复杂性
2. ✅ **简化 API**: 提供简单易用的方法
3. ✅ **封装细节**: 内部处理 writer 管理
4. ✅ **类型安全**: 不暴露 tantivy 类型

### 2.2 Inversearch Crate API 架构

**文件结构**:
```
crates/inversearch/src/api/
├── mod.rs
├── core/                     # 核心 API (重新导出所有功能)
│   └── mod.rs               # 大量 re-export
├── embedded/                 # 嵌入式 API (embedded 特性)
│   ├── mod.rs
│   └── index.rs             # EmbeddedIndex 高级封装
└── server/                   # 服务端 API (service 特性)
    └── mod.rs
```

**核心 API (api/core/mod.rs)**:

```rust
// 大量 re-export，包括:
pub use crate::index::Index;
pub use crate::search::{search, SearchResult, SearchOptions};
pub use crate::document::{Document, Batch, BatchOperation};
pub use crate::highlight::{highlight_document, HighlightProcessor};
pub use crate::resolver::{Resolver, combine_search_results};
pub use crate::storage::common::trait::StorageInterface;
// ... 等等
```

**嵌入式 API (api/embedded/index.rs)** - **推荐用于 GraphDB**:

```rust
pub struct EmbeddedIndex {
    index: Index,
    config: EmbeddedConfig,
    document_store: HashMap<DocId, String>,
}

impl EmbeddedIndex {
    pub fn create() -> Result<Self> { }
    pub fn create_at(path: impl Into<PathBuf>) -> Result<Self> { }
    pub fn with_config(config: EmbeddedConfig) -> Result<Self> { }
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> { }
    
    pub fn add(&mut self, id: DocId, content: impl Into<String>) -> Result<()> { }
    pub fn add_with_fields(&mut self, id: DocId, fields: Vec<(String, String)>) -> Result<()> { }
    pub fn update(&mut self, id: DocId, content: impl Into<String>) -> Result<()> { }
    pub fn remove(&mut self, id: DocId) -> Result<()> { }
    pub fn get(&self, id: DocId) -> Option<&str> { }
    pub fn search(&self, query: impl Into<String>) -> Result<Vec<EmbeddedSearchResult>> { }
    pub fn stats(&self) -> EmbeddedIndexStats { }
    pub fn save(&self) -> Result<()> { }
    pub fn load(&mut self) -> Result<()> { }
}
```

**关键优势**:
1. ✅ **文档存储**: 内置文档存储功能
2. ✅ **持久化**: 提供 save()/load() 方法
3. ✅ **简化 API**: 隐藏复杂的配置
4. ✅ **类型安全**: 不暴露内部类型

---

## 3. 设计问题识别

### 3.1 主要设计问题

| 问题 | 严重程度 | 描述 |
|------|----------|------|
| **直接使用内部组件** | 🔴 严重 | 直接使用 `IndexManager`, `Index`, `IndexWriter` 等内部类型 |
| **暴露第三方依赖** | 🔴 严重 | BM25 adapter 直接依赖 `tantivy::IndexWriter` |
| **复杂的 Writer 管理** | 🟡 中等 | BM25 adapter 需要手动管理 IndexWriter 生命周期 |
| **缺少持久化支持** | 🟡 中等 | Inversearch adapter 未使用 crate 提供的持久化功能 |
| **代码重复** | 🟡 中等 | 两个 adapter 都实现了类似的 writer/lock 管理逻辑 |
| **空实现风险** | 🟡 中等 | Inversearch 的 commit()/rollback() 为空实现 |

### 3.2 违反的设计原则

1. **封装原则 (Encapsulation)**:
   - ❌ 当前实现暴露了底层实现细节 (IndexManager, IndexWriter)
   - ✅ 应使用 API 层封装这些细节

2. **依赖倒置原则 (Dependency Inversion)**:
   - ❌ 直接依赖具体实现 (tantivy::IndexWriter)
   - ✅ 应依赖抽象 (API 层接口)

3. **单一职责原则 (Single Responsibility)**:
   - ❌ Bm25SearchEngine 同时管理索引和 writer
   - ✅ 应只负责适配，writer 管理交给 API 层

4. **最小知识原则 (Law of Demeter)**:
   - ❌ 调用 `bm25_service::index::delete::delete_document_with_writer`
   - ✅ 应通过 API 层间接调用

### 3.3 具体代码问题

#### BM25 Adapter

**问题 1**: 直接使用 tantivy 类型
```rust
use tantivy::IndexWriter;  // ❌ 暴露第三方依赖

pub struct Bm25SearchEngine {
    writer: Arc<Mutex<Option<IndexWriter>>>,  // ❌ 类型耦合
}
```

**问题 2**: 复杂的 writer 获取逻辑
```rust
async fn get_or_create_writer(&self) -> Result<IndexWriter, SearchError> {
    let mut writer_guard = self.writer.lock().await;
    if writer_guard.is_none() {
        let writer = self.manager.writer()?;  // ❌ 每次创建新 writer
        *writer_guard = Some(writer);
    }
    Ok(writer_guard.take().unwrap())  // ❌ 使用 unwrap
}
```

**问题 3**: spawn_blocking 中的复杂逻辑
```rust
tokio::task::spawn_blocking(move || {
    let mut writer_guard = futures::executor::block_on(writer.lock());
    if writer_guard.is_none() {
        return Err(SearchError::Internal("Writer not initialized".to_string()));
    }
    let writer_ref = writer_guard.as_mut().unwrap();
    // ... 复杂操作
})
```

#### Inversearch Adapter

**问题 1**: 直接使用核心 Index
```rust
use inversearch_service::Index;  // ❌ 内部组件

pub struct InversearchEngine {
    index: Mutex<Index>,  // ❌ 直接持有
}
```

**问题 2**: 空实现
```rust
async fn commit(&self) -> Result<(), SearchError> {
    Ok(())  // ❌ 空实现，数据可能丢失
}

async fn rollback(&self) -> Result<(), SearchError> {
    Ok(())  // ❌ 空实现
}
```

**问题 3**: 未使用持久化功能
```rust
// crates/inversearch 提供了完整的持久化功能
// 但 adapter 中未使用
pub fn load(_path: &Path, config: InversearchConfig) -> Result<Self, SearchError> {
    // ❌ 忽略 path 参数，不加载任何数据
    let index = Index::new(options)?;
    Ok(Self { index: Mutex::new(index), config })
}
```

---

## 4. 改进建议

### 4.1 总体架构建议

**推荐方案**: 使用 crates 的 **embedded API** 层

```
src/search/adapters/
├── bm25_adapter.rs           # 使用 bm25_service::api::embedded::Bm25Index
└── inversearch_adapter.rs    # 使用 inversearch_service::api::embedded::EmbeddedIndex
```

**优势**:
1. ✅ **解耦**: 不直接依赖内部组件和第三方库
2. ✅ **简化**: API 层处理复杂性
3. ✅ **一致性**: 两个 adapter 使用相同的抽象级别
4. ✅ **可维护**: 底层变化不影响 adapter

### 4.2 BM25 Adapter 重构方案

**当前实现** (368 行，复杂):
```rust
pub struct Bm25SearchEngine {
    manager: Arc<IndexManager>,
    schema: IndexSchema,
    index_path: PathBuf,
    writer: Arc<Mutex<Option<IndexWriter>>>,
    operation_count: Arc<AtomicUsize>,
    batch_size: usize,
}
```

**重构后** (预计 150 行，简洁):
```rust
use bm25_service::api::embedded::Bm25Index;

pub struct Bm25SearchEngine {
    index: Bm25Index,
    index_path: PathBuf,
}

impl Bm25SearchEngine {
    pub fn open_or_create(path: &Path) -> Result<Self, SearchError> {
        let index = Bm25Index::open_or_create(path)
            .map_err(|e| SearchError::Bm25Error(e.to_string()))?;
        Ok(Self {
            index,
            index_path: path.to_path_buf(),
        })
    }
}

#[async_trait]
impl SearchEngine for Bm25SearchEngine {
    async fn index(&self, doc_id: &str, content: &str) -> Result<(), SearchError> {
        // 简单调用 API 层
        self.index.add_document(doc_id, "", content)
            .map_err(|e| SearchError::Bm25Error(e.to_string()))?;
        Ok(())
    }
    
    async fn commit(&self) -> Result<(), SearchError> {
        self.index.commit()
            .map_err(|e| SearchError::Bm25Error(e.to_string()))?;
        Ok(())
    }
    
    // 其他方法类似简化
}
```

**关键改进**:
1. ✅ 不直接持有 `IndexWriter`
2. ✅ 使用 `Bm25Index` 的高级 API
3. ✅ 移除复杂的 writer 管理逻辑
4. ✅ 移除 `unwrap()` 使用

### 4.3 Inversearch Adapter 重构方案

**当前实现** (192 行，缺少持久化):
```rust
pub struct InversearchEngine {
    index: Mutex<Index>,
    config: InversearchConfig,
}
```

**重构后** (预计 120 行，完整功能):
```rust
use inversearch_service::api::embedded::EmbeddedIndex;

pub struct InversearchEngine {
    index: EmbeddedIndex,
}

impl InversearchEngine {
    pub fn new(config: InversearchConfig) -> Result<Self, SearchError> {
        let mut index = EmbeddedIndex::create()
            .map_err(|e| SearchError::InversearchError(e.to_string()))?;
        
        // 如果有持久化路径，加载数据
        if let Some(path) = config.persistence_path {
            if path.exists() {
                index.load_from(path)
                    .map_err(|e| SearchError::InversearchError(e.to_string()))?;
            }
        }
        
        Ok(Self { index })
    }
}

#[async_trait]
impl SearchEngine for InversearchEngine {
    async fn index(&self, doc_id: &str, content: &str) -> Result<(), SearchError> {
        let id = doc_id.parse::<u64>()
            .map_err(|_| SearchError::InvalidDocId(doc_id.to_string()))?;
        
        // 使用 API 层的 add 方法
        self.index.add(id, content)
            .map_err(|e| SearchError::InversearchError(e.to_string()))?;
        Ok(())
    }
    
    async fn commit(&self) -> Result<(), SearchError> {
        // 使用 API 层的 save 方法
        self.index.save()
            .map_err(|e| SearchError::InversearchError(e.to_string()))?;
        Ok(())
    }
    
    async fn close(&self) -> Result<(), SearchError> {
        // 关闭前保存
        self.index.save()
            .map_err(|e| SearchError::InversearchError(e.to_string()))?;
        Ok(())
    }
}
```

**关键改进**:
1. ✅ 使用 `EmbeddedIndex` 的高级 API
2. ✅ 实现真正的持久化
3. ✅ commit()/close() 不再为空实现
4. ✅ 文档自动存储

### 4.4 迁移步骤

**阶段 1: 准备工作**
1. 确认 crates 的 embedded API 功能完整
2. 更新 Cargo.toml 确保启用 `embedded` 特性
3. 编写 API 层使用文档

**阶段 2: BM25 Adapter 重构**
1. 创建新的 `Bm25SearchEngineV2` 使用 `Bm25Index`
2. 并行测试新旧实现
3. 替换旧实现

**阶段 3: Inversearch Adapter 重构**
1. 创建新的 `InversearchEngineV2` 使用 `EmbeddedIndex`
2. 实现持久化功能
3. 替换旧实现

**阶段 4: 清理与优化**
1. 移除不再使用的导入
2. 简化错误处理
3. 更新测试用例

---

## 5. 对比分析

### 5.1 代码复杂度对比

| 指标 | 当前实现 | 重构后 | 改进 |
|------|----------|--------|------|
| **BM25 行数** | 368 行 | ~150 行 | -59% |
| **Inversearch 行数** | 192 行 | ~120 行 | -38% |
| **直接依赖** | tantivy, IndexManager, Index | Bm25Index, EmbeddedIndex | 解耦 |
| **Writer 管理** | 手动管理 (复杂) | API 层自动管理 | 简化 |
| **持久化** | 部分缺失 | 完整支持 | 完善 |

### 5.2 功能对比

| 功能 | 当前 BM25 | 重构后 BM25 | 当前 Inversearch | 重构后 Inversearch |
|------|-----------|-------------|------------------|-------------------|
| 索引文档 | ✅ | ✅ | ✅ | ✅ |
| 批量索引 | ✅ | ✅ | ✅ | ✅ |
| 搜索 | ✅ | ✅ | ✅ | ✅ |
| 删除 | ✅ | ✅ | ✅ | ✅ |
| commit() | ✅ | ✅ (简化) | ❌ (空实现) | ✅ (真正持久化) |
| rollback() | ✅ | ✅ | ❌ (空实现) | ✅ (通过 API) |
| close() | ✅ | ✅ | ❌ (空实现) | ✅ (保存后关闭) |
| 持久化 | ⚠️ (依赖 commit) | ✅ | ❌ | ✅ (save/load) |

### 5.3 风险对比

| 风险 | 当前实现 | 重构后 |
|------|----------|--------|
| **数据丢失** | 中 (commit 空实现) | 低 (API 层保证) |
| **资源泄漏** | 中 (Writer 未关闭) | 低 (API 层管理) |
| **维护成本** | 高 (复杂逻辑) | 低 (简单调用) |
| **依赖耦合** | 高 (直接依赖 tantivy) | 低 (通过 API) |
| **测试难度** | 高 (需要 mock tantivy) | 低 (mock API) |

---

## 6. 实施建议

### 6.1 立即行动项

1. **高优先级** 🔴:
   - [ ] 修复 Inversearch adapter 的 commit()/close() 空实现
   - [ ] 使用 embedded API 替换 BM25 adapter 的内部组件调用
   - [ ] 移除 tantivy::IndexWriter 的直接依赖

2. **中优先级** 🟡:
   - [ ] 实现 Inversearch adapter 的持久化功能
   - [ ] 简化 BM25 adapter 的 writer 管理逻辑
   - [ ] 移除所有 unwrap() 调用

3. **低优先级** 🟢:
   - [ ] 代码重构和简化
   - [ ] 添加更多单元测试
   - [ ] 编写 API 使用文档

### 6.2 长期改进

1. **架构优化**:
   - 考虑是否需要统一的 SearchEngine API
   - 评估是否需要抽象更多的搜索功能
   - 考虑添加搜索缓存层

2. **性能优化**:
   - 评估批量操作的性能
   - 优化持久化策略
   - 考虑异步索引

3. **功能扩展**:
   - 添加高亮支持
   - 支持多字段搜索
   - 支持自定义评分

---

## 7. 结论

### 7.1 核心发现

1. **当前设计不合理**: `src/search/adapters` 直接使用 crates 的内部组件，违反了封装原则。

2. **应该使用 API 层**: 
   - BM25: 使用 `bm25_service::api::embedded::Bm25Index`
   - Inversearch: 使用 `inversearch_service::api::embedded::EmbeddedIndex`

3. **主要问题**:
   - 直接依赖 tantivy (第三方库)
   - 复杂的 writer 管理逻辑
   - 部分功能空实现 (commit/rollback/close)
   - 缺少持久化支持

### 7.2 建议

**强烈建议**重构 `src/search/adapters` 目录，使用 crates 的 embedded API 层：

1. **解耦**: 不再直接依赖内部组件和第三方库
2. **简化**: 代码量减少 40-60%
3. **完善**: 实现真正的持久化功能
4. **安全**: 移除 unwrap()，改善错误处理

### 7.3 风险评估

**不重构的风险**:
- 🔴 数据丢失风险 (空实现)
- 🟡 维护成本高 (复杂逻辑)
- 🟡 依赖耦合 (难以升级)

**重构的风险**:
- 🟢 低风险 (API 层已测试)
- 🟢 向后兼容 (接口不变)
- 🟢 渐进式迁移 (可并行测试)

---

## 附录

### A. 相关文件

- `src/search/adapters/bm25_adapter.rs`
- `src/search/adapters/inversearch_adapter.rs`
- `crates/bm25/src/api/embedded/index.rs`
- `crates/inversearch/src/api/embedded/index.rs`
- `docs/crates/search_engines_final_analysis.md`

### B. 参考资料

- [BM25 Crate API 设计](crates/bm25/docs/architecture/api_module_design.md)
- [Inversearch API 重构设计](crates/inversearch/docs/architecture/api_refactoring_design.md)
- [Fulltext 集成计划](docs/extend/plan/README.md)

### C. 联系信息

如有问题，请参考 crates 文档或联系维护者。
