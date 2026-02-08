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

pub use crate::core::types::DataType;

/// 统一的数据库错误类型
#[derive(Error, Debug, Clone)]
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
    Io(String),

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

    #[error("全文索引错误: {0}")]
    FulltextIndex(#[from] crate::index::FulltextIndexError),
}

/// 统一的结果类型
pub type DBResult<T> = Result<T, DBError>;

/// Manager操作结果类型
pub type ManagerResult<T> = Result<T, ManagerError>;

/// 存储层结果类型
pub type StorageResult<T> = Result<T, StorageError>;

/// 存储层错误类型
///
/// 涵盖数据库底层存储操作相关的错误
#[derive(Error, Debug, Clone)]
pub enum StorageError {
    #[error("数据库错误: {0}")]
    DbError(String),
    #[error("存储错误: {0}")]
    StorageError(String),
    #[error("序列化错误: {0}")]
    SerializeError(String),
    #[error("反序列化错误: {0}")]
    DeserializeError(String),
    #[error("节点未找到: {0:?}")]
    NodeNotFound(crate::core::Value),
    #[error("边未找到: {0:?}")]
    EdgeNotFound(crate::core::Value),
    #[error("事务错误: {0}")]
    TransactionError(String),
    #[error("事务未找到: {0}")]
    TransactionNotFound(u64),
    #[error("操作不支持: {0}")]
    NotSupported(String),
    #[error("冲突错误: {0}")]
    Conflict(String),
    #[error("锁错误: {0}")]
    LockError(String),
    #[error("锁超时: {0}")]
    LockTimeout(String),
    #[error("死锁检测")]
    Deadlock,
    #[error("连接错误: {0}")]
    ConnectionError(String),
    #[error("IO错误: {0}")]
    IOError(String),
    #[error("未找到: {0}")]
    NotFound(String),
    #[error("已存在: {0}")]
    AlreadyExists(String),
    #[error("无效输入: {0}")]
    InvalidInput(String),
    #[error("索引错误: {0}")]
    IndexError(String),
    #[error("解析错误: {0}")]
    ParseError(String),
}

impl StorageError {
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            StorageError::LockTimeout(_) | StorageError::Deadlock | StorageError::ConnectionError(_)
        )
    }
}

impl From<std::io::Error> for StorageError {
    fn from(e: std::io::Error) -> Self {
        StorageError::DbError(e.to_string())
    }
}

impl From<redb::Error> for StorageError {
    fn from(e: redb::Error) -> Self {
        StorageError::DbError(e.to_string())
    }
}

impl From<String> for StorageError {
    fn from(s: String) -> Self {
        StorageError::DbError(s)
    }
}

impl From<&str> for StorageError {
    fn from(s: &str) -> Self {
        StorageError::DbError(s.to_string())
    }
}

impl<T> From<std::sync::PoisonError<T>> for StorageError {
    fn from(e: std::sync::PoisonError<T>) -> Self {
        StorageError::LockError(e.to_string())
    }
}

/// 查询层错误类型
///
/// 涵盖查询解析、验证和执行过程中的错误
#[derive(Error, Debug, Clone, PartialEq)]
pub enum QueryError {
    #[error("存储错误: {0}")]
    StorageError(String),

    #[error("解析错误: {0}")]
    ParseError(String),

    #[error("规划错误: {0}")]
    PlanningError(String),

    #[error("优化错误: {0}")]
    OptimizationError(String),

    #[error("无效查询: {0}")]
    InvalidQuery(String),

    #[error("执行错误: {0}")]
    ExecutionError(String),

    #[error("表达式错误: {0}")]
    ExpressionError(String),
}

impl QueryError {
    pub fn pipeline_parse_error<E: std::error::Error>(e: E) -> Self {
        QueryError::ParseError(e.to_string())
    }
    
    pub fn pipeline_validation_error<E: std::error::Error>(e: E) -> Self {
        QueryError::InvalidQuery(e.to_string())
    }
    
