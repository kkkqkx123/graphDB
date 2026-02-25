# GraphDB 嵌入式 API 扩展方案（简化版）

## 概述

本文档基于 [embedded_api_design.md](./embedded_api_design.md) 的设计规范，分析 `src\api\embedded` 目录现状与目标设计的差异，提供**简化后**的扩展实施方案。

> **重要**: 本方案已移除过度设计的功能（C FFI、异步 API、保存点等），专注于嵌入式场景的核心需求。

---

## 现状分析

### 当前目录结构

```
src\api\embedded\
├── mod.rs              # 模块入口，导出 GraphDb 和 EmbeddedConfig
└── embedded_api.rs     # 核心实现（237行）
```

### 当前已实现功能

| 功能 | 状态 | 说明 |
|------|------|------|
| GraphDb 结构体 | ✅ | 包含 query_api, txn_api, schema_api, storage, txn_manager |
| open() | ⚠️ | 接口存在，实现为 todo!() |
| execute() | ✅ | 基础实现完成 |
| execute_with_params() | ✅ | 基础实现完成 |
| begin_transaction() | ✅ | 基础实现完成 |
| execute_in_transaction() | ✅ | 基础实现完成 |
| commit_transaction() | ✅ | 基础实现完成 |
| rollback_transaction() | ✅ | 基础实现完成 |
| close() | ⚠️ | 接口存在，实现为空 |
| EmbeddedConfig | ✅ | 配置结构体完成 |

### 当前缺失功能（相对于简化后设计）

| 功能 | 优先级 | 说明 |
|------|--------|------|
| Session 概念 | P0 | 设计文档中的核心抽象，当前缺失 |
| 预编译语句 (PreparedStatement) | P1 | 高性能查询支持 |
| 批量操作 (BatchInserter) | P1 | 大批量数据导入 |
| QueryResult 完整实现 | P1 | 结果集遍历、类型转换等 |

---

## 扩展方案（简化版）

### 1. 目录结构重构

```
src\api\embedded\
├── mod.rs                    # 模块入口
├── database.rs               # GraphDatabase 实现（原 embedded_api.rs 重构）
├── session.rs                # Session 管理（新增）
├── transaction.rs            # 事务管理（新增，简化版，无保存点）
├── statement.rs              # 预编译语句（新增）
├── result.rs                 # 查询结果处理（新增）
├── batch.rs                  # 批量操作（新增）
└── config.rs                 # 配置管理（从 embedded_api.rs 分离）
```

> **注意**: 已移除以下过度设计的模块：
> - `async_api.rs` - 异步 API（嵌入式场景不需要）
> - `ffi.rs` - C FFI 绑定（优先使用专用绑定库）

---

### 2. 详细扩展计划

#### 2.1 Session 模块（新增）session.rs

**设计目标**：实现设计文档中的 Session 概念，作为查询执行的上下文。

**核心结构**：
```rust
/// 会话 - 执行上下文
pub struct Session<S: StorageClient> {
    db: Arc<GraphDatabase<S>>,
    space_id: Option<u64>,
    space_name: Option<String>,
    auto_commit: bool,
}

impl<S: StorageClient> Session<S> {
    /// 切换图空间
    pub fn use_space(&mut self, space_name: &str) -> Result<(), EmbeddedError>;
    
    /// 执行查询
    pub fn execute(&self, query: &str) -> Result<QueryResult, EmbeddedError>;
    
    /// 执行参数化查询
    pub fn execute_with_params(
        &self,
        query: &str,
        params: HashMap<String, Value>
    ) -> Result<QueryResult, EmbeddedError>;
    
    /// 开始事务
    pub fn begin_transaction(&self) -> Result<Transaction<S>, EmbeddedError>;
    
    /// 获取当前图空间
    pub fn current_space(&self) -> Option<&str>;
    
    /// 创建批量插入器
    pub fn batch_inserter(&self, batch_size: usize) -> BatchInserter<S>;
}
```

**与现有代码的关系**：
- 将 `GraphDb::execute*` 方法迁移到 Session
- GraphDb 增加 `session()` 方法创建会话
- 保持向后兼容，GraphDb 保留快捷方法

---

#### 2.2 事务模块（简化版）transaction.rs

