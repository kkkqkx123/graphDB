//! API 核心层 - 与传输层无关的业务逻辑
//!
//! 提供查询执行、事务管理、Schema 操作等核心功能，
//! 被嵌入式层和网络服务层复用。

pub mod error;
pub mod types;
pub mod query_api;
pub mod transaction_api;
pub mod schema_api;

pub use error::{CoreError, CoreResult};
pub use types::*;
pub use query_api::QueryApi;
pub use transaction_api::TransactionApi;
pub use schema_api::SchemaApi;

// 从 core 层重新导出统计类型
pub use crate::core::{StatsManager, QueryMetrics, QueryProfile, MetricType, MetricValue, QueryPhase, ErrorType, ErrorInfo, ErrorSummary, QueryStatus};
