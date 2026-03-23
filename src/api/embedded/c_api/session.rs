//! C API 会话管理模块
//!
//! 提供会话的创建、销毁和基本管理功能

use crate::api::embedded::c_api::database::GraphDbHandle;
use crate::api::embedded::c_api::error::{
    error_code_from_core_error, extended_error_code_from_core_error, graphdb_error_code_t,
    set_last_error_message,
};
use crate::api::embedded::c_api::types::{
    graphdb_commit_hook_callback, graphdb_extended_error_code_t, graphdb_rollback_hook_callback,
    graphdb_session_t, graphdb_t, graphdb_trace_callback, graphdb_update_hook_callback,
};
use crate::api::embedded::Session;
use crate::storage::RedbStorage;
use std::ffi::{c_char, c_int, c_void, CStr, CString};
use std::ptr;

/// 会话句柄内部结构
pub struct GraphDbSessionHandle {
    pub(crate) inner: Session<RedbStorage>,
    pub(crate) last_error: Option<CString>,
    /// 忙等待超时（毫秒）
    pub(crate) busy_timeout_ms: u32,
    /// 最后错误位置偏移量
    pub(crate) last_error_offset: Option<usize>,
    /// 最后扩展错误码
    pub(crate) last_extended_error: Option<graphdb_extended_error_code_t>,
    /// SQL 追踪回调
    pub(crate) trace_callback: graphdb_trace_callback,
    /// SQL 追踪回调用户数据
    pub(crate) trace_user_data: *mut c_void,
    /// 提交钩子回调
    pub(crate) commit_hook: graphdb_commit_hook_callback,
    /// 提交钩子用户数据
    pub(crate) commit_hook_user_data: *mut c_void,
    /// 回滚钩子回调
    pub(crate) rollback_hook: graphdb_rollback_hook_callback,
    /// 回滚钩子用户数据
    pub(crate) rollback_hook_user_data: *mut c_void,
    /// 更新钩子回调
    pub(crate) update_hook: graphdb_update_hook_callback,
    /// 更新钩子用户数据
    pub(crate) update_hook_user_data: *mut c_void,
}

// 手动实现 Send 和 Sync，因为 *mut c_void 不是线程安全的
// 但这里我们只在 C API 层使用，由调用者保证线程安全
unsafe impl Send for GraphDbSessionHandle {}
unsafe impl Sync for GraphDbSessionHandle {}

impl GraphDbSessionHandle {
    /// 创建新的会话句柄
    pub(crate) fn new(inner: Session<RedbStorage>) -> Self {
        Self {
            inner,
            last_error: None,
            busy_timeout_ms: 5000, // 默认 5 秒
            last_error_offset: None,
            last_extended_error: None,
            trace_callback: None,
            trace_user_data: ptr::null_mut(),
            commit_hook: None,
            commit_hook_user_data: ptr::null_mut(),
            rollback_hook: None,
            rollback_hook_user_data: ptr::null_mut(),
            update_hook: None,
            update_hook_user_data: ptr::null_mut(),
        }
    }

    /// 调用更新钩子
    pub(crate) fn invoke_update_hook(&self, operation: i32, space_name: &str, rowid: i64) {
        if let Some(callback) = self.update_hook {
            if let Ok(c_space) = CString::new(space_name) {
                // 对于图数据库，table 参数使用空字符串
                let empty_table = CString::new("").expect("Failed to create empty table CString");
                callback(
                    self.update_hook_user_data,
                    operation,
                    c_space.as_ptr(),
                    empty_table.as_ptr(),
                    rowid,
                );
            }
        }
    }

    /// 调用 SQL 追踪回调
    pub(crate) fn trace(&self, sql: &str) {
        if let Some(callback) = self.trace_callback {
            if let Ok(c_sql) = CString::new(sql) {
                callback(c_sql.as_ptr(), self.trace_user_data);
            }
        }
    }

    /// 设置错误信息
    pub(crate) fn set_error(
        &mut self,
        message: String,
        offset: Option<usize>,
        extended_code: Option<graphdb_extended_error_code_t>,
    ) {
        self.last_error = CString::new(message.clone()).ok();
        self.last_error_offset = offset;
        self.last_extended_error = extended_code;
        set_last_error_message(message);
    }

