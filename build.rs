//! Build script
//!
//! Used to generate C API header files and configure the build environment

use std::env;
use std::path::PathBuf;

fn main() {
    // Only generate header file when c_api feature is enabled
    if env::var("CARGO_FEATURE_C_API").is_ok() {
        generate_c_header();
    }

    // Set link parameters
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/api/embedded/c_api/");
}

/// Generate C header file
fn generate_c_header() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let output_path = PathBuf::from(&crate_dir).join("include").join("graphdb.h");

    std::fs::create_dir_all(output_path.parent().expect("Output path should have a parent directory"))
        .expect("Failed to create include directory");

    // Attempt to generate header file using cbindgen
    match try_cbindgen(&crate_dir, &output_path) {
        Ok(_) => println!("cargo:warning=Generated C header at {:?}", output_path),
        Err(e) => {
            println!(
                "cargo:warning=Failed to generate C header with cbindgen: {}",
                e
            );
            // If cbindgen fails, use fallback solution
            generate_fallback_header(&output_path);
        }
    }
}

/// Attempt to generate header file using cbindgen
fn try_cbindgen(crate_dir: &str, output_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = PathBuf::from(crate_dir).join("cbindgen.toml");

    // Check if cbindgen config exists
    if !config_path.exists() {
        return Err("cbindgen.toml not found".into());
    }

    // Generate header file using cbindgen
    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_config(cbindgen::Config::from_root_or_default(crate_dir))
        .generate()
        .map_err(|e| format!("cbindgen generation failed: {}", e))?
        .write_to_file(output_path);

    Ok(())
}

