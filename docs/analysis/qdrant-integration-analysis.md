# Qdrant 集成分析与重构建议

## 一、当前实现概览

当前 `vector-client` crate（`crates/vector-client/`）是一个**多后端向量数据库客户端抽象层**，包装了上游 `qdrant-client` v1.17 的 gRPC 客户端。

### 架构分层

```
vector-client crate
├── engine/VectorEngine trait           ← 异步抽象接口（24 方法）
├── engine/qdrant/QdrantEngine          ← qdrant-client gRPC 实现
├── manager/VectorManager               ← 生命周期管理 + 操作转发
├── api/embedded/VectorClient           ← 上层 Fluent API
├── types/                              ← 类型定义
├── embedding/                          ← 嵌入服务（HTTP + llama.cpp）
└── config/                             ← 配置体系

graphDB 主项目集成
├── src/api/mod.rs                      ← 初始化 VectorManager + VectorSyncCoordinator
├── src/sync/manager.rs                 ← SyncManager 统一管理全文+向量同步
├── src/sync/vector_sync.rs             ← VectorSyncCoordinator 处理顶点/边变更
└── src/sync/external_index/            ← ExternalIndexClient 适配器层（含重试+死信队列）
```

---

## 二、现有问题

### 2.1 存储集成问题

| 问题 | 严重度 | 位置 | 说明 |
|------|--------|------|------|
| `enabled=false` 仍连接 Qdrant | **致命** | `manager/mod.rs:48-54` | 禁用时仍创建 QdrantEngine 并尝试连接 localhost:6333 |
| `list_payload_indexes` 空实现 | **高** | `engine/qdrant/mod.rs:572-579` | 始终返回空数组，未调用 Qdrant API |
| Manhattan 距离映射错误 | **高** | `engine/qdrant/config.rs:16` | L1 → L2 静默映射，语义不一致 |

### 2.2 搜索集成问题

| 问题 | 严重度 | 位置 | 说明 |
|------|--------|------|------|
| 搜索结果 vector 被丢弃 | **高** | `engine/qdrant/utils.rs:58` | `vector: None`，即使请求 `with_vector=true` |
| `search_batch` 串行 | **中** | `engine/qdrant/mod.rs:365-368` | 未使用 Qdrant SearchBatch gRPC，多查询性能差 |

### 2.3 同步处理问题

| 问题 | 严重度 | 位置 | 说明 |
|------|--------|------|------|
| 边删除未清理向量索引 | **高** | `src/sync/manager.rs:220-263` | 只处理了全文索引，忽略向量索引 |
| IndexMetadataWrapper 前缀解析错误 | **中** | `src/sync/vector_sync.rs:856-862` | 解析 `space` 而非 `space_vec`，且位置分割脆弱 |

---

## 三、参考实现分析（`ref/qdrant`）

`ref/qdrant` 目录包含一个更成熟的 Qdrant HTTP REST 客户端实现：

### 架构

```
ref/qdrant/
├── client.rs           ← QdrantClient 门面（协调各操作模块）
├── config.rs           ← 配置（HNSW、WAL、量化、预置方案）
├── error.rs            ← 错误分类（可重试/临时/永久）
├── types.rs            ← 类型（VectorPoint、SearchQuery、SearchResult 等）
├── estimator.rs        ← 集合大小估算 + 预置方案推荐
├── retrieval.rs        ← VectorRetrievalTrait 实现（混合/稠密/稀疏搜索）
├── scheduler.rs        ← 配置升级调度器
├── upgrade.rs          ← 配置升级服务（自动扩缩容）
├── operations/
│   ├── collection.rs   ← 集合 CRUD + payload 索引
│   ├── points.rs       ← Point upsert/delete（含并发限制）
│   ├── search.rs       ← 搜索（过滤、阈值、HNSW ef）
│   └── summary.rs      ← 摘要集合操作
```

### 亮点

1. **HTTP REST 直连** — 无外部 gRPC 依赖，仅需 `reqwest` + `serde_json`
2. **完整的 enabled/disabled 处理** — `QdrantError::Disabled`，各操作前检查
3. **错误分类体系** — `is_retryable()` / `is_transient()` / `is_permanent()`
4. **混合搜索** — 支持 dense + sparse 向量混合搜索（RRF fusion）
5. **预置方案** — Tiny/Small/Medium/Large 随数据量自动调整配置
6. **配置升级** — 后台调度器自动检测并升级集合配置
7. **并发控制** — `Semaphore` 限制并发 upsert
8. **大小估算** — CollectionSizeEstimator 预估存储和内存

---

## 四、是否改用 HTTP REST 的分析

### 当前 gRPC 方式的问题

1. **`qdrant-client` 是巨型依赖** — 包含 protobuf 生成代码 + gRPC 运行时
2. **功能缺失难修补** — `list_payload_indexes` 等缺失需要深入上游 API
3. **调试困难** — gRPC 二进制协议，难以抓包 Debug
4. **版本锁定** — 上游 v1.17，落后于 Qdrant 最新功能
5. **测试困难** — 需要运行 Qdrant gRPC 实例

### HTTP REST 方案的优势

1. **极简依赖** — 只需 `reqwest` + `serde_json` + `tokio`
2. **完整控制** — 自定义请求体、处理任意响应格式
3. **易于调试** — JSON over HTTP，可用 curl 验证
4. **零版本锁定** — 兼容所有 Qdrant 版本（API 稳定）
5. **易于测试** — 可用 mock server 或录播
6. **无重量级编译** — 避免 protobuf 编译开销

### 结论：建议采用 HTTP REST 方案

**核心理由**：本项目是对外无依赖的单机轻量级图数据库，引入 gRPC + protobuf 的复杂依赖与项目目标相悖。`ref/qdrant` 的 HTTP REST 实现恰好符合"最小外部依赖、单一可执行文件"的理念。

---

## 五、重构建议

### 5.1 替换方向

将 `crates/vector-client` 的 `engine/qdrant` 实现从 gRPC 替换为 HTTP REST，保留上层抽象不变：

```
保留的层：                   替换的层：
VectorEngine trait  ← 不变    QdrantEngine (gRPC)  →  QdrantHttpEngine (REST)
VectorManager       ← 不变    
VectorSyncCoordinator ← 不变  
ExternalIndexClient   ← 不变  
```

### 5.2 关键设计决策

1. **适配 ref/qdrant 但去耦合** — 提取核心逻辑，剥离对主项目 config 的依赖
2. **保留 VectorEngine trait** — 保持上层集成代码不变
3. **重写 QdrantEngine 实现** — 基于 `reqwest::Client` + REST API
4. **嵌入 ref 的预置方案和估算** — 作为可选增值功能
5. **删除 embedding 模块** — 与本项目无关，应独立维护

### 5.3 优先级

1. **P0**: 修复 `enabled=false` 连接问题（当前 gRPC 方案下即可修复）
2. **P0**: 修复搜索结果 vector 丢弃
3. **P0**: 修复边删除向量索引清理
4. **P1**: 实现 `list_payload_indexes`（当前 gRPC 方案下即可修复）
5. **P1**: 修复 Manhattan 距离映射
6. **P1**: 修复 IndexMetadataWrapper 前缀解析
7. **P2**: 将 QdrantEngine 替换为 HTTP REST 方案
8. **P2**: 引入预置方案和配置升级
