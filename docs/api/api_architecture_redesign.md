# GraphDB API 架构重构设计（方案B）

## 概述

本文档描述将 `src/api` 从网络服务导向重构为分层架构的设计方案，提取通用业务逻辑到核心层，实现嵌入式和网络服务的对等架构。

---

## 架构目标

1. **分层清晰** - 核心层、嵌入式层、网络层职责明确
2. **代码复用** - 业务逻辑只实现一次，多场景复用
3. **对等架构** - 嵌入式和网络服务是平等的顶层适配层
4. **可扩展性** - 易于添加新的传输层（gRPC、WebSocket等）

---

## 架构设计

### 整体架构图

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           应用层 (Applications)                          │
│  ┌─────────────────────┐  ┌─────────────────────┐  ┌─────────────────┐ │
│  │   嵌入式应用         │  │   HTTP/RPC 客户端   │  │   CLI 工具       │ │
│  │  (Embedded App)     │  │  (Network Client)   │  │  (Command Line) │ │
│  └──────────┬──────────┘  └──────────┬──────────┘  └────────┬────────┘ │
└─────────────┼────────────────────────┼──────────────────────┼──────────┘
              │                        │                      │
              ▼                        ▼                      ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                           API 适配层 (API Adapters)                      │
│  ┌─────────────────────┐  ┌─────────────────────┐  ┌─────────────────┐ │
│  │   embedded/          │  │   server/           │  │   ffi/          │ │
│  │  嵌入式API           │  │  网络服务API         │  │  C FFI绑定      │ │
│  │  - GraphDatabase     │  │  - HttpServer       │  │  - C API        │ │
│  │  - Session           │  │  - AuthService      │  │  - 类型转换     │ │
│  │  - Transaction       │  │  - SessionManager   │  │                 │ │
│  └──────────┬──────────┘  └──────────┬──────────┘  └────────┬────────┘ │
└─────────────┼────────────────────────┼──────────────────────┼──────────┘
              │                        │                      │
              └────────────────────────┼──────────────────────┘
                                       │
                                       ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                           API 核心层 (API Core)                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │  query_api   │  │  txn_api     │  │  schema_api  │  │  admin_api   │ │
│  │  查询执行    │  │  事务管理    │  │  Schema操作  │  │  管理操作    │ │
│  └──────────────┘  └──────────────┘  └──────────────┘  └──────────────┘ │
└─────────────────────────────────────────────────────────────────────────┘
                                       │
                                       ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                           核心引擎层 (Core Engine)                       │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │    query/    │  │   storage/   │  │ transaction/ │  │    core/     │ │
│  │  查询引擎    │  │  存储引擎    │  │  事务管理器  │  │  类型定义    │ │
│  └──────────────┘  └──────────────┘  └──────────────┘  └──────────────┘ │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 目录结构设计

### 重构后的目录结构

```
src/api/
├── mod.rs                      # API 模块入口，导出公共接口
├── core/                       # 核心层 - 与传输层无关的业务逻辑
│   ├── mod.rs
│   ├── query_api.rs            # 查询执行 API
│   ├── transaction_api.rs      # 事务管理 API
│   ├── schema_api.rs           # Schema 操作 API
│   ├── admin_api.rs            # 管理操作 API
│   ├── types.rs                # 核心类型定义
│   └── error.rs                # 核心错误类型
├── embedded/                   # 嵌入式适配层
│   ├── mod.rs
│   ├── database.rs             # GraphDatabase
│   ├── session.rs              # EmbeddedSession
│   ├── transaction.rs          # EmbeddedTransaction
│   ├── result.rs               # QueryResult, Row
│   ├── statement.rs            # PreparedStatement
│   ├── batch.rs                # BatchInserter
│   └── error.rs                # 嵌入式错误转换
├── server/                     # 网络服务层（原 service + session）
│   ├── mod.rs
│   ├── http/                   # HTTP 服务
│   │   ├── mod.rs
│   │   ├── server.rs           # HttpServer
│   │   ├── handlers.rs         # 请求处理器
│   │   └── middleware.rs       # 中间件
│   ├── auth/                   # 认证（原 authenticator）
│   │   ├── mod.rs
│   │   ├── authenticator.rs
│   │   └── types.rs
│   ├── session/                # 网络会话（原 session）
│   │   ├── mod.rs
│   │   ├── session_manager.rs
│   │   ├── network_session.rs  # 重命名自 client_session
│   │   └── types.rs
│   ├── permission/             # 权限（原 permission_manager）
│   │   ├── mod.rs
│   │   ├── permission_manager.rs
│   │   └── checker.rs
│   └── stats/                  # 统计（原 stats_manager）
│       ├── mod.rs
│       └── stats_manager.rs
└── ffi/                        # C FFI 绑定（可选）
    ├── mod.rs
    ├── c_api.rs
    └── types.rs
```

