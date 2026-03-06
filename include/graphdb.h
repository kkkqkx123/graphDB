/**
 * @file graphdb.h
 * @brief GraphDB C API 头文件
 * 
 * 提供 GraphDB 数据库的 C 语言接口
 * 
 * @version 0.1.0
 */

#ifndef GRAPHDB_H
#define GRAPHDB_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ==================== 类型定义 ==================== */

/**
 * @brief 值类型枚举
 */
typedef enum {
    GRAPHDB_NULL = 0,      /**< 空值 */
    GRAPHDB_BOOL = 1,      /**< 布尔值 */
    GRAPHDB_INT = 2,       /**< 整数 */
    GRAPHDB_FLOAT = 3,     /**< 浮点数 */
    GRAPHDB_STRING = 4,    /**< 字符串 */
    GRAPHDB_LIST = 5,      /**< 列表 */
    GRAPHDB_MAP = 6,       /**< 映射 */
    GRAPHDB_VERTEX = 7,    /**< 顶点 */
    GRAPHDB_EDGE = 8,       /**< 边 */
    GRAPHDB_PATH = 9       /**< 路径 */
} graphdb_value_type_t;

/**
 * @brief 错误码枚举
 */
typedef enum {
    GRAPHDB_OK = 0,          /**< 成功 */
    GRAPHDB_ERROR = 1,       /**< 一般错误 */
    GRAPHDB_INTERNAL = 2,     /**< 内部错误 */
    GRAPHDB_PERM = 3,        /**< 权限被拒绝 */
    GRAPHDB_ABORT = 4,       /**< 操作被中止 */
    GRAPHDB_BUSY = 5,        /**< 数据库忙 */
    GRAPHDB_LOCKED = 6,      /**< 数据库被锁定 */
    GRAPHDB_NOMEM = 7,       /**< 内存不足 */
    GRAPHDB_READONLY = 8,     /**< 只读 */
    GRAPHDB_INTERRUPT = 9,    /**< 操作被中断 */
    GRAPHDB_IOERR = 10,      /**< IO 错误 */
    GRAPHDB_CORRUPT = 11,    /**< 数据损坏 */
    GRAPHDB_NOTFOUND = 12,    /**< 未找到 */
    GRAPHDB_FULL = 13,        /**< 磁盘已满 */
    GRAPHDB_CANTOPEN = 14,    /**< 无法打开 */
    GRAPHDB_PROTOCOL = 15,    /**< 协议错误 */
    GRAPHDB_SCHEMA = 16,      /**< 模式错误 */
    GRAPHDB_TOOBIG = 17,      /**< 数据过大 */
    GRAPHDB_CONSTRAINT = 18,  /**< 约束违反 */
    GRAPHDB_MISMATCH = 19,    /**< 类型不匹配 */
    GRAPHDB_MISUSE = 20,     /**< 误用 */
    GRAPHDB_RANGE = 21        /**< 超出范围 */
} graphdb_error_code_t;

/**
 * @brief 字符串结构
 */
typedef struct {
    const char* data;  /**< 字符串数据 */
    size_t len;       /**< 字符串长度 */
} graphdb_string_t;

/**
 * @brief 值数据联合体
 */
typedef union {
    bool boolean;               /**< 布尔值 */
    int64_t integer;           /**< 整数 */
    double floating;            /**< 浮点数 */
    graphdb_string_t string;   /**< 字符串 */
    void* ptr;                /**< 指针 */
} graphdb_value_data_t;

/**
 * @brief 值结构
 */
typedef struct {
    graphdb_value_type_t type_;  /**< 值类型 */
    graphdb_value_data_t data;   /**< 值数据 */
} graphdb_value_t;

/**
 * @brief 数据库配置
 */
typedef struct {
    bool read_only;           /**< 是否只读 */
    bool create_if_missing;   /**< 如果不存在是否创建 */
    int cache_size_mb;        /**< 缓存大小（MB） */
    int max_open_files;       /**< 最大打开文件数 */
    bool enable_compression;   /**< 是否启用压缩 */
} graphdb_config_t;

