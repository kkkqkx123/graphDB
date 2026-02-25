# GraphDB 嵌入式数据库 API 设计方案

## 概述

本文档定义 GraphDB 作为嵌入式数据库的完整 API 设计方案，参考 SQLite 的简洁性和 Neo4j 的类型安全性，为 Rust 应用提供原生图数据库支持。

---

## 设计原则

1. **简洁性**: 像 SQLite 一样，用最少的 API 完成最常见的操作
2. **类型安全**: 利用 Rust 的类型系统，在编译期捕获错误
3. **零成本抽象**: 高级 API 不带来运行时开销
4. **可扩展性**: 支持从简单脚本到复杂应用的多种使用场景

---

## API 架构层次

```
┌─────────────────────────────────────────────────────────────┐
│                    公共 API 层 (Public API)                   │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │  同步 API     │  │  异步 API     │  │   C FFI API      │  │
│  │  (embedded)  │  │  (embedded)  │  │  (跨语言绑定)     │  │
│  └──────────────┘  └──────────────┘  └──────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│                   核心引擎层 (Core Engine)                    │
│         查询解析 → 计划生成 → 执行引擎 → 存储层               │
└─────────────────────────────────────────────────────────────┘
```

---

## 核心 API 设计

### 1. 数据库连接管理

```rust
/// 数据库实例 - 对应 SQLite 的 sqlite3
pub struct GraphDatabase {
    storage: Arc<StorageEngine>,
    transaction_manager: Arc<TransactionManager>,
    config: DatabaseConfig,
}

/// 数据库配置
pub struct DatabaseConfig {
    pub path: Option<PathBuf>,           // None = 内存模式
    pub cache_size: usize,               // 缓存大小
    pub max_connections: usize,          // 最大连接数
    pub default_timeout: Duration,       // 默认超时
}

impl GraphDatabase {
    /// 打开数据库（文件模式）
    /// 对应: sqlite3_open()
    pub fn open(path: impl AsRef<Path>) -> Result<Self, GraphDbError>;
    
    /// 创建内存数据库
    /// 对应: sqlite3_open(":memory:")
    pub fn open_in_memory() -> Result<Self, GraphDbError>;
    
    /// 关闭数据库
    /// 对应: sqlite3_close()
    pub fn close(self) -> Result<(), GraphDbError>;
    
    /// 创建会话
    /// 对应: Neo4j 的 session()
    pub fn session(&self) -> Result<Session, GraphDbError>;
    
    /// 执行简单查询（便捷方法）
    /// 对应: sqlite3_exec()
    pub fn execute(&self, query: &str) -> Result<QueryResult, GraphDbError>;
    
    /// 执行参数化查询
    pub fn execute_with_params(
        &self, 
        query: &str, 
        params: &HashMap<String, Value>
    ) -> Result<QueryResult, GraphDbError>;
}
```

### 2. 会话管理

```rust
/// 会话 - 执行上下文
pub struct Session {
    db: Arc<GraphDatabase>,
    space: Option<String>,               // 当前图空间
    auto_commit: bool,                   // 自动提交模式
}

impl Session {
    /// 切换图空间
    /// 对应: USE <space>
    pub fn use_space(&mut self, space_name: &str) -> Result<(), GraphDbError>;
    
    /// 执行查询
    /// 对应: Neo4j session.run()
    pub fn execute(&self, query: &str) -> Result<QueryResult, GraphDbError>;
    
    /// 执行参数化查询
    pub fn execute_with_params(
        &self,
        query: &str,
        params: &HashMap<String, Value>
    ) -> Result<QueryResult, GraphDbError>;
    
    /// 开始事务
    /// 对应: sqlite3_begin_transaction()
    pub fn begin_transaction(&mut self) -> Result<Transaction, GraphDbError>;
    
    /// 设置自动提交
    pub fn set_auto_commit(&mut self, enabled: bool);
    
    /// 检查是否自动提交
    pub fn is_auto_commit(&self) -> bool;
    
    /// 获取当前图空间
    pub fn current_space(&self) -> Option<&str>;
}
```

### 3. 事务管理

