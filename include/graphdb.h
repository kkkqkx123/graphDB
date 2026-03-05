#ifndef GRAPHDB_H
#define GRAPHDB_H

/* Generated with cbindgen:0.29.2 */

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
#include <stddef.h>

/**
 * God角色的Space ID标记（全局角色，不绑定特定Space）
 */
#define GOD_SPACE_ID -1

#define DEFAULT_MAX_ALLOWED_CONNECTIONS 100

#define INDEX_KEY_SEPARATOR 255

/**
 * 等值查询默认选择性（假设10个不同值）
 */
#define EQUALITY 0.1

/**
 * 范围查询默认选择性（假设选择1/3的数据）
 */
#define RANGE 0.333

/**
 * 小于/大于查询默认选择性
 */
#define COMPARISON 0.333

/**
 * 不等查询默认选择性
 */
#define NOT_EQUAL 0.9

/**
 * IS NULL 查询选择性（通常很少为null）
 */
#define IS_NULL 0.05

/**
 * IS NOT NULL 查询选择性
 */
#define IS_NOT_NULL 0.95

/**
 * IN 查询默认选择性（假设3个值）
 */
#define IN_LIST 0.3

/**
 * EXISTS 查询选择性
 */
#define EXISTS 0.5

/**
 * 布尔AND操作的选择性惩罚
 */
#define AND_CORRELATION 0.9

/**
 * 布尔OR操作的选择性惩罚
 */
#define OR_CORRELATION 0.9

/**
 * 无效ID常量
 */
#define INVALID_ID -1

/**
 * 值类型
 */
typedef enum graphdb_value_type_t {
  /**
   * 空值
   */
  GRAPHDB_NULL = 0,
  /**
   * 布尔值
   */
  GRAPHDB_BOOL = 1,
  /**
   * 整数
   */
  GRAPHDB_INT = 2,
  /**
   * 浮点数
   */
  GRAPHDB_FLOAT = 3,
  /**
   * 字符串
   */
  GRAPHDB_STRING = 4,
  /**
   * 列表
   */
  GRAPHDB_LIST = 5,
  /**
   * 映射
   */
  GRAPHDB_MAP = 6,
  /**
   * 顶点
   */
  GRAPHDB_VERTEX = 7,
  /**
   * 边
   */
  GRAPHDB_EDGE = 8,
  /**
   * 路径
   */
  GRAPHDB_PATH = 9,
} graphdb_value_type_t;

/**
 * 数据库句柄（不透明指针）
 */
typedef struct graphdb_t {

} graphdb_t;

/**
 * 会话句柄（不透明指针）
 */
typedef struct graphdb_session_t {

} graphdb_session_t;

/**
 * 结果集句柄（不透明指针）
 */
typedef struct graphdb_result_t {

} graphdb_result_t;

/**
 * 字符串结构
 */
typedef struct graphdb_string_t {
  /**
   * 字符串数据
   */
  const char *data;
  /**
   * 字符串长度
   */
  uintptr_t len;
} graphdb_string_t;

/**
 * 值数据联合体
 */
typedef union graphdb_value_data_t {
  /**
   * 布尔值
   */
  bool boolean;
  /**
   * 整数
   */
  int64_t integer;
  /**
   * 浮点数
   */
  double floating;
  /**
   * 字符串
   */
  struct graphdb_string_t string;
  /**
   * 指针
   */
  void *ptr;
} graphdb_value_data_t;

/**
 * 值结构
 */
typedef struct graphdb_value_t {
  /**
   * 值类型
   */
  enum graphdb_value_type_t type_;
  /**
   * 值数据
   */
  union graphdb_value_data_t data;
} graphdb_value_t;

/**
 * 预编译语句句柄（不透明指针）
 */
typedef struct graphdb_stmt_t {

} graphdb_stmt_t;

/**
 * 事务句柄（不透明指针）
 */
typedef struct graphdb_txn_t {

} graphdb_txn_t;

/**
 * 批量操作句柄（不透明指针）
 */
typedef struct graphdb_batch_t {

} graphdb_batch_t;

/**
 * 获取最后一个错误消息（线程安全）
 *
 * # 参数
 * - `msg`: 输出缓冲区
 * - `len`: 缓冲区长度
 *
 * # 返回
 * - 实际写入的字符数（不包括 null 终止符）
 */