**设计目标**：简化事务管理，**移除保存点支持**。

**核心结构**：
```rust
/// 事务句柄
pub struct Transaction<'sess, S: StorageClient> {
    session: &'sess Session<S>,
    txn_handle: TransactionHandle,
    committed: bool,
}

impl<'sess, S: StorageClient> Transaction<'sess, S> {
    /// 在事务中执行查询
    pub fn execute(&self, query: &str) -> Result<QueryResult, EmbeddedError>;
    
    /// 执行参数化查询
    pub fn execute_with_params(
        &self,
        query: &str,
        params: HashMap<String, Value>
    ) -> Result<QueryResult, EmbeddedError>;
    
    /// 提交事务
    pub fn commit(mut self) -> Result<(), EmbeddedError>;
    
    /// 回滚事务
    pub fn rollback(mut self) -> Result<(), EmbeddedError>;
}

/// 托管事务（简化版）
impl<S: StorageClient> Session<S> {
    /// 在事务中执行操作（自动提交/回滚）
    pub fn with_transaction<F, T>(&self, f: F) -> Result<T, EmbeddedError>
    where
        F: FnOnce(&Transaction<S>) -> Result<T, EmbeddedError>;
}
```

**与现有代码的关系**：
- 替换现有的 `TransactionHandle` 裸句柄方式
- 利用生命周期确保事务安全
- 复用 `api::core::TransactionApi` 底层能力
- ~~移除 Savepoint 相关代码~~

---

#### 2.3 预编译语句模块（新增）statement.rs

**设计目标**：实现高性能的预编译查询支持。

**核心结构**：
```rust
/// 预编译语句
pub struct PreparedStatement<S: StorageClient> {
    query_plan: Arc<ExecutionPlan>,
    parameter_types: HashMap<String, DataType>,
    query_api: QueryApi<S>,
    bound_params: HashMap<String, Value>,
}

impl<S: StorageClient> PreparedStatement<S> {
    /// 绑定参数
    pub fn bind(&mut self, name: &str, value: Value) -> Result<(), EmbeddedError>;
    
    /// 执行（返回结果集）
    pub fn execute(&self) -> Result<QueryResult, EmbeddedError>;
    
    /// 执行更新（返回影响行数）
    pub fn execute_update(&self) -> Result<usize, EmbeddedError>;
    
    /// 重置语句（可重复执行）
    pub fn reset(&mut self);
    
    /// 清除参数绑定
    pub fn clear_bindings(&mut self);
}

impl<S: StorageClient> GraphDatabase<S> {
    /// 预编译查询
    pub fn prepare(&self, query: &str) -> Result<PreparedStatement<S>, EmbeddedError>;
}
```

**依赖需求**：
- 需要查询引擎支持执行计划缓存
- 需要参数类型推断能力

---

#### 2.4 结果集模块（新增）result.rs

**设计目标**：提供完善的查询结果处理能力。

**核心结构**：
```rust
/// 查询结果
pub struct QueryResult {
    columns: Vec<String>,
    rows: Vec<Row>,
    metadata: ResultMetadata,
}

/// 结果行
pub struct Row {
    values: HashMap<String, Value>,
    column_index: HashMap<String, usize>,
}

/// 结果元数据（简化版）
pub struct ResultMetadata {
    pub execution_time: Duration,
    pub rows_returned: usize,
    pub rows_scanned: usize,
}

impl QueryResult {
    /// 获取列名
    pub fn columns(&self) -> &[String];
    
    /// 获取行数
    pub fn len(&self) -> usize;
    
    /// 是否为空
    pub fn is_empty(&self) -> bool;
    
    /// 获取指定行
    pub fn get(&self, index: usize) -> Option<&Row>;
    
    /// 迭代器
    pub fn iter(&self) -> impl Iterator<Item = &Row>;
    
    /// 转换为 JSON
    pub fn to_json(&self) -> Result<String, EmbeddedError>;
}

impl Row {
    /// 按列名获取值
    pub fn get(&self, column: &str) -> Option<&Value>;
    
    /// 按索引获取值
    pub fn get_by_index(&self, index: usize) -> Option<&Value>;
    
    /// 类型化获取方法
    pub fn get_string(&self, column: &str) -> Option<String>;
    pub fn get_int(&self, column: &str) -> Option<i64>;
    pub fn get_float(&self, column: &str) -> Option<f64>;
    pub fn get_bool(&self, column: &str) -> Option<bool>;
    pub fn get_vertex(&self, column: &str) -> Option<&Vertex>;
    pub fn get_edge(&self, column: &str) -> Option<&Edge>;
    pub fn get_path(&self, column: &str) -> Option<&Path>;
}
```

