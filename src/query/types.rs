use crate::core::{Edge, NullType, Tag, Value, Vertex};
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum QueryError {
    #[error("Storage error: {0}")]
    StorageError(#[from] crate::storage::StorageError),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Invalid query: {0}")]
    InvalidQuery(String),
    #[error("Execution error: {0}")]
    ExecutionError(String),
    #[error("Expression error: {0}")]
    ExpressionError(String),
}

#[derive(Debug, Clone)]
pub enum Query {
    CreateNode {
        id: Option<Value>,
        tags: Vec<Tag>,
    },
    CreateEdge {
        src: Value,
        dst: Value,
        edge_type: String,
        name: String,
        ranking: i64,
        properties: std::collections::HashMap<String, Value>,
    },
    MatchNodes {
        tags: Option<Vec<String>>, // Filter by tag names
        conditions: Vec<Condition>,
    },
    DeleteNode {
        id: Value,
    },
    UpdateNode {
        id: Value,
        tags: Vec<Tag>,
    },
}

#[derive(Debug, Clone)]
pub enum Condition {
    PropertyEquals(String, Value),
    PropertyGreaterThan(String, Value),
    PropertyLessThan(String, Value),
    PropertyIn(String, Vec<Value>),
}

#[derive(Debug)]
pub enum QueryResult {
    NodeId(Value),
    EdgeId(Value),
    Nodes(Vec<Vertex>),
    Edges(Vec<Edge>),
    Count(usize),
    Success,
}