---

## 核心层设计（api/core/）

### 1. 核心查询 API

```rust
// src/api/core/query_api.rs

use crate::query::QueryPipelineManager;
use crate::storage::StorageClient;
use crate::core::value::Value;

/// 查询执行上下文 - 与传输层无关
#[derive(Debug, Clone)]
pub struct QueryContext {
    pub space_id: Option<u64>,
    pub auto_commit: bool,
    pub transaction_id: Option<u64>,
    pub parameters: Option<HashMap<String, Value>>,
}

impl Default for QueryContext {
    fn default() -> Self {
        Self {
            space_id: None,
            auto_commit: true,
            transaction_id: None,
            parameters: None,
        }
    }
}

/// 查询执行结果 - 结构化数据
#[derive(Debug)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Row>,
    pub metadata: ExecutionMetadata,
}

#[derive(Debug)]
pub struct Row {
    pub values: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
pub struct ExecutionMetadata {
    pub execution_time_ms: u64,
    pub rows_scanned: u64,
    pub rows_returned: u64,
    pub cache_hit: bool,
}

/// 通用查询 API - 核心层
pub struct QueryApi<S: StorageClient> {
    pipeline_manager: QueryPipelineManager<S>,
}

impl<S: StorageClient + Clone + 'static> QueryApi<S> {
    pub fn new(storage: Arc<S>) -> Self {
        let stats_manager = Arc::new(StatsManager::new());
        Self {
            pipeline_manager: QueryPipelineManager::new(
                Arc::new(Mutex::new((*storage).clone())),
                stats_manager,
            ),
        }
    }
    
    /// 执行查询 - 核心逻辑
    pub async fn execute(
        &mut self,
        query: &str,
        ctx: QueryContext,
    ) -> CoreResult<QueryResult> {
        let space_info = ctx.space_id.map(|id| SpaceInfo {
            space_id: id,
            space_name: String::new(),
            vid_type: DataType::String,
            tags: Vec::new(),
            edge_types: Vec::new(),
            version: MetadataVersion::default(),
            comment: None,
        });
        
        let execution_result = self
            .pipeline_manager
            .execute_query_with_space(query, space_info)
            .await
            .map_err(|e| CoreError::QueryExecutionFailed(e.to_string()))?;
        
        // 转换为结构化结果
        Self::convert_to_query_result(execution_result)
    }
    
    /// 执行参数化查询
    pub async fn execute_with_params(
        &mut self,
        query: &str,
        params: HashMap<String, Value>,
        ctx: QueryContext,
    ) -> CoreResult<QueryResult> {
        let mut ctx = ctx;
        ctx.parameters = Some(params);
        self.execute(query, ctx).await
    }
    
    fn convert_to_query_result(execution: ExecutionResult) -> CoreResult<QueryResult> {
        // 转换逻辑...
        todo!("实现结果转换")
    }
}
```

### 2. 核心事务 API

