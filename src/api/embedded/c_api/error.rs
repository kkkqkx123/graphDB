//! C API 错误处理
//!
//! 提供错误码转换和错误信息管理功能

use crate::api::core::{CoreError, ExtendedErrorCode};
use crate::api::embedded::c_api::types::{graphdb_extended_error_code_t, graphdb_session_t};
use std::cell::RefCell;
use std::ffi::CString;

thread_local! {
    static LAST_ERROR_MESSAGE: RefCell<Option<CString>> = RefCell::new(None);
}

/// 设置最后的错误消息
pub(crate) fn set_last_error_message(msg: String) {
    LAST_ERROR_MESSAGE.with(|m| {
        *m.borrow_mut() = CString::new(msg).ok();
    });
}

/// 从 CoreError 推断扩展错误码
pub fn extended_error_code_from_core_error(error: &CoreError) -> graphdb_extended_error_code_t {
    match error {
        CoreError::DetailedQueryError { extended_code, .. } => {
            extended_error_code_from_internal(*extended_code)
        }
        CoreError::QueryExecutionFailed(msg) => {
            if msg.contains("syntax") {
                graphdb_extended_error_code_t::GRAPHDB_ERROR_SYNTAX
            } else if msg.contains("semantic") {
                graphdb_extended_error_code_t::GRAPHDB_ERROR_SEMANTIC
            } else if msg.contains("type mismatch") {
                graphdb_extended_error_code_t::GRAPHDB_ERROR_TYPE_MISMATCH
            } else if msg.contains("constraint") {
                graphdb_extended_error_code_t::GRAPHDB_ERROR_CHECK
            } else if msg.contains("division by zero") {
                graphdb_extended_error_code_t::GRAPHDB_ERROR_DIVISION_BY_ZERO
            } else if msg.contains("out of range") {
                graphdb_extended_error_code_t::GRAPHDB_ERROR_OUT_OF_RANGE
            } else if msg.contains("duplicate") {
                graphdb_extended_error_code_t::GRAPHDB_ERROR_DUPLICATE_KEY
            } else if msg.contains("not null") {
                graphdb_extended_error_code_t::GRAPHDB_ERROR_NOT_NULL
            } else if msg.contains("unique") {
                graphdb_extended_error_code_t::GRAPHDB_ERROR_UNIQUE
            } else if msg.contains("foreign key") {
                graphdb_extended_error_code_t::GRAPHDB_ERROR_FOREIGN_KEY
            } else if msg.contains("deadlock") {
                graphdb_extended_error_code_t::GRAPHDB_ERROR_DEADLOCK
            } else if msg.contains("timeout") {
                graphdb_extended_error_code_t::GRAPHDB_ERROR_LOCK_TIMEOUT
            } else if msg.contains("connection") {
                graphdb_extended_error_code_t::GRAPHDB_ERROR_CONNECTION_LOST
            } else if msg.contains("vertex") {
                graphdb_extended_error_code_t::GRAPHDB_ERROR_INVALID_VERTEX
            } else if msg.contains("edge") {
                graphdb_extended_error_code_t::GRAPHDB_ERROR_INVALID_EDGE
            } else if msg.contains("path") {
                graphdb_extended_error_code_t::GRAPHDB_ERROR_PATH_NOT_FOUND
            } else {
                graphdb_extended_error_code_t::GRAPHDB_EXTENDED_NONE
            }
        }
        CoreError::StorageError(_) => graphdb_extended_error_code_t::GRAPHDB_EXTENDED_NONE,
        CoreError::TransactionFailed(_) => graphdb_extended_error_code_t::GRAPHDB_EXTENDED_NONE,
        CoreError::SchemaOperationFailed(_) => graphdb_extended_error_code_t::GRAPHDB_EXTENDED_NONE,
        CoreError::InvalidParameter(_) => graphdb_extended_error_code_t::GRAPHDB_EXTENDED_NONE,
        CoreError::NotFound(_) => graphdb_extended_error_code_t::GRAPHDB_EXTENDED_NONE,
        CoreError::Internal(_) => graphdb_extended_error_code_t::GRAPHDB_EXTENDED_NONE,
    }
}

