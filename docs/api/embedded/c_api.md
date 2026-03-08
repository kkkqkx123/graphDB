# GraphDB C API 详细文档

## 概述

GraphDB C API 提供与 SQLite 类似的 C 语言接口，允许在 C/C++ 程序中嵌入图数据库功能。

## 头文件

```c
#include <graphdb.h>
```

## 核心类型

### 不透明句柄类型

```c
// 数据库句柄
typedef struct graphdb_t graphdb_t;

// 会话句柄
typedef struct graphdb_session_t graphdb_session_t;

// 预编译语句句柄
typedef struct graphdb_stmt_t graphdb_stmt_t;

// 事务句柄
typedef struct graphdb_txn_t graphdb_txn_t;

// 结果集句柄
typedef struct graphdb_result_t graphdb_result_t;

// 批量操作句柄
typedef struct graphdb_batch_t graphdb_batch_t;
```

### 值类型枚举

```c
typedef enum {
    GRAPHDB_NULL = 0,      // 空值
    GRAPHDB_BOOL = 1,      // 布尔值
    GRAPHDB_INT = 2,       // 整数
    GRAPHDB_FLOAT = 3,     // 浮点数
    GRAPHDB_STRING = 4,    // 字符串
    GRAPHDB_LIST = 5,      // 列表
    GRAPHDB_MAP = 6,       // 映射
    GRAPHDB_VERTEX = 7,    // 顶点
    GRAPHDB_EDGE = 8,      // 边
    GRAPHDB_PATH = 9       // 路径
} graphdb_value_type_t;
```

### 值结构

```c
typedef struct {
    graphdb_value_type_t type_;
    union {
        bool boolean;           // 布尔值
        int64_t integer;        // 整数
        double floating;        // 浮点数
        struct {
            const char* data;   // 字符串数据
            size_t len;         // 字符串长度
        } string;
        void* ptr;              // 指针
    } data;
} graphdb_value_t;
```

### 错误码

```c
typedef enum {
    GRAPHDB_OK = 0,          // 成功
    GRAPHDB_ERROR = 1,       // 一般错误
    GRAPHDB_INTERNAL = 2,    // 内部错误
    GRAPHDB_PERM = 3,        // 权限被拒绝
    GRAPHDB_ABORT = 4,       // 操作被中止
    GRAPHDB_BUSY = 5,        // 数据库忙
    GRAPHDB_LOCKED = 6,      // 数据库被锁定
    GRAPHDB_NOMEM = 7,       // 内存不足
    GRAPHDB_READONLY = 8,    // 只读
    GRAPHDB_INTERRUPT = 9,   // 操作被中断
    GRAPHDB_IOERR = 10,      // IO 错误
    GRAPHDB_CORRUPT = 11,    // 数据损坏
    GRAPHDB_NOTFOUND = 12,   // 未找到
    GRAPHDB_FULL = 13,       // 磁盘已满
    GRAPHDB_CANTOPEN = 14,   // 无法打开
    GRAPHDB_PROTOCOL = 15,   // 协议错误
    GRAPHDB_SCHEMA = 16,     // 模式错误
    GRAPHDB_TOOBIG = 17,     // 数据过大
    GRAPHDB_CONSTRAINT = 18, // 约束违反
    GRAPHDB_MISMATCH = 19,   // 类型不匹配
    GRAPHDB_MISUSE = 20,     // 误用
    GRAPHDB_RANGE = 21       // 超出范围
} graphdb_error_code_t;
```

---

## 数据库操作

### graphdb_open()

打开或创建数据库。

```c
int graphdb_open(const char* path, graphdb_t** db);
```

**参数：**
- `path`: 数据库文件路径（UTF-8 编码）
- `db`: 输出参数，数据库句柄

**返回：**
- `GRAPHDB_OK`: 成功
- 其他: 错误码

**示例：**
```c
graphdb_t* db = NULL;
int rc = graphdb_open("my_database.db", &db);
if (rc != GRAPHDB_OK) {
    fprintf(stderr, "打开数据库失败: %d\n", rc);
    return 1;
}
```

### graphdb_close()

关闭数据库。

```c
int graphdb_close(graphdb_t* db);
```

**参数：**
- `db`: 数据库句柄

**返回：**
- `GRAPHDB_OK`: 成功
- `GRAPHDB_MISUSE`: 无效参数

