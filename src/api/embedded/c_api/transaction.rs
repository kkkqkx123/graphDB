//! C API 事务管理模块
//!
//! 提供事务管理功能，包括事务开始、提交、回滚和保存点

use crate::api::core::TransactionHandle;
use crate::api::embedded::c_api::error::{
    error_code_from_core_error, graphdb_error_code_t, set_last_error_message,
};
use crate::api::embedded::c_api::result::GraphDbResultHandle;
use crate::api::embedded::c_api::session::GraphDbSessionHandle;
use crate::api::embedded::c_api::types::{graphdb_result_t, graphdb_session_t, graphdb_txn_t};
use crate::api::embedded::transaction::TransactionConfig;
use crate::transaction::TransactionManager;
use std::ffi::{c_char, c_int, CStr};
use std::ptr;
use std::sync::Arc;

/// 事务句柄内部结构
///
/// 注意：此结构体持有会话指针，但不拥有会话的所有权。
/// 调用者必须确保在事务完成之前不关闭会话。
pub struct GraphDbTxnHandle {
    pub(crate) session: *mut GraphDbSessionHandle,
    pub(crate) txn_manager: Arc<TransactionManager>,
    pub(crate) txn_handle: Option<TransactionHandle>,
    pub(crate) committed: bool,
    pub(crate) rolled_back: bool,
}

impl GraphDbTxnHandle {
    /// 检查会话是否仍然有效
    fn is_session_valid(&self) -> bool {
        !self.session.is_null()
    }

    /// 获取会话引用（如果有效）
    fn get_session(&self) -> Option<&GraphDbSessionHandle> {
        if self.is_session_valid() {
            Some(unsafe { &*self.session })
        } else {
            None
        }
    }
}

impl Drop for GraphDbTxnHandle {
    fn drop(&mut self) {
        if !self.committed && !self.rolled_back {
            if let Some(txn_handle) = self.txn_handle.take() {
                let _ = self.txn_manager.abort_transaction(txn_handle.0);
            }
        }
    }
}

/// 开始事务
///
/// # 参数
/// - `session`: 会话句柄
/// - `txn`: 输出参数，事务句柄
///
/// # 返回
/// - 成功: GRAPHDB_OK
/// - 失败: 错误码
///
/// # Safety
/// - `session` must be a valid session handle created by `graphdb_session_create`
/// - `txn` must be a valid pointer to store the transaction handle
/// - The session must not have been closed
/// - The caller is responsible for freeing the transaction using `graphdb_txn_free` when done
#[no_mangle]
pub unsafe extern "C" fn graphdb_txn_begin(
    session: *mut graphdb_session_t,
    txn: *mut *mut graphdb_txn_t,
) -> c_int {
    if session.is_null() || txn.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    let handle = &*(session as *mut GraphDbSessionHandle);

    match handle.inner.begin_transaction() {
        Ok(txn_obj) => {
            let txn_manager = handle.inner.txn_manager();
            let txn_handle = txn_obj.handle();
            let handle = Box::new(GraphDbTxnHandle {
                session: session as *mut GraphDbSessionHandle,
                txn_manager,
                txn_handle: Some(txn_handle),
                committed: false,
                rolled_back: false,
            });
            *txn = Box::into_raw(handle) as *mut graphdb_txn_t;
            graphdb_error_code_t::GRAPHDB_OK as c_int
        }
        Err(e) => {
            let (error_code, _) = error_code_from_core_error(&e);
            let error_msg = format!("{}", e);
            set_last_error_message(error_msg);
            *txn = ptr::null_mut();
            error_code
        }
    }
}

/// 开始只读事务
///
/// # 参数
/// - `session`: 会话句柄
/// - `txn`: 输出参数，事务句柄
///
/// # 返回
/// - 成功: GRAPHDB_OK
/// - 失败: 错误码
///
/// # Safety
/// - `session` must be a valid session handle created by `graphdb_session_create`
/// - `txn` must be a valid pointer to store the transaction handle
/// - The session must not have been closed
/// - The caller is responsible for freeing the transaction using `graphdb_txn_free` when done
#[no_mangle]
pub unsafe extern "C" fn graphdb_txn_begin_readonly(
    session: *mut graphdb_session_t,
    txn: *mut *mut graphdb_txn_t,
) -> c_int {
    if session.is_null() || txn.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    let handle = &*(session as *mut GraphDbSessionHandle);

    let config = TransactionConfig::new().read_only();
    match handle.inner.begin_transaction_with_config(config) {
        Ok(txn_obj) => {
            let txn_manager = handle.inner.txn_manager();
            let handle_id = txn_obj.handle();
            let txn_handle = Box::new(GraphDbTxnHandle {
                session: session as *mut GraphDbSessionHandle,
                txn_manager,
                txn_handle: Some(handle_id),
                committed: false,
                rolled_back: false,
            });
            *txn = Box::into_raw(txn_handle) as *mut graphdb_txn_t;
            graphdb_error_code_t::GRAPHDB_OK as c_int
        }
        Err(e) => {
            let (error_code, _) = error_code_from_core_error(&e);
            let error_msg = format!("{}", e);
            set_last_error_message(error_msg);
            *txn = ptr::null_mut();
            error_code
        }
    }
}

