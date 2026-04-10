pub mod error;
pub mod types;
pub mod config;
pub mod engine;
pub mod api;
pub mod embedding;
pub mod manager;

pub use error::{Result, VectorClientError};
pub use types::*;
pub use config::*;
pub use engine::VectorEngine;

#[cfg(feature = "qdrant")]
pub use engine::QdrantEngine;

#[cfg(feature = "mock")]
pub use engine::MockEngine;

pub use api::VectorClient;
pub use api::{CollectionApi, PointApi, SearchApi};
pub use embedding::{EmbeddingConfig, EmbeddingError, EmbeddingService, EmbeddingProvider};
pub use manager::VectorManager;
