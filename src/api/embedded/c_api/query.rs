//! C API 查询执行模块
//!
//! 提供查询执行功能，包括简单查询和参数化查询

use crate::api::embedded::c_api::error::{
    error_code_from_core_error, extended_error_code_from_core_error, graphdb_error_code_t,
};
use crate::api::embedded::c_api::result::GraphDbResultHandle;
use crate::api::embedded::c_api::session::GraphDbSessionHandle;
use crate::api::embedded::c_api::types::{graphdb_result_t, graphdb_session_t, graphdb_value_t};
use crate::core::Value;
use std::collections::HashMap;
use std::ffi::{c_char, c_int, CStr};
use std::ptr;

/// 执行简单查询
///
/// # 参数
/// - `session`: 会话句柄
/// - `query`: 查询语句（UTF-8 编码）
/// - `result`: 输出参数，结果集句柄
///
/// # 返回
/// - 成功: GRAPHDB_OK
/// - 失败: 错误码
#[no_mangle]
pub extern "C" fn graphdb_execute(
    session: *mut graphdb_session_t,
    query: *const c_char,
    result: *mut *mut graphdb_result_t,
) -> c_int {
    if session.is_null() || query.is_null() || result.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    let query_str = unsafe {
        match CStr::from_ptr(query).to_str() {
            Ok(s) => s,
            Err(_) => return graphdb_error_code_t::GRAPHDB_MISUSE as c_int,
        }
    };

    unsafe {
        let handle = &mut *(session as *mut GraphDbSessionHandle);

        // 调用 SQL 追踪回调
        handle.trace(query_str);

        match handle.inner.execute(query_str) {
            Ok(query_result) => {
                handle.clear_error();

                // 检查是否是数据修改操作，并调用更新钩子
                if let Some((operation, rowid)) = detect_data_modification(query_str, &query_result)
                {
                    let space_name = handle.inner.current_space().unwrap_or("default");
                    handle.invoke_update_hook(operation, space_name, rowid);
                }

                let result_handle = Box::new(GraphDbResultHandle {
                    inner: query_result,
                });
                *result = Box::into_raw(result_handle) as *mut graphdb_result_t;
                graphdb_error_code_t::GRAPHDB_OK as c_int
            }
            Err(e) => {
                let (error_code, _) = error_code_from_core_error(&e);
                let error_msg = format!("{}", e);
                let offset = e.error_offset();
                let extended_code = Some(extended_error_code_from_core_error(&e));
                handle.set_error(error_msg, offset, extended_code);
                *result = ptr::null_mut();
                error_code
            }
        }
    }
}

/// 执行参数化查询
///
/// # 参数
/// - `session`: 会话句柄
/// - `query`: 查询语句（UTF-8 编码）
/// - `params`: 参数数组
/// - `param_count`: 参数数量
/// - `result`: 输出参数，结果集句柄
///
/// # 返回
/// - 成功: GRAPHDB_OK
/// - 失败: 错误码
#[no_mangle]
pub extern "C" fn graphdb_execute_params(
    session: *mut graphdb_session_t,
    query: *const c_char,
    params: *const graphdb_value_t,
    param_count: usize,
    result: *mut *mut graphdb_result_t,
) -> c_int {
    if session.is_null() || query.is_null() || result.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    let query_str = unsafe {
        match CStr::from_ptr(query).to_str() {
            Ok(s) => s,
            Err(_) => return graphdb_error_code_t::GRAPHDB_MISUSE as c_int,
        }
    };

    let mut params_map = HashMap::new();

    if !params.is_null() && param_count > 0 {
        for i in 0..param_count {
            unsafe {
                let param = &*params.add(i);
                let param_name = format!("param_{}", i);
                let value = convert_c_value_to_rust(param);
                params_map.insert(param_name, value);
            }
        }
    }

    unsafe {
        let handle = &mut *(session as *mut GraphDbSessionHandle);

        match handle.inner.execute_with_params(query_str, params_map) {
            Ok(query_result) => {
                handle.clear_error();

                // 检查是否是数据修改操作，并调用更新钩子
                if let Some((operation, rowid)) = detect_data_modification(query_str, &query_result)
                {
                    let space_name = handle.inner.current_space().unwrap_or("default");
                    handle.invoke_update_hook(operation, space_name, rowid);
                }

                let result_handle = Box::new(GraphDbResultHandle {
                    inner: query_result,
                });
                *result = Box::into_raw(result_handle) as *mut graphdb_result_t;
                graphdb_error_code_t::GRAPHDB_OK as c_int
            }
            Err(e) => {
                let (error_code, _) = error_code_from_core_error(&e);
                let error_msg = format!("{}", e);
                let offset = e.error_offset();
                let extended_code = Some(extended_error_code_from_core_error(&e));
                handle.set_error(error_msg, offset, extended_code);
                *result = ptr::null_mut();
                error_code
            }
        }
    }
}

