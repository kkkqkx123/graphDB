//! Compact Transaction
//!
//! Provides compact transaction for MVCC-based graph database.
//! A compact transaction performs garbage collection and storage optimization,
//! including CSR compaction and removal of old versions.

use super::read_transaction::INVALID_TIMESTAMP;
use super::version_manager::{VersionManager, VersionManagerError};
use super::wal::types::WalHeader;
use super::wal::Timestamp;
use super::wal::writer::WalWriter;
use crate::interfaces::compact::{CompactConfig, CompactError, CompactStats};
use crate::interfaces::CompactTarget;

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

    #[error("Compact error: {0}")]
    CompactError(#[from] CompactError),
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
/// - Structure compaction: Rebuilds storage structures to remove deleted data
/// - Version cleanup: Removes old MVCC versions that are no longer visible
/// - Space reclamation: Frees unused storage space
///
/// # Example
///
/// ```rust,ignore
/// let mut txn = CompactTransaction::new(&mut graph, &version_manager, &mut wal_writer, true, 0.8)?;
/// txn.commit()?;
/// ```
pub struct CompactTransaction<'a, T: CompactTarget + ?Sized> {
    graph: &'a mut T,
    version_manager: &'a VersionManager,
    wal_writer: &'a mut dyn WalWriter,
    config: CompactConfig,
    timestamp: Timestamp,
    wal_buffer: Vec<u8>,
}

impl<'a, T: CompactTarget + ?Sized> CompactTransaction<'a, T> {
    /// Create a new compact transaction
    ///
    /// # Arguments
    /// * `graph` - The graph to compact
    /// * `version_manager` - Version manager for timestamp management
    /// * `wal_writer` - WAL writer for logging
    /// * `compact_csr` - Whether to compact CSR structures
    /// * `reserve_ratio` - Ratio of space to reserve (0.0 - 1.0)
    pub fn new(
        graph: &'a mut T,
        version_manager: &'a VersionManager,
        wal_writer: &'a mut dyn WalWriter,
        compact_csr: bool,
        reserve_ratio: f32,
    ) -> CompactTransactionResult<Self> {
        let timestamp = version_manager.acquire_update_timestamp()?;
        let wal_buffer = vec![0; WalHeader::SIZE];
        let config = CompactConfig::new(compact_csr, reserve_ratio);

        Ok(Self {
            graph,
            version_manager,
            wal_writer,
            config,
            timestamp,
            wal_buffer,
        })
    }

    /// Get the transaction's timestamp
    pub fn timestamp(&self) -> Timestamp {
        self.timestamp
    }

    pub fn compact_csr(&self) -> bool {
        self.config.enable_structure_compaction
    }

    pub fn reserve_ratio(&self) -> f32 {
        self.config.reserve_ratio
    }

    pub fn storage_stats(&self) -> CompactStats {
        self.graph.get_compact_stats()
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

        self.graph
            .compact(&self.config, self.timestamp)?;

        log::info!("Completed compaction at timestamp {}", self.timestamp);

        self.version_manager
            .release_update_timestamp(self.timestamp);
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

impl<'a, T: CompactTarget + ?Sized> Drop for CompactTransaction<'a, T> {
    fn drop(&mut self) {
        if self.timestamp != INVALID_TIMESTAMP {
            self.version_manager
                .release_update_timestamp(self.timestamp);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::wal::writer::DummyWalWriter;
    use super::*;
    use crate::interfaces::compact::{CompactConfig, CompactResult};

    struct MockCompactTarget;

    impl CompactTarget for MockCompactTarget {
        fn compact(
            &mut self,
            _config: &CompactConfig,
            _ts: Timestamp,
        ) -> CompactResult<()> {
            Ok(())
        }

        fn get_compact_stats(&self) -> CompactStats {
            CompactStats::new(1024, 512)
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

        let stats = txn.storage_stats();
        assert_eq!(stats.total_size, 1024);
        assert_eq!(stats.used_size, 512);
    }
}
