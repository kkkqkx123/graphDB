# GraphDB 与 SQLite API 对比分析

## 概述

本文档对比 GraphDB 嵌入式 API 与 SQLite C API 的功能差异，分析当前实现的优势和不足，并提出改进建议。

## SQLite C API 核心功能总结

### 1. 数据库连接管理

| 函数 | 功能 | GraphDB 对应 |
|------|------|--------------|
| `sqlite3_open()` | 打开数据库 | `graphdb_open()` |
| `sqlite3_open_v2()` | 带标志打开数据库 | ❌ 缺失 |
| `sqlite3_close()` | 关闭数据库 | `graphdb_close()` |
| `sqlite3_libversion()` | 获取版本 | `graphdb_libversion()` |
| `sqlite3_threadsafe()` | 检查线程安全 | ❌ 缺失 |

### 2. 查询执行

| 函数 | 功能 | GraphDB 对应 |
|------|------|--------------|
| `sqlite3_exec()` | 执行SQL（回调方式） | ❌ 缺失 |
| `sqlite3_prepare_v2()` | 预编译语句 | `graphdb_prepare()` |
| `sqlite3_step()` | 执行并获取一行 | ❌ 缺失 |
| `sqlite3_finalize()` | 释放语句 | `graphdb_finalize()` |
| `sqlite3_reset()` | 重置语句 | `graphdb_reset()` |
| `sqlite3_clear_bindings()` | 清除绑定 | `graphdb_clear_bindings()` |

### 3. 参数绑定

| 函数 | 功能 | GraphDB 对应 |
|------|------|--------------|
| `sqlite3_bind_null()` | 绑定NULL | `graphdb_bind_null()` |
| `sqlite3_bind_int()` | 绑定整数 | `graphdb_bind_int()` |
| `sqlite3_bind_int64()` | 绑定64位整数 | ❌ 缺失（合并到bind_int） |
| `sqlite3_bind_double()` | 绑定浮点 | `graphdb_bind_float()` |
| `sqlite3_bind_text()` | 绑定文本 | `graphdb_bind_string()` |
| `sqlite3_bind_blob()` | 绑定二进制数据 | ❌ 缺失 |
| `sqlite3_bind_parameter_count()` | 获取参数数量 | `graphdb_bind_parameter_count()` |
| `sqlite3_bind_parameter_name()` | 获取参数名称 | `graphdb_bind_parameter_name()` |
| `sqlite3_bind_parameter_index()` | 获取参数索引 | `graphdb_bind_parameter_index()` |

### 4. 结果获取

| 函数 | 功能 | GraphDB 对应 |
|------|------|--------------|
| `sqlite3_column_count()` | 获取列数 | `graphdb_column_count()` |
| `sqlite3_column_name()` | 获取列名 | `graphdb_column_name()` |
| `sqlite3_column_type()` | 获取列类型 | ❌ 缺失 |
| `sqlite3_column_int()` | 获取整数值 | `graphdb_get_int()` |
| `sqlite3_column_int64()` | 获取64位整数 | ❌ 缺失 |
| `sqlite3_column_double()` | 获取浮点值 | ❌ 缺失 |
| `sqlite3_column_text()` | 获取文本值 | `graphdb_get_string()` |
| `sqlite3_column_blob()` | 获取二进制数据 | ❌ 缺失 |
| `sqlite3_column_bytes()` | 获取数据长度 | ❌ 缺失 |

### 5. 事务管理

| 函数 | 功能 | GraphDB 对应 |
|------|------|--------------|
| `sqlite3_exec(db, "BEGIN")` | 开始事务 | `graphdb_txn_begin()` |
| `sqlite3_exec(db, "COMMIT")` | 提交事务 | `graphdb_txn_commit()` |
| `sqlite3_exec(db, "ROLLBACK")` | 回滚事务 | `graphdb_txn_rollback()` |
| `sqlite3_get_autocommit()` | 获取自动提交模式 | `graphdb_session_get_autocommit()` |
| `sqlite3_savepoint()` | 保存点 | `graphdb_txn_savepoint()` |