/* ==================== 不透明句柄类型 ==================== */

typedef struct graphdb_t graphdb_t;                  /**< 数据库句柄 */
typedef struct graphdb_session_t graphdb_session_t;    /**< 会话句柄 */
typedef struct graphdb_stmt_t graphdb_stmt_t;          /**< 预编译语句句柄 */
typedef struct graphdb_txn_t graphdb_txn_t;            /**< 事务句柄 */
typedef struct graphdb_result_t graphdb_result_t;      /**< 结果集句柄 */
typedef struct graphdb_batch_t graphdb_batch_t;        /**< 批量操作句柄 */

/* ==================== 数据库管理函数 ==================== */

/**
 * @brief 打开数据库
 * 
 * @param path 数据库文件路径（UTF-8 编码）
 * @param db 输出参数，数据库句柄
 * @return 成功返回 GRAPHDB_OK，失败返回错误码
 */
int graphdb_open(const char* path, graphdb_t** db);

/**
 * @brief 关闭数据库
 * 
 * @param db 数据库句柄
 * @return 成功返回 GRAPHDB_OK，失败返回错误码
 */
int graphdb_close(graphdb_t* db);

/**
 * @brief 获取错误码
 * 
 * @param db 数据库句柄
 * @return 错误码，如果没有错误返回 GRAPHDB_OK
 */
int graphdb_errcode(graphdb_t* db);

/**
 * @brief 获取库版本
 * 
 * @return 版本字符串
 */
const char* graphdb_libversion(void);

/**
 * @brief 释放字符串
 * 
 * @param str 字符串指针
 */
void graphdb_free_string(char* str);

/**
 * @brief 释放内存
 * 
 * @param ptr 内存指针
 */
void graphdb_free(void* ptr);

/* ==================== 会话管理函数 ==================== */

/**
 * @brief 创建会话
 * 
 * @param db 数据库句柄
 * @param session 输出参数，会话句柄
 * @return 成功返回 GRAPHDB_OK，失败返回错误码
 */
int graphdb_session_create(graphdb_t* db, graphdb_session_t** session);

/**
 * @brief 关闭会话
 * 
 * @param session 会话句柄
 * @return 成功返回 GRAPHDB_OK，失败返回错误码
 */
int graphdb_session_close(graphdb_session_t* session);

/**
 * @brief 切换图空间
 * 
 * @param session 会话句柄
 * @param space_name 图空间名称（UTF-8 编码）
 * @return 成功返回 GRAPHDB_OK，失败返回错误码
 */
int graphdb_session_use_space(graphdb_session_t* session, const char* space_name);

/**
 * @brief 获取当前图空间
 * 
 * @param session 会话句柄
 * @return 当前图空间名称（UTF-8 编码），如果没有则返回 NULL
 */
const char* graphdb_session_current_space(graphdb_session_t* session);

/**
 * @brief 设置自动提交模式
 * 
 * @param session 会话句柄
 * @param autocommit 是否自动提交
 * @return 成功返回 GRAPHDB_OK，失败返回错误码
 */
int graphdb_session_set_autocommit(graphdb_session_t* session, bool autocommit);

/**
 * @brief 获取自动提交模式
 * 
 * @param session 会话句柄
 * @return 是否自动提交
 */
bool graphdb_session_get_autocommit(graphdb_session_t* session);

/* ==================== 查询执行函数 ==================== */

/**
 * @brief 执行简单查询
 * 
 * @param session 会话句柄
 * @param query 查询语句（UTF-8 编码）
 * @param result 输出参数，结果集句柄
 * @return 成功返回 GRAPHDB_OK，失败返回错误码
 */
int graphdb_execute(graphdb_session_t* session, const char* query, graphdb_result_t** result);

