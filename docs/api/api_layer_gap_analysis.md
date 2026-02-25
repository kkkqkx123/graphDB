# GraphDB API 层功能缺口分析

## 概述

本文档分析当前 `src/api` 层在支持嵌入式数据库架构时需要补充的功能，为实施 `embedded` 子模块提供具体指导。

---

## 当前 API 层架构回顾

### 现有模块职责

```
src/api/
├── mod.rs                    # 服务启动入口
├── service/                  # 网络服务层
│   ├── graph_service.rs      # GraphService：HTTP/RPC 服务入口
│   ├── query_processor.rs    # QueryEngine：查询执行（已存在）
│   ├── authenticator.rs      # 用户认证
│   ├── permission_manager.rs # 权限管理
│   └── stats_manager.rs      # 统计监控
└── session/                  # 网络会话层
    ├── client_session.rs     # ClientSession：带网络概念的会话
    ├── session_manager.rs    # GraphSessionManager：连接池管理
    └── query_manager.rs      # 查询管理
```

### 现有核心能力

| 组件 | 现有功能 | 位置 |
|-----|---------|------|
| 查询执行 | `QueryEngine::execute()` | `service/query_processor.rs` |
| 查询管道 | `QueryPipelineManager` | `query/query_pipeline_manager.rs` |
| 事务管理 | `TransactionManager` | `transaction/manager.rs` |
| 会话管理 | `ClientSession` | `session/client_session.rs` |

---

## 功能缺口分析

### 缺口 1：同步查询执行接口

**问题描述：**
现有 `QueryEngine::execute()` 是异步的（`fn`），但嵌入式 API 需要同步接口以简化使用。

**现有代码：**
```rust
// src/api/service/query_processor.rs
impl<S: StorageClient + Clone + 'static> QueryEngine<S> {
    pub fn execute(&mut self, rctx: RequestContext) -> ExecutionResponse {
        // 异步执行
        match self.pipeline_manager.execute_query_with_space(...) {
            // ...
        }
    }
}
```

**需要的补充：**
```rust
// 嵌入式层需要同步包装
impl QueryEngine {
    pub fn execute_sync(&mut self, query: &str, space: Option<&str>) -> EmbeddedResult<QueryResult> {
        // 创建运行时或阻塞执行
        let rt = tokio::runtime::Handle::try_current()
            .unwrap_or_else(|_| {
                // 创建新的运行时
                tokio::runtime::Runtime::new().unwrap().handle().clone()
            });
        rt.block_on(self.execute_async(query, space))
    }
}
```

**实施建议：**
- 方案A：在 `embedded` 层创建同步包装器
- 方案B：在 `query` 层添加同步执行模式
- **推荐方案A**，保持核心引擎异步，嵌入式层按需同步

---

### 缺口 2：结构化结果集处理

**问题描述：**
现有 `ExecutionResponse` 只返回字符串，嵌入式 API 需要结构化访问结果。

**现有代码：**
```rust
// src/api/service/query_processor.rs
#[derive(Debug)]
pub struct ExecutionResponse {
    pub result: Result<String, String>,  // 只有字符串结果
    pub latency_us: u64,
}
```

**需要的补充：**
```rust
// embedded/result.rs
pub struct QueryResult {
    columns: Vec<String>,
    rows: Vec<Row>,
    metadata: ResultMetadata,
}

pub struct Row {
    values: HashMap<String, Value>,
}

impl Row {
    pub fn get_string(&self, column: &str) -> Option<String>;
    pub fn get_int(&self, column: &str) -> Option<i64>;
    pub fn get_vertex(&self, column: &str) -> Option<&Vertex>;
    // ...
}
```

**依赖分析：**
- 需要访问 `query::executor::base::ExecutionResult` 的内部结构
- 需要转换 `core::value::Value` 到嵌入式友好的接口

---

### 缺口 3：参数化查询支持

**问题描述：**
现有接口不支持参数绑定，需要解析 `RequestContext.parameters` 字符串映射。

**现有代码：**
```rust
// 参数是字符串 HashMap
pub struct RequestContext {
    pub parameters: std::collections::HashMap<String, String>,
    // ...
}
```

**需要的补充：**
```rust
// embedded/session.rs
pub fn execute_with_params(
    &self,
    query: &str,
    params: &HashMap<String, crate::core::value::Value>,  // 类型化参数
) -> EmbeddedResult<QueryResult>;

// 支持参数绑定语法：$param_name
```

**依赖分析：**
- 需要修改或扩展查询解析器以支持参数占位符
- 需要类型检查和转换逻辑

---

### 缺口 4：预编译语句接口

**问题描述：**
现有架构没有预编译语句机制，每次查询都重新解析和生成计划。