/// 从内部 ExtendedErrorCode 转换为 C API 扩展错误码
fn extended_error_code_from_internal(code: ExtendedErrorCode) -> graphdb_extended_error_code_t {
    match code {
        ExtendedErrorCode::None => graphdb_extended_error_code_t::GRAPHDB_EXTENDED_NONE,
        ExtendedErrorCode::SyntaxError => graphdb_extended_error_code_t::GRAPHDB_ERROR_SYNTAX,
        ExtendedErrorCode::SemanticError => graphdb_extended_error_code_t::GRAPHDB_ERROR_SEMANTIC,
        ExtendedErrorCode::UnexpectedToken => {
            graphdb_extended_error_code_t::GRAPHDB_ERROR_UNEXPECTED_TOKEN
        }
        ExtendedErrorCode::UnterminatedLiteral => {
            graphdb_extended_error_code_t::GRAPHDB_ERROR_UNTERMINATED_LITERAL
        }
        ExtendedErrorCode::TypeMismatch => graphdb_extended_error_code_t::GRAPHDB_ERROR_TYPE_MISMATCH,
        ExtendedErrorCode::DivisionByZero => graphdb_extended_error_code_t::GRAPHDB_ERROR_DIVISION_BY_ZERO,
        ExtendedErrorCode::OutOfRange => graphdb_extended_error_code_t::GRAPHDB_ERROR_OUT_OF_RANGE,
        ExtendedErrorCode::DuplicateKey => graphdb_extended_error_code_t::GRAPHDB_ERROR_DUPLICATE_KEY,
        ExtendedErrorCode::ForeignKeyConstraint => {
            graphdb_extended_error_code_t::GRAPHDB_ERROR_FOREIGN_KEY
        }
        ExtendedErrorCode::NotNullConstraint => graphdb_extended_error_code_t::GRAPHDB_ERROR_NOT_NULL,
        ExtendedErrorCode::UniqueConstraint => graphdb_extended_error_code_t::GRAPHDB_ERROR_UNIQUE,
        ExtendedErrorCode::CheckConstraint => graphdb_extended_error_code_t::GRAPHDB_ERROR_CHECK,
        ExtendedErrorCode::ConnectionLost => graphdb_extended_error_code_t::GRAPHDB_ERROR_CONNECTION_LOST,
        ExtendedErrorCode::Deadlock => graphdb_extended_error_code_t::GRAPHDB_ERROR_DEADLOCK,
        ExtendedErrorCode::LockTimeout => graphdb_extended_error_code_t::GRAPHDB_ERROR_LOCK_TIMEOUT,
        ExtendedErrorCode::InvalidVertex => graphdb_extended_error_code_t::GRAPHDB_ERROR_INVALID_VERTEX,
        ExtendedErrorCode::InvalidEdge => graphdb_extended_error_code_t::GRAPHDB_ERROR_INVALID_EDGE,
        ExtendedErrorCode::PathNotFound => graphdb_extended_error_code_t::GRAPHDB_ERROR_PATH_NOT_FOUND,
    }
}

/// 错误码
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum graphdb_error_code_t {
    /// 成功
    GRAPHDB_OK = 0,
    /// 一般错误
    GRAPHDB_ERROR = 1,
    /// 内部错误
    GRAPHDB_INTERNAL = 2,
    /// 权限被拒绝
    GRAPHDB_PERM = 3,
    /// 操作被中止
    GRAPHDB_ABORT = 4,
    /// 数据库忙
    GRAPHDB_BUSY = 5,
    /// 数据库被锁定
    GRAPHDB_LOCKED = 6,
    /// 内存不足
    GRAPHDB_NOMEM = 7,
    /// 只读
    GRAPHDB_READONLY = 8,
    /// 操作被中断
    GRAPHDB_INTERRUPT = 9,
    /// IO 错误
    GRAPHDB_IOERR = 10,
    /// 数据损坏
    GRAPHDB_CORRUPT = 11,
    /// 未找到
    GRAPHDB_NOTFOUND = 12,
    /// 磁盘已满
    GRAPHDB_FULL = 13,
    /// 无法打开
    GRAPHDB_CANTOPEN = 14,
    /// 协议错误
    GRAPHDB_PROTOCOL = 15,
    /// 模式错误
    GRAPHDB_SCHEMA = 16,
    /// 数据过大
    GRAPHDB_TOOBIG = 17,
    /// 约束违反
    GRAPHDB_CONSTRAINT = 18,
    /// 类型不匹配
    GRAPHDB_MISMATCH = 19,
    /// 误用
    GRAPHDB_MISUSE = 20,
    /// 超出范围
    GRAPHDB_RANGE = 21,
    /// 未实现
    GRAPHDB_NOT_IMPLEMENTED = 22,
}