int32_t graphdb_errmsg(char *msg, uintptr_t len);

/**
 * 获取错误码描述
 *
 * # 参数
 * - `code`: 错误码
 *
 * # 返回
 * - 错误描述字符串（静态生命周期）
 */
const char *graphdb_error_string(int32_t code);

/**
 * 打开数据库
 *
 * # 参数
 * - `path`: 数据库文件路径（UTF-8 编码）
 * - `db`: 输出参数，数据库句柄
 *
 * # 返回
 * - 成功: GRAPHDB_OK
 * - 失败: 错误码
 */
int graphdb_open(const char *path, struct graphdb_t **db);

/**
 * 关闭数据库
 *
 * # 参数
 * - `db`: 数据库句柄
 *
 * # 返回
 * - 成功: GRAPHDB_OK
 * - 失败: 错误码
 */
int graphdb_close(struct graphdb_t *db);

/**
 * 获取错误码
 *
 * # 参数
 * - `db`: 数据库句柄
 *
 * # 返回
 * - 错误码，如果没有错误返回 GRAPHDB_OK
 */
int graphdb_errcode(struct graphdb_t *db);

/**
 * 获取库版本
 *
 * # 返回
 * - 版本字符串
 */
const char *graphdb_libversion(void);

/**
 * 释放字符串（由 GraphDB 分配的字符串）
 *
 * # 参数
 * - `str`: 字符串指针
 */
void graphdb_free_string(char *str);

/**
 * 释放内存（由 GraphDB 分配的内存）
 *
 * # 参数
 * - `ptr`: 内存指针
 */
void graphdb_free(void *ptr);

/**
 * 创建会话
 *
 * # 参数
 * - `db`: 数据库句柄
 * - `session`: 输出参数，会话句柄
 *
 * # 返回
 * - 成功: GRAPHDB_OK
 * - 失败: 错误码
 */
int graphdb_session_create(struct graphdb_t *db, struct graphdb_session_t **session);

/**
 * 关闭会话
 *
 * # 参数
 * - `session`: 会话句柄
 *
 * # 返回
 * - 成功: GRAPHDB_OK
 * - 失败: 错误码
 */
int graphdb_session_close(struct graphdb_session_t *session);

/**
 * 切换图空间
 *
 * # 参数
 * - `session`: 会话句柄
 * - `space_name`: 图空间名称（UTF-8 编码）
 *
 * # 返回
 * - 成功: GRAPHDB_OK
 * - 失败: 错误码
 */
int graphdb_session_use_space(struct graphdb_session_t *session, const char *space_name);

/**
 * 获取当前图空间
 *
 * # 参数
 * - `session`: 会话句柄
 *
 * # 返回
 * - 当前图空间名称（UTF-8 编码），如果没有则返回 NULL
 */
const char *graphdb_session_current_space(struct graphdb_session_t *session);

/**
 * 设置自动提交模式
 *
 * # 参数
 * - `session`: 会话句柄
 * - `autocommit`: 是否自动提交
 *
 * # 返回
 * - 成功: GRAPHDB_OK
 * - 失败: 错误码
 */
int graphdb_session_set_autocommit(struct graphdb_session_t *session, bool autocommit);

/**
 * 获取自动提交模式
 *
 * # 参数
 * - `session`: 会话句柄
 *
 * # 返回
 * - 是否自动提交
 */
bool graphdb_session_get_autocommit(struct graphdb_session_t *session);

/**
 * 执行简单查询
 *
 * # 参数
 * - `session`: 会话句柄
 * - `query`: 查询语句（UTF-8 编码）
 * - `result`: 输出参数，结果集句柄
 *
 * # 返回
 * - 成功: GRAPHDB_OK
 * - 失败: 错误码
 */
int graphdb_execute(struct graphdb_session_t *session,
                    const char *query,
                    struct graphdb_result_t **result);

/**
 * 执行参数化查询
 *
 * # 参数
 * - `session`: 会话句柄
 * - `query`: 查询语句（UTF-8 编码）
 * - `params`: 参数数组
 * - `param_count`: 参数数量
 * - `result`: 输出参数，结果集句柄
 *
 * # 返回
 * - 成功: GRAPHDB_OK
 * - 失败: 错误码
 */
