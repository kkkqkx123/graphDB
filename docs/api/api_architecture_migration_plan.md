# GraphDB API 架构重构实施计划（方案B）

## 概述

本文档详细描述将 `src/api` 从现有结构重构为分层架构的具体实施步骤、风险控制和验证方法。

---

## 实施原则

1. **渐进式重构** - 分阶段实施，每阶段保持可运行状态
2. **向后兼容** - 尽可能保持现有 API 接口不变
3. **测试驱动** - 每步都有测试验证
4. **快速回滚** - 每个阶段都可以快速回退到稳定状态

---

## 实施阶段

### 阶段 1：创建核心层框架（第 1-2 天）

**目标**：建立 `api/core/` 目录结构，创建基础类型和错误定义

#### 1.1 创建目录结构

```bash
# 创建核心层目录
mkdir -p src/api/core
mkdir -p src/api/embedded
mkdir -p src/api/server/http
mkdir -p src/api/server/auth
mkdir -p src/api/server/session
mkdir -p src/api/server/permission
mkdir -p src/api/server/stats

# 创建文件
touch src/api/core/mod.rs
touch src/api/core/error.rs
touch src/api/core/types.rs
touch src/api/core/query_api.rs
touch src/api/core/transaction_api.rs
touch src/api/core/schema_api.rs

touch src/api/embedded/mod.rs
touch src/api/embedded/database.rs
touch src/api/embedded/session.rs
touch src/api/embedded/transaction.rs
touch src/api/embedded/result.rs
touch src/api/embedded/error.rs

touch src/api/server/mod.rs
touch src/api/server/http/mod.rs
touch src/api/server/http/server.rs
touch src/api/server/auth/mod.rs
touch src/api/server/session/mod.rs
touch src/api/server/permission/mod.rs
touch src/api/server/stats/mod.rs
```

#### 1.2 实现核心错误类型

**文件**：`src/api/core/error.rs`

```rust
use thiserror::Error;

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

// 转换实现
impl From<crate::core::error::QueryError> for CoreError {
    fn from(err: crate::core::error::QueryError) -> Self {
        CoreError::QueryExecutionFailed(err.to_string())
    }
}
```

#### 1.3 实现核心类型

**文件**：`src/api/core/types.rs`

```rust
use crate::core::value::Value;
use std::collections::HashMap;

/// 查询上下文
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

/// 查询结果
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

/// 事务句柄
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransactionHandle(pub u64);

/// 保存点 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SavepointId(pub u64);
```

#### 1.4 更新核心模块入口

**文件**：`src/api/core/mod.rs`

```rust
//! API 核心层 - 与传输层无关的业务逻辑

pub mod error;
pub mod types;
pub mod query_api;
pub mod transaction_api;
pub mod schema_api;

pub use error::{CoreError, CoreResult};
pub use types::*;
pub use query_api::QueryApi;
pub use transaction_api::TransactionApi;
pub use schema_api::SchemaApi;
```

#### 1.5 验证步骤

```bash
# 编译检查
cargo check

# 确保没有破坏现有代码
cargo test --lib

# 提交
# git add src/api/core/
# git commit -m "Phase 1: Create api/core/ framework"
```

---

### 阶段 2：提取查询 API 到核心层（第 3-4 天）

**目标**：将 `QueryEngine` 的核心逻辑提取到 `api/core/query_api.rs`

#### 2.1 分析现有 QueryEngine

**现有代码**：`src/api/service/query_processor.rs`

```rust
pub struct QueryEngine<S: StorageClient + 'static> {
    storage: Arc<Mutex<S>>,
    pipeline_manager: QueryPipelineManager<S>,
}

impl<S: StorageClient + Clone + 'static> QueryEngine<S> {
    pub fn execute(&mut self, rctx: RequestContext) -> ExecutionResponse {
        // 核心逻辑在这里
        match self.pipeline_manager.execute_query_with_space(...) {
            Ok(result) => ExecutionResponse { result: Ok(...), ... },
            Err(e) => ExecutionResponse { result: Err(...), ... },
        }
    }
}
```

#### 2.2 实现核心 QueryApi

**文件**：`src/api/core/query_api.rs`