**示例：**
```c
int rc = graphdb_close(db);
if (rc != GRAPHDB_OK) {
    fprintf(stderr, "关闭数据库失败: %d\n", rc);
}
```

### graphdb_libversion()

获取库版本。

```c
const char* graphdb_libversion(void);
```

**返回：**
- 版本字符串（静态生命周期，无需释放）

**示例：**
```c
printf("GraphDB 版本: %s\n", graphdb_libversion());
```

### graphdb_free_string()

释放由 GraphDB 分配的字符串。

```c
void graphdb_free_string(char* str);
```

**参数：**
- `str`: 字符串指针

### graphdb_free()

释放由 GraphDB 分配的内存。

```c
void graphdb_free(void* ptr);
```

**参数：**
- `ptr`: 内存指针

---

## 会话管理

### graphdb_session_create()

创建会话。

```c
int graphdb_session_create(graphdb_t* db, graphdb_session_t** session);
```

**参数：**
- `db`: 数据库句柄
- `session`: 输出参数，会话句柄

**返回：**
- `GRAPHDB_OK`: 成功
- 其他: 错误码

**示例：**
```c
graphdb_session_t* session = NULL;
int rc = graphdb_session_create(db, &session);
if (rc != GRAPHDB_OK) {
    fprintf(stderr, "创建会话失败: %d\n", rc);
    graphdb_close(db);
    return 1;
}
```

### graphdb_session_close()

关闭会话。

```c
int graphdb_session_close(graphdb_session_t* session);
```

**参数：**
- `session`: 会话句柄

**返回：**
- `GRAPHDB_OK`: 成功
- `GRAPHDB_MISUSE`: 无效参数

### graphdb_session_use_space()

切换图空间。

```c
int graphdb_session_use_space(graphdb_session_t* session, const char* space_name);
```

**参数：**
- `session`: 会话句柄
- `space_name`: 图空间名称（UTF-8 编码）

**返回：**
- `GRAPHDB_OK`: 成功
- 其他: 错误码

**示例：**
```c
int rc = graphdb_session_use_space(session, "test_space");
if (rc != GRAPHDB_OK) {
    fprintf(stderr, "切换图空间失败: %d\n", rc);
}
```

### graphdb_session_current_space()

获取当前图空间。

```c
const char* graphdb_session_current_space(graphdb_session_t* session);
```

**参数：**
- `session`: 会话句柄

**返回：**
- 当前图空间名称（需要调用 `graphdb_free_string` 释放）
- `NULL`: 未选择图空间或出错

### graphdb_session_set_autocommit()

设置自动提交模式。

```c
int graphdb_session_set_autocommit(graphdb_session_t* session, bool autocommit);
```

**参数：**
- `session`: 会话句柄
- `autocommit`: 是否自动提交

**返回：**
- `GRAPHDB_OK`: 成功

### graphdb_session_get_autocommit()

获取自动提交模式。

```c
bool graphdb_session_get_autocommit(graphdb_session_t* session);
```

**参数：**
- `session`: 会话句柄

**返回：**
- `true`: 自动提交开启
- `false`: 自动提交关闭

---

## 查询执行

### graphdb_execute()

执行简单查询。

```c
int graphdb_execute(
    graphdb_session_t* session,
    const char* query,
    graphdb_result_t** result
);
```

**参数：**
- `session`: 会话句柄
- `query`: 查询语句（UTF-8 编码）
- `result`: 输出参数，结果集句柄

**返回：**
- `GRAPHDB_OK`: 成功
- 其他: 错误码

**示例：**
```c
graphdb_result_t* result = NULL;
int rc = graphdb_execute(session, "MATCH (n) RETURN n LIMIT 10", &result);
if (rc != GRAPHDB_OK) {
    fprintf(stderr, "查询失败: %d\n", rc);
} else {
    // 处理结果
    graphdb_result_free(result);
}
```

### graphdb_execute_params()

执行参数化查询。

```c
int graphdb_execute_params(
    graphdb_session_t* session,
    const char* query,
    const graphdb_value_t* params,
    size_t param_count,
    graphdb_result_t** result
);
```

**参数：**
- `session`: 会话句柄
- `query`: 查询语句（UTF-8 编码）
- `params`: 参数数组
- `param_count`: 参数数量
- `result`: 输出参数，结果集句柄

