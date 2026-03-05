//! C API 预编译语句模块
//!
//! 提供预编译语句功能，支持语句准备、参数绑定和重复执行

use crate::api::embedded::c_api::error::{error_code_from_core_error, graphdb_error_code_t};
use crate::api::embedded::c_api::result::GraphDbResultHandle;
use crate::api::embedded::c_api::session::GraphDbSessionHandle;
use crate::api::embedded::c_api::types::{graphdb_result_t, graphdb_session_t, graphdb_stmt_t, graphdb_value_t};
use crate::api::embedded::c_api::types::graphdb_value_type_t;
use crate::api::embedded::statement::PreparedStatement;
use crate::core::Value;
use std::collections::HashMap;
use std::ffi::{CStr, CString, c_char, c_int};
use std::ptr;

/// 预编译语句句柄内部结构
pub struct GraphDbStmtHandle {
    pub(crate) inner: PreparedStatement<crate::storage::RedbStorage>,
    pub(crate) last_error: Option<CString>,
}

/// 准备语句
///
/// # 参数
/// - `session`: 会话句柄
/// - `query`: 查询语句（UTF-8 编码）
/// - `stmt`: 输出参数，语句句柄
///
/// # 返回
/// - 成功: GRAPHDB_OK
/// - 失败: 错误码
#[no_mangle]
pub extern "C" fn graphdb_prepare(
    session: *mut graphdb_session_t,
    query: *const c_char,
    stmt: *mut *mut graphdb_stmt_t,
) -> c_int {
    if session.is_null() || query.is_null() || stmt.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    let query_str = unsafe {
        match CStr::from_ptr(query).to_str() {
            Ok(s) => s,
            Err(_) => return graphdb_error_code_t::GRAPHDB_MISUSE as c_int,
        }
    };

    unsafe {
        let handle = &*(session as *mut GraphDbSessionHandle);

        match handle.inner.prepare(query_str) {
            Ok(prepared_stmt) => {
                let stmt_handle = Box::new(GraphDbStmtHandle {
                    inner: prepared_stmt,
                    last_error: None,
                });
                *stmt = Box::into_raw(stmt_handle) as *mut graphdb_stmt_t;
                graphdb_error_code_t::GRAPHDB_OK as c_int
            }
            Err(e) => {
                let error_code = error_code_from_core_error(&e);
                *stmt = ptr::null_mut();
                error_code
            }
        }
    }
}

