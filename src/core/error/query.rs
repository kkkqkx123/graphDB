//! Query layer error type
//!
//! This includes errors that occur during the processes of query parsing, validation, and execution.

use thiserror::Error;

use crate::core::error::codes::{ErrorCode, PublicError, ToPublicError};
use crate::core::error::expression::{ExpressionError, ExpressionErrorType};
use crate::core::error::manager::ManagerError;
use crate::core::error::permission::PermissionError;
use crate::core::error::session::SessionError;
use crate::core::error::storage::StorageError;
use crate::core::error::DBError;

/// Query processing phase enumeration
///
/// Used to identify which phase of query processing an error occurred in
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryPhase {
    Parse,
    Validate,
    Plan,
    Optimize,
    Execute,
}

impl std::fmt::Display for QueryPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryPhase::Parse => write!(f, "parse"),
            QueryPhase::Validate => write!(f, "validate"),
            QueryPhase::Plan => write!(f, "plan"),
            QueryPhase::Optimize => write!(f, "optimize"),
            QueryPhase::Execute => write!(f, "execute"),
        }
    }
}

/// Error type during the planned node access
///
/// Errors that occur during the query plan traversal and validation processes
#[derive(Debug, Clone)]
pub enum PlanNodeVisitError {
    VisitError {
        node_id: Option<String>,
        message: String,
    },
    TraversalError {
        path: String,
        message: String,
    },
    ValidationError {
        node_type: String,
        message: String,
    },
}

impl std::fmt::Display for PlanNodeVisitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlanNodeVisitError::VisitError { node_id, message } => {
                if let Some(id) = node_id {
                    write!(f, "Visit error at node {}: {}", id, message)
                } else {
                    write!(f, "Visit error: {}", message)
                }
            }
            PlanNodeVisitError::TraversalError { path, message } => {
                write!(f, "Traversal error in {}: {}", path, message)
            }
            PlanNodeVisitError::ValidationError { node_type, message } => {
                write!(f, "Validation failed for {}: {}", node_type, message)
            }
        }
    }
}

impl std::error::Error for PlanNodeVisitError {}

impl PlanNodeVisitError {
    pub fn visit_error(message: impl Into<String>) -> Self {
        PlanNodeVisitError::VisitError {
            node_id: None,
            message: message.into(),
        }
    }

    pub fn visit_error_with_node(node_id: impl Into<String>, message: impl Into<String>) -> Self {
        PlanNodeVisitError::VisitError {
            node_id: Some(node_id.into()),
            message: message.into(),
        }
    }

    pub fn traversal_error(path: impl Into<String>, message: impl Into<String>) -> Self {
        PlanNodeVisitError::TraversalError {
            path: path.into(),
            message: message.into(),
        }
    }

    pub fn validation_error(
        node_type: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        PlanNodeVisitError::ValidationError {
            node_type: node_type.into(),
            message: message.into(),
        }
    }
}

/// Query operation result type aliases
pub type QueryResult<T> = Result<T, QueryError>;

/// Structured parse error information
///
/// Preserves detailed error context from the parser for better error reporting.
/// This type is Clone-friendly, unlike the original ParseError which contains
/// a `Box<dyn Error>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredParseError {
    /// Error category
    pub kind: ParseErrorKind,
    /// Human-readable error message
    pub message: String,
    /// Line and column position in the source
    pub position: crate::core::types::Position,
    /// Byte offset in the source (if available)
    pub offset: Option<usize>,
    /// The unexpected token that caused the error
    pub unexpected_token: Option<String>,
    /// List of expected tokens at the error location
    pub expected_tokens: Vec<String>,
    /// Helpful hints for fixing the error
    pub hints: Vec<String>,
    /// Context information (converted to string for Clone support)
    pub context: Option<String>,
}

/// Parse error kind enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseErrorKind {
    LexicalError,
    SyntaxError,
    UnexpectedToken,
    UnterminatedString,
    UnterminatedComment,
    InvalidNumber,
    InvalidEscapeSequence,
    UnicodeEscapeError,
    UnexpectedEndOfInput,
    InvalidCharacter,
    UnknownKeyword,
    RecursionLimitExceeded,
    UnsupportedFeature,
    SemanticError,
}

