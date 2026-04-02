# 全文检索架构设计决策文档

## 决策概述

**决策日期**: 2026-04-02  
**决策状态**: 已确定  
**相关文档**: 
- [嵌入式全文检索集成设计方案](./fulltext_embedded_design.md)
- [全文检索服务集成设计方案](./ref/fulltext_integration_design.md)
- [全文检索嵌入式集成分析报告](./fulltext_embedding_analysis.md)

---

## 背景

在评估全文检索集成方案时，我们面临两个核心问题：

1. **架构选择**: gRPC 服务架构 vs 嵌入式架构
2. **抽象层次**: 是否需要 `SearchEngine` Trait 抽象？同步逻辑应该放在哪一层？

经过分析，我们确定了以下架构原则。

---

## 核心决策

### 决策 1: 采用嵌入式架构

**结论**: 使用嵌入式架构，将 BM25 和 Inversearch 作为库直接嵌入 GraphDB 进程。

**理由**:
- 符合 GraphDB "轻量级单节点" 的设计目标
- 保持"单可执行文件"的部署优势
- 零网络开销，内存直接调用
- 简化运维，单进程监控

**反对意见及回应**:
- 存储结构差异问题 → 通过程序层面的协调器解决，不在存储层耦合
- 事务协调困难 → 采用最终一致性模型，异步同步

---

### 决策 2: 保留 SearchEngine Trait，但重新定位

**结论**: 保留 `SearchEngine` Trait，但其定位是**"引擎抽象接口"**而非**"存储抽象接口"**。

**澄清**:
- ❌ 不是用于抽象存储层
- ✅ 是用于解耦 BM25 和 Inversearch 两个引擎
- ✅ 便于测试、替换和扩展

**Trait 定义位置**: `src/search/engine.rs`（服务层，非存储层）

---

### 决策 3: 同步逻辑放在程序层面

**结论**: 数据同步逻辑**不上沉到存储层**，而是在**程序层面的协调器**中处理。

**架构层次**:

```
┌─────────────────────────────────────────┐
│  程序层面 (Application Layer)            │
│  ┌─────────────────────────────────┐    │
│  │  FulltextCoordinator            │    │
│  │  - 监听数据变更事件              │    │
│  │  - 调用 SearchEngine 接口        │    │
│  │  - 管理索引生命周期              │    │
│  └─────────────────────────────────┘    │
└─────────────────────────────────────────┘
                    │
┌─────────────────────────────────────────┐
│  服务层 (Service Layer)                  │
│  ┌─────────────────────────────────┐    │
│  │  SearchEngine Trait             │    │
│  │  ├─ Bm25SearchEngine            │    │
│  │  └─ InversearchEngine           │    │
│  └─────────────────────────────────┘    │
└─────────────────────────────────────────┘
                    │
┌─────────────────────────────────────────┐
│  存储层 (Storage Layer)                  │
│  ┌─────────────────────────────────┐    │
│  │  RedbStorage                    │    │
│  │  - 图数据存储 (纯净，无全文耦合)  │    │
│  └─────────────────────────────────┘    │
└─────────────────────────────────────────┘
```

**关键原则**:
1. **存储层纯净**: Redb 存储只负责图数据，不感知全文索引
2. **异步同步**: 数据变更成功后，异步通知全文索引更新
3. **最终一致性**: 全文索引允许短暂滞后
4. **松耦合**: 全文索引是独立服务，通过协调器与存储层交互

---

## 推荐的组件设计

### 1. FulltextCoordinator（程序层面）

```rust
// src/coordinator/fulltext.rs

pub struct FulltextCoordinator {
    engine: Arc<dyn SearchEngine>,
    index_mappings: HashMap<(SpaceId, TagName, FieldName), IndexName>,
}

impl FulltextCoordinator {
    /// 在顶点插入成功后调用（由上层业务逻辑调用）
    pub async fn on_vertex_inserted(&self, vertex: &Vertex) -> Result<()> {
        // 异步索引，不阻塞主流程
        for (field_name, value) in &vertex.properties {
            if let Some(index_name) = self.get_index_name(&vertex.tag, field_name) {
                if let Value::String(text) = value {
                    self.engine.index(
                        &vertex.id.to_string(),
                        text
                    ).await?;
                }
            }
        }
        Ok(())
    }
    
    /// 搜索接口
    pub async fn search(&self, index_name: &str, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        self.engine.search(query, limit).await
    }
}
```

### 2. SearchEngine Trait（服务层）

