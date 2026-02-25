# GraphDB 嵌入式 API 实现方案（方案A）

## 概述

本文档详细描述在 `src/api` 下新建 `embedded` 子模块的实现方案，明确与现有 `service`、`session` 模块的职责划分。

---

## 当前架构分析

### 现有 `src/api` 目录结构

```
src/api/
├── mod.rs                    # API 模块入口，服务启动函数
├── service/                  # 网络服务层
│   ├── mod.rs
│   ├── graph_service.rs      # HTTP/RPC 服务入口
│   ├── authenticator.rs      # 用户认证
│   ├── permission_manager.rs # 权限管理
│   ├── permission_checker.rs # 权限检查
│   ├── query_processor.rs    # 查询处理
│   └── stats_manager.rs      # 统计监控
└── session/                  # 网络会话层
    ├── mod.rs
    ├── client_session.rs     # 客户端会话（含网络信息）
    ├── session_manager.rs    # 会话管理（连接池）
    ├── query_manager.rs      # 查询管理
    ├── request_context.rs    # 请求上下文
    └── types.rs              # 会话类型定义
```

### 现有模块职责

| 模块 | 核心职责 | 关键特性 |
|-----|---------|---------|
| `service/` | 提供网络服务能力 | 认证、权限、HTTP/RPC、多用户、连接池 |
| `session/` | 管理网络会话生命周期 | IP地址、超时控制、角色权限、并发限制 |

### 现有代码的关键特征

**`GraphService` 网络服务特征：**
```rust
pub struct GraphService<S: StorageClient + Clone + 'static> {
    session_manager: Arc<GraphSessionManager>,
    query_engine: Arc<Mutex<QueryEngine<S>>>,
    authenticator: PasswordAuthenticator,        // 认证
    permission_manager: Arc<PermissionManager>,  // 权限
    stats_manager: Arc<StatsManager>,            // 统计
    // ...
}
```

**`ClientSession` 网络会话特征：**
```rust
pub struct ClientSession {
    session: Arc<RwLock<Session>>,               // 含 user_name、graph_addr
    roles: Arc<RwLock<HashMap<i64, RoleType>>>,  // 权限角色
    idle_start_time: Arc<RwLock<Instant>>,       // 空闲超时（网络概念）
    contexts: Arc<RwLock<HashMap<u32, String>>>, // 查询上下文
    // ...
}
```

**问题：** 现有代码深度耦合网络服务概念，不适合嵌入式场景的单用户、无认证、直接访问需求。

---

## 方案A：新建 `embedded` 子模块

### 设计原则

1. **职责分离**：嵌入式 API 与网络服务 API 完全独立
2. **底层复用**：两者共用 `query`、`storage`、`transaction` 核心引擎
3. **零成本抽象**：嵌入式 API 不引入运行时开销
4. **渐进式实现**：先实现核心功能，再扩展高级特性

### 目标目录结构

```
src/api/
├── mod.rs                    # 导出所有子模块
├── service/                  # 网络服务层（保持现有）
│   ├── mod.rs
│   ├── graph_service.rs
│   ├── authenticator.rs
│   ├── permission_manager.rs
│   ├── permission_checker.rs
│   ├── query_processor.rs
│   └── stats_manager.rs
├── session/                  # 网络会话层（保持现有）
│   ├── mod.rs
│   ├── client_session.rs
│   ├── session_manager.rs
│   ├── query_manager.rs
│   ├── request_context.rs
│   └── types.rs
└── embedded/                 # 新增：嵌入式 API 层
    ├── mod.rs                # 嵌入式模块入口
    ├── database.rs           # GraphDatabase 实现
    ├── session.rs            # EmbeddedSession 实现
    ├── transaction.rs        # Transaction、Savepoint 实现
    ├── result.rs             # QueryResult、Row 实现
    ├── statement.rs          # PreparedStatement 实现
    ├── batch.rs              # BatchInserter 实现
    ├── error.rs              # 嵌入式专用错误类型
    ├── types.rs              # 嵌入式类型定义
    └── ffi/                  # C FFI 绑定（子模块）
        ├── mod.rs
        ├── c_api.rs          # C 接口实现
        └── types.rs          # C 类型转换
```

