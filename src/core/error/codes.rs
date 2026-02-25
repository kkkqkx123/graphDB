//! 对外错误码定义
//!
//! 本模块定义标准化的错误码体系，用于：
//! - 客户端响应
//! - API 返回
//! - 协议序列化
//!
//! 错误码格式: XXYY
//! - XX: 错误类别 (00=成功, 01=语法, 02=执行, 03=验证, 04=权限, 05=资源, 09=系统)
//! - YY: 具体错误

use serde::{Deserialize, Serialize};

/// 对外错误码 - 用于客户端响应
///
/// 设计原则：
/// 1. 稳定性：错误码一旦定义不应随意修改，保证客户端兼容性
/// 2. 精简性：只暴露必要的错误信息，不包含内部实现细节
/// 3. 标准化：遵循 HTTP/GraphQL 等常见错误码设计规范
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorCode {
    // ==================== 成功 (00xx) ====================
    Success = 0,

    // ==================== 语法错误 (01xx) ====================
    /// 通用语法错误
    SyntaxError = 100,
    /// 解析错误
    ParseError = 101,
    /// 无效语句
    InvalidStatement = 102,
    /// 缺少必要参数
    MissingParameter = 103,

    // ==================== 执行错误 (02xx) ====================
    /// 通用执行错误
    ExecutionError = 200,
    /// 执行超时
    Timeout = 201,
    /// 资源不足
    ResourceExhausted = 202,
    /// 并发冲突
    Conflict = 203,
    /// 死锁检测
    Deadlock = 204,

    // ==================== 验证错误 (03xx) ====================
    /// 通用验证错误
    ValidationError = 300,
    /// 类型错误
    TypeError = 301,
    /// 无效输入
    InvalidInput = 302,
    /// 约束违反
    ConstraintViolation = 303,

    // ==================== 权限错误 (04xx) ====================
    /// 权限不足
    PermissionDenied = 400,
    /// 未认证
    Unauthorized = 401,
    /// 禁止访问
    Forbidden = 403,

    // ==================== 资源错误 (05xx) ====================
    /// 资源未找到
    ResourceNotFound = 500,
    /// 资源已存在
    ResourceAlreadyExists = 501,
    /// 资源不可用
    ResourceUnavailable = 502,

    // ==================== 系统错误 (09xx) ====================
    /// 内部服务器错误
    InternalError = 900,
    /// 服务不可用
    ServiceUnavailable = 901,
    /// 未知错误
    Unknown = 999,
}

impl ErrorCode {
    /// 获取错误码的 i32 值
    pub fn as_i32(&self) -> i32 {
        *self as i32
    }

    /// 根据 i32 值获取错误码
    pub fn from_i32(code: i32) -> Option<Self> {
        match code {
            0 => Some(ErrorCode::Success),
            100 => Some(ErrorCode::SyntaxError),
            101 => Some(ErrorCode::ParseError),
            102 => Some(ErrorCode::InvalidStatement),
            103 => Some(ErrorCode::MissingParameter),
            200 => Some(ErrorCode::ExecutionError),
            201 => Some(ErrorCode::Timeout),
            202 => Some(ErrorCode::ResourceExhausted),
            203 => Some(ErrorCode::Conflict),
            204 => Some(ErrorCode::Deadlock),
            300 => Some(ErrorCode::ValidationError),
            301 => Some(ErrorCode::TypeError),
            302 => Some(ErrorCode::InvalidInput),
            303 => Some(ErrorCode::ConstraintViolation),
            400 => Some(ErrorCode::PermissionDenied),
            401 => Some(ErrorCode::Unauthorized),
            403 => Some(ErrorCode::Forbidden),
            500 => Some(ErrorCode::ResourceNotFound),
            501 => Some(ErrorCode::ResourceAlreadyExists),
            502 => Some(ErrorCode::ResourceUnavailable),
            900 => Some(ErrorCode::InternalError),
            901 => Some(ErrorCode::ServiceUnavailable),
            999 => Some(ErrorCode::Unknown),
            _ => None,
        }
    }

    /// 获取错误类别
    pub fn category(&self) -> ErrorCategory {
        match self.as_i32() {
            0 => ErrorCategory::Success,
            100..=199 => ErrorCategory::Syntax,
            200..=299 => ErrorCategory::Execution,
            300..=399 => ErrorCategory::Validation,
            400..=499 => ErrorCategory::Permission,
            500..=599 => ErrorCategory::Resource,
            900..=999 => ErrorCategory::System,
            _ => ErrorCategory::Unknown,
        }
    }