/// 从核心错误转换为 C 错误码和扩展错误码
pub fn error_code_from_core_error(error: &CoreError) -> (i32, graphdb_extended_error_code_t) {
    match error {
        CoreError::DetailedQueryError { extended_code, .. } => {
            let basic_code = match extended_code {
                ExtendedErrorCode::SyntaxError
                | ExtendedErrorCode::SemanticError
                | ExtendedErrorCode::UnexpectedToken
                | ExtendedErrorCode::UnterminatedLiteral => graphdb_error_code_t::GRAPHDB_ERROR as i32,
                ExtendedErrorCode::TypeMismatch => graphdb_error_code_t::GRAPHDB_MISMATCH as i32,
                ExtendedErrorCode::DivisionByZero => graphdb_error_code_t::GRAPHDB_RANGE as i32,
                ExtendedErrorCode::OutOfRange => graphdb_error_code_t::GRAPHDB_RANGE as i32,
                ExtendedErrorCode::DuplicateKey => graphdb_error_code_t::GRAPHDB_CONSTRAINT as i32,
                ExtendedErrorCode::ForeignKeyConstraint => graphdb_error_code_t::GRAPHDB_CONSTRAINT as i32,
                ExtendedErrorCode::NotNullConstraint => graphdb_error_code_t::GRAPHDB_CONSTRAINT as i32,
                ExtendedErrorCode::UniqueConstraint => graphdb_error_code_t::GRAPHDB_CONSTRAINT as i32,
                ExtendedErrorCode::CheckConstraint => graphdb_error_code_t::GRAPHDB_CONSTRAINT as i32,
                ExtendedErrorCode::ConnectionLost => graphdb_error_code_t::GRAPHDB_IOERR as i32,
                ExtendedErrorCode::Deadlock => graphdb_error_code_t::GRAPHDB_BUSY as i32,
                ExtendedErrorCode::LockTimeout => graphdb_error_code_t::GRAPHDB_BUSY as i32,
                ExtendedErrorCode::InvalidVertex | ExtendedErrorCode::InvalidEdge | ExtendedErrorCode::PathNotFound => {
                    graphdb_error_code_t::GRAPHDB_NOTFOUND as i32
                }
                ExtendedErrorCode::None => graphdb_error_code_t::GRAPHDB_OK as i32,
            };
            (basic_code, extended_error_code_from_internal(*extended_code))
        }
        CoreError::StorageError(_) => (
            graphdb_error_code_t::GRAPHDB_IOERR as i32,
            graphdb_extended_error_code_t::GRAPHDB_EXTENDED_NONE,
        ),
        CoreError::QueryExecutionFailed(msg) => {
            let extended_code = if msg.contains("syntax") {
                graphdb_extended_error_code_t::GRAPHDB_ERROR_SYNTAX
            } else if msg.contains("semantic") {
                graphdb_extended_error_code_t::GRAPHDB_ERROR_SEMANTIC
            } else if msg.contains("type mismatch") {
                graphdb_extended_error_code_t::GRAPHDB_ERROR_TYPE_MISMATCH
            } else if msg.contains("constraint") {
                graphdb_extended_error_code_t::GRAPHDB_ERROR_CHECK
            } else if msg.contains("division by zero") {
                graphdb_extended_error_code_t::GRAPHDB_ERROR_DIVISION_BY_ZERO
            } else if msg.contains("out of range") {
                graphdb_extended_error_code_t::GRAPHDB_ERROR_OUT_OF_RANGE
            } else if msg.contains("duplicate") {
                graphdb_extended_error_code_t::GRAPHDB_ERROR_DUPLICATE_KEY
            } else if msg.contains("not null") {
                graphdb_extended_error_code_t::GRAPHDB_ERROR_NOT_NULL
            } else if msg.contains("unique") {
                graphdb_extended_error_code_t::GRAPHDB_ERROR_UNIQUE
            } else if msg.contains("foreign key") {
                graphdb_extended_error_code_t::GRAPHDB_ERROR_FOREIGN_KEY
            } else if msg.contains("deadlock") {
                graphdb_extended_error_code_t::GRAPHDB_ERROR_DEADLOCK
            } else if msg.contains("timeout") {
                graphdb_extended_error_code_t::GRAPHDB_ERROR_LOCK_TIMEOUT
            } else if msg.contains("connection") {
                graphdb_extended_error_code_t::GRAPHDB_ERROR_CONNECTION_LOST
            } else if msg.contains("vertex") {
                graphdb_extended_error_code_t::GRAPHDB_ERROR_INVALID_VERTEX
            } else if msg.contains("edge") {
                graphdb_extended_error_code_t::GRAPHDB_ERROR_INVALID_EDGE
            } else if msg.contains("path") {
                graphdb_extended_error_code_t::GRAPHDB_ERROR_PATH_NOT_FOUND
            } else {
                graphdb_extended_error_code_t::GRAPHDB_EXTENDED_NONE
            };
            (graphdb_error_code_t::GRAPHDB_ERROR as i32, extended_code)
        }
        CoreError::TransactionFailed(_) => (
            graphdb_error_code_t::GRAPHDB_ABORT as i32,
            graphdb_extended_error_code_t::GRAPHDB_EXTENDED_NONE,
        ),
        CoreError::SchemaOperationFailed(_) => (
            graphdb_error_code_t::GRAPHDB_SCHEMA as i32,
            graphdb_extended_error_code_t::GRAPHDB_EXTENDED_NONE,
        ),
        CoreError::Internal(_) => (
            graphdb_error_code_t::GRAPHDB_INTERNAL as i32,
            graphdb_extended_error_code_t::GRAPHDB_EXTENDED_NONE,
        ),
        CoreError::NotFound(_) => (
            graphdb_error_code_t::GRAPHDB_NOTFOUND as i32,
            graphdb_extended_error_code_t::GRAPHDB_EXTENDED_NONE,
        ),
        CoreError::InvalidParameter(_) => (
            graphdb_error_code_t::GRAPHDB_MISUSE as i32,
            graphdb_extended_error_code_t::GRAPHDB_EXTENDED_NONE,
        ),
    }
}