```rust
/// 事务句柄
pub struct Transaction<'sess> {
    session: &'sess mut Session,
    txn_id: TransactionId,
    committed: bool,
}

impl<'sess> Transaction<'sess> {
    /// 在事务中执行查询
    pub fn execute(&self, query: &str) -> Result<QueryResult, GraphDbError>;
    
    /// 执行参数化查询
    pub fn execute_with_params(
        &self,
        query: &str,
        params: &HashMap<String, Value>
    ) -> Result<QueryResult, GraphDbError>;
    
    /// 提交事务
    /// 对应: sqlite3_commit()
    pub fn commit(mut self) -> Result<(), GraphDbError>;
    
    /// 回滚事务
    /// 对应: sqlite3_rollback()
    pub fn rollback(mut self) -> Result<(), GraphDbError>;
    
    /// 创建保存点
    pub fn savepoint(&self, name: &str) -> Result<Savepoint, GraphDbError>;
}

/// 保存点
pub struct Savepoint<'txn> {
    transaction: &'txn Transaction<'txn>,
    name: String,
}

impl<'txn> Savepoint<'txn> {
    /// 回滚到保存点
    pub fn rollback_to(self) -> Result<(), GraphDbError>;
    
    /// 释放保存点
    pub fn release(self) -> Result<(), GraphDbError>;
}

/// 托管事务（自动重试）
impl Session {
    /// 执行写操作事务（带重试）
    /// 对应: Neo4j execute_write()
    pub fn with_write_transaction<F, T>(&self, f: F) -> Result<T, GraphDbError>
    where
        F: FnMut(&Transaction) -> Result<T, GraphDbError>;
    
    /// 执行读操作事务
    /// 对应: Neo4j execute_read()
    pub fn with_read_transaction<F, T>(&self, f: F) -> Result<T, GraphDbError>
    where
        F: FnMut(&Transaction) -> Result<T, GraphDbError>;
}
```

### 4. 查询结果处理

```rust
/// 查询结果
pub struct QueryResult {
    columns: Vec<String>,                // 列名
    rows: Vec<Row>,                      // 数据行
    metadata: ResultMetadata,            // 元数据
}

/// 结果行
pub struct Row {
    values: HashMap<String, Value>,
}

/// 结果元数据
pub struct ResultMetadata {
    pub execution_time: Duration,
    pub rows_returned: usize,
    pub rows_scanned: usize,
    pub is_cache_hit: bool,
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
    pub fn to_json(&self) -> Result<String, GraphDbError>;
}

impl Row {
    /// 按列名获取值
    pub fn get(&self, column: &str) -> Option<&Value>;
    
    /// 按索引获取值
    pub fn get_by_index(&self, index: usize) -> Option<&Value>;
    
    /// 获取指定类型值
    pub fn get_string(&self, column: &str) -> Option<String>;
    pub fn get_int(&self, column: &str) -> Option<i64>;
    pub fn get_float(&self, column: &str) -> Option<f64>;
    pub fn get_bool(&self, column: &str) -> Option<bool>;
    pub fn get_vertex(&self, column: &str) -> Option<&Vertex>;
    pub fn get_edge(&self, column: &str) -> Option<&Edge>;
    pub fn get_path(&self, column: &str) -> Option<&Path>;
}
```

### 5. 预编译语句（高性能）

```rust
/// 预编译语句
/// 对应: sqlite3_stmt
pub struct PreparedStatement {
    query_plan: Arc<ExecutionPlan>,
    parameter_types: HashMap<String, DataType>,
}

impl GraphDatabase {
    /// 预编译查询
    /// 对应: sqlite3_prepare_v2()
    pub fn prepare(&self, query: &str) -> Result<PreparedStatement, GraphDbError>;
}

impl PreparedStatement {
    /// 绑定参数
    /// 对应: sqlite3_bind_*()
    pub fn bind(&mut self, name: &str, value: Value) -> Result<(), GraphDbError>;
    
    /// 执行（返回结果）
    /// 对应: sqlite3_step() + 结果获取
    pub fn execute(&self) -> Result<QueryResult, GraphDbError>;
    
    /// 执行（无结果）
    pub fn execute_update(&self) -> Result<usize, GraphDbError>; // 返回影响行数
    
    /// 重置语句（可重复执行）
    /// 对应: sqlite3_reset()
    pub fn reset(&mut self);
    
    /// 清除参数绑定
    pub fn clear_bindings(&mut self);
}
```

### 6. 批量操作 API

