//! C API 错误处理
//!
//! 提供错误码转换和错误信息管理功能

use crate::api::embedded::c_api::types::graphdb_error_code_t;
use crate::api::core::CoreError;
use std::ffi::CString;

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

/// 创建 C 字符串错误信息
pub fn create_error_string(error: &CoreError) -> CString {
    let error_msg = format!("{}", error);
    CString::new(error_msg).unwrap_or_else(|_| CString::new("错误信息格式化失败").unwrap())
}
