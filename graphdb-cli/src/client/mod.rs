//! Client module for GraphDB CLI
//!
//! Provides HTTP client for connecting to GraphDB server.

pub mod client_trait;
pub mod http;

pub use client_trait::{ClientConfig, ClientFactory, GraphDbClient, SessionInfo};

// Re-export HTTP client types
pub use http::{EdgeTypeInfo, FieldInfo, QueryResult, SpaceInfo, TagInfo};
