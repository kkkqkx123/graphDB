# GraphDB 向量检索与全文检索隔离机制分析报告

## 1. 概述

本报告基于 `docs\extend\ref\vector_fulltext_isolation_research.md` 中的最佳实践，对 GraphDB 当前实现的隔离机制进行深入分析，并提出改进建议。

## 2. 当前隔离机制现状

### 2.1 整体架构隔离

| 组件 | 隔离方式 | 实现位置 |
|------|----------|----------|
| 数据类型 | `IndexData` 枚举区分 | `src/sync/external_index/trait_def.rs:7-10` |
| 客户端 | `FulltextClient` / `VectorClient` 独立 trait 实现 | `src/sync/external_index/` |
| 处理器 | 分离的 `DashMap` | `src/sync/coordinator/coordinator.rs:36-37` |
| 存储路径 | 分离目录 | `fulltext` vs `vector` |

### 2.2 全文检索隔离（BM25 / Inversearch）

```
FulltextIndexManager
├── engines: DashMap<IndexKey, Arc<dyn SearchEngine>>
├── metadata: DashMap<IndexKey, IndexMetadata>
└── 索引路径: {base_path}/{space_id}_{tag_name}_{field_name}
```

**当前实现特点**：
- 统一通过 `FulltextIndexManager` 管理，支持 BM25 和 Inversearch 两种引擎
- 每个索引使用独立的存储路径，路径格式：`{base_path}/{space_id}_{tag_name}_{field_name}`
- 引擎通过 `SearchEngineFactory` 按需创建，共享管理器实例

**隔离评估**：
- ✅ **逻辑隔离**：通过 `IndexKey` (space_id, tag_name, field_name) 区分
- ✅ **存储隔离**：每个索引独立目录/文件
- ⚠️ **资源隔离**：两种引擎共享 `FulltextIndexManager`，无资源配额限制

### 2.3 向量检索隔离（Qdrant）

**collection 命名策略**（`src/sync/external_index/vector_client.rs:132-134`）：
```rust
fn collection_name(&self) -> String {
    format!("{}_{}_{}", self.space_id, self.tag_name, self.field_name)
}
```

**当前实现特点**：
- 通过 collection 名称前缀实现逻辑隔离
- Qdrant 作为远程服务，GraphDB 通过 HTTP API 与之交互
- 使用 `VectorManager` 统一管理 collection 的创建和访问

**隔离评估**：
- ✅ **Collection 级隔离**：不同 space 的数据落入不同 collection
- ⚠️ **服务端依赖**：隔离能力依赖 Qdrant 服务端配置
- ⚠️ **命名冲突风险**：无服务端命名空间强制约束

## 3. 与参考文档最佳实践对比

| 维度 | PostgreSQL/Neo4j 最佳实践 | GraphDB 当前实现 | 差距 |
|------|--------------------------|------------------|------|
| **命名空间** | Schema/Database 级别硬隔离 | space_id 逻辑隔离 | 中等差距 |
| **存储隔离** | Tablespace/独立目录 | 分离目录 | ✅ 已实现 |
| **访问控制** | Schema/Database 级权限 | 无 | 需增强 |
| **资源配额** | 可配置存储限额 | 无 | 需增强 |

## 4. 改进建议

### 4.1 BM25 / Inversearch 隔离增强

**建议方案**：为每种引擎类型维护独立的 manager 实例

```rust
// 方案 A：按引擎类型分离 Manager
pub struct FulltextIndexManager {
    bm25_engines: DashMap<IndexKey, Arc<Bm25SearchEngine>>,
    inversearch_engines: DashMap<IndexKey, Arc<InversearchEngine>>,
    // ...
}

// 方案 B：引入引擎级别隔离层（推荐）
pub struct SearchNamespaceManager {
    namespaces: DashMap<space_id, Arc<EngineNamespace>>,
}

pub struct EngineNamespace {
    bm25: Option<Arc<FulltextIndexManager>>,
    inversearch: Option<Arc<FulltextIndexManager>>,
}
```

**评估**：当前实现对单节点场景已足够，引擎分离可在多租户场景下作为优化项。

### 4.2 Qdrant 隔离增强

**问题**：Qdrant 作为外部服务，GraphDB 无法直接控制服务端的访问控制。

**建议方案**：

#### 方案 A：Collection 命名空间前缀（当前已实现，建议增强）

```rust
// 当前命名格式
fn collection_name(&self) -> String {
    format!("{}_{}_{}", self.space_id, self.tag_name, self.field_name)
}

// 建议增加统一前缀和版本隔离
fn collection_name(&self) -> String {
    format!("graphdb_{}_{}_{}_{}", self.version, self.space_id, self.tag_name, self.field_name)
}
```

#### 方案 B：Qdrant 访问控制集成

若 Qdrant 服务支持 RBAC（如 Qdrant Cloud），可在配置中指定：
```yaml
vector:
  qdrant:
    url: "https://xxx.qdrant.io"
    api_key: "${QDRANT_API_KEY}"
    collection_prefix: "graphdb_"  # 强制前缀
    tenant_isolation: true          # 启用租户隔离
```

#### 方案 C：本地代理层

在 GraphDB 内部实现 Qdrant 请求代理，强制注入 space_id 检查：
```rust
pub struct QdrantProxy {
    inner: QdrantClient,
    space_validator: SpaceAccessValidator,
}

impl QdrantProxy {
    async fn search(&self, space_id: u64, request: SearchRequest) -> Result<...> {
        // 验证 request 中的 collection 是否属于该 space_id
        self.space_validator.validate(space_id, &request.collection)?;
        self.inner.search(request).await
    }
}
```

## 5. 结论

### 5.1 当前评估

| 组件 | 隔离充分性 | 说明 |
|------|-----------|------|
| 全文检索（本地引擎） | ✅ 充分 | 文件级隔离满足单节点需求 |
| 向量检索（Qdrant） | ⚠️ 基本满足 | 依赖 collection 命名规范，无服务端强制约束 |

### 5.2 改进优先级

| 优先级 | 改进项 | 说明 |
|--------|--------|------|
| **P1** | Qdrant 命名增强 | 增加统一前缀，避免测试/生产环境冲突 |
| **P2** | Qdrant 代理验证层 | 防止误操作导致的跨 space 访问 |
| **P3** | 多引擎资源配额 | 多租户场景下的资源管理 |

### 5.3 最终建议

1. **短期**：保持当前架构，通过配置规范和代码审查确保 `collection_name()` 的正确使用
2. **中期**：为 Qdrant 客户端增加 namespace 前缀和访问验证
3. **长期**：考虑引入独立的向量存储服务（如 Milvus），实现完全自主的隔离控制

---

*生成时间：2026-04-27*