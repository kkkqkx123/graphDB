//! 统一错误处理系统 for GraphDB
//!
//! ## 设计理念
//!
//! 1. **按需设计**：根据错误复杂度选择合适的结构
//!    - 核心错误（如表达式）使用结构化设计，保留完整错误链和位置信息
//!    - 简单错误（如事务、索引）使用枚举设计，简洁高效
//!
//! 2. **分层转换**：
//!    - 核心错误使用 `#[from]` 注解自动转换，保留完整错误信息
//!    - 外部错误使用自定义 `From` 实现转换为字符串，降低模块耦合
//!
//! 3. **统一接口**：`DBResult<T>` 提供统一的返回类型，简化错误传播

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// 统一的数据库错误类型
#[derive(Error, Debug)]
pub enum DBError {
    #[error("存储错误: {0}")]
    Storage(#[from] StorageError),

    #[error("查询错误: {0}")]
    Query(#[from] QueryError),

    #[error("表达式错误: {0}")]
    Expression(#[from] ExpressionError),

    #[error("计划节点访问错误: {0}")]
    Plan(#[from] PlanNodeVisitError),

    #[error("锁操作错误: {0}")]
    Lock(#[from] LockError),

    #[error("管理器错误: {0}")]
    Manager(#[from] ManagerError),

    #[error("验证错误: {0}")]
    Validation(String),

    #[error("IO错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("类型推导错误: {0}")]
    TypeDeduction(String),

    #[error("序列化错误: {0}")]
    Serialization(String),

    #[error("索引错误: {0}")]
    Index(String),

    #[error("事务错误: {0}")]
    Transaction(String),

    #[error("内部错误: {0}")]
    Internal(String),

    #[error("会话错误: {0}")]
    Session(#[from] SessionError),

    #[error("权限错误: {0}")]
    Permission(#[from] PermissionError),
}

/// 统一的结果类型
pub type DBResult<T> = Result<T, DBError>;

/// Manager操作结果类型
pub type ManagerResult<T> = Result<T, ManagerError>;

/// 存储层错误类型
///
/// 涵盖数据库底层存储操作相关的错误
#[derive(Error, Debug, Clone)]
pub enum StorageError {
    #[error("数据库错误: {0}")]
    DbError(String),
    #[error("序列化错误: {0}")]
    SerializationError(String),
    #[error("节点未找到: {0:?}")]
    NodeNotFound(crate::core::Value),
    #[error("边未找到: {0:?}")]
    EdgeNotFound(crate::core::Value),
    #[error("事务错误: {0}")]
    TransactionError(String),
    #[error("事务未找到: {0}")]
    TransactionNotFound(u64),
}

/// 查询层错误类型
///
/// 涵盖查询解析、验证和执行过程中的错误
#[derive(Error, Debug, Clone)]
pub enum QueryError {
    #[error("存储错误: {0}")]
    StorageError(String),
    #[error("解析错误: {0}")]
    ParseError(String),
    #[error("无效查询: {0}")]
    InvalidQuery(String),
    #[error("执行错误: {0}")]
    ExecutionError(String),
    #[error("表达式错误: {0}")]
    ExpressionError(String),
}

/// 表达式错误（结构化设计）
///
/// 包含错误类型、错误消息和可选的位置信息
/// 支持序列化/反序列化，用于跨模块传递
#[derive(Error, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpressionError {
    /// 错误类型
    pub error_type: ExpressionErrorType,
    /// 错误消息
    pub message: String,
    /// 错误位置
    pub position: Option<ExpressionPosition>,
}

/// 表达式错误类型枚举
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExpressionErrorType {
    /// 类型错误
    TypeError,
    /// 未定义变量
    UndefinedVariable,
    /// 未定义函数
    UndefinedFunction,
    /// 未知函数
    UnknownFunction,
    /// 函数错误
    FunctionError,
    /// 参数数量错误
    ArgumentCountError,
    /// 无效参数数量
    InvalidArgumentCount,
    /// 除零错误
    DivisionByZero,
    /// 溢出错误
    Overflow,
    /// 索引越界
    IndexOutOfBounds,
    /// 空值错误
    NullError,
    /// 语法错误
    SyntaxError,
    /// 无效操作
    InvalidOperation,
    /// 属性未找到
    PropertyNotFound,
    /// 运行时错误
    RuntimeError,
    /// 不支持的操作
    UnsupportedOperation,
    /// 类型转换错误
    TypeConversionError,
    /// 操作符错误
    OperatorError,
    /// 标签未找到
    LabelNotFound,
    /// 边未找到
    EdgeNotFound,
    /// 路径错误
    PathError,
    /// 范围错误
    RangeError,
    /// 聚合函数错误
    AggregateError,
    /// 验证错误
    ValidationError,
}

/// 表达式错误位置信息
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpressionPosition {
    /// 行号
    pub line: usize,
    /// 列号
    pub column: usize,
    /// 偏移量
    pub offset: usize,
    /// 长度
    pub length: usize,
}

impl ExpressionError {
    /// 创建新的表达式错误
    pub fn new(error_type: ExpressionErrorType, message: impl Into<String>) -> Self {
        Self {
            error_type,
            message: message.into(),
            position: None,
        }
    }

    /// 设置错误位置
    pub fn with_position(
        mut self,
        line: usize,
        column: usize,
        offset: usize,
        length: usize,
    ) -> Self {
        self.position = Some(ExpressionPosition {
            line,
            column,
            offset,
            length,
        });
        self
    }

    /// 创建类型错误
    pub fn type_error(message: impl Into<String>) -> Self {
        Self::new(ExpressionErrorType::TypeError, message)
    }

    /// 创建未定义变量错误
    pub fn undefined_variable(name: impl Into<String>) -> Self {
        Self::new(
            ExpressionErrorType::UndefinedVariable,
            format!("未定义的变量: {}", name.into()),
        )
    }

    /// 创建未定义函数错误
    pub fn undefined_function(name: impl Into<String>) -> Self {
        Self::new(
            ExpressionErrorType::UndefinedFunction,
            format!("未定义的函数: {}", name.into()),
        )
    }

    /// 创建参数数量错误
    pub fn argument_count_error(expected: usize, actual: usize) -> Self {
        Self::new(
            ExpressionErrorType::ArgumentCountError,
            format!("参数数量错误: 期望 {}, 实际 {}", expected, actual),
        )
    }

    /// 创建除零错误
    pub fn division_by_zero() -> Self {
        Self::new(ExpressionErrorType::DivisionByZero, "除零错误".to_string())
    }

    /// 创建溢出错误
    pub fn overflow(message: impl Into<String>) -> Self {
        Self::new(ExpressionErrorType::Overflow, message)
    }

    /// 创建索引越界错误
    pub fn index_out_of_bounds(index: isize, size: usize) -> Self {
        Self::new(
            ExpressionErrorType::IndexOutOfBounds,
            format!("索引越界: 索引 {}, 大小 {}", index, size),
        )
    }

    /// 创建空值错误
    pub fn null_error(message: impl Into<String>) -> Self {
        Self::new(ExpressionErrorType::NullError, message)
    }

    /// 创建语法错误
    pub fn syntax_error(message: impl Into<String>) -> Self {
        Self::new(ExpressionErrorType::SyntaxError, message)
    }

    /// 创建运行时错误
    pub fn runtime_error(message: impl Into<String>) -> Self {
        Self::new(ExpressionErrorType::RuntimeError, message)
    }

    /// 创建函数错误
    pub fn function_error(message: impl Into<String>) -> Self {
        Self::new(ExpressionErrorType::FunctionError, message)
    }

    /// 创建无效操作错误
    pub fn invalid_operation(message: impl Into<String>) -> Self {
        Self::new(ExpressionErrorType::InvalidOperation, message)
    }

    /// 创建属性未找到错误
    pub fn property_not_found(message: impl Into<String>) -> Self {
        Self::new(ExpressionErrorType::PropertyNotFound, message)
    }

    /// 创建未知函数错误
    pub fn unknown_function(name: impl Into<String>) -> Self {
        Self::new(
            ExpressionErrorType::UnknownFunction,
            format!("未知函数: {}", name.into()),
        )
    }

    /// 创建无效参数数量错误
    pub fn invalid_argument_count(name: impl Into<String>) -> Self {
        Self::new(
            ExpressionErrorType::InvalidArgumentCount,
            format!("无效参数数量: {}", name.into()),
        )
    }

    /// 创建不支持的操作错误
    pub fn unsupported_operation(
        operation: impl Into<String>,
        suggestion: impl Into<String>,
    ) -> Self {
        Self::new(
            ExpressionErrorType::UnsupportedOperation,
            format!(
                "不支持的操作: {}, 建议: {}",
                operation.into(),
                suggestion.into()
            ),
        )
    }

    /// 创建类型转换错误
    pub fn type_conversion_error(from_type: impl Into<String>, to_type: impl Into<String>) -> Self {
        Self::new(
            ExpressionErrorType::TypeConversionError,
            format!(
                "类型转换错误: 无法从 {} 转换为 {}",
                from_type.into(),
                to_type.into()
            ),
        )
    }

    /// 创建操作符错误
    pub fn operator_error(operator: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(
            ExpressionErrorType::OperatorError,
            format!("操作符错误: {}: {}", operator.into(), message.into()),
        )
    }

    /// 创建标签未找到错误
    pub fn label_not_found(label: impl Into<String>) -> Self {
        Self::new(
            ExpressionErrorType::LabelNotFound,
            format!("标签未找到: {}", label.into()),
        )
    }

    /// 创建边未找到错误
    pub fn edge_not_found(edge: impl Into<String>) -> Self {
        Self::new(
            ExpressionErrorType::EdgeNotFound,
            format!("边未找到: {}", edge.into()),
        )
    }

    /// 创建路径错误
    pub fn path_error(message: impl Into<String>) -> Self {
        Self::new(ExpressionErrorType::PathError, message)
    }

    /// 创建范围错误
    pub fn range_error(message: impl Into<String>) -> Self {
        Self::new(ExpressionErrorType::RangeError, message)
    }

    /// 创建聚合函数错误
    pub fn aggregate_error(message: impl Into<String>) -> Self {
        Self::new(ExpressionErrorType::AggregateError, message)
    }

    /// 创建验证错误
    pub fn validation_error(message: impl Into<String>) -> Self {
        Self::new(ExpressionErrorType::ValidationError, message)
    }
}

impl fmt::Display for ExpressionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.error_type, self.message)
    }
}

/// 计划节点访问错误类型
///
/// 涵盖查询计划遍历和验证过程中的错误
#[derive(Error, Debug, Clone)]
pub enum PlanNodeVisitError {
    #[error("访问错误: {0}")]
    VisitError(String),
    #[error("遍历错误: {0}")]
    TraversalError(String),
    #[error("验证错误: {0}")]
    ValidationError(String),
}

/// 锁操作错误类型
///
/// 涵盖并发控制中锁相关的错误
#[derive(Error, Debug, Clone)]
pub enum LockError {
    #[error("Mutex锁被污染: {reason}")]
    MutexPoisoned { reason: String },

    #[error("RwLock读锁被污染: {reason}")]
    RwLockReadPoisoned { reason: String },

    #[error("RwLock写锁被污染: {reason}")]
    RwLockWritePoisoned { reason: String },

    #[error("锁操作超时: {reason}")]
    LockTimeout { reason: String },
}

/// 管理器错误类型
///
/// 涵盖Schema管理器、索引管理器、存储客户端等Manager层的错误
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ManagerError {
    #[error("资源未找到: {0}")]
    NotFound(String),

    #[error("资源已存在: {0}")]
    AlreadyExists(String),

    #[error("无效输入: {0}")]
    InvalidInput(String),

    #[error("存储错误: {0}")]
    StorageError(String),

    #[error("Schema错误: {0}")]
    SchemaError(String),

    #[error("索引错误: {0}")]
    IndexError(String),

    #[error("事务错误: {0}")]
    TransactionError(String),

    #[error("连接错误: {0}")]
    ConnectionError(String),

    #[error("超时错误: {0}")]
    TimeoutError(String),

    #[error("权限错误: {0}")]
    PermissionError(String),

    #[error("其他错误: {0}")]
    Other(String),
}

impl ManagerError {
    /// 获取错误分类
    pub fn category(&self) -> ErrorCategory {
        match self {
            ManagerError::StorageError(_)
            | ManagerError::ConnectionError(_)
            | ManagerError::TimeoutError(_) => ErrorCategory::Retryable,
            _ => ErrorCategory::NonRetryable,
        }
    }

    /// 检查是否可重试
    pub fn is_retryable(&self) -> bool {
        matches!(self.category(), ErrorCategory::Retryable)
    }

    /// 创建未找到错误
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    /// 创建已存在错误
    pub fn already_exists(msg: impl Into<String>) -> Self {
        Self::AlreadyExists(msg.into())
    }

    /// 创建无效输入错误
    pub fn invalid_input(msg: impl Into<String>) -> Self {
        Self::InvalidInput(msg.into())
    }

    /// 创建存储错误
    pub fn storage_error(msg: impl Into<String>) -> Self {
        Self::StorageError(msg.into())
    }

    /// 创建Schema错误
    pub fn schema_error(msg: impl Into<String>) -> Self {
        Self::SchemaError(msg.into())
    }

    /// 创建索引错误
    pub fn index_error(msg: impl Into<String>) -> Self {
        Self::IndexError(msg.into())
    }

    /// 创建事务错误
    pub fn transaction_error(msg: impl Into<String>) -> Self {
        Self::TransactionError(msg.into())
    }

    /// 创建连接错误
    pub fn connection_error(msg: impl Into<String>) -> Self {
        Self::ConnectionError(msg.into())
    }

    /// 创建超时错误
    pub fn timeout_error(msg: impl Into<String>) -> Self {
        Self::TimeoutError(msg.into())
    }

    /// 创建权限错误
    pub fn permission_error(msg: impl Into<String>) -> Self {
        Self::PermissionError(msg.into())
    }
}

/// 错误分类
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// 可重试错误
    Retryable,
    /// 不可重试错误
    NonRetryable,
}

// ExpressionError 是本地定义，无需转换实现
// PlanNodeVisitError 的 From 实现已通过 #[from] 属性自动生成（第22行）

impl From<crate::query::visitor::TypeDeductionError> for DBError {
    fn from(err: crate::query::visitor::TypeDeductionError) -> Self {
        DBError::TypeDeduction(err.to_string())
    }
}

impl From<crate::graph::IndexError> for DBError {
    fn from(err: crate::graph::IndexError) -> Self {
        DBError::Index(err.to_string())
    }
}

impl From<crate::graph::TransactionError> for DBError {
    fn from(err: crate::graph::TransactionError) -> Self {
        DBError::Transaction(err.to_string())
    }
}

impl From<crate::common::FsError> for DBError {
    fn from(err: crate::common::FsError) -> Self {
        DBError::Internal(err.to_string())
    }
}

impl From<serde_json::Error> for DBError {
    fn from(err: serde_json::Error) -> Self {
        DBError::Serialization(err.to_string())
    }
}

impl From<crate::query::context::validate::schema::SchemaValidationError> for DBError {
    fn from(err: crate::query::context::validate::schema::SchemaValidationError) -> Self {
        DBError::Validation(err.to_string())
    }
}

impl From<crate::query::planner::planner::PlannerError> for DBError {
    fn from(err: crate::query::planner::planner::PlannerError) -> Self {
        DBError::Query(QueryError::ExecutionError(err.to_string()))
    }
}

impl From<crate::query::optimizer::optimizer::OptimizerError> for DBError {
    fn from(err: crate::query::optimizer::optimizer::OptimizerError) -> Self {
        DBError::Query(QueryError::ExecutionError(err.to_string()))
    }
}

impl From<crate::query::parser::lexer::LexError> for DBError {
    fn from(err: crate::query::parser::lexer::LexError) -> Self {
        DBError::Query(QueryError::ParseError(err.to_string()))
    }
}

/// 会话相关错误
#[derive(Error, Debug, Clone)]
pub enum SessionError {
    #[error("会话不存在: {0}")]
    SessionNotFound(i64),
    
    #[error("权限不足，无法执行此操作")]
    PermissionDenied,
    
    #[error("会话已过期")]
    SessionExpired,
    
    #[error("超过最大连接数限制")]
    MaxConnectionsExceeded,
    
    #[error("查询不存在: {0}")]
    QueryNotFound(u32),
    
    #[error("无法终止会话: {0}")]
    KillSessionFailed(String),
    
    #[error("会话管理器错误: {0}")]
    ManagerError(String),
}

/// 权限相关错误
#[derive(Error, Debug, Clone)]
pub enum PermissionError {
    #[error("权限不足")]
    InsufficientPermission,
    
    #[error("角色不存在: {0}")]
    RoleNotFound(String),
    
    #[error("无法授予角色: {0}")]
    GrantRoleFailed(String),
    
    #[error("无法撤销角色: {0}")]
    RevokeRoleFailed(String),
    
    #[error("用户不存在: {0}")]
    UserNotFound(String),
}

/// 类型别名，用于向后兼容
pub type GraphDBResult<T> = DBResult<T>;

/// 会话操作结果类型别名
pub type SessionResult<T> = Result<T, SessionError>;

/// 权限操作结果类型别名
pub type PermissionResult<T> = Result<T, PermissionError>;

/// 查询操作结果类型别名
pub type QueryResult<T> = Result<T, SessionError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dberror_creation() {
        let storage_err = StorageError::NodeNotFound(crate::core::Value::Int(42));
        let db_err: DBError = storage_err.into();
        assert!(matches!(db_err, DBError::Storage(_)));
    }

    #[test]
    fn test_error_conversion() {
        let query_err = QueryError::ParseError("test error".to_string());
        let db_err: DBError = query_err.into();
        assert!(matches!(db_err, DBError::Query(_)));
    }
}
