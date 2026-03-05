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
 * 打开内存数据库
 *
 * # 参数
 * - `db`: 输出参数，数据库句柄
 *
 * # 返回
 * - 成功: GRAPHDB_OK
 * - 失败: 错误码
 */
int graphdb_open_memory(struct graphdb_t **db);

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