    /// 清除错误信息
    pub(crate) fn clear_error(&mut self) {
        self.last_error = None;
        self.last_error_offset = None;
        self.last_extended_error = None;
    }
}

/// 创建会话
///
/// # Arguments
/// - `db`: Database handle
/// - `session`: Output parameter, session handle
///
/// # Returns
/// - Success: GRAPHDB_OK
/// - Failure: Error code
///
/// # Safety
/// - `db` must be a valid database handle created by `graphdb_open` or `graphdb_open_v2`
/// - `session` must be a valid pointer to store the session handle
/// - The caller is responsible for closing the session using `graphdb_session_close` when done
/// - The session handle must not be used after closing
#[no_mangle]
pub unsafe extern "C" fn graphdb_session_create(
    db: *mut graphdb_t,
    session: *mut *mut graphdb_session_t,
) -> c_int {
    if db.is_null() || session.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    let db_handle = &*(db as *mut GraphDbHandle);

    match db_handle.inner.session() {
        Ok(sess) => {
            let handle = Box::new(GraphDbSessionHandle::new(sess));
            *session = Box::into_raw(handle) as *mut graphdb_session_t;
            graphdb_error_code_t::GRAPHDB_OK as c_int
        }
        Err(e) => {
            let (error_code, _) = error_code_from_core_error(&e);
            let error_msg = format!("{}", e);
            set_last_error_message(error_msg);
            *session = ptr::null_mut();
            error_code
        }
    }
}

/// 关闭会话
///
/// # Arguments
/// - `session`: Session handle
///
/// # Returns
/// - Success: GRAPHDB_OK
/// - Failure: Error code
///
/// # Safety
/// - `session` must be a valid session handle created by `graphdb_session_create`
/// - After calling this function, the session handle becomes invalid and must not be used
/// - All pending transactions will be rolled back
#[no_mangle]
pub unsafe extern "C" fn graphdb_session_close(session: *mut graphdb_session_t) -> c_int {
    if session.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    let _ = Box::from_raw(session as *mut GraphDbSessionHandle);

    graphdb_error_code_t::GRAPHDB_OK as c_int
}

/// 切换图空间
///
/// # Arguments
/// - `session`: Session handle
/// - `space_name`: Graph space name (UTF-8 encoded)
///
/// # Returns
/// - Success: GRAPHDB_OK
/// - Failure: Error code
///
/// # Safety
/// - `session` must be a valid session handle created by `graphdb_session_create`
/// - `space_name` must be a valid pointer to a null-terminated UTF-8 string
#[no_mangle]
pub unsafe extern "C" fn graphdb_session_use_space(
    session: *mut graphdb_session_t,
    space_name: *const c_char,
) -> c_int {
    if session.is_null() || space_name.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    let name_str = match CStr::from_ptr(space_name).to_str() {
        Ok(s) => s,
        Err(_) => return graphdb_error_code_t::GRAPHDB_MISUSE as c_int,
    };

    let handle = &mut *(session as *mut GraphDbSessionHandle);

    match handle.inner.use_space(name_str) {
        Ok(_) => {
            handle.clear_error();
            graphdb_error_code_t::GRAPHDB_OK as c_int
        }
        Err(e) => {
            let (error_code, _) = error_code_from_core_error(&e);
            let error_msg = format!("{}", e);
            let offset = e.error_offset();
            let extended_code = Some(extended_error_code_from_core_error(&e));
            handle.set_error(error_msg, offset, extended_code);
            error_code
        }
    }
}

