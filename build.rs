//! 构建脚本
//!
//! 用于生成 C API 头文件和配置构建环境

use std::env;
use std::path::PathBuf;

fn main() {
    // 只在启用 c_api 特性时生成头文件
    if env::var("CARGO_FEATURE_C_API").is_ok() {
        generate_c_header();
    }

    // 设置链接参数
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/api/embedded/c_api/");
}

/// 生成 C 头文件
fn generate_c_header() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let output_path = PathBuf::from(&crate_dir).join("include").join("graphdb.h");

    // 确保 include 目录存在
    std::fs::create_dir_all(output_path.parent().unwrap())
        .expect("Failed to create include directory");

    // 尝试使用 cbindgen 生成头文件
    match try_cbindgen(&crate_dir, &output_path) {
        Ok(_) => println!("cargo:warning=Generated C header at {:?}", output_path),
        Err(e) => {
            println!("cargo:warning=Failed to generate C header with cbindgen: {}", e);
            // 如果 cbindgen 失败，使用备用方案
            generate_fallback_header(&output_path);
        }
    }
}

/// 尝试使用 cbindgen 生成头文件
fn try_cbindgen(crate_dir: &str, output_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = PathBuf::from(crate_dir).join("cbindgen.toml");

    // 检查 cbindgen 配置是否存在
    if !config_path.exists() {
        return Err("cbindgen.toml not found".into());
    }

    // 使用 cbindgen 生成头文件
    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_config(cbindgen::Config::from_root_or_default(crate_dir))
        .generate()
        .map_err(|e| format!("cbindgen generation failed: {}", e))?
        .write_to_file(output_path);

    Ok(())
}

