// Service module - only compiled with "service" feature

pub mod config;
pub mod grpc;
pub mod metrics;
pub mod proto;

// Re-export service API
pub use config::{Config, IndexConfig, RedisConfig, ServerConfig};
pub use grpc::{run_server, BM25Service};
pub use metrics::{init_logging, init_metrics};
