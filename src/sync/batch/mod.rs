pub mod buffer;
pub mod config;
pub mod error;
pub mod processor;
pub mod trait_def;

pub use buffer::BatchBuffer;
pub use config::BatchConfig;
pub use error::BatchError;
pub use processor::GenericBatchProcessor;
pub use processor::TransactionBatchBuffer;
pub use trait_def::BatchProcessor;
pub use trait_def::TransactionBuffer;

// Compatibility aliases
pub type TaskBuffer = TransactionBatchBuffer;
pub type BufferError = BatchError;
