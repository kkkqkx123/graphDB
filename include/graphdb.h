#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * God角色的Space ID标记（全局角色，不绑定特定Space）
 */
#define GOD_SPACE_ID -1

#define DEFAULT_MAX_ALLOWED_CONNECTIONS 100

/**
 * 数据库打开标志
 */
#define GRAPHDB_OPEN_READONLY 1

#define GRAPHDB_OPEN_READWRITE 2

#define GRAPHDB_OPEN_CREATE 4

#define GRAPHDB_OPEN_NOMUTEX 32768

#define GRAPHDB_OPEN_FULLMUTEX 65536

#define GRAPHDB_OPEN_SHAREDCACHE 131072

#define GRAPHDB_OPEN_PRIVATECACHE 262144

/**
 * 钩子类型常量
 */
#define GRAPHDB_HOOK_INSERT 1

#define GRAPHDB_HOOK_UPDATE 2

#define GRAPHDB_HOOK_DELETE 3

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
 * 索引键类型标记
 */
#define KEY_TYPE_VERTEX_REVERSE 1

#define KEY_TYPE_EDGE_REVERSE 2

#define KEY_TYPE_VERTEX_FORWARD 3

#define KEY_TYPE_EDGE_FORWARD 4

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
  /**
   * 二进制数据
   */
  GRAPHDB_BLOB = 10,
} graphdb_value_type_t;

/**
 * C 函数上下文结构（不透明指针）
 */
typedef struct CFunctionContext CFunctionContext;

/**
 * 会话句柄（不透明指针）
 */
typedef struct graphdb_session_t {

} graphdb_session_t;

/**
 * 批量操作句柄（不透明指针）
 */
typedef struct graphdb_batch_t {

} graphdb_batch_t;

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
 * 二进制数据结构
 */
typedef struct graphdb_blob_t {
  /**
   * 数据指针
   */
  const uint8_t *data;
  /**
   * 数据长度
   */
  uintptr_t len;
} graphdb_blob_t;

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
   * 二进制数据
   */
  struct graphdb_blob_t blob;
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
 * 数据库句柄（不透明指针）
 */
typedef struct graphdb_t {

} graphdb_t;

/**
 * 函数执行上下文（不透明指针）
 */
typedef struct graphdb_context_t {
  /**
   * 内部上下文
   */
  struct CFunctionContext inner;
} graphdb_context_t;

/**
 * 标量函数回调类型
 */
typedef void (*graphdb_scalar_function_callback)(struct graphdb_context_t *context,
                                                 int argc,
                                                 struct graphdb_value_t *argv);

/**
 * 函数析构回调类型
 */
typedef void (*graphdb_function_destroy_callback)(void *user_data);

/**
 * 聚合函数步骤回调类型
 */
typedef void (*graphdb_aggregate_step_callback)(struct graphdb_context_t *context,
                                                int argc,
                                                struct graphdb_value_t *argv);

/**
 * 聚合函数最终回调类型
 */
typedef void (*graphdb_aggregate_final_callback)(struct graphdb_context_t *context);

/**
 * 结果集句柄（不透明指针）
 */
typedef struct graphdb_result_t {

} graphdb_result_t;

/**
 * SQL 追踪回调类型
 */
typedef void (*graphdb_trace_callback)(const char *sql, void *user_data);

/**
 * 钩子回调类型
 */
typedef int (*graphdb_commit_hook_callback)(void *user_data);

typedef void (*graphdb_rollback_hook_callback)(void *user_data);

typedef void (*graphdb_update_hook_callback)(void *user_data,
                                             int operation,
                                             const char *database,
                                             const char *table,
                                             int64_t rowid);

/**
 * 事务句柄（不透明指针）
 */
typedef struct graphdb_txn_t {

} graphdb_txn_t;

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
 *
 * # Safety
 * - `session` must be a valid session handle created by `graphdb_session_create`
 * - `batch_size` must be a positive integer (if <= 0, defaults to 100)
 * - `batch` must be a valid pointer to store the batch handle
 * - The created batch handle holds a session pointer but does not own the session
 * - The caller must ensure the session is not closed before the batch handle is freed
 * - The caller is responsible for freeing the batch handle using `graphdb_batch_inserter_free` when done
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
 *
 * # Safety
 * - `batch` 必须是通过 `graphdb_batch_inserter_create` 创建的有效批量操作句柄
 * - `tag_name` 必须是指向以 null 结尾的 UTF-8 字符串的有效指针
 * - 如果 `properties` 不为 null,则必须指向至少 `prop_count` 个有效的 `graphdb_value_t` 元素
 * - 调用者必须确保在调用此函数时,关联的会话仍然有效
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
 *
 * # Safety
 * - `batch` 必须是通过 `graphdb_batch_inserter_create` 创建的有效批量操作句柄
 * - `edge_type` 必须是指向以 null 结尾的 UTF-8 字符串的有效指针
 * - 如果 `properties` 不为 null,则必须指向至少 `prop_count` 个有效的 `graphdb_value_t` 元素
 * - 调用者必须确保在调用此函数时,关联的会话仍然有效
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
 *
 * # Safety
 * - `batch` 必须是通过 `graphdb_batch_inserter_create` 创建的有效批量操作句柄
 * - 调用者必须确保在调用此函数时,关联的会话仍然有效
 * - 此函数会触发实际的数据库写入操作,可能涉及 I/O 操作
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
 *
 * # Safety
 * - `batch` 必须是通过 `graphdb_batch_inserter_create` 创建的有效批量操作句柄
 * - 调用者必须确保在调用此函数时,关联的会话仍然有效
 */
