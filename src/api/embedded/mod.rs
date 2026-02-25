//! 嵌入式 API 模块
//!
//! 提供单机使用的嵌入式 GraphDB 接口，类似 SQLite 的使用方式

pub mod embedded_api;

pub use embedded_api::{GraphDb, EmbeddedConfig};
