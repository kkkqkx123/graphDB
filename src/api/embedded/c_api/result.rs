//! C API 结果处理模块
//!
//! 提供查询结果的处理功能

use crate::api::embedded::c_api::error::graphdb_error_code_t;
use crate::api::embedded::c_api::types::graphdb_result_t;
use crate::api::embedded::result::QueryResult;
use std::ffi::{CStr, CString, c_char, c_int};
use std::ptr;

/// 结果集句柄内部结构
pub struct GraphDbResultHandle {
    pub(crate) inner: QueryResult,
}

/// 释放结果集
///
/// # 参数
/// - `result`: 结果集句柄
///
/// # 返回
/// - 成功: GRAPHDB_OK
/// - 失败: 错误码
#[no_mangle]
pub extern "C" fn graphdb_result_free(result: *mut graphdb_result_t) -> c_int {
    if result.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    unsafe {
        let _ = Box::from_raw(result as *mut GraphDbResultHandle);
    }

    graphdb_error_code_t::GRAPHDB_OK as c_int
}

/// 获取结果集列数
///
/// # 参数
/// - `result`: 结果集句柄
///
/// # 返回
/// - 列数，错误返回 -1
#[no_mangle]
pub extern "C" fn graphdb_column_count(result: *mut graphdb_result_t) -> c_int {
    if result.is_null() {
        return -1;
    }

    unsafe {
        let handle = &*(result as *mut GraphDbResultHandle);
        handle.inner.columns().len() as c_int
    }
}

/// 获取结果集行数
///
/// # 参数
/// - `result`: 结果集句柄
///
/// # 返回
/// - 行数，错误返回 -1
#[no_mangle]
pub extern "C" fn graphdb_row_count(result: *mut graphdb_result_t) -> c_int {
    if result.is_null() {
        return -1;
    }

    unsafe {
        let handle = &*(result as *mut GraphDbResultHandle);
        handle.inner.len() as c_int
    }
}

/// 获取列名
///
/// # 参数
/// - `result`: 结果集句柄
/// - `index`: 列索引（从 0 开始）
///
/// # 返回
/// - 列名（UTF-8 编码），错误返回 NULL
#[no_mangle]
pub extern "C" fn graphdb_column_name(
    result: *mut graphdb_result_t,
    index: c_int,
) -> *const c_char {
    if result.is_null() {
        return ptr::null();
    }

    unsafe {
        let handle = &*(result as *mut GraphDbResultHandle);
        
        match handle.inner.columns().get(index as usize) {
            Some(name) => {
                match CString::new(name.as_str()) {
                    Ok(c_name) => c_name.into_raw(),
                    Err(_) => ptr::null(),
                }
            }
            None => ptr::null(),
        }
    }
}

/// 获取整数值
///
/// # 参数
/// - `result`: 结果集句柄
/// - `row`: 行索引（从 0 开始）
/// - `col`: 列名（UTF-8 编码）
/// - `value`: 输出参数，整数值
///
/// # 返回
/// - 成功: GRAPHDB_OK
/// - 失败: 错误码
#[no_mangle]
pub extern "C" fn graphdb_get_int(
    result: *mut graphdb_result_t,
    row: c_int,
    col: *const c_char,
    value: *mut i64,
) -> c_int {
    if result.is_null() || col.is_null() || value.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    let col_str = unsafe {
        match CStr::from_ptr(col).to_str() {
            Ok(s) => s,
            Err(_) => return graphdb_error_code_t::GRAPHDB_MISUSE as c_int,
        }
    };

    unsafe {
        let handle = &*(result as *mut GraphDbResultHandle);
        
        match handle.inner.get(row as usize) {
            Some(row_data) => {
                match row_data.get(col_str) {
                    Some(crate::core::Value::Int(i)) => {
                        *value = *i;
                        graphdb_error_code_t::GRAPHDB_OK as c_int
                    }
                    Some(_) => graphdb_error_code_t::GRAPHDB_MISMATCH as c_int,
                    None => graphdb_error_code_t::GRAPHDB_NOTFOUND as c_int,
                }
            }
            None => graphdb_error_code_t::GRAPHDB_NOTFOUND as c_int,
        }
    }
}