int graphdb_batch_buffered_vertices(struct graphdb_batch_t *batch);

/**
 * 获取缓冲的边数量
 *
 * # Arguments
 * - `batch`: Batch operation handle
 *
 * # Returns
 * - Number of buffered edges
 *
 * # Safety
 * - `batch` must be a valid batch handle created by `graphdb_batch_inserter_create`
 * - Caller must ensure the associated session is still valid when calling this function
 */
int graphdb_batch_buffered_edges(struct graphdb_batch_t *batch);

/**
 * 释放批量操作句柄
 *
 * # Arguments
 * - `batch`: Batch operation handle
 *
 * # Returns
 * - Success: GRAPHDB_OK
 * - Failure: Error code
 *
 * # Safety
 * - `batch` must be a valid batch handle created by `graphdb_batch_inserter_create`
 * - After calling this function, the batch handle becomes invalid and must not be used
 * - This function does not close the associated session; the caller must close the session separately
 */
int graphdb_batch_free(struct graphdb_batch_t *batch);

/**
 * 打开数据库
 *
 * # Arguments
 * - `path`: Database file path (UTF-8 encoded)
 * - `db`: Output parameter, database handle
 *
 * # Returns
 * - Success: GRAPHDB_OK
 * - Failure: Error code
 *
 * # Safety
 * - `path` must be a valid pointer to a null-terminated UTF-8 string
 * - `db` must be a valid pointer to store the database handle
 * - The caller is responsible for closing the database using `graphdb_close` when done
 * - The database handle must not be used after closing
 */
int graphdb_open(const char *path, struct graphdb_t **db);

/**
 * 使用标志打开数据库
 *
 * # Arguments
 * - `path`: Database file path (UTF-8 encoded)
 * - `db`: Output parameter, database handle
 * - `flags`: Open flags
 * - `vfs`: VFS name (reserved parameter, currently unused, can be NULL)
 *
 * # Returns
 * - Success: GRAPHDB_OK
 * - Failure: Error code
 *
 * # Flags
 * - GRAPHDB_OPEN_READONLY: Read-only mode
 * - GRAPHDB_OPEN_READWRITE: Read-write mode
 * - GRAPHDB_OPEN_CREATE: Create database if it doesn't exist
 *
 * # Safety
 * - `path` must be a valid pointer to a null-terminated UTF-8 string
 * - `db` must be a valid pointer to store the database handle
 * - The caller is responsible for closing the database using `graphdb_close` when done
 * - The database handle must not be used after closing
 */
int graphdb_open_v2(const char *path, struct graphdb_t **db, int flags, const char *_vfs);

/**
 * 关闭数据库
 *
 * # Arguments
 * - `db`: Database handle
 *
 * # Returns
 * - Success: GRAPHDB_OK
 * - Failure: Error code
 *
 * # Safety
 * - `db` must be a valid database handle created by `graphdb_open` or `graphdb_open_v2`
 * - After calling this function, the database handle becomes invalid and must not be used
 * - All sessions associated with this database must be closed before calling this function
 */
int graphdb_close(struct graphdb_t *db);

/**
 * 获取错误码
 *
 * # Arguments
 * - `db`: Database handle
 *
 * # Returns
 * - Error code, returns GRAPHDB_OK if no error
 *
 * # Safety
 * - `db` must be a valid database handle created by `graphdb_open` or `graphdb_open_v2`
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
 * # Arguments
 * - `str`: String pointer
 *
 * # Safety
 * - `str` must be a valid pointer to a string allocated by GraphDB
 * - After calling this function, the pointer becomes invalid and must not be used
 * - This function should only be called on strings that were allocated by GraphDB C API functions
 */
void graphdb_free_string(char *str);

/**
 * 释放内存（由 GraphDB 分配的内存）
 *
 * # Arguments
 * - `ptr`: Memory pointer
 *
 * # Safety
 * - `ptr` must be a valid pointer to memory allocated by GraphDB
 * - After calling this function, the pointer becomes invalid and must not be used
 * - This function should only be called on memory that was allocated by GraphDB C API functions
 */
void graphdb_free(void *ptr);