---

## 详细模块设计

### 1. `embedded/mod.rs` - 模块入口

```rust
//! GraphDB 嵌入式数据库 API
//!
//! 提供单用户、无网络、直接访问的图数据库接口
//! 适用于嵌入式应用、单机工具、测试场景

pub mod database;
pub mod session;
pub mod transaction;
pub mod result;
pub mod statement;
pub mod batch;
pub mod error;
pub mod types;

// 便捷导出
pub use database::{GraphDatabase, DatabaseConfig};
pub use session::Session;
pub use transaction::{Transaction, Savepoint};
pub use result::{QueryResult, Row, ResultMetadata};
pub use statement::PreparedStatement;
pub use batch::{BatchInserter, BatchResult};
pub use error::{EmbeddedError, EmbeddedResult};
pub use types::*;

// 条件编译 FFI 模块
#[cfg(feature = "ffi")]
pub mod ffi;
```

### 2. `embedded/database.rs` - 数据库实例

```rust
use std::path::Path;
use std::sync::Arc;
use crate::storage::StorageClient;
use crate::transaction::TransactionManager;
use crate::query::QueryEngine;

/// 嵌入式数据库实例
/// 
/// 对应 SQLite 的 sqlite3，是嵌入式 API 的入口点
pub struct GraphDatabase {
    storage: Arc<dyn StorageClient>,
    transaction_manager: Arc<TransactionManager>,
    query_engine: Arc<QueryEngine>,
    config: DatabaseConfig,
}

/// 数据库配置
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// 数据库文件路径，None 表示内存模式
    pub path: Option<std::path::PathBuf>,
    /// 缓存大小（字节）
    pub cache_size: usize,
    /// 默认查询超时
    pub query_timeout: std::time::Duration,
    /// 事务超时
    pub transaction_timeout: std::time::Duration,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            path: None,
            cache_size: 16 * 1024 * 1024,  // 16MB
            query_timeout: std::time::Duration::from_secs(30),
            transaction_timeout: std::time::Duration::from_secs(300),
        }
    }
}

impl GraphDatabase {
    /// 打开数据库（文件模式）
    /// 
    /// # 示例
    /// ```rust
    /// let db = GraphDatabase::open("./my_graph.db")?;
    /// ```
    pub fn open(path: impl AsRef<Path>) -> EmbeddedResult<Self> {
        let config = DatabaseConfig {
            path: Some(path.as_ref().to_path_buf()),
            ..Default::default()
        };
        Self::open_with_config(config)
    }
    
    /// 创建内存数据库
    /// 
    /// # 示例
    /// ```rust
    /// let db = GraphDatabase::open_in_memory()?;
    /// ```
    pub fn open_in_memory() -> EmbeddedResult<Self> {
        Self::open_with_config(DatabaseConfig::default())
    }
    
    /// 使用配置打开数据库
    pub fn open_with_config(config: DatabaseConfig) -> EmbeddedResult<Self> {
        // 1. 初始化存储引擎
        // 2. 初始化事务管理器
        // 3. 初始化查询引擎
        todo!("实现数据库初始化逻辑")
    }
    
    /// 关闭数据库
    /// 
    /// 确保所有数据刷写到磁盘，释放资源
    pub fn close(self) -> EmbeddedResult<()> {
        // 确保所有事务完成
        // 关闭存储引擎
        todo!("实现关闭逻辑")
    }
    
    /// 创建会话
    /// 
    /// 会话是执行查询的上下文，一个数据库可以有多个会话
    pub fn session(&self) -> EmbeddedResult<Session> {
        Session::new(
            self.storage.clone(),
            self.transaction_manager.clone(),
            self.query_engine.clone(),
        )
    }
    
    /// 便捷方法：直接执行查询（无需显式会话）
    /// 
    /// 适用于简单的单次查询场景
    pub fn execute(&self, query: &str) -> EmbeddedResult<QueryResult> {
        let session = self.session()?;
        session.execute(query)
    }
    
    /// 便捷方法：执行参数化查询
    pub fn execute_with_params(
        &self,
        query: &str,
        params: &std::collections::HashMap<String, crate::core::value::Value>,
    ) -> EmbeddedResult<QueryResult> {
        let session = self.session()?;
        session.execute_with_params(query, params)
    }
    
    /// 预编译查询语句
    /// 
    /// 对于需要重复执行的查询，预编译可以显著提高性能
    pub fn prepare(&self, query: &str) -> EmbeddedResult<PreparedStatement> {
        PreparedStatement::new(
            self.query_engine.clone(),
            query,
        )
    }
    
    /// 获取数据库统计信息
    pub fn stats(&self) -> DatabaseStats {
        todo!("实现统计信息收集")
    }
}

