//! Client module for GraphDB CLI
//!
//! Provides HTTP client for connecting to GraphDB server.

pub mod client_trait;
pub mod http;

pub use client_trait::{
    BatchError, BatchItem, BatchResult, BatchStatus, BatchType, ClientConfig, ClientFactory,
    DataType, DatabaseStatistics, EdgeData, GraphDbClient, PropertyDef, QueryStatistics,
    QueryTypeStatistics, SessionInfo, SessionStatistics, SlowQueryInfo, TransactionInfo,
    TransactionOptions, VertexData,
};

// Re-export HTTP client types
pub use http::{EdgeTypeInfo, FieldInfo, QueryResult, SpaceInfo, TagInfo};
