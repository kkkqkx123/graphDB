# vector-client 多后端扩展计划

## 背景

`vector-client` 目前仅支持 Qdrant（HTTP + gRPC），但需支持 Pinecone、Milvus 等更多向量数据库。当前骨架（`VectorEngine` trait + `EngineType` 枚举 + `dyn` dispatch）合理，但有三处阻碍扩展的问题。

## 核心问题总览

| 问题 | 影响 | 修改范围 |
|------|------|---------|
| trait 无默认方法，新后端必须实现全部 18 个方法 | 扩后端成本高 | engine/mod.rs + error.rs |
| VectorManager 用 cfg feature 优先级代替 config.engine | EngineType 枚举被 bypass | manager/mod.rs |
| feature flag 无分组策略 | 多后端时爆炸 | Cargo.toml |

---

## Phase 1：Trait 默认方法（预备）

### 目标
新后端只需实现核心方法，其余可降级返回错误。

### 改动

**1.1 error.rs** — 新增 `NotSupported` 变体

```rust
#[error("Operation not supported by this engine: {0}")]
NotSupported(String),
```

**1.2 engine/mod.rs** — 为 VectorEngine trait 中非通用方法添加默认实现

添加默认实现的方法：
- `search_batch` → 默认遍历调用 `search`
- `delete_by_filter` → 默认 `Err(NotSupported)`
- `set_payload` → 默认 `Err(NotSupported)`
- `delete_payload` → 默认 `Err(NotSupported)`
- `scroll` → 默认 `Err(NotSupported)`
- `create_payload_index` → 默认 `Err(NotSupported)`
- `delete_payload_index` → 默认 `Err(NotSupported)`
- `list_payload_indexes` → 默认 `Err(NotSupported)`

必须保持强制实现的方法：
- `name, version, health_check`
- `create_collection, delete_collection, collection_exists, collection_info`
- `upsert, upsert_batch`
- `delete, delete_batch`
- `search`
- `get, get_batch`
- `count`

---

## Phase 2：统一运行时 dispatch（主要）

### 目标
消除 VectorManager 的编译时 feature 优先级逻辑，改为与 VectorClient 一致的 `match config.engine` 运行时选择。

### 改动

**2.1 manager/mod.rs**

当前三个 `create_engine` 函数：

```rust
// 问题：完全不看 config.engine
#[cfg(feature = "qdrant-grpc")]
async fn create_engine(config: VectorClientConfig) -> Result<Arc<dyn VectorEngine>> {
    QdrantGrpcEngine::new(config).await  // 硬编码 gRPC
}
```

改为统一的 `match config.engine` 分发：

```rust
async fn create_engine(config: VectorClientConfig) -> Result<Arc<dyn VectorEngine>> {
    match config.engine {
        EngineType::Qdrant => {
            // 优先 gRPC（更高效），回退 HTTP
            #[cfg(feature = "qdrant-grpc")]
            {
                let engine = QdrantGrpcEngine::new(config).await?;
                return Ok(Arc::new(engine));
            }
            #[cfg(all(feature = "qdrant-http", not(feature = "qdrant-grpc")))]
            {
                let engine = QdrantEngine::new(config).await?;
                return Ok(Arc::new(engine));
            }
            #[cfg(not(any(feature = "qdrant-http", feature = "qdrant-grpc")))]
            {
                Err(VectorClientError::EngineNotAvailable(
                    "Qdrant engine feature not enabled".to_string(),
                ))
            }
        }
    }
}
```

这样 VectorManager 和 VectorClient 行为一致，EngineType 枚举真正生效。

---

## Phase 3：Feature Flag 分组（可选）

### 目标
为新后端提供清晰的特征命名空间，避免 `pinecone-grpc`、`milvus-http` 等特征扁平爆炸。

### 改动

**3.1 Cargo.toml**

```toml
[features]
default = ["qdrant"]
qdrant = ["qdrant-http"]
qdrant-http = []
qdrant-grpc = ["dep:tonic", "dep:prost", "dep:prost-types"]
# 未来扩展：
# pinecone = []
# milvus = ["milvus-grpc"]
# milvus-grpc = ["dep:tonic", "dep:..."]
```

`qdrant` 作为后端级特征，默认启用 HTTP 协议。用户选择 `qdrant-grpc` 替换它。

---

## 扩展新后端的标准化步骤

1. 在 `EngineType` 添加变体（如 `Pinecone`）
2. 创建 `src/engine/{backend}/mod.rs`，实现 `VectorEngine`
3. 在 `Cargo.toml` 添加 feature flag
4. 在 `engine/mod.rs` 添加 `#[cfg(feature = ...)]` 注册
5. 在 `VectorClient::new` 和 `VectorManager::create_engine` 的 `match` 中添加新分支
6. 对不支持操作，trait 默认方法自动返回 `NotSupported`
