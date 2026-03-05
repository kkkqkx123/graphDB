//! C API 核心类型定义
//!
//! 定义 C API 中使用的所有数据类型和常量

use std::ffi::{c_char, c_int, c_void};

/// 值类型
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum graphdb_value_type_t {
    /// 空值
    GRAPHDB_NULL = 0,
    /// 布尔值
    GRAPHDB_BOOL = 1,
    /// 整数
    GRAPHDB_INT = 2,
    /// 浮点数
    GRAPHDB_FLOAT = 3,
    /// 字符串
    GRAPHDB_STRING = 4,
    /// 列表
    GRAPHDB_LIST = 5,
    /// 映射
    GRAPHDB_MAP = 6,
    /// 顶点
    GRAPHDB_VERTEX = 7,
    /// 边
    GRAPHDB_EDGE = 8,
    /// 路径
    GRAPHDB_PATH = 9,
}

/// 数据库句柄（不透明指针）
#[repr(C)]
pub struct graphdb_t;

/// 会话句柄（不透明指针）
#[repr(C)]
pub struct graphdb_session_t;

/// 预编译语句句柄（不透明指针）
#[repr(C)]
pub struct graphdb_stmt_t;

/// 事务句柄（不透明指针）
#[repr(C)]
pub struct graphdb_txn_t;

/// 结果集句柄（不透明指针）
#[repr(C)]
pub struct graphdb_result_t;

/// 批量操作句柄（不透明指针）
#[repr(C)]
pub struct graphdb_batch_t;

/// 字符串结构
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct graphdb_string_t {
    /// 字符串数据
    pub data: *const c_char,
    /// 字符串长度
    pub len: usize,
}

/// 值结构
#[repr(C)]
#[derive(Clone, Copy)]
pub struct graphdb_value_t {
    /// 值类型
    pub type_: graphdb_value_type_t,
    /// 值数据
    pub data: graphdb_value_data_t,
}

impl std::fmt::Debug for graphdb_value_t {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("graphdb_value_t")
            .field("type_", &self.type_)
            .finish()
    }
}

/// 值数据联合体
#[repr(C)]
#[derive(Clone, Copy)]
pub union graphdb_value_data_t {
    /// 布尔值
    pub boolean: bool,
    /// 整数
    pub integer: i64,
    /// 浮点数
    pub floating: f64,
    /// 字符串
    pub string: graphdb_string_t,
    /// 指针
    pub ptr: *mut c_void,
}

/// 数据库配置
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct graphdb_config_t {
    /// 是否只读
    pub read_only: bool,
    /// 如果不存在是否创建
    pub create_if_missing: bool,
    /// 缓存大小（MB）
    pub cache_size_mb: c_int,
    /// 最大打开文件数
    pub max_open_files: c_int,
    /// 是否启用压缩
    pub enable_compression: bool,
}

impl Default for graphdb_config_t {
    fn default() -> Self {
        Self {
            read_only: false,
            create_if_missing: true,
            cache_size_mb: 256,
            max_open_files: 1000,
            enable_compression: true,
        }
    }
}
