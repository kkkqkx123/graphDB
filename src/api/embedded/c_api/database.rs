//! C API 数据库管理模块
//!
//! 提供数据库的打开、关闭和基本管理功能

use crate::api::embedded::c_api::error::{error_code_from_core_error, graphdb_error_code_t};
use crate::api::embedded::c_api::types::graphdb_t;
use crate::api::embedded::GraphDatabase;
use crate::storage::RedbStorage;
use std::ffi::{CStr, CString, c_char, c_int, c_void};
use std::ptr;
use std::sync::Arc;

/// 数据库句柄内部结构
pub struct GraphDbHandle {
    pub(crate) inner: Arc<GraphDatabase<RedbStorage>>,
    pub(crate) last_error: Option<CString>,
}

/// 打开数据库
///
/// # 参数
/// - `path`: 数据库文件路径（UTF-8 编码）
/// - `db`: 输出参数，数据库句柄
///
/// # 返回
/// - 成功: GRAPHDB_OK
/// - 失败: 错误码
#[no_mangle]
pub extern "C" fn graphdb_open(path: *const c_char, db: *mut *mut graphdb_t) -> c_int {
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
            let error_code = error_code_from_core_error(&e);
            unsafe {
                // 创建一个带错误信息的句柄
                let handle = Box::new(GraphDbHandle {
                    inner: Arc::new(GraphDatabase::open_in_memory().unwrap_or_else(|_| {
                        // 如果内存数据库也失败，创建一个空句柄
                        panic!("无法创建数据库句柄")
                    })),
                    last_error: Some(CString::new(format!("{}", e)).unwrap_or_default()),
                });
                *db = Box::into_raw(handle) as *mut graphdb_t;
            }
            error_code
        }
    }
}

/// 打开内存数据库
///
/// # 参数
/// - `db`: 输出参数，数据库句柄
///
/// # 返回
/// - 成功: GRAPHDB_OK
/// - 失败: 错误码
#[no_mangle]
pub extern "C" fn graphdb_open_memory(db: *mut *mut graphdb_t) -> c_int {
    if db.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    match GraphDatabase::open_in_memory() {
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
            let error_code = error_code_from_core_error(&e);
            unsafe {
                *db = ptr::null_mut();
            }
            error_code
        }
    }
}

/// 关闭数据库
///
/// # 参数
/// - `db`: 数据库句柄
///
/// # 返回
/// - 成功: GRAPHDB_OK
/// - 失败: 错误码
#[no_mangle]
pub extern "C" fn graphdb_close(db: *mut graphdb_t) -> c_int {
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
/// # 参数
/// - `db`: 数据库句柄
///
/// # 返回
/// - 错误码，如果没有错误返回 GRAPHDB_OK
#[no_mangle]
pub extern "C" fn graphdb_errcode(db: *mut graphdb_t) -> c_int {
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
/// # 参数
/// - `str`: 字符串指针
#[no_mangle]
pub extern "C" fn graphdb_free_string(str: *mut c_char) {
    if !str.is_null() {
        unsafe {
            let _ = CString::from_raw(str);
        }
    }
}

/// 释放内存（由 GraphDB 分配的内存）
///
/// # 参数
/// - `ptr`: 内存指针
#[no_mangle]
pub extern "C" fn graphdb_free(ptr: *mut c_void) {
    if !ptr.is_null() {
        unsafe {
            let _ = Box::from_raw(ptr);
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
        temp_dir.join(format!("test_{}_{}.db", std::process::id(), counter))
    }

    #[test]
    fn test_graphdb_libversion() {
        let version = unsafe {
            CStr::from_ptr(graphdb_libversion())
                .to_str()
                .unwrap()
        };
        assert!(!version.is_empty());
    }

    #[test]
    fn test_graphdb_open_close_file() {
        let db_path = get_test_db_path();
        let path_cstring = CString::new(db_path.to_str().unwrap()).unwrap();
        let mut db: *mut graphdb_t = ptr::null_mut();
        
        let rc = graphdb_open(path_cstring.as_ptr(), &mut db);
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_OK as c_int);
        assert!(!db.is_null());

        let rc = graphdb_close(db);
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_OK as c_int);
    }

    #[test]
    fn test_graphdb_null_params() {
        let rc = graphdb_open(ptr::null(), ptr::null_mut());
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_MISUSE as c_int);

        let rc = graphdb_open_memory(ptr::null_mut());
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_MISUSE as c_int);

        let rc = graphdb_close(ptr::null_mut());
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_MISUSE as c_int);
    }
}
