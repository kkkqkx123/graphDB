#![cfg(feature = "service")]

pub mod config;
pub mod grpc;
pub mod metrics;
pub mod proto;

pub use config::{ServerConfig, ServiceConfig};
pub use grpc::{
    run_server, run_server_with_service, run_server_with_storage, InversearchService,
};
