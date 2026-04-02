// Core modules (always compiled)
pub mod config;
pub mod error;
pub mod index;

// Service module (conditional compilation)
#[cfg(feature = "service")]
pub mod service;

// Re-export core API (available in both library and service mode)
pub use config::{Bm25Config, FieldWeights, SearchConfig};
pub use error::{Bm25Error, Result};
pub use index::{IndexManager, IndexSchema};

// Re-export service API (only available in service mode)
#[cfg(feature = "service")]
pub use service::{Config, ServerConfig, RedisConfig, IndexConfig};

#[cfg(feature = "service")]
pub use service::{init_logging, init_metrics};

#[cfg(feature = "service")]
pub use service::{BM25Service, run_server};
