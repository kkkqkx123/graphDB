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
    /// 二进制数据
    GRAPHDB_BLOB = 10,
}

/// 二进制数据结构
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct graphdb_blob_t {
    /// 数据指针
    pub data: *const u8,
    /// 数据长度
    pub len: usize,
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
    /// 二进制数据
    pub blob: graphdb_blob_t,
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

/// 数据库打开标志
pub const GRAPHDB_OPEN_READONLY: c_int = 0x00000001;
pub const GRAPHDB_OPEN_READWRITE: c_int = 0x00000002;
pub const GRAPHDB_OPEN_CREATE: c_int = 0x00000004;
pub const GRAPHDB_OPEN_NOMUTEX: c_int = 0x00008000;
pub const GRAPHDB_OPEN_FULLMUTEX: c_int = 0x00010000;
pub const GRAPHDB_OPEN_SHAREDCACHE: c_int = 0x00020000;
pub const GRAPHDB_OPEN_PRIVATECACHE: c_int = 0x00040000;

/// SQL 追踪回调类型
#[allow(non_camel_case_types)]
pub type graphdb_trace_callback = Option<extern "C" fn(sql: *const c_char, user_data: *mut c_void)>;

/// 钩子回调类型
#[allow(non_camel_case_types)]
pub type graphdb_commit_hook_callback = Option<extern "C" fn(user_data: *mut c_void) -> c_int>;
#[allow(non_camel_case_types)]
pub type graphdb_rollback_hook_callback = Option<extern "C" fn(user_data: *mut c_void)>;
#[allow(non_camel_case_types)]
pub type graphdb_update_hook_callback = Option<extern "C" fn(
    user_data: *mut c_void,
    operation: c_int,
    database: *const c_char,
    table: *const c_char,
    rowid: i64,
)>;

/// 钩子类型常量
pub const GRAPHDB_HOOK_INSERT: c_int = 1;
pub const GRAPHDB_HOOK_UPDATE: c_int = 2;
pub const GRAPHDB_HOOK_DELETE: c_int = 3;

/// 扩展错误码
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum graphdb_extended_error_code_t {
    /// 无扩展错误
    GRAPHDB_EXTENDED_NONE = 0,

    // 解析相关 (1000-1099)
    GRAPHDB_ERROR_SYNTAX = 1000,
    GRAPHDB_ERROR_SEMANTIC = 1001,
    GRAPHDB_ERROR_UNEXPECTED_TOKEN = 1002,
    GRAPHDB_ERROR_UNTERMINATED_LITERAL = 1003,

    // 类型相关 (1100-1199)
    GRAPHDB_ERROR_TYPE_MISMATCH = 1100,
    GRAPHDB_ERROR_DIVISION_BY_ZERO = 1101,
    GRAPHDB_ERROR_OUT_OF_RANGE = 1102,

    // 约束相关 (1200-1299)
    GRAPHDB_ERROR_DUPLICATE_KEY = 1200,
    GRAPHDB_ERROR_FOREIGN_KEY = 1201,
    GRAPHDB_ERROR_NOT_NULL = 1202,
    GRAPHDB_ERROR_UNIQUE = 1203,
    GRAPHDB_ERROR_CHECK = 1204,

    // 并发相关 (1300-1399)
    GRAPHDB_ERROR_CONNECTION_LOST = 1300,
    GRAPHDB_ERROR_DEADLOCK = 1301,
    GRAPHDB_ERROR_LOCK_TIMEOUT = 1302,

    // 图相关 (1400-1499)
    GRAPHDB_ERROR_INVALID_VERTEX = 1400,
    GRAPHDB_ERROR_INVALID_EDGE = 1401,
    GRAPHDB_ERROR_PATH_NOT_FOUND = 1402,
}
