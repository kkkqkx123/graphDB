//! Client module for GraphDB CLI
//!
//! Provides HTTP client for connecting to GraphDB server.

mod batch;
mod client;
mod config;
mod config_types;
mod request_types;
mod response_types;
mod schema;
mod stats;
mod transaction;
mod types;
mod validation;
mod vector;

pub use batch::{BatchError, BatchItem, BatchResult, BatchStatus, BatchType, EdgeData, VertexData};
pub use client::HttpClient;
pub use config::{ClientConfig, SessionInfo};
pub use config_types::{ConfigItem, ConfigSection, ServerConfig};
pub use schema::{DataType, PropertyDef};
pub use stats::{
    DatabaseStatistics, QueryStatistics, QueryTypeStatistics, SessionStatistics, SlowQueryInfo,
};
pub use transaction::{TransactionInfo, TransactionOptions};
pub use types::{EdgeTypeInfo, FieldInfo, QueryResult, SpaceInfo, TagInfo};
pub use validation::{ValidationError, ValidationResult, ValidationWarning};
pub use vector::{VectorMatch, VectorSearchResult};