    /// 获取默认的错误消息
    pub fn default_message(&self) -> &'static str {
        match self {
            ErrorCode::Success => "成功",
            ErrorCode::SyntaxError => "语法错误",
            ErrorCode::ParseError => "解析错误",
            ErrorCode::InvalidStatement => "无效语句",
            ErrorCode::MissingParameter => "缺少必要参数",
            ErrorCode::ExecutionError => "执行错误",
            ErrorCode::Timeout => "执行超时",
            ErrorCode::ResourceExhausted => "资源不足",
            ErrorCode::Conflict => "并发冲突",
            ErrorCode::Deadlock => "死锁检测",
            ErrorCode::ValidationError => "验证错误",
            ErrorCode::TypeError => "类型错误",
            ErrorCode::InvalidInput => "无效输入",
            ErrorCode::ConstraintViolation => "约束违反",
            ErrorCode::PermissionDenied => "权限不足",
            ErrorCode::Unauthorized => "未认证",
            ErrorCode::Forbidden => "禁止访问",
            ErrorCode::ResourceNotFound => "资源未找到",
            ErrorCode::ResourceAlreadyExists => "资源已存在",
            ErrorCode::ResourceUnavailable => "资源不可用",
            ErrorCode::InternalError => "内部服务器错误",
            ErrorCode::ServiceUnavailable => "服务不可用",
            ErrorCode::Unknown => "未知错误",
        }
    }

    /// 判断是否为成功状态
    pub fn is_success(&self) -> bool {
        matches!(self, ErrorCode::Success)
    }

    /// 判断是否为客户端错误 (4xx 类错误)
    pub fn is_client_error(&self) -> bool {
        let code = self.as_i32();
        (100..=499).contains(&code)
    }

    /// 判断是否为服务器错误 (5xx/9xx 类错误)
    pub fn is_server_error(&self) -> bool {
        let code = self.as_i32();
        (500..=599).contains(&code) || (900..=999).contains(&code)
    }

    /// 判断错误是否可重试
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            ErrorCode::Timeout
                | ErrorCode::Conflict
                | ErrorCode::Deadlock
                | ErrorCode::ResourceExhausted
                | ErrorCode::ServiceUnavailable
        )
    }
}

impl Default for ErrorCode {
    fn default() -> Self {
        ErrorCode::Success
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.as_i32(), self.default_message())
    }
}

/// 错误类别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCategory {
    Success,
    Syntax,
    Execution,
    Validation,
    Permission,
    Resource,
    System,
    Unknown,
}

impl ErrorCategory {
    /// 获取类别的 HTTP 状态码映射
    pub fn to_http_status(&self) -> u16 {
        match self {
            ErrorCategory::Success => 200,
            ErrorCategory::Syntax => 400,
            ErrorCategory::Execution => 500,
            ErrorCategory::Validation => 422,
            ErrorCategory::Permission => 403,
            ErrorCategory::Resource => 404,
            ErrorCategory::System => 500,
            ErrorCategory::Unknown => 500,
        }
    }
}

/// 对外错误信息 - 用于序列化到响应中
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicError {
    /// 错误码
    pub code: ErrorCode,
    /// 错误消息
    pub message: String,
}

impl PublicError {
    /// 创建新的对外错误
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    /// 使用默认消息创建错误
    pub fn with_default_message(code: ErrorCode) -> Self {
        Self {
            code,
            message: code.default_message().to_string(),
        }
    }

    /// 创建成功响应
    pub fn success() -> Self {
        Self {
            code: ErrorCode::Success,
            message: "成功".to_string(),
        }
    }
}

/// 内部错误到对外错误的转换 trait
///
/// 实现此 trait 可以将内部错误转换为对外错误，过滤敏感信息
pub trait ToPublicError {
    /// 转换为对外错误
    fn to_public_error(&self) -> PublicError;

    /// 获取对外错误码
    fn to_error_code(&self) -> ErrorCode;

    /// 获取对外错误消息（过滤敏感信息）
    fn to_public_message(&self) -> String;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_as_i32() {
        assert_eq!(ErrorCode::Success.as_i32(), 0);
        assert_eq!(ErrorCode::SyntaxError.as_i32(), 100);
        assert_eq!(ErrorCode::InternalError.as_i32(), 900);
    }

    #[test]
    fn test_error_code_from_i32() {
        assert_eq!(ErrorCode::from_i32(0), Some(ErrorCode::Success));
        assert_eq!(ErrorCode::from_i32(100), Some(ErrorCode::SyntaxError));
        assert_eq!(ErrorCode::from_i32(999), Some(ErrorCode::Unknown));
        assert_eq!(ErrorCode::from_i32(12345), None);
    }

    #[test]
    fn test_error_code_category() {
        assert_eq!(ErrorCode::Success.category(), ErrorCategory::Success);
        assert_eq!(ErrorCode::SyntaxError.category(), ErrorCategory::Syntax);
        assert_eq!(ErrorCode::ExecutionError.category(), ErrorCategory::Execution);
        assert_eq!(ErrorCode::InternalError.category(), ErrorCategory::System);
    }

    #[test]
    fn test_error_code_is_success() {
        assert!(ErrorCode::Success.is_success());
        assert!(!ErrorCode::SyntaxError.is_success());
        assert!(!ErrorCode::InternalError.is_success());
    }

    #[test]
    fn test_error_code_is_retryable() {
        assert!(ErrorCode::Timeout.is_retryable());
        assert!(ErrorCode::Conflict.is_retryable());
        assert!(ErrorCode::Deadlock.is_retryable());
        assert!(!ErrorCode::SyntaxError.is_retryable());
        assert!(!ErrorCode::PermissionDenied.is_retryable());
    }

    #[test]
    fn test_public_error() {
        let err = PublicError::new(ErrorCode::ResourceNotFound, "用户不存在".to_string());
        assert_eq!(err.code, ErrorCode::ResourceNotFound);
        assert_eq!(err.message, "用户不存在");

        let default_err = PublicError::with_default_message(ErrorCode::Timeout);
        assert_eq!(default_err.code, ErrorCode::Timeout);
        assert_eq!(default_err.message, "执行超时");
    }
}