/// 获取当前图空间
///
/// # Arguments
/// - `session`: Session handle
///
/// # Returns
/// - Current graph space name (UTF-8 encoded), returns NULL if none
///
/// # Memory Management
/// The returned string is dynamically allocated and must be freed by the caller using `graphdb_free_string`
/// to avoid memory leaks.
///
/// # Example
/// ```c
/// char* space = graphdb_session_current_space(session);
/// if (space) {
///     printf("Current space: %s\n", space);
///     graphdb_free_string(space);  // Must free
/// }
/// ```
///
/// # Safety
/// - `session` must be a valid session handle created by `graphdb_session_create`
/// - The returned pointer must be freed by the caller to avoid memory leaks
#[no_mangle]
pub unsafe extern "C" fn graphdb_session_current_space(session: *mut graphdb_session_t) -> *mut c_char {
    if session.is_null() {
        return ptr::null_mut();
    }

    let handle = &*(session as *mut GraphDbSessionHandle);

    match handle.inner.current_space() {
        Some(name) => {
            match CString::new(name) {
                Ok(c_name) => {
                    // 将 CString 转换为原始指针
                    // 调用者需要使用 graphdb_free_string 释放
                    c_name.into_raw()
                }
                Err(_) => ptr::null_mut(),
            }
        }
        None => ptr::null_mut(),
    }
}

/// 设置自动提交模式
///
/// # Arguments
/// - `session`: Session handle
/// - `autocommit`: Whether to enable autocommit
///
/// # Returns
/// - Success: GRAPHDB_OK
/// - Failure: Error code
///
/// # Safety
/// - `session` must be a valid session handle created by `graphdb_session_create`
#[no_mangle]
pub unsafe extern "C" fn graphdb_session_set_autocommit(
    session: *mut graphdb_session_t,
    autocommit: bool,
) -> c_int {
    if session.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    let handle = &mut *(session as *mut GraphDbSessionHandle);
    handle.inner.set_auto_commit(autocommit);
    graphdb_error_code_t::GRAPHDB_OK as c_int
}

/// 获取自动提交模式
///
/// # Arguments
/// - `session`: Session handle
///
/// # Returns
/// - Whether autocommit is enabled
///
/// # Safety
/// - `session` must be a valid session handle created by `graphdb_session_create`
#[no_mangle]
pub unsafe extern "C" fn graphdb_session_get_autocommit(session: *mut graphdb_session_t) -> bool {
    if session.is_null() {
        return true; // 默认自动提交
    }

    let handle = &*(session as *mut GraphDbSessionHandle);
    handle.inner.auto_commit()
}

/// 获取上次操作影响的行数
///
/// # Arguments
/// - `session`: Session handle
///
/// # Returns
/// - Number of rows affected by last operation, returns 0 if session is invalid
///
/// # Safety
/// - `session` must be a valid session handle created by `graphdb_session_create`
#[no_mangle]
pub unsafe extern "C" fn graphdb_changes(session: *mut graphdb_session_t) -> c_int {
    if session.is_null() {
        return 0;
    }

    let handle = &*(session as *mut GraphDbSessionHandle);
    handle.inner.changes() as c_int
}

/// 获取自数据库打开以来的总变更数量
///
/// # Arguments
/// - `session`: Session handle
///
/// # Returns
/// - Total number of changes
///
/// # Safety
/// - `session` must be a valid session handle created by `graphdb_session_create`
#[no_mangle]
pub unsafe extern "C" fn graphdb_total_changes(session: *mut graphdb_session_t) -> i64 {
    if session.is_null() {
        return 0;
    }

    let handle = &*(session as *mut GraphDbSessionHandle);
    handle.inner.total_changes() as i64
}

/// 获取最后插入的顶点 ID
///
/// # Arguments
/// - `session`: Session handle
///
/// # Returns
/// - Last inserted vertex ID, returns 0 if none
///
/// # Safety
/// - `session` must be a valid session handle created by `graphdb_session_create`
#[no_mangle]
pub unsafe extern "C" fn graphdb_last_insert_vertex_id(session: *mut graphdb_session_t) -> i64 {
    if session.is_null() {
        return -1;
    }

    let handle = &*(session as *mut GraphDbSessionHandle);
    handle.inner.last_insert_vertex_id().unwrap_or(-1)
}

