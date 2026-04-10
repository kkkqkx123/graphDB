//! Vector Search Module - Simplified
//!
//! Provides coordination between graph data and vector search.
//! Most vector functionality has been moved to vector_client crate and sync module.

// Re-export from vector_client for backward compatibility
pub use vector_client::{
    EmbeddingConfig, EmbeddingError, EmbeddingProvider, EmbeddingService,
    VectorEngine, VectorManager,
    SearchQuery, SearchResult, VectorFilter, VectorPoint,
    DistanceMetric, CollectionConfig,
};

// Re-export from sync for backward compatibility
pub use crate::sync::vector_sync::{
    VectorSyncCoordinator, VectorChangeContext, VectorChangeType,
    VectorIndexLocation, SearchOptions, VectorPointData,
};

// Deprecated: Keep old exports for backward compatibility only
// These will be removed in future versions

