//! Client module for GraphDB CLI
//!
//! Provides unified interface for connecting to GraphDB via HTTP or embedded mode.

pub mod client_trait;
pub mod embedded;
pub mod http;

pub use client_trait::{ClientConfig, ClientFactory, ConnectionMode, GraphDbClient, SessionInfo};

// Re-export HTTP client types
pub use http::{EdgeTypeInfo, FieldInfo, QueryResult, SpaceInfo, TagInfo};

// Re-export embedded client
pub use embedded::EmbeddedClient;
