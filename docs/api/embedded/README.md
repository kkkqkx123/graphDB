# GraphDB 嵌入式 API 文档

## 概述

GraphDB 嵌入式 API 提供类似 SQLite 的单机使用方式，允许开发者直接在应用程序中嵌入图数据库功能，无需独立的服务器进程。

## 架构设计

```
┌─────────────────────────────────────────────────────────────┐
│                    嵌入式 API 层                             │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │  Rust API    │  │   C API      │  │  其他语言绑定     │  │
│  │  (embedded)  │  │  (c_api)     │  │  (未来支持)      │  │
│  └──────────────┘  └──────────────┘  └──────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│                    核心 API 层 (core)                        │
├─────────────────────────────────────────────────────────────┤
│                    存储引擎层 (storage)                      │
└─────────────────────────────────────────────────────────────┘
```

## 模块结构

### Rust API (src/api/embedded/)

| 模块 | 文件 | 功能描述 |
|------|------|----------|
| mod | mod.rs | 模块入口，重新导出所有公共类型 |
| database | database.rs | GraphDatabase 结构体，数据库主入口 |
| session | session.rs | Session 结构体，查询执行上下文 |
| transaction | transaction.rs | Transaction 结构体，事务管理 |
| config | config.rs | DatabaseConfig 等配置类型 |
| result | result.rs | QueryResult、Row 等结果类型 |
| batch | batch.rs | BatchInserter，批量数据导入 |
| statement | statement/ | PreparedStatement，预编译语句 |

### C API (src/api/embedded/c_api/)

| 模块 | 文件 | 功能描述 |
|------|------|----------|
| types | types.rs | C 语言类型定义 |
| error | error.rs | 错误码和错误处理 |
| database | database.rs | 数据库打开/关闭 |
| session | session.rs | 会话管理 |
| query | query.rs | 查询执行 |
| statement | statement.rs | 预编译语句 |
| transaction | transaction.rs | 事务管理 |
| batch | batch.rs | 批量操作 |
| result | result.rs | 结果处理 |

## 核心概念

### 1. 数据库 (GraphDatabase)

数据库是嵌入式 API 的主要入口点，对应 SQLite 的 sqlite3 结构体。

**主要功能：**
- 打开/创建数据库（文件模式或内存模式）
- 创建会话
- 执行简单查询（便捷方法）
- 管理图空间

### 2. 会话 (Session)

会话是查询执行的基本单元，维护当前图空间、事务状态等上下文信息。

**主要功能：**
- 切换图空间 (use_space)
- 执行查询 (execute)
- 事务管理 (begin_transaction)
- 批量操作 (batch_inserter)
- 预编译语句 (prepare)

### 3. 事务 (Transaction)

提供完整的事务管理功能，包括保存点支持。

**主要功能：**
- 开始事务
- 提交事务 (commit)
- 回滚事务 (rollback)
- 保存点管理 (create_savepoint, rollback_to_savepoint)

### 4. 查询结果 (QueryResult)

封装查询结果，提供方便的访问方法。

**主要功能：**
- 获取列名列表
- 按行/列访问数据
- 类型化获取方法 (get_string, get_int, get_vertex 等)
- JSON 序列化

### 5. 预编译语句 (PreparedStatement)

预编译的查询语句，可以重复执行并绑定不同的参数。

**主要功能：**
- 参数绑定 (bind)
- 查询执行 (execute)
- 批量执行 (execute_batch)
- 执行统计 (stats)

### 6. 批量插入器 (BatchInserter)

高效的大批量数据导入工具。

**主要功能：**
- 批量添加顶点/边
- 自动刷新缓冲区
- 错误收集

## 使用示例

### Rust API 示例

```rust
use graphdb::api::embedded::{GraphDatabase, DatabaseConfig};

// 打开数据库
let db = GraphDatabase::open("my_database")?;

// 创建会话
let mut session = db.session()?;

// 切换图空间
session.use_space("test_space")?;

// 执行查询
let result = session.execute("MATCH (n) RETURN n")?;

// 使用事务
let txn = session.begin_transaction()?;
txn.execute("CREATE TAG user(name string)")?;
txn.commit()?;
```

### C API 示例

