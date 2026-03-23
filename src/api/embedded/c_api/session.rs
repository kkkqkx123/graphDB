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
/// # 参数
/// - `db`: 数据库句柄
/// - `session`: 输出参数，会话句柄
///
/// # 返回
/// - 成功: GRAPHDB_OK
/// - 失败: 错误码
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
/// # 参数
/// - `session`: 会话句柄
///
/// # 返回
/// - 成功: GRAPHDB_OK
/// - 失败: 错误码
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
/// # 参数
/// - `session`: 会话句柄
/// - `space_name`: 图空间名称（UTF-8 编码）
///
/// # 返回
/// - 成功: GRAPHDB_OK
/// - 失败: 错误码
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
/// # 参数
/// - `session`: 会话句柄
///
/// # 返回
/// - 当前图空间名称（UTF-8 编码），如果没有则返回 NULL
///
/// # 内存管理
/// 返回的字符串是动态分配的，调用者必须使用 `graphdb_free_string` 释放，
/// 以避免内存泄漏。
///
/// # 示例
/// ```c
/// char* space = graphdb_session_current_space(session);
/// if (space) {
///     printf("Current space: %s\n", space);
///     graphdb_free_string(space);  // 必须释放
/// }
/// ```
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
/// # 参数
/// - `session`: 会话句柄
/// - `autocommit`: 是否自动提交
///
/// # 返回
/// - 成功: GRAPHDB_OK
/// - 失败: 错误码
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
/// # 参数
/// - `session`: 会话句柄
///
/// # 返回
/// - 是否自动提交
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
/// # 参数
/// - `session`: 会话句柄
///
/// # 返回
/// - 影响的行数，如果会话无效则返回 0
#[no_mangle]
pub unsafe extern "C" fn graphdb_changes(session: *mut graphdb_session_t) -> c_int {
    if session.is_null() {
        return 0;
    }

    let handle = &*(session as *mut GraphDbSessionHandle);
    handle.inner.changes() as c_int
}

/// 获取总会话变更数
///
/// # 参数
/// - `session`: 会话句柄
///
/// # 返回
/// - 总会话变更数，如果会话无效则返回 0
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
/// # 参数
/// - `session`: 会话句柄
///
/// # 返回
/// - 最后插入的顶点 ID，如果没有则返回 -1
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
/// # 参数
/// - `session`: 会话句柄
///
/// # 返回
/// - 最后插入的边 ID，如果没有则返回 -1
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
/// # 参数
/// - `session`: 会话句柄
/// - `timeout_ms`: 超时时间（毫秒），0 表示不等待
///
/// # 返回
/// - 成功: GRAPHDB_OK
/// - 失败: 错误码
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
/// # 参数
/// - `session`: 会话句柄
///
/// # 返回
/// - 超时时间（毫秒），如果会话无效则返回 0
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
/// # 参数
/// - `session`: 会话句柄
/// - `callback`: 追踪回调函数，NULL 表示取消追踪
/// - `user_data`: 用户数据指针，将传递给回调函数
///
/// # 返回
/// - 成功: GRAPHDB_OK
/// - 失败: 错误码
///
/// # 示例
/// ```c
/// extern void my_trace_callback(const char* sql, void* data) {
///     printf("Executing: %s\n", sql);
/// }
///
/// graphdb_trace(session, my_trace_callback, NULL);
/// ```
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
/// # 参数
/// - `session`: 会话句柄
/// - `callback`: 提交钩子回调函数，NULL 表示取消钩子
/// - `user_data`: 用户数据指针，将传递给回调函数
///
/// # 返回
/// - 之前的钩子用户数据指针（如果有）
///
/// # 说明
/// 提交钩子在事务提交前被调用。如果回调返回非零值，事务将回滚。
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
/// # 参数
/// - `session`: 会话句柄
/// - `callback`: 回滚钩子回调函数，NULL 表示取消钩子
/// - `user_data`: 用户数据指针，将传递给回调函数
///
/// # 返回
/// - 之前的钩子用户数据指针（如果有）
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
/// 当数据库中的数据发生变更时调用回调函数
///
/// # 参数
/// - `session`: 会话句柄
/// - `callback`: 更新钩子回调函数，NULL 表示取消钩子
/// - `user_data`: 用户数据指针，将传递给回调函数
///
/// # 返回
/// - 之前的钩子用户数据指针（如果有）
///
/// # 回调参数说明
/// - `operation`: 操作类型（1=INSERT, 2=UPDATE, 3=DELETE）
/// - `database`: 数据库/空间名称
/// - `table`: 表名称（图数据库中为空字符串）
/// - `rowid`: 受影响的行 ID
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
