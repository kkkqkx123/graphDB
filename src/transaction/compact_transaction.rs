//! Compact Transaction
//!
//! Provides compact transaction for MVCC-based graph database.
//! A compact transaction performs garbage collection and storage optimization,
//! including CSR compaction and removal of old versions.

use std::sync::Arc;

use super::read_transaction::INVALID_TIMESTAMP;
use super::version_manager::{VersionManager, VersionManagerError};
use super::wal::types::{Timestamp, WalHeader};
use super::wal::writer::WalWriter;

/// Compact transaction error
#[derive(Debug, Clone, thiserror::Error)]
pub enum CompactTransactionError {
    #[error("Version manager error: {0}")]
    VersionManagerError(#[from] VersionManagerError),

    #[error("WAL error: {0}")]
    WalError(String),

    #[error("Transaction already released")]
    AlreadyReleased,

    #[error("Compact failed: {0}")]
    CompactFailed(String),
}

/// Compact transaction result type
pub type CompactTransactionResult<T> = Result<T, CompactTransactionError>;

/// Compact Transaction
///
/// A transaction that performs storage compaction and garbage collection.
/// Like update transactions, compact transactions require exclusive access.
///
/// # Compaction Operations
///
/// - CSR compaction: Rebuilds CSR structures to remove deleted edges
/// - Version cleanup: Removes old MVCC versions that are no longer visible
/// - Space reclamation: Frees unused storage space
///
/// # Example
///
/// ```rust,ignore
/// let mut txn = CompactTransaction::new(&mut graph, &version_manager, &mut wal_writer, true, 0.8)?;
/// txn.commit()?;
/// ```
pub struct CompactTransaction<'a> {
    graph: &'a mut dyn CompactTarget,
    version_manager: &'a VersionManager,
    wal_writer: &'a mut dyn WalWriter,
    compact_csr: bool,
    reserve_ratio: f32,
    timestamp: Timestamp,
    wal_buffer: Vec<u8>,
}

/// Target for compact operations (will be PropertyGraph in phase 2)
pub trait CompactTarget: Send + Sync {
    /// Compact the graph storage
    ///
    /// # Arguments
    /// * `compact_csr` - Whether to compact CSR structures
    /// * `reserve_ratio` - Ratio of space to reserve (0.0 - 1.0)
    /// * `ts` - Transaction timestamp
    fn compact(&mut self, compact_csr: bool, reserve_ratio: f32, ts: Timestamp) -> CompactTransactionResult<()>;

    /// Get the current storage size
    fn storage_size(&self) -> usize;

    /// Get the used storage size
    fn used_storage_size(&self) -> usize;
}

impl<'a> CompactTransaction<'a> {
    /// Create a new compact transaction
    ///
    /// # Arguments
    /// * `graph` - The graph to compact
    /// * `version_manager` - Version manager for timestamp management
    /// * `wal_writer` - WAL writer for logging
    /// * `compact_csr` - Whether to compact CSR structures
    /// * `reserve_ratio` - Ratio of space to reserve (0.0 - 1.0)
    pub fn new(
        graph: &'a mut dyn CompactTarget,
        version_manager: &'a VersionManager,
        wal_writer: &'a mut dyn WalWriter,
        compact_csr: bool,
        reserve_ratio: f32,
    ) -> CompactTransactionResult<Self> {
        let timestamp = version_manager.acquire_update_timestamp()?;
        let mut wal_buffer = Vec::new();
        wal_buffer.resize(WalHeader::SIZE, 0);

        Ok(Self {
            graph,
            version_manager,
            wal_writer,
            compact_csr,
            reserve_ratio: reserve_ratio.clamp(0.0, 1.0),
            timestamp,
            wal_buffer,
        })
    }

    /// Get the transaction's timestamp
    pub fn timestamp(&self) -> Timestamp {
        self.timestamp
    }

    /// Get whether CSR compaction is enabled
    pub fn compact_csr(&self) -> bool {
        self.compact_csr
    }

    /// Get the reserve ratio
    pub fn reserve_ratio(&self) -> f32 {
        self.reserve_ratio
    }

    /// Get storage statistics before compaction
    pub fn storage_stats(&self) -> (usize, usize) {
        (self.graph.storage_size(), self.graph.used_storage_size())
    }