```c
#include <graphdb.h>

graphdb_t* db = NULL;
graphdb_session_t* session = NULL;
graphdb_result_t* result = NULL;

// 打开数据库
int rc = graphdb_open("my_database.db", &db);
if (rc != GRAPHDB_OK) {
    // 处理错误
}

// 创建会话
rc = graphdb_session_create(db, &session);

// 切换图空间
rc = graphdb_session_use_space(session, "test_space");

// 执行查询
rc = graphdb_execute(session, "MATCH (n) RETURN n", &result);

// 清理
graphdb_result_free(result);
graphdb_session_close(session);
graphdb_close(db);
```

## 配置选项

### DatabaseConfig

| 选项 | 类型 | 默认值 | 描述 |
|------|------|--------|------|
| path | Option<PathBuf> | None | 数据库路径，None 表示内存模式 |
| cache_size_mb | usize | 64 | 缓存大小（MB） |
| default_timeout | Duration | 30s | 默认超时 |
| enable_wal | bool | true | 是否启用 WAL |
| sync_mode | SyncMode | Normal | 同步模式 |

### SyncMode

- **Full**: 完全同步，每次写入都同步到磁盘（最安全，最慢）
- **Normal**: 正常同步，定期同步（平衡）
- **Off**: 异步模式，由操作系统决定何时同步（最快，有风险）

### TransactionConfig

| 选项 | 类型 | 默认值 | 描述 |
|------|------|--------|------|
| timeout | Option<Duration> | None | 事务超时时间 |
| read_only | bool | false | 是否只读 |
| durability | DurabilityLevel | Immediate | 持久性级别 |

## 错误处理

### Rust API 错误类型

所有操作返回 `CoreResult<T>`，错误类型为 `CoreError`：

- `StorageError`: 存储层错误
- `QueryExecutionFailed`: 查询执行失败
- `TransactionFailed`: 事务失败
- `SchemaOperationFailed`: 模式操作失败
- `Internal`: 内部错误
- `NotFound`: 未找到
- `InvalidParameter`: 无效参数

### C API 错误码

| 错误码 | 值 | 描述 |
|--------|-----|------|
| GRAPHDB_OK | 0 | 成功 |
| GRAPHDB_ERROR | 1 | 一般错误 |
| GRAPHDB_INTERNAL | 2 | 内部错误 |
| GRAPHDB_PERM | 3 | 权限被拒绝 |
| GRAPHDB_ABORT | 4 | 操作被中止 |
| GRAPHDB_BUSY | 5 | 数据库忙 |
| GRAPHDB_LOCKED | 6 | 数据库被锁定 |
| GRAPHDB_NOMEM | 7 | 内存不足 |
| GRAPHDB_READONLY | 8 | 只读 |
| GRAPHDB_IOERR | 10 | IO 错误 |
| GRAPHDB_CORRUPT | 11 | 数据损坏 |
| GRAPHDB_NOTFOUND | 12 | 未找到 |
| GRAPHDB_SCHEMA | 16 | 模式错误 |
| GRAPHDB_MISUSE | 20 | 误用 |

## 线程安全

- `GraphDatabase`: 实现 `Send + Sync`，可安全跨线程共享
- `Session`: 实现 `Send + Sync`，但建议每个线程使用独立会话
- `Transaction`: 绑定到创建它的 Session，不能跨线程使用

## 性能优化建议

1. **使用预编译语句**: 对于重复执行的查询，使用 PreparedStatement
2. **批量插入**: 大量数据导入时使用 BatchInserter
3. **事务批处理**: 将多个操作放在同一个事务中
4. **合理配置缓存**: 根据数据量调整 cache_size_mb
5. **选择合适的同步模式**: 根据数据安全需求选择 SyncMode

## 与 SQLite 的对比

| 特性 | GraphDB | SQLite |
|------|---------|--------|
| 数据模型 | 图数据（顶点/边） | 关系型（表/行） |
| 查询语言 | nGQL (类Cypher) | SQL |
| 事务支持 | 是（含保存点） | 是（含保存点） |
| 预编译语句 | 是 | 是 |
| 批量操作 | 是 | 是 |
| 内存模式 | 是 | 是 |
| 文件模式 | 是 | 是 |
| C API | 是 | 是 |
