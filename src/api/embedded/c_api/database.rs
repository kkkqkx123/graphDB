//! C API 数据库管理模块
//!
//! 提供数据库的打开、关闭和基本管理功能

use crate::api::embedded::c_api::error::{
    error_code_from_core_error, graphdb_error_code_t, set_last_error_message,
};
use crate::api::embedded::c_api::types::{
    graphdb_t, GRAPHDB_OPEN_CREATE, GRAPHDB_OPEN_READONLY, GRAPHDB_OPEN_READWRITE,
};
use crate::api::embedded::{DatabaseConfig, GraphDatabase};
use crate::storage::RedbStorage;
use std::ffi::{c_char, c_int, c_void, CStr, CString};
use std::ptr;
use std::sync::Arc;

/// 数据库句柄内部结构
pub struct GraphDbHandle {
    pub(crate) inner: Arc<GraphDatabase<RedbStorage>>,
    pub(crate) last_error: Option<CString>,
}

/// 打开数据库
///
/// # Arguments
/// - `path`: Database file path (UTF-8 encoded)
/// - `db`: Output parameter, database handle
///
/// # Returns
/// - Success: GRAPHDB_OK
/// - Failure: Error code
///
/// # Safety
/// - `path` must be a valid pointer to a null-terminated UTF-8 string
/// - `db` must be a valid pointer to store the database handle
/// - The caller is responsible for closing the database using `graphdb_close` when done
/// - The database handle must not be used after closing
#[no_mangle]
pub unsafe extern "C" fn graphdb_open(path: *const c_char, db: *mut *mut graphdb_t) -> c_int {
    // 参数验证
    if path.is_null() || db.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    // 转换路径字符串
    let path_str = unsafe {
        match CStr::from_ptr(path).to_str() {
            Ok(s) => s,
            Err(_) => return graphdb_error_code_t::GRAPHDB_MISUSE as c_int,
        }
    };

    // 打开数据库
    match GraphDatabase::open(path_str) {
        Ok(graphdb) => {
            let handle = Box::new(GraphDbHandle {
                inner: Arc::new(graphdb),
                last_error: None,
            });
            unsafe {
                *db = Box::into_raw(handle) as *mut graphdb_t;
            }
            graphdb_error_code_t::GRAPHDB_OK as c_int
        }
        Err(e) => {
            let (error_code, _) = error_code_from_core_error(&e);
            let error_msg = format!("{}", e);
            set_last_error_message(error_msg);
            unsafe {
                *db = ptr::null_mut();
            }
            error_code
        }
    }
}

/// 使用标志打开数据库
///
/// # Arguments
/// - `path`: Database file path (UTF-8 encoded)
/// - `db`: Output parameter, database handle
/// - `flags`: Open flags
/// - `vfs`: VFS name (reserved parameter, currently unused, can be NULL)
///
/// # Returns
/// - Success: GRAPHDB_OK
/// - Failure: Error code
///
/// # Flags
/// - GRAPHDB_OPEN_READONLY: Read-only mode
/// - GRAPHDB_OPEN_READWRITE: Read-write mode
/// - GRAPHDB_OPEN_CREATE: Create database if it doesn't exist
///
/// # Safety
/// - `path` must be a valid pointer to a null-terminated UTF-8 string
/// - `db` must be a valid pointer to store the database handle
/// - The caller is responsible for closing the database using `graphdb_close` when done
/// - The database handle must not be used after closing
#[no_mangle]
pub unsafe extern "C" fn graphdb_open_v2(
    path: *const c_char,
    db: *mut *mut graphdb_t,
    flags: c_int,
    _vfs: *const c_char,
) -> c_int {
    // 参数验证（vfs 可以为 NULL）
    if path.is_null() || db.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    // 转换路径字符串
    let path_str = unsafe {
        match CStr::from_ptr(path).to_str() {
            Ok(s) => s,
            Err(_) => return graphdb_error_code_t::GRAPHDB_MISUSE as c_int,
        }
    };

    // 解析标志
    let read_only = (flags & GRAPHDB_OPEN_READONLY) != 0;
    let read_write = (flags & GRAPHDB_OPEN_READWRITE) != 0;
    let create = (flags & GRAPHDB_OPEN_CREATE) != 0;

    // 验证标志组合
    if read_only && read_write {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    // 构建配置
    let mut config = if read_only {
        DatabaseConfig::file(path_str).with_read_only(true)
    } else {
        DatabaseConfig::file(path_str)
    };

    if create {
        config = config.with_create_if_missing(true);
    }

    // 打开数据库
    match GraphDatabase::open_with_config(config) {
        Ok(graphdb) => {
            let handle = Box::new(GraphDbHandle {
                inner: Arc::new(graphdb),
                last_error: None,
            });
            unsafe {
                *db = Box::into_raw(handle) as *mut graphdb_t;
            }
            graphdb_error_code_t::GRAPHDB_OK as c_int
        }
        Err(e) => {
            let (error_code, _) = error_code_from_core_error(&e);
            let error_msg = format!("{}", e);
            set_last_error_message(error_msg);
            unsafe {
                *db = ptr::null_mut();
            }
            error_code
        }
    }
}

/// 关闭数据库
///
/// # Arguments
/// - `db`: Database handle
///
/// # Returns
/// - Success: GRAPHDB_OK
/// - Failure: Error code
///
/// # Safety
/// - `db` must be a valid database handle created by `graphdb_open` or `graphdb_open_v2`
/// - After calling this function, the database handle becomes invalid and must not be used
/// - All sessions associated with this database must be closed before calling this function
#[no_mangle]
pub unsafe extern "C" fn graphdb_close(db: *mut graphdb_t) -> c_int {
    if db.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    unsafe {
        // 将原始指针转换回 Box，在函数结束时自动释放
        let _ = Box::from_raw(db as *mut GraphDbHandle);
    }

    graphdb_error_code_t::GRAPHDB_OK as c_int
}

/// 获取错误码
///
/// # Arguments
/// - `db`: Database handle
///
/// # Returns
/// - Error code, returns GRAPHDB_OK if no error
///
/// # Safety
/// - `db` must be a valid database handle created by `graphdb_open` or `graphdb_open_v2`
#[no_mangle]
pub unsafe extern "C" fn graphdb_errcode(db: *mut graphdb_t) -> c_int {
    if db.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    unsafe {
        let handle = &*(db as *mut GraphDbHandle);
        if handle.last_error.is_some() {
            graphdb_error_code_t::GRAPHDB_ERROR as c_int
        } else {
            graphdb_error_code_t::GRAPHDB_OK as c_int
        }
    }
}

/// 获取库版本
///
/// # 返回
/// - 版本字符串
#[no_mangle]
pub extern "C" fn graphdb_libversion() -> *const c_char {
    static VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), "\0");
    VERSION.as_ptr() as *const c_char
}