    pub fn pipeline_planning_error<E: std::error::Error>(e: E) -> Self {
        QueryError::PlanningError(e.to_string())
    }
    
    pub fn pipeline_optimization_error<E: std::error::Error>(e: E) -> Self {
        QueryError::OptimizationError(e.to_string())
    }
    
    pub fn pipeline_execution_error<E: std::error::Error>(e: E) -> Self {
        QueryError::ExecutionError(e.to_string())
    }
    
    pub fn pipeline_error(phase: &str, message: String) -> Self {
        match phase {
            "parse" => QueryError::ParseError(message),
            "validate" | "validation" => QueryError::InvalidQuery(message),
            "plan" | "planning" => QueryError::PlanningError(message),
            "optimize" | "optimization" => QueryError::OptimizationError(message),
            "execute" | "execution" => QueryError::ExecutionError(message),
            _ => QueryError::ExecutionError(format!("[{}] {}", phase, message)),
        }
    }
}

impl From<StorageError> for QueryError {
    fn from(e: StorageError) -> Self {
        QueryError::StorageError(e.to_string())
    }
}

impl From<DBError> for QueryError {
    fn from(e: DBError) -> Self {
        match e {
            DBError::Query(qe) => qe,
            DBError::Storage(se) => QueryError::StorageError(se.to_string()),
            DBError::Expression(expression) => QueryError::ExpressionError(expression.to_string()),
            DBError::Plan(plan) => QueryError::ExecutionError(plan.to_string()),
            DBError::Lock(lock) => QueryError::ExecutionError(lock.to_string()),
            DBError::Manager(manager) => QueryError::ExecutionError(manager.to_string()),
            DBError::Validation(msg) => QueryError::InvalidQuery(msg),
            DBError::Io(io) => QueryError::ExecutionError(io.to_string()),
            DBError::TypeDeduction(msg) => QueryError::ExecutionError(msg),
            DBError::Serialization(msg) => QueryError::ExecutionError(msg),
            DBError::Index(msg) => QueryError::ExecutionError(msg),
            DBError::Transaction(msg) => QueryError::ExecutionError(msg),
            DBError::Internal(msg) => QueryError::ExecutionError(msg),
            DBError::Session(session) => QueryError::ExecutionError(session.to_string()),
            DBError::Permission(permission) => QueryError::ExecutionError(permission.to_string()),
            DBError::FulltextIndex(ft) => QueryError::ExecutionError(ft.to_string()),
        }
    }
}

impl From<std::io::Error> for QueryError {
    fn from(e: std::io::Error) -> Self {
        QueryError::ExecutionError(e.to_string())
    }
}

impl From<LockError> for QueryError {
    fn from(e: LockError) -> Self {
        QueryError::ExecutionError(e.to_string())
    }
}

impl From<ManagerError> for QueryError {
    fn from(e: ManagerError) -> Self {
        QueryError::ExecutionError(e.to_string())
    }
}

impl From<SessionError> for QueryError {
    fn from(e: SessionError) -> Self {
        QueryError::ExecutionError(e.to_string())
    }
}

impl From<PermissionError> for QueryError {
    fn from(e: PermissionError) -> Self {
        QueryError::ExecutionError(e.to_string())
    }
}

impl From<ExpressionError> for QueryError {
    fn from(e: ExpressionError) -> Self {
        QueryError::ExpressionError(e.to_string())
    }
}

