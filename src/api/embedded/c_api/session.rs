//! C API 会话管理模块
//!
//! 提供会话的创建、销毁和基本管理功能

use crate::api::embedded::c_api::database::GraphDbHandle;
use crate::api::embedded::c_api::error::{error_code_from_core_error, graphdb_error_code_t};
use crate::api::embedded::c_api::types::{graphdb_t, graphdb_session_t};
use crate::api::embedded::Session;
use crate::storage::RedbStorage;
use std::ffi::{CStr, CString, c_char, c_int};
use std::ptr;

/// 会话句柄内部结构
pub struct GraphDbSessionHandle {
    pub(crate) inner: Session<RedbStorage>,
    pub(crate) last_error: Option<CString>,
}

/// 创建会话
///
/// # 参数
/// - `db`: 数据库句柄
/// - `session`: 输出参数，会话句柄
///
/// # 返回
/// - 成功: GRAPHDB_OK
/// - 失败: 错误码
#[no_mangle]
pub extern "C" fn graphdb_session_create(
    db: *mut graphdb_t,
    session: *mut *mut graphdb_session_t,
) -> c_int {
    if db.is_null() || session.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    unsafe {
        let db_handle = &*(db as *mut GraphDbHandle);
        
        match db_handle.inner.session() {
            Ok(sess) => {
                let handle = Box::new(GraphDbSessionHandle {
                    inner: sess,
                    last_error: None,
                });
                *session = Box::into_raw(handle) as *mut graphdb_session_t;
                graphdb_error_code_t::GRAPHDB_OK as c_int
            }
            Err(e) => {
                let error_code = error_code_from_core_error(&e);
                *session = ptr::null_mut();
                error_code
            }
        }
    }
}

/// 关闭会话
///
/// # 参数
/// - `session`: 会话句柄
///
/// # 返回
/// - 成功: GRAPHDB_OK
/// - 失败: 错误码
#[no_mangle]
pub extern "C" fn graphdb_session_close(session: *mut graphdb_session_t) -> c_int {
    if session.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    unsafe {
        let _ = Box::from_raw(session as *mut GraphDbSessionHandle);
    }

    graphdb_error_code_t::GRAPHDB_OK as c_int
}

/// 切换图空间
///
/// # 参数
/// - `session`: 会话句柄
/// - `space_name`: 图空间名称（UTF-8 编码）
///
/// # 返回
/// - 成功: GRAPHDB_OK
/// - 失败: 错误码
#[no_mangle]
pub extern "C" fn graphdb_session_use_space(
    session: *mut graphdb_session_t,
    space_name: *const c_char,
) -> c_int {
    if session.is_null() || space_name.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    let name_str = unsafe {
        match CStr::from_ptr(space_name).to_str() {
            Ok(s) => s,
            Err(_) => return graphdb_error_code_t::GRAPHDB_MISUSE as c_int,
        }
    };

    unsafe {
        let handle = &mut *(session as *mut GraphDbSessionHandle);
        
        match handle.inner.use_space(name_str) {
            Ok(_) => graphdb_error_code_t::GRAPHDB_OK as c_int,
            Err(e) => {
                let error_code = error_code_from_core_error(&e);
                handle.last_error = Some(CString::new(format!("{}", e)).unwrap_or_default());
                error_code
            }
        }
    }
}

/// 获取当前图空间
///
/// # 参数
/// - `session`: 会话句柄
///
/// # 返回
/// - 当前图空间名称（UTF-8 编码），如果没有则返回 NULL
#[no_mangle]
pub extern "C" fn graphdb_session_current_space(
    session: *mut graphdb_session_t,
) -> *const c_char {
    if session.is_null() {
        return ptr::null();
    }

    unsafe {
        let handle = &*(session as *mut GraphDbSessionHandle);
        
        match handle.inner.current_space() {
            Some(name) => {
                // 注意：这里返回的字符串生命周期与 session 绑定
                // 调用者不应释放此字符串
                match CString::new(name) {
                    Ok(c_name) => {
                        // 将 CString 转换为原始指针并泄漏它
                        // 调用者需要使用 graphdb_free_string 释放
                        c_name.into_raw()
                    }
                    Err(_) => ptr::null(),
                }
            }
            None => ptr::null(),
        }
    }
}

/// 设置自动提交模式
///
/// # 参数
/// - `session`: 会话句柄
/// - `autocommit`: 是否自动提交
///
/// # 返回
/// - 成功: GRAPHDB_OK
/// - 失败: 错误码
#[no_mangle]
pub extern "C" fn graphdb_session_set_autocommit(
    session: *mut graphdb_session_t,
    autocommit: bool,
) -> c_int {
    if session.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    unsafe {
        let handle = &mut *(session as *mut GraphDbSessionHandle);
        handle.inner.set_auto_commit(autocommit);
        graphdb_error_code_t::GRAPHDB_OK as c_int
    }
}

/// 获取自动提交模式
///
/// # 参数
/// - `session`: 会话句柄
///
/// # 返回
/// - 是否自动提交
#[no_mangle]
pub extern "C" fn graphdb_session_get_autocommit(session: *mut graphdb_session_t) -> bool {
    if session.is_null() {
        return true; // 默认自动提交
    }

    unsafe {
        let handle = &*(session as *mut GraphDbSessionHandle);
        handle.inner.auto_commit()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::embedded::c_api::database::{graphdb_close, graphdb_open};
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn create_test_db() -> *mut graphdb_t {
        let counter = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let temp_dir = std::env::temp_dir().join("graphdb_c_api_test");
        std::fs::create_dir_all(&temp_dir).ok();
        let db_path = temp_dir.join(format!("test_{}_{}.db", std::process::id(), counter));
        
        let path_cstring = CString::new(db_path.to_str().unwrap()).unwrap();
        let mut db: *mut graphdb_t = ptr::null_mut();
        
        let rc = graphdb_open(path_cstring.as_ptr(), &mut db);
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_OK as c_int);
        assert!(!db.is_null());
        
        db
    }

    #[test]
    fn test_session_create_close() {
        let db = create_test_db();
        let mut session: *mut graphdb_session_t = ptr::null_mut();

        // 创建会话
        let rc = graphdb_session_create(db, &mut session);
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_OK as c_int);
        assert!(!session.is_null());

        // 关闭会话
        let rc = graphdb_session_close(session);
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_OK as c_int);

        // 关闭数据库
        let rc = graphdb_close(db);
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_OK as c_int);
    }

    #[test]
    fn test_session_autocommit() {
        let db = create_test_db();
        let mut session: *mut graphdb_session_t = ptr::null_mut();

        let rc = graphdb_session_create(db, &mut session);
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_OK as c_int);

        // 默认自动提交
        let autocommit = graphdb_session_get_autocommit(session);
        assert!(autocommit);

        // 关闭自动提交
        let rc = graphdb_session_set_autocommit(session, false);
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_OK as c_int);

        let autocommit = graphdb_session_get_autocommit(session);
        assert!(!autocommit);

        graphdb_session_close(session);
        graphdb_close(db);
    }
}