/// 释放字符串（由 GraphDB 分配的字符串）
///
/// # Arguments
/// - `str`: String pointer
///
/// # Safety
/// - `str` must be a valid pointer to a string allocated by GraphDB
/// - After calling this function, the pointer becomes invalid and must not be used
/// - This function should only be called on strings that were allocated by GraphDB C API functions
#[no_mangle]
pub unsafe extern "C" fn graphdb_free_string(str: *mut c_char) {
    if !str.is_null() {
        unsafe {
            let _ = CString::from_raw(str);
        }
    }
}

/// 释放内存（由 GraphDB 分配的内存）
///
/// # Arguments
/// - `ptr`: Memory pointer
///
/// # Safety
/// - `ptr` must be a valid pointer to memory allocated by GraphDB
/// - After calling this function, the pointer becomes invalid and must not be used
/// - This function should only be called on memory that was allocated by GraphDB C API functions
#[no_mangle]
pub unsafe extern "C" fn graphdb_free(ptr: *mut c_void) {
    if !ptr.is_null() {
        unsafe {
            let _ = Box::from_raw(ptr as *mut u8);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn get_test_db_path() -> std::path::PathBuf {
        let counter = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let temp_dir = std::env::temp_dir().join("graphdb_c_api_test");
        std::fs::create_dir_all(&temp_dir).ok();
        let db_path = temp_dir.join(format!("test_db_{}_{}.db", std::process::id(), counter));

        // 确保数据库文件不存在
        if db_path.exists() {
            std::fs::remove_file(&db_path).ok();
            // 等待文件系统完成删除操作
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        db_path
    }

    #[test]
    fn test_graphdb_libversion() {
        let version = unsafe {
            CStr::from_ptr(graphdb_libversion())
                .to_str()
                .expect("Failed to convert version to str")
        };
        assert!(!version.is_empty());
    }

    #[test]
    fn test_graphdb_open_close_file() {
        let db_path = get_test_db_path();

        let path_cstring = CString::new(db_path.to_str().expect("Invalid path"))
            .expect("Failed to create CString");
        let mut db: *mut graphdb_t = ptr::null_mut();

        let rc = unsafe { graphdb_open(path_cstring.as_ptr(), &mut db) };
        if rc != graphdb_error_code_t::GRAPHDB_OK as c_int {
            panic!("打开数据库失败，错误码: {}, 路径: {:?}", rc, db_path);
        }
        assert!(!db.is_null());

        let rc = unsafe { graphdb_close(db) };
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_OK as c_int);
    }

    #[test]
    fn test_graphdb_null_params() {
        let rc = unsafe { graphdb_open(ptr::null(), ptr::null_mut()) };
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_MISUSE as c_int);

        let rc = unsafe { graphdb_close(ptr::null_mut()) };
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_MISUSE as c_int);
    }
}