/**
 * 获取最后一个错误消息（线程安全）
 *
 * # Arguments
 * - `msg`: Output buffer
 * - `len`: Buffer length
 *
 * # Returns
 * - Number of characters actually written (excluding null terminator)
 *
 * # Safety
 * - `msg` must be a valid pointer to a buffer with at least `len` bytes
 * - The buffer must be large enough to hold the error message including null terminator
 * - If the message is longer than `len - 1`, it will be truncated
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
 * 获取错误码对应的字符串描述（类似 SQLite 的 sqlite3_errstr）
 *
 * # 参数
 * - `code`: 错误码
 *
 * # 返回
 * - 错误描述字符串（静态生命周期，不需要释放）
 */
const char *graphdb_errstr(int32_t code);

/**
 * 获取最后的错误消息
 *
 * # 返回
 * - 错误消息字符串指针（线程局部存储，不需要释放）
 */
const char *graphdb_get_last_error_message(void);

/**
 * 获取 SQL 错误位置（字符偏移量）
 *
 * # 参数
 * - `session`: 会话句柄
 *
 * # 返回
 * - 错误位置的字符偏移量，如果没有错误或无效会话返回 -1
 */
int graphdb_error_offset(struct graphdb_session_t *session);

/**
 * 获取扩展错误码
 *
 * # 参数
 * - `session`: 会话句柄
 *
 * # 返回
 * - 扩展错误码，如果没有错误或无效会话返回 0 (GRAPHDB_EXTENDED_NONE)
 */
int graphdb_extended_errcode(struct graphdb_session_t *session);

/**
 * 创建自定义标量函数
 *
 * # Arguments
 * - `session`: Session handle
 * - `name`: Function name
 * - `argc`: Number of arguments, -1 for variable arguments
 * - `user_data`: User data pointer
 * - `x_func`: Scalar function callback
 * - `x_destroy`: Destructor callback, can be NULL
 *
 * # Returns
 * - Success: GRAPHDB_OK
 * - Failure: Error code
 *
 * # Example
 * ```c
 * extern void my_function(graphdb_context_t* ctx, int argc, graphdb_value_t* argv) {
 *     // Implement function logic
 * }
 *
 * graphdb_create_function(session, "my_func", 2, NULL, my_function, NULL);
 * ```
 *
 * # Safety
 * - `session` must be a valid session handle created by `graphdb_session_create`
 * - `name` must be a valid pointer to a null-terminated UTF-8 string
 * - `x_func` must be a valid function pointer
 * - `user_data` is passed to the callback and must remain valid for the lifetime of the function
 */
int graphdb_create_function(struct graphdb_session_t *session,
                            const char *name,
                            int argc,
                            void *user_data,
                            graphdb_scalar_function_callback x_func,
                            graphdb_function_destroy_callback _x_destroy);

/**
 * 创建自定义聚合函数
 *
 * # Arguments
 * - `session`: Session handle
 * - `name`: Function name
 * - `argc`: Number of arguments, -1 for variable arguments
 * - `user_data`: User data pointer
 * - `x_step`: Aggregate step callback
 * - `x_final`: Aggregate final callback
 * - `x_destroy`: Destructor callback, can be NULL
 *
 * # Returns
 * - Success: GRAPHDB_OK
 * - Failure: Error code
 *
 * # Safety
 * - `session` must be a valid session handle created by `graphdb_session_create`
 * - `name` must be a valid pointer to a null-terminated UTF-8 string
 * - `x_step` and `x_final` must be valid function pointers
 * - `user_data` is passed to the callbacks and must remain valid for the lifetime of the function
 */
int graphdb_create_aggregate(struct graphdb_session_t *session,
                             const char *name,
                             int argc,
                             void *user_data,
                             graphdb_aggregate_step_callback x_step,
                             graphdb_aggregate_final_callback x_final,
                             graphdb_function_destroy_callback _x_destroy);

/**
 * 删除自定义函数
 *
 * # Arguments
 * - `session`: Session handle
 * - `name`: Function name
 *
 * # Returns
 * - Success: GRAPHDB_OK
 * - Failure: Error code
 *
 * # Safety
 * - `session` must be a valid session handle created by `graphdb_session_create`
 * - `name` must be a valid pointer to a null-terminated UTF-8 string
 */
int graphdb_delete_function(struct graphdb_session_t *session, const char *name);

/**
 * 设置函数返回值
 *
 * # Arguments
 * - `context`: Function execution context
 * - `value`: Return value
 *
 * # Description
 * Call this function in the scalar function or aggregate function's xFinal callback to set the return value
 *
 * # Safety
 * - `context` must be a valid function context pointer passed to the callback
 * - `value` must be a valid pointer to a value structure, or NULL to set a null result
 * - This function should only be called from within a registered function callback
 */
int graphdb_context_set_result(struct graphdb_context_t *context,
                               const struct graphdb_value_t *value);

/**
 * 获取函数返回值的类型
 *
 * # Arguments
 * - `context`: Function execution context
 *
 * # Returns
 * - Value type
 *
 * # Safety
 * - `context` must be a valid function context pointer passed to the callback
 * - This function should only be called from within a registered function callback
 */