/// 在事务中执行查询
///
/// # 参数
/// - `txn`: 事务句柄
/// - `query`: 查询语句（UTF-8 编码）
/// - `result`: 输出参数，结果集句柄
///
/// # 返回
/// - 成功: GRAPHDB_OK
/// - 失败: 错误码
///
/// # Safety
/// - `txn` must be a valid transaction handle created by `graphdb_txn_begin` or `graphdb_txn_begin_readonly`
/// - `query` must be a valid pointer to a null-terminated UTF-8 string
/// - `result` must be a valid pointer to store the result handle
/// - The transaction must not have been committed or rolled back
/// - The caller is responsible for freeing the result using `graphdb_result_free` when done
#[no_mangle]
pub unsafe extern "C" fn graphdb_txn_execute(
    txn: *mut graphdb_txn_t,
    query: *const c_char,
    result: *mut *mut graphdb_result_t,
) -> c_int {
    if txn.is_null() || query.is_null() || result.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    let query_str = match CStr::from_ptr(query).to_str() {
        Ok(s) => s,
        Err(_) => return graphdb_error_code_t::GRAPHDB_MISUSE as c_int,
    };

    let handle = &mut *(txn as *mut GraphDbTxnHandle);

    if handle.committed || handle.rolled_back {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    // 检查会话有效性
    let session = match handle.get_session() {
        Some(s) => s,
        None => return graphdb_error_code_t::GRAPHDB_MISUSE as c_int,
    };

    let txn_handle = match handle.txn_handle.as_ref() {
        Some(h) => h,
        None => return graphdb_error_code_t::GRAPHDB_INTERNAL as c_int,
    };

    let ctx = crate::api::core::QueryRequest {
        space_id: session.inner.space_id(),
        auto_commit: false,
        transaction_id: Some(txn_handle.0),
        parameters: None,
    };

    let mut query_api = session.inner.query_api();
    match query_api.execute(query_str, ctx) {
        Ok(core_result) => {
            let query_result =
                crate::api::embedded::result::QueryResult::from_core(core_result);
            let result_handle = Box::new(GraphDbResultHandle {
                inner: query_result,
            });
            *result = Box::into_raw(result_handle) as *mut graphdb_result_t;
            graphdb_error_code_t::GRAPHDB_OK as c_int
        }
        Err(e) => {
            let (error_code, _) = error_code_from_core_error(&e);
            let error_msg = format!("{}", e);
            set_last_error_message(error_msg);
            *result = ptr::null_mut();
            error_code
        }
    }
}

/// 提交事务
///
/// # 参数
/// - `txn`: 事务句柄
///
/// # 返回
/// - 成功: GRAPHDB_OK
/// - 失败: 错误码
///
/// # Safety
/// - `txn` must be a valid transaction handle created by `graphdb_txn_begin` or `graphdb_txn_begin_readonly`
/// - The transaction must not have been committed or rolled back already
/// - The associated session must still be valid
/// - After calling this function, the transaction handle should be freed using `graphdb_txn_free`
#[no_mangle]
pub unsafe extern "C" fn graphdb_txn_commit(txn: *mut graphdb_txn_t) -> c_int {
    if txn.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    let handle = &mut *(txn as *mut GraphDbTxnHandle);

    if handle.committed || handle.rolled_back {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    // 检查会话有效性
    let session = match handle.get_session() {
        Some(s) => s,
        None => return graphdb_error_code_t::GRAPHDB_MISUSE as c_int,
    };

    if let Some(callback) = session.commit_hook {
        let result = callback(session.commit_hook_user_data);
        if result != 0 {
            return graphdb_txn_rollback(txn);
        }
    }

    let txn_handle = match handle.txn_handle.take() {
        Some(h) => h,
        None => return graphdb_error_code_t::GRAPHDB_INTERNAL as c_int,
    };

    match handle.txn_manager.commit_transaction(txn_handle.0) {
        Ok(_) => {
            handle.committed = true;
            graphdb_error_code_t::GRAPHDB_OK as c_int
        }
        Err(e) => {
            let error_code = graphdb_error_code_t::GRAPHDB_ABORT as c_int;
            let error_msg = format!("{}", e);
            set_last_error_message(error_msg);
            error_code
        }
    }
}

/// 回滚事务
///
/// # 参数
/// - `txn`: 事务句柄
///
/// # 返回
/// - 成功: GRAPHDB_OK
/// - 失败: 错误码
///
/// # Safety
/// - `txn` must be a valid transaction handle created by `graphdb_txn_begin` or `graphdb_txn_begin_readonly`
/// - The transaction must not have been committed or rolled back already
/// - The associated session must still be valid
/// - After calling this function, the transaction handle should be freed using `graphdb_txn_free`
#[no_mangle]
pub unsafe extern "C" fn graphdb_txn_rollback(txn: *mut graphdb_txn_t) -> c_int {
    if txn.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    let handle = &mut *(txn as *mut GraphDbTxnHandle);

    if handle.committed || handle.rolled_back {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    // 检查会话有效性
    let session = match handle.get_session() {
        Some(s) => s,
        None => return graphdb_error_code_t::GRAPHDB_MISUSE as c_int,
    };

    if let Some(callback) = session.rollback_hook {
        callback(session.rollback_hook_user_data);
    }

    let txn_handle = match handle.txn_handle.take() {
        Some(h) => h,
        None => return graphdb_error_code_t::GRAPHDB_INTERNAL as c_int,
    };

    match handle.txn_manager.abort_transaction(txn_handle.0) {
        Ok(_) => {
            handle.rolled_back = true;
            graphdb_error_code_t::GRAPHDB_OK as c_int
        }
        Err(e) => {
            let error_code = graphdb_error_code_t::GRAPHDB_ABORT as c_int;
            let error_msg = format!("{}", e);
            set_last_error_message(error_msg);
            error_code
        }
    }
}

/// 创建保存点
///
/// # 参数
/// - `txn`: 事务句柄
/// - `name`: 保存点名称（UTF-8 编码）
///
/// # 返回
/// - 成功: 保存点 ID
/// - 失败: -1
///
/// # Safety
/// - `txn` must be a valid transaction handle created by `graphdb_txn_begin` or `graphdb_txn_begin_readonly`
/// - `name` must be a valid pointer to a null-terminated UTF-8 string
/// - The transaction must not have been committed or rolled back
#[no_mangle]
pub unsafe extern "C" fn graphdb_txn_savepoint(txn: *mut graphdb_txn_t, name: *const c_char) -> i64 {
    if txn.is_null() || name.is_null() {
        return -1;
    }

    let name_str = match CStr::from_ptr(name).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    let handle = &mut *(txn as *mut GraphDbTxnHandle);

    if handle.committed || handle.rolled_back {
        return -1;
    }

    let txn_handle = match handle.txn_handle.as_ref() {
        Some(h) => h,
        None => return -1,
    };

    match handle
        .txn_manager
        .create_savepoint(txn_handle.0, Some(name_str.to_string()))
    {
        Ok(id) => id as i64,
        Err(_) => -1,
    }
}

/// 释放保存点
///
/// # 参数
/// - `txn`: 事务句柄
/// - `savepoint_id`: 保存点 ID
///
/// # 返回
/// - 成功: GRAPHDB_OK
/// - 失败: 错误码
///
/// # Safety
/// - `txn` must be a valid transaction handle created by `graphdb_txn_begin` or `graphdb_txn_begin_readonly`
/// - `savepoint_id` must be a valid savepoint ID returned by `graphdb_txn_savepoint`
/// - The transaction must not have been committed or rolled back
#[no_mangle]
pub unsafe extern "C" fn graphdb_txn_release_savepoint(
    txn: *mut graphdb_txn_t,
    savepoint_id: i64,
) -> c_int {
    if txn.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    let handle = &mut *(txn as *mut GraphDbTxnHandle);

    if handle.committed || handle.rolled_back {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    let txn_handle = match handle.txn_handle.as_ref() {
        Some(h) => h,
        None => return graphdb_error_code_t::GRAPHDB_MISUSE as c_int,
    };

    match handle
        .txn_manager
        .release_savepoint(txn_handle.0, savepoint_id as u64)
    {
        Ok(_) => graphdb_error_code_t::GRAPHDB_OK as c_int,
        Err(e) => {
            let core_error = crate::api::core::CoreError::TransactionFailed(format!("{}", e));
            let (error_code, _) = error_code_from_core_error(&core_error);
            let error_msg = format!("{}", e);
            set_last_error_message(error_msg);
            error_code
        }
    }
}

/// 回滚到保存点
///
/// # 参数
/// - `txn`: 事务句柄
/// - `savepoint_id`: 保存点 ID
///
/// # 返回
/// - 成功: GRAPHDB_OK
/// - 失败: 错误码
///
/// # Safety
/// - `txn` must be a valid transaction handle created by `graphdb_txn_begin` or `graphdb_txn_begin_readonly`
/// - `savepoint_id` must be a valid savepoint ID returned by `graphdb_txn_savepoint`
/// - The transaction must not have been committed or rolled back
#[no_mangle]
pub unsafe extern "C" fn graphdb_txn_rollback_to_savepoint(
    txn: *mut graphdb_txn_t,
    savepoint_id: i64,
) -> c_int {
    if txn.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    let handle = &mut *(txn as *mut GraphDbTxnHandle);

    if handle.committed || handle.rolled_back {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    let txn_handle = match handle.txn_handle.as_ref() {
        Some(h) => h,
        None => return graphdb_error_code_t::GRAPHDB_INTERNAL as c_int,
    };

    match handle
        .txn_manager
        .rollback_to_savepoint(txn_handle.0, savepoint_id as u64)
    {
        Ok(_) => graphdb_error_code_t::GRAPHDB_OK as c_int,
        Err(e) => {
            let core_error = crate::api::core::CoreError::TransactionFailed(format!("{}", e));
            let (error_code, _) = error_code_from_core_error(&core_error);
            let error_msg = format!("{}", e);
            set_last_error_message(error_msg);
            error_code
        }
    }
}

/// 释放事务句柄
///
/// # 参数
/// - `txn`: 事务句柄
///
/// # 返回
/// - 成功: GRAPHDB_OK
/// - 失败: 错误码
///
/// # Safety
/// - `txn` must be a valid transaction handle created by `graphdb_txn_begin` or `graphdb_txn_begin_readonly`
/// - After calling this function, the transaction handle becomes invalid and must not be used
/// - If the transaction has not been committed or rolled back, it will be automatically rolled back
/// - It is safe to call this function multiple times on the same handle (idempotent)
#[no_mangle]
pub unsafe extern "C" fn graphdb_txn_free(txn: *mut graphdb_txn_t) -> c_int {
    if txn.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    let _ = Box::from_raw(txn as *mut GraphDbTxnHandle);

    graphdb_error_code_t::GRAPHDB_OK as c_int
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::embedded::c_api::database::{graphdb_close, graphdb_open};
    use crate::api::embedded::c_api::session::{graphdb_session_close, graphdb_session_create};
    use crate::api::embedded::c_api::types::graphdb_t;
    use std::ffi::CString;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn create_test_db() -> *mut graphdb_t {
        let counter = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let temp_dir = std::env::temp_dir().join("graphdb_c_api_test");
        std::fs::create_dir_all(&temp_dir).ok();
        let db_path = temp_dir.join(format!("test_txn_{}_{}.db", std::process::id(), counter));

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
    fn test_txn_begin_null_params() {
        let rc = unsafe { graphdb_txn_begin(ptr::null_mut(), ptr::null_mut()) };
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_MISUSE as c_int);
    }

    #[test]
    fn test_txn_free_null() {
        let rc = unsafe { graphdb_txn_free(ptr::null_mut()) };
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_MISUSE as c_int);
    }

    #[test]
    fn test_txn_begin_and_free() {
        let db = create_test_db();
        let mut session: *mut graphdb_session_t = ptr::null_mut();

        let rc = unsafe { graphdb_session_create(db, &mut session) };
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_OK as c_int);

        let mut txn: *mut graphdb_txn_t = ptr::null_mut();
        let rc = unsafe { graphdb_txn_begin(session, &mut txn) };
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_OK as c_int);
        assert!(!txn.is_null());

        let rc = unsafe { graphdb_txn_free(txn) };
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_OK as c_int);

        unsafe { graphdb_session_close(session) };
        unsafe { graphdb_close(db) };
    }
}