```rust
use crate::query::QueryPipelineManager;
use crate::storage::StorageClient;
use crate::api::core::{CoreResult, CoreError, QueryContext, QueryResult, Row, ExecutionMetadata};
use std::sync::Arc;
use parking_lot::Mutex;

pub struct QueryApi<S: StorageClient + 'static> {
    pipeline_manager: QueryPipelineManager<S>,
}

impl<S: StorageClient + Clone + 'static> QueryApi<S> {
    pub fn new(storage: Arc<S>) -> Self {
        let stats_manager = Arc::new(crate::api::service::StatsManager::new());
        Self {
            pipeline_manager: QueryPipelineManager::new(
                Arc::new(Mutex::new((*storage).clone())),
                stats_manager,
            ),
        }
    }
    
    pub fn execute(
        &mut self,
        query: &str,
        ctx: QueryContext,
    ) -> CoreResult<QueryResult> {
        let space_info = ctx.space_id.map(|id| crate::core::types::SpaceInfo {
            space_id: id,
            space_name: String::new(),
            vid_type: crate::core::types::DataType::String,
            tags: Vec::new(),
            edge_types: Vec::new(),
            version: crate::core::types::MetadataVersion::default(),
            comment: None,
        });
        
        let execution_result = self
            .pipeline_manager
            .execute_query_with_space(query, space_info)
            
            .map_err(|e| CoreError::QueryExecutionFailed(e.to_string()))?;
        
        Self::convert_to_query_result(execution_result)
    }
    
    fn convert_to_query_result(
        execution: crate::query::executor::base::ExecutionResult
    ) -> CoreResult<QueryResult> {
        // TODO: 实现转换逻辑
        // 从 ExecutionResult 提取 columns、rows、metadata
        todo!("实现结果转换")
    }
}
```

#### 2.3 保持 QueryEngine 向后兼容

**修改**：`src/api/service/query_processor.rs`

```rust
// 保留现有接口，内部调用新的 QueryApi
pub struct QueryEngine<S: StorageClient + 'static> {
    query_api: crate::api::core::QueryApi<S>,
    // 保留其他字段...
}

impl<S: StorageClient + Clone + 'static> QueryEngine<S> {
    pub fn new(storage: Arc<S>) -> Self {
        Self {
            query_api: crate::api::core::QueryApi::new(storage.clone()),
            // ...
        }
    }
    
    pub fn execute(&mut self, rctx: RequestContext) -> ExecutionResponse {
        // 转换为新的上下文
        let ctx = crate::api::core::QueryContext {
            space_id: None, // 从 rctx 提取
            auto_commit: true,
            transaction_id: None,
            parameters: None,
        };
        
        // 调用核心层
        match self.query_api.execute(&rctx.statement, ctx) {
            Ok(result) => ExecutionResponse {
                result: Ok(format!("{:?}", result)),
                latency_us: result.metadata.execution_time_ms * 1000,
            },
            Err(e) => ExecutionResponse {
                result: Err(e.to_string()),
                latency_us: 0,
            },
        }
    }
}
```

#### 2.4 验证步骤

```bash
# 编译检查
cargo check

# 运行现有测试
cargo test --lib api::service

# 确保功能正常
cargo test query_processor

# 提交
# git add src/api/core/query_api.rs
# git add src/api/service/query_processor.rs
# git commit -m "Phase 2: Extract QueryApi to core layer"
```

---

### 阶段 3：提取事务 API 到核心层（第 5 天）

**目标**：将事务管理逻辑提取到 `api/core/transaction_api.rs`

#### 3.1 实现核心 TransactionApi

**文件**：`src/api/core/transaction_api.rs`

```rust
use crate::transaction::{TransactionManager, TransactionOptions};
use crate::api::core::{CoreResult, CoreError, TransactionHandle, SavepointId};
use std::sync::Arc;

pub struct TransactionApi {
    txn_manager: Arc<TransactionManager>,
}

impl TransactionApi {
    pub fn new(txn_manager: Arc<TransactionManager>) -> Self {
        Self { txn_manager }
    }
    
    pub fn begin(&self, options: TransactionOptions) -> CoreResult<TransactionHandle> {
        let txn_id = self.txn_manager
            .begin_transaction(options)
            .map_err(|e| CoreError::TransactionFailed(e.to_string()))?;
        Ok(TransactionHandle(txn_id))
    }
    
    pub fn commit(&self, handle: TransactionHandle) -> CoreResult<()> {
        self.txn_manager
            .commit_transaction(handle.0)
            .map_err(|e| CoreError::TransactionFailed(e.to_string()))
    }
    
    pub fn rollback(&self, handle: TransactionHandle) -> CoreResult<()> {
        self.txn_manager
            .rollback_transaction(handle.0)
            .map_err(|e| CoreError::TransactionFailed(e.to_string()))
    }
}
```

#### 3.2 验证步骤

```bash
cargo check
cargo test --lib
# git commit -m "Phase 3: Extract TransactionApi to core layer"
```

---

### 阶段 4：实现嵌入式层（第 6-8 天）

**目标**：创建完整的嵌入式 API 实现

#### 4.1 实现嵌入式错误类型