/// 生成备用头文件（当 cbindgen 失败时使用）
fn generate_fallback_header(output_path: &PathBuf) {
    let header_content = r#"/**
 * GraphDB C API
 *
 * GraphDB C API 头文件
 * 提供 GraphDB 的 C 语言接口
 *
 * 版本: 0.1.0
 * 许可: Apache-2.0
 *
 * 更多信息请访问: https://github.com/kkkqkx123/graphDB
 */

#ifndef GRAPHDB_H
#define GRAPHDB_H

#pragma once

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ==================== 错误码定义 ==================== */

/**
 * 错误码枚举
 */
typedef enum {
    GRAPHDB_OK = 0,           /**< 成功 */
    GRAPHDB_ERROR = 1,        /**< 一般错误 */
    GRAPHDB_INTERNAL = 2,     /**< 内部错误 */
    GRAPHDB_PERM = 3,         /**< 权限被拒绝 */
    GRAPHDB_ABORT = 4,        /**< 操作被中止 */
    GRAPHDB_BUSY = 5,         /**< 数据库忙 */
    GRAPHDB_LOCKED = 6,       /**< 数据库被锁定 */
    GRAPHDB_NOMEM = 7,        /**< 内存不足 */
    GRAPHDB_READONLY = 8,     /**< 只读 */
    GRAPHDB_INTERRUPT = 9,    /**< 操作被中断 */
    GRAPHDB_IOERR = 10,       /**< IO 错误 */
    GRAPHDB_CORRUPT = 11,     /**< 数据损坏 */
    GRAPHDB_NOTFOUND = 12,    /**< 未找到 */
    GRAPHDB_FULL = 13,        /**< 磁盘已满 */
    GRAPHDB_CANTOPEN = 14,    /**< 无法打开 */
    GRAPHDB_PROTOCOL = 15,    /**< 协议错误 */
    GRAPHDB_SCHEMA = 16,      /**< 模式错误 */
    GRAPHDB_TOOBIG = 17,      /**< 数据过大 */
    GRAPHDB_CONSTRAINT = 18,  /**< 约束违反 */
    GRAPHDB_MISMATCH = 19,    /**< 类型不匹配 */
    GRAPHDB_MISUSE = 20,      /**< API 误用 */
    GRAPHDB_RANGE = 21,       /**< 超出范围 */
} graphdb_error_code_t;

/* ==================== 类型定义 ==================== */

/**
 * 数据库句柄（不透明指针）
 */
typedef struct graphdb_t graphdb_t;

/**
 * 会话句柄（不透明指针）
 */
typedef struct graphdb_session_t graphdb_session_t;

/**
 * 预编译语句句柄（不透明指针）
 */
typedef struct graphdb_stmt_t graphdb_stmt_t;

/**
 * 事务句柄（不透明指针）
 */
typedef struct graphdb_txn_t graphdb_txn_t;

/**
 * 结果集句柄（不透明指针）
 */
typedef struct graphdb_result_t graphdb_result_t;

/**
 * 值类型枚举
 */
typedef enum {
    GRAPHDB_NULL = 0,     /**< 空值 */
    GRAPHDB_BOOL = 1,     /**< 布尔值 */
    GRAPHDB_INT = 2,      /**< 整数 */
    GRAPHDB_FLOAT = 3,    /**< 浮点数 */
    GRAPHDB_STRING = 4,   /**< 字符串 */
    GRAPHDB_LIST = 5,     /**< 列表 */
    GRAPHDB_MAP = 6,      /**< 映射 */
    GRAPHDB_VERTEX = 7,   /**< 顶点 */
    GRAPHDB_EDGE = 8,     /**< 边 */
    GRAPHDB_PATH = 9,     /**< 路径 */
} graphdb_value_type_t;

/**
 * 值结构
 */
typedef struct {
    graphdb_value_type_t type;  /**< 值类型 */
    union {
        bool boolean;           /**< 布尔值 */
        int64_t integer;        /**< 整数 */
        double floating;        /**< 浮点数 */
        struct {
            const char *data;   /**< 字符串数据 */
            size_t len;         /**< 字符串长度 */
        } string;               /**< 字符串 */
        void *ptr;              /**< 其他类型指针 */
    } data;                     /**< 值数据 */
} graphdb_value_t;

/**
 * 数据库配置结构
 */
typedef struct {
    bool read_only;             /**< 只读模式 */
    bool create_if_missing;     /**< 如果不存在则创建 */
    int cache_size_mb;          /**< 缓存大小（MB） */
    int max_open_files;         /**< 最大打开文件数 */
    bool enable_compression;    /**< 启用压缩 */
} graphdb_config_t;

/* ==================== 数据库管理 API ==================== */

/**
 * 打开数据库
 *
 * @param path 数据库文件路径（UTF-8 编码）
 * @param db 输出参数，数据库句柄
 * @return 错误码，GRAPHDB_OK 表示成功
 */
int graphdb_open(const char *path, graphdb_t **db);

/**
 * 打开内存数据库
 *
 * @param db 输出参数，数据库句柄
 * @return 错误码，GRAPHDB_OK 表示成功
 */
int graphdb_open_memory(graphdb_t **db);

/**
 * 使用配置打开数据库
 *
 * @param path 数据库文件路径（UTF-8 编码）
 * @param config 数据库配置
 * @param db 输出参数，数据库句柄
 * @return 错误码，GRAPHDB_OK 表示成功
 */
int graphdb_open_config(const char *path, const graphdb_config_t *config, graphdb_t **db);

/**
 * 关闭数据库
 *
 * @param db 数据库句柄
 * @return 错误码，GRAPHDB_OK 表示成功
 */
int graphdb_close(graphdb_t *db);

/**
 * 获取错误码
 *
 * @param db 数据库句柄
 * @return 错误码
 */
int graphdb_errcode(graphdb_t *db);

/**
 * 获取错误信息
 *
 * @param db 数据库句柄
 * @return 错误信息字符串（UTF-8 编码），如果无错误返回 NULL
 */
const char *graphdb_errmsg(graphdb_t *db);

/**
 * 获取库版本
 *
 * @return 版本字符串
 */
const char *graphdb_libversion(void);

/* ==================== 会话管理 API ==================== */

/**
 * 创建会话
 *
 * @param db 数据库句柄
 * @param session 输出参数，会话句柄
 * @return 错误码，GRAPHDB_OK 表示成功
 */
int graphdb_session_create(graphdb_t *db, graphdb_session_t **session);

/**
 * 关闭会话
 *
 * @param session 会话句柄
 * @return 错误码，GRAPHDB_OK 表示成功
 */
int graphdb_session_close(graphdb_session_t *session);

/**
 * 切换图空间
 *
 * @param session 会话句柄
 * @param space_name 图空间名称（UTF-8 编码）
 * @return 错误码，GRAPHDB_OK 表示成功
 */
int graphdb_session_use_space(graphdb_session_t *session, const char *space_name);

/**
 * 获取当前图空间
 *
 * @param session 会话句柄
 * @return 当前图空间名称（UTF-8 编码），如果没有则返回 NULL
 */
const char *graphdb_session_current_space(graphdb_session_t *session);

/**
 * 设置自动提交模式
 *
 * @param session 会话句柄
 * @param autocommit 是否自动提交
 * @return 错误码，GRAPHDB_OK 表示成功
 */
int graphdb_session_set_autocommit(graphdb_session_t *session, bool autocommit);

/**
 * 获取自动提交模式
 *
 * @param session 会话句柄
 * @return 是否自动提交
 */
bool graphdb_session_get_autocommit(graphdb_session_t *session);

/* ==================== 查询执行 API ==================== */

/**
 * 执行查询
 *
 * @param session 会话句柄
 * @param query 查询语句（UTF-8 编码）
 * @param result 输出参数，结果集句柄
 * @return 错误码，GRAPHDB_OK 表示成功
 */
int graphdb_execute(graphdb_session_t *session, const char *query, graphdb_result_t **result);

/**
 * 释放结果集
 *
 * @param result 结果集句柄
 * @return 错误码，GRAPHDB_OK 表示成功
 */
int graphdb_result_free(graphdb_result_t *result);

/**
 * 获取结果集列数
 *
 * @param result 结果集句柄
 * @return 列数，错误返回 -1
 */
int graphdb_column_count(graphdb_result_t *result);

/**
 * 获取结果集行数
 *
 * @param result 结果集句柄
 * @return 行数，错误返回 -1
 */
int graphdb_row_count(graphdb_result_t *result);

/**
 * 获取列名
 *
 * @param result 结果集句柄
 * @param index 列索引（从 0 开始）
 * @return 列名（UTF-8 编码），错误返回 NULL
 */
const char *graphdb_column_name(graphdb_result_t *result, int index);

/* ==================== 内存管理 API ==================== */

/**
 * 释放字符串（由 GraphDB 分配的字符串）
 *
 * @param str 字符串指针
 */
void graphdb_free_string(char *str);

/**
 * 释放内存（由 GraphDB 分配的内存）
 *
 * @param ptr 内存指针
 */
void graphdb_free(void *ptr);

#ifdef __cplusplus
} /* extern "C" */
#endif

#endif /* GRAPHDB_H */
"#;

    std::fs::write(output_path, header_content)
        .expect("Failed to write fallback header");
}
