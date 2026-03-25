//! API Core Layer Error Types
//!
//! Business logic errors not related to the transport layer

use thiserror::Error;

/// Extended Error Code Types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtendedErrorCode {
    // No extension error
    None = 0,

    // Parsing Related (1000-1099)
    SyntaxError = 1000,
    SemanticError = 1001,
    UnexpectedToken = 1002,
    UnterminatedLiteral = 1003,

    // Type-related (1100-1199)
    TypeMismatch = 1100,
    DivisionByZero = 1101,
    OutOfRange = 1102,

    // Binding related (1200-1299)
    DuplicateKey = 1200,
    ForeignKeyConstraint = 1201,
    NotNullConstraint = 1202,
    UniqueConstraint = 1203,
    CheckConstraint = 1204,

    // Concurrency-related (1300-1399)
    ConnectionLost = 1300,
    Deadlock = 1301,
    LockTimeout = 1302,

    // Figure correlation (1400-1499)
    InvalidVertex = 1400,
    InvalidEdge = 1401,
    PathNotFound = 1402,
}

impl ExtendedErrorCode {
    /// Convert to integer error code
    pub fn as_i32(&self) -> i32 {
        *self as i32
    }
}

/// Core layer error types
#[derive(Error, Debug, Clone)]
pub enum CoreError {
    #[error("Query execution failed: {0}")]
    QueryExecutionFailed(String),

    #[error("Transaction operation failed: {0}")]
    TransactionFailed(String),

    #[error("Schema operation failed: {0}")]
    SchemaOperationFailed(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Internal error: {0}")]
    Internal(String),

    /// Query error with details
    #[error("Query error: {message}")]
    DetailedQueryError {
        message: String,
        extended_code: ExtendedErrorCode,
        offset: Option<usize>,
    },
}

impl CoreError {
    /// Get Extended Error Code
    pub fn extended_code(&self) -> ExtendedErrorCode {
        match self {
            CoreError::DetailedQueryError { extended_code, .. } => *extended_code,
            _ => ExtendedErrorCode::None,
        }
    }

    /// Get error position offset
    pub fn error_offset(&self) -> Option<usize> {
        match self {
            CoreError::DetailedQueryError { offset, .. } => *offset,
            _ => None,
        }
    }

    /// Creating Query Errors with Details
    pub fn detailed_query_error(
        message: impl Into<String>,
        extended_code: ExtendedErrorCode,
        offset: Option<usize>,
    ) -> Self {
        CoreError::DetailedQueryError {
            message: message.into(),
            extended_code,
            offset,
        }
    }
}

/// Core layer result types
pub type CoreResult<T> = Result<T, CoreError>;

// Conversion from underlying error
impl From<crate::core::error::QueryError> for CoreError {
    fn from(err: crate::core::error::QueryError) -> Self {
        CoreError::QueryExecutionFailed(err.to_string())
    }
}

impl From<crate::storage::StorageError> for CoreError {
    fn from(err: crate::storage::StorageError) -> Self {
        CoreError::StorageError(err.to_string())
    }
}

impl From<crate::core::error::DBError> for CoreError {
    fn from(err: crate::core::error::DBError) -> Self {
        match err {
            crate::core::error::DBError::Query(e) => CoreError::QueryExecutionFailed(e.to_string()),
            crate::core::error::DBError::Storage(e) => CoreError::StorageError(e.to_string()),
            crate::core::error::DBError::Transaction(s) => CoreError::TransactionFailed(s),
            _ => CoreError::Internal(err.to_string()),
        }
    }
}