```rust
// src/api/core/transaction_api.rs

use crate::transaction::{TransactionManager, TransactionOptions, TransactionId};

/// 事务句柄
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransactionHandle(pub u64);

/// 保存点 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SavepointId(pub u64);

/// 通用事务 API - 核心层
pub struct TransactionApi {
    txn_manager: Arc<TransactionManager>,
}

impl TransactionApi {
    pub fn new(txn_manager: Arc<TransactionManager>) -> Self {
        Self { txn_manager }
    }
    
    /// 开始事务
    pub fn begin(&self, options: TransactionOptions) -> CoreResult<TransactionHandle> {
        let txn_id = self.txn_manager
            .begin_transaction(options)
            .map_err(|e| CoreError::TransactionFailed(e.to_string()))?;
        Ok(TransactionHandle(txn_id))
    }
    
    /// 提交事务
    pub fn commit(&self, handle: TransactionHandle) -> CoreResult<()> {
        self.txn_manager
            .commit_transaction(handle.0)
            .map_err(|e| CoreError::TransactionFailed(e.to_string()))
    }
    
    /// 回滚事务
    pub fn rollback(&self, handle: TransactionHandle) -> CoreResult<()> {
        self.txn_manager
            .rollback_transaction(handle.0)
            .map_err(|e| CoreError::TransactionFailed(e.to_string()))
    }
    
    /// 创建保存点
    pub fn savepoint(
        &self,
        handle: TransactionHandle,
        name: &str,
    ) -> CoreResult<SavepointId> {
        let sp_id = self.txn_manager
            .create_savepoint(handle.0, name)
            .map_err(|e| CoreError::TransactionFailed(e.to_string()))?;
        Ok(SavepointId(sp_id))
    }
    
    /// 回滚到保存点
    pub fn rollback_to_savepoint(
        &self,
        handle: TransactionHandle,
        sp_id: SavepointId,
    ) -> CoreResult<()> {
        self.txn_manager
            .rollback_to_savepoint(handle.0, sp_id.0)
            .map_err(|e| CoreError::TransactionFailed(e.to_string()))
    }
    
    /// 释放保存点
    pub fn release_savepoint(
        &self,
        handle: TransactionHandle,
        sp_id: SavepointId,
    ) -> CoreResult<()> {
        self.txn_manager
            .release_savepoint(handle.0, sp_id.0)
            .map_err(|e| CoreError::TransactionFailed(e.to_string()))
    }
}
```

### 3. 核心 Schema API

```rust
// src/api/core/schema_api.rs

use crate::storage::StorageClient;

/// Schema 操作 API - 核心层
pub struct SchemaApi<S: StorageClient> {
    storage: Arc<S>,
}

impl<S: StorageClient> SchemaApi<S> {
    pub fn new(storage: Arc<S>) -> Self {
        Self { storage }
    }
    
    /// 创建图空间
    pub async fn create_space(
        &self,
        name: &str,
        config: SpaceConfig,
    ) -> CoreResult<()> {
        todo!("实现创建空间")
    }
    
    /// 删除图空间
    pub async fn drop_space(&self, name: &str) -> CoreResult<()> {
        todo!("实现删除空间")
    }
    
    /// 创建标签
    pub async fn create_tag(
        &self,
        space_id: u64,
        name: &str,
        properties: Vec<PropertyDef>,
    ) -> CoreResult<()> {
        todo!("实现创建标签")
    }
    
    /// 创建边类型
    pub async fn create_edge_type(
        &self,
        space_id: u64,
        name: &str,
        properties: Vec<PropertyDef>,
    ) -> CoreResult<()> {
        todo!("实现创建边类型")
    }
    
    /// 创建索引
    pub async fn create_index(
        &self,
        space_id: u64,
        name: &str,
        target: IndexTarget,
    ) -> CoreResult<()> {
        todo!("实现创建索引")
    }
}
```

### 4. 核心错误类型

```rust
// src/api/core/error.rs

use thiserror::Error;

/// 核心层错误类型
#[derive(Error, Debug)]
pub enum CoreError {
    #[error("查询执行失败: {0}")]
    QueryExecutionFailed(String),
    
    #[error("事务操作失败: {0}")]
    TransactionFailed(String),
    
    #[error("Schema操作失败: {0}")]
    SchemaOperationFailed(String),
    
    #[error("存储错误: {0}")]
    StorageError(String),
    
    #[error("无效的参数: {0}")]
    InvalidParameter(String),
    
    #[error("资源不存在: {0}")]
    NotFound(String),
    
    #[error("内部错误: {0}")]
    Internal(String),
}

pub type CoreResult<T> = Result<T, CoreError>;

// 从底层错误转换
impl From<crate::core::error::QueryError> for CoreError {
    fn from(err: crate::core::error::QueryError) -> Self {
        CoreError::QueryExecutionFailed(err.to_string())
    }
}

impl From<crate::storage::StorageError> for CoreError {
    fn from(err: crate::storage::StorageError) -> Self {
        CoreError::StorageError(err.to_string())
    }
}
```