```rust
// src/search/engine.rs

#[async_trait]
pub trait SearchEngine: Send + Sync + std::fmt::Debug {
    fn name(&self) -> &str;
    async fn index(&self, doc_id: &str, content: &str) -> Result<()>;
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>>;
    async fn delete(&self, doc_id: &str) -> Result<()>;
    async fn commit(&self) -> Result<()>;
    async fn close(&self) -> Result<()>;
}
```

### 3. 引擎适配器（服务层）

```rust
// src/search/adapters/bm25_adapter.rs

pub struct Bm25SearchEngine {
    manager: IndexManager,
    schema: IndexSchema,
    writer: Mutex<IndexWriter>,
}

#[async_trait]
impl SearchEngine for Bm25SearchEngine {
    // 实现 Trait 方法，包装 bm25-service 的 API
}
```

```rust
// src/search/adapters/inversearch_adapter.rs

pub struct InversearchEngine {
    index: Mutex<Index>,
    persistence_path: Option<PathBuf>,
}

#[async_trait]
impl SearchEngine for InversearchEngine {
    // 实现 Trait 方法，包装 inversearch-service 的 API
}
```

---

## 包路径更新

由于包已从 `ref/` 迁移到 `crates/`，Cargo.toml 配置更新如下：

```toml
[dependencies]
# BM25 - 纯库模式（不含 service feature）
bm25-service = { path = "../crates/bm25", default-features = false }

# Inversearch - 纯库模式（不含 service feature）
inversearch-service = { path = "../crates/inversearch", default-features = false, features = ["cache", "store"] }
```

---

## 数据流设计

### 创建全文索引

```
用户
  │ CREATE FULLTEXT INDEX idx_content ON Post(content) USING bm25
  ▼
查询引擎
  │ 1. 解析 SQL
  │ 2. 创建索引元数据 (Redb)
  ▼
FulltextCoordinator
  │ 调用 engine = Bm25SearchEngine::open_or_create(path)
  │ 保存 engine 到 registry
  ▼
返回成功
```

### 插入数据同步

```
用户
  │ INSERT VERTEX Post(content) VALUES "图数据库文章"
  ▼
存储层 (Redb)
  │ 1. 写入图数据 (事务)
  │ 2. 提交事务成功
  │ 3. 返回成功给用户
  ▼
查询引擎（异步，不阻塞）
  │ 触发 FulltextCoordinator::on_vertex_inserted()
  ▼
FulltextCoordinator
  │ 检查 Post.content 是否有全文索引
  │ 是 → 调用 engine.index(doc_id, content)
  ▼
Bm25SearchEngine
  │ 添加文档到 Tantivy IndexWriter
  │ (定期 commit，非每次操作)
  ▼
异步完成
```

### 全文搜索

```
用户
  │ MATCH (p:Post) WHERE p.content MATCH "图数据库"
  ▼
查询引擎
  │ 1. 识别全文搜索条件
  │ 2. 调用 FulltextCoordinator::search()
  ▼
FulltextCoordinator
  │ 获取对应引擎
  │ 调用 engine.search("图数据库", limit)
  ▼
Bm25SearchEngine
  │ Tantivy 搜索，返回 doc_ids
  ▼
查询引擎
  │ 根据 doc_ids 查询完整顶点数据 (Redb)
  ▼
返回结果给用户
```

---

## 关键设计原则总结

| 原则 | 说明 |
|------|------|
| **存储层纯净** | Redb 只负责图数据，不耦合全文索引逻辑 |
| **程序层协调** | 同步逻辑放在 `FulltextCoordinator`，由上层调用 |
| **服务层抽象** | `SearchEngine` Trait 用于引擎解耦，非存储抽象 |
| **异步非阻塞** | 全文索引更新异步执行，不阻塞主事务 |
| **最终一致性** | 允许全文索引短暂滞后，定期同步 |
| **引擎可替换** | 通过 Trait 抽象，BM25 和 Inversearch 可互换 |

---

## 实施路径

1. **Phase 1**: 创建 `SearchEngine` Trait 和两个适配器实现
2. **Phase 2**: 实现 `FulltextCoordinator` 协调器
3. **Phase 3**: 在查询引擎中集成全文搜索语法
4. **Phase 4**: 实现数据变更的异步同步机制
5. **Phase 5**: 测试和性能优化

---

## 附录：与原始方案的对比

| 方面 | 原始嵌入式方案 | 调整后方案 |
|------|---------------|-----------|
| Trait 定位 | 存储层抽象 | 服务层引擎抽象 |
| 同步逻辑 | 存储层耦合 (`on_vertex_change`) | 程序层协调器 (`FulltextCoordinator`) |
| 存储层职责 | 包含全文索引管理 | 纯净，只负责图数据 |
| 耦合度 | 高 | 低 |
| 可测试性 | 低 | 高 |
