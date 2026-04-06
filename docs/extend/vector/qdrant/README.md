# Qdrant 集成文档

> 分析日期: 2026-04-06

本目录包含Qdrant向量数据库集成的完整分析文档。

---

## 文档列表

| 文档 | 描述 |
|------|------|
| [qdrant_api_usage.md](./qdrant_api_usage.md) | Qdrant Rust客户端API使用说明 |
| [qdrant_features.md](./qdrant_features.md) | Qdrant功能概述和特性说明 |
| [qdrant_configuration.md](./qdrant_configuration.md) | Qdrant配置参数详解 |
| [client_crate_architecture.md](./client_crate_architecture.md) | 向量客户端Crate架构分析 |

---

## 快速概览

### Qdrant核心特性

- **高性能向量搜索**: 基于HNSW索引的快速相似度检索
- **Payload过滤**: 支持丰富的元数据过滤条件
- **量化支持**: 标量量化、乘积量化、二进制量化
- **多向量支持**: 单个点可包含多个命名向量
- **分布式部署**: 支持分片和副本

### Rust客户端关键API

```rust
use qdrant_client::{Qdrant, Payload, PointStruct};
use qdrant_client::qdrant::{
    CreateCollectionBuilder, VectorParamsBuilder, Distance,
    UpsertPointsBuilder, SearchPointsBuilder,
};

// 连接
let client = Qdrant::from_url("http://localhost:6334").build()?;

// 创建集合
client.create_collection(
    CreateCollectionBuilder::new("my_collection")
        .vectors_config(VectorParamsBuilder::new(768, Distance::Cosine))
).await?;

// 插入向量
let point = PointStruct::new(1, vec![0.1; 768], payload);
client.upsert_points(UpsertPointsBuilder::new("my_collection", vec![point])).await?;

// 搜索
let results = client.search_points(
    SearchPointsBuilder::new("my_collection", query_vector, 10)
        .with_payload(true)
).await?;
```

---

## 客户端Crate架构建议

**推荐创建独立的 `vector-client` crate**，遵循现有BM25、Inversearch的架构模式：

```
crates/vector-client/
├── src/
│   ├── api/          # API层
│   ├── engine/       # VectorEngine trait + 实现
│   ├── types/        # 类型定义
│   ├── config/       # 配置管理
│   └── error.rs      # 错误定义
└── Cargo.toml
```

核心优势：
- 架构一致性
- 独立测试能力
- 可扩展性（支持其他向量引擎）
- 清晰的模块边界

---

## 与GraphDB集成要点

1. **集合命名**: `space_{space_id}_{tag}_{field}`
2. **Payload设计**: 存储vertex_id、tag_name、关键属性
3. **同步机制**: 复用现有SyncManager
4. **错误处理**: 统一VectorError类型

---

## 参考链接

- [Qdrant官方文档](https://qdrant.tech/documentation/)
- [Qdrant Rust Client](https://docs.rs/qdrant-client/latest/qdrant_client/)
- [Qdrant GitHub](https://github.com/qdrant/qdrant)
