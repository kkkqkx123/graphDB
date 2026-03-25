//! API Core Layer – Business logic that is independent of the transport layer
//!
//! It provides core functions such as query execution, transaction management, and Schema operations.
//! It is reused by the embedded layer and the network service layer.

pub mod error;
pub mod query_api;
pub mod schema_api;
pub mod transaction_api;
pub mod types;

pub use error::{CoreError, CoreResult, ExtendedErrorCode};
pub use query_api::QueryApi;
pub use schema_api::SchemaApi;
pub use types::*;

// Re-export the statistical types from the core layer.
pub use crate::core::{
    ErrorInfo, ErrorSummary, ErrorType, MetricType, MetricValue, QueryMetrics, QueryPhase,
    QueryProfile, QueryStatus, StatsManager,
};