/// 数据库统计信息
#[derive(Debug, Clone)]
pub struct DatabaseStats {
    pub total_vertices: u64,
    pub total_edges: u64,
    pub disk_size: u64,
    pub cache_hit_rate: f64,
}
```

### 3. `embedded/session.rs` - 会话管理

```rust
use std::sync::Arc;
use crate::storage::StorageClient;
use crate::transaction::TransactionManager;
use crate::query::QueryEngine;

/// 嵌入式会话
/// 
/// 与网络版的 ClientSession 不同，EmbeddedSession：
/// - 不包含用户认证信息
/// - 不包含 IP 地址等网络信息
/// - 不管理连接池
/// - 直接操作存储引擎
pub struct Session {
    storage: Arc<dyn StorageClient>,
    transaction_manager: Arc<TransactionManager>,
    query_engine: Arc<QueryEngine>,
    current_space: Option<String>,
    auto_commit: bool,
}

impl Session {
    /// 创建新会话
    pub(crate) fn new(
        storage: Arc<dyn StorageClient>,
        transaction_manager: Arc<TransactionManager>,
        query_engine: Arc<QueryEngine>,
    ) -> EmbeddedResult<Self> {
        Ok(Self {
            storage,
            transaction_manager,
            query_engine,
            current_space: None,
            auto_commit: true,
        })
    }
    
    /// 切换图空间
    /// 
    /// # 示例
    /// ```rust
    /// session.use_space("social_network")?;
    /// ```
    pub fn use_space(&mut self, space_name: &str) -> EmbeddedResult<()> {
        // 验证空间存在
        // 切换当前空间
        self.current_space = Some(space_name.to_string());
        Ok(())
    }
    
    /// 获取当前图空间
    pub fn current_space(&self) -> Option<&str> {
        self.current_space.as_deref()
    }
    
    /// 执行查询
    /// 
    /// # 示例
    /// ```rust
    /// let result = session.execute("MATCH (p:Person) RETURN p.name")?;
    /// ```
    pub fn execute(&self, query: &str) -> EmbeddedResult<QueryResult> {
        // 1. 解析查询
        // 2. 生成执行计划
        // 3. 执行查询
        // 4. 返回结果
        todo!("实现查询执行逻辑")
    }
    
    /// 执行参数化查询
    /// 
    /// 参数化查询可以防止注入攻击，并提高性能
    pub fn execute_with_params(
        &self,
        query: &str,
        params: &std::collections::HashMap<String, crate::core::value::Value>,
    ) -> EmbeddedResult<QueryResult> {
        todo!("实现参数化查询执行")
    }
    
