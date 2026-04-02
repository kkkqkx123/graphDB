// Service module - only compiled with "service" feature

pub mod config;
pub mod metrics;
pub mod proto;
pub mod grpc;

// Re-export service API
pub use config::{Config, ServerConfig, RedisConfig, IndexConfig};
pub use grpc::{BM25Service, run_server};
pub use metrics::{init_logging, init_metrics};