/// 绑定 NULL 值（按索引）
///
/// # 参数
/// - `stmt`: 语句句柄
/// - `index`: 参数索引（从 1 开始）
///
/// # 返回
/// - 成功: GRAPHDB_OK
/// - 失败: 错误码
#[no_mangle]
pub extern "C" fn graphdb_bind_null(stmt: *mut graphdb_stmt_t, index: c_int) -> c_int {
    if stmt.is_null() || index < 1 {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    unsafe {
        let handle = &mut *(stmt as *mut GraphDbStmtHandle);
        let param_name = format!("param_{}", index - 1);

        match handle.inner.bind(&param_name, Value::Null(crate::core::value::NullType::Null)) {
            Ok(_) => graphdb_error_code_t::GRAPHDB_OK as c_int,
            Err(e) => {
                let error_code = error_code_from_core_error(&e);
                handle.last_error = Some(CString::new(format!("{}", e)).unwrap_or_default());
                error_code
            }
        }
    }
}

/// 绑定布尔值（按索引）
///
/// # 参数
/// - `stmt`: 语句句柄
/// - `index`: 参数索引（从 1 开始）
/// - `value`: 布尔值
///
/// # 返回
/// - 成功: GRAPHDB_OK
/// - 失败: 错误码
#[no_mangle]
pub extern "C" fn graphdb_bind_bool(
    stmt: *mut graphdb_stmt_t,
    index: c_int,
    value: bool,
) -> c_int {
    if stmt.is_null() || index < 1 {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    unsafe {
        let handle = &mut *(stmt as *mut GraphDbStmtHandle);
        let param_name = format!("param_{}", index - 1);

        match handle.inner.bind(&param_name, Value::Bool(value)) {
            Ok(_) => graphdb_error_code_t::GRAPHDB_OK as c_int,
            Err(e) => {
                let error_code = error_code_from_core_error(&e);
                handle.last_error = Some(CString::new(format!("{}", e)).unwrap_or_default());
                error_code
            }
        }
    }
}

/// 绑定整数值（按索引）
///
/// # 参数
/// - `stmt`: 语句句柄
/// - `index`: 参数索引（从 1 开始）
/// - `value`: 整数值
///
/// # 返回
/// - 成功: GRAPHDB_OK
/// - 失败: 错误码
#[no_mangle]
pub extern "C" fn graphdb_bind_int(
    stmt: *mut graphdb_stmt_t,
    index: c_int,
    value: i64,
) -> c_int {
    if stmt.is_null() || index < 1 {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    unsafe {
        let handle = &mut *(stmt as *mut GraphDbStmtHandle);
        let param_name = format!("param_{}", index - 1);

        match handle.inner.bind(&param_name, Value::Int(value)) {
            Ok(_) => graphdb_error_code_t::GRAPHDB_OK as c_int,
            Err(e) => {
                let error_code = error_code_from_core_error(&e);
                handle.last_error = Some(CString::new(format!("{}", e)).unwrap_or_default());
                error_code
            }
        }
    }
}

/// 绑定浮点值（按索引）
///
/// # 参数
/// - `stmt`: 语句句柄
/// - `index`: 参数索引（从 1 开始）
/// - `value`: 浮点值
///
/// # 返回
/// - 成功: GRAPHDB_OK
/// - 失败: 错误码
#[no_mangle]
pub extern "C" fn graphdb_bind_float(
    stmt: *mut graphdb_stmt_t,
    index: c_int,
    value: f64,
) -> c_int {
    if stmt.is_null() || index < 1 {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    unsafe {
        let handle = &mut *(stmt as *mut GraphDbStmtHandle);
        let param_name = format!("param_{}", index - 1);

        match handle.inner.bind(&param_name, Value::Float(value)) {
            Ok(_) => graphdb_error_code_t::GRAPHDB_OK as c_int,
            Err(e) => {
                let error_code = error_code_from_core_error(&e);
                handle.last_error = Some(CString::new(format!("{}", e)).unwrap_or_default());
                error_code
            }
        }
    }
}

/// 绑定字符串值（按索引）
///
/// # 参数
/// - `stmt`: 语句句柄
/// - `index`: 参数索引（从 1 开始）
/// - `value`: 字符串值（UTF-8 编码）
/// - `len`: 字符串长度（-1 表示自动计算）
///
/// # 返回
/// - 成功: GRAPHDB_OK
/// - 失败: 错误码
#[no_mangle]
pub extern "C" fn graphdb_bind_string(
    stmt: *mut graphdb_stmt_t,
    index: c_int,
    value: *const c_char,
    len: c_int,
) -> c_int {
    if stmt.is_null() || index < 1 || value.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    let string_value = unsafe {
        let c_str = CStr::from_ptr(value);
        if len < 0 {
            c_str.to_str().unwrap_or("").to_string()
        } else {
            let slice = std::slice::from_raw_parts(value as *const u8, len as usize);
            String::from_utf8_lossy(slice).to_string()
        }
    };

    unsafe {
        let handle = &mut *(stmt as *mut GraphDbStmtHandle);
        let param_name = format!("param_{}", index - 1);

        match handle.inner.bind(&param_name, Value::String(string_value)) {
            Ok(_) => graphdb_error_code_t::GRAPHDB_OK as c_int,
            Err(e) => {
                let error_code = error_code_from_core_error(&e);
                handle.last_error = Some(CString::new(format!("{}", e)).unwrap_or_default());
                error_code
            }
        }
    }
}

/// 绑定参数（按名称）
///
/// # 参数
/// - `stmt`: 语句句柄
/// - `name`: 参数名称（UTF-8 编码）
/// - `value`: 值
///
/// # 返回
/// - 成功: GRAPHDB_OK
/// - 失败: 错误码
#[no_mangle]
pub extern "C" fn graphdb_bind_by_name(
    stmt: *mut graphdb_stmt_t,
    name: *const c_char,
    value: graphdb_value_t,
) -> c_int {
    if stmt.is_null() || name.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    let param_name = unsafe {
        match CStr::from_ptr(name).to_str() {
            Ok(s) => s,
            Err(_) => return graphdb_error_code_t::GRAPHDB_MISUSE as c_int,
        }
    };

    let rust_value = unsafe { convert_c_value_to_rust(&value) };

    unsafe {
        let handle = &mut *(stmt as *mut GraphDbStmtHandle);

        match handle.inner.bind(param_name, rust_value) {
            Ok(_) => graphdb_error_code_t::GRAPHDB_OK as c_int,
            Err(e) => {
                let error_code = error_code_from_core_error(&e);
                handle.last_error = Some(CString::new(format!("{}", e)).unwrap_or_default());
                error_code
            }
        }
    }
}

/// 重置语句
///
/// 清除所有绑定的参数，使语句可以重新执行
///
/// # 参数
/// - `stmt`: 语句句柄
///
/// # 返回
/// - 成功: GRAPHDB_OK
/// - 失败: 错误码
#[no_mangle]
pub extern "C" fn graphdb_reset(stmt: *mut graphdb_stmt_t) -> c_int {
    if stmt.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    unsafe {
        let handle = &mut *(stmt as *mut GraphDbStmtHandle);
        handle.inner.reset();
        graphdb_error_code_t::GRAPHDB_OK as c_int
    }
}

/// 清除绑定
///
/// 清除所有绑定的参数
///
/// # 参数
/// - `stmt`: 语句句柄
///
/// # 返回
/// - 成功: GRAPHDB_OK
/// - 失败: 错误码
#[no_mangle]
pub extern "C" fn graphdb_clear_bindings(stmt: *mut graphdb_stmt_t) -> c_int {
    if stmt.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    unsafe {
        let handle = &mut *(stmt as *mut GraphDbStmtHandle);
        handle.inner.clear_bindings();
        graphdb_error_code_t::GRAPHDB_OK as c_int
    }
}

/// 释放语句
///
/// # 参数
/// - `stmt`: 语句句柄
///
/// # 返回
/// - 成功: GRAPHDB_OK
/// - 失败: 错误码
#[no_mangle]
pub extern "C" fn graphdb_finalize(stmt: *mut graphdb_stmt_t) -> c_int {
    if stmt.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    unsafe {
        let _ = Box::from_raw(stmt as *mut GraphDbStmtHandle);
    }

    graphdb_error_code_t::GRAPHDB_OK as c_int
}

/// 获取参数索引
///
/// # 参数
/// - `stmt`: 语句句柄
/// - `name`: 参数名称（UTF-8 编码）
///
/// # 返回
/// - 参数索引（从 1 开始），未找到返回 0
#[no_mangle]
pub extern "C" fn graphdb_bind_parameter_index(
    stmt: *mut graphdb_stmt_t,
    name: *const c_char,
) -> c_int {
    if stmt.is_null() || name.is_null() {
        return 0;
    }

    let param_name = unsafe {
        match CStr::from_ptr(name).to_str() {
            Ok(s) => s,
            Err(_) => return 0,
        }
    };

    unsafe {
        let handle = &*(stmt as *mut GraphDbStmtHandle);

        for (idx, key) in handle.inner.parameters().keys().enumerate() {
            if key == param_name {
                return (idx + 1) as c_int;
            }
        }
    }

    0
}

/// 获取参数名称
///
/// # 参数
/// - `stmt`: 语句句柄
/// - `index`: 参数索引（从 1 开始）
///
/// # 返回
/// - 参数名称（UTF-8 编码），未找到返回 NULL
#[no_mangle]
pub extern "C" fn graphdb_bind_parameter_name(
    stmt: *mut graphdb_stmt_t,
    index: c_int,
) -> *const c_char {
    if stmt.is_null() || index < 1 {
        return ptr::null();
    }

    unsafe {
        let handle = &*(stmt as *mut GraphDbStmtHandle);

        let keys: Vec<&String> = handle.inner.parameters().keys().collect();
        if let Some(key) = keys.get((index - 1) as usize) {
            match CString::new(key.as_str()) {
                Ok(c_name) => c_name.into_raw(),
                Err(_) => ptr::null(),
            }
        } else {
            ptr::null()
        }
    }
}

/// 获取参数数量
///
/// # 参数
/// - `stmt`: 语句句柄
///
/// # 返回
/// - 参数数量
#[no_mangle]
pub extern "C" fn graphdb_bind_parameter_count(stmt: *mut graphdb_stmt_t) -> c_int {
    if stmt.is_null() {
        return 0;
    }

    unsafe {
        let handle = &*(stmt as *mut GraphDbStmtHandle);
        handle.inner.parameters().len() as c_int
    }
}

/// 将 C 值转换为 Rust 值
unsafe fn convert_c_value_to_rust(c_value: &graphdb_value_t) -> Value {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::embedded::c_api::database::{graphdb_close, graphdb_open};
    use crate::api::embedded::c_api::session::{graphdb_session_close, graphdb_session_create};
    use crate::api::embedded::c_api::types::graphdb_t;
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
    fn test_prepare_null_params() {
        let rc = graphdb_prepare(ptr::null_mut(), ptr::null(), ptr::null_mut());
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_MISUSE as c_int);
    }

    #[test]
    fn test_bind_null_invalid_index() {
        let rc = graphdb_bind_null(ptr::null_mut(), 0);
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_MISUSE as c_int);

        let rc = graphdb_bind_null(ptr::null_mut(), -1);
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_MISUSE as c_int);
    }

    #[test]
    fn test_finalize_null() {
        let rc = graphdb_finalize(ptr::null_mut());
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_MISUSE as c_int);
    }

    #[test]
    fn test_prepare_and_finalize() {
        let db = create_test_db();
        let mut session: *mut graphdb_session_t = ptr::null_mut();

        let rc = graphdb_session_create(db, &mut session);
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_OK as c_int);

        let query = CString::new("SHOW SPACES").unwrap();
        let mut stmt: *mut graphdb_stmt_t = ptr::null_mut();

        let rc = graphdb_prepare(session, query.as_ptr(), &mut stmt);
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_OK as c_int);
        assert!(!stmt.is_null());

        let rc = graphdb_finalize(stmt);
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_OK as c_int);

        graphdb_session_close(session);
        graphdb_close(db);
    }
}