    /// Commit the compact transaction
    ///
    /// Performs the actual compaction and writes WAL.
    pub fn commit(mut self) -> CompactTransactionResult<()> {
        if self.timestamp == INVALID_TIMESTAMP {
            return Ok(());
        }

        let header = WalHeader::new(
            super::wal::types::WalOpType::DeleteVertex,
            self.timestamp,
            0,
        );
        let header_bytes = header.as_bytes();
        self.wal_buffer[..WalHeader::SIZE].copy_from_slice(header_bytes);

        self.wal_writer
            .append(&self.wal_buffer)
            .map_err(|e| CompactTransactionError::WalError(e.to_string()))?;

        self.wal_buffer.clear();

        log::info!("Starting compaction at timestamp {}", self.timestamp);

        self.graph.compact(self.compact_csr, self.reserve_ratio, self.timestamp)?;

        log::info!("Completed compaction at timestamp {}", self.timestamp);

        self.version_manager.release_update_timestamp(self.timestamp);
        self.version_manager.clear();
        self.timestamp = INVALID_TIMESTAMP;

        Ok(())
    }

    /// Abort the compact transaction
    ///
    /// Reverts the timestamp without performing compaction.
    pub fn abort(mut self) -> CompactTransactionResult<()> {
        if self.timestamp != INVALID_TIMESTAMP {
            self.wal_buffer.clear();
            self.version_manager.revert_update_timestamp(self.timestamp);
            self.timestamp = INVALID_TIMESTAMP;
        }
        Ok(())
    }
}

impl<'a> Drop for CompactTransaction<'a> {
    fn drop(&mut self) {
        if self.timestamp != INVALID_TIMESTAMP {
            self.version_manager.release_update_timestamp(self.timestamp);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::wal::writer::DummyWalWriter;

    struct MockCompactTarget;

    impl CompactTarget for MockCompactTarget {
        fn compact(&mut self, _compact_csr: bool, _reserve_ratio: f32, _ts: Timestamp) -> CompactTransactionResult<()> {
            Ok(())
        }

        fn storage_size(&self) -> usize {
            1024
        }

        fn used_storage_size(&self) -> usize {
            512
        }
    }

    #[test]
    fn test_compact_transaction_basic() {
        let vm = VersionManager::new();
        let mut target = MockCompactTarget;
        let mut wal = DummyWalWriter::new();

        let txn = CompactTransaction::new(&mut target, &vm, &mut wal, true, 0.8)
            .expect("Failed to create compact transaction");

        assert!(txn.timestamp() >= 1);
        assert!(txn.compact_csr());
        assert!((txn.reserve_ratio() - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_compact_transaction_commit() {
        let vm = VersionManager::new();
        let mut target = MockCompactTarget;
        let mut wal = DummyWalWriter::new();

        let txn = CompactTransaction::new(&mut target, &vm, &mut wal, true, 0.8)
            .expect("Failed to create compact transaction");

        txn.commit().expect("Commit failed");

        assert!(!vm.is_update_in_progress());
    }

    #[test]
    fn test_compact_transaction_abort() {
        let vm = VersionManager::new();
        let mut target = MockCompactTarget;
        let mut wal = DummyWalWriter::new();

        let txn = CompactTransaction::new(&mut target, &vm, &mut wal, true, 0.8)
            .expect("Failed to create compact transaction");

        txn.abort().expect("Abort failed");

        assert!(!vm.is_update_in_progress());
    }

    #[test]
    fn test_compact_transaction_reserve_ratio_clamp() {
        let vm = VersionManager::new();
        let mut target = MockCompactTarget;
        let mut wal = DummyWalWriter::new();

        let txn = CompactTransaction::new(&mut target, &vm, &mut wal, true, 1.5)
            .expect("Failed to create compact transaction");

        assert!((txn.reserve_ratio() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_compact_transaction_storage_stats() {
        let vm = VersionManager::new();
        let mut target = MockCompactTarget;
        let mut wal = DummyWalWriter::new();

        let txn = CompactTransaction::new(&mut target, &vm, &mut wal, true, 0.8)
            .expect("Failed to create compact transaction");

        let (total, used) = txn.storage_stats();
        assert_eq!(total, 1024);
        assert_eq!(used, 512);
    }
}
