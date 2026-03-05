//! C API 错误处理
//!
//! 提供错误码转换和错误信息管理功能

use crate::api::core::CoreError;

/// 错误码
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
}

/// 从核心错误转换为 C 错误码
pub fn error_code_from_core_error(error: &CoreError) -> i32 {
    match error {
        CoreError::StorageError(_) => graphdb_error_code_t::GRAPHDB_IOERR as i32,
        CoreError::QueryExecutionFailed(_) => graphdb_error_code_t::GRAPHDB_ERROR as i32,
        CoreError::TransactionFailed(_) => graphdb_error_code_t::GRAPHDB_ABORT as i32,
        CoreError::SchemaOperationFailed(_) => graphdb_error_code_t::GRAPHDB_SCHEMA as i32,
        CoreError::Internal(_) => graphdb_error_code_t::GRAPHDB_INTERNAL as i32,
        CoreError::NotFound(_) => graphdb_error_code_t::GRAPHDB_NOTFOUND as i32,
        CoreError::InvalidParameter(_) => graphdb_error_code_t::GRAPHDB_MISUSE as i32,
    }
}

/// 获取错误码对应的描述字符串
pub fn error_code_to_string(code: graphdb_error_code_t) -> &'static str {
    match code {
        graphdb_error_code_t::GRAPHDB_OK => "成功",
        graphdb_error_code_t::GRAPHDB_ERROR => "一般错误",
        graphdb_error_code_t::GRAPHDB_INTERNAL => "内部错误",
        graphdb_error_code_t::GRAPHDB_PERM => "权限被拒绝",
        graphdb_error_code_t::GRAPHDB_ABORT => "操作被中止",
        graphdb_error_code_t::GRAPHDB_BUSY => "数据库忙",
        graphdb_error_code_t::GRAPHDB_LOCKED => "数据库被锁定",
        graphdb_error_code_t::GRAPHDB_NOMEM => "内存不足",
        graphdb_error_code_t::GRAPHDB_READONLY => "只读",
        graphdb_error_code_t::GRAPHDB_INTERRUPT => "操作被中断",
        graphdb_error_code_t::GRAPHDB_IOERR => "IO 错误",
        graphdb_error_code_t::GRAPHDB_CORRUPT => "数据损坏",
        graphdb_error_code_t::GRAPHDB_NOTFOUND => "未找到",
        graphdb_error_code_t::GRAPHDB_FULL => "磁盘已满",
        graphdb_error_code_t::GRAPHDB_CANTOPEN => "无法打开",
        graphdb_error_code_t::GRAPHDB_PROTOCOL => "协议错误",
        graphdb_error_code_t::GRAPHDB_SCHEMA => "模式错误",
        graphdb_error_code_t::GRAPHDB_TOOBIG => "数据过大",
        graphdb_error_code_t::GRAPHDB_CONSTRAINT => "约束违反",
        graphdb_error_code_t::GRAPHDB_MISMATCH => "类型不匹配",
        graphdb_error_code_t::GRAPHDB_MISUSE => "误用",
        graphdb_error_code_t::GRAPHDB_RANGE => "超出范围",
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

    let message = "错误信息功能待实现";
    let c_message = std::ffi::CString::new(message).unwrap_or_default();
    let bytes = c_message.as_bytes_with_nul();

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

    let desc = error_code_to_string(error_code);
    // 注意：这里返回的字符串是静态的，不需要释放
    desc.as_ptr() as *const std::ffi::c_char
}