```rust
/// 批量插入器
pub struct BatchInserter {
    session: &Session,
    batch_size: usize,
    vertex_buffer: Vec<Vertex>,
    edge_buffer: Vec<Edge>,
}

impl Session {
    /// 创建批量插入器
    pub fn batch_inserter(&self, batch_size: usize) -> BatchInserter;
}

impl BatchInserter {
    /// 添加顶点
    pub fn add_vertex(&mut self, vertex: Vertex) -> &mut Self;
    
    /// 添加边
    pub fn add_edge(&mut self, edge: Edge) -> &mut Self;
    
    /// 执行批量插入
    pub fn execute(&mut self) -> Result<BatchResult, GraphDbError>;
    
    /// 自动刷新（达到 batch_size 时自动执行）
    pub fn auto_flush(&mut self) -> Result<(), GraphDbError>;
}

/// 批量操作结果
pub struct BatchResult {
    pub vertices_inserted: usize,
    pub edges_inserted: usize,
    pub errors: Vec<BatchError>,
}
```

---

## 使用示例

```rust
use graphdb::{GraphDatabase, Value, Vertex, Edge};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 打开数据库（文件模式）
    let db = GraphDatabase::open("./my_graph.db")?;
    
    // 2. 创建会话
    let mut session = db.session()?;
    
    // 3. 创建图空间
    session.execute("CREATE SPACE IF NOT EXISTS social_network")?;
    session.use_space("social_network")?;
    
    // 4. 创建 Schema
    session.execute("CREATE TAG IF NOT EXISTS Person(name string, age int)")?;
    session.execute("CREATE EDGE IF NOT EXISTS FRIEND(since timestamp)")?;
    
    // 5. 简单插入
    session.execute(r#"
        INSERT VERTEX Person(name, age) VALUES "alice":("Alice", 30)
    "#)?;
    
    // 6. 参数化查询
    let mut params = HashMap::new();
    params.insert("name".to_string(), Value::String("Bob".to_string()));
    params.insert("age".to_string(), Value::Int(25));
    
    session.execute_with_params(r#"
        INSERT VERTEX Person(name, age) VALUES "bob":($name, $age)
    "#, &params)?;
    
    // 7. 查询并处理结果
    let result = session.execute(r#"
        MATCH (p:Person) RETURN p.name, p.age
    "#)?;
    
    for row in result.iter() {
        let name = row.get_string("p.name").unwrap_or_default();
        let age = row.get_int("p.age").unwrap_or(0);
        println!("{}: {}", name, age);
    }
    
    // 8. 事务操作
    let mut txn = session.begin_transaction()?;
    
    txn.execute(r#"
        INSERT VERTEX Person(name, age) VALUES "carol":("Carol", 28)
    "#)?;
    
    txn.execute(r#"
        INSERT EDGE FRIEND(since) VALUES "alice"->"carol":(timestamp())
    "#)?;
    
    txn.commit()?;
    
    // 9. 预编译语句（高性能重复执行）
    let mut stmt = db.prepare(r#"
        MATCH (p:Person {name: $name}) RETURN p.age
    "#)?;
    
    for name in &["Alice", "Bob", "Carol"] {
        stmt.bind("name", Value::String(name.to_string()))?;
        let result = stmt.execute()?;
        if let Some(row) = result.get(0) {
            println!("{}'s age: {:?}", name, row.get_int("p.age"));
        }
        stmt.reset();
    }
    
    // 10. 批量插入
    let mut batch = session.batch_inserter(1000);
    for i in 0..10000 {
        batch.add_vertex(Vertex::new("Person")
            .property("name", format!("User{}", i))
            .property("age", i as i64 % 100)
        );
        
        // 每 1000 条自动执行
        if i % 1000 == 0 {
            batch.auto_flush()?;
        }
    }
    batch.execute()?; // 刷新剩余
    
    // 11. 关闭数据库
    db.close()?;
    
    Ok(())
}
```

---

## C FFI API