impl std::fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseErrorKind::LexicalError => write!(f, "Lexical error"),
            ParseErrorKind::SyntaxError => write!(f, "Syntax error"),
            ParseErrorKind::UnexpectedToken => write!(f, "Unexpected token"),
            ParseErrorKind::UnterminatedString => write!(f, "Unterminated string"),
            ParseErrorKind::UnterminatedComment => write!(f, "Unterminated comment"),
            ParseErrorKind::InvalidNumber => write!(f, "Invalid number"),
            ParseErrorKind::InvalidEscapeSequence => write!(f, "Invalid escape sequence"),
            ParseErrorKind::UnicodeEscapeError => write!(f, "Unicode escape error"),
            ParseErrorKind::UnexpectedEndOfInput => write!(f, "Unexpected end of input"),
            ParseErrorKind::InvalidCharacter => write!(f, "Invalid character"),
            ParseErrorKind::UnknownKeyword => write!(f, "Unknown keyword"),
            ParseErrorKind::RecursionLimitExceeded => write!(f, "Recursion limit exceeded"),
            ParseErrorKind::UnsupportedFeature => write!(f, "Unsupported feature"),
            ParseErrorKind::SemanticError => write!(f, "Semantic error"),
        }
    }
}

impl std::fmt::Display for StructuredParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} at line {}, column {}: {}",
            self.kind, self.position.line, self.position.column, self.message
        )?;

        if let Some(ref token) = self.unexpected_token {
            writeln!(f, "\n  Unexpected token: {}", token)?;
        }

        if !self.expected_tokens.is_empty() {
            writeln!(
                f,
                "\n  Expected one of: {}",
                self.expected_tokens.join(", ")
            )?;
        }

        if let Some(ref context) = self.context {
            writeln!(f, "\n  Context: {}", context)?;
        }

        if !self.hints.is_empty() {
            writeln!(f, "\n  Hint(s):")?;
            for hint in &self.hints {
                writeln!(f, "    - {}", hint)?;
            }
        }

        Ok(())
    }
}

/// Query layer error type
#[derive(Debug, Clone)]
pub enum QueryError {
    StorageError(StorageErrorWrapper),
    ParseError(StructuredParseError),
    PlanningError(String),
    OptimizationError(String),
    InvalidQuery(String),
    ExecutionError(String),
    ExpressionError(ExpressionErrorWrapper),
    PlanNodeVisitError(PlanNodeVisitError),
    SessionError(SessionError),
    PermissionError(PermissionError),
    TransactionError(String),
    TypeError(String),
    Timeout(String),
}

impl std::fmt::Display for QueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryError::StorageError(e) => write!(f, "Storage error: {}", e),
            QueryError::ParseError(e) => write!(f, "Parse error: {}", e),
            QueryError::PlanningError(msg) => write!(f, "Planning error: {}", msg),
            QueryError::OptimizationError(msg) => write!(f, "Optimization error: {}", msg),
            QueryError::InvalidQuery(msg) => write!(f, "Invalid query: {}", msg),
            QueryError::ExecutionError(msg) => write!(f, "Execution error: {}", msg),
            QueryError::ExpressionError(e) => write!(f, "Expression error: {}", e),
            QueryError::PlanNodeVisitError(e) => write!(f, "Plan node visit error: {}", e),
            QueryError::SessionError(e) => write!(f, "Session error: {}", e),
            QueryError::PermissionError(e) => write!(f, "Permission error: {}", e),
            QueryError::TransactionError(msg) => write!(f, "Transaction error: {}", msg),
            QueryError::TypeError(msg) => write!(f, "Type error: {}", msg),
            QueryError::Timeout(msg) => write!(f, "Timeout: {}", msg),
        }
    }
}

impl std::error::Error for QueryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            QueryError::StorageError(e) => Some(e),
            QueryError::ExpressionError(e) => Some(e),
            QueryError::PlanNodeVisitError(e) => Some(e),
            QueryError::SessionError(e) => Some(e),
            QueryError::PermissionError(e) => Some(e),
            _ => None,
        }
    }
}

