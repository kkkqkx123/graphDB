//! API 核心层 - 与传输层无关的业务逻辑
//!
//! 提供查询执行、事务管理、Schema 操作等核心功能，
//! 被嵌入式层和网络服务层复用。

pub mod error;
pub mod query_api;
pub mod schema_api;
pub mod transaction_api;
pub mod types;

pub use error::{CoreError, CoreResult};
pub use query_api::QueryApi;
pub use schema_api::SchemaApi;
pub use transaction_api::TransactionApi;
pub use types::*;

// 从 core 层重新导出统计类型
pub use crate::core::{
    ErrorInfo, ErrorSummary, ErrorType, MetricType, MetricValue, QueryMetrics, QueryPhase,
    QueryProfile, QueryStatus, StatsManager,
};