**注意：** SQLite 通过 SQL 语句管理事务，而 GraphDB 提供专门的 C API 函数。

### 6. 错误处理

| 函数 | 功能 | GraphDB 对应 |
|------|------|--------------|
| `sqlite3_errcode()` | 获取错误码 | `graphdb_errcode()` |
| `sqlite3_extended_errcode()` | 获取扩展错误码 | ❌ 缺失 |
| `sqlite3_errmsg()` | 获取错误消息 | `graphdb_errmsg()` |
| `sqlite3_errstr()` | 错误码转字符串 | `graphdb_error_string()` |
| `sqlite3_error_offset()` | 获取SQL错误位置 | ❌ 缺失 |

### 7. 辅助功能

| 函数 | 功能 | GraphDB 对应 |
|------|------|--------------|
| `sqlite3_changes()` | 获取影响行数 | ❌ 缺失 |
| `sqlite3_total_changes()` | 获取总变更数 | ❌ 缺失 |
| `sqlite3_last_insert_rowid()` | 获取最后插入ID | ❌ 缺失 |
| `sqlite3_busy_timeout()` | 设置忙等待超时 | ❌ 缺失 |
| `sqlite3_busy_handler()` | 设置忙处理回调 | ❌ 缺失 |

### 8. 高级功能（SQLite特有）

| 函数 | 功能 | GraphDB 对应 |
|------|------|--------------|
| `sqlite3_backup_init()` | 初始化备份 | ❌ 缺失 |
| `sqlite3_backup_step()` | 执行备份 | ❌ 缺失 |
| `sqlite3_backup_finish()` | 完成备份 | ❌ 缺失 |
| `sqlite3_create_function()` | 创建自定义函数 | ❌ 缺失 |
| `sqlite3_create_collation()` | 创建自定义排序 | ❌ 缺失 |
| `sqlite3_commit_hook()` | 提交钩子 | ❌ 缺失 |
| `sqlite3_rollback_hook()` | 回滚钩子 | ❌ 缺失 |
| `sqlite3_update_hook()` | 更新钩子 | ❌ 缺失 |
| `sqlite3_progress_handler()` | 进度处理器 | ❌ 缺失 |
| `sqlite3_trace_v2()` | SQL追踪 | ❌ 缺失 |
| `sqlite3_enable_load_extension()` | 启用扩展加载 | ❌ 缺失 |
| `sqlite3_load_extension()` | 加载扩展 | ❌ 缺失 |

## 功能对比矩阵

### 已实现功能 ✅

| 功能类别 | GraphDB | SQLite | 对比说明 |
|----------|---------|--------|----------|
| 数据库打开/关闭 | ✅ | ✅ | 功能对等 |
| 会话管理 | ✅ | ✅ | GraphDB 显式会话，SQLite 隐式连接 |
| 简单查询执行 | ✅ | ✅ | 功能对等 |
| 参数化查询 | ✅ | ✅ | 功能对等 |
| 预编译语句 | ✅ | ✅ | 功能对等 |
| 基本参数绑定 | ✅ | ✅ | 类型支持略少 |
| 事务管理 | ✅ | ✅ | GraphDB 显式 API，SQLite SQL 驱动 |
| 保存点 | ✅ | ✅ | 功能对等 |
| 错误码 | ✅ | ✅ | GraphDB 错误码设计类似 SQLite |
| 错误消息 | ✅ | ✅ | 功能对等 |
| 批量操作 | ✅ | ❌ | GraphDB 特有功能 |

### 缺失功能 ❌