/// Wrapper for StorageError to implement Clone
#[derive(Error, Debug, Clone)]
#[error("{0}")]
pub struct StorageErrorWrapper(String);

impl From<StorageError> for StorageErrorWrapper {
    fn from(e: StorageError) -> Self {
        StorageErrorWrapper(e.to_string())
    }
}

/// Wrapper for ExpressionError to implement Clone
#[derive(Error, Debug, Clone)]
#[error("{0}")]
pub struct ExpressionErrorWrapper(String);

impl From<ExpressionError> for ExpressionErrorWrapper {
    fn from(e: ExpressionError) -> Self {
        ExpressionErrorWrapper(e.to_string())
    }
}

impl From<ExpressionErrorType> for ExpressionErrorWrapper {
    fn from(e: ExpressionErrorType) -> Self {
        ExpressionErrorWrapper(e.to_string())
    }
}

impl QueryError {
    pub fn storage_error(message: impl Into<String>) -> Self {
        QueryError::StorageError(StorageErrorWrapper(message.into()))
    }

    pub fn parse_error(message: impl Into<String>) -> Self {
        QueryError::ParseError(StructuredParseError {
            kind: ParseErrorKind::SyntaxError,
            message: message.into(),
            position: crate::core::types::Position::new(0, 0),
            offset: None,
            unexpected_token: None,
            expected_tokens: Vec::new(),
            hints: Vec::new(),
            context: None,
        })
    }

    pub fn parse_error_with_offset(message: impl Into<String>, offset: usize) -> Self {
        QueryError::ParseError(StructuredParseError {
            kind: ParseErrorKind::SyntaxError,
            message: message.into(),
            position: crate::core::types::Position::new(0, 0),
            offset: Some(offset),
            unexpected_token: None,
            expected_tokens: Vec::new(),
            hints: Vec::new(),
            context: None,
        })
    }

    pub fn parse_error_with_location(
        message: impl Into<String>,
        offset: usize,
        location: impl Into<String>,
    ) -> Self {
        QueryError::ParseError(StructuredParseError {
            kind: ParseErrorKind::SyntaxError,
            message: message.into(),
            position: crate::core::types::Position::new(0, 0),
            offset: Some(offset),
            unexpected_token: None,
            expected_tokens: Vec::new(),
            hints: vec![location.into()],
            context: None,
        })
    }

    pub fn structured_parse_error(err: StructuredParseError) -> Self {
        QueryError::ParseError(err)
    }

    pub fn offset(&self) -> Option<usize> {
        match self {
            QueryError::ParseError(e) => e.offset,
            _ => None,
        }
    }

    pub fn location(&self) -> Option<&str> {
        match self {
            QueryError::ParseError(e) => {
                if e.hints.is_empty() {
                    None
                } else {
                    Some(&e.hints[0])
                }
            }
            _ => None,
        }
    }

    pub fn parse_error_position(&self) -> Option<&crate::core::types::Position> {
        match self {
            QueryError::ParseError(e) => Some(&e.position),
            _ => None,
        }
    }

    pub fn parse_error_kind(&self) -> Option<ParseErrorKind> {
        match self {
            QueryError::ParseError(e) => Some(e.kind),
            _ => None,
        }
    }