/// Generate fallback header file (used when cbindgen fails)
fn generate_fallback_header(output_path: &PathBuf) {
    let header_content = r#"/**
 * GraphDB C API
 *
 * GraphDB C API Header File
 * Provides C language interface for GraphDB
 *
 * Version: 0.1.0
 * License: Apache-2.0
 *
 * For more information, visit: https://github.com/kkkqkx123/graphDB
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

/* ==================== Error Code Definitions ==================== */

/**
 * Error code enumeration
 */
typedef enum {
    GRAPHDB_OK = 0,           /**< Success */
    GRAPHDB_ERROR = 1,        /**< General error */
    GRAPHDB_INTERNAL = 2,     /**< Internal error */
    GRAPHDB_PERM = 3,         /**< Permission denied */
    GRAPHDB_ABORT = 4,        /**< Operation aborted */
    GRAPHDB_BUSY = 5,         /**< Database busy */
    GRAPHDB_LOCKED = 6,       /**< Database locked */
    GRAPHDB_NOMEM = 7,        /**< Out of memory */
    GRAPHDB_READONLY = 8,     /**< Read-only */
    GRAPHDB_INTERRUPT = 9,    /**< Operation interrupted */
    GRAPHDB_IOERR = 10,       /**< I/O error */
    GRAPHDB_CORRUPT = 11,     /**< Data corruption */
    GRAPHDB_NOTFOUND = 12,    /**< Not found */
    GRAPHDB_FULL = 13,        /**< Disk full */
    GRAPHDB_CANTOPEN = 14,    /**< Cannot open */
    GRAPHDB_PROTOCOL = 15,    /**< Protocol error */
    GRAPHDB_SCHEMA = 16,      /**< Schema error */
    GRAPHDB_TOOBIG = 17,      /**< Data too large */
    GRAPHDB_CONSTRAINT = 18,  /**< Constraint violation */
    GRAPHDB_MISMATCH = 19,    /**< Type mismatch */
    GRAPHDB_MISUSE = 20,      /**< API misuse */
    GRAPHDB_RANGE = 21,       /**< Out of range */
} graphdb_error_code_t;

/* ==================== Type Definitions ==================== */

/**
 * Database handle (opaque pointer)
 */
typedef struct graphdb_t graphdb_t;

/**
 * Session handle (opaque pointer)
 */
typedef struct graphdb_session_t graphdb_session_t;

/**
 * Transaction handle (opaque pointer)
 */
typedef struct graphdb_txn_t graphdb_txn_t;

/**
 * Result set handle (opaque pointer)
 */
typedef struct graphdb_result_t graphdb_result_t;

/**
 * Value type enumeration
 */
typedef enum {
    GRAPHDB_NULL = 0,     /**< Null value */
    GRAPHDB_BOOL = 1,     /**< Boolean value */
    GRAPHDB_INT = 2,      /**< Integer */
    GRAPHDB_FLOAT = 3,    /**< Floating point number */
    GRAPHDB_STRING = 4,   /**< String */
    GRAPHDB_LIST = 5,     /**< List */
    GRAPHDB_MAP = 6,      /**< Map */
    GRAPHDB_VERTEX = 7,   /**< Vertex */
    GRAPHDB_EDGE = 8,     /**< Edge */
    GRAPHDB_PATH = 9,     /**< Path */
} graphdb_value_type_t;

/**
 * Value structure
 */
typedef struct {
    graphdb_value_type_t type;  /**< Value type */
    union {
        bool boolean;           /**< Boolean value */
        int64_t integer;        /**< Integer */
        double floating;        /**< Floating point number */
        struct {
            const char *data;   /**< String data */
            size_t len;         /**< String length */
        } string;               /**< String */
        void *ptr;              /**< Pointer for other types */
    } data;                     /**< Value data */
} graphdb_value_t;

/**
 * Database configuration structure
 */
typedef struct {
    bool read_only;             /**< Read-only mode */
    bool create_if_missing;     /**< Create if missing */
    int cache_size_mb;          /**< Cache size (MB) */
    int max_open_files;         /**< Maximum number of open files */
    bool enable_compression;    /**< Enable compression */
} graphdb_config_t;

/* ==================== Database Management API ==================== */

/**
 * Open database
 *
 * @param path Database file path (UTF-8 encoded)
 * @param db Output parameter, database handle
 * @return Error code, GRAPHDB_OK indicates success
 */
int graphdb_open(const char *path, graphdb_t **db);

/**
 * Open in-memory database
 *
 * @param db Output parameter, database handle
 * @return Error code, GRAPHDB_OK indicates success
 */
int graphdb_open_memory(graphdb_t **db);

/**
 * Open database with configuration
 *
 * @param path Database file path (UTF-8 encoded)
 * @param config Database configuration
 * @param db Output parameter, database handle
 * @return Error code, GRAPHDB_OK indicates success
 */
int graphdb_open_config(const char *path, const graphdb_config_t *config, graphdb_t **db);

/**
 * Close database
 *
 * @param db Database handle
 * @return Error code, GRAPHDB_OK indicates success
 */
int graphdb_close(graphdb_t *db);

/**
 * Get error code
 *
 * @param db Database handle
 * @return Error code
 */
int graphdb_errcode(graphdb_t *db);

/**
 * Get error message
 *
 * @param db Database handle
 * @return Error message string (UTF-8 encoded), returns NULL if no error
 */
const char *graphdb_errmsg(graphdb_t *db);

/**
 * Get library version
 *
 * @return Version string
 */
const char *graphdb_libversion(void);

/* ==================== Session Management API ==================== */

/**
 * Create session
 *
 * @param db Database handle
 * @param session Output parameter, session handle
 * @return Error code, GRAPHDB_OK indicates success
 */
int graphdb_session_create(graphdb_t *db, graphdb_session_t **session);

/**
 * Close session
 *
 * @param session Session handle
 * @return Error code, GRAPHDB_OK indicates success
 */
int graphdb_session_close(graphdb_session_t *session);

/**
 * Switch graph space
 *
 * @param session Session handle
 * @param space_name Graph space name (UTF-8 encoded)
 * @return Error code, GRAPHDB_OK indicates success
 */
int graphdb_session_use_space(graphdb_session_t *session, const char *space_name);

/**
 * Get current graph space
 *
 * @param session Session handle
 * @return Current graph space name (UTF-8 encoded), returns NULL if none
 */
const char *graphdb_session_current_space(graphdb_session_t *session);

/**
 * Set auto-commit mode
 *
 * @param session Session handle
 * @param autocommit Whether to auto-commit
 * @return Error code, GRAPHDB_OK indicates success
 */
int graphdb_session_set_autocommit(graphdb_session_t *session, bool autocommit);

/**
 * Get auto-commit mode
 *
 * @param session Session handle
 * @return Whether auto-commit is enabled
 */
bool graphdb_session_get_autocommit(graphdb_session_t *session);

/* ==================== Query Execution API ==================== */

/**
 * Execute query
 *
 * @param session Session handle
 * @param query Query statement (UTF-8 encoded)
 * @param result Output parameter, result set handle
 * @return Error code, GRAPHDB_OK indicates success
 */
int graphdb_execute(graphdb_session_t *session, const char *query, graphdb_result_t **result);

/**
 * Free result set
 *
 * @param result Result set handle
 * @return Error code, GRAPHDB_OK indicates success
 */
int graphdb_result_free(graphdb_result_t *result);

/**
 * Get number of columns in result set
 *
 * @param result Result set handle
 * @return Number of columns, returns -1 on error
 */
int graphdb_column_count(graphdb_result_t *result);

/**
 * Get number of rows in result set
 *
 * @param result Result set handle
 * @return Number of rows, returns -1 on error
 */
int graphdb_row_count(graphdb_result_t *result);

/**
 * Get column name
 *
 * @param result Result set handle
 * @param index Column index (starting from 0)
 * @return Column name (UTF-8 encoded), returns NULL on error
 */
const char *graphdb_column_name(graphdb_result_t *result, int index);

/* ==================== Memory Management API ==================== */

/**
 * Free string (string allocated by GraphDB)
 *
 * @param str String pointer
 */
void graphdb_free_string(char *str);

/**
 * Free memory (memory allocated by GraphDB)
 *
 * @param ptr Memory pointer
 */
void graphdb_free(void *ptr);

#ifdef __cplusplus
} /* extern "C" */
#endif

#endif /* GRAPHDB_H */
"#;

    std::fs::write(output_path, header_content).expect("Failed to write fallback header");
}