enum graphdb_value_type_t graphdb_context_result_type(struct graphdb_context_t *context);

/**
 * 设置错误消息
 *
 * # Arguments
 * - `context`: Function execution context
 * - `error_msg`: Error message
 *
 * # Description
 * Call this function to set an error message when the function execution fails
 *
 * # Safety
 * - `context` must be a valid function context pointer passed to the callback
 * - `error_msg` must be a valid pointer to a null-terminated UTF-8 string
 * - This function should only be called from within a registered function callback
 */
int graphdb_context_set_error(struct graphdb_context_t *context, const char *error_msg);

/**
 * 从上下文获取参数值（辅助函数）
 *
 * # Arguments
 * - `context`: Function execution context
 * - `index`: Argument index
 *
 * # Returns
 * - Argument value pointer, returns NULL if index is out of bounds
 *
 * # Safety
 * - `context` must be a valid function context pointer passed to the callback
 * - `index` must be a valid argument index (0 <= index < argc)
 * - The returned pointer is only valid for the duration of the callback
 * - This function should only be called from within a registered function callback
 */
const struct graphdb_value_t *graphdb_context_get_arg(struct graphdb_context_t *_context,
                                                      int _index);

/**
 * 获取参数数量
 *
 * # Arguments
 * - `context`: Function execution context
 *
 * # Returns
 * - Number of arguments
 *
 * # Safety
 * - `context` must be a valid function context pointer passed to the callback
 * - This function should only be called from within a registered function callback
 */
int graphdb_context_arg_count(struct graphdb_context_t *_context);

/**
 * 执行简单查询
 *
 * # Arguments
 * - `session`: Session handle
 * - `query`: Query statement (UTF-8 encoded)
 * - `result`: Output parameter, result set handle
 *
 * # Returns
 * - Success: GRAPHDB_OK
 * - Failure: Error code
 *
 * # Safety
 * - `session` must be a valid session handle created by `graphdb_session_create`
 * - `query` must be a valid pointer to a null-terminated UTF-8 string
 * - `result` must be a valid pointer to store the result handle
 * - The caller is responsible for freeing the result handle using `graphdb_result_free` when done
 */
int graphdb_execute(struct graphdb_session_t *session,
                    const char *query,
                    struct graphdb_result_t **result);

/**
 * 执行参数化查询
 *
 * # Arguments
 * - `session`: Session handle
 * - `query`: Query statement (UTF-8 encoded)
 * - `params`: Parameter array
 * - `param_count`: Number of parameters
 * - `result`: Output parameter, result set handle
 *
 * # Returns
 * - Success: GRAPHDB_OK
 * - Failure: Error code
 *
 * # Safety
 * - `session` must be a valid session handle created by `graphdb_session_create`
 * - `query` must be a valid pointer to a null-terminated UTF-8 string
 * - `result` must be a valid pointer to store the result handle
 * - If `params` is not NULL, it must point to at least `param_count` valid `graphdb_value_t` elements
 * - The caller is responsible for freeing the result handle using `graphdb_result_free` when done
 */
int graphdb_execute_params(struct graphdb_session_t *session,
                           const char *query,
                           const struct graphdb_value_t *params,
                           uintptr_t param_count,
                           struct graphdb_result_t **result);

/**
 * 释放结果集
 *
 * # Arguments
 * - `result`: Result set handle
 *
 * # Returns
 * - Success: GRAPHDB_OK
 * - Failure: Error code
 *
 * # Safety
 * - `result` must be a valid result handle created by `graphdb_execute` or `graphdb_execute_params`
 * - After calling this function, the result handle becomes invalid and must not be used
 * - Any string pointers obtained from this result set become invalid after this call
 */
int graphdb_result_free(struct graphdb_result_t *result);

/**
 * 获取结果集列数
 *
 * # Arguments
 * - `result`: Result set handle
 *
 * # Returns
 * - Number of columns, returns -1 on error
 *
 * # Safety
 * - `result` must be a valid result handle created by `graphdb_execute` or `graphdb_execute_params`
 */
int graphdb_column_count(struct graphdb_result_t *result);

/**
 * 获取结果集行数
 *
 * # Arguments
 * - `result`: Result set handle
 *
 * # Returns
 * - Number of rows, returns -1 on error
 *
 * # Safety
 * - `result` must be a valid result handle created by `graphdb_execute` or `graphdb_execute_params`
 */
int graphdb_row_count(struct graphdb_result_t *result);

/**
 * 获取列名
 *
 * # Arguments
 * - `result`: Result set handle
 * - `index`: Column index (starting from 0)
 *
 * # Returns
 * - Column name (UTF-8 encoded), returns NULL on error
 *
 * # Memory Management
 * The returned string is dynamically allocated and must be freed by the caller using `graphdb_free_string`
 * to avoid memory leaks.
 *
 * # Safety
 * - `result` must be a valid result handle created by `graphdb_execute` or `graphdb_execute_params`
 * - `index` must be a valid column index (0 <= index < column count)
 * - The returned pointer must be freed by the caller to avoid memory leaks
 */