**文件**：`src/api/embedded/error.rs`

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EmbeddedError {
    #[error("数据库连接失败: {0}")]
    ConnectionFailed(String),
    
    #[error("查询执行失败: {0}")]
    QueryExecutionFailed(String),
    
    #[error("事务错误: {0}")]
    TransactionError(String),
    
    #[error("IO错误: {0}")]
    Io(#[from] std::io::Error),
}

pub type EmbeddedResult<T> = Result<T, EmbeddedError>;

impl From<crate::api::core::CoreError> for EmbeddedError {
    fn from(err: crate::api::core::CoreError) -> Self {
        match err {
            crate::api::core::CoreError::QueryExecutionFailed(s) => {
                EmbeddedError::QueryExecutionFailed(s)
            }
            crate::api::core::CoreError::TransactionFailed(s) => {
                EmbeddedError::TransactionError(s)
            }
            _ => EmbeddedError::ConnectionFailed(err.to_string()),
        }
    }
}
```

#### 4.2 实现 GraphDatabase

**文件**：`src/api/embedded/database.rs`

```rust
use crate::api::core::{QueryApi, TransactionApi, SchemaApi};
use crate::storage::redb_storage::DefaultStorage;
use crate::transaction::{TransactionManager, TransactionManagerConfig};
use std::path::Path;
use std::sync::Arc;

pub struct GraphDatabase {
    storage: Arc<DefaultStorage>,
    txn_manager: Arc<TransactionManager>,
    query_api: QueryApi<DefaultStorage>,
    txn_api: TransactionApi,
    // schema_api: SchemaApi<DefaultStorage>,
}

impl GraphDatabase {
    pub fn open_in_memory() -> EmbeddedResult<Self> {
        let storage = Arc::new(DefaultStorage::new()?);
        let txn_manager = Arc::new(TransactionManager::new(
            storage.get_db().clone(),
            TransactionManagerConfig::default(),
        ));
        
        Ok(Self {
            query_api: QueryApi::new(storage.clone()),
            txn_api: TransactionApi::new(txn_manager.clone()),
            storage,
            txn_manager,
        })
    }
    
    pub fn session(&self) -> EmbeddedResult<Session> {
        Ok(Session::new(
            self.query_api.clone(),
            self.txn_api.clone(),
        ))
    }
}
```

#### 4.3 实现 EmbeddedSession

**文件**：`src/api/embedded/session.rs`

```rust
pub struct Session {
    query_api: QueryApi<DefaultStorage>,
    txn_api: TransactionApi,
    current_space: Option<String>,
    auto_commit: bool,
}

impl Session {
    pub(crate) fn new(
        query_api: QueryApi<DefaultStorage>,
        txn_api: TransactionApi,
    ) -> Self {
        Self {
            query_api,
            txn_api,
            current_space: None,
            auto_commit: true,
        }
    }
    
    pub fn execute(&self, query: &str) -> EmbeddedResult<QueryResult> {
        // 同步执行
        let rt = tokio::runtime::Runtime::new()?;
        let ctx = crate::api::core::QueryContext::default();
        let result = rt.block_on(self.query_api.execute(query, ctx))?;
        Ok(result)
    }
}
```

#### 4.4 更新嵌入式模块入口

**文件**：`src/api/embedded/mod.rs`

```rust
pub mod database;
pub mod session;
pub mod error;

pub use database::GraphDatabase;
pub use session::Session;
pub use error::{EmbeddedError, EmbeddedResult};
```

#### 4.5 添加嵌入式测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_embedded_database() {
        let db = GraphDatabase::open_in_memory().unwrap();
        let session = db.session().unwrap();
        // 测试查询...
    }
}
```

#### 4.6 验证步骤

```bash
cargo check
cargo test --lib api::embedded
# git commit -m "Phase 4: Implement embedded API layer"
```

---

### 阶段 5：重构网络服务层（第 9-12 天）

**目标**：将现有 service/ 和 session/ 重构为 server/ 子模块

#### 5.1 移动认证模块

```bash
# 创建新位置
mkdir -p src/api/server/auth

# 移动文件（保留原文件作为备份）
cp src/api/service/authenticator.rs src/api/server/auth/
```

**文件**：`src/api/server/auth/mod.rs`

```rust
pub mod authenticator;
pub use authenticator::{Authenticator, PasswordAuthenticator, AuthenticatorFactory};
```

#### 5.2 移动会话管理模块

```bash
mkdir -p src/api/server/session
cp src/api/session/*.rs src/api/server/session/
```

**重命名**：`client_session.rs` → `network_session.rs`

**文件**：`src/api/server/session/mod.rs`

```rust
pub mod network_session;
pub mod session_manager;
pub mod query_manager;
pub mod request_context;
pub mod types;

pub use network_session::NetworkSession;  // 重命名
pub use session_manager::{SessionManager, SessionInfo};
```

#### 5.3 创建 HTTP 服务器模块

**文件**：`src/api/server/http/server.rs`

```rust
use crate::api::core::{QueryApi, TransactionApi};
use crate::api::server::auth::AuthService;
use crate::api::server::session::SessionManager;

pub struct HttpServer {
    query_api: QueryApi,
    txn_api: TransactionApi,
    auth_service: AuthService,
    session_manager: SessionManager,
}

impl HttpServer {
    pub fn new(storage: Arc<dyn StorageClient>, txn_manager: Arc<TransactionManager>) -> Self {
        Self {
            query_api: QueryApi::new(storage.clone()),
            txn_api: TransactionApi::new(txn_manager.clone()),
            auth_service: AuthService::new(),
            session_manager: SessionManager::new(),
        }
    }
}
```

#### 5.4 更新 server 模块入口

**文件**：`src/api/server/mod.rs`

```rust
pub mod http;
pub mod auth;
pub mod session;
pub mod permission;
pub mod stats;

// 向后兼容导出
pub use http::HttpServer;
pub use auth::*;
pub use session::*;
```

#### 5.5 保持向后兼容

**文件**：`src/api/service/mod.rs`（保留，但标记为废弃）

```rust
//! 此模块已废弃，请使用 `api::server`
//! 
//! 保留用于向后兼容，将在 v2.0 移除

pub use crate::api::server::*;

#[deprecated(since = "0.2.0", note = "请使用 api::server::HttpServer")]
pub use crate::api::server::HttpServer as GraphService;
```

#### 5.6 验证步骤

```bash
cargo check
cargo test --lib api::server
cargo test --lib api::service  # 确保向后兼容
# git commit -m "Phase 5: Refactor server layer structure"
```

---

### 阶段 6：更新主模块入口（第 13 天）

**目标**：更新 `src/api/mod.rs` 统一导出

#### 6.1 更新 api/mod.rs

**文件**：`src/api/mod.rs`

```rust
//! GraphDB API 模块
//!
//! 提供多种访问方式：
//! - `core` - 核心 API（与传输层无关）
//! - `embedded` - 嵌入式 API（单机使用）
//! - `server` - 网络服务 API（HTTP/RPC）

pub mod core;
pub mod embedded;
pub mod server;

// 向后兼容
#[deprecated(since = "0.2.0", note = "请使用 api::server")]
pub mod service {
    pub use crate::api::server::*;
}

#[deprecated(since = "0.2.0", note = "请使用 api::server::session")]
pub mod session {
    pub use crate::api::server::session::*;
}

// 便捷导出
pub use core::{QueryApi, TransactionApi, SchemaApi};
pub use embedded::{GraphDatabase, Session};
pub use server::HttpServer;

// 服务启动函数（保留）
pub fn start_service(config_path: String) -> anyhow::Result<()> {
    server::start_service(config_path)
}
```

#### 6.2 验证完整构建

```bash
cargo clean
cargo build
cargo test --lib
# git commit -m "Phase 6: Update api module exports"
```

---

## 风险控制

### 风险 1：破坏现有功能

**缓解措施：**
- 每阶段都有完整的测试验证
- 保持向后兼容的导出
- 使用 `#[deprecated]` 标记旧接口

### 风险 2：编译错误

**缓解措施：**
- 频繁运行 `cargo check`
- 分小步骤提交
- 准备回滚脚本

### 风险 3：性能回归

**缓解措施：**
- 在阶段 2 和 5 进行性能测试
- 对比重构前后的基准测试
- 保留原始实现作为备选

---

## 验证清单

### 每阶段验证

- [ ] `cargo check` 无错误
- [ ] `cargo test --lib` 通过
- [ ] 现有示例代码可编译
- [ ] 新功能有基本测试

### 最终验证

- [ ] 完整构建通过
- [ ] 所有测试通过
- [ ] 文档更新完成
- [ ] 向后兼容验证
- [ ] 性能基准对比

---

## 回滚计划

如果重构出现问题，可以按以下步骤回滚：

```bash
# 回滚到阶段 5
git revert HEAD~1

# 回滚到阶段 1
git reset --hard <phase-1-commit>

# 或者保留新代码，恢复旧导出
# 修改 src/api/mod.rs 恢复旧导出路径
```

---

## 时间估算

| 阶段 | 天数 | 主要工作 |
|-----|------|---------|
| 1 | 2 | 创建核心层框架 |
| 2 | 2 | 提取查询 API |
| 3 | 1 | 提取事务 API |
| 4 | 3 | 实现嵌入式层 |
| 5 | 4 | 重构网络服务层 |
| 6 | 1 | 更新模块入口 |
| **总计** | **13** | |

---

## 总结

本实施计划通过 6 个阶段，将 `src/api` 从现有结构重构为分层架构：

1. **核心层** (`api/core/`) - 业务逻辑只实现一次
2. **嵌入式层** (`api/embedded/`) - 单机使用
3. **网络服务层** (`api/server/`) - HTTP/RPC 服务

每阶段都保持可运行状态，通过向后兼容确保平滑过渡。
