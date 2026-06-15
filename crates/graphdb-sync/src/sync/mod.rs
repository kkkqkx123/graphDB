pub mod batch;
pub mod circuit_breaker;
#[cfg(feature = "fulltext-search")]
pub mod coordinator;
pub mod dead_letter_queue;
pub mod manager;
pub mod retry;
pub mod types;
pub mod vector_error;
#[cfg(feature = "qdrant")]
pub mod vector_sync;

pub use crate::search::SyncConfig;
pub use batch::{
    BatchConfig, BatchError, BatchProcessor, TransactionBatchBuffer,
};
#[cfg(feature = "fulltext-search")]
pub use batch::FulltextBatchProcessor;
pub use circuit_breaker::{
    with_circuit_breaker, CircuitBreaker, CircuitBreakerConfig, CircuitBreakerError,
    CircuitBreakerStats, CircuitState,
};
#[cfg(feature = "fulltext-search")]
pub use coordinator::{
    ChangeContext, ChangeData, ChangeType, IndexType, RecoveryResult, SyncCoordinator,
    SyncCoordinatorError,
};
pub use dead_letter_queue::{DeadLetterEntry, DeadLetterQueue, DeadLetterQueueConfig};
pub use manager::{EdgeProps, EdgeRef, SyncError, SyncManager};
pub use retry::{with_retry, RetryConfig};
#[cfg(feature = "qdrant")]
pub use vector_sync::VectorSyncCoordinator;