int graphdb_execute_params(struct graphdb_session_t *session,
                           const char *query,
                           const struct graphdb_value_t *params,
                           uintptr_t param_count,
                           struct graphdb_result_t **result);

/**
 * 准备语句
 *
 * # 参数
 * - `session`: 会话句柄
 * - `query`: 查询语句（UTF-8 编码）
 * - `stmt`: 输出参数，语句句柄
 *
 * # 返回
 * - 成功: GRAPHDB_OK
 * - 失败: 错误码
 */
int graphdb_prepare(struct graphdb_session_t *session,
                    const char *query,
                    struct graphdb_stmt_t **stmt);

/**
 * 绑定 NULL 值（按索引）
 *
 * # 参数
 * - `stmt`: 语句句柄
 * - `index`: 参数索引（从 1 开始）
 *
 * # 返回
 * - 成功: GRAPHDB_OK
 * - 失败: 错误码
 */
int graphdb_bind_null(struct graphdb_stmt_t *stmt, int index);

/**
 * 绑定布尔值（按索引）
 *
 * # 参数
 * - `stmt`: 语句句柄
 * - `index`: 参数索引（从 1 开始）
 * - `value`: 布尔值
 *
 * # 返回
 * - 成功: GRAPHDB_OK
 * - 失败: 错误码
 */
int graphdb_bind_bool(struct graphdb_stmt_t *stmt, int index, bool value);

/**
 * 绑定整数值（按索引）
 *
 * # 参数
 * - `stmt`: 语句句柄
 * - `index`: 参数索引（从 1 开始）
 * - `value`: 整数值
 *
 * # 返回
 * - 成功: GRAPHDB_OK
 * - 失败: 错误码
 */
int graphdb_bind_int(struct graphdb_stmt_t *stmt, int index, int64_t value);

/**
 * 绑定浮点值（按索引）
 *
 * # 参数
 * - `stmt`: 语句句柄
 * - `index`: 参数索引（从 1 开始）
 * - `value`: 浮点值
 *
 * # 返回
 * - 成功: GRAPHDB_OK
 * - 失败: 错误码
 */
int graphdb_bind_float(struct graphdb_stmt_t *stmt, int index, double value);

/**
 * 绑定字符串值（按索引）
 *
 * # 参数
 * - `stmt`: 语句句柄
 * - `index`: 参数索引（从 1 开始）
 * - `value`: 字符串值（UTF-8 编码）
 * - `len`: 字符串长度（-1 表示自动计算）
 *
 * # 返回
 * - 成功: GRAPHDB_OK
 * - 失败: 错误码
 */
int graphdb_bind_string(struct graphdb_stmt_t *stmt, int index, const char *value, int len);

/**
 * 绑定参数（按名称）
 *
 * # 参数
 * - `stmt`: 语句句柄
 * - `name`: 参数名称（UTF-8 编码）
 * - `value`: 值
 *
 * # 返回
 * - 成功: GRAPHDB_OK
 * - 失败: 错误码
 */
int graphdb_bind_by_name(struct graphdb_stmt_t *stmt,
                         const char *name,
                         struct graphdb_value_t value);

/**
 * 重置语句
 *
 * 清除所有绑定的参数，使语句可以重新执行
 *
 * # 参数
 * - `stmt`: 语句句柄
 *
 * # 返回
 * - 成功: GRAPHDB_OK
 * - 失败: 错误码
 */
int graphdb_reset(struct graphdb_stmt_t *stmt);

/**
 * 清除绑定
 *
 * 清除所有绑定的参数
 *
 * # 参数
 * - `stmt`: 语句句柄
 *
 * # 返回
 * - 成功: GRAPHDB_OK
 * - 失败: 错误码
 */
int graphdb_clear_bindings(struct graphdb_stmt_t *stmt);

/**
 * 释放语句
 *
 * # 参数
 * - `stmt`: 语句句柄
 *
 * # 返回
 * - 成功: GRAPHDB_OK
 * - 失败: 错误码
 */
int graphdb_finalize(struct graphdb_stmt_t *stmt);

/**
 * 获取参数索引
 *
 * # 参数
 * - `stmt`: 语句句柄
 * - `name`: 参数名称（UTF-8 编码）
 *
 * # 返回
 * - 参数索引（从 1 开始），未找到返回 0
 */