/// 获取错误码对应的描述消息（null 终止）
pub fn error_code_to_message(code: graphdb_error_code_t) -> &'static [u8] {
    match code {
        graphdb_error_code_t::GRAPHDB_OK => "OK\0".as_bytes(),
        graphdb_error_code_t::GRAPHDB_ERROR => "General error\0".as_bytes(),
        graphdb_error_code_t::GRAPHDB_INTERNAL => "Internal error\0".as_bytes(),
        graphdb_error_code_t::GRAPHDB_PERM => "Permission denied\0".as_bytes(),
        graphdb_error_code_t::GRAPHDB_ABORT => "Operation aborted\0".as_bytes(),
        graphdb_error_code_t::GRAPHDB_BUSY => "Database busy\0".as_bytes(),
        graphdb_error_code_t::GRAPHDB_LOCKED => "Database locked\0".as_bytes(),
        graphdb_error_code_t::GRAPHDB_NOMEM => "Out of memory\0".as_bytes(),
        graphdb_error_code_t::GRAPHDB_READONLY => "Read only\0".as_bytes(),
        graphdb_error_code_t::GRAPHDB_INTERRUPT => "Operation interrupted\0".as_bytes(),
        graphdb_error_code_t::GRAPHDB_IOERR => "IO error\0".as_bytes(),
        graphdb_error_code_t::GRAPHDB_CORRUPT => "Data corruption\0".as_bytes(),
        graphdb_error_code_t::GRAPHDB_NOTFOUND => "Not found\0".as_bytes(),
        graphdb_error_code_t::GRAPHDB_FULL => "Disk full\0".as_bytes(),
        graphdb_error_code_t::GRAPHDB_CANTOPEN => "Cannot open\0".as_bytes(),
        graphdb_error_code_t::GRAPHDB_PROTOCOL => "Protocol error\0".as_bytes(),
        graphdb_error_code_t::GRAPHDB_SCHEMA => "Schema error\0".as_bytes(),
        graphdb_error_code_t::GRAPHDB_TOOBIG => "Data too big\0".as_bytes(),
        graphdb_error_code_t::GRAPHDB_CONSTRAINT => "Constraint violation\0".as_bytes(),
        graphdb_error_code_t::GRAPHDB_MISMATCH => "Type mismatch\0".as_bytes(),
        graphdb_error_code_t::GRAPHDB_MISUSE => "Misuse\0".as_bytes(),
        graphdb_error_code_t::GRAPHDB_RANGE => "Out of range\0".as_bytes(),
        graphdb_error_code_t::GRAPHDB_NOT_IMPLEMENTED => "Not implemented\0".as_bytes(),
    }
}