    pub fn pipeline_parse_error<E: std::error::Error>(e: E) -> Self {
        QueryError::parse_error(e.to_string())
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

    pub fn pipeline_error(phase: QueryPhase, message: String) -> Self {
        match phase {
            QueryPhase::Parse => QueryError::parse_error(message),
            QueryPhase::Validate => QueryError::InvalidQuery(message),
            QueryPhase::Plan => QueryError::PlanningError(message),
            QueryPhase::Optimize => QueryError::OptimizationError(message),
            QueryPhase::Execute => QueryError::ExecutionError(message),
        }
    }
}

impl From<StorageError> for QueryError {
    fn from(e: StorageError) -> Self {
        QueryError::StorageError(e.into())
    }
}

impl From<DBError> for QueryError {
    fn from(e: DBError) -> Self {
        match e {
            DBError::Query(qe) => qe,
            DBError::Storage(se) => QueryError::StorageError(se.into()),
            DBError::Expression(expression) => {
                QueryError::ExpressionError(expression.into())
            }
            DBError::Plan(plan) => QueryError::ExecutionError(plan.to_string()),
            DBError::Manager(manager) => QueryError::ExecutionError(manager.to_string()),
            DBError::Validation(msg) => QueryError::InvalidQuery(msg),
            DBError::Io(io) => QueryError::ExecutionError(io.to_string()),
            DBError::TypeDeduction(msg) => QueryError::TypeError(msg),
            DBError::Serialization(msg) => QueryError::ExecutionError(msg),
            DBError::Index(msg) => QueryError::ExecutionError(msg),
            DBError::Transaction(msg) => QueryError::TransactionError(msg),
            DBError::Internal(msg) => QueryError::ExecutionError(msg),
            DBError::Session(session) => QueryError::SessionError(session),
            DBError::Auth(auth) => QueryError::ExecutionError(auth.to_string()),
            DBError::Permission(permission) => QueryError::PermissionError(permission),
            DBError::MemoryLimitExceeded(msg) => QueryError::ExecutionError(msg),
            DBError::Fulltext(fe) => QueryError::ExecutionError(fe.to_string()),
            DBError::Coordinator(ce) => QueryError::ExecutionError(ce.to_string()),
            DBError::Vector(ve) => QueryError::ExecutionError(ve.to_string()),
            DBError::VectorCoordinator(vce) => QueryError::ExecutionError(vce.to_string()),
            DBError::Search(se) => QueryError::ExecutionError(se),
            DBError::GraphService(gs) => QueryError::ExecutionError(gs),
        }
    }
}

impl From<std::io::Error> for QueryError {
    fn from(e: std::io::Error) -> Self {
        QueryError::ExecutionError(e.to_string())
    }
}

impl From<PlanNodeVisitError> for QueryError {
    fn from(e: PlanNodeVisitError) -> Self {
        QueryError::PlanNodeVisitError(e)
    }
}

impl From<ManagerError> for QueryError {
    fn from(e: ManagerError) -> Self {
        QueryError::ExecutionError(e.to_string())
    }
}

impl From<SessionError> for QueryError {
    fn from(e: SessionError) -> Self {
        QueryError::SessionError(e)
    }
}

impl From<PermissionError> for QueryError {
    fn from(e: PermissionError) -> Self {
        QueryError::PermissionError(e)
    }
}

impl From<ExpressionError> for QueryError {
    fn from(e: ExpressionError) -> Self {
        QueryError::ExpressionError(e.into())
    }
}

impl From<ExpressionErrorType> for QueryError {
    fn from(e: ExpressionErrorType) -> Self {
        QueryError::ExpressionError(e.into())
    }
}

impl ToPublicError for QueryError {
    fn to_public_error(&self) -> PublicError {
        PublicError::new(self.to_error_code(), self.to_public_message())
    }

    fn to_error_code(&self) -> ErrorCode {
        match self {
            QueryError::ParseError { .. } => ErrorCode::ParseError,
            QueryError::InvalidQuery(_) => ErrorCode::ValidationError,
            QueryError::PlanningError(_) => ErrorCode::ExecutionError,
            QueryError::OptimizationError(_) => ErrorCode::ExecutionError,
            QueryError::ExecutionError(_) => ErrorCode::ExecutionError,
            QueryError::ExpressionError(_) => ErrorCode::ExecutionError,
            QueryError::StorageError(_) => ErrorCode::InternalError,
            QueryError::PlanNodeVisitError(_) => ErrorCode::ExecutionError,
            QueryError::SessionError(se) => se.to_error_code(),
            QueryError::PermissionError(pe) => pe.to_error_code(),
            QueryError::TransactionError(_) => ErrorCode::ExecutionError,
            QueryError::TypeError(_) => ErrorCode::TypeError,
            QueryError::Timeout(_) => ErrorCode::Timeout,
        }
    }

    fn to_public_message(&self) -> String {
        match self {
            QueryError::SessionError(se) => se.to_public_message(),
            QueryError::PermissionError(pe) => pe.to_public_message(),
            QueryError::StorageError(_) => "Storage operation failed".to_string(),
            _ => self.to_string(),
        }
    }
}