char *graphdb_column_name(struct graphdb_result_t *result,
                          int index);

/**
 * 获取整数值
 *
 * # Arguments
 * - `result`: Result set handle
 * - `row`: Row index (starting from 0)
 * - `col`: Column name (UTF-8 encoded)
 * - `value`: Output parameter, integer value
 *
 * # Returns
 * - Success: GRAPHDB_OK
 * - Failure: Error code
 *
 * # Safety
 * - `result` must be a valid result handle created by `graphdb_execute` or `graphdb_execute_params`
 * - `col` must be a valid pointer to a null-terminated UTF-8 string
 * - `value` must be a valid pointer to store the result
 * - `row` must be a valid row index (0 <= row < row count)
 */
int graphdb_get_int(struct graphdb_result_t *result, int row, const char *col, int64_t *value);

/**
 * 获取字符串值
 *
 * # Arguments
 * - `result`: Result set handle
 * - `row`: Row index (starting from 0)
 * - `col`: Column name (UTF-8 encoded)
 * - `len`: Output parameter, string length
 *
 * # Returns
 * - String value (UTF-8 encoded), returns NULL on error
 *
 * # Memory Management
 * The returned string is dynamically allocated and must be freed by the caller using `graphdb_free_string`
 * to avoid memory leaks.
 *
 * # Safety
 * - `result` must be a valid result handle created by `graphdb_execute` or `graphdb_execute_params`
 * - `col` must be a valid pointer to a null-terminated UTF-8 string
 * - `len` must be a valid pointer to store the string length, or NULL if not needed
 * - `row` must be a valid row index (0 <= row < row count)
 * - The returned pointer must be freed by the caller to avoid memory leaks
 */
char *graphdb_get_string(struct graphdb_result_t *result,
                         int row,
                         const char *col,
                         int *len);

/**
 * 获取二进制数据
 *
 * # Arguments
 * - `result`: Result set handle
 * - `row`: Row index (starting from 0)
 * - `col`: Column name (UTF-8 encoded)
 * - `len`: Output parameter, data length (in bytes)
 *
 * # Returns
 * - Data pointer, returns NULL on error
 *
 * # Note
 * The returned pointer's lifetime is bound to the result set; the caller should not free it
 *
 * # Safety
 * - `result` must be a valid result handle created by `graphdb_execute` or `graphdb_execute_params`
 * - `col` must be a valid pointer to a null-terminated UTF-8 string
 * - `len` must be a valid pointer to store the data length, or NULL if not needed
 * - `row` must be a valid row index (0 <= row < row count)
 * - The returned pointer is only valid as long as the result set is not freed
 */
const uint8_t *graphdb_get_blob(struct graphdb_result_t *result,
                                int row,
                                const char *col,
                                int *len);

/**
 * 获取整数值（按列索引）
 *
 * # Arguments
 * - `result`: Result set handle
 * - `row`: Row index (starting from 0)
 * - `col`: Column index (starting from 0)
 * - `value`: Output parameter, integer value
 *
 * # Returns
 * - Success: GRAPHDB_OK
 * - Failure: Error code
 *
 * # Safety
 * - `result` must be a valid result handle created by `graphdb_execute` or `graphdb_execute_params`
 * - `value` must be a valid pointer to store the result
 * - `row` must be a valid row index (0 <= row < row count)
 * - `col` must be a valid column index (0 <= col < column count)
 */
int graphdb_get_int_by_index(struct graphdb_result_t *result, int row, int col, int64_t *value);

/**
 * 获取字符串值（按列索引）
 *
 * # Arguments
 * - `result`: Result set handle
 * - `row`: Row index (starting from 0)
 * - `col`: Column index (starting from 0)
 * - `len`: Output parameter, string length
 *
 * # Returns
 * - String value (UTF-8 encoded), returns NULL on error
 *
 * # Memory Management
 * The returned string is dynamically allocated and must be freed by the caller using `graphdb_free_string`
 * to avoid memory leaks.
 *
 * # Safety
 * - `result` must be a valid result handle created by `graphdb_execute` or `graphdb_execute_params`
 * - `len` must be a valid pointer to store the string length, or NULL if not needed
 * - `row` must be a valid row index (0 <= row < row count)
 * - `col` must be a valid column index (0 <= col < column count)
 * - The returned pointer must be freed by the caller to avoid memory leaks
 */
char *graphdb_get_string_by_index(struct graphdb_result_t *result,
                                  int row,
                                  int col,
                                  int *len);

