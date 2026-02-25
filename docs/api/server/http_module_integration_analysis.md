# HTTP 模块与核心模块集成分析

## 1. 概述

本文档分析 `src\api\server\http` 目录如何与 api core 模块以及 query、storage、transaction 模块集成。

---

## 2. HTTP 模块结构

`src\api\server\http` 目录采用分层架构设计：

| 文件/目录 | 职责 |
|-----------|------|
| `mod.rs` | 模块入口，导出公共类型 |
| `server.rs` | HTTP 服务器核心，聚合所有 API 实例 |
| `state.rs` | Axum 应用状态，包装 HttpServer |
| `router.rs` | 路由定义 |
| `error.rs` | HTTP 错误处理 |
| `handlers/` | 请求处理器（query, auth, session, health） |
| `middleware/` | 中间件（auth, logging, error） |

---

## 3. 与 API Core 模块的集成

### 3.1 集成方式：组合模式

`HttpServer` 直接组合三个 Core API：

```rust
pub struct HttpServer<S: StorageClient + Clone + 'static> {
    graph_service: Arc<GraphService<S>>,
    query_api: QueryApi<S>,           // 来自 api::core
    txn_api: TransactionApi,          // 来自 api::core
    schema_api: SchemaApi<S>,         // 来自 api::core
    auth_service: PasswordAuthenticator,
    session_manager: Arc<GraphSessionManager>,
    permission_manager: Arc<PermissionManager>,
    stats_manager: Arc<StatsManager>,
}
```

### 3.2 初始化流程

1. 接收 `GraphService`、`StorageClient`、`TransactionManager`、`Config`
2. 用 `storage` 创建 `QueryApi` 和 `SchemaApi`
3. 用 `txn_manager` 创建 `TransactionApi`
4. 从 `GraphService` 获取会话和权限管理器

---

## 4. 与 Query 模块的集成

### 4.1 调用链

```
HTTP Handler → HttpServer.query_api → QueryApi → QueryPipelineManager → 查询执行
```

### 4.2 关键连接点

| 层级 | 文件 | 职责 |
|------|------|------|
| HTTP Handler | `handlers/query.rs` | 接收 HTTP 请求，调用 server |
| Core API | `api/core/query_api.rs` | 封装查询执行逻辑 |
| Query 层 | `query/query_pipeline_manager.rs` | 查询管道管理 |

`QueryApi` 封装了 `QueryPipelineManager`：

```rust
pub struct QueryApi<S: StorageClient + 'static> {
    pipeline_manager: QueryPipelineManager<S>,
}
```

---

## 5. 与 Storage 模块的集成

### 5.1 StorageClient Trait 作为抽象边界

`StorageClient` 定义了存储接口，HTTP 模块通过泛型参数 `S: StorageClient` 与之解耦：

```rust
// state.rs
pub struct AppState<S: StorageClient + Clone + 'static> {
    pub server: Arc<HttpServer<S>>,
}

// server.rs
impl<S: StorageClient + Clone + 'static> HttpServer<S> {
    pub fn new(
        graph_service: Arc<GraphService<S>>,
        storage: Arc<Mutex<S>>,  // 存储客户端
        ...
    ) -> Self
}
```

### 5.2 数据流向

```
HttpServer → QueryApi/SchemaApi → QueryPipelineManager → StorageClient → RedbStorage
```

---

## 6. 与 Transaction 模块的集成

### 6.1 通过 TransactionApi 封装

`TransactionApi` 封装了事务管理器：

```rust
pub struct TransactionApi {
    txn_manager: Arc<TransactionManager>,
}
```

### 6.2 提供的方法

- `begin()` - 开始事务
- `commit()` - 提交事务
- `rollback()` - 回滚事务
- `is_active()` - 检查事务状态

### 6.3 集成路径

```
HttpServer.txn_api → TransactionApi → TransactionManager → redb::Database
```

---

## 7. 架构图

```
┌─────────────────────────────────────────────────────────────┐
│                    HTTP Server Layer                        │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │   Router    │  │   State     │  │      Handlers       │ │
│  │  (路由)      │  │ (AppState)  │  │ (query/auth/session)│ │
│  └──────┬──────┘  └──────┬──────┘  └──────────┬──────────┘ │
│         └─────────────────┴────────────────────┘            │
│                           │                                 │
│                    ┌──────▼──────┐                          │
│                    │  HttpServer  │                          │
│                    │  (聚合核心)   │                          │
│                    └──────┬──────┘                          │
└───────────────────────────┼─────────────────────────────────┘
                            │
        ┌───────────────────┼───────────────────┐
        ▼                   ▼                   ▼
┌───────────────┐  ┌───────────────┐  ┌───────────────┐
│   QueryApi    │  │ TransactionApi│  │   SchemaApi   │
│  (api::core)  │  │  (api::core)  │  │  (api::core)  │
└───────┬───────┘  └───────┬───────┘  └───────┬───────┘
        │                  │                  │
        ▼                  ▼                  ▼
┌───────────────┐  ┌───────────────┐  ┌───────────────┐
│QueryPipeline  │  │   Transaction │  │   Storage     │
│  Manager      │  │   Manager     │  │   (Schema)    │
│  (query)      │  │ (transaction) │  │               │
└───────┬───────┘  └───────┬───────┘  └───────────────┘
        │                  │
        ▼                  ▼
┌───────────────┐  ┌───────────────┐
│ StorageClient │  │   redb::      │
│   (trait)     │  │   Database    │
└───────┬───────┘  └───────────────┘
        │
        ▼
┌───────────────┐
│  RedbStorage  │
│  (storage)    │
└───────────────┘
```

---

## 8. 设计特点

1. **分层清晰**：HTTP → Core API → 业务模块（query/storage/transaction）
2. **泛型抽象**：通过 `StorageClient` trait 实现存储层解耦
3. **Arc<Mutex<>> 共享状态**：支持并发访问
4. **组合优于继承**：HttpServer 组合多个 API 实例而非继承
5. **请求上下文传递**：通过 `AppState` 在 Axum 处理器间共享状态

---

## 9. 相关文件路径

- `src/api/server/http/mod.rs`
- `src/api/server/http/server.rs`
- `src/api/server/http/state.rs`
- `src/api/server/http/router.rs`
- `src/api/core/mod.rs`
- `src/api/core/query_api.rs`
- `src/api/core/transaction_api.rs`
- `src/api/core/schema_api.rs`
- `src/query/query_pipeline_manager.rs`
- `src/storage/storage_client.rs`
- `src/transaction/manager.rs`
