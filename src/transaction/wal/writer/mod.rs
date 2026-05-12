//! WAL Writer Module
//!
//! Provides Write-Ahead Log writing functionality with:
//! - Local file-based WAL writer
//! - Group commit batching
//! - Configurable compression (Zstd)
//! - Multiple sync policies
//! - File rotation and cleanup
//! - Archive support

mod traits;
mod dummy;
mod factory;
mod group_commit;
mod compression;
mod sync;
mod local;

pub use compression::decompress_payload;
pub use dummy::DummyWalWriter;
pub use group_commit::GroupCommitManager;
pub use local::LocalWalWriter;
pub use traits::WalWriter;
pub use factory::WalWriterFactory;