/// 获取字符串值
///
/// # 参数
/// - `result`: 结果集句柄
/// - `row`: 行索引（从 0 开始）
/// - `col`: 列名（UTF-8 编码）
/// - `len`: 输出参数，字符串长度
///
/// # 返回
/// - 字符串值（UTF-8 编码），错误返回 NULL
#[no_mangle]
pub extern "C" fn graphdb_get_string(
    result: *mut graphdb_result_t,
    row: c_int,
    col: *const c_char,
    len: *mut c_int,
) -> *const c_char {
    if result.is_null() || col.is_null() {
        if !len.is_null() {
            unsafe { *len = -1; }
        }
        return ptr::null();
    }

    let col_str = unsafe {
        match CStr::from_ptr(col).to_str() {
            Ok(s) => s,
            Err(_) => {
                if !len.is_null() {
                    *len = -1;
                }
                return ptr::null();
            }
        }
    };

    unsafe {
        let handle = &*(result as *mut GraphDbResultHandle);
        
        match handle.inner.get(row as usize) {
            Some(row_data) => {
                match row_data.get(col_str) {
                    Some(crate::core::Value::String(s)) => {
                        if !len.is_null() {
                            *len = s.len() as c_int;
                        }
                        match CString::new(s.as_str()) {
                            Ok(c_str) => c_str.into_raw(),
                            Err(_) => ptr::null(),
                        }
                    }
                    Some(_) => {
                        if !len.is_null() {
                            *len = -1;
                        }
                        ptr::null()
                    }
                    None => ptr::null(),
                }
            }
            None => ptr::null(),
        }
    }
}

/// 获取二进制数据
///
/// # 参数
/// - `result`: 结果集句柄
/// - `row`: 行索引（从 0 开始）
/// - `col`: 列名（UTF-8 编码）
/// - `len`: 输出参数，数据长度（字节）
///
/// # 返回
/// - 数据指针，错误返回 NULL
///
/// # 注意
/// 返回的指针生命周期与结果集绑定，调用者不应释放
#[no_mangle]
pub extern "C" fn graphdb_get_blob(
    result: *mut graphdb_result_t,
    row: c_int,
    col: *const c_char,
    len: *mut c_int,
) -> *const u8 {
    if result.is_null() || col.is_null() {
        if !len.is_null() {
            unsafe { *len = -1; }
        }
        return ptr::null();
    }

    let col_str = unsafe {
        match CStr::from_ptr(col).to_str() {
            Ok(s) => s,
            Err(_) => {
                if !len.is_null() {
                    *len = -1;
                }
                return ptr::null();
            }
        }
    };

    unsafe {
        let handle = &*(result as *mut GraphDbResultHandle);

        match handle.inner.get(row as usize) {
            Some(row_data) => {
                match row_data.get(col_str) {
                    Some(crate::core::Value::Blob(blob)) => {
                        if !len.is_null() {
                            *len = blob.len() as c_int;
                        }
                        blob.as_ptr()
                    }
                    Some(_) => {
                        if !len.is_null() {
                            *len = -1;
                        }
                        ptr::null()
                    }
                    None => {
                        if !len.is_null() {
                            *len = -1;
                        }
                        ptr::null()
                    }
                }
            }
            None => {
                if !len.is_null() {
                    *len = -1;
                }
                ptr::null()
            }
        }
    }
}