/**
 * @brief 执行参数化查询
 * 
 * @param session 会话句柄
 * @param query 查询语句（UTF-8 编码）
 * @param params 参数数组
 * @param param_count 参数数量
 * @param result 输出参数，结果集句柄
 * @return 成功返回 GRAPHDB_OK，失败返回错误码
 */
int graphdb_execute_params(graphdb_session_t* session, const char* query, 
                         const graphdb_value_t* params, size_t param_count, 
                         graphdb_result_t** result);

/* ==================== 结果处理函数 ==================== */

/**
 * @brief 释放结果集
 * 
 * @param result 结果集句柄
 * @return 成功返回 GRAPHDB_OK，失败返回错误码
 */
int graphdb_result_free(graphdb_result_t* result);

/**
 * @brief 获取结果集列数
 * 
 * @param result 结果集句柄
 * @return 列数，错误返回 -1
 */
int graphdb_column_count(graphdb_result_t* result);

/**
 * @brief 获取结果集行数
 * 
 * @param result 结果集句柄
 * @return 行数，错误返回 -1
 */
int graphdb_row_count(graphdb_result_t* result);

/**
 * @brief 获取列名
 * 
 * @param result 结果集句柄
 * @param index 列索引（从 0 开始）
 * @return 列名（UTF-8 编码），错误返回 NULL
 */
const char* graphdb_column_name(graphdb_result_t* result, int index);

/**
 * @brief 获取整数值
 * 
 * @param result 结果集句柄
 * @param row 行索引（从 0 开始）
 * @param col 列名（UTF-8 编码）
 * @param value 输出参数，整数值
 * @return 成功返回 GRAPHDB_OK，失败返回错误码
 */
int graphdb_get_int(graphdb_result_t* result, int row, const char* col, int64_t* value);

/**
 * @brief 获取字符串值
 * 
 * @param result 结果集句柄
 * @param row 行索引（从 0 开始）
 * @param col 列名（UTF-8 编码）
 * @param len 输出参数，字符串长度
 * @return 字符串值（UTF-8 编码），错误返回 NULL
 */
const char* graphdb_get_string(graphdb_result_t* result, int row, const char* col, int* len);

/* ==================== 预编译语句函数 ==================== */

/**
 * @brief 准备语句
 * 
 * @param session 会话句柄
 * @param query 查询语句（UTF-8 编码）
 * @param stmt 输出参数，语句句柄
 * @return 成功返回 GRAPHDB_OK，失败返回错误码
 */
int graphdb_prepare(graphdb_session_t* session, const char* query, graphdb_stmt_t** stmt);

/**
 * @brief 绑定 NULL 值（按索引）
 * 
 * @param stmt 语句句柄
 * @param index 参数索引（从 1 开始）
 * @return 成功返回 GRAPHDB_OK，失败返回错误码
 */
int graphdb_bind_null(graphdb_stmt_t* stmt, int index);

/**
 * @brief 绑定布尔值（按索引）
 * 
 * @param stmt 语句句柄
 * @param index 参数索引（从 1 开始）
 * @param value 布尔值
 * @return 成功返回 GRAPHDB_OK，失败返回错误码
 */
int graphdb_bind_bool(graphdb_stmt_t* stmt, int index, bool value);

/**
 * @brief 绑定整数值（按索引）
 * 
 * @param stmt 语句句柄
 * @param index 参数索引（从 1 开始）
 * @param value 整数值
 * @return 成功返回 GRAPHDB_OK，失败返回错误码
 */
int graphdb_bind_int(graphdb_stmt_t* stmt, int index, int64_t value);

/**
 * @brief 绑定浮点值（按索引）
 * 
 * @param stmt 语句句柄
 * @param index 参数索引（从 1 开始）
 * @param value 浮点值
 * @return 成功返回 GRAPHDB_OK，失败返回错误码
 */
int graphdb_bind_float(graphdb_stmt_t* stmt, int index, double value);

