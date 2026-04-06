# 向量检索集成分析文档

> 分析日期: 2026-04-06

本目录包含为GraphDB添加向量检索支持的完整分析文档。

---

## 文档列表

| 文档 | 描述 |
|------|------|
| [vector_search_integration_analysis.md](./vector_search_integration_analysis.md) | 向量检索集成总体分析，包括架构设计、实现计划 |
| [qdrant_adapter_implementation.md](./qdrant_adapter_implementation.md) | Qdrant适配器的详细实现细节 |
| [query_integration.md](./query_integration.md) | 查询引擎集成方案，包括SQL语法扩展 |
| [data_sync_mechanism.md](./data_sync_mechanism.md) | 数据同步机制设计 |

---

## 快速概览

### 架构设计

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
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│                     Sync Layer (复用)                        │
│  SyncManager + TaskBuffer + RecoveryManager                 │
└─────────────────────────────────────────────────────────────┘
```

### 核心组件

| 组件 | 职责 | 对应全文检索组件 |
|------|------|-----------------|
| `VectorEngine` Trait | 向量搜索引擎抽象接口 | `SearchEngine` Trait |
| `VectorIndexManager` | 向量索引生命周期管理 | `FulltextIndexManager` |
| `VectorCoordinator` | 数据变更协调 | `FulltextCoordinator` |
| `QdrantAdapter` | Qdrant客户端适配器 | `Bm25SearchEngine` |
| `SyncTask` (扩展) | 向量同步任务 | 复用现有 |

### SQL语法示例

```sql
-- 创建向量索引
CREATE VECTOR INDEX idx_embedding 
ON Document(embedding) 
WITH (vector_size = 768, distance = 'cosine');

-- 向量搜索
SEARCH VECTOR idx_embedding
WITH vector = [0.1, 0.2, ...]
LIMIT 10
RETURN id, content, score;

-- 与图查询结合
MATCH (d:Document)
WHERE d.embedding SIMILAR TO [0.1, ...] WITH threshold = 0.8
RETURN d
LIMIT 10;
```

---

## 实现阶段

| 阶段 | 内容 | 预计时间 |
|------|------|---------|
| Phase 1 | 核心接口和Qdrant适配器 | 3-4天 |
| Phase 2 | 索引管理器 | 2-3天 |
| Phase 3 | 协调器和同步扩展 | 2-3天 |
| Phase 4 | 查询引擎集成 | 3-4天 |
| Phase 5 | 嵌入服务集成（可选） | 2-3天 |

---

## 依赖项

```toml
[dependencies]
qdrant-client = "1.7"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
async-trait = "0.1"
thiserror = "1"
```

---

## 与全文检索的对比

| 特性 | 全文检索 | 向量检索 |
|------|---------|---------|
| 搜索方式 | 关键词匹配 | 向量相似度 |
| 索引类型 | 倒排索引 | HNSW |
| 查询类型 | 文本查询 | 向量查询 |
| 距离度量 | BM25评分 | Cosine/Euclidean/Dot |
| 存储引擎 | Tantivy/Inversearch | Qdrant |
| 同步机制 | 相同（复用SyncManager） | 相同（复用SyncManager） |

---

## 参考文档

- [Qdrant Rust Client](https://github.com/qdrant/rust-client)
- [Qdrant Documentation](https://qdrant.tech/documentation/)
- 现有全文检索模块: `src/search/`, `src/coordinator/fulltext.rs`, `src/sync/`
