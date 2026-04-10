//! Vector Search Module - Simplified
//!
//! Provides coordination between graph data and vector search.
//! Most vector functionality has been moved to vector_client crate and sync module.

// Re-export from vector_client for backward compatibility
pub use vector_client::{
    CollectionConfig, DistanceMetric, EmbeddingConfig, EmbeddingError, EmbeddingProvider,
    EmbeddingService, SearchQuery, SearchResult, VectorClientConfig as VectorConfig, VectorEngine,
    VectorFilter, VectorManager as VectorIndexManager, VectorPoint,
};

// Re-export from sync for backward compatibility
pub use crate::sync::vector_sync::{
    SearchOptions, VectorChangeContext, VectorChangeType, VectorIndexLocation, VectorPointData,
    VectorSyncCoordinator,
};

// Backward compatibility aliases
/// @deprecated Use VectorSyncCoordinator instead
pub type VectorCoordinator = VectorSyncCoordinator;