/**
 * @brief 绑定字符串值（按索引）
 * 
 * @param stmt 语句句柄
 * @param index 参数索引（从 1 开始）
 * @param value 字符串值（UTF-8 编码）
 * @param len 字符串长度（-1 表示自动计算）
 * @return 成功返回 GRAPHDB_OK，失败返回错误码
 */
int graphdb_bind_string(graphdb_stmt_t* stmt, int index, const char* value, int len);

/**
 * @brief 绑定参数（按名称）
 * 
 * @param stmt 语句句柄
 * @param name 参数名称（UTF-8 编码）
 * @param value 值
 * @return 成功返回 GRAPHDB_OK，失败返回错误码
 */
int graphdb_bind_by_name(graphdb_stmt_t* stmt, const char* name, graphdb_value_t value);

/**
 * @brief 重置语句
 * 
 * 清除所有绑定的参数，使语句可以重新执行
 * 
 * @param stmt 语句句柄
 * @return 成功返回 GRAPHDB_OK，失败返回错误码
 */
int graphdb_reset(graphdb_stmt_t* stmt);

/**
 * @brief 清除绑定
 * 
 * 清除所有绑定的参数
 * 
 * @param stmt 语句句柄
 * @return 成功返回 GRAPHDB_OK，失败返回错误码
 */
int graphdb_clear_bindings(graphdb_stmt_t* stmt);

/**
 * @brief 释放语句
 * 
 * @param stmt 语句句柄
 * @return 成功返回 GRAPHDB_OK，失败返回错误码
 */
int graphdb_finalize(graphdb_stmt_t* stmt);

/**
 * @brief 获取参数索引
 * 
 * @param stmt 语句句柄
 * @param name 参数名称（UTF-8 编码）
 * @return 参数索引（从 1 开始），未找到返回 0
 */
int graphdb_bind_parameter_index(graphdb_stmt_t* stmt, const char* name);

/**
 * @brief 获取参数名称
 * 
 * @param stmt 语句句柄
 * @param index 参数索引（从 1 开始）
 * @return 参数名称（UTF-8 编码），未找到返回 NULL
 */
const char* graphdb_bind_parameter_name(graphdb_stmt_t* stmt, int index);

/**
 * @brief 获取参数数量
 * 
 * @param stmt 语句句柄
 * @return 参数数量
 */
int graphdb_bind_parameter_count(graphdb_stmt_t* stmt);

/* ==================== 事务管理函数 ==================== */

/**
 * @brief 开始事务
 * 
 * @param session 会话句柄
 * @param txn 输出参数，事务句柄
 * @return 成功返回 GRAPHDB_OK，失败返回错误码
 */
int graphdb_txn_begin(graphdb_session_t* session, graphdb_txn_t** txn);

/**
 * @brief 开始只读事务
 * 
 * @param session 会话句柄
 * @param txn 输出参数，事务句柄
 * @return 成功返回 GRAPHDB_OK，失败返回错误码
 */
int graphdb_txn_begin_readonly(graphdb_session_t* session, graphdb_txn_t** txn);

/**
 * @brief 在事务中执行查询
 * 
 * @param txn 事务句柄
 * @param query 查询语句（UTF-8 编码）
 * @param result 输出参数，结果集句柄
 * @return 成功返回 GRAPHDB_OK，失败返回错误码
 */
int graphdb_txn_execute(graphdb_txn_t* txn, const char* query, graphdb_result_t** result);

/**
 * @brief 提交事务
 * 
 * @param txn 事务句柄
 * @return 成功返回 GRAPHDB_OK，失败返回错误码
 */
int graphdb_txn_commit(graphdb_txn_t* txn);

/**
 * @brief 回滚事务
 * 
 * @param txn 事务句柄
 * @return 成功返回 GRAPHDB_OK，失败返回错误码
 */
int graphdb_txn_rollback(graphdb_txn_t* txn);

/**
 * @brief 创建保存点
 * 
 * @param txn 事务句柄
 * @param name 保存点名称（UTF-8 编码）
 * @return 成功返回保存点 ID，失败返回 -1
 */