/// 获取扩展错误码对应的描述消息（null 终止）
pub fn extended_error_code_to_message(code: graphdb_extended_error_code_t) -> &'static [u8] {
    match code {
        graphdb_extended_error_code_t::GRAPHDB_EXTENDED_NONE => "No error\0".as_bytes(),
        graphdb_extended_error_code_t::GRAPHDB_ERROR_SYNTAX => "Syntax error\0".as_bytes(),
        graphdb_extended_error_code_t::GRAPHDB_ERROR_SEMANTIC => "Semantic error\0".as_bytes(),
        graphdb_extended_error_code_t::GRAPHDB_ERROR_UNEXPECTED_TOKEN => "Unexpected token\0".as_bytes(),
        graphdb_extended_error_code_t::GRAPHDB_ERROR_UNTERMINATED_LITERAL => "Unterminated literal\0".as_bytes(),
        graphdb_extended_error_code_t::GRAPHDB_ERROR_TYPE_MISMATCH => "Type mismatch\0".as_bytes(),
        graphdb_extended_error_code_t::GRAPHDB_ERROR_DIVISION_BY_ZERO => "Division by zero\0".as_bytes(),
        graphdb_extended_error_code_t::GRAPHDB_ERROR_OUT_OF_RANGE => "Out of range\0".as_bytes(),
        graphdb_extended_error_code_t::GRAPHDB_ERROR_DUPLICATE_KEY => "Duplicate key\0".as_bytes(),
        graphdb_extended_error_code_t::GRAPHDB_ERROR_FOREIGN_KEY => "Foreign key constraint\0".as_bytes(),
        graphdb_extended_error_code_t::GRAPHDB_ERROR_NOT_NULL => "Not null constraint\0".as_bytes(),
        graphdb_extended_error_code_t::GRAPHDB_ERROR_UNIQUE => "Unique constraint\0".as_bytes(),
        graphdb_extended_error_code_t::GRAPHDB_ERROR_CHECK => "Check constraint\0".as_bytes(),
        graphdb_extended_error_code_t::GRAPHDB_ERROR_CONNECTION_LOST => "Connection lost\0".as_bytes(),
        graphdb_extended_error_code_t::GRAPHDB_ERROR_DEADLOCK => "Deadlock\0".as_bytes(),
        graphdb_extended_error_code_t::GRAPHDB_ERROR_LOCK_TIMEOUT => "Lock timeout\0".as_bytes(),
        graphdb_extended_error_code_t::GRAPHDB_ERROR_INVALID_VERTEX => "Invalid vertex\0".as_bytes(),
        graphdb_extended_error_code_t::GRAPHDB_ERROR_INVALID_EDGE => "Invalid edge\0".as_bytes(),
        graphdb_extended_error_code_t::GRAPHDB_ERROR_PATH_NOT_FOUND => "Path not found\0".as_bytes(),
    }
}