**返回：**
- `GRAPHDB_OK`: 成功
- 其他: 错误码

**示例：**
```c
graphdb_value_t params[2];
params[0].type_ = GRAPHDB_INT;
params[0].data.integer = 1;

params[1].type_ = GRAPHDB_STRING;
params[1].data.string.data = "Alice";
params[1].data.string.len = 5;

graphdb_result_t* result = NULL;
int rc = graphdb_execute_params(
    session,
    "MATCH (n:User {id: $param_0, name: $param_1}) RETURN n",
    params, 2, &result
);
```

---

## 结果处理

### graphdb_result_free()

释放结果集。

```c
int graphdb_result_free(graphdb_result_t* result);
```

**参数：**
- `result`: 结果集句柄

**返回：**
- `GRAPHDB_OK`: 成功

### graphdb_column_count()

获取结果集列数。

```c
int graphdb_column_count(graphdb_result_t* result);
```

**参数：**
- `result`: 结果集句柄

**返回：**
- 列数，错误返回 -1

### graphdb_row_count()

获取结果集行数。

```c
int graphdb_row_count(graphdb_result_t* result);
```

**参数：**
- `result`: 结果集句柄

**返回：**
- 行数，错误返回 -1

### graphdb_column_name()

获取列名。

```c
const char* graphdb_column_name(graphdb_result_t* result, int index);
```

**参数：**
- `result`: 结果集句柄
- `index`: 列索引（从 0 开始）

**返回：**
- 列名（需要调用 `graphdb_free_string` 释放）
- `NULL`: 索引越界或出错

### graphdb_get_int()

获取整数值。

```c
int graphdb_get_int(
    graphdb_result_t* result,
    int row,
    const char* col,
    int64_t* value
);
```

**参数：**
- `result`: 结果集句柄
- `row`: 行索引（从 0 开始）
- `col`: 列名（UTF-8 编码）
- `value`: 输出参数，整数值

**返回：**
- `GRAPHDB_OK`: 成功
- `GRAPHDB_NOTFOUND`: 行或列不存在
- `GRAPHDB_MISMATCH`: 类型不匹配

### graphdb_get_string()

获取字符串值。

```c
const char* graphdb_get_string(
    graphdb_result_t* result,
    int row,
    const char* col,
    int* len
);
```

**参数：**
- `result`: 结果集句柄
- `row`: 行索引（从 0 开始）
- `col`: 列名（UTF-8 编码）
- `len`: 输出参数，字符串长度

**返回：**
- 字符串值（需要调用 `graphdb_free_string` 释放）
- `NULL`: 出错

---

## 预编译语句

### graphdb_prepare()

准备语句。

```c
int graphdb_prepare(
    graphdb_session_t* session,
    const char* query,
    graphdb_stmt_t** stmt
);
```

**参数：**
- `session`: 会话句柄
- `query`: 查询语句（UTF-8 编码）
- `stmt`: 输出参数，语句句柄

**返回：**
- `GRAPHDB_OK`: 成功
- 其他: 错误码

**示例：**
```c
graphdb_stmt_t* stmt = NULL;
int rc = graphdb_prepare(session, "MATCH (n:User {id: $id}) RETURN n", &stmt);
if (rc != GRAPHDB_OK) {
    fprintf(stderr, "准备语句失败: %d\n", rc);
}
```

### graphdb_bind_null()

绑定 NULL 值（按索引）。

```c
int graphdb_bind_null(graphdb_stmt_t* stmt, int index);
```

**参数：**
- `stmt`: 语句句柄
- `index`: 参数索引（从 1 开始）

**返回：**
- `GRAPHDB_OK`: 成功
- 其他: 错误码

### graphdb_bind_bool()

绑定布尔值（按索引）。

```c
int graphdb_bind_bool(graphdb_stmt_t* stmt, int index, bool value);
```

### graphdb_bind_int()

绑定整数值（按索引）。

```c
int graphdb_bind_int(graphdb_stmt_t* stmt, int index, int64_t value);
```

### graphdb_bind_float()

绑定浮点值（按索引）。

```c
int graphdb_bind_float(graphdb_stmt_t* stmt, int index, double value);
```

### graphdb_bind_string()

绑定字符串值（按索引）。

```c
int graphdb_bind_string(
    graphdb_stmt_t* stmt,
    int index,
    const char* value,
    int len
);
```

