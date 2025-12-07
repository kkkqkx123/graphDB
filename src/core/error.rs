//! Error handling system for GraphDB
//!
//! This module provides error types similar to NebulaGraph's Status/StatusOr system

use std::fmt;

/// Represents various error codes similar to NebulaGraph's Status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    /// Operation was successful
    Ok,
    /// Value was inserted
    Inserted,
    /// General error with message
    Error(String),
    /// File not found
    NoSuchFile(String),
    /// Feature not supported
    NotSupported(String),
    /// Syntax error in query
    SyntaxError(String),
    /// Semantic error in query
    SemanticError(String),
    /// Graph memory exceeded
    GraphMemoryExceeded,
    /// No statement to execute
    StatementEmpty,
    /// Key not found in storage
    KeyNotFound,
    /// Partial success in operation
    PartialSuccess,
    /// Storage memory exceeded
    StorageMemoryExceeded,
    /// Space not found
    SpaceNotFound,
    /// Host not found
    HostNotFound,
    /// Tag not found
    TagNotFound,
    /// Edge not found
    EdgeNotFound,
    /// User not found
    UserNotFound,
    /// Index not found
    IndexNotFound,
    /// Group not found
    GroupNotFound,
    /// Zone not found
    ZoneNotFound,
    /// Leader changed
    LeaderChanged,
    /// Balanced
    Balanced,
    /// Part not found
    PartNotFound,
    /// Listener not found
    ListenerNotFound,
    /// Session not found
    SessionNotFound,
    /// Permission error
    PermissionError,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::Ok => write!(f, "OK"),
            Status::Inserted => write!(f, "Inserted"),
            Status::Error(msg) => write!(f, "Error: {}", msg),
            Status::NoSuchFile(path) => write!(f, "No such file: {}", path),
            Status::NotSupported(feature) => write!(f, "Not supported: {}", feature),
            Status::SyntaxError(msg) => write!(f, "Syntax error: {}", msg),
            Status::SemanticError(msg) => write!(f, "Semantic error: {}", msg),
            Status::GraphMemoryExceeded => write!(f, "Graph memory exceeded"),
            Status::StatementEmpty => write!(f, "Statement is empty"),
            Status::KeyNotFound => write!(f, "Key not found"),
            Status::PartialSuccess => write!(f, "Partial success"),
            Status::StorageMemoryExceeded => write!(f, "Storage memory exceeded"),
            Status::SpaceNotFound => write!(f, "Space not found"),
            Status::HostNotFound => write!(f, "Host not found"),
            Status::TagNotFound => write!(f, "Tag not found"),
            Status::EdgeNotFound => write!(f, "Edge not found"),
            Status::UserNotFound => write!(f, "User not found"),
            Status::IndexNotFound => write!(f, "Index not found"),
            Status::GroupNotFound => write!(f, "Group not found"),
            Status::ZoneNotFound => write!(f, "Zone not found"),
            Status::LeaderChanged => write!(f, "Leader changed"),
            Status::Balanced => write!(f, "Balanced"),
            Status::PartNotFound => write!(f, "Part not found"),
            Status::ListenerNotFound => write!(f, "Listener not found"),
            Status::SessionNotFound => write!(f, "Session not found"),
            Status::PermissionError => write!(f, "Permission error"),
        }
    }
}

impl std::error::Error for Status {}

impl Status {
    /// Create an OK status
    pub fn ok() -> Self {
        Status::Ok
    }

    /// Check if the status is OK
    pub fn is_ok(&self) -> bool {
        matches!(self, Status::Ok)
    }

    /// Create an error status
    pub fn error(msg: impl Into<String>) -> Self {
        Status::Error(msg.into())
    }

    /// Create a syntax error status
    pub fn syntax_error(msg: impl Into<String>) -> Self {
        Status::SyntaxError(msg.into())
    }

    /// Create a semantic error status
    pub fn semantic_error(msg: impl Into<String>) -> Self {
        Status::SemanticError(msg.into())
    }

    /// Create a not supported status
    pub fn not_supported(feature: impl Into<String>) -> Self {
        Status::NotSupported(feature.into())
    }

    /// Create a key not found status
    pub fn key_not_found() -> Self {
        Status::KeyNotFound
    }

    /// Get the error message if this is an error status
    pub fn message(&self) -> String {
        format!("{}", self)
    }
}

/// Type alias for a result that contains either a value or a Status
pub type StatusOr<T> = Result<T, Status>;

/// Legacy type alias for compatibility
pub type GraphDBResult<T> = StatusOr<T>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_creation() {
        let ok_status = Status::ok();
        assert!(ok_status.is_ok());

        let error_status = Status::error("Something went wrong");
        assert_eq!(format!("{}", error_status), "Error: Something went wrong");

        let syntax_error = Status::syntax_error("Invalid syntax");
        assert_eq!(format!("{}", syntax_error), "Syntax error: Invalid syntax");
    }

    #[test]
    fn test_statusor_usage() {
        let success: StatusOr<i32> = Ok(42);
        let error: StatusOr<i32> = Err(Status::KeyNotFound);

        assert!(success.is_ok());
        assert!(error.is_err());

        if let Ok(value) = success {
            assert_eq!(value, 42);
        }

        if let Err(status) = error {
            assert_eq!(status, Status::KeyNotFound);
        }
    }
}