/**
 * 获取布尔值（按列索引）
 *
 * # Arguments
 * - `result`: Result set handle
 * - `row`: Row index (starting from 0)
 * - `col`: Column index (starting from 0)
 * - `value`: Output parameter, boolean value
 *
 * # Returns
 * - Success: GRAPHDB_OK
 * - Failure: Error code
 *
 * # Safety
 * - `result` must be a valid result handle created by `graphdb_execute` or `graphdb_execute_params`
 * - `value` must be a valid pointer to store the result
 * - `row` must be a valid row index (0 <= row < row count)
 * - `col` must be a valid column index (0 <= col < column count)
 */
int graphdb_get_bool_by_index(struct graphdb_result_t *result, int row, int col, bool *value);

/**
 * 获取浮点值（按列索引）
 *
 * # Arguments
 * - `result`: Result set handle
 * - `row`: Row index (starting from 0)
 * - `col`: Column index (starting from 0)
 * - `value`: Output parameter, float value
 *
 * # Returns
 * - Success: GRAPHDB_OK
 * - Failure: Error code
 *
 * # Safety
 * - `result` must be a valid result handle created by `graphdb_execute` or `graphdb_execute_params`
 * - `value` must be a valid pointer to store the result
 * - `row` must be a valid row index (0 <= row < row count)
 * - `col` must be a valid column index (0 <= col < column count)
 */
int graphdb_get_float_by_index(struct graphdb_result_t *result, int row, int col, double *value);

/**
 * 获取二进制数据（按列索引）
 *
 * # Arguments
 * - `result`: Result set handle
 * - `row`: Row index (starting from 0)
 * - `col`: Column index (starting from 0)
 * - `len`: Output parameter, data length (in bytes)
 *
 * # Returns
 * - Data pointer, returns NULL on error
 *
 * # Note
 * The returned pointer's lifetime is bound to the result set; the caller should not free it
 *
 * # Safety
 * - `result` must be a valid result handle created by `graphdb_execute` or `graphdb_execute_params`
 * - `len` must be a valid pointer to store the data length, or NULL if not needed
 * - `row` must be a valid row index (0 <= row < row count)
 * - `col` must be a valid column index (0 <= col < column count)
 * - The returned pointer is only valid as long as the result set is not freed
 */
const uint8_t *graphdb_get_blob_by_index(struct graphdb_result_t *result,
                                         int row,
                                         int col,
                                         int *len);

/**
 * 获取列类型
 *
 * # Arguments
 * - `result`: Result set handle
 * - `col`: Column index (starting from 0)
 *
 * # Returns
 * - Column type, returns GRAPHDB_NULL on error
 *
 * # Safety
 * - `result` must be a valid result handle created by `graphdb_execute` or `graphdb_execute_params`
 * - `col` must be a valid column index (0 <= col < column count)
 */
enum graphdb_value_type_t graphdb_column_type(struct graphdb_result_t *result, int col);

/**
 * 创建会话
 *
 * # Arguments
 * - `db`: Database handle
 * - `session`: Output parameter, session handle
 *
 * # Returns
 * - Success: GRAPHDB_OK
 * - Failure: Error code
 *
 * # Safety
 * - `db` must be a valid database handle created by `graphdb_open` or `graphdb_open_v2`
 * - `session` must be a valid pointer to store the session handle
 * - The caller is responsible for closing the session using `graphdb_session_close` when done
 * - The session handle must not be used after closing
 */
int graphdb_session_create(struct graphdb_t *db, struct graphdb_session_t **session);

/**
 * 关闭会话
 *
 * # Arguments
 * - `session`: Session handle
 *
 * # Returns
 * - Success: GRAPHDB_OK
 * - Failure: Error code
 *
 * # Safety
 * - `session` must be a valid session handle created by `graphdb_session_create`
 * - After calling this function, the session handle becomes invalid and must not be used
 * - All pending transactions will be rolled back
 */
int graphdb_session_close(struct graphdb_session_t *session);

/**
 * 切换图空间
 *
 * # Arguments
 * - `session`: Session handle
 * - `space_name`: Graph space name (UTF-8 encoded)
 *
 * # Returns
 * - Success: GRAPHDB_OK
 * - Failure: Error code
 *
 * # Safety
 * - `session` must be a valid session handle created by `graphdb_session_create`
 * - `space_name` must be a valid pointer to a null-terminated UTF-8 string
 */
int graphdb_session_use_space(struct graphdb_session_t *session, const char *space_name);

/**
 * 获取当前图空间
 *
 * # Arguments
 * - `session`: Session handle
 *
 * # Returns
 * - Current graph space name (UTF-8 encoded), returns NULL if none
 *
 * # Memory Management
 * The returned string is dynamically allocated and must be freed by the caller using `graphdb_free_string`
 * to avoid memory leaks.
 *
 * # Example
 * ```c
 * char* space = graphdb_session_current_space(session);
 * if (space) {
 *     printf("Current space: %s\n", space);
 *     graphdb_free_string(space);  // Must free
 * }
 * ```
 *
 * # Safety
 * - `session` must be a valid session handle created by `graphdb_session_create`
 * - The returned pointer must be freed by the caller to avoid memory leaks
 */
char *graphdb_session_current_space(struct graphdb_session_t *session);