**参数：**
- `stmt`: 语句句柄
- `index`: 参数索引（从 1 开始）
- `value`: 字符串值（UTF-8 编码）
- `len`: 字符串长度（-1 表示自动计算）

**示例：**
```c
// 绑定参数
graphdb_bind_int(stmt, 1, 1);
graphdb_bind_string(stmt, 2, "Alice", -1);

// 执行
graphdb_result_t* result = NULL;
int rc = graphdb_stmt_execute(stmt, &result);
if (rc == GRAPHDB_OK) {
    // 处理结果
    graphdb_result_free(result);
}
```

### graphdb_bind_by_name()

绑定参数（按名称）。

```c
int graphdb_bind_by_name(
    graphdb_stmt_t* stmt,
    const char* name,
    graphdb_value_t value
);
```

### graphdb_reset()

重置语句。

```c
int graphdb_reset(graphdb_stmt_t* stmt);
```

### graphdb_clear_bindings()

清除绑定。

```c
int graphdb_clear_bindings(graphdb_stmt_t* stmt);
```

### graphdb_finalize()

释放语句。

```c
int graphdb_finalize(graphdb_stmt_t* stmt);
```

**参数：**
- `stmt`: 语句句柄

**返回：**
- `GRAPHDB_OK`: 成功

### graphdb_bind_parameter_index()

获取参数索引。

```c
int graphdb_bind_parameter_index(graphdb_stmt_t* stmt, const char* name);
```

**参数：**
- `stmt`: 语句句柄
- `name`: 参数名称（UTF-8 编码）

**返回：**
- 参数索引（从 1 开始）
- 0: 未找到

### graphdb_bind_parameter_name()

获取参数名称。

```c
const char* graphdb_bind_parameter_name(graphdb_stmt_t* stmt, int index);
```

**参数：**
- `stmt`: 语句句柄
- `index`: 参数索引（从 1 开始）

**返回：**
- 参数名称（需要调用 `graphdb_free_string` 释放）
- `NULL`: 未找到

### graphdb_bind_parameter_count()

获取参数数量。

```c
int graphdb_bind_parameter_count(graphdb_stmt_t* stmt);
```

---

## 事务管理

### graphdb_txn_begin()

开始事务。

```c
int graphdb_txn_begin(graphdb_session_t* session, graphdb_txn_t** txn);
```

**参数：**
- `session`: 会话句柄
- `txn`: 输出参数，事务句柄

**返回：**
- `GRAPHDB_OK`: 成功
- 其他: 错误码

**示例：**
```c
graphdb_txn_t* txn = NULL;
int rc = graphdb_txn_begin(session, &txn);
if (rc != GRAPHDB_OK) {
    fprintf(stderr, "开始事务失败: %d\n", rc);
}
```

### graphdb_txn_begin_readonly()

开始只读事务。

```c
int graphdb_txn_begin_readonly(graphdb_session_t* session, graphdb_txn_t** txn);
```

### graphdb_txn_execute()

在事务中执行查询。

```c
int graphdb_txn_execute(
    graphdb_txn_t* txn,
    const char* query,
    graphdb_result_t** result
);
```

**示例：**
```c
graphdb_result_t* result = NULL;
int rc = graphdb_txn_execute(txn, "CREATE TAG user(name string)", &result);
if (rc == GRAPHDB_OK && result) {
    graphdb_result_free(result);
}
```

### graphdb_txn_commit()

提交事务。

```c
int graphdb_txn_commit(graphdb_txn_t* txn);
```

**示例：**
```c
int rc = graphdb_txn_commit(txn);
if (rc != GRAPHDB_OK) {
    fprintf(stderr, "提交事务失败: %d\n", rc);
}
```

### graphdb_txn_rollback()

回滚事务。

```c
int graphdb_txn_rollback(graphdb_txn_t* txn);
```

### graphdb_txn_savepoint()

创建保存点。

```c
int64_t graphdb_txn_savepoint(graphdb_txn_t* txn, const char* name);
```

**参数：**
- `txn`: 事务句柄
- `name`: 保存点名称（UTF-8 编码）

**返回：**
- 保存点 ID（正数）
- -1: 失败

**示例：**
```c
int64_t sp = graphdb_txn_savepoint(txn, "checkpoint1");
if (sp < 0) {
    fprintf(stderr, "创建保存点失败\n");
}
```