    /// 开始事务
    /// 
    /// 返回 Transaction 对象，用于显式控制事务边界
    pub fn begin_transaction(&mut self) -> EmbeddedResult<Transaction> {
        // 如果当前有事务，报错
        // 创建新事务
        todo!("实现事务开始逻辑")
    }
    
    /// 设置自动提交模式
    /// 
    /// 默认开启，每条语句自动提交
    pub fn set_auto_commit(&mut self, enabled: bool) {
        self.auto_commit = enabled;
    }
    
    /// 检查是否自动提交
    pub fn is_auto_commit(&self) -> bool {
        self.auto_commit
    }
    
    /// 创建批量插入器
    /// 
    /// 用于高效批量导入数据
    pub fn batch_inserter(&self, batch_size: usize) -> BatchInserter {
        BatchInserter::new(
            self.storage.clone(),
            batch_size,
        )
    }
    
    /// 执行托管事务（自动重试）
    /// 
    /// # 示例
    /// ```rust
    /// session.with_transaction(|txn| {
    ///     txn.execute("INSERT VERTEX Person(name) VALUES 'alice':('Alice')")?;
    ///     txn.execute("INSERT VERTEX Person(name) VALUES 'bob':('Bob')")?;
    ///     Ok(())
    /// })?;
    /// ```
    pub fn with_transaction<F, T>(&mut self, f: F) -> EmbeddedResult<T>
    where
        F: FnMut(&Transaction) -> EmbeddedResult<T>,
    {
        todo!("实现托管事务逻辑")
    }
}
```

### 4. `embedded/transaction.rs` - 事务管理

```rust
use std::sync::Arc;

/// 事务句柄
/// 
/// 通过 `Session::begin_transaction()` 创建
/// 必须调用 `commit()` 或 `rollback()` 结束事务
pub struct Transaction {
    session: *mut Session,  // 裸指针避免循环引用
    txn_id: u64,
    committed: bool,
    rolled_back: bool,
}

impl Transaction {
    /// 在事务中执行查询
    pub fn execute(&self, query: &str) -> EmbeddedResult<QueryResult> {
        // 确保事务有效
        // 在事务上下文中执行查询
        todo!("实现事务内查询执行")
    }
    
    /// 在事务中执行参数化查询
    pub fn execute_with_params(
        &self,
        query: &str,
        params: &std::collections::HashMap<String, crate::core::value::Value>,
    ) -> EmbeddedResult<QueryResult> {
        todo!("实现事务内参数化查询")
    }
    
    /// 提交事务
    /// 
    /// 消费 self，确保事务只能提交一次
    pub fn commit(mut self) -> EmbeddedResult<()> {
        if self.committed || self.rolled_back {
            return Err(EmbeddedError::TransactionAlreadyFinished);
        }
        // 提交事务
        self.committed = true;
        todo!("实现事务提交")
    }
    
    /// 回滚事务
    /// 
    /// 消费 self，确保事务只能回滚一次
    pub fn rollback(mut self) -> EmbeddedResult<()> {
        if self.committed || self.rolled_back {
            return Err(EmbeddedError::TransactionAlreadyFinished);
        }
        // 回滚事务
        self.rolled_back = true;
        todo!("实现事务回滚")
    }
    
    /// 创建保存点
    /// 
    /// 保存点允许部分回滚事务
    pub fn savepoint(&self, name: &str) -> EmbeddedResult<Savepoint> {
        Savepoint::new(self, name)
    }
}

impl Drop for Transaction {
    fn drop(&mut self) {
        // 如果事务未完成，自动回滚
        if !self.committed && !self.rolled_back {
            // 自动回滚
        }
    }
}

/// 保存点
/// 
/// 允许在事务内部设置回滚点
pub struct Savepoint<'txn> {
    transaction: &'txn Transaction,
    name: String,
    active: bool,
}

impl<'txn> Savepoint<'txn> {
    fn new(transaction: &'txn Transaction, name: &str) -> EmbeddedResult<Self> {
        // 创建保存点
        todo!("实现保存点创建")
    }
    
