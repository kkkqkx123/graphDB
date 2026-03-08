//! C API 会话管理模块
//!
//! 提供会话的创建、销毁和基本管理功能

use crate::api::embedded::c_api::database::GraphDbHandle;
use crate::api::embedded::c_api::error::{error_code_from_core_error, graphdb_error_code_t, set_last_error_message};
use crate::api::embedded::c_api::types::{graphdb_t, graphdb_session_t};
use crate::api::embedded::Session;
use crate::storage::RedbStorage;
use std::ffi::{CStr, CString, c_char, c_int};
use std::ptr;

/// 会话句柄内部结构
pub struct GraphDbSessionHandle {
    pub(crate) inner: Session<RedbStorage>,
    pub(crate) last_error: Option<CString>,
    /// 忙等待超时（毫秒）
    pub(crate) busy_timeout_ms: u32,
}

impl GraphDbSessionHandle {
    /// 创建新的会话句柄
    pub(crate) fn new(inner: Session<RedbStorage>) -> Self {
        Self {
            inner,
            last_error: None,
            busy_timeout_ms: 5000, // 默认 5 秒
        }
    }
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
                let handle = Box::new(GraphDbSessionHandle::new(sess));
                *session = Box::into_raw(handle) as *mut graphdb_session_t;
                graphdb_error_code_t::GRAPHDB_OK as c_int
            }
            Err(e) => {
                let error_code = error_code_from_core_error(&e);
                let error_msg = format!("{}", e);
                set_last_error_message(error_msg);
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
                let error_msg = format!("{}", e);
                set_last_error_message(error_msg.clone());
                handle.last_error = Some(CString::new(error_msg).unwrap_or_default());
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

/// 获取上次操作影响的行数
///
/// # 参数
/// - `session`: 会话句柄
///
/// # 返回
/// - 影响的行数，如果会话无效则返回 0
#[no_mangle]
pub extern "C" fn graphdb_changes(session: *mut graphdb_session_t) -> c_int {
    if session.is_null() {
        return 0;
    }

    unsafe {
        let handle = &*(session as *mut GraphDbSessionHandle);
        handle.inner.changes() as c_int
    }
}

/// 获取总会话变更数
///
/// # 参数
/// - `session`: 会话句柄
///
/// # 返回
/// - 总会话变更数，如果会话无效则返回 0
#[no_mangle]
pub extern "C" fn graphdb_total_changes(session: *mut graphdb_session_t) -> i64 {
    if session.is_null() {
        return 0;
    }

    unsafe {
        let handle = &*(session as *mut GraphDbSessionHandle);
        handle.inner.total_changes() as i64
    }
}

/// 获取最后插入的顶点 ID
///
/// # 参数
/// - `session`: 会话句柄
///
/// # 返回
/// - 最后插入的顶点 ID，如果没有则返回 -1
#[no_mangle]
pub extern "C" fn graphdb_last_insert_vertex_id(
    session: *mut graphdb_session_t,
) -> i64 {
    if session.is_null() {
        return -1;
    }

    unsafe {
        let handle = &*(session as *mut GraphDbSessionHandle);
        handle.inner.last_insert_vertex_id().unwrap_or(-1)
    }
}

/// 获取最后插入的边 ID
///
/// # 参数
/// - `session`: 会话句柄
///
/// # 返回
/// - 最后插入的边 ID，如果没有则返回 -1
#[no_mangle]
pub extern "C" fn graphdb_last_insert_edge_id(
    session: *mut graphdb_session_t,
) -> i64 {
    if session.is_null() {
        return -1;
    }

    unsafe {
        let handle = &*(session as *mut GraphDbSessionHandle);
        handle.inner.last_insert_edge_id().unwrap_or(-1)
    }
}

/// 设置忙等待超时
///
/// # 参数
/// - `session`: 会话句柄
/// - `timeout_ms`: 超时时间（毫秒），0 表示不等待
///
/// # 返回
/// - 成功: GRAPHDB_OK
/// - 失败: 错误码
#[no_mangle]
pub extern "C" fn graphdb_busy_timeout(
    session: *mut graphdb_session_t,
    timeout_ms: c_int,
) -> c_int {
    if session.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    unsafe {
        let handle = &mut *(session as *mut GraphDbSessionHandle);
        // 存储超时设置到句柄中
        handle.busy_timeout_ms = timeout_ms.max(0) as u32;
        graphdb_error_code_t::GRAPHDB_OK as c_int
    }
}

/// 获取忙等待超时
///
/// # 参数
/// - `session`: 会话句柄
///
/// # 返回
/// - 超时时间（毫秒），如果会话无效则返回 0
#[no_mangle]
pub extern "C" fn graphdb_busy_timeout_get(
    session: *mut graphdb_session_t,
) -> c_int {
    if session.is_null() {
        return 0;
    }

    unsafe {
        let handle = &*(session as *mut GraphDbSessionHandle);
        handle.busy_timeout_ms as c_int
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
        let db_path = temp_dir.join(format!("test_session_{}_{}.db", std::process::id(), counter));
        
        // 确保数据库文件不存在
        if db_path.exists() {
            std::fs::remove_file(&db_path).ok();
            // 等待文件系统完成删除操作
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        
        let path_cstring = CString::new(db_path.to_str().unwrap()).unwrap();
        let mut db: *mut graphdb_t = ptr::null_mut();
        
        let rc = graphdb_open(path_cstring.as_ptr(), &mut db);
        if rc != graphdb_error_code_t::GRAPHDB_OK as c_int {
            panic!("打开数据库失败，错误码: {}, 路径: {:?}", rc, db_path);
        }
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