impl From<ExpressionErrorType> for QueryError {
    fn from(e: ExpressionErrorType) -> Self {
        QueryError::ExpressionError(e.to_string())
    }
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

impl std::fmt::Display for ExpressionErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpressionErrorType::TypeError => write!(f, "类型错误"),
            ExpressionErrorType::UndefinedVariable => write!(f, "未定义变量"),
            ExpressionErrorType::UndefinedFunction => write!(f, "未定义函数"),
            ExpressionErrorType::UnknownFunction => write!(f, "未知函数"),
            ExpressionErrorType::FunctionError => write!(f, "函数错误"),
            ExpressionErrorType::ArgumentCountError => write!(f, "参数数量错误"),
            ExpressionErrorType::InvalidArgumentCount => write!(f, "无效参数数量"),
            ExpressionErrorType::DivisionByZero => write!(f, "除零错误"),
            ExpressionErrorType::Overflow => write!(f, "溢出错误"),
            ExpressionErrorType::IndexOutOfBounds => write!(f, "索引越界"),
            ExpressionErrorType::NullError => write!(f, "空值错误"),
            ExpressionErrorType::SyntaxError => write!(f, "语法错误"),
            ExpressionErrorType::InvalidOperation => write!(f, "无效操作"),
            ExpressionErrorType::PropertyNotFound => write!(f, "属性未找到"),
            ExpressionErrorType::RuntimeError => write!(f, "运行时错误"),
            ExpressionErrorType::UnsupportedOperation => write!(f, "不支持的操作"),
            ExpressionErrorType::TypeConversionError => write!(f, "类型转换错误"),
            ExpressionErrorType::OperatorError => write!(f, "操作符错误"),
            ExpressionErrorType::LabelNotFound => write!(f, "标签未找到"),
            ExpressionErrorType::EdgeNotFound => write!(f, "边未找到"),
            ExpressionErrorType::PathError => write!(f, "路径错误"),
            ExpressionErrorType::RangeError => write!(f, "范围错误"),
            ExpressionErrorType::AggregateError => write!(f, "聚合函数错误"),
            ExpressionErrorType::ValidationError => write!(f, "验证错误"),
        }
    }
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

impl From<crate::index::IndexError> for DBError {
    fn from(err: crate::index::IndexError) -> Self {
        DBError::Index(err.to_string())
    }
}

impl From<serde_json::Error> for DBError {
    fn from(err: serde_json::Error) -> Self {
        DBError::Serialization(err.to_string())
    }
}

impl From<crate::query::planner::planner::PlannerError> for DBError {
    fn from(err: crate::query::planner::planner::PlannerError) -> Self {
        DBError::Query(QueryError::ExecutionError(err.to_string()))
    }
}

impl From<crate::query::optimizer::OptimizerError> for DBError {
    fn from(err: crate::query::optimizer::OptimizerError) -> Self {
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

/// 验证错误类型枚举（统一版本）
///
/// 提供完整的验证错误分类，与QueryError::InvalidQuery对应
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ValidationErrorType {
    SyntaxError,
    SemanticError,
    TypeError,
    AliasError,
    AggregateError,
    PaginationError,
    ExpressionDepthError,
    VariableNotFound,
    CyclicReference,
    DivisionByZero,
    TooManyArguments,
    TooManyElements,
    DuplicateKey,
}

impl fmt::Display for ValidationErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationErrorType::SyntaxError => write!(f, "语法错误"),
            ValidationErrorType::SemanticError => write!(f, "语义错误"),
            ValidationErrorType::TypeError => write!(f, "类型错误"),
            ValidationErrorType::AliasError => write!(f, "别名错误"),
            ValidationErrorType::AggregateError => write!(f, "聚合函数错误"),
            ValidationErrorType::PaginationError => write!(f, "分页错误"),
            ValidationErrorType::ExpressionDepthError => write!(f, "表达式深度错误"),
            ValidationErrorType::VariableNotFound => write!(f, "变量未找到"),
            ValidationErrorType::CyclicReference => write!(f, "循环引用"),
            ValidationErrorType::DivisionByZero => write!(f, "除零错误"),
            ValidationErrorType::TooManyArguments => write!(f, "参数过多"),
            ValidationErrorType::TooManyElements => write!(f, "元素过多"),
            ValidationErrorType::DuplicateKey => write!(f, "重复键"),
        }
    }
}

impl From<ValidationErrorType> for QueryError {
    fn from(e: ValidationErrorType) -> Self {
        match e {
            ValidationErrorType::SyntaxError => QueryError::ParseError(e.to_string()),
            _ => QueryError::InvalidQuery(e.to_string()),
        }
    }
}