    /// 回滚到保存点
    /// 
    /// 消费 self，确保保存点只能回滚一次
    pub fn rollback_to(mut self) -> EmbeddedResult<()> {
        // 回滚到保存点
        self.active = false;
        todo!("实现保存点回滚")
    }
    
    /// 释放保存点
    /// 
    /// 释放后不能再回滚到该保存点
    pub fn release(mut self) -> EmbeddedResult<()> {
        // 释放保存点
        self.active = false;
        Ok(())
    }
}
```

### 5. `embedded/result.rs` - 结果集处理

```rust
use crate::core::value::Value;

/// 查询结果集
/// 
/// 封装查询返回的数据和元数据
pub struct QueryResult {
    columns: Vec<String>,
    rows: Vec<Row>,
    metadata: ResultMetadata,
}

/// 结果行
pub struct Row {
    values: std::collections::HashMap<String, Value>,
    column_order: Vec<String>,
}

/// 结果元数据
#[derive(Debug, Clone)]
pub struct ResultMetadata {
    /// 查询执行时间
    pub execution_time: std::time::Duration,
    /// 返回行数
    pub rows_returned: usize,
    /// 扫描行数
    pub rows_scanned: usize,
    /// 是否命中缓存
    pub is_cache_hit: bool,
    /// 查询计划（可选）
    pub plan: Option<String>,
}

impl QueryResult {
    /// 获取列名列表
    pub fn columns(&self) -> &[String] {
        &self.columns
    }
    
    /// 获取行数
    pub fn len(&self) -> usize {
        self.rows.len()
    }
    
    /// 是否为空结果
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
    
    /// 获取指定行
    pub fn get(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }
    
    /// 获取第一行（便捷方法）
    pub fn first(&self) -> Option<&Row> {
        self.rows.first()
    }
    
    /// 迭代器
    pub fn iter(&self) -> impl Iterator<Item = &Row> {
        self.rows.iter()
    }
    
    /// 获取元数据
    pub fn metadata(&self) -> &ResultMetadata {
        &self.metadata
    }
    
    /// 转换为 JSON 字符串
    pub fn to_json(&self) -> EmbeddedResult<String> {
        todo!("实现 JSON 序列化")
    }
    
    /// 转换为 CSV 字符串
    pub fn to_csv(&self) -> EmbeddedResult<String> {
        todo!("实现 CSV 序列化")
    }
}

impl Row {
    /// 按列名获取值
    pub fn get(&self, column: &str) -> Option<&Value> {
        self.values.get(column)
    }
    
    /// 按索引获取值
    pub fn get_by_index(&self, index: usize) -> Option<&Value> {
        self.column_order.get(index)
            .and_then(|col| self.values.get(col))
    }
    
    /// 获取字符串值
    pub fn get_string(&self, column: &str) -> Option<String> {
        self.get(column).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        })
    }
    
    /// 获取整数值
    pub fn get_int(&self, column: &str) -> Option<i64> {
        self.get(column).and_then(|v| match v {
            Value::Int(i) => Some(*i),
            _ => None,
        })
    }
    
    /// 获取浮点值
    pub fn get_float(&self, column: &str) -> Option<f64> {
        self.get(column).and_then(|v| match v {
            Value::Float(f) => Some(*f),
            Value::Double(d) => Some(*d),
            _ => None,
        })
    }
    
    /// 获取布尔值
    pub fn get_bool(&self, column: &str) -> Option<bool> {
        self.get(column).and_then(|v| match v {
            Value::Bool(b) => Some(*b),
            _ => None,
        })
    }
    
    /// 获取顶点值
    pub fn get_vertex(&self, column: &str) -> Option<&crate::core::vertex_edge_path::Vertex> {
        self.get(column).and_then(|v| match v {
            Value::Vertex(vertex) => Some(vertex.as_ref()),
            _ => None,
        })
    }
    
    /// 获取边值
    pub fn get_edge(&self, column: &str) -> Option<&crate::core::vertex_edge_path::Edge> {
        self.get(column).and_then(|v| match v {
            Value::Edge(edge) => Some(edge),
            _ => None,
        })
    }
    
    /// 获取路径值
    pub fn get_path(&self, column: &str) -> Option<&crate::core::vertex_edge_path::Path> {
        self.get(column).and_then(|v| match v {
            Value::Path(path) => Some(path),
            _ => None,
        })
    }
}
```

### 6. `embedded/error.rs` - 错误处理

```rust
use thiserror::Error;