/// 获取整数值（按列索引）
///
/// # 参数
/// - `result`: 结果集句柄
/// - `row`: 行索引（从 0 开始）
/// - `col`: 列索引（从 0 开始）
/// - `value`: 输出参数，整数值
///
/// # 返回
/// - 成功: GRAPHDB_OK
/// - 失败: 错误码
#[no_mangle]
pub extern "C" fn graphdb_get_int_by_index(
    result: *mut graphdb_result_t,
    row: c_int,
    col: c_int,
    value: *mut i64,
) -> c_int {
    if result.is_null() || value.is_null() || col < 0 {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    unsafe {
        let handle = &*(result as *mut GraphDbResultHandle);

        // 获取列名
        let columns = handle.inner.columns();
        let col_name = match columns.get(col as usize) {
            Some(name) => name.as_str(),
            None => return graphdb_error_code_t::GRAPHDB_NOTFOUND as c_int,
        };

        match handle.inner.get(row as usize) {
            Some(row_data) => {
                match row_data.get(col_name) {
                    Some(crate::core::Value::Int(i)) => {
                        *value = *i;
                        graphdb_error_code_t::GRAPHDB_OK as c_int
                    }
                    Some(_) => graphdb_error_code_t::GRAPHDB_MISMATCH as c_int,
                    None => graphdb_error_code_t::GRAPHDB_NOTFOUND as c_int,
                }
            }
            None => graphdb_error_code_t::GRAPHDB_NOTFOUND as c_int,
        }
    }
}

/// 获取字符串值（按列索引）
///
/// # 参数
/// - `result`: 结果集句柄
/// - `row`: 行索引（从 0 开始）
/// - `col`: 列索引（从 0 开始）
/// - `len`: 输出参数，字符串长度
///
/// # 返回
/// - 字符串值（UTF-8 编码），错误返回 NULL
#[no_mangle]
pub extern "C" fn graphdb_get_string_by_index(
    result: *mut graphdb_result_t,
    row: c_int,
    col: c_int,
    len: *mut c_int,
) -> *const c_char {
    if result.is_null() || col < 0 {
        if !len.is_null() {
            unsafe { *len = -1; }
        }
        return ptr::null();
    }

    unsafe {
        let handle = &*(result as *mut GraphDbResultHandle);

        let columns = handle.inner.columns();
        let col_name = match columns.get(col as usize) {
            Some(name) => name.as_str(),
            None => {
                if !len.is_null() {
                    *len = -1;
                }
                return ptr::null();
            }
        };

        match handle.inner.get(row as usize) {
            Some(row_data) => {
                match row_data.get(col_name) {
                    Some(crate::core::Value::String(s)) => {
                        if !len.is_null() {
                            *len = s.len() as c_int;
                        }
                        match CString::new(s.as_str()) {
                            Ok(c_str) => c_str.into_raw(),
                            Err(_) => ptr::null(),
                        }
                    }
                    Some(_) => {
                        if !len.is_null() {
                            *len = -1;
                        }
                        ptr::null()
                    }
                    None => {
                        if !len.is_null() {
                            *len = -1;
                        }
                        ptr::null()
                    }
                }
            }
            None => {
                if !len.is_null() {
                    *len = -1;
                }
                ptr::null()
            }
        }
    }
}

/// 获取列类型
///
/// # 参数
/// - `result`: 结果集句柄
/// - `col`: 列索引（从 0 开始）
///
/// # 返回
/// - 列类型，错误返回 GRAPHDB_NULL
#[no_mangle]
pub extern "C" fn graphdb_column_type(
    result: *mut graphdb_result_t,
    col: c_int,
) -> crate::api::embedded::c_api::types::graphdb_value_type_t {
    use crate::api::embedded::c_api::types::graphdb_value_type_t;

    if result.is_null() || col < 0 {
        return graphdb_value_type_t::GRAPHDB_NULL;
    }

    unsafe {
        let handle = &*(result as *mut GraphDbResultHandle);

        // 获取第一行来确定类型
        match handle.inner.first() {
            Some(row) => {
                let columns = handle.inner.columns();
                let col_name = match columns.get(col as usize) {
                    Some(name) => name.as_str(),
                    None => return graphdb_value_type_t::GRAPHDB_NULL,
                };

                match row.get(col_name) {
                    Some(value) => match value {
                        crate::core::Value::Null(_) => graphdb_value_type_t::GRAPHDB_NULL,
                        crate::core::Value::Bool(_) => graphdb_value_type_t::GRAPHDB_BOOL,
                        crate::core::Value::Int(_) => graphdb_value_type_t::GRAPHDB_INT,
                        crate::core::Value::Float(_) => graphdb_value_type_t::GRAPHDB_FLOAT,
                        crate::core::Value::String(_) => graphdb_value_type_t::GRAPHDB_STRING,
                        crate::core::Value::Blob(_) => graphdb_value_type_t::GRAPHDB_BLOB,
                        crate::core::Value::List(_) => graphdb_value_type_t::GRAPHDB_LIST,
                        crate::core::Value::Map(_) => graphdb_value_type_t::GRAPHDB_MAP,
                        crate::core::Value::Vertex(_) => graphdb_value_type_t::GRAPHDB_VERTEX,
                        crate::core::Value::Edge(_) => graphdb_value_type_t::GRAPHDB_EDGE,
                        crate::core::Value::Path(_) => graphdb_value_type_t::GRAPHDB_PATH,
                        _ => graphdb_value_type_t::GRAPHDB_NULL,
                    }
                    None => graphdb_value_type_t::GRAPHDB_NULL,
                }
            }
            None => graphdb_value_type_t::GRAPHDB_NULL,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_result_null_params() {
        let rc = graphdb_result_free(ptr::null_mut());
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_MISUSE as c_int);

        let count = graphdb_column_count(ptr::null_mut());
        assert_eq!(count, -1);

        let count = graphdb_row_count(ptr::null_mut());
        assert_eq!(count, -1);

        let name = graphdb_column_name(ptr::null_mut(), 0);
        assert!(name.is_null());
    }
}
