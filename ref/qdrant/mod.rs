//! Qdrant vector database client
//!
//! This module provides a client for interacting with Qdrant vector database
//! via HTTP REST API.
//!
//! # Configuration Presets
//!
//! The client supports configuration presets optimized for different data sizes:
//!
//! | Preset | Vector Count | HNSW m | HNSW ef_construct |
//! |--------|--------------|--------|-------------------|
//! | Tiny   | <= 2,000     | -      | - (full scan)     |
//! | Small  | 2,000-10,000 | 16     | 128               |
//! | Medium | 10,000-100,000 | 32   | 256               |
//! | Large  | > 100,000    | 64     | 512               |

// Core public API modules
pub mod client;
pub mod config;
pub mod error;
pub mod types;

// Internal implementation modules (crate-only access)
pub(crate) mod estimator;
pub(crate) mod operations;
pub(crate) mod retrieval;
pub(crate) mod scheduler;
pub(crate) mod upgrade;

// Re-export main types
pub use client::QdrantClient;
pub use config::{
    CollectionPreset, DistanceMetric, HnswConfig, QdrantConfig, VectorStorageConfig, WalConfig,
};
pub use error::QdrantError;
pub use estimator::{
    CollectionSizeEstimate, CollectionSizeEstimator, DEFAULT_AVG_VECTORS_PER_FILE,
    DEFAULT_BYTES_PER_VECTOR, PresetGuideline, PresetGuidelines, SizeDifference,
    SizeEstimateBuilder,
};
pub use operations::SummarySearchResult;
pub use scheduler::{
    ConfigUpgradeScheduler, DEFAULT_CHECK_INTERVAL_SECS, DEFAULT_MAX_CONCURRENT_UPGRADES,
    SchedulerConfig, SchedulerStatus, UpgradeEvent, UpgradeWindow,
};

// Re-export broadcast channel types for event subscription
pub use retrieval::QdrantRetrieval;
pub use tokio::sync::broadcast::Receiver as UpgradeEventReceiver;
pub use types::{
    CollectionInfo, CollectionStatus, CompactPayload, HnswConfigInfo, IndexingMetadata, Payload,
    SearchQuery, SearchResult, SizeEstimation, SparseVector, VectorPoint,
};
pub use upgrade::{
    ConfigUpgradeService, StepStatus, UpgradeProgress, UpgradeStatus, UpgradeStep,
    UpgradeThresholds,
};
