//! Error handling system for GraphDB
//! 
//! This module provides error types and Result wrappers similar to NebulaGraph's Status/StatusOr system

use std::fmt;

/// Represents various error codes similar to NebulaGraph's ErrorCode
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphDBError {
    // Successful codes
    Ok,
    Inserted,

    // General errors
    Error(String),
    NoSuchFile,
    NotSupported,

    // Graph engine errors
    SyntaxError(String),
    SemanticError(String),
    GraphMemoryExceeded,

    // Statement errors
    StatementEmpty,

    // Storage engine errors
    KeyNotFound,
    PartialSuccess,
    StorageMemoryExceeded,

    // Meta/service errors
    SpaceNotFound,
    HostNotFound,
    TagNotFound,
    EdgeNotFound,
    UserNotFound,
    IndexNotFound,
    GroupNotFound,
    ZoneNotFound,
    LeaderChanged,
    Balanced,
    PartNotFound,
    ListenerNotFound,
    SessionNotFound,

    // Permission errors
    PermissionError,
}

impl fmt::Display for GraphDBError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GraphDBError::Ok => write!(f, "OK"),
            GraphDBError::Inserted => write!(f, "Inserted"),
            GraphDBError::Error(msg) => write!(f, "Error: {}", msg),
            GraphDBError::NoSuchFile => write!(f, "No such file"),
            GraphDBError::NotSupported => write!(f, "Not supported"),
            GraphDBError::SyntaxError(msg) => write!(f, "Syntax error: {}", msg),
            GraphDBError::SemanticError(msg) => write!(f, "Semantic error: {}", msg),
            GraphDBError::GraphMemoryExceeded => write!(f, "Graph memory exceeded"),
            GraphDBError::StatementEmpty => write!(f, "Statement is empty"),
            GraphDBError::KeyNotFound => write!(f, "Key not found"),
            GraphDBError::PartialSuccess => write!(f, "Partial success"),
            GraphDBError::StorageMemoryExceeded => write!(f, "Storage memory exceeded"),
            GraphDBError::SpaceNotFound => write!(f, "Space not found"),
            GraphDBError::HostNotFound => write!(f, "Host not found"),
            GraphDBError::TagNotFound => write!(f, "Tag not found"),
            GraphDBError::EdgeNotFound => write!(f, "Edge not found"),
            GraphDBError::UserNotFound => write!(f, "User not found"),
            GraphDBError::IndexNotFound => write!(f, "Index not found"),
            GraphDBError::GroupNotFound => write!(f, "Group not found"),
            GraphDBError::ZoneNotFound => write!(f, "Zone not found"),
            GraphDBError::LeaderChanged => write!(f, "Leader changed"),
            GraphDBError::Balanced => write!(f, "Balanced"),
            GraphDBError::PartNotFound => write!(f, "Part not found"),
            GraphDBError::ListenerNotFound => write!(f, "Listener not found"),
            GraphDBError::SessionNotFound => write!(f, "Session not found"),
            GraphDBError::PermissionError => write!(f, "Permission error"),
        }
    }
}

impl std::error::Error for GraphDBError {}

/// Result type that can either contain a value or a GraphDBError
pub type GraphDBResult<T> = Result<T, GraphDBError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        assert_eq!(format!("{}", GraphDBError::Ok), "OK");
        assert_eq!(format!("{}", GraphDBError::Error("test".to_string())), "Error: test");
    }

    #[test]
    fn test_result_usage() {
        let success: GraphDBResult<i32> = Ok(42);
        let error: GraphDBResult<i32> = Err(GraphDBError::KeyNotFound);

        assert!(success.is_ok());
        assert!(error.is_err());
    }
}