/// 将 C 值转换为 Rust 值
unsafe fn convert_c_value_to_rust(c_value: &graphdb_value_t) -> Value {
    use crate::api::embedded::c_api::types::graphdb_value_type_t;

    match c_value.type_ {
        graphdb_value_type_t::GRAPHDB_NULL => Value::Null(crate::core::value::NullType::Null),
        graphdb_value_type_t::GRAPHDB_BOOL => Value::Bool(c_value.data.boolean),
        graphdb_value_type_t::GRAPHDB_INT => Value::Int(c_value.data.integer),
        graphdb_value_type_t::GRAPHDB_FLOAT => Value::Float(c_value.data.floating),
        graphdb_value_type_t::GRAPHDB_STRING => {
            if c_value.data.string.data.is_null() || c_value.data.string.len == 0 {
                Value::String(String::new())
            } else {
                let slice = std::slice::from_raw_parts(
                    c_value.data.string.data as *const u8,
                    c_value.data.string.len,
                );
                let s = String::from_utf8_unchecked(slice.to_vec());
                Value::String(s)
            }
        }
        _ => Value::Null(crate::core::value::NullType::Null),
    }
}

/// 检测查询是否是数据修改操作
///
/// 返回 (操作类型, 行ID) 的元组，如果不是数据修改操作则返回 None
/// 操作类型：1=INSERT, 2=UPDATE, 3=DELETE
fn detect_data_modification(
    query: &str,
    _result: &crate::api::embedded::result::QueryResult,
) -> Option<(i32, i64)> {
    let query_upper = query.trim().to_uppercase();

    // 检查是否是 INSERT 操作
    if query_upper.starts_with("INSERT") {
        return Some((1, 0));
    }

    // 检查是否是 UPDATE 操作
    if query_upper.starts_with("UPDATE") {
        return Some((2, 0));
    }

    // 检查是否是 DELETE 操作
    if query_upper.starts_with("DELETE") {
        return Some((3, 0));
    }

    // 检查是否是 REMOVE 操作
    if query_upper.starts_with("REMOVE") {
        return Some((2, 0));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::embedded::c_api::database::{graphdb_close, graphdb_open};
    use crate::api::embedded::c_api::result::graphdb_result_free;
    use crate::api::embedded::c_api::session::{graphdb_session_close, graphdb_session_create};
    use crate::api::embedded::c_api::types::graphdb_t;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::ffi::CString;

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn create_test_db() -> *mut graphdb_t {
        let counter = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let temp_dir = std::env::temp_dir().join("graphdb_c_api_test");
        std::fs::create_dir_all(&temp_dir).ok();
        let db_path = temp_dir.join(format!("test_{}_{}.db", std::process::id(), counter));

        let path_cstring = CString::new(db_path.to_str().expect("Invalid path"))
            .expect("Failed to create CString");
        let mut db: *mut graphdb_t = ptr::null_mut();

        let rc = graphdb_open(path_cstring.as_ptr(), &mut db);
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_OK as c_int);
        assert!(!db.is_null());

        db
    }

    #[test]
    fn test_execute_null_params() {
        let rc = graphdb_execute(ptr::null_mut(), ptr::null(), ptr::null_mut());
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_MISUSE as c_int);

        let mut result: *mut graphdb_result_t = ptr::null_mut();
        let rc = graphdb_execute(ptr::null_mut(), ptr::null(), &mut result);
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_MISUSE as c_int);
    }

    #[test]
    fn test_execute_params_null_params() {
        let rc = graphdb_execute_params(
            ptr::null_mut(),
            ptr::null(),
            ptr::null(),
            0,
            ptr::null_mut(),
        );
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_MISUSE as c_int);

        let mut result: *mut graphdb_result_t = ptr::null_mut();
        let rc = graphdb_execute_params(ptr::null_mut(), ptr::null(), ptr::null(), 0, &mut result);
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_MISUSE as c_int);
    }

    #[test]
    #[ignore]
    fn test_execute_simple_query() {
        let db = create_test_db();
        let mut session: *mut graphdb_session_t = ptr::null_mut();

        let rc = graphdb_session_create(db, &mut session);
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_OK as c_int);

        let query = CString::new("RETURN 1").expect("Failed to create query CString");
        let mut result: *mut graphdb_result_t = ptr::null_mut();

        let rc = graphdb_execute(session, query.as_ptr(), &mut result);
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_OK as c_int);
        assert!(!result.is_null());

        graphdb_result_free(result);
        graphdb_session_close(session);
        graphdb_close(db);
    }
}