/// 统一验证错误结构
///
/// 包含错误类型、错误消息和位置信息
/// 支持序列化/反序列化，用于跨模块传递
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationError {
    pub message: String,
    pub error_type: ValidationErrorType,
    pub context: Option<String>,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub query_position: Option<usize>,
}

impl ValidationError {
    pub fn new(message: String, error_type: ValidationErrorType) -> Self {
        Self {
            message,
            error_type,
            context: None,
            line: None,
            column: None,
            query_position: None,
        }
    }

    pub fn with_context(mut self, context: String) -> Self {
        self.context = Some(context);
        self
    }

    pub fn with_location(mut self, line: usize, column: usize) -> Self {
        self.line = Some(line);
        self.column = Some(column);
        self
    }

    pub fn with_position(mut self, position: usize) -> Self {
        self.query_position = Some(position);
        self
    }

    pub fn location_string(&self) -> String {
        match (self.line, self.column) {
            (Some(line), Some(col)) => format!("第{}行第{}列", line, col),
            (Some(line), None) => format!("第{}行", line),
            _ => String::from("未知位置"),
        }
    }

    pub fn to_db_error(&self) -> DBError {
        self.clone().into()
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.error_type, self.message)
    }
}

impl std::error::Error for ValidationError {}

impl From<ValidationError> for DBError {
    fn from(err: ValidationError) -> Self {
        let error_msg = if let Some(ref ctx) = err.context {
            format!("{} (上下文: {})", err.message, ctx)
        } else {
            err.message.clone()
        };

        match err.error_type {
            ValidationErrorType::SyntaxError => {
                DBError::Query(QueryError::ParseError(error_msg))
            }
            ValidationErrorType::SemanticError | ValidationErrorType::TypeError => {
                DBError::Query(QueryError::InvalidQuery(error_msg))
            }
            _ => DBError::Query(QueryError::ExecutionError(error_msg)),
        }
    }
}

/// 验证结果类型别名
pub type ValidationResult = Result<(), ValidationError>;

/// Schema验证模式
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValidationMode {
    Strict,
    Lenient,
    RequiredOnly,
}

/// Schema验证错误类型（统一版本）
///
/// 使用DataType替代String，提高类型安全性
#[derive(Debug, Clone, PartialEq)]
pub enum SchemaValidationError {
    FieldNotFound(String),
    TypeMismatch(String, DataType, DataType),
    MissingRequiredField(String),
    ExtraField(String),
}

impl fmt::Display for SchemaValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SchemaValidationError::FieldNotFound(field) => {
                write!(f, "字段 '{}' 在Schema中不存在", field)
            }
            SchemaValidationError::TypeMismatch(field, expected, actual) => {
                write!(
                    f,
                    "字段 '{}' 类型不匹配: 期望 '{:?}', 实际 '{:?}'",
                    field, expected, actual
                )
            }
            SchemaValidationError::MissingRequiredField(field) => {
                write!(f, "缺少必需字段 '{}'", field)
            }
            SchemaValidationError::ExtraField(field) => {
                write!(f, "变量中包含Schema中未定义的字段 '{}'", field)
            }
        }
    }
}

impl std::error::Error for SchemaValidationError {}

impl From<SchemaValidationError> for DBError {
    fn from(err: SchemaValidationError) -> Self {
        DBError::Validation(err.to_string())
    }
}

/// Schema验证结果
#[derive(Debug, Clone)]
pub struct SchemaValidationResult {
    pub is_valid: bool,
    pub errors: Vec<SchemaValidationError>,
}

impl SchemaValidationResult {
    pub fn success() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
        }
    }

    pub fn failure(errors: Vec<SchemaValidationError>) -> Self {
        Self {
            is_valid: false,
            errors,
        }
    }

    pub fn add_error(&mut self, error: SchemaValidationError) {
        self.is_valid = false;
        self.errors.push(error);
    }
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