int graphdb_bind_parameter_index(struct graphdb_stmt_t *stmt, const char *name);

/**
 * 获取参数名称
 *
 * # 参数
 * - `stmt`: 语句句柄
 * - `index`: 参数索引（从 1 开始）
 *
 * # 返回
 * - 参数名称（UTF-8 编码），未找到返回 NULL
 */
const char *graphdb_bind_parameter_name(struct graphdb_stmt_t *stmt, int index);

/**
 * 获取参数数量
 *
 * # 参数
 * - `stmt`: 语句句柄
 *
 * # 返回
 * - 参数数量
 */
int graphdb_bind_parameter_count(struct graphdb_stmt_t *stmt);

/**
 * 开始事务
 *
 * # 参数
 * - `session`: 会话句柄
 * - `txn`: 输出参数，事务句柄
 *
 * # 返回
 * - 成功: GRAPHDB_OK
 * - 失败: 错误码
 */
int graphdb_txn_begin(struct graphdb_session_t *session, struct graphdb_txn_t **txn);

/**
 * 开始只读事务
 *
 * # 参数
 * - `session`: 会话句柄
 * - `txn`: 输出参数，事务句柄
 *
 * # 返回
 * - 成功: GRAPHDB_OK
 * - 失败: 错误码
 */
int graphdb_txn_begin_readonly(struct graphdb_session_t *session, struct graphdb_txn_t **txn);

/**
 * 在事务中执行查询
 *
 * # 参数
 * - `txn`: 事务句柄
 * - `query`: 查询语句（UTF-8 编码）
 * - `result`: 输出参数，结果集句柄
 *
 * # 返回
 * - 成功: GRAPHDB_OK
 * - 失败: 错误码
 */
int graphdb_txn_execute(struct graphdb_txn_t *txn,
                        const char *query,
                        struct graphdb_result_t **result);

/**
 * 提交事务
 *
 * # 参数
 * - `txn`: 事务句柄
 *
 * # 返回
 * - 成功: GRAPHDB_OK
 * - 失败: 错误码
 */
int graphdb_txn_commit(struct graphdb_txn_t *txn);

/**
 * 回滚事务
 *
 * # 参数
 * - `txn`: 事务句柄
 *
 * # 返回
 * - 成功: GRAPHDB_OK
 * - 失败: 错误码
 */
int graphdb_txn_rollback(struct graphdb_txn_t *txn);

/**
 * 创建保存点
 *
 * # 参数
 * - `txn`: 事务句柄
 * - `name`: 保存点名称（UTF-8 编码）
 *
 * # 返回
 * - 成功: 保存点 ID
 * - 失败: -1
 */
int64_t graphdb_txn_savepoint(struct graphdb_txn_t *txn, const char *name);

/**
 * 释放保存点
 *
 * # 参数
 * - `txn`: 事务句柄
 * - `savepoint_id`: 保存点 ID
 *
 * # 返回
 * - 成功: GRAPHDB_OK
 * - 失败: 错误码
 */
int graphdb_txn_release_savepoint(struct graphdb_txn_t *txn, int64_t savepoint_id);

/**
 * 回滚到保存点
 *
 * # 参数
 * - `txn`: 事务句柄
 * - `savepoint_id`: 保存点 ID
 *
 * # 返回
 * - 成功: GRAPHDB_OK
 * - 失败: 错误码
 */
int graphdb_txn_rollback_to_savepoint(struct graphdb_txn_t *txn, int64_t savepoint_id);

/**
 * 释放事务句柄
 *
 * # 参数
 * - `txn`: 事务句柄
 *
 * # 返回
 * - 成功: GRAPHDB_OK
 * - 失败: 错误码
 */
int graphdb_txn_free(struct graphdb_txn_t *txn);

/**
 * 创建批量插入器
 *
 * # 参数
 * - `session`: 会话句柄
 * - `batch_size`: 批次大小
 * - `batch`: 输出参数，批量操作句柄
 *
 * # 返回
 * - 成功: GRAPHDB_OK
 * - 失败: 错误码
 */
int graphdb_batch_inserter_create(struct graphdb_session_t *session,
                                  int batch_size,
                                  struct graphdb_batch_t **batch);