/// 嵌入式 API 专用错误类型
#[derive(Error, Debug)]
pub enum EmbeddedError {
    #[error("数据库连接失败: {0}")]
    ConnectionFailed(String),
    
    #[error("数据库已关闭")]
    DatabaseClosed,
    
    #[error("查询执行失败: {0}")]
    QueryExecutionFailed(String),
    
    #[error("语法错误: {0}")]
    SyntaxError(String),
    
    #[error("事务已结束")]
    TransactionAlreadyFinished,
    
    #[error("没有活动事务")]
    NoActiveTransaction,
    
    #[error("保存点不存在: {0}")]
    SavepointNotFound(String),
    
    #[error("图空间不存在: {0}")]
    SpaceNotFound(String),
    
    #[error("存储错误: {0}")]
    StorageError(String),
    
    #[error("序列化错误: {0}")]
    SerializationError(String),
    
    #[error("参数绑定错误: {0}")]
    ParameterBindingError(String),
    
    #[error("超时")]
    Timeout,
    
    #[error("内部错误: {0}")]
    Internal(String),
}

/// 嵌入式 API 结果类型
pub type EmbeddedResult<T> = Result<T, EmbeddedError>;

impl From<crate::core::error::QueryError> for EmbeddedError {
    fn from(err: crate::core::error::QueryError) -> Self {
        EmbeddedError::QueryExecutionFailed(err.to_string())
    }
}

impl From<crate::storage::StorageError> for EmbeddedError {
    fn from(err: crate::storage::StorageError) -> Self {
        EmbeddedError::StorageError(err.to_string())
    }
}
```

---

## 与现有代码的集成关系

### 复用关系图

```
┌─────────────────────────────────────────────────────────────────┐
│                        应用层 (Application)                      │
│  ┌─────────────────────┐        ┌─────────────────────────────┐ │
│  │   网络服务应用       │        │      嵌入式应用              │ │
│  │  (HTTP/RPC Server)  │        │  (CLI Tool/Library)         │ │
│  └──────────┬──────────┘        └─────────────┬───────────────┘ │
└─────────────┼─────────────────────────────────┼─────────────────┘
              │                                 │
              ▼                                 ▼
┌─────────────────────────────────┐  ┌─────────────────────────────┐
│  src/api/service/               │  │  src/api/embedded/          │
│  - GraphService                 │  │  - GraphDatabase            │
│  - Authenticator                │  │  - Session                  │
│  - PermissionManager            │  │  - Transaction              │
│  - ClientSession (网络版)        │  │  - QueryResult              │
└───────────────┬─────────────────┘  └─────────────┬───────────────┘
                │                                  │
                └──────────────┬───────────────────┘
                               │
                               ▼
        ┌────────────────────────────────────────────────┐
        │              核心引擎层 (Core Engine)           │
        │  ┌─────────────┐  ┌─────────────┐  ┌─────────┐ │
        │  │   query/    │  │  storage/   │  │transaction│
        │  │ QueryEngine │  │StorageClient│  │TransactionManager│
        │  └─────────────┘  └─────────────┘  └─────────┘ │
        └────────────────────────────────────────────────┘