int64_t graphdb_txn_savepoint(graphdb_txn_t* txn, const char* name);

/**
 * @brief 释放保存点
 * 
 * @param txn 事务句柄
 * @param savepoint_id 保存点 ID
 * @return 成功返回 GRAPHDB_OK，失败返回错误码
 */
int graphdb_txn_release_savepoint(graphdb_txn_t* txn, int64_t savepoint_id);

/**
 * @brief 回滚到保存点
 * 
 * @param txn 事务句柄
 * @param savepoint_id 保存点 ID
 * @return 成功返回 GRAPHDB_OK，失败返回错误码
 */
int graphdb_txn_rollback_to_savepoint(graphdb_txn_t* txn, int64_t savepoint_id);

/**
 * @brief 释放事务句柄
 * 
 * @param txn 事务句柄
 * @return 成功返回 GRAPHDB_OK，失败返回错误码
 */
int graphdb_txn_free(graphdb_txn_t* txn);

/* ==================== 批量操作函数 ==================== */

/**
 * @brief 创建批量插入器
 * 
 * @param session 会话句柄
 * @param batch_size 批次大小
 * @param batch 输出参数，批量操作句柄
 * @return 成功返回 GRAPHDB_OK，失败返回错误码
 */
int graphdb_batch_inserter_create(graphdb_session_t* session, int batch_size, graphdb_batch_t** batch);

/**
 * @brief 添加顶点
 * 
 * @param batch 批量操作句柄
 * @param vid 顶点 ID
 * @param tag_name 标签名称（UTF-8 编码）
 * @param properties 属性数组
 * @param prop_count 属性数量
 * @return 成功返回 GRAPHDB_OK，失败返回错误码
 */
int graphdb_batch_add_vertex(graphdb_batch_t* batch, int64_t vid, const char* tag_name,
                          const graphdb_value_t* properties, size_t prop_count);

/**
 * @brief 添加边
 * 
 * @param batch 批量操作句柄
 * @param src_vid 源顶点 ID
 * @param dst_vid 目标顶点 ID
 * @param edge_type 边类型名称（UTF-8 编码）
 * @param rank 排名
 * @param properties 属性数组
 * @param prop_count 属性数量
 * @return 成功返回 GRAPHDB_OK，失败返回错误码
 */
int graphdb_batch_add_edge(graphdb_batch_t* batch, int64_t src_vid, int64_t dst_vid,
                        const char* edge_type, int64_t rank,
                        const graphdb_value_t* properties, size_t prop_count);

/**
 * @brief 执行批量插入
 * 
 * @param batch 批量操作句柄
 * @return 成功返回 GRAPHDB_OK，失败返回错误码
 */
int graphdb_batch_flush(graphdb_batch_t* batch);

/**
 * @brief 获取缓冲的顶点数量
 * 
 * @param batch 批量操作句柄
 * @return 缓冲的顶点数量
 */
int graphdb_batch_buffered_vertices(graphdb_batch_t* batch);

/**
 * @brief 获取缓冲的边数量
 * 
 * @param batch 批量操作句柄
 * @return 缓冲的边数量
 */
int graphdb_batch_buffered_edges(graphdb_batch_t* batch);

/**
 * @brief 释放批量操作句柄
 * 
 * @param batch 批量操作句柄
 * @return 成功返回 GRAPHDB_OK，失败返回错误码
 */
int graphdb_batch_free(graphdb_batch_t* batch);

/* ==================== 错误处理函数 ==================== */

/**
 * @brief 获取错误码描述
 * 
 * @param code 错误码
 * @return 错误描述字符串（静态生命周期）
 */
const char* graphdb_error_string(int code);

/**
 * @brief 获取最后一个错误消息
 * 
 * @param msg 输出缓冲区
 * @param len 缓冲区长度
 * @return 实际写入的字符数（不包括 null 终止符）
 */
int graphdb_errmsg(char* msg, size_t len);

#ifdef __cplusplus
}
#endif

#endif /* GRAPHDB_H */