### graphdb_txn_release_savepoint()

释放保存点。

```c
int graphdb_txn_release_savepoint(graphdb_txn_t* txn, int64_t savepoint_id);
```

### graphdb_txn_rollback_to_savepoint()

回滚到保存点。

```c
int graphdb_txn_rollback_to_savepoint(graphdb_txn_t* txn, int64_t savepoint_id);
```

### graphdb_txn_free()

释放事务句柄。

```c
int graphdb_txn_free(graphdb_txn_t* txn);
```

**注意：**
- 如果事务未提交或回滚，会自动回滚

---

## 批量操作

### graphdb_batch_inserter_create()

创建批量插入器。

```c
int graphdb_batch_inserter_create(
    graphdb_session_t* session,
    int batch_size,
    graphdb_batch_t** batch
);
```

**参数：**
- `session`: 会话句柄
- `batch_size`: 批次大小
- `batch`: 输出参数，批量操作句柄

**返回：**
- `GRAPHDB_OK`: 成功

### graphdb_batch_add_vertex()

添加顶点。

```c
int graphdb_batch_add_vertex(
    graphdb_batch_t* batch,
    int64_t vid,
    const char* tag_name,
    const graphdb_value_t* properties,
    size_t prop_count
);
```

**参数：**
- `batch`: 批量操作句柄
- `vid`: 顶点 ID
- `tag_name`: 标签名称（UTF-8 编码）
- `properties`: 属性数组
- `prop_count`: 属性数量

### graphdb_batch_add_edge()

添加边。

```c
int graphdb_batch_add_edge(
    graphdb_batch_t* batch,
    int64_t src_vid,
    int64_t dst_vid,
    const char* edge_type,
    int64_t rank,
    const graphdb_value_t* properties,
    size_t prop_count
);
```

### graphdb_batch_flush()

执行批量插入。

```c
int graphdb_batch_flush(graphdb_batch_t* batch);
```

### graphdb_batch_buffered_vertices()

获取缓冲的顶点数量。

```c
int graphdb_batch_buffered_vertices(graphdb_batch_t* batch);
```

### graphdb_batch_buffered_edges()

获取缓冲的边数量。

```c
int graphdb_batch_buffered_edges(graphdb_batch_t* batch);
```

### graphdb_batch_free()

释放批量操作句柄。

```c
int graphdb_batch_free(graphdb_batch_t* batch);
```

---

## 错误处理

### graphdb_errmsg()

获取最后一个错误消息。

```c
int graphdb_errmsg(char* msg, size_t len);
```

**参数：**
- `msg`: 输出缓冲区
- `len`: 缓冲区长度

**返回：**
- 实际写入的字符数（不包括 null 终止符）

**示例：**
```c
char error_msg[256];
int len = graphdb_errmsg(error_msg, sizeof(error_msg));
if (len > 0) {
    fprintf(stderr, "错误: %s\n", error_msg);
}
```

### graphdb_error_string()

获取错误码描述。

```c
const char* graphdb_error_string(int code);
```

**参数：**
- `code`: 错误码

**返回：**
- 错误描述字符串（静态生命周期，无需释放）

**示例：**
```c
int rc = graphdb_execute(session, "INVALID QUERY", &result);
if (rc != GRAPHDB_OK) {
    fprintf(stderr, "错误 (%d): %s\n", rc, graphdb_error_string(rc));
}
```

### graphdb_get_last_error_message()

获取最后的错误消息（线程安全）。

```c
const char* graphdb_get_last_error_message(void);
```

**返回：**
- 错误消息字符串指针（线程局部存储，不需要释放）
- `NULL`: 没有错误

---

## 完整示例

### 基本使用示例