```c
// ==================== C FFI API ====================

// 数据库句柄
typedef struct graphdb graphdb;
typedef struct graphdb_session graphdb_session;
typedef struct graphdb_result graphdb_result;
typedef struct graphdb_stmt graphdb_stmt;

// 错误码
typedef enum {
    GRAPHDB_OK = 0,
    GRAPHDB_ERROR = 1,
    GRAPHDB_NOMEM = 2,
    GRAPHDB_BUSY = 3,
    GRAPHDB_NOTFOUND = 4,
    GRAPHDB_INVALID = 5,
} graphdb_error;

// 打开数据库
graphdb_error graphdb_open(const char* path, graphdb** db);

// 关闭数据库
graphdb_error graphdb_close(graphdb* db);

// 创建会话
graphdb_error graphdb_session(graphdb* db, graphdb_session** session);

// 执行查询
graphdb_error graphdb_execute(
    graphdb_session* session,
    const char* query,
    graphdb_result** result
);

// 获取结果行数
int graphdb_result_row_count(graphdb_result* result);

// 获取列数
int graphdb_result_column_count(graphdb_result* result);

// 获取列名
const char* graphdb_result_column_name(graphdb_result* result, int col);

// 获取字符串值
const char* graphdb_result_get_string(graphdb_result* result, int row, int col);

// 获取整数值
long long graphdb_result_get_int(graphdb_result* result, int row, int col);

// 释放结果
void graphdb_result_free(graphdb_result* result);

// 预编译语句
graphdb_error graphdb_prepare(
    graphdb* db,
    const char* query,
    graphdb_stmt** stmt
);

// 绑定参数
graphdb_error graphdb_bind_string(graphdb_stmt* stmt, const char* name, const char* value);
graphdb_error graphdb_bind_int(graphdb_stmt* stmt, const char* name, long long value);

// 执行预编译语句
graphdb_error graphdb_stmt_execute(graphdb_stmt* stmt, graphdb_result** result);

// 释放语句
void graphdb_stmt_free(graphdb_stmt* stmt);
```

---

## 异步 API

```rust
// ==================== 异步 API ====================

pub struct AsyncGraphDatabase {
    inner: GraphDatabase,
    runtime: Handle,
}

impl AsyncGraphDatabase {
    /// 异步打开数据库
    pub async fn open(path: impl AsRef<Path>) -> Result<Self, GraphDbError>;
    
    /// 异步创建会话
    pub async fn session(&self) -> Result<AsyncSession, GraphDbError>;
}

pub struct AsyncSession {
    inner: Session,
}

impl AsyncSession {
    /// 异步执行查询
    pub async fn execute(&self, query: &str) -> Result<QueryResult, GraphDbError>;
    
    /// 异步流式查询
    pub async fn execute_stream(
        &self,
        query: &str
    ) -> Result<impl Stream<Item = Result<Row, GraphDbError>>, GraphDbError>;
    
    /// 异步事务
    pub async fn with_transaction<F, T>(&self, f: F) -> Result<T, GraphDbError>
    where
        F: AsyncFnMut(&AsyncTransaction) -> Result<T, GraphDbError>;
}
```

---

## API 优先级

| 优先级 | API 类别 | 说明 |
|-------|---------|------|
| **P0** | 基础连接 API | `GraphDatabase::open()`, `close()`, `session()` |
| **P0** | 查询执行 API | `Session::execute()`, `QueryResult` |
| **P0** | 事务 API | `begin_transaction()`, `commit()`, `rollback()` |
| **P1** | 参数化查询 | `execute_with_params()` |
| **P1** | 预编译语句 | `prepare()`, `Statement` |
| **P1** | C FFI API | 跨语言绑定基础 |
| **P2** | 批量操作 | `BatchInserter` |
| **P2** | 异步 API | `AsyncGraphDatabase` |
| **P3** | 高级特性 | 保存点、流式结果、监控指标 |

---

## 与现有代码的对应关系

| 新 API | 现有实现 | 说明 |
|-------|---------|------|
| `GraphDatabase` | `GraphService` | 简化包装，去除网络层 |
| `Session` | `ClientSession` | 简化，去除认证相关 |
| `Transaction` | `TransactionManager` | 直接调用现有事务管理器 |
| `execute()` | `QueryEngine` | 复用现有查询引擎 |
| `QueryResult` | `ExecutionResult` | 适配现有结果格式 |
| `Value` | `core::value::Value` | 直接使用现有类型 |

---

## 多语言支持路径

根据 `rust与C作为嵌入数据库的区别.txt` 的分析，建议：

1. **Rust 原生 SDK**: 直接使用上述 API
2. **C FFI 层**: 提供 C 兼容接口，让其他语言通过 FFI 调用
3. **专用绑定**（可选）: 使用 PyO3、Napi-rs 等为 Python/Node.js 提供地道 API

---

## 参考文档

- [SQLite C API 参考](https://sqlite.org/capi3ref.html)
- [Neo4j Python Driver 文档](https://neo4j.com/docs/python-manual/current/)
- [SDK功能需求分析](./SDK功能需求分析.md)
- [Rust与C作为嵌入数据库的区别](./rust与C作为嵌入数据库的区别.txt)
