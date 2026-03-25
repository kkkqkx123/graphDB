//! Statistical Information Module
//!
//! Provides query metrics, query portraits and error statistics.
//!
//! ## Module structure
//!
//! - `metrics`: lightweight query metrics (for return to client)
//! - `profile`: detailed query profile (for monitoring and analysis)
//! - `error_stats`: error statistics
//! - `manager`: unified manager
//!
//! ## QueryMetrics vs QueryProfile
//!
//! ### QueryMetrics (lightweight)
//! - Purpose: Query metrics returned to the client
//! - Accuracy: microseconds (us)
//! - Content: execution time, number of nodes, number of results
//! - Usage scenarios: API response, client-side display
//!
//! ### QueryProfile (detailed)
//! - Purpose: Internal analysis and monitoring
//! - Accuracy: milliseconds (ms)
//! - Contents: execution time, actuator statistics, error messages, slow query logs
//! - Usage scenarios: performance analysis, problem diagnosis, monitoring alarms

pub mod error_stats;
pub mod manager;
pub mod metrics;
pub mod profile;

// Re-export common types
pub use error_stats::{ErrorInfo, ErrorStatsManager, ErrorSummary, ErrorType, QueryPhase};
pub use manager::{MetricType, MetricValue, StatsManager};
pub use metrics::QueryMetrics;
pub use profile::{ExecutorStat, QueryProfile, QueryStatus, StageMetrics};