/// 获取最后插入的边 ID
///
/// # Arguments
/// - `session`: Session handle
///
/// # Returns
/// - Last inserted edge ID, returns 0 if none
///
/// # Safety
/// - `session` must be a valid session handle created by `graphdb_session_create`
#[no_mangle]
pub unsafe extern "C" fn graphdb_last_insert_edge_id(session: *mut graphdb_session_t) -> i64 {
    if session.is_null() {
        return -1;
    }

    let handle = &*(session as *mut GraphDbSessionHandle);
    handle.inner.last_insert_edge_id().unwrap_or(-1)
}

/// 设置忙等待超时
///
/// # Arguments
/// - `session`: Session handle
/// - `timeout_ms`: Timeout in milliseconds
///
/// # Returns
/// - Success: GRAPHDB_OK
/// - Failure: Error code
///
/// # Safety
/// - `session` must be a valid session handle created by `graphdb_session_create`
#[no_mangle]
pub unsafe extern "C" fn graphdb_busy_timeout(
    session: *mut graphdb_session_t,
    timeout_ms: c_int,
) -> c_int {
    if session.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    let handle = &mut *(session as *mut GraphDbSessionHandle);
    // 存储超时设置到句柄中
    handle.busy_timeout_ms = timeout_ms.max(0) as u32;
    graphdb_error_code_t::GRAPHDB_OK as c_int
}

/// 获取忙等待超时
///
/// # Arguments
/// - `session`: Session handle
///
/// # Returns
/// - Timeout in milliseconds, returns -1 on error
///
/// # Safety
/// - `session` must be a valid session handle created by `graphdb_session_create`
#[no_mangle]
pub unsafe extern "C" fn graphdb_busy_timeout_get(session: *mut graphdb_session_t) -> c_int {
    if session.is_null() {
        return 0;
    }

    let handle = &*(session as *mut GraphDbSessionHandle);
    handle.busy_timeout_ms as c_int
}

/// 设置 SQL 追踪回调
///
/// # Arguments
/// - `session`: Session handle
/// - `callback`: Trace callback function, NULL to disable tracing
/// - `user_data`: User data pointer, will be passed to the callback
///
/// # Returns
/// - Success: GRAPHDB_OK
/// - Failure: Error code
///
/// # Example
/// ```c
/// extern void my_trace_callback(const char* sql, void* data) {
///     printf("Executing: %s\n", sql);
/// }
///
/// graphdb_trace(session, my_trace_callback, NULL);
/// ```
///
/// # Safety
/// - `session` must be a valid session handle created by `graphdb_session_create`
/// - `callback` must be a valid function pointer, or NULL to disable tracing
/// - `user_data` is passed to the callback and must remain valid for the lifetime of the callback
#[no_mangle]
pub unsafe extern "C" fn graphdb_trace(
    session: *mut graphdb_session_t,
    callback: graphdb_trace_callback,
    user_data: *mut c_void,
) -> c_int {
    if session.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    let handle = &mut *(session as *mut GraphDbSessionHandle);
    handle.trace_callback = callback;
    handle.trace_user_data = user_data;
    graphdb_error_code_t::GRAPHDB_OK as c_int
}

/// 设置提交钩子
///
/// # Arguments
/// - `session`: Session handle
/// - `callback`: Commit hook callback function, NULL to disable the hook
/// - `user_data`: User data pointer, will be passed to the callback
///
/// # Returns
/// - Previous hook user data pointer (if any)
///
/// # Description
/// The commit hook is called before a transaction is committed. If the callback returns a non-zero value,
/// the transaction will be rolled back.
///
/// # Safety
/// - `session` must be a valid session handle created by `graphdb_session_create`
/// - `callback` must be a valid function pointer, or NULL to disable the hook
/// - `user_data` is passed to the callback and must remain valid for the lifetime of the callback
#[no_mangle]
pub unsafe extern "C" fn graphdb_commit_hook(
    session: *mut graphdb_session_t,
    callback: graphdb_commit_hook_callback,
    user_data: *mut c_void,
) -> *mut c_void {
    if session.is_null() {
        return ptr::null_mut();
    }

    let handle = &mut *(session as *mut GraphDbSessionHandle);
    let old_user_data = handle.commit_hook_user_data;
    handle.commit_hook = callback;
    handle.commit_hook_user_data = user_data;
    old_user_data
}