**与现有代码的关系**：
- 复用 `api::core::QueryResult` 和 `Row`
- 增加类型化获取方法和 JSON 序列化
- ~~移除 `is_cache_hit` 字段~~

---

#### 2.5 批量操作模块（新增）batch.rs

**设计目标**：支持高效的大批量数据导入。

**核心结构**：
```rust
/// 批量插入器
pub struct BatchInserter<'sess, S: StorageClient> {
    session: &'sess Session<S>,
    batch_size: usize,
    vertex_buffer: Vec<Vertex>,
    edge_buffer: Vec<Edge>,
    total_inserted: usize,
}

impl<'sess, S: StorageClient> BatchInserter<'sess, S> {
    /// 添加顶点
    pub fn add_vertex(&mut self, vertex: Vertex) -> &mut Self;
    
    /// 添加边
    pub fn add_edge(&mut self, edge: Edge) -> &mut Self;
    
    /// 执行批量插入
    pub fn execute(&mut self) -> Result<BatchResult, EmbeddedError>;
    
    /// 自动刷新（达到 batch_size 时自动执行）
    pub fn auto_flush(&mut self) -> Result<(), EmbeddedError>;
}

/// 批量操作结果
pub struct BatchResult {
    pub vertices_inserted: usize,
    pub edges_inserted: usize,
    pub errors: Vec<BatchError>,
}

/// 批量错误
pub struct BatchError {
    pub index: usize,
    pub item_type: BatchItemType,
    pub error: EmbeddedError,
}

pub enum BatchItemType {
    Vertex,
    Edge,
}

impl<S: StorageClient> Session<S> {
    /// 创建批量插入器
    pub fn batch_inserter(&self, batch_size: usize) -> BatchInserter<S>;
}
```

---

#### 2.6 配置模块（分离）config.rs

**设计目标**：将配置从 embedded_api.rs 分离，**移除连接池相关配置**。

**核心结构**：
```rust
/// 数据库配置（简化版）
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// 数据库路径，None 表示内存模式
    pub path: Option<PathBuf>,
    /// 缓存大小（MB）
    pub cache_size_mb: usize,
    /// 默认超时
    pub default_timeout: Duration,
}

impl DatabaseConfig {
    /// 内存数据库配置
    pub fn memory() -> Self;
    
    /// 文件数据库配置
    pub fn file(path: impl Into<PathBuf>) -> Self;
    
    /// 链式配置方法
    pub fn with_cache_size(mut self, size_mb: usize) -> Self;
    pub fn with_timeout(mut self, timeout: Duration) -> Self;
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            path: None,              // 默认内存模式
            cache_size_mb: 64,
            default_timeout: Duration::from_secs(30),
        }
    }
}
```

**与现有代码的关系**：
- 扩展现有的 `EmbeddedConfig`
- ~~移除 `max_connections` 字段~~（嵌入式不需要连接池）
- 保持 `Default` 实现提供合理默认值

---

#### 2.7 数据库主模块（重构）database.rs

**设计目标**：重构现有的 `embedded_api.rs`，引入 Session 概念。