---

## 嵌入式层设计（api/embedded/）

### 1. 嵌入式数据库

```rust
// src/api/embedded/database.rs

use crate::api::core::{QueryApi, TransactionApi, SchemaApi};

/// 嵌入式数据库实例
pub struct GraphDatabase {
    storage: Arc<dyn StorageClient>,
    txn_manager: Arc<TransactionManager>,
    query_api: QueryApi,
    txn_api: TransactionApi,
    schema_api: SchemaApi,
    config: DatabaseConfig,
}

impl GraphDatabase {
    /// 打开数据库文件
    pub fn open(path: impl AsRef<Path>) -> EmbeddedResult<Self> {
        let config = DatabaseConfig {
            path: Some(path.as_ref().to_path_buf()),
            ..Default::default()
        };
        Self::open_with_config(config)
    }
    
    /// 打开内存数据库
    pub fn open_in_memory() -> EmbeddedResult<Self> {
        Self::open_with_config(DatabaseConfig::default())
    }
    
    /// 使用配置打开
    pub fn open_with_config(config: DatabaseConfig) -> EmbeddedResult<Self> {
        // 初始化存储
        let storage = Arc::new(DefaultStorage::new()?);
        
        // 初始化事务管理器
        let txn_manager = Arc::new(TransactionManager::new(
            storage.get_db().clone(),
            TransactionManagerConfig::default(),
        ));
        
        // 初始化核心 API
        let query_api = QueryApi::new(storage.clone());
        let txn_api = TransactionApi::new(txn_manager.clone());
        let schema_api = SchemaApi::new(storage.clone());
        
        Ok(Self {
            storage,
            txn_manager,
            query_api,
            txn_api,
            schema_api,
            config,
        })
    }
    
    /// 创建会话
    pub fn session(&self) -> EmbeddedResult<Session> {
        Session::new(
            self.query_api.clone(),
            self.txn_api.clone(),
            self.schema_api.clone(),
        )
    }
    
    /// 便捷方法：直接执行查询
    pub fn execute(&self, query: &str) -> EmbeddedResult<QueryResult> {
        let session = self.session()?;
        session.execute(query)
    }
    
    /// 关闭数据库
    pub fn close(self) -> EmbeddedResult<()> {
        // 清理资源
        Ok(())
    }
}
```

### 2. 嵌入式会话

```rust
// src/api/embedded/session.rs

use crate::api::core::{QueryApi, TransactionApi, SchemaApi, QueryContext};

/// 嵌入式会话
pub struct Session {
    query_api: QueryApi,
    txn_api: TransactionApi,
    schema_api: SchemaApi,
    current_space: Option<String>,
    auto_commit: bool,
}

impl Session {
    pub(crate) fn new(
        query_api: QueryApi,
        txn_api: TransactionApi,
        schema_api: SchemaApi,
    ) -> EmbeddedResult<Self> {
        Ok(Self {
            query_api,
            txn_api,
            schema_api,
            current_space: None,
            auto_commit: true,
        })
    }
    
    /// 切换图空间
    pub fn use_space(&mut self, space_name: &str) -> EmbeddedResult<()> {
        self.current_space = Some(space_name.to_string());
        Ok(())
    }
    
    /// 执行查询
    pub fn execute(&self, query: &str) -> EmbeddedResult<QueryResult> {
        let ctx = QueryContext {
            space_id: None, // TODO: 从 current_space 解析
            auto_commit: self.auto_commit,
            transaction_id: None,
            parameters: None,
        };
        
        // 同步执行异步核心 API
        let rt = tokio::runtime::Handle::try_current()
            .or_else(|_| {
                tokio::runtime::Runtime::new()
                    .map(|rt| rt.handle().clone())
            })?;
        
        let result = rt.block_on(self.query_api.execute(query, ctx))?;
        Ok(result)
    }
    
    /// 开始事务
    pub fn begin_transaction(&mut self) -> EmbeddedResult<Transaction> {
        let handle = self.txn_api.begin(TransactionOptions::default())?;
        Ok(Transaction::new(self.txn_api.clone(), handle))
    }
}
```