/**
 * 设置自动提交模式
 *
 * # Arguments
 * - `session`: Session handle
 * - `autocommit`: Whether to enable autocommit
 *
 * # Returns
 * - Success: GRAPHDB_OK
 * - Failure: Error code
 *
 * # Safety
 * - `session` must be a valid session handle created by `graphdb_session_create`
 */
int graphdb_session_set_autocommit(struct graphdb_session_t *session, bool autocommit);

/**
 * 获取自动提交模式
 *
 * # Arguments
 * - `session`: Session handle
 *
 * # Returns
 * - Whether autocommit is enabled
 *
 * # Safety
 * - `session` must be a valid session handle created by `graphdb_session_create`
 */
bool graphdb_session_get_autocommit(struct graphdb_session_t *session);

/**
 * 获取上次操作影响的行数
 *
 * # Arguments
 * - `session`: Session handle
 *
 * # Returns
 * - Number of rows affected by last operation, returns 0 if session is invalid
 *
 * # Safety
 * - `session` must be a valid session handle created by `graphdb_session_create`
 */
int graphdb_changes(struct graphdb_session_t *session);

/**
 * 获取自数据库打开以来的总变更数量
 *
 * # Arguments
 * - `session`: Session handle
 *
 * # Returns
 * - Total number of changes
 *
 * # Safety
 * - `session` must be a valid session handle created by `graphdb_session_create`
 */
int64_t graphdb_total_changes(struct graphdb_session_t *session);

/**
 * 获取最后插入的顶点 ID
 *
 * # Arguments
 * - `session`: Session handle
 *
 * # Returns
 * - Last inserted vertex ID, returns 0 if none
 *
 * # Safety
 * - `session` must be a valid session handle created by `graphdb_session_create`
 */
int64_t graphdb_last_insert_vertex_id(struct graphdb_session_t *session);

/**
 * 获取最后插入的边 ID
 *
 * # Arguments
 * - `session`: Session handle
 *
 * # Returns
 * - Last inserted edge ID, returns 0 if none
 *
 * # Safety
 * - `session` must be a valid session handle created by `graphdb_session_create`
 */
int64_t graphdb_last_insert_edge_id(struct graphdb_session_t *session);

/**
 * 设置忙等待超时
 *
 * # Arguments
 * - `session`: Session handle
 * - `timeout_ms`: Timeout in milliseconds
 *
 * # Returns
 * - Success: GRAPHDB_OK
 * - Failure: Error code
 *
 * # Safety
 * - `session` must be a valid session handle created by `graphdb_session_create`
 */
int graphdb_busy_timeout(struct graphdb_session_t *session, int timeout_ms);

/**
 * 获取忙等待超时
 *
 * # Arguments
 * - `session`: Session handle
 *
 * # Returns
 * - Timeout in milliseconds, returns -1 on error
 *
 * # Safety
 * - `session` must be a valid session handle created by `graphdb_session_create`
 */
int graphdb_busy_timeout_get(struct graphdb_session_t *session);

/**
 * 设置 SQL 追踪回调
 *
 * # Arguments
 * - `session`: Session handle
 * - `callback`: Trace callback function, NULL to disable tracing
 * - `user_data`: User data pointer, will be passed to the callback
 *
 * # Returns
 * - Success: GRAPHDB_OK
 * - Failure: Error code
 *
 * # Example
 * ```c
 * extern void my_trace_callback(const char* sql, void* data) {
 *     printf("Executing: %s\n", sql);
 * }
 *
 * graphdb_trace(session, my_trace_callback, NULL);
 * ```
 *
 * # Safety
 * - `session` must be a valid session handle created by `graphdb_session_create`
 * - `callback` must be a valid function pointer, or NULL to disable tracing
 * - `user_data` is passed to the callback and must remain valid for the lifetime of the callback
 */
int graphdb_trace(struct graphdb_session_t *session,
                  graphdb_trace_callback callback,
                  void *user_data);

/**
 * 设置提交钩子
 *
 * # Arguments
 * - `session`: Session handle
 * - `callback`: Commit hook callback function, NULL to disable the hook
 * - `user_data`: User data pointer, will be passed to the callback
 *
 * # Returns
 * - Previous hook user data pointer (if any)
 *
 * # Description
 * The commit hook is called before a transaction is committed. If the callback returns a non-zero value,
 * the transaction will be rolled back.
 *
 * # Safety
 * - `session` must be a valid session handle created by `graphdb_session_create`
 * - `callback` must be a valid function pointer, or NULL to disable the hook
 * - `user_data` is passed to the callback and must remain valid for the lifetime of the callback
 */
void *graphdb_commit_hook(struct graphdb_session_t *session,
                          graphdb_commit_hook_callback callback,
                          void *user_data);

