# GraphDB C API 实现分析

## 目录

1. [概述](#概述)
2. [现有 Embedded API 结构分析](#现有-embedded-api-结构分析)
3. [SQLite C API 设计模式参考](#sqlite-c-api-设计模式参考)
4. [GraphDB C API 架构设计](#graphdb-c-api-架构设计)
5. [实现方案](#实现方案)
6. [代码示例](#代码示例)
7. [技术细节](#技术细节)
8. [注意事项](#注意事项)

---

## 概述

本文档分析了如何为 GraphDB 提供标准 C API，参考 SQLite 的 C API 设计模式，为嵌入式使用场景提供类似 SQLite 的接口。

### 目标

- 提供符合 C 语言习惯的 API 接口
- 保持与现有 Rust embedded API 的功能一致性
- 支持跨语言调用（C、C++、Python、Go 等）
- 提供线程安全和高性能的接口
- 简化错误处理和资源管理

---

## 现有 Embedded API 结构分析

### 核心模块

GraphDB 的 embedded API 位于 `src/api/embedded` 目录，包含以下核心模块：

#### 1. **database.rs** - 数据库主入口

```rust
pub struct GraphDatabase<S: StorageClient + Clone + 'static> {
    inner: Arc<GraphDatabaseInner<S>>,
    config: DatabaseConfig,
}
```

**主要功能：**
- `open(path)` - 打开文件数据库
- `open_in_memory()` - 打开内存数据库
- `open_with_config(config)` - 使用配置打开数据库
- `session()` - 创建会话
- `execute(query)` - 执行简单查询

**对应 SQLite：** `sqlite3*` 数据库句柄

#### 2. **session.rs** - 会话管理

```rust
pub struct Session<S: StorageClient + Clone + 'static> {
    db: Arc<GraphDatabaseInner<S>>,
    space_id: Option<u64>,
    space_name: Option<String>,
    auto_commit: bool,
}
```

**主要功能：**
- `use_space(name)` - 切换图空间
- `execute(query)` - 执行查询
- `execute_with_params(query, params)` - 执行参数化查询
- `begin_transaction()` - 开始事务
- `prepare(query)` - 预编译语句

**对应 SQLite：** 无直接对应，功能集成在数据库句柄中

#### 3. **statement.rs** - 预编译语句

```rust
pub struct PreparedStatement<S: StorageClient + 'static> {
    query_api: Arc<Mutex<QueryApi<S>>>,
    query: String,
    parameter_types: HashMap<String, DataType>,
    bound_params: HashMap<String, Value>,
    space_id: Option<u64>,
    config: StatementConfig,
    stats: ExecutionStats,
}
```

**主要功能：**
- `bind(name, value)` - 绑定参数
- `execute()` - 执行语句
- `reset()` - 重置语句
- `clear_bindings()` - 清除绑定
- `stats()` - 获取执行统计

**对应 SQLite：** `sqlite3_stmt*` 预编译语句句柄

#### 4. **transaction.rs** - 事务管理

```rust
pub struct Transaction<'sess, S: StorageClient + Clone + 'static> {
    session: &'sess Session<S>,
    txn_handle: TransactionHandle,
    committed: bool,
    rolled_back: bool,
}
```

**主要功能：**
- `execute(query)` - 在事务中执行查询
- `commit()` - 提交事务
- `rollback()` - 回滚事务
- `create_savepoint()` - 创建保存点
- `rollback_to_savepoint()` - 回滚到保存点

**对应 SQLite：** 事务通过 `BEGIN`、`COMMIT`、`ROLLBACK` 语句实现

#### 5. **result.rs** - 查询结果

```rust
pub struct QueryResult {
    columns: Vec<String>,
    rows: Vec<Row>,
    metadata: ResultMetadata,
}

pub struct Row {
    values: HashMap<String, Value>,
    column_index: HashMap<String, usize>,
}
```

**主要功能：**
- `columns()` - 获取列名
- `len()` - 获取行数
- `get(index)` - 获取行
- `iter()` - 行迭代器
- `to_json()` - 转换为 JSON

**对应 SQLite：** `sqlite3_step()` + `sqlite3_column_*()` 系列

---

## SQLite C API 设计模式参考

### 核心设计原则

1. **句柄模式**：所有资源都通过不透明指针（句柄）访问
2. **错误码机制**：每个函数返回错误码，错误信息通过专用函数获取
3. **生命周期管理**：显式的打开/关闭函数
4. **预编译语句**：支持语句预编译和参数绑定
5. **结果迭代**：通过 `step()` 函数逐行获取结果

### 核心数据结构

```c
// 数据库句柄
typedef struct sqlite3 sqlite3;

// 预编译语句句柄
typedef struct sqlite3_stmt sqlite3_stmt;

// 备份句柄
typedef struct sqlite3_backup sqlite3_backup;
```

### 核心函数

#### 数据库操作

```c
// 打开数据库
int sqlite3_open(const char *filename, sqlite3 **ppDb);

// 关闭数据库
int sqlite3_close(sqlite3 *db);

// 执行 SQL 语句
int sqlite3_exec(
    sqlite3 *db,                   /* 数据库句柄 */
    const char *sql,               /* SQL 语句 */
    int (*callback)(void*,int,char**,char**),  /* 回调函数 */
    void *arg,                    /* 回调参数 */
    char **errmsg                  /* 错误信息 */
);
```

#### 预编译语句

```c
// 准备语句
int sqlite3_prepare_v2(
    sqlite3 *db,            /* 数据库句柄 */
    const char *zSql,       /* SQL 语句 */
    int nByte,              /* SQL 长度（-1 表示自动计算） */
    sqlite3_stmt **ppStmt,  /* 输出：语句句柄 */
    const char **pzTail     /* 输出：未使用的 SQL 部分 */
);

// 绑定参数
int sqlite3_bind_int(sqlite3_stmt*, int, int);
int sqlite3_bind_int64(sqlite3_stmt*, int, sqlite3_int64);
int sqlite3_bind_double(sqlite3_stmt*, int, double);
int sqlite3_bind_text(sqlite3_stmt*, int, const char*, int, void(*)(void*));
int sqlite3_bind_blob(sqlite3_stmt*, int, const void*, int, void(*)(void*));
int sqlite3_bind_null(sqlite3_stmt*, int);

// 执行语句
int sqlite3_step(sqlite3_stmt*);

// 重置语句
int sqlite3_reset(sqlite3_stmt*);

// 清除绑定
int sqlite3_clear_bindings(sqlite3_stmt*);

// 释放语句
int sqlite3_finalize(sqlite3_stmt*);
```

#### 结果获取

```c
// 获取列数
int sqlite3_column_count(sqlite3_stmt *pStmt);

// 获取列名
const char *sqlite3_column_name(sqlite3_stmt*, int N);

// 获取列值
int sqlite3_column_int(sqlite3_stmt*, int iCol);
sqlite3_int64 sqlite3_column_int64(sqlite3_stmt*, int iCol);
double sqlite3_column_double(sqlite3_stmt*, int iCol);
const unsigned char *sqlite3_column_text(sqlite3_stmt*, int iCol);
const void *sqlite3_column_blob(sqlite3_stmt*, int iCol);
int sqlite3_column_bytes(sqlite3_stmt*, int iCol);
int sqlite3_column_type(sqlite3_stmt*, int iCol);
```

#### 错误处理

```c
// 获取错误码
int sqlite3_errcode(sqlite3 *db);

// 获取扩展错误码
int sqlite3_extended_errcode(sqlite3 *db);

// 获取错误信息
const char *sqlite3_errmsg(sqlite3 *db);

// 获取错误位置
int sqlite3_error_offset(sqlite3 *db);
```

#### 事务控制

```c
// 设置自动提交模式
int sqlite3_get_autocommit(sqlite3 *db);

// 保存点
int sqlite3_savepoint(sqlite3 *db, const char *zName);
int sqlite3_release(sqlite3 *db, const char *zName);
int sqlite3_rollback_to(sqlite3 *db, const char *zName);
```

---

## GraphDB C API 架构设计

### 设计原则

1. **与 SQLite API 风格一致**：采用相似的命名和调用模式
2. **保持 Rust API 功能**：不牺牲现有功能
3. **类型安全**：提供强类型的 C 接口
4. **线程安全**：支持多线程并发访问
5. **资源管理**：明确的资源生命周期

### 核心数据结构

```c
// 数据库句柄（不透明指针）
typedef struct graphdb_t graphdb_t;

// 会话句柄
typedef struct graphdb_session_t graphdb_session_t;

// 预编译语句句柄
typedef struct graphdb_stmt_t graphdb_stmt_t;

// 事务句柄
typedef struct graphdb_txn_t graphdb_txn_t;

// 结果集句柄
typedef struct graphdb_result_t graphdb_result_t;

// 值类型
typedef enum {
    GRAPHDB_NULL = 0,
    GRAPHDB_BOOL = 1,
    GRAPHDB_INT = 2,
    GRAPHDB_FLOAT = 3,
    GRAPHDB_STRING = 4,
    GRAPHDB_LIST = 5,
    GRAPHDB_MAP = 6,
    GRAPHDB_VERTEX = 7,
    GRAPHDB_EDGE = 8,
    GRAPHDB_PATH = 9,
} graphdb_value_type_t;

// 值结构
typedef struct {
    graphdb_value_type_t type;
    union {
        bool boolean;
        int64_t integer;
        double floating;
        struct {
            const char *data;
            size_t len;
        } string;
        // ... 其他类型
    };
} graphdb_value_t;

// 错误码
typedef enum {
    GRAPHDB_OK = 0,
    GRAPHDB_ERROR = 1,
    GRAPHDB_INTERNAL = 2,
    GRAPHDB_PERM = 3,
    GRAPHDB_ABORT = 4,
    GRAPHDB_BUSY = 5,
    GRAPHDB_LOCKED = 6,
    GRAPHDB_NOMEM = 7,
    GRAPHDB_READONLY = 8,
    GRAPHDB_INTERRUPT = 9,
    GRAPHDB_IOERR = 10,
    GRAPHDB_CORRUPT = 11,
    GRAPHDB_NOTFOUND = 12,
    GRAPHDB_FULL = 13,
    GRAPHDB_CANTOPEN = 14,
    GRAPHDB_PROTOCOL = 15,
    GRAPHDB_SCHEMA = 16,
    GRAPHDB_TOOBIG = 17,
    GRAPHDB_CONSTRAINT = 18,
    GRAPHDB_MISMATCH = 19,
    GRAPHDB_MISUSE = 20,
    GRAPHDB_RANGE = 21,
} graphdb_error_code_t;
```

### API 函数分类

#### 1. 数据库管理

```c
// 打开数据库
int graphdb_open(const char *path, graphdb_t **db);

// 打开内存数据库
int graphdb_open_memory(graphdb_t **db);

// 使用配置打开数据库
int graphdb_open_config(const char *path, const graphdb_config_t *config, graphdb_t **db);

// 关闭数据库
int graphdb_close(graphdb_t *db);

// 获取错误码
int graphdb_errcode(graphdb_t *db);

// 获取错误信息
const char *graphdb_errmsg(graphdb_t *db);
```

#### 2. 会话管理

```c
// 创建会话
int graphdb_session_create(graphdb_t *db, graphdb_session_t **session);

// 关闭会话
int graphdb_session_close(graphdb_session_t *session);

// 切换图空间
int graphdb_session_use_space(graphdb_session_t *session, const char *space_name);

// 获取当前图空间
const char *graphdb_session_current_space(graphdb_session_t *session);

// 设置自动提交模式
int graphdb_session_set_autocommit(graphdb_session_t *session, bool autocommit);

// 获取自动提交模式
bool graphdb_session_get_autocommit(graphdb_session_t *session);
```

#### 3. 查询执行

```c
// 执行查询（简单方式）
int graphdb_execute(graphdb_session_t *session, const char *query, graphdb_result_t **result);

// 执行参数化查询
int graphdb_execute_params(
    graphdb_session_t *session,
    const char *query,
    const graphdb_value_t *params,
    size_t param_count,
    graphdb_result_t **result
);
```

#### 4. 预编译语句

```c
// 准备语句
int graphdb_prepare(
    graphdb_session_t *session,
    const char *query,
    graphdb_stmt_t **stmt
);

// 绑定参数（按索引）
int graphdb_bind_null(graphdb_stmt_t *stmt, int index);
int graphdb_bind_bool(graphdb_stmt_t *stmt, int index, bool value);
int graphdb_bind_int(graphdb_stmt_t *stmt, int index, int64_t value);
int graphdb_bind_float(graphdb_stmt_t *stmt, int index, double value);
int graphdb_bind_string(graphdb_stmt_t *stmt, int index, const char *value, int len);
int graphdb_bind_blob(graphdb_stmt_t *stmt, int index, const void *data, int len);

// 绑定参数（按名称）
int graphdb_bind_null_by_name(graphdb_stmt_t *stmt, const char *name);
int graphdb_bind_int_by_name(graphdb_stmt_t *stmt, const char *name, int64_t value);

// 执行语句
int graphdb_step(graphdb_stmt_t *stmt);

// 重置语句
int graphdb_reset(graphdb_stmt_t *stmt);

// 清除绑定
int graphdb_clear_bindings(graphdb_stmt_t *stmt);

// 释放语句
int graphdb_finalize(graphdb_stmt_t *stmt);

// 获取参数索引
int graphdb_bind_parameter_index(graphdb_stmt_t *stmt, const char *name);

// 获取参数名称
const char *graphdb_bind_parameter_name(graphdb_stmt_t *stmt, int index);
```

#### 5. 结果处理

```c
// 获取列数
int graphdb_column_count(graphdb_result_t *result);

// 获取列名
const char *graphdb_column_name(graphdb_result_t *result, int index);

// 获取行数
int graphdb_row_count(graphdb_result_t *result);

// 获取值（按列索引）
int graphdb_column_type(graphdb_result_t *result, int row, int col);
int graphdb_column_bool(graphdb_result_t *result, int row, int col, bool *value);
int graphdb_column_int(graphdb_result_t *result, int row, int col, int64_t *value);
int graphdb_column_float(graphdb_result_t *result, int row, int col, double *value);
const char *graphdb_column_string(graphdb_result_t *result, int row, int col, int *len);
const void *graphdb_column_blob(graphdb_result_t *result, int row, int col, int *len);

// 获取值（按列名）
int graphdb_get_bool(graphdb_result_t *result, int row, const char *col, bool *value);
int graphdb_get_int(graphdb_result_t *result, int row, const char *col, int64_t *value);
int graphdb_get_float(graphdb_result_t *result, int row, const char *col, double *value);
const char *graphdb_get_string(graphdb_result_t *result, int row, const char *col, int *len);

// 释放结果
int graphdb_result_free(graphdb_result_t *result);
```

#### 6. 事务管理

```c
// 开始事务
int graphdb_txn_begin(graphdb_session_t *session, graphdb_txn_t **txn);

// 开始只读事务
int graphdb_txn_begin_readonly(graphdb_session_t *session, graphdb_txn_t **txn);

// 在事务中执行查询
int graphdb_txn_execute(graphdb_txn_t *txn, const char *query, graphdb_result_t **result);

// 提交事务
int graphdb_txn_commit(graphdb_txn_t *txn);

// 回滚事务
int graphdb_txn_rollback(graphdb_txn_t *txn);

// 创建保存点
int graphdb_txn_savepoint(graphdb_txn_t *txn, const char *name);

// 释放保存点
int graphdb_txn_release_savepoint(graphdb_txn_t *txn, const char *name);

// 回滚到保存点
int graphdb_txn_rollback_to_savepoint(graphdb_txn_t *txn, const char *name);
```

#### 7. 批量操作

```c
// 创建批量插入器
int graphdb_batch_inserter_create(
    graphdb_session_t *session,
    const char *tag_name,
    graphdb_batch_inserter_t **inserter
);

// 添加顶点
int graphdb_batch_add_vertex(
    graphdb_batch_inserter_t *inserter,
    int64_t vid,
    const graphdb_value_t *properties,
    size_t prop_count
);

// 添加边
int graphdb_batch_add_edge(
    graphdb_batch_inserter_t *inserter,
    int64_t src_vid,
    int64_t dst_vid,
    int64_t rank,
    const char *edge_type,
    const graphdb_value_t *properties,
    size_t prop_count
);

// 执行批量插入
int graphdb_batch_flush(graphdb_batch_inserter_t *inserter);

// 释放批量插入器
int graphdb_batch_inserter_free(graphdb_batch_inserter_t *inserter);
```

#### 8. 配置和元数据

```c
// 数据库配置
typedef struct {
    bool read_only;
    bool create_if_missing;
    int cache_size_mb;
    int max_open_files;
    bool enable_compression;
    // ... 其他配置项
} graphdb_config_t;

// 获取数据库版本
const char *graphdb_libversion(void);

// 获取数据库源 ID
const char *graphdb_sourceid(void);

// 获取数据库统计信息
int graphdb_db_status(
    graphdb_t *db,
    int op,
    int *current,
    int *highwater
);
```

---

## 实现方案

### 技术栈

1. **Rust FFI**：使用 `std::ffi` 模块提供 C 兼容接口
2. **cbindgen**：自动生成 C 头文件
3. **libc**：与 C 标准库交互
4. **Arc<Mutex<T>>**：线程安全的内部状态管理

### 项目结构

```
src/
├── api/
│   ├── embedded/
│   │   ├── mod.rs              # Rust embedded API
│   │   ├── database.rs
│   │   ├── session.rs
│   │   ├── statement.rs
│   │   ├── transaction.rs
│   │   └── result.rs
│   └── c_api/                  # 新增 C API 模块
│       ├── mod.rs             # C API 入口
│       ├── types.rs           # C 类型定义
│       ├── database.rs        # 数据库操作
│       ├── session.rs         # 会话管理
│       ├── statement.rs       # 预编译语句
│       ├── transaction.rs     # 事务管理
│       ├── result.rs          # 结果处理
│       └── error.rs           # 错误处理
include/
└── graphdb.h                  # 自动生成的 C 头文件
```

### 实现步骤

#### 步骤 1：创建 C API 模块

```rust
// src/api/c_api/mod.rs
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

pub mod types;
pub mod database;
pub mod session;
pub mod statement;
pub mod transaction;
pub mod result;
pub mod error;

use std::ffi::{CStr, CString, c_char, c_int, c_void};
use std::sync::Arc;

// 重新导出核心类型
pub use types::*;
pub use database::*;
pub use session::*;
pub use statement::*;
pub use transaction::*;
pub use result::*;
pub use error::*;
```

#### 步骤 2：实现类型转换

```rust
// src/api/c_api/types.rs
use crate::core::Value;
use std::ffi::{c_char, c_void};
use std::mem;

// C 值类型
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum graphdb_value_type_t {
    GRAPHDB_NULL = 0,
    GRAPHDB_BOOL = 1,
    GRAPHDB_INT = 2,
    GRAPHDB_FLOAT = 3,
    GRAPHDB_STRING = 4,
    GRAPHDB_LIST = 5,
    GRAPHDB_MAP = 6,
    GRAPHDB_VERTEX = 7,
    GRAPHDB_EDGE = 8,
    GRAPHDB_PATH = 9,
}

// C 值结构
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct graphdb_value_t {
    pub type_: graphdb_value_type_t,
    pub data: graphdb_value_data_t,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub union graphdb_value_data_t {
    pub boolean: bool,
    pub integer: i64,
    pub floating: f64,
    pub string: graphdb_string_t,
    pub ptr: *mut c_void,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct graphdb_string_t {
    pub data: *const c_char,
    pub len: usize,
}

impl Value {
    // 从 Rust Value 转换为 C graphdb_value_t
    pub fn to_c_value(&self) -> graphdb_value_t {
        match self {
            Value::Null => graphdb_value_t {
                type_: graphdb_value_type_t::GRAPHDB_NULL,
                data: graphdb_value_data_t { ptr: std::ptr::null_mut() },
            },
            Value::Bool(b) => graphdb_value_t {
                type_: graphdb_value_type_t::GRAPHDB_BOOL,
                data: graphdb_value_data_t { boolean: *b },
            },
            Value::Int(i) => graphdb_value_t {
                type_: graphdb_value_type_t::GRAPHDB_INT,
                data: graphdb_value_data_t { integer: *i },
            },
            Value::Float(f) => graphdb_value_t {
                type_: graphdb_value_type_t::GRAPHDB_FLOAT,
                data: graphdb_value_data_t { floating: *f },
            },
            Value::String(s) => {
                let c_string = CString::new(s.as_str()).expect("Invalid UTF-8");
                let ptr = c_string.into_raw();
                graphdb_value_t {
                    type_: graphdb_value_type_t::GRAPHDB_STRING,
                    data: graphdb_value_data_t {
                        string: graphdb_string_t {
                            data: ptr,
                            len: s.len(),
                        },
                    },
                }
            }
            // ... 其他类型
            _ => graphdb_value_t {
                type_: graphdb_value_type_t::GRAPHDB_NULL,
                data: graphdb_value_data_t { ptr: std::ptr::null_mut() },
            },
        }
    }

    // 从 C graphdb_value_t 转换为 Rust Value
    pub unsafe fn from_c_value(c_value: &graphdb_value_t) -> Self {
        match c_value.type_ {
            graphdb_value_type_t::GRAPHDB_NULL => Value::Null,
            graphdb_value_type_t::GRAPHDB_BOOL => Value::Bool(c_value.data.boolean),
            graphdb_value_type_t::GRAPHDB_INT => Value::Int(c_value.data.integer),
            graphdb_value_type_t::GRAPHDB_FLOAT => Value::Float(c_value.data.floating),
            graphdb_value_type_t::GRAPHDB_STRING => {
                let slice = std::slice::from_raw_parts(
                    c_value.data.string.data as *const u8,
                    c_value.data.string.len,
                );
                let s = String::from_utf8_unchecked(slice.to_vec());
                Value::String(s)
            }
            // ... 其他类型
            _ => Value::Null,
        }
    }
}
```

#### 步骤 3：实现数据库操作

```rust
// src/api/c_api/database.rs
use crate::api::c_api::types::*;
use crate::api::c_api::error::*;
use crate::api::embedded::{GraphDatabase, DatabaseConfig};
use std::ffi::{CStr, CString, c_char};
use std::sync::Arc;
use std::ptr;

// 数据库句柄（不透明指针）
#[repr(C)]
pub struct graphdb_t {
    inner: Arc<GraphDatabase<crate::storage::RedbStorage>>,
    last_error: Option<CString>,
}

// 打开数据库
#[no_mangle]
pub extern "C" fn graphdb_open(path: *const c_char, db: *mut *mut graphdb_t) -> c_int {
    if path.is_null() || db.is_null() {
        return GRAPHDB_MISUSE as c_int;
    }

    let path_str = unsafe {
        match CStr::from_ptr(path).to_str() {
            Ok(s) => s,
            Err(_) => return GRAPHDB_MISUSE as c_int,
        }
    };

    match GraphDatabase::open(path_str) {
        Ok(graphdb) => {
            let c_db = Box::new(graphdb_t {
                inner: Arc::new(graphdb),
                last_error: None,
            });
            unsafe {
                *db = Box::into_raw(c_db);
            }
            GRAPHDB_OK as c_int
        }
        Err(e) => {
            unsafe {
                if !(*db).is_null() {
                    let c_db = &mut *(*db);
                    c_db.last_error = Some(CString::new(format!("{}", e)).unwrap());
                }
            }
            error_code_from_core_error(&e)
        }
    }
}

// 打开内存数据库
#[no_mangle]
pub extern "C" fn graphdb_open_memory(db: *mut *mut graphdb_t) -> c_int {
    if db.is_null() {
        return GRAPHDB_MISUSE as c_int;
    }

    match GraphDatabase::open_in_memory() {
        Ok(graphdb) => {
            let c_db = Box::new(graphdb_t {
                inner: Arc::new(graphdb),
                last_error: None,
            });
            unsafe {
                *db = Box::into_raw(c_db);
            }
            GRAPHDB_OK as c_int
        }
        Err(e) => error_code_from_core_error(&e),
    }
}

// 关闭数据库
#[no_mangle]
pub extern "C" fn graphdb_close(db: *mut graphdb_t) -> c_int {
    if db.is_null() {
        return GRAPHDB_MISUSE as c_int;
    }

    unsafe {
        let _ = Box::from_raw(db);
        *db = ptr::null_mut();
    }
    GRAPHDB_OK as c_int
}

// 获取错误码
#[no_mangle]
pub extern "C" fn graphdb_errcode(db: *mut graphdb_t) -> c_int {
    if db.is_null() {
        return GRAPHDB_MISUSE as c_int;
    }

    unsafe {
        if (*db).last_error.is_some() {
            GRAPHDB_ERROR as c_int
        } else {
            GRAPHDB_OK as c_int
        }
    }
}

// 获取错误信息
#[no_mangle]
pub extern "C" fn graphdb_errmsg(db: *mut graphdb_t) -> *const c_char {
    if db.is_null() {
        return ptr::null();
    }

    unsafe {
        match &(*db).last_error {
            Some(msg) => msg.as_ptr(),
            None => ptr::null(),
        }
    }
}
```

#### 步骤 4：实现会话管理

```rust
// src/api/c_api/session.rs
use crate::api::c_api::types::*;
use crate::api::c_api::database::graphdb_t;
use crate::api::embedded::Session;
use std::ffi::{CStr, CString, c_char};
use std::ptr;

// 会话句柄
#[repr(C)]
pub struct graphdb_session_t {
    inner: Session<crate::storage::RedbStorage>,
    last_error: Option<CString>,
}

// 创建会话
#[no_mangle]
pub extern "C" fn graphdb_session_create(
    db: *mut graphdb_t,
    session: *mut *mut graphdb_session_t,
) -> c_int {
    if db.is_null() || session.is_null() {
        return GRAPHDB_MISUSE as c_int;
    }

    unsafe {
        let graphdb = &(*db).inner;
        match graphdb.session() {
            Ok(sess) => {
                let c_session = Box::new(graphdb_session_t {
                    inner: sess,
                    last_error: None,
                });
                *session = Box::into_raw(c_session);
                GRAPHDB_OK as c_int
            }
            Err(e) => error_code_from_core_error(&e),
        }
    }
}

// 关闭会话
#[no_mangle]
pub extern "C" fn graphdb_session_close(session: *mut graphdb_session_t) -> c_int {
    if session.is_null() {
        return GRAPHDB_MISUSE as c_int;
    }

    unsafe {
        let _ = Box::from_raw(session);
        *session = ptr::null_mut();
    }
    GRAPHDB_OK as c_int
}

// 切换图空间
#[no_mangle]
pub extern "C" fn graphdb_session_use_space(
    session: *mut graphdb_session_t,
    space_name: *const c_char,
) -> c_int {
    if session.is_null() || space_name.is_null() {
        return GRAPHDB_MISUSE as c_int;
    }

    let name_str = unsafe {
        match CStr::from_ptr(space_name).to_str() {
            Ok(s) => s,
            Err(_) => return GRAPHDB_MISUSE as c_int,
        }
    };

    unsafe {
        match (*session).inner.use_space(name_str) {
            Ok(_) => GRAPHDB_OK as c_int,
            Err(e) => {
                (*session).last_error = Some(CString::new(format!("{}", e)).unwrap());
                error_code_from_core_error(&e)
            }
        }
    }
}

// 获取当前图空间
#[no_mangle]
pub extern "C" fn graphdb_session_current_space(
    session: *mut graphdb_session_t,
) -> *const c_char {
    if session.is_null() {
        return ptr::null();
    }

    unsafe {
        match (*session).inner.current_space() {
            Some(name) => {
                match CString::new(name) {
                    Ok(c_name) => c_name.into_raw(),
                    Err(_) => ptr::null(),
                }
            }
            None => ptr::null(),
        }
    }
}
```

#### 步骤 5：实现预编译语句

```rust
// src/api/c_api/statement.rs
use crate::api::c_api::types::*;
use crate::api::c_api::session::graphdb_session_t;
use crate::api::embedded::PreparedStatement;
use std::ffi::{CStr, CString, c_char};
use std::collections::HashMap;
use std::ptr;

// 预编译语句句柄
#[repr(C)]
pub struct graphdb_stmt_t {
    inner: PreparedStatement<crate::storage::RedbStorage>,
    bound_params: HashMap<String, Value>,
    last_error: Option<CString>,
}

// 准备语句
#[no_mangle]
pub extern "C" fn graphdb_prepare(
    session: *mut graphdb_session_t,
    query: *const c_char,
    stmt: *mut *mut graphdb_stmt_t,
) -> c_int {
    if session.is_null() || query.is_null() || stmt.is_null() {
        return GRAPHDB_MISUSE as c_int;
    }

    let query_str = unsafe {
        match CStr::from_ptr(query).to_str() {
            Ok(s) => s,
            Err(_) => return GRAPHDB_MISUSE as c_int,
        }
    };

    unsafe {
        let sess = &(*session).inner;
        match sess.prepare(query_str) {
            Ok(prepared) => {
                let c_stmt = Box::new(graphdb_stmt_t {
                    inner: prepared,
                    bound_params: HashMap::new(),
                    last_error: None,
                });
                *stmt = Box::into_raw(c_stmt);
                GRAPHDB_OK as c_int
            }
            Err(e) => {
                (*session).last_error = Some(CString::new(format!("{}", e)).unwrap());
                error_code_from_core_error(&e)
            }
        }
    }
}

// 绑定整数
#[no_mangle]
pub extern "C" fn graphdb_bind_int(
    stmt: *mut graphdb_stmt_t,
    index: c_int,
    value: i64,
) -> c_int {
    if stmt.is_null() {
        return GRAPHDB_MISUSE as c_int;
    }

    unsafe {
        let param_name = format!("${}", index);
        (*stmt).bound_params.insert(param_name, Value::Int(value));
        GRAPHDB_OK as c_int
    }
}

// 绑定字符串
#[no_mangle]
pub extern "C" fn graphdb_bind_string(
    stmt: *mut graphdb_stmt_t,
    index: c_int,
    value: *const c_char,
    len: c_int,
) -> c_int {
    if stmt.is_null() || value.is_null() {
        return GRAPHDB_MISUSE as c_int;
    }

    let value_str = unsafe {
        if len < 0 {
            match CStr::from_ptr(value).to_str() {
                Ok(s) => s.to_string(),
                Err(_) => return GRAPHDB_MISUSE as c_int,
            }
        } else {
            let slice = std::slice::from_raw_parts(value as *const u8, len as usize);
            match String::from_utf8(slice.to_vec()) {
                Ok(s) => s,
                Err(_) => return GRAPHDB_MISUSE as c_int,
            }
        }
    };

    unsafe {
        let param_name = format!("${}", index);
        (*stmt).bound_params.insert(param_name, Value::String(value_str));
        GRAPHDB_OK as c_int
    }
}

// 执行语句
#[no_mangle]
pub extern "C" fn graphdb_step(stmt: *mut graphdb_stmt_t) -> c_int {
    if stmt.is_null() {
        return GRAPHDB_MISUSE as c_int;
    }

    unsafe {
        // 将绑定的参数应用到语句
        for (name, value) in (*stmt).bound_params.iter() {
            match (*stmt).inner.bind(name, value.clone()) {
                Ok(_) => {}
                Err(e) => {
                    (*stmt).last_error = Some(CString::new(format!("{}", e)).unwrap());
                    return error_code_from_core_error(&e);
                }
            }
        }

        // 执行语句
        match (*stmt).inner.execute() {
            Ok(_) => {
                (*stmt).bound_params.clear();
                GRAPHDB_OK as c_int
            }
            Err(e) => {
                (*stmt).last_error = Some(CString::new(format!("{}", e)).unwrap());
                error_code_from_core_error(&e)
            }
        }
    }
}

// 重置语句
#[no_mangle]
pub extern "C" fn graphdb_reset(stmt: *mut graphdb_stmt_t) -> c_int {
    if stmt.is_null() {
        return GRAPHDB_MISUSE as c_int;
    }

    unsafe {
        (*stmt).inner.reset();
        (*stmt).bound_params.clear();
        GRAPHDB_OK as c_int
    }
}

// 清除绑定
#[no_mangle]
pub extern "C" fn graphdb_clear_bindings(stmt: *mut graphdb_stmt_t) -> c_int {
    if stmt.is_null() {
        return GRAPHDB_MISUSE as c_int;
    }

    unsafe {
        (*stmt).bound_params.clear();
        GRAPHDB_OK as c_int
    }
}

// 释放语句
#[no_mangle]
pub extern "C" fn graphdb_finalize(stmt: *mut graphdb_stmt_t) -> c_int {
    if stmt.is_null() {
        return GRAPHDB_MISUSE as c_int;
    }

    unsafe {
        let _ = Box::from_raw(stmt);
        *stmt = ptr::null_mut();
    }
    GRAPHDB_OK as c_int
}
```

#### 步骤 6：实现结果处理

```rust
// src/api/c_api/result.rs
use crate::api::c_api::types::*;
use crate::api::c_api::session::graphdb_session_t;
use crate::api::embedded::QueryResult;
use std::ffi::{CString, c_char};
use std::ptr;

// 结果集句柄
#[repr(C)]
pub struct graphdb_result_t {
    inner: QueryResult,
    current_row: usize,
    last_error: Option<CString>,
}

// 执行查询
#[no_mangle]
pub extern "C" fn graphdb_execute(
    session: *mut graphdb_session_t,
    query: *const c_char,
    result: *mut *mut graphdb_result_t,
) -> c_int {
    if session.is_null() || query.is_null() || result.is_null() {
        return GRAPHDB_MISUSE as c_int;
    }

    let query_str = unsafe {
        match CStr::from_ptr(query).to_str() {
            Ok(s) => s,
            Err(_) => return GRAPHDB_MISUSE as c_int,
        }
    };

    unsafe {
        let sess = &(*session).inner;
        match sess.execute(query_str) {
            Ok(query_result) => {
                let c_result = Box::new(graphdb_result_t {
                    inner: query_result,
                    current_row: 0,
                    last_error: None,
                });
                *result = Box::into_raw(c_result);
                GRAPHDB_OK as c_int
            }
            Err(e) => {
                (*session).last_error = Some(CString::new(format!("{}", e)).unwrap());
                error_code_from_core_error(&e)
            }
        }
    }
}

// 获取列数
#[no_mangle]
pub extern "C" fn graphdb_column_count(result: *mut graphdb_result_t) -> c_int {
    if result.is_null() {
        return -1;
    }

    unsafe {
        (*result).inner.columns().len() as c_int
    }
}

// 获取列名
#[no_mangle]
pub extern "C" fn graphdb_column_name(
    result: *mut graphdb_result_t,
    index: c_int,
) -> *const c_char {
    if result.is_null() {
        return ptr::null();
    }

    unsafe {
        match (*result).inner.columns().get(index as usize) {
            Some(name) => {
                match CString::new(name.as_str()) {
                    Ok(c_name) => c_name.into_raw(),
                    Err(_) => ptr::null(),
                }
            }
            None => ptr::null(),
        }
    }
}

// 获取行数
#[no_mangle]
pub extern "C" fn graphdb_row_count(result: *mut graphdb_result_t) -> c_int {
    if result.is_null() {
        return -1;
    }

    unsafe {
        (*result).inner.len() as c_int
    }
}

// 获取整数值
#[no_mangle]
pub extern "C" fn graphdb_get_int(
    result: *mut graphdb_result_t,
    row: c_int,
    col: *const c_char,
    value: *mut i64,
) -> c_int {
    if result.is_null() || col.is_null() || value.is_null() {
        return GRAPHDB_MISUSE as c_int;
    }

    let col_str = unsafe {
        match CStr::from_ptr(col).to_str() {
            Ok(s) => s,
            Err(_) => return GRAPHDB_MISUSE as c_int,
        }
    };

    unsafe {
        match (*result).inner.get(row as usize) {
            Some(row_data) => {
                match row_data.get(col_str) {
                    Some(Value::Int(i)) => {
                        *value = *i;
                        GRAPHDB_OK as c_int
                    }
                    Some(_) => GRAPHDB_MISMATCH as c_int,
                    None => GRAPHDB_NOTFOUND as c_int,
                }
            }
            None => GRAPHDB_NOTFOUND as c_int,
        }
    }
}

// 获取字符串值
#[no_mangle]
pub extern "C" fn graphdb_get_string(
    result: *mut graphdb_result_t,
    row: c_int,
    col: *const c_char,
    len: *mut c_int,
) -> *const c_char {
    if result.is_null() || col.is_null() {
        return ptr::null();
    }

    let col_str = unsafe {
        match CStr::from_ptr(col).to_str() {
            Ok(s) => s,
            Err(_) => return ptr::null(),
        }
    };

    unsafe {
        match (*result).inner.get(row as usize) {
            Some(row_data) => {
                match row_data.get(col_str) {
                    Some(Value::String(s)) => {
                        if !len.is_null() {
                            *len = s.len() as c_int;
                        }
                        match CString::new(s.as_str()) {
                            Ok(c_str) => c_str.into_raw(),
                            Err(_) => ptr::null(),
                        }
                    }
                    Some(_) => {
                        if !len.is_null() {
                            *len = -1;
                        }
                        ptr::null()
                    }
                    None => ptr::null(),
                }
            }
            None => ptr::null(),
        }
    }
}

// 释放结果
#[no_mangle]
pub extern "C" fn graphdb_result_free(result: *mut graphdb_result_t) -> c_int {
    if result.is_null() {
        return GRAPHDB_MISUSE as c_int;
    }

    unsafe {
        let _ = Box::from_raw(result);
        *result = ptr::null_mut();
    }
    GRAPHDB_OK as c_int
}
```

#### 步骤 7：实现事务管理

```rust
// src/api/c_api/transaction.rs
use crate::api::c_api::types::*;
use crate::api::c_api::session::graphdb_session_t;
use crate::api::embedded::Transaction;
use std::ffi::{CString, c_char};
use std::ptr;

// 事务句柄
#[repr(C)]
pub struct graphdb_txn_t {
    inner: Transaction<'static, crate::storage::RedbStorage>,
    last_error: Option<CString>,
}

// 开始事务
#[no_mangle]
pub extern "C" fn graphdb_txn_begin(
    session: *mut graphdb_session_t,
    txn: *mut *mut graphdb_txn_t,
) -> c_int {
    if session.is_null() || txn.is_null() {
        return GRAPHDB_MISUSE as c_int;
    }

    unsafe {
        let sess = &(*session).inner;
        // 注意：这里需要处理生命周期问题
        // 实际实现中可能需要使用 Arc 或其他方式
        match sess.begin_transaction() {
            Ok(transaction) => {
                let c_txn = Box::new(graphdb_txn_t {
                    inner: transaction,
                    last_error: None,
                });
                *txn = Box::into_raw(c_txn);
                GRAPHDB_OK as c_int
            }
            Err(e) => {
                (*session).last_error = Some(CString::new(format!("{}", e)).unwrap());
                error_code_from_core_error(&e)
            }
        }
    }
}

// 提交事务
#[no_mangle]
pub extern "C" fn graphdb_txn_commit(txn: *mut graphdb_txn_t) -> c_int {
    if txn.is_null() {
        return GRAPHDB_MISUSE as c_int;
    }

    unsafe {
        match (*txn).inner.commit() {
            Ok(_) => {
                let _ = Box::from_raw(txn);
                *txn = ptr::null_mut();
                GRAPHDB_OK as c_int
            }
            Err(e) => {
                (*txn).last_error = Some(CString::new(format!("{}", e)).unwrap());
                error_code_from_core_error(&e)
            }
        }
    }
}

// 回滚事务
#[no_mangle]
pub extern "C" fn graphdb_txn_rollback(txn: *mut graphdb_txn_t) -> c_int {
    if txn.is_null() {
        return GRAPHDB_MISUSE as c_int;
    }

    unsafe {
        match (*txn).inner.rollback() {
            Ok(_) => {
                let _ = Box::from_raw(txn);
                *txn = ptr::null_mut();
                GRAPHDB_OK as c_int
            }
            Err(e) => {
                (*txn).last_error = Some(CString::new(format!("{}", e)).unwrap());
                error_code_from_core_error(&e)
            }
        }
    }
}
```

#### 步骤 8：实现错误处理

```rust
// src/api/c_api/error.rs
use crate::api::core::CoreError;

// 错误码枚举
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum graphdb_error_code_t {
    GRAPHDB_OK = 0,
    GRAPHDB_ERROR = 1,
    GRAPHDB_INTERNAL = 2,
    GRAPHDB_PERM = 3,
    GRAPHDB_ABORT = 4,
    GRAPHDB_BUSY = 5,
    GRAPHDB_LOCKED = 6,
    GRAPHDB_NOMEM = 7,
    GRAPHDB_READONLY = 8,
    GRAPHDB_INTERRUPT = 9,
    GRAPHDB_IOERR = 10,
    GRAPHDB_CORRUPT = 11,
    GRAPHDB_NOTFOUND = 12,
    GRAPHDB_FULL = 13,
    GRAPHDB_CANTOPEN = 14,
    GRAPHDB_PROTOCOL = 15,
    GRAPHDB_SCHEMA = 16,
    GRAPHDB_TOOBIG = 17,
    GRAPHDB_CONSTRAINT = 18,
    GRAPHDB_MISMATCH = 19,
    GRAPHDB_MISUSE = 20,
    GRAPHDB_RANGE = 21,
}

// 从核心错误转换为 C 错误码
pub fn error_code_from_core_error(error: &CoreError) -> i32 {
    match error {
        CoreError::StorageError(_) => GRAPHDB_IOERR as i32,
        CoreError::QueryExecutionFailed(_) => GRAPHDB_ERROR as i32,
        CoreError::ValidationError(_) => GRAPHDB_CONSTRAINT as i32,
        CoreError::TransactionError(_) => GRAPHDB_ABORT as i32,
        CoreError::SchemaError(_) => GRAPHDB_SCHEMA as i32,
        CoreError::Internal(_) => GRAPHDB_INTERNAL as i32,
        CoreError::NotFound => GRAPHDB_NOTFOUND as i32,
        CoreError::PermissionDenied => GRAPHDB_PERM as i32,
        CoreError::Timeout => GRAPHDB_BUSY as i32,
        CoreError::LockError => GRAPHDB_LOCKED as i32,
        CoreError::OutOfMemory => GRAPHDB_NOMEM as i32,
        CoreError::ReadOnly => GRAPHDB_READONLY as i32,
        CoreError::InvalidInput => GRAPHDB_MISUSE as i32,
        CoreError::OutOfRange => GRAPHDB_RANGE as i32,
        CoreError::TypeMismatch => GRAPHDB_MISMATCH as i32,
        CoreError::CorruptedData => GRAPHDB_CORRUPT as i32,
        CoreError::DiskFull => GRAPHDB_FULL as i32,
        CoreError::CannotOpen => GRAPHDB_CANTOPEN as i32,
        CoreError::ProtocolError => GRAPHDB_PROTOCOL as i32,
    }
}
```

#### 步骤 9：配置 cbindgen

创建 `cbindgen.toml` 文件：

```toml
language = "C"

[export]
prefix = "GRAPHDB_"

[parse]
parse_deps = true
include = ["graphdb"]

[fn]
prefix = "GRAPHDB_"

[struct]
prefix = "GRAPHDB_"

[enum]
prefix = "GRAPHDB_"
```

在 `Cargo.toml` 中添加构建脚本：

```toml
[build-dependencies]
cbindgen = "0.24"
```

创建 `build.rs`：

```rust
fn main() {
    cbindgen::Builder::default()
        .with_crate(".")
        .with_language(cbindgen::Language::C)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file("include/graphdb.h");
}
```

---

## 代码示例

### 示例 1：基本查询

```c
#include <stdio.h>
#include "graphdb.h"

int main() {
    graphdb_t *db = NULL;
    graphdb_session_t *session = NULL;
    graphdb_result_t *result = NULL;

    // 打开数据库
    int rc = graphdb_open("test.db", &db);
    if (rc != GRAPHDB_OK) {
        fprintf(stderr, "无法打开数据库: %s\n", graphdb_errmsg(db));
        return 1;
    }

    // 创建会话
    rc = graphdb_session_create(db, &session);
    if (rc != GRAPHDB_OK) {
        fprintf(stderr, "无法创建会话: %s\n", graphdb_errmsg(db));
        graphdb_close(db);
        return 1;
    }

    // 切换图空间
    rc = graphdb_session_use_space(session, "my_space");
    if (rc != GRAPHDB_OK) {
        fprintf(stderr, "无法切换图空间: %s\n", graphdb_errmsg(db));
    }

    // 执行查询
    rc = graphdb_execute(session, "MATCH (n) RETURN n LIMIT 10", &result);
    if (rc != GRAPHDB_OK) {
        fprintf(stderr, "查询失败: %s\n", graphdb_errmsg(db));
    } else {
        // 处理结果
        int row_count = graphdb_row_count(result);
        int col_count = graphdb_column_count(result);

        printf("查询结果: %d 行, %d 列\n", row_count, col_count);

        for (int i = 0; i < row_count; i++) {
            for (int j = 0; j < col_count; j++) {
                const char *col_name = graphdb_column_name(result, j);
                int64_t value;
                if (graphdb_get_int(result, i, col_name, &value) == GRAPHDB_OK) {
                    printf("%s: %lld  ", col_name, value);
                }
            }
            printf("\n");
        }

        graphdb_result_free(result);
    }

    // 清理资源
    graphdb_session_close(session);
    graphdb_close(db);

    return 0;
}
```

### 示例 2：预编译语句

```c
#include <stdio.h>
#include "graphdb.h"

int main() {
    graphdb_t *db = NULL;
    graphdb_session_t *session = NULL;
    graphdb_stmt_t *stmt = NULL;

    // 打开数据库
    graphdb_open("test.db", &db);
    graphdb_session_create(db, &session);
    graphdb_session_use_space(session, "my_space");

    // 准备语句
    int rc = graphdb_prepare(session, "MATCH (n:User {id: $id}) RETURN n.name", &stmt);
    if (rc != GRAPHDB_OK) {
        fprintf(stderr, "准备语句失败: %s\n", graphdb_errmsg(db));
        goto cleanup;
    }

    // 绑定参数并执行
    for (int i = 1; i <= 10; i++) {
        graphdb_bind_int(stmt, 1, i);

        rc = graphdb_step(stmt);
        if (rc == GRAPHDB_OK) {
            printf("查询用户 %d 成功\n", i);
        } else {
            fprintf(stderr, "查询失败: %s\n", graphdb_errmsg(db));
        }

        graphdb_reset(stmt);
    }

cleanup:
    if (stmt) graphdb_finalize(stmt);
    if (session) graphdb_session_close(session);
    if (db) graphdb_close(db);

    return 0;
}
```

### 示例 3：事务操作

```c
#include <stdio.h>
#include "graphdb.h"

int main() {
    graphdb_t *db = NULL;
    graphdb_session_t *session = NULL;
    graphdb_txn_t *txn = NULL;

    // 打开数据库
    graphdb_open("test.db", &db);
    graphdb_session_create(db, &session);
    graphdb_session_use_space(session, "my_space");

    // 开始事务
    int rc = graphdb_txn_begin(session, &txn);
    if (rc != GRAPHDB_OK) {
        fprintf(stderr, "无法开始事务: %s\n", graphdb_errmsg(db));
        goto cleanup;
    }

    // 在事务中执行多个操作
    rc = graphdb_txn_execute(txn, "CREATE TAG user(name string, age int)", NULL);
    if (rc != GRAPHDB_OK) {
        fprintf(stderr, "创建标签失败: %s\n", graphdb_errmsg(db));
        graphdb_txn_rollback(txn);
        goto cleanup;
    }

    rc = graphdb_txn_execute(txn, "INSERT VERTEX user(name, age) VALUES \"1\":(\"Alice\", 30)", NULL);
    if (rc != GRAPHDB_OK) {
        fprintf(stderr, "插入顶点失败: %s\n", graphdb_errmsg(db));
        graphdb_txn_rollback(txn);
        goto cleanup;
    }

    rc = graphdb_txn_execute(txn, "INSERT VERTEX user(name, age) VALUES \"2\":(\"Bob\", 25)", NULL);
    if (rc != GRAPHDB_OK) {
        fprintf(stderr, "插入顶点失败: %s\n", graphdb_errmsg(db));
        graphdb_txn_rollback(txn);
        goto cleanup;
    }

    // 提交事务
    rc = graphdb_txn_commit(txn);
    if (rc != GRAPHDB_OK) {
        fprintf(stderr, "提交事务失败: %s\n", graphdb_errmsg(db));
    } else {
        printf("事务提交成功\n");
    }

cleanup:
    if (session) graphdb_session_close(session);
    if (db) graphdb_close(db);

    return 0;
}
```

### 示例 4：批量插入

```c
#include <stdio.h>
#include "graphdb.h"

int main() {
    graphdb_t *db = NULL;
    graphdb_session_t *session = NULL;
    graphdb_batch_inserter_t *inserter = NULL;

    // 打开数据库
    graphdb_open("test.db", &db);
    graphdb_session_create(db, &session);
    graphdb_session_use_space(session, "my_space");

    // 创建批量插入器
    int rc = graphdb_batch_inserter_create(session, "user", &inserter);
    if (rc != GRAPHDB_OK) {
        fprintf(stderr, "无法创建批量插入器: %s\n", graphdb_errmsg(db));
        goto cleanup;
    }

    // 批量添加顶点
    for (int i = 0; i < 10000; i++) {
        char name[64];
        snprintf(name, sizeof(name), "User_%d", i);

        graphdb_value_t props[2];
        props[0] = graphdb_value_string(name);
        props[1] = graphdb_value_int(20 + i % 50);

        rc = graphdb_batch_add_vertex(inserter, i, props, 2);
        if (rc != GRAPHDB_OK) {
            fprintf(stderr, "添加顶点失败: %s\n", graphdb_errmsg(db));
            break;
        }
    }

    // 执行批量插入
    rc = graphdb_batch_flush(inserter);
    if (rc == GRAPHDB_OK) {
        printf("批量插入成功\n");
    } else {
        fprintf(stderr, "批量插入失败: %s\n", graphdb_errmsg(db));
    }

cleanup:
    if (inserter) graphdb_batch_inserter_free(inserter);
    if (session) graphdb_session_close(session);
    if (db) graphdb_close(db);

    return 0;
}
```

---

## 技术细节

### 1. 内存管理

#### 所有权模型

- **Rust 端**：使用 `Box` 包装 C API 结构体，确保 Rust 的所有权语义
- **C 端**：通过不透明指针访问，不直接管理内存
- **释放**：提供显式的 `*_close`、`*_free` 函数释放资源

#### 字符串处理

```rust
// C 字符串转 Rust 字符串
let c_str = unsafe { CStr::from_ptr(c_ptr) };
let rust_str = c_str.to_str()?;

// Rust 字符串转 C 字符串
let c_string = CString::new(rust_str)?;
let c_ptr = c_string.into_raw(); // 转移所有权给 C

// 释放 C 字符串（由 C 端调用）
unsafe {
    let _ = CString::from_raw(c_ptr);
}
```

#### 生命周期管理

```rust
// 使用 Arc 共享数据库实例
pub struct graphdb_t {
    inner: Arc<GraphDatabase<RedbStorage>>,
    last_error: Option<CString>,
}

// 会话引用数据库
pub struct graphdb_session_t {
    inner: Session<RedbStorage>,
    last_error: Option<CString>,
}
```

### 2. 线程安全

#### Rust 端线程安全

```rust
// 使用 Arc<Mutex<T>> 确保线程安全
pub struct GraphDatabase<S: StorageClient + Clone + 'static> {
    inner: Arc<GraphDatabaseInner<S>>,
    config: DatabaseConfig,
}

// 实现 Send + Sync
unsafe impl<S: StorageClient + Clone + 'static> Send for GraphDatabase<S> {}
unsafe impl<S: StorageClient + Clone + 'static> Sync for GraphDatabase<S> {}
```

#### C 端线程安全

- 数据库句柄可以跨线程共享
- 会话句柄建议单线程使用
- 预编译语句与创建它的会话绑定

### 3. 错误处理

#### 错误码映射

```rust
pub fn error_code_from_core_error(error: &CoreError) -> i32 {
    match error {
        CoreError::StorageError(_) => GRAPHDB_IOERR as i32,
        CoreError::QueryExecutionFailed(_) => GRAPHDB_ERROR as i32,
        CoreError::ValidationError(_) => GRAPHDB_CONSTRAINT as i32,
        // ... 其他错误
    }
}
```

#### 错误信息存储

```rust
pub struct graphdb_t {
    inner: Arc<GraphDatabase<RedbStorage>>,
    last_error: Option<CString>,  // 存储最后一次错误信息
}

// 获取错误信息
#[no_mangle]
pub extern "C" fn graphdb_errmsg(db: *mut graphdb_t) -> *const c_char {
    if db.is_null() {
        return ptr::null();
    }

    unsafe {
        match &(*db).last_error {
            Some(msg) => msg.as_ptr(),
            None => ptr::null(),
        }
    }
}
```

### 4. 类型转换

#### Value 类型转换

```rust
impl Value {
    // Rust Value -> C Value
    pub fn to_c_value(&self) -> graphdb_value_t {
        match self {
            Value::Null => graphdb_value_t {
                type_: graphdb_value_type_t::GRAPHDB_NULL,
                data: graphdb_value_data_t { ptr: std::ptr::null_mut() },
            },
            Value::Int(i) => graphdb_value_t {
                type_: graphdb_value_type_t::GRAPHDB_INT,
                data: graphdb_value_data_t { integer: *i },
            },
            // ... 其他类型
        }
    }

    // C Value -> Rust Value
    pub unsafe fn from_c_value(c_value: &graphdb_value_t) -> Self {
        match c_value.type_ {
            graphdb_value_type_t::GRAPHDB_NULL => Value::Null,
            graphdb_value_type_t::GRAPHDB_INT => Value::Int(c_value.data.integer),
            // ... 其他类型
        }
    }
}
```

### 5. 性能优化

#### 预编译语句缓存

```rust
pub struct PreparedStatement<S: StorageClient + 'static> {
    query_api: Arc<Mutex<QueryApi<S>>>,
    query: String,
    parameter_types: HashMap<String, DataType>,
    bound_params: HashMap<String, Value>,
    config: StatementConfig,  // 包含缓存配置
    stats: ExecutionStats,
}
```

#### 批量操作

```rust
pub struct BatchInserter<S: StorageClient + 'static> {
    session: Arc<GraphDatabaseInner<S>>,
    buffer: Vec<VertexData>,
    config: BatchConfig,
}
```

#### 零拷贝转换

```rust
// 对于字符串等类型，尽量使用零拷贝转换
pub struct graphdb_string_t {
    pub data: *const c_char,
    pub len: usize,
}
```

---

## 注意事项

### 1. 安全性

#### 避免内存泄漏

- 确保所有分配的资源都有对应的释放函数
- 使用 `Box::from_raw` 和 `Box::into_raw` 正确管理所有权
- 文档中明确说明资源释放责任

#### 避免悬垂指针

- 使用 `Arc` 共享数据库实例，确保生命周期足够长
- 会话和语句的生命周期不能超过数据库
- 文档中明确说明对象之间的依赖关系

#### 避免数据竞争

- 使用 `Mutex` 保护共享状态
- 明确说明哪些操作是线程安全的
- 提供线程安全的使用示例

### 2. 兼容性

#### 平台兼容性

- 使用标准 C 类型（`c_int`、`c_char` 等）
- 避免使用平台特定的类型
- 测试不同平台的编译和运行

#### 编译器兼容性

- 使用 `#[no_mangle]` 确保函数名不被修改
- 使用 `extern "C"` 确保调用约定正确
- 避免使用 Rust 特有的类型特性

#### ABI 稳定性

- 保持 API 的向后兼容性
- 使用版本号管理 API 变化
- 提供迁移指南

### 3. 文档

#### API 文档

- 为每个函数提供详细的文档
- 包含参数说明、返回值说明、错误码说明
- 提供使用示例

#### 错误码文档

- 列出所有错误码及其含义
- 说明每个错误码的可能原因
- 提供错误处理建议

#### 最佳实践

- 提供性能优化建议
- 说明常见陷阱和避免方法
- 提供完整的使用示例

### 4. 测试

#### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graphdb_open_close() {
        let mut db: *mut graphdb_t = std::ptr::null_mut();
        let rc = graphdb_open(std::ptr::null(), &mut db);
        assert_eq!(rc, GRAPHDB_MISUSE as i32);

        let rc = graphdb_close(db);
        assert_eq!(rc, GRAPHDB_OK as i32);
    }
}
```

#### 集成测试

```c
// tests/c_api_test.c
#include <stdio.h>
#include <assert.h>
#include "graphdb.h"

void test_basic_operations() {
    graphdb_t *db = NULL;
    int rc = graphdb_open("test.db", &db);
    assert(rc == GRAPHDB_OK);

    graphdb_session_t *session = NULL;
    rc = graphdb_session_create(db, &session);
    assert(rc == GRAPHDB_OK);

    graphdb_session_close(session);
    graphdb_close(db);
}

int main() {
    test_basic_operations();
    printf("所有测试通过\n");
    return 0;
}
```

#### 性能测试

- 测试预编译语句的性能提升
- 测试批量操作的性能
- 测试并发访问的性能

### 5. 构建和发布

#### 构建脚本

```rust
// build.rs
fn main() {
    cbindgen::Builder::default()
        .with_crate(".")
        .with_language(cbindgen::Language::C)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file("include/graphdb.h");
}
```

#### Cargo.toml 配置

```toml
[lib]
name = "graphdb"
crate-type = ["cdylib", "rlib"]

[build-dependencies]
cbindgen = "0.24"

[dependencies]
# ... 其他依赖
```

#### 发布

- 同时发布 Rust crate 和 C 库
- 提供预编译的二进制文件
- 提供详细的安装说明

---

## 总结

本文档详细分析了如何为 GraphDB 实现标准 C API，参考了 SQLite 的设计模式，提供了完整的架构设计和实现方案。

### 主要特点

1. **与 SQLite API 风格一致**：采用相似的命名和调用模式，降低学习成本
2. **保持 Rust API 功能**：不牺牲现有功能，提供完整的图数据库功能
3. **类型安全**：提供强类型的 C 接口，减少错误
4. **线程安全**：支持多线程并发访问
5. **资源管理**：明确的资源生命周期，避免内存泄漏

### 实现步骤

1. 创建 C API 模块结构
2. 实现类型转换（Rust ↔ C）
3. 实现核心 API 函数
4. 配置 cbindgen 生成头文件
5. 编写测试用例
6. 编写文档和示例

### 后续工作

- 实现完整的 C API 代码
- 编写全面的测试用例
- 提供多语言绑定示例（Python、Go、Java 等）
- 性能优化和基准测试
- 发布和文档完善

通过这个 C API，GraphDB 可以被各种编程语言使用，大大扩展了其应用场景和用户群体。