---

## 网络服务层设计（api/server/）

### 1. HTTP 服务

```rust
// src/api/server/http/server.rs

use crate::api::core::{QueryApi, TransactionApi, SchemaApi};
use crate::api::server::auth::AuthService;
use crate::api::server::session::SessionManager;

/// HTTP 服务器
pub struct HttpServer {
    query_api: QueryApi,
    txn_api: TransactionApi,
    schema_api: SchemaApi,
    auth_service: AuthService,
    session_manager: SessionManager,
}

impl HttpServer {
    pub fn new(
        storage: Arc<dyn StorageClient>,
        txn_manager: Arc<TransactionManager>,
        config: ServerConfig,
    ) -> Self {
        Self {
            query_api: QueryApi::new(storage.clone()),
            txn_api: TransactionApi::new(txn_manager.clone()),
            schema_api: SchemaApi::new(storage.clone()),
            auth_service: AuthService::new(&config.auth),
            session_manager: SessionManager::new(config.max_connections),
        }
    }
    
    /// 处理查询请求
    pub async fn handle_query(&self, req: QueryRequest) -> HttpResponse {
        // 1. 认证
        let session = match self.auth_service.authenticate(&req.token).await {
            Ok(s) => s,
            Err(e) => return HttpResponse::unauthorized(e),
        };
        
        // 2. 构建核心上下文
        let ctx = QueryContext {
            space_id: session.space_id,
            auto_commit: session.auto_commit,
            transaction_id: session.transaction_id,
            parameters: Some(req.parameters),
        };
        
        // 3. 调用核心层
        match self.query_api.execute(&req.query, ctx).await {
            Ok(result) => HttpResponse::success(result),
            Err(e) => HttpResponse::error(e),
        }
    }
}
```

---

## 架构优势

### 1. 代码复用

```
业务逻辑只在 core/ 实现一次
    ↓
embedded/ 和 server/ 都是薄适配层
    ↓
减少重复代码，一致性行为
```

### 2. 测试友好

```rust
// 可以独立测试核心层
#[test]
fn test_query_api() {
    let api = QueryApi::new(mock_storage());
    let result = api.execute("MATCH (n) RETURN n", QueryContext::default());
    // 断言结果...
}

// 测试嵌入式层
#[test]
fn test_embedded() {
    let db = GraphDatabase::open_in_memory().unwrap();
    let session = db.session().unwrap();
    // 测试...
}

// 测试网络层
#[test]
fn test_http_server() {
    let server = HttpServer::new(...);
    // 测试...
}
```

### 3. 易于扩展

添加 gRPC 支持只需：
```rust
// src/api/server/grpc/mod.rs
pub struct GrpcServer {
    query_api: QueryApi,  // 复用核心层
    // ...
}
```

---

## 与现有代码的关系

| 现有代码 | 新位置 | 处理方式 |
|---------|-------|---------|
| `api/service/graph_service.rs` | `api/server/http/server.rs` | 重构使用 core/ |
| `api/service/authenticator.rs` | `api/server/auth/` | 移动并适配 |
| `api/service/permission_manager.rs` | `api/server/permission/` | 移动 |
| `api/service/query_processor.rs` | `api/core/query_api.rs` | 提取核心逻辑 |
| `api/service/stats_manager.rs` | `api/server/stats/` | 移动（网络特有） |
| `api/session/session_manager.rs` | `api/server/session/` | 移动 |
| `api/session/client_session.rs` | `api/server/session/network_session.rs` | 重命名并适配 |
| `api/mod.rs` | `api/mod.rs` | 更新导出 |

---

## 总结

方案B通过提取通用核心层，实现了：

1. ✅ **架构对等** - 嵌入式和网络服务是平等的顶层适配层
2. ✅ **代码复用** - 业务逻辑只在 core/ 实现一次
3. ✅ **职责清晰** - 每层只关注自己的职责
4. ✅ **可扩展** - 易于添加新的传输层
5. ✅ **可测试** - 每层可以独立测试

这是长期维护性和架构清晰度的最佳选择。