/**
 * 设置回滚钩子
 *
 * # Arguments
 * - `session`: Session handle
 * - `callback`: Rollback hook callback function, NULL to disable the hook
 * - `user_data`: User data pointer, will be passed to the callback
 *
 * # Returns
 * - Previous hook user data pointer (if any)
 *
 * # Safety
 * - `session` must be a valid session handle created by `graphdb_session_create`
 * - `callback` must be a valid function pointer, or NULL to disable the hook
 * - `user_data` is passed to the callback and must remain valid for the lifetime of the callback
 */
void *graphdb_rollback_hook(struct graphdb_session_t *session,
                            graphdb_rollback_hook_callback callback,
                            void *user_data);

/**
 * 设置更新钩子
 *
 * When data in the database changes, the callback function is called
 *
 * # Arguments
 * - `session`: Session handle
 * - `callback`: Update hook callback function, NULL to disable the hook
 * - `user_data`: User data pointer, will be passed to the callback
 *
 * # Returns
 * - Previous hook user data pointer (if any)
 *
 * # Callback Parameters
 * - `operation`: Operation type (1=INSERT, 2=UPDATE, 3=DELETE)
 * - `database`: Database/space name
 * - `table`: Table name (empty string for graph database)
 * - `rowid`: Affected row ID
 *
 * # Safety
 * - `session` must be a valid session handle created by `graphdb_session_create`
 * - `callback` must be a valid function pointer, or NULL to disable the hook
 * - `user_data` is passed to the callback and must remain valid for the lifetime of the callback
 */
void *graphdb_update_hook(struct graphdb_session_t *session,
                          graphdb_update_hook_callback callback,
                          void *user_data);

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
 *
 * # Safety
 * - `session` must be a valid session handle created by `graphdb_session_create`
 * - `txn` must be a valid pointer to store the transaction handle
 * - The session must not have been closed
 * - The caller is responsible for freeing the transaction using `graphdb_txn_free` when done
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
 *
 * # Safety
 * - `session` must be a valid session handle created by `graphdb_session_create`
 * - `txn` must be a valid pointer to store the transaction handle
 * - The session must not have been closed
 * - The caller is responsible for freeing the transaction using `graphdb_txn_free` when done
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
 *
 * # Safety
 * - `txn` must be a valid transaction handle created by `graphdb_txn_begin` or `graphdb_txn_begin_readonly`
 * - `query` must be a valid pointer to a null-terminated UTF-8 string
 * - `result` must be a valid pointer to store the result handle
 * - The transaction must not have been committed or rolled back
 * - The caller is responsible for freeing the result using `graphdb_result_free` when done
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
 *
 * # Safety
 * - `txn` must be a valid transaction handle created by `graphdb_txn_begin` or `graphdb_txn_begin_readonly`
 * - The transaction must not have been committed or rolled back already
 * - The associated session must still be valid
 * - After calling this function, the transaction handle should be freed using `graphdb_txn_free`
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
 *
 * # Safety
 * - `txn` must be a valid transaction handle created by `graphdb_txn_begin` or `graphdb_txn_begin_readonly`
 * - The transaction must not have been committed or rolled back already
 * - The associated session must still be valid
 * - After calling this function, the transaction handle should be freed using `graphdb_txn_free`
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
 *
 * # Safety
 * - `txn` must be a valid transaction handle created by `graphdb_txn_begin` or `graphdb_txn_begin_readonly`
 * - `name` must be a valid pointer to a null-terminated UTF-8 string
 * - The transaction must not have been committed or rolled back
 */
int64_t graphdb_txn_savepoint(struct graphdb_txn_t *txn,
                              const char *name);

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
 *
 * # Safety
 * - `txn` must be a valid transaction handle created by `graphdb_txn_begin` or `graphdb_txn_begin_readonly`
 * - `savepoint_id` must be a valid savepoint ID returned by `graphdb_txn_savepoint`
 * - The transaction must not have been committed or rolled back
 */
int graphdb_txn_release_savepoint(struct graphdb_txn_t *txn,
                                  int64_t savepoint_id);

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
 *
 * # Safety
 * - `txn` must be a valid transaction handle created by `graphdb_txn_begin` or `graphdb_txn_begin_readonly`
 * - `savepoint_id` must be a valid savepoint ID returned by `graphdb_txn_savepoint`
 * - The transaction must not have been committed or rolled back
 */
int graphdb_txn_rollback_to_savepoint(struct graphdb_txn_t *txn,
                                      int64_t savepoint_id);

/**
 * 释放事务句柄
 *
 * # 参数
 * - `txn`: 事务句柄
 *
 * # 返回
 * - 成功: GRAPHDB_OK
 * - 失败: 错误码
 *
 * # Safety
 * - `txn` must be a valid transaction handle created by `graphdb_txn_begin` or `graphdb_txn_begin_readonly`
 * - After calling this function, the transaction handle becomes invalid and must not be used
 * - If the transaction has not been committed or rolled back, it will be automatically rolled back
 * - It is safe to call this function multiple times on the same handle (idempotent)
 */
int graphdb_txn_free(struct graphdb_txn_t *txn);