| 功能类别 | 重要性 | 建议 |
|----------|--------|------|
| 数据库备份 API | 高 | 建议实现 |
| 二进制数据(BLOB)支持 | 高 | 建议实现 |
| 忙等待/超时机制 | 高 | 建议实现 |
| 影响行数统计 | 中 | 建议实现 |
| 最后插入ID获取 | 中 | 建议实现 |
| SQL 错误位置 | 中 | 建议实现 |
| 扩展错误码 | 低 | 可选实现 |
| 自定义函数 | 低 | 可选实现 |
| 自定义排序 | 低 | 可选实现 |
| 钩子机制 | 低 | 可选实现 |
| 扩展加载 | 低 | 可选实现 |
| SQL 追踪 | 低 | 可选实现 |

## 详细分析

### 1. 数据库打开模式

**SQLite:**
```c
// 支持多种打开模式
sqlite3_open_v2("db.db", &db,
    SQLITE_OPEN_READONLY |      // 只读
    SQLITE_OPEN_READWRITE |     // 读写
    SQLITE_OPEN_CREATE |        // 不存在则创建
    SQLITE_OPEN_NOMUTEX |       // 无互斥锁
    SQLITE_OPEN_FULLMUTEX |     // 全互斥锁
    SQLITE_OPEN_SHAREDCACHE |   // 共享缓存
    SQLITE_OPEN_PRIVATECACHE |  // 私有缓存
    SQLITE_OPEN_URI,            // URI 文件名
    NULL);
```

**GraphDB 当前:**
```c
// 仅支持基本打开
graphdb_open("db.db", &db);
```

**建议改进:**
添加 `graphdb_open_v2()` 支持更多打开选项。

### 2. 查询执行模式

**SQLite:**
```c
// 方式1: 回调方式执行
sqlite3_exec(db, "SELECT * FROM t", callback, data, &errmsg);

// 方式2: 预编译+步进
sqlite3_prepare_v2(db, sql, -1, &stmt, NULL);
while (sqlite3_step(stmt) == SQLITE_ROW) {
    // 处理每一行
}
sqlite3_finalize(stmt);
```

**GraphDB 当前:**
```c
// 直接执行，返回完整结果集
graphdb_execute(session, sql, &result);
// 处理结果集...
graphdb_result_free(result);
```

**分析:**
- GraphDB 的方式更简单，适合中小结果集
- SQLite 的步进方式更适合大结果集（流式处理）
- 建议保留当前方式，但可考虑添加流式查询 API

### 3. 结果集访问

**SQLite:**
```c
// 按列索引访问
int id = sqlite3_column_int(stmt, 0);
const char* name = sqlite3_column_text(stmt, 1);

// 获取类型信息
int type = sqlite3_column_type(stmt, col);
```

**GraphDB 当前:**
```c
// 按列名访问
int64_t value;
graphdb_get_int(result, row, "column_name", &value);

// 不支持按索引访问
// 不支持类型检查
```

**建议改进:**
1. 添加按列索引访问的 API
2. 添加获取列类型的 API
3. 添加获取值类型的 API

### 4. 错误处理

**SQLite:**
```c
// 丰富的错误信息
int errcode = sqlite3_errcode(db);
int extended = sqlite3_extended_errcode(db);
const char* errmsg = sqlite3_errmsg(db);
int offset = sqlite3_error_offset(db);  // SQL错误位置
```

**GraphDB 当前:**
```c
// 基本错误信息
int rc = graphdb_execute(session, sql, &result);
if (rc != GRAPHDB_OK) {
    char msg[256];
    graphdb_errmsg(msg, sizeof(msg));
}
```

**建议改进:**
1. 添加 SQL 错误位置信息（需要 parser 支持）
2. 添加扩展错误码机制

### 5. 辅助功能

**SQLite:**
```c
// 变更统计
int changes = sqlite3_changes(db);
sqlite3_int64 total = sqlite3_total_changes64(db);
sqlite3_int64 rowid = sqlite3_last_insert_rowid(db);

// 忙处理
sqlite3_busy_timeout(db, 5000);  // 5秒超时
sqlite3_busy_handler(db, callback, data);  // 自定义处理
```