/// 获取最后一个错误消息（线程安全）
///
/// # 参数
/// - `msg`: 输出缓冲区
/// - `len`: 缓冲区长度
///
/// # 返回
/// - 实际写入的字符数（不包括 null 终止符）
#[no_mangle]
pub extern "C" fn graphdb_errmsg(
    msg: *mut std::ffi::c_char,
    len: usize,
) -> i32 {
    if msg.is_null() || len == 0 {
        return 0;
    }

    let message = LAST_ERROR_MESSAGE.with(|m| {
        m.borrow().as_ref().map(|s| s.clone()).unwrap_or_else(|| {
            CString::new("No error message").unwrap_or_else(|_| {
                CString::new("?").expect("Failed to create fallback error message")
            })
        })
    });
    
    let bytes = message.as_bytes_with_nul();
    let copy_len = std::cmp::min(len - 1, bytes.len() - 1);
    
    unsafe {
        std::ptr::copy_nonoverlapping(
            bytes.as_ptr() as *const std::ffi::c_char,
            msg,
            copy_len,
        );
        *msg.add(copy_len) = 0;
    }
    
    copy_len as i32
}

/// 获取错误码描述
///
/// # 参数
/// - `code`: 错误码
///
/// # 返回
/// - 错误描述字符串（静态生命周期）
#[no_mangle]
pub extern "C" fn graphdb_error_string(code: i32) -> *const std::ffi::c_char {
    let error_code = match code {
        0 => graphdb_error_code_t::GRAPHDB_OK,
        1 => graphdb_error_code_t::GRAPHDB_ERROR,
        2 => graphdb_error_code_t::GRAPHDB_INTERNAL,
        3 => graphdb_error_code_t::GRAPHDB_PERM,
        4 => graphdb_error_code_t::GRAPHDB_ABORT,
        5 => graphdb_error_code_t::GRAPHDB_BUSY,
        6 => graphdb_error_code_t::GRAPHDB_LOCKED,
        7 => graphdb_error_code_t::GRAPHDB_NOMEM,
        8 => graphdb_error_code_t::GRAPHDB_READONLY,
        9 => graphdb_error_code_t::GRAPHDB_INTERRUPT,
        10 => graphdb_error_code_t::GRAPHDB_IOERR,
        11 => graphdb_error_code_t::GRAPHDB_CORRUPT,
        12 => graphdb_error_code_t::GRAPHDB_NOTFOUND,
        13 => graphdb_error_code_t::GRAPHDB_FULL,
        14 => graphdb_error_code_t::GRAPHDB_CANTOPEN,
        15 => graphdb_error_code_t::GRAPHDB_PROTOCOL,
        16 => graphdb_error_code_t::GRAPHDB_SCHEMA,
        17 => graphdb_error_code_t::GRAPHDB_TOOBIG,
        18 => graphdb_error_code_t::GRAPHDB_CONSTRAINT,
        19 => graphdb_error_code_t::GRAPHDB_MISMATCH,
        20 => graphdb_error_code_t::GRAPHDB_MISUSE,
        21 => graphdb_error_code_t::GRAPHDB_RANGE,
        _ => graphdb_error_code_t::GRAPHDB_ERROR,
    };

    let desc = error_code_to_message(error_code);
    // 注意：这里返回的字符串是静态的，不需要释放
    desc.as_ptr() as *const std::ffi::c_char
}