**需要的补充：**
```rust
// embedded/statement.rs
pub struct PreparedStatement {
    query_plan: Arc<ExecutionPlan>,
    parameter_types: HashMap<String, DataType>,
}

impl PreparedStatement {
    pub fn bind(&mut self, name: &str, value: Value) -> EmbeddedResult<()>;
    pub fn execute(&self) -> EmbeddedResult<QueryResult>;
    pub fn reset(&mut self);
}
```

**依赖分析：**
- 需要访问 `query::planner::ExecutionPlan`
- 需要计划缓存机制（`PlanCache` 已存在，可复用）
- 需要参数类型推断

---

### 缺口 5：批量操作接口

**问题描述：**
现有接口没有批量插入优化，大量小事务性能差。

**需要的补充：**
```rust
// embedded/batch.rs
pub struct BatchInserter {
    storage: Arc<dyn StorageClient>,
    batch_size: usize,
    vertex_buffer: Vec<Vertex>,
    edge_buffer: Vec<Edge>,
}

impl BatchInserter {
    pub fn add_vertex(&mut self, vertex: Vertex) -> &mut Self;
    pub fn add_edge(&mut self, edge: Edge) -> &mut Self;
    pub fn execute(&mut self) -> EmbeddedResult<BatchResult>;
}
```

**依赖分析：**
- 需要直接操作 `storage::StorageClient`
- 需要批量事务优化
- 可能需要绕过查询引擎直接写入存储

---

### 缺口 6：简化会话管理

**问题描述：**
现有 `ClientSession` 包含太多网络相关概念，不适合嵌入式。

**现有代码对比：**
```rust
// 现有 ClientSession（网络版）
pub struct ClientSession {
    session: Arc<RwLock<Session>>,      // 含 user_name、graph_addr
    roles: Arc<RwLock<HashMap<i64, RoleType>>>,  // 权限
    idle_start_time: Arc<RwLock<Instant>>,       // 网络超时
    contexts: Arc<RwLock<HashMap<u32, String>>>, // 查询上下文
    current_transaction: Arc<RwLock<Option<TransactionId>>>,
    // ...
}

// 需要的 EmbeddedSession
pub struct EmbeddedSession {
    storage: Arc<dyn StorageClient>,
    transaction_manager: Arc<TransactionManager>,
    query_engine: Arc<QueryEngine>,
    current_space: Option<String>,
    auto_commit: bool,
}
```

**实施建议：**
- 完全新建 `embedded/session.rs`，不修改现有 `ClientSession`
- 复用事务管理逻辑（调用 `TransactionManager`）

---

### 缺口 7：错误类型转换

**问题描述：**
现有错误类型分散在各模块，需要统一的嵌入式错误接口。

**现有错误类型：**
```rust
// 分散在各模块的错误类型
crate::core::error::QueryError
crate::storage::StorageError
crate::transaction::TransactionError
// ...
```

**需要的补充：**
```rust
// embedded/error.rs
#[derive(Error, Debug)]
pub enum EmbeddedError {
    #[error("查询执行失败: {0}")]
    QueryExecutionFailed(String),
    #[error("存储错误: {0}")]
    StorageError(String),
    #[error("事务错误: {0}")]
    TransactionError(String),
    // ...
}

// 实现 From 转换
impl From<crate::core::error::QueryError> for EmbeddedError { ... }
impl From<crate::storage::StorageError> for EmbeddedError { ... }
```

---

### 缺口 8：数据库生命周期管理

**问题描述：**
现有架构没有数据库实例概念，直接操作存储引擎。

**需要的补充：**
```rust
// embedded/database.rs
pub struct GraphDatabase {
    storage: Arc<dyn StorageClient>,
    transaction_manager: Arc<TransactionManager>,
    query_engine: Arc<QueryEngine>,
    config: DatabaseConfig,
}

impl GraphDatabase {
    pub fn open(path: impl AsRef<Path>) -> EmbeddedResult<Self>;
    pub fn open_in_memory() -> EmbeddedResult<Self>;
    pub fn close(self) -> EmbeddedResult<()>;
    pub fn session(&self) -> EmbeddedResult<EmbeddedSession>;
}
```

**依赖分析：**
- 需要初始化 `StorageClient`、`TransactionManager`、`QueryEngine`
- 需要配置管理
- 需要资源清理逻辑

---

### 缺口 9：C FFI 绑定层

**问题描述：**
需要为其他语言提供 C 兼容接口。

**需要的补充：**
```rust
// embedded/ffi/mod.rs
#[no_mangle]
pub extern "C" fn graphdb_open(path: *const c_char, db: *mut *mut GraphDatabase) -> c_int;

#[no_mangle]
pub extern "C" fn graphdb_execute(
    session: *mut EmbeddedSession,
    query: *const c_char,
    result: *mut *mut QueryResult
) -> c_int;

// ... 更多 C API
```

