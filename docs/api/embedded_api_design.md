# GraphDB 嵌入式数据库 API 设计方案

## 概述

本文档定义 GraphDB 作为嵌入式数据库的完整 API 设计方案，参考 SQLite 的简洁性和 Neo4j 的类型安全性，为 Rust 应用提供原生图数据库支持。

> **设计简化说明**: 本版本已移除过度设计的功能（C FFI API、异步 API、保存点、流式查询等），专注于嵌入式场景的核心需求。

---

## 设计原则

1. **简洁性**: 像 SQLite 一样，用最少的 API 完成最常见的操作
2. **类型安全**: 利用 Rust 的类型系统，在编译期捕获错误
3. **零成本抽象**: 高级 API 不带来运行时开销
4. **可扩展性**: 支持从简单脚本到复杂应用的多种使用场景
5. **务实性**: 优先实现核心功能，避免过度设计

---

## API 架构层次

```
┌─────────────────────────────────────────────────────────────┐
│                    公共 API 层 (Public API)                   │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │  同步 API     │  │  高级特性     │  │   批量操作        │  │
│  │  (embedded)  │  │  (预编译语句) │  │  (数据导入)      │  │
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
    pub cache_size: usize,               // 缓存大小（MB）
    pub default_timeout: Duration,       // 默认超时
}

impl GraphDatabase {
    /// 打开数据库（文件模式）
    /// 对应: sqlite3_open()
    pub fn open(path: impl AsRef<Path>) -> Result<Self, GraphDbError>;
    
    /// 创建内存数据库
    /// 对应: sqlite3_open(":memory:")
    pub fn open_in_memory() -> Result<Self, GraphDbError>;
    
    /// 使用配置打开数据库
    pub fn open_with_config(config: DatabaseConfig) -> Result<Self, GraphDbError>;
    
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
    
    /// 预编译查询
    pub fn prepare(&self, query: &str) -> Result<PreparedStatement, GraphDbError>;
}
```

> **设计说明**: 移除 `max_connections` 配置，嵌入式数据库是单进程访问，不需要连接池管理。

---

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
    
    /// 创建批量插入器
    pub fn batch_inserter(&self, batch_size: usize) -> BatchInserter;
}
```

---

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
}

/// 托管事务（自动重试）- 简化版本
impl Session {
    /// 在事务中执行操作（自动提交/回滚）
    pub fn with_transaction<F, T>(&self, f: F) -> Result<T, GraphDbError>
    where
        F: FnOnce(&Transaction) -> Result<T, GraphDbError>;
}
```

> **设计说明**: 
> - 移除保存点（Savepoint）功能，嵌入式场景事务通常简单，嵌套事务需求较少
> - 移除 `with_write_transaction` / `with_read_transaction` 的区分，简化为统一的 `with_transaction`

---

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

> **设计说明**: 移除 `is_cache_hit` 字段，缓存命中统计属于监控指标，非核心功能。

---

### 5. 预编译语句（高性能）

```rust
/// 预编译语句
/// 对应: sqlite3_stmt
pub struct PreparedStatement {
    query_plan: Arc<ExecutionPlan>,
    parameter_types: HashMap<String, DataType>,
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

---

### 6. 批量操作 API

```rust
/// 批量插入器
pub struct BatchInserter<'sess> {
    session: &'sess Session,
    batch_size: usize,
    vertex_buffer: Vec<Vertex>,
    edge_buffer: Vec<Edge>,
}

impl<'sess> BatchInserter<'sess> {
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

/// 批量错误
pub struct BatchError {
    pub index: usize,
    pub item_type: BatchItemType,
    pub error: GraphDbError,
}

pub enum BatchItemType {
    Vertex,
    Edge,
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

## API 优先级

| 优先级 | API 类别 | 说明 |
|-------|---------|------|
| **P0** | 基础连接 API | `GraphDatabase::open()`, `close()`, `session()` |
| **P0** | 查询执行 API | `Session::execute()`, `QueryResult` |
| **P0** | 事务 API | `begin_transaction()`, `commit()`, `rollback()` |
| **P1** | 参数化查询 | `execute_with_params()` |
| **P1** | 预编译语句 | `prepare()`, `PreparedStatement` |
| **P1** | 批量操作 | `BatchInserter` |
| **P2** | 托管事务 | `with_transaction()` |
| **P3** | ~~C FFI API~~ | ~~跨语言绑定基础~~ (移除：嵌入式场景优先 Rust 原生) |
| **P3** | ~~异步 API~~ | ~~AsyncGraphDatabase~~ (移除：嵌入式场景同步 API 足够) |
| **P3** | ~~保存点~~ | ~~事务内子事务~~ (移除：嵌入式场景需求较少) |
| **P3** | ~~流式查询~~ | ~~execute_stream~~ (移除：结果集通常在内存中处理) |

---

## 移除的功能说明

### 1. C FFI API（已移除）
**移除原因**:
- 项目当前是单机嵌入式数据库，目标用户主要是 Rust 开发者
- 如需支持其他语言，建议使用 PyO3/Napi-rs 等专用绑定库直接包装 Rust API
- C FFI 层开发成本高，且需要维护 unsafe 代码边界

**替代方案**:
- Python: 使用 PyO3 直接包装 Rust API
- Node.js: 使用 Napi-rs 直接包装 Rust API
- 其他语言: 通过专用绑定生成器

### 2. 异步 API（已移除）
**移除原因**:
- 嵌入式数据库通常在同一线程/进程内使用
- SQLite 没有专门的异步 API，同步 API 已满足需求
- 存储引擎层面的 I/O 异步已足够

**替代方案**:
- 如需异步，可在应用层使用 `tokio::task::spawn_blocking` 包装同步调用

### 3. 保存点 Savepoint（已移除）
**移除原因**:
- 嵌入式场景事务通常简单，嵌套事务需求较少
- 增加 API 复杂度

**替代方案**:
- 如需嵌套事务，可在应用层实现逻辑隔离

### 4. 流式查询 execute_stream（已移除）
**移除原因**:
- 嵌入式数据库结果集通常在内存中处理
- 流式查询增加实现复杂度
- 大数据量查询可通过 LIMIT/OFFSET 分页

### 5. 连接池/多连接管理（已移除）
**移除原因**:
- 嵌入式数据库是单进程访问，不需要连接池
- 移除 `max_connections` 配置项

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

1. **Rust 原生 SDK**: 直接使用上述 API（当前重点）
2. **专用绑定**（未来可选）: 使用 PyO3、Napi-rs 等为 Python/Node.js 提供地道 API

> **注意**: 不推荐维护 C FFI 层，直接使用专用绑定库开发效率更高，用户体验更好。

---

## 参考文档

- [SQLite C API 参考](https://sqlite.org/capi3ref.html)
- [Neo4j Python Driver 文档](https://neo4j.com/docs/python-manual/current/)
- [SDK功能需求分析](./SDK功能需求分析.md)
- [Rust与C作为嵌入数据库的区别](./rust与C作为嵌入数据库的区别.txt)
- [嵌入式 API 扩展方案](./embedded_api_expansion_plan.md)