/// 获取错误码对应的字符串描述（类似 SQLite 的 sqlite3_errstr）
///
/// # 参数
/// - `code`: 错误码
///
/// # 返回
/// - 错误描述字符串（静态生命周期，不需要释放）
#[no_mangle]
pub extern "C" fn graphdb_errstr(code: i32) -> *const std::ffi::c_char {
    let error_code = match code {
        0 => graphdb_error_code_t::GRAPHDB_OK,
        1 => graphdb_error_code_t::GRAPHDB_ERROR,
        2 => graphdb_error_code_t::GRAPHDB_INTERNAL,
        3 => graphdb_error_code_t::GRAPHDB_PERM,
        4 => graphdb_error_code_t::GRAPHDB_ABORT,
        5 => graphdb_error_code_t::GRAPHDB_BUSY,
        6 => graphdb_error_code_t::GRAPHDB_LOCKED,
        7 => graphdb_error_code_t::GRAPHDB_NOMEM,
        8 => graphdb_error_code_t::GRAPHDB_READONLY,
        9 => graphdb_error_code_t::GRAPHDB_INTERRUPT,
        10 => graphdb_error_code_t::GRAPHDB_IOERR,
        11 => graphdb_error_code_t::GRAPHDB_CORRUPT,
        12 => graphdb_error_code_t::GRAPHDB_NOTFOUND,
        13 => graphdb_error_code_t::GRAPHDB_FULL,
        14 => graphdb_error_code_t::GRAPHDB_CANTOPEN,
        15 => graphdb_error_code_t::GRAPHDB_PROTOCOL,
        16 => graphdb_error_code_t::GRAPHDB_SCHEMA,
        17 => graphdb_error_code_t::GRAPHDB_TOOBIG,
        18 => graphdb_error_code_t::GRAPHDB_CONSTRAINT,
        19 => graphdb_error_code_t::GRAPHDB_MISMATCH,
        20 => graphdb_error_code_t::GRAPHDB_MISUSE,
        21 => graphdb_error_code_t::GRAPHDB_RANGE,
        _ => graphdb_error_code_t::GRAPHDB_ERROR,
    };

    let desc = error_code_to_message(error_code);
    desc.as_ptr() as *const std::ffi::c_char
}

/// 获取最后的错误消息
///
/// # 返回
/// - 错误消息字符串指针（线程局部存储，不需要释放）
#[no_mangle]
pub extern "C" fn graphdb_get_last_error_message() -> *const std::ffi::c_char {
    LAST_ERROR_MESSAGE.with(|m| {
        match m.borrow().as_ref() {
            Some(s) => s.as_ptr() as *const std::ffi::c_char,
            None => std::ptr::null(),
        }
    })
}

/// 获取 SQL 错误位置（字符偏移量）
///
/// # 参数
/// - `session`: 会话句柄
///
/// # 返回
/// - 错误位置的字符偏移量，如果没有错误或无效会话返回 -1
#[no_mangle]
pub extern "C" fn graphdb_error_offset(session: *mut graphdb_session_t) -> std::ffi::c_int {
    if session.is_null() {
        return -1;
    }

    unsafe {
        let handle = &*(session as *mut crate::api::embedded::c_api::session::GraphDbSessionHandle);
        handle.last_error_offset.map(|o| o as std::ffi::c_int).unwrap_or(-1)
    }
}

/// 获取扩展错误码
///
/// # 参数
/// - `session`: 会话句柄
///
/// # 返回
/// - 扩展错误码，如果没有错误或无效会话返回 0 (GRAPHDB_EXTENDED_NONE)
#[no_mangle]
pub extern "C" fn graphdb_extended_errcode(
    session: *mut graphdb_session_t,
) -> std::ffi::c_int {
    if session.is_null() {
        return graphdb_extended_error_code_t::GRAPHDB_EXTENDED_NONE as std::ffi::c_int;
    }

    unsafe {
        let handle = &*(session as *mut crate::api::embedded::c_api::session::GraphDbSessionHandle);
        handle
            .last_extended_error
            .map(|e| e as std::ffi::c_int)
            .unwrap_or(graphdb_extended_error_code_t::GRAPHDB_EXTENDED_NONE as std::ffi::c_int)
    }
}
