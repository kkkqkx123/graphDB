pub mod api;

#[cfg(all(feature = "qdrant-http", feature = "qdrant-grpc"))]
compile_error!(
    "Features 'qdrant-http' and 'qdrant-grpc' cannot be enabled simultaneously. "
    "Choose one: qdrant-http (HTTP REST) or qdrant-grpc (gRPC)."
);

pub mod config;
pub mod embedding;
pub mod engine;
pub mod error;
pub mod manager;
pub mod types;

pub use config::*;
pub use engine::VectorEngine;
pub use error::{Result, VectorClientError};
pub use types::*;

#[cfg(feature = "qdrant-http")]
pub use engine::QdrantEngine;

#[cfg(feature = "qdrant-grpc")]
pub use engine::QdrantGrpcEngine;

pub use api::VectorClient;
pub use api::{CollectionApi, PointApi, SearchApi};
pub use embedding::{EmbeddingConfig, EmbeddingError, EmbeddingProvider, EmbeddingService};
pub use manager::VectorManager;