**依赖分析：**
- 需要 `libc` crate
- 需要类型转换层（Rust 类型 ↔ C 类型）
- 需要内存管理（所有权转移）

---

## 功能缺口汇总表

| 缺口 | 优先级 | 复杂度 | 依赖模块 | 实施位置 |
|-----|-------|-------|---------|---------|
| 同步查询接口 | P0 | 中 | `query` | `embedded/session.rs` |
| 结构化结果集 | P0 | 中 | `query`, `core` | `embedded/result.rs` |
| 简化会话管理 | P0 | 低 | `transaction` | `embedded/session.rs` |
| 数据库生命周期 | P0 | 中 | `storage`, `transaction`, `query` | `embedded/database.rs` |
| 错误类型统一 | P0 | 低 | 所有模块 | `embedded/error.rs` |
| 参数化查询 | P1 | 高 | `query`（需扩展） | `embedded/session.rs` |
| 预编译语句 | P1 | 高 | `query`, `planner` | `embedded/statement.rs` |
| 批量操作 | P1 | 中 | `storage` | `embedded/batch.rs` |
| C FFI 绑定 | P2 | 高 | 所有嵌入式模块 | `embedded/ffi/` |

---

## 实施依赖关系

```
┌─────────────────────────────────────────────────────────────┐
│                      实施阶段 1 (P0)                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │  error.rs    │  │  types.rs    │  │  database.rs │      │
│  │  (错误定义)   │  │  (类型定义)   │  │  (数据库实例) │      │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘      │
│         └─────────────────┴─────────────────┘               │
│                           │                                 │
│                           ▼                                 │
│                  ┌─────────────────┐                       │
│                  │   session.rs    │                       │
│                  │ (简化会话管理)   │                       │
│                  └────────┬────────┘                       │
│                           │                                 │
│                           ▼                                 │
│                  ┌─────────────────┐                       │
│                  │   result.rs     │                       │
│                  │ (结构化结果集)   │                       │
│                  └─────────────────┘                       │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      实施阶段 2 (P1)                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │  statement.rs│  │  batch.rs    │  │  session.rs  │      │
│  │ (预编译语句)  │  │  (批量操作)   │  │ (参数化查询)  │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      实施阶段 3 (P2)                         │
│  ┌─────────────────────────────────────────────────────┐   │
│  │                    ffi/                              │   │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐          │   │
│  │  │ mod.rs   │  │ c_api.rs │  │ types.rs │          │   │
│  │  │(模块入口)│  │(C API)   │  │(类型转换)│          │   │
│  │  └──────────┘  └──────────┘  └──────────┘          │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

---

## 对现有代码的修改需求

### 无需修改的模块（纯复用）

| 模块 | 复用方式 | 说明 |
|-----|---------|------|
| `query::QueryPipelineManager` | 直接实例化 | 执行查询管道 |
| `transaction::TransactionManager` | 直接实例化 | 事务管理 |
| `storage::StorageClient` | Arc<dyn> 持有 | 存储引擎 |
| `core::value::Value` | 直接使用 | 值类型 |
| `core::vertex_edge_path` | 直接使用 | 图类型 |

### 可能需要扩展的模块

| 模块 | 扩展需求 | 说明 |
|-----|---------|------|
| `query::parser` | 支持参数占位符 | `$param_name` 语法 |
| `query::planner` | 暴露计划缓存 | 预编译语句需要 |
| `storage` | 批量写入接口 | 批量操作优化 |

### 建议保持不变的模块

- `api::service::*` - 网络服务层完全独立
- `api::session::*` - 网络会话层完全独立
- `api::mod.rs` - 只需添加 `pub mod embedded;`

---

## 风险与缓解措施

| 风险 | 影响 | 缓解措施 |
|-----|------|---------|
| 同步/异步转换性能损耗 | 中 | 使用 `block_on` 或独立运行时， benchmark 验证 |
| 结果集转换内存开销 | 中 | 实现流式结果迭代器，避免全量加载 |
| 与现有代码冲突 | 低 | 完全新建模块，不修改现有代码 |
| FFI 内存安全问题 | 高 | 严格所有权管理，使用 `Box::into_raw`/`from_raw` |

---

## 总结

当前 `src/api` 层为支持嵌入式数据库架构，需要补充以下核心功能：

1. **同步执行层** - 包装异步查询引擎
2. **结构化结果集** - 从字符串结果到类型化访问
3. **简化会话管理** - 去除网络概念，专注数据操作
4. **数据库实例管理** - 统一生命周期管理
5. **错误类型统一** - 提供用户友好的错误接口
6. **高级特性** - 参数化查询、预编译语句、批量操作
7. **FFI 绑定** - 跨语言支持

这些补充功能将在新建的 `src/api/embedded/` 模块中实现，与现有 `service/` 和 `session/` 完全独立，通过复用底层引擎（`query`、`storage`、`transaction`）保持一致性。