```c
#include <graphdb.h>
#include <stdio.h>
#include <stdlib.h>

int main() {
    printf("GraphDB 版本: %s\n", graphdb_libversion());

    // 打开数据库
    graphdb_t* db = NULL;
    int rc = graphdb_open("test.db", &db);
    if (rc != GRAPHDB_OK) {
        fprintf(stderr, "打开数据库失败: %s\n", graphdb_error_string(rc));
        return 1;
    }

    // 创建会话
    graphdb_session_t* session = NULL;
    rc = graphdb_session_create(db, &session);
    if (rc != GRAPHDB_OK) {
        fprintf(stderr, "创建会话失败: %s\n", graphdb_error_string(rc));
        graphdb_close(db);
        return 1;
    }

    // 切换图空间
    rc = graphdb_session_use_space(session, "test_space");
    if (rc != GRAPHDB_OK) {
        fprintf(stderr, "切换图空间失败: %s\n", graphdb_error_string(rc));
        graphdb_session_close(session);
        graphdb_close(db);
        return 1;
    }

    // 执行查询
    graphdb_result_t* result = NULL;
    rc = graphdb_execute(session, "MATCH (n) RETURN n LIMIT 10", &result);
    if (rc != GRAPHDB_OK) {
        fprintf(stderr, "查询失败: %s\n", graphdb_error_string(rc));
    } else {
        int row_count = graphdb_row_count(result);
        int col_count = graphdb_column_count(result);
        printf("查询结果: %d 行, %d 列\n", row_count, col_count);
        graphdb_result_free(result);
    }

    // 清理
    graphdb_session_close(session);
    graphdb_close(db);

    return 0;
}
```

### 事务使用示例

```c
#include <graphdb.h>
#include <stdio.h>

int main() {
    graphdb_t* db = NULL;
    graphdb_session_t* session = NULL;
    graphdb_txn_t* txn = NULL;

    // 打开数据库和创建会话
    graphdb_open("test.db", &db);
    graphdb_session_create(db, &session);
    graphdb_session_use_space(session, "test_space");

    // 开始事务
    int rc = graphdb_txn_begin(session, &txn);
    if (rc != GRAPHDB_OK) {
        fprintf(stderr, "开始事务失败\n");
        goto cleanup;
    }

    // 创建保存点
    int64_t sp = graphdb_txn_savepoint(txn, "after_create");

    // 执行操作
    graphdb_result_t* result = NULL;
    rc = graphdb_txn_execute(txn, "CREATE TAG user(name string)", &result);
    if (rc == GRAPHDB_OK && result) {
        graphdb_result_free(result);
    }

    rc = graphdb_txn_execute(txn, 
        "INSERT VERTEX user(name) VALUES \"1\":(\"Alice\")", 
        &result);
    if (rc == GRAPHDB_OK && result) {
        graphdb_result_free(result);
    }

    // 如果需要回滚到保存点
    // graphdb_txn_rollback_to_savepoint(txn, sp);

    // 提交事务
    rc = graphdb_txn_commit(txn);
    if (rc != GRAPHDB_OK) {
        fprintf(stderr, "提交事务失败，回滚...\n");
        graphdb_txn_rollback(txn);
    }

cleanup:
    graphdb_session_close(session);
    graphdb_close(db);

    return 0;
}
```

### 预编译语句示例

```c
#include <graphdb.h>
#include <stdio.h>

int main() {
    graphdb_t* db = NULL;
    graphdb_session_t* session = NULL;
    graphdb_stmt_t* stmt = NULL;

    graphdb_open("test.db", &db);
    graphdb_session_create(db, &session);
    graphdb_session_use_space(session, "test_space");

    // 准备语句
    int rc = graphdb_prepare(session, 
        "MATCH (n:User {id: $id}) RETURN n.name", 
        &stmt);
    if (rc != GRAPHDB_OK) {
        fprintf(stderr, "准备语句失败\n");
        goto cleanup;
    }

    // 绑定参数并执行多次
    for (int i = 1; i <= 10; i++) {
        graphdb_reset(stmt);
        graphdb_bind_int(stmt, 1, i);

        graphdb_result_t* result = NULL;
        rc = graphdb_stmt_execute(stmt, &result);
        if (rc == GRAPHDB_OK) {
            // 处理结果
            int row_count = graphdb_row_count(result);
            printf("查询 %d: %d 行\n", i, row_count);
            graphdb_result_free(result);
        }
    }

    // 释放语句
    graphdb_finalize(stmt);

cleanup:
    graphdb_session_close(session);
    graphdb_close(db);

    return 0;
}
```

### 编译和链接

```bash
# 编译
gcc -o myapp myapp.c -I/path/to/graphdb/include -L/path/to/graphdb/lib -lgraphdb

# 运行（确保库文件在库路径中）
LD_LIBRARY_PATH=/path/to/graphdb/lib ./myapp
```