**核心结构**：
```rust
/// 数据库实例 - 对应 SQLite 的 sqlite3
pub struct GraphDatabase<S: StorageClient> {
    storage: Arc<S>,
    transaction_manager: Arc<TransactionManager>,
    config: DatabaseConfig,
    query_api: QueryApi<S>,
    txn_api: TransactionApi,
    schema_api: SchemaApi<S>,
}

impl<S: StorageClient> GraphDatabase<S> {
    /// 打开数据库（文件模式）
    pub fn open(path: impl AsRef<Path>) -> Result<Self, EmbeddedError>;
    
    /// 创建内存数据库
    pub fn open_in_memory() -> Result<Self, EmbeddedError>;
    
    /// 使用配置打开
    pub fn open_with_config(config: DatabaseConfig) -> Result<Self, EmbeddedError>;
    
    /// 创建会话
    pub fn session(&self) -> Result<Session<S>, EmbeddedError>;
    
    /// 执行简单查询（便捷方法）
    pub fn execute(&self, query: &str) -> Result<QueryResult, EmbeddedError>;
    
    /// 执行参数化查询（便捷方法）
    pub fn execute_with_params(
        &self,
        query: &str,
        params: HashMap<String, Value>
    ) -> Result<QueryResult, EmbeddedError>;
    
    /// 预编译语句
    pub fn prepare(&self, query: &str) -> Result<PreparedStatement<S>, EmbeddedError>;
    
    /// 关闭数据库
    pub fn close(self) -> Result<(), EmbeddedError>;
}
```

---

### 3. 模块导出更新（mod.rs）

```rust
//! 嵌入式 API 模块
//!
//! 提供单机使用的嵌入式 GraphDB 接口，类似 SQLite 的使用方式

// 子模块
pub mod config;
pub mod database;
pub mod session;
pub mod transaction;
pub mod result;
pub mod statement;
pub mod batch;

// 重新导出主要类型
pub use config::DatabaseConfig;
pub use database::GraphDatabase;
pub use session::Session;
pub use transaction::Transaction;
pub use result::{QueryResult, Row, ResultMetadata};
pub use statement::PreparedStatement;
pub use batch::{BatchInserter, BatchResult, BatchError};

// 错误类型
pub use crate::api::core::CoreError as EmbeddedError;

// 向后兼容导出（废弃警告）
#[deprecated(since = "0.2.0", note = "使用 GraphDatabase 替代")]
pub use database::GraphDatabase as GraphDb;

#[deprecated(since = "0.2.0", note = "使用 DatabaseConfig 替代")]
pub use config::DatabaseConfig as EmbeddedConfig;
```

---

### 4. Cargo.toml 更新（简化版）

```toml
[features]
default = []
# 已移除：和 ffi 特性（过度设计）

[dependencies]
# 现有依赖...

# JSON 序列化（用于 QueryResult::to_json）
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# 已移除：
# - tokio（异步支持）- 嵌入式场景不需要专门的异步 API
# - libc（FFI 支持）- 优先使用专用绑定库
```

---

### 5. 实施优先级（简化后）

| 阶段 | 模块 | 优先级 | 预估工作量 | 依赖 |
|------|------|--------|-----------|------|
| 1 | result.rs | P0 | 1天 | api::core::QueryResult |
| 1 | config.rs | P0 | 0.5天 | 现有 EmbeddedConfig |
| 2 | session.rs | P0 | 2天 | result.rs, config.rs |
| 2 | database.rs | P0 | 2天 | session.rs（重构现有代码） |
| 3 | transaction.rs | P1 | 1天 | session.rs（简化版，无保存点） |
| 3 | statement.rs | P1 | 2天 | 查询引擎支持 |
| 4 | batch.rs | P1 | 1.5天 | session.rs |

> **注意**: 已移除以下模块的实施计划：
> - ~~async_api.rs~~ - 异步 API（P2 → 移除）
> - ~~ffi.rs~~ - C FFI 绑定（P1 → 移除）

---

### 6. 与现有代码的兼容性

#### 6.1 向后兼容策略

```rust
// 在 mod.rs 中提供兼容层
#[deprecated(since = "0.2.0", note = "使用 GraphDatabase::session() 替代")]
pub struct GraphDb<S: StorageClient> {
    inner: GraphDatabase<S>,
}

impl<S: StorageClient> GraphDb<S> {
    #[deprecated(since = "0.2.0", note = "使用 GraphDatabase::open() 替代")]
    pub fn open(path: &str) -> Result<Self, EmbeddedError> {
        let inner = GraphDatabase::open(path)?;
        Ok(Self { inner })
    }
    
    // 其他方法的兼容包装...
}
```

#### 6.2 迁移示例