**GraphDB 当前:**
无对应功能。

**建议改进:**
1. 添加变更统计 API
2. 添加忙等待/超时机制（多线程环境下重要）

### 6. 高级功能

SQLite 提供的高级功能：
- 在线备份 API
- 自定义函数/聚合函数
- 自定义排序规则
- 钩子机制（提交/回滚/更新）
- 扩展加载机制
- SQL 追踪

**建议:**
这些功能优先级较低，可在后续版本中逐步添加。

## 改进建议汇总

### 高优先级（建议尽快实现）

1. **添加数据库打开选项**
   ```c
   int graphdb_open_v2(const char* path, graphdb_t** db, int flags);
   ```

2. **添加忙等待机制**
   ```c
   int graphdb_busy_timeout(graphdb_session_t* session, int ms);
   int graphdb_busy_handler(graphdb_session_t* session, 
                            int (*callback)(void*, int), void* data);
   ```

3. **添加变更统计**
   ```c
   int graphdb_changes(graphdb_session_t* session);
   int64_t graphdb_last_insert_id(graphdb_session_t* session);
   ```

4. **增强结果集访问**
   ```c
   // 按索引访问
   int graphdb_get_int_by_index(graphdb_result_t* result, 
                                 int row, int col, int64_t* value);
   
   // 获取类型
   int graphdb_column_type(graphdb_result_t* result, int col);
   ```

5. **添加二进制数据支持**
   ```c
   int graphdb_bind_blob(graphdb_stmt_t* stmt, int index, 
                         const void* data, int len);
   const void* graphdb_get_blob(graphdb_result_t* result, 
                                 int row, const char* col, int* len);
   ```

### 中优先级（建议后续实现）

1. **添加 SQL 错误位置**
   ```c
   int graphdb_error_offset(graphdb_session_t* session);
   ```

2. **添加数据库备份 API**
   ```c
   graphdb_backup_t* graphdb_backup_init(graphdb_t* dest, graphdb_t* src);
   int graphdb_backup_step(graphdb_backup_t* backup, int pages);
   int graphdb_backup_finish(graphdb_backup_t* backup);
   ```

3. **添加扩展错误码**
   ```c
   int graphdb_extended_errcode(graphdb_session_t* session);
   ```

### 低优先级（可选实现）

1. 自定义函数/聚合函数
2. 自定义排序规则
3. 钩子机制
4. 扩展加载
5. SQL 追踪

## 总结

### GraphDB 的优势

1. **图数据模型**: 专为图数据设计，支持顶点、边、路径等图特有类型
2. **批量操作**: 内置高效的批量插入功能
3. **显式会话管理**: 清晰的会话概念，便于多线程使用
4. **保存点支持**: 完整的事务保存点功能
5. **类型安全**: Rust 实现提供内存安全保证

### 需要改进的地方

1. **API 完整性**: 相比 SQLite，缺少一些辅助功能和高级功能
2. **流式查询**: 当前结果集是一次性返回，大结果集可能占用较多内存
3. **忙等待机制**: 多线程环境下需要更好的并发控制
4. **错误信息**: 可以更丰富，包括 SQL 错误位置等
5. **二进制数据**: 缺少 BLOB 类型支持

### 与 SQLite 的设计差异

| 方面 | GraphDB | SQLite |
|------|---------|--------|
| 数据模型 | 图数据 | 关系型 |
| 查询语言 | nGQL | SQL |
| 事务管理 | 显式 C API | SQL 语句 |
| 会话概念 | 显式 | 隐式（连接级别） |
| 结果集处理 | 一次性返回 | 步进式/回调式 |
| 扩展机制 | 尚未实现 | 成熟完善 |

总体而言，GraphDB 的嵌入式 API 设计合理，基本功能完整，但在一些辅助功能和高级特性上还有提升空间。建议优先实现高优先级的改进项，以提供更好的开发体验和功能完整性。