/// 设置回滚钩子
///
/// # Arguments
/// - `session`: Session handle
/// - `callback`: Rollback hook callback function, NULL to disable the hook
/// - `user_data`: User data pointer, will be passed to the callback
///
/// # Returns
/// - Previous hook user data pointer (if any)
///
/// # Safety
/// - `session` must be a valid session handle created by `graphdb_session_create`
/// - `callback` must be a valid function pointer, or NULL to disable the hook
/// - `user_data` is passed to the callback and must remain valid for the lifetime of the callback
#[no_mangle]
pub unsafe extern "C" fn graphdb_rollback_hook(
    session: *mut graphdb_session_t,
    callback: graphdb_rollback_hook_callback,
    user_data: *mut c_void,
) -> *mut c_void {
    if session.is_null() {
        return ptr::null_mut();
    }

    let handle = &mut *(session as *mut GraphDbSessionHandle);
    let old_user_data = handle.rollback_hook_user_data;
    handle.rollback_hook = callback;
    handle.rollback_hook_user_data = user_data;
    old_user_data
}

/// 设置更新钩子
///
/// When data in the database changes, the callback function is called
///
/// # Arguments
/// - `session`: Session handle
/// - `callback`: Update hook callback function, NULL to disable the hook
/// - `user_data`: User data pointer, will be passed to the callback
///
/// # Returns
/// - Previous hook user data pointer (if any)
///
/// # Callback Parameters
/// - `operation`: Operation type (1=INSERT, 2=UPDATE, 3=DELETE)
/// - `database`: Database/space name
/// - `table`: Table name (empty string for graph database)
/// - `rowid`: Affected row ID
///
/// # Safety
/// - `session` must be a valid session handle created by `graphdb_session_create`
/// - `callback` must be a valid function pointer, or NULL to disable the hook
/// - `user_data` is passed to the callback and must remain valid for the lifetime of the callback
#[no_mangle]
pub unsafe extern "C" fn graphdb_update_hook(
    session: *mut graphdb_session_t,
    callback: graphdb_update_hook_callback,
    user_data: *mut c_void,
) -> *mut c_void {
    if session.is_null() {
        return ptr::null_mut();
    }

    let handle = &mut *(session as *mut GraphDbSessionHandle);
    let old_user_data = handle.update_hook_user_data;
    handle.update_hook = callback;
    handle.update_hook_user_data = user_data;
    old_user_data
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
        let db_path = temp_dir.join(format!(
            "test_session_{}_{}.db",
            std::process::id(),
            counter
        ));

        // 确保数据库文件不存在
        if db_path.exists() {
            std::fs::remove_file(&db_path).ok();
            // 等待文件系统完成删除操作
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        let path_cstring = CString::new(db_path.to_str().expect("Invalid path"))
            .expect("Failed to create CString");
        let mut db: *mut graphdb_t = ptr::null_mut();

        let rc = unsafe { graphdb_open(path_cstring.as_ptr(), &mut db) };
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
        let rc = unsafe { graphdb_session_create(db, &mut session) };
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_OK as c_int);
        assert!(!session.is_null());

        // 关闭会话
        let rc = unsafe { graphdb_session_close(session) };
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_OK as c_int);

        // 关闭数据库
        let rc = unsafe { graphdb_close(db) };
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_OK as c_int);
    }

    #[test]
    fn test_session_autocommit() {
        let db = create_test_db();
        let mut session: *mut graphdb_session_t = ptr::null_mut();

        let rc = unsafe { graphdb_session_create(db, &mut session) };
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_OK as c_int);

        // 默认自动提交
        let autocommit = unsafe { graphdb_session_get_autocommit(session) };
        assert!(autocommit);

        // 关闭自动提交
        let rc = unsafe { graphdb_session_set_autocommit(session, false) };
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_OK as c_int);

        let autocommit = unsafe { graphdb_session_get_autocommit(session) };
        assert!(!autocommit);

        unsafe { graphdb_session_close(session) };
        unsafe { graphdb_close(db) };
    }
}
