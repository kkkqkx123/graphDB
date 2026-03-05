# GraphDB C API API 参考文档

## 数据库管理

### graphdb_open

打开数据库文件。

```c
int graphdb_open(const char *path, graphdb_t **db);
```

**参数**:
- `path`: 数据库文件路径
- `db`: 输出参数，返回数据库句柄

**返回值**:
- `GRAPHDB_OK`: 成功
- 其他错误码: 失败

**示例**:
```c
graphdb_t *db = NULL;
int rc = graphdb_open("test.db", &db);
if (rc != GRAPHDB_OK) {
    fprintf(stderr, "错误: %s\n", graphdb_errmsg(db));
}
```

### graphdb_open_memory

打开内存数据库。

```c
int graphdb_open_memory(graphdb_t **db);
```

**参数**:
- `db`: 输出参数，返回数据库句柄

**返回值**:
- `GRAPHDB_OK`: 成功
- 其他错误码: 失败

### graphdb_close

关闭数据库。

```c
int graphdb_close(graphdb_t *db);
```

**参数**:
- `db`: 数据库句柄

**返回值**:
- `GRAPHDB_OK`: 成功
- 其他错误码: 失败

### graphdb_errcode

获取最后的错误码。

```c
int graphdb_errcode(graphdb_t *db);
```

**参数**:
- `db`: 数据库句柄

**返回值**:
- 错误码

### graphdb_errmsg

获取最后的错误信息。

```c
const char *graphdb_errmsg(graphdb_t *db);
```

**参数**:
- `db`: 数据库句柄

**返回值**:
- 错误信息字符串

## 错误码

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
| GRAPHDB_INTERRUPT | 9 | 操作被中断 |
| GRAPHDB_IOERR | 10 | IO 错误 |
| GRAPHDB_CORRUPT | 11 | 数据损坏 |
| GRAPHDB_NOTFOUND | 12 | 未找到 |
| GRAPHDB_FULL | 13 | 磁盘已满 |
| GRAPHDB_CANTOPEN | 14 | 无法打开 |
| GRAPHDB_PROTOCOL | 15 | 协议错误 |
| GRAPHDB_SCHEMA | 16 | 模式错误 |
| GRAPHDB_TOOBIG | 17 | 数据过大 |
| GRAPHDB_CONSTRAINT | 18 | 约束违反 |
| GRAPHDB_MISMATCH | 19 | 类型不匹配 |
| GRAPHDB_MISUSE | 20 | 误用 |
| GRAPHDB_RANGE | 21 | 超出范围 |

## 类型定义

### graphdb_t

数据库句柄（不透明指针）。

### graphdb_session_t

会话句柄（不透明指针）。

### graphdb_stmt_t

预编译语句句柄（不透明指针）。

### graphdb_txn_t

事务句柄（不透明指针）。

### graphdb_result_t

结果集句柄（不透明指针）。

### graphdb_value_type_t

值类型枚举。

| 类型 | 值 | 描述 |
|------|-----|------|
| GRAPHDB_NULL | 0 | 空值 |
| GRAPHDB_BOOL | 1 | 布尔值 |
| GRAPHDB_INT | 2 | 整数 |
| GRAPHDB_FLOAT | 3 | 浮点数 |
| GRAPHDB_STRING | 4 | 字符串 |
| GRAPHDB_LIST | 5 | 列表 |
| GRAPHDB_MAP | 6 | 映射 |
| GRAPHDB_VERTEX | 7 | 顶点 |
| GRAPHDB_EDGE | 8 | 边 |
| GRAPHDB_PATH | 9 | 路径 |