```rust
// 旧代码（0.1.x）
let db = GraphDb::open("my_db")?;
let result = db.execute("MATCH (n) RETURN n")?;

// 新代码（0.2.x）
let db = GraphDatabase::open("my_db")?;
let session = db.session()?;
let result = session.execute("MATCH (n) RETURN n")?;
```

---

### 7. 测试策略

#### 7.1 单元测试

每个模块应包含对应的单元测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_session_use_space() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on({
            let db = GraphDatabase::open_in_memory().unwrap();
            let mut session = db.session().unwrap();
            session.use_space("test_space").unwrap();
            assert_eq!(session.current_space(), Some("test_space"));
        });
    }
}
```

#### 7.2 集成测试

在 `tests/` 目录下创建嵌入式 API 集成测试：

```
tests/
├── embedded_basic.rs      # 基础功能测试
├── embedded_transaction.rs # 事务测试（简化版，无保存点）
└── embedded_batch.rs      # 批量操作测试
```

---

## 移除的功能说明

### 1. ~~异步 API 模块~~（已移除）

**原设计**: `async_api.rs` 提供 `AsyncGraphDatabase`、`AsyncSession` 等异步接口

**移除原因**:
- 嵌入式数据库通常在同一线程/进程内使用
- SQLite 没有专门的异步 API，同步 API 已满足需求
- 存储引擎层面的 I/O 异步已足够

**替代方案**:
```rust
// 如需异步，在应用层使用 spawn_blocking
let result = tokio::task::spawn_blocking(move || {
    let db = GraphDatabase::open("my_db")?;
    let session = db.session()?;
    session.execute("MATCH (n) RETURN n")
})?;
```

### 2. ~~C FFI 模块~~（已移除）

**原设计**: `ffi.rs` 提供 C 语言兼容的 FFI 接口

**移除原因**:
- 项目当前是单机嵌入式数据库，目标用户主要是 Rust 开发者
- C FFI 层开发成本高，且需要维护大量 unsafe 代码
- 根据 `rust与C作为嵌入数据库的区别.txt` 分析，专用绑定库更高效

**替代方案**:
- Python: 使用 PyO3 直接包装 Rust API
- Node.js: 使用 Napi-rs 直接包装 Rust API
- 其他语言: 通过专用绑定生成器

### 3. ~~保存点 Savepoint~~（已移除）

**原设计**: `transaction.rs` 中包含 `Savepoint` 结构体和 `savepoint()` 方法

**移除原因**:
- 嵌入式场景事务通常简单，嵌套事务需求较少
- 增加 API 复杂度和实现难度

**替代方案**:
- 如需嵌套事务，可在应用层实现逻辑隔离

### 4. ~~流式查询 execute_stream~~（已移除）

**原设计**: `AsyncSession::execute_stream()` 返回 `Stream<Item = Result<Row, Error>>`

**移除原因**:
- 嵌入式数据库结果集通常在内存中处理
- 流式查询增加实现复杂度
- 大数据量查询可通过 LIMIT/OFFSET 分页

### 5. ~~连接池配置~~（已移除）

**原设计**: `DatabaseConfig` 包含 `max_connections` 字段

**移除原因**:
- 嵌入式数据库是单进程访问，不需要连接池
- 简化配置项

---

## 总结

本简化版扩展方案将 `src\api\embedded` 从当前的单一文件结构扩展为模块化的架构，但**已移除过度设计的功能**：

### 保留的核心功能
1. **核心层**: Session、Transaction（简化版）、QueryResult
2. **高级层**: PreparedStatement、BatchInserter
3. **配置层**: DatabaseConfig（简化版）

### 已移除的过度设计
| 功能 | 移除原因 | 替代方案 |
|------|---------|---------|
| 异步 API | 嵌入式场景不需要 | 应用层使用 spawn_blocking |
| C FFI | 开发成本高，维护困难 | 专用绑定库（PyO3/Napi-rs）|
| 保存点 | 嵌入式场景需求少 | 应用层逻辑隔离 |
| 流式查询 | 增加复杂度 | LIMIT/OFFSET 分页 |
| 连接池 | 单进程访问不需要 | 移除配置项 |

通过务实的简化，本方案专注于嵌入式场景的核心需求，降低实现复杂度，同时保持 API 的简洁性和可用性。