/**
 * 添加顶点
 *
 * # 参数
 * - `batch`: 批量操作句柄
 * - `vid`: 顶点 ID
 * - `tag_name`: 标签名称（UTF-8 编码）
 * - `properties`: 属性数组
 * - `prop_count`: 属性数量
 *
 * # 返回
 * - 成功: GRAPHDB_OK
 * - 失败: 错误码
 */
int graphdb_batch_add_vertex(struct graphdb_batch_t *batch,
                             int64_t vid,
                             const char *tag_name,
                             const struct graphdb_value_t *properties,
                             uintptr_t prop_count);

/**
 * 添加边
 *
 * # 参数
 * - `batch`: 批量操作句柄
 * - `src_vid`: 源顶点 ID
 * - `dst_vid`: 目标顶点 ID
 * - `edge_type`: 边类型名称（UTF-8 编码）
 * - `rank`: 排名
 * - `properties`: 属性数组
 * - `prop_count`: 属性数量
 *
 * # 返回
 * - 成功: GRAPHDB_OK
 * - 失败: 错误码
 */
int graphdb_batch_add_edge(struct graphdb_batch_t *batch,
                           int64_t src_vid,
                           int64_t dst_vid,
                           const char *edge_type,
                           int64_t rank,
                           const struct graphdb_value_t *properties,
                           uintptr_t prop_count);

/**
 * 执行批量插入
 *
 * # 参数
 * - `batch`: 批量操作句柄
 *
 * # 返回
 * - 成功: GRAPHDB_OK
 * - 失败: 错误码
 */
int graphdb_batch_flush(struct graphdb_batch_t *batch);

/**
 * 获取缓冲的顶点数量
 *
 * # 参数
 * - `batch`: 批量操作句柄
 *
 * # 返回
 * - 缓冲的顶点数量
 */
int graphdb_batch_buffered_vertices(struct graphdb_batch_t *batch);

/**
 * 获取缓冲的边数量
 *
 * # 参数
 * - `batch`: 批量操作句柄
 *
 * # 返回
 * - 缓冲的边数量
 */
int graphdb_batch_buffered_edges(struct graphdb_batch_t *batch);

/**
 * 释放批量操作句柄
 *
 * # 参数
 * - `batch`: 批量操作句柄
 *
 * # 返回
 * - 成功: GRAPHDB_OK
 * - 失败: 错误码
 */
int graphdb_batch_free(struct graphdb_batch_t *batch);

/**
 * 释放结果集
 *
 * # 参数
 * - `result`: 结果集句柄
 *
 * # 返回
 * - 成功: GRAPHDB_OK
 * - 失败: 错误码
 */
int graphdb_result_free(struct graphdb_result_t *result);

/**
 * 获取结果集列数
 *
 * # 参数
 * - `result`: 结果集句柄
 *
 * # 返回
 * - 列数，错误返回 -1
 */
int graphdb_column_count(struct graphdb_result_t *result);

/**
 * 获取结果集行数
 *
 * # 参数
 * - `result`: 结果集句柄
 *
 * # 返回
 * - 行数，错误返回 -1
 */
int graphdb_row_count(struct graphdb_result_t *result);

/**
 * 获取列名
 *
 * # 参数
 * - `result`: 结果集句柄
 * - `index`: 列索引（从 0 开始）
 *
 * # 返回
 * - 列名（UTF-8 编码），错误返回 NULL
 */
const char *graphdb_column_name(struct graphdb_result_t *result, int index);

/**
 * 获取整数值
 *
 * # 参数
 * - `result`: 结果集句柄
 * - `row`: 行索引（从 0 开始）
 * - `col`: 列名（UTF-8 编码）
 * - `value`: 输出参数，整数值
 *
 * # 返回
 * - 成功: GRAPHDB_OK
 * - 失败: 错误码
 */
int graphdb_get_int(struct graphdb_result_t *result, int row, const char *col, int64_t *value);

/**
 * 获取字符串值
 *
 * # 参数
 * - `result`: 结果集句柄
 * - `row`: 行索引（从 0 开始）
 * - `col`: 列名（UTF-8 编码）
 * - `len`: 输出参数，字符串长度
 *
 * # 返回
 * - 字符串值（UTF-8 编码），错误返回 NULL
 */
const char *graphdb_get_string(struct graphdb_result_t *result, int row, const char *col, int *len);

#endif  /* GRAPHDB_H */