```

### 复用组件清单

| 组件 | 位置 | 复用方式 |
|-----|------|---------|
| 查询引擎 | `query::QueryEngine` | 直接实例化调用 |
| 存储引擎 | `storage::StorageClient` | 通过 Arc<dyn> 持有 |
| 事务管理 | `transaction::TransactionManager` | 直接实例化调用 |
| 值类型 | `core::value::Value` | 直接使用 |
| 图类型 | `core::vertex_edge_path::{Vertex, Edge, Path}` | 直接使用 |
| 错误类型 | `core::error::QueryError` | 转换后使用 |

---

## 实施步骤

### 第一阶段：核心框架（P0）

1. **创建目录结构**
   ```bash
   mkdir -p src/api/embedded/ffi
   touch src/api/embedded/mod.rs
   touch src/api/embedded/database.rs
   touch src/api/embedded/session.rs
   touch src/api/embedded/transaction.rs
   touch src/api/embedded/result.rs
   touch src/api/embedded/error.rs
   touch src/api/embedded/types.rs
   touch src/api/embedded/ffi/mod.rs
   ```

2. **实现错误类型** (`error.rs`)
   - 定义 `EmbeddedError` 枚举
   - 实现与现有错误类型的转换

3. **实现数据库实例** (`database.rs`)
   - 实现 `GraphDatabase::open_in_memory()`
   - 实现 `GraphDatabase::session()`

4. **实现基础会话** (`session.rs`)
   - 实现 `Session::execute()`
   - 实现 `Session::use_space()`

5. **实现结果集** (`result.rs`)
   - 实现 `QueryResult` 和 `Row`
   - 实现基础类型获取方法

6. **更新模块导出** (`src/api/mod.rs`)
   - 添加 `pub mod embedded;`

### 第二阶段：事务支持（P1）

1. **实现事务** (`transaction.rs`)
   - 实现 `Transaction::execute()`
   - 实现 `Transaction::commit()` / `rollback()`

2. **实现保存点** (`transaction.rs`)
   - 实现 `Savepoint::rollback_to()`

3. **实现托管事务** (`session.rs`)
   - 实现 `Session::with_transaction()`

### 第三阶段：高级特性（P2）

1. **实现预编译语句** (`statement.rs`)
   - 实现 `PreparedStatement::bind()`
   - 实现 `PreparedStatement::execute()`

2. **实现批量操作** (`batch.rs`)
   - 实现 `BatchInserter`

3. **实现文件模式** (`database.rs`)
   - 实现 `GraphDatabase::open()`

### 第四阶段：FFI 绑定（P3）

1. **实现 C API** (`ffi/c_api.rs`)
   - 实现基础连接/查询接口
   - 实现类型转换

2. **添加 FFI 特性门控**
   - 在 `Cargo.toml` 添加 `ffi` feature

---

## 与现有代码的兼容性

### 不破坏现有功能

- `service/` 和 `session/` 保持完全不变
- 现有网络服务可以继续正常运行
- 新增 `embedded/` 是独立的附加功能

### 共享核心引擎

- 不重复实现查询、存储、事务逻辑
- 通过核心引擎层复用代码
- 保持行为一致性

### 渐进式迁移（可选）

如果未来需要，可以考虑：
- 将 `service/` 中的部分逻辑下沉到核心引擎
- 让 `service/` 和 `embedded/` 共享更多代码
- 但这不是必须的，当前设计已足够清晰

---

## 总结

方案A通过新建 `embedded` 子模块，实现了：

1. ✅ **职责清晰分离**：嵌入式 vs 网络服务完全独立
2. ✅ **底层代码复用**：共用 query、storage、transaction 核心
3. ✅ **零成本抽象**：嵌入式 API 直接操作引擎，无中间层
4. ✅ **渐进式实现**：可分阶段开发，不影响现有功能
5. ✅ **扩展性良好**：易于添加 FFI、异步等高级特性

这是实现 GraphDB 嵌入式数据库 API 的最佳方案。
