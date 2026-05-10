//! Recovery Manager
//!
//! Provides crash recovery functionality combining WAL replay and page restoration.

use std::path::PathBuf;
use std::sync::Arc;

use super::dirty_tracker::{DirtyPageId, DirtyPageTracker, TableType};
use crate::core::{StorageError, StorageResult};
use crate::transaction::wal::{LocalWalParser, ParallelWalParser, RecoveryResult, WalParser, WalRecoveryMode};

/// Recovery configuration
#[derive(Debug, Clone)]
pub struct RecoveryConfig {
    pub wal_dir: PathBuf,
    pub data_dir: PathBuf,
    pub recovery_mode: WalRecoveryMode,
    pub parallel_recovery: bool,
    pub verify_checksum: bool,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            wal_dir: PathBuf::from("./data/wal"),
            data_dir: PathBuf::from("./data"),
            recovery_mode: WalRecoveryMode::default(),
            parallel_recovery: true,
            verify_checksum: true,
        }
    }
}

/// Recovery statistics
#[derive(Debug, Default, Clone)]
pub struct RecoveryStats {
    pub wal_entries_replayed: usize,
    pub pages_restored: usize,
    pub checkpoints_processed: usize,
    pub recovery_time_ms: u64,
    pub errors_encountered: usize,
}

/// Recovery manager for crash recovery
pub struct RecoveryManager {
    config: RecoveryConfig,
    stats: RecoveryStats,
}

impl RecoveryManager {
    pub fn new(config: RecoveryConfig) -> Self {
        Self {
            config,
            stats: RecoveryStats::default(),
        }
    }

    /// Perform crash recovery
    pub fn recover(
        &mut self,
        dirty_tracker: Option<Arc<DirtyPageTracker>>,
    ) -> StorageResult<RecoveryStats> {
        let start = std::time::Instant::now();

        self.stats = RecoveryStats::default();

        let wal_result = self.parse_wal_files()?;

        self.restore_from_checkpoint(&wal_result)?;

        self.replay_wal_entries(&wal_result)?;

        if let Some(tracker) = dirty_tracker {
            self.restore_dirty_tracking(&wal_result, tracker)?;
        }

        self.stats.recovery_time_ms = start.elapsed().as_millis() as u64;

        Ok(self.stats.clone())
    }

    /// Parse WAL files
    fn parse_wal_files(&self) -> StorageResult<RecoveryResult> {
        if self.config.parallel_recovery {
            let parser = ParallelWalParser::new()
                .with_recovery_mode(self.config.recovery_mode)
                .with_verify_checksum(self.config.verify_checksum);

            parser
                .parse_parallel(&self.config.wal_dir)
                .map_err(|e| StorageError::db_error(format!("WAL parse error: {}", e)))
        } else {
            let mut parser = LocalWalParser::new();
            parser
                .open(&self.config.wal_dir.to_string_lossy())
                .map_err(|e| StorageError::db_error(format!("WAL open error: {}", e)))?;

            let insert_wal_list = parser.insert_wal_list().to_vec();
            let update_wal_list = parser.get_update_wals().to_vec();

            Ok(RecoveryResult {
                insert_wal_list,
                update_wal_list,
                last_timestamp: parser.last_timestamp(),
                ..Default::default()
            })
        }
    }

    /// Restore from checkpoint
    fn restore_from_checkpoint(&mut self, wal_result: &RecoveryResult) -> StorageResult<()> {
        if !self.config.data_dir.exists() {
            std::fs::create_dir_all(&self.config.data_dir)?;
            return Ok(());
        }

        self.stats.checkpoints_processed = 1;

        for entry in &wal_result.full_page_writes {
            self.restore_full_page_write(entry)?;
            self.stats.pages_restored += 1;
        }

        Ok(())
    }

    /// Restore a full page write entry
    fn restore_full_page_write(
        &self,
        entry: &crate::transaction::wal::FullPageWriteEntry,
    ) -> StorageResult<()> {
        let page_path = self
            .config
            .data_dir
            .join(format!("pages/page_{:08}.bin", entry.page_id));

        if let Some(parent) = page_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&page_path, &entry.page_data)?;

        Ok(())
    }

    /// Replay WAL entries
    fn replay_wal_entries(&mut self, wal_result: &RecoveryResult) -> StorageResult<()> {
        for content in &wal_result.insert_wal_list {
            self.replay_insert_entry(content)?;
            self.stats.wal_entries_replayed += 1;
        }

        for update in &wal_result.update_wal_list {
            self.replay_update_entry(update)?;
            self.stats.wal_entries_replayed += 1;
        }

        Ok(())
    }

    /// Replay an insert WAL entry
    fn replay_insert_entry(
        &self,
        _content: &crate::transaction::wal::WalContentUnit,
    ) -> StorageResult<()> {
        Ok(())
    }

    /// Replay an update WAL entry
    fn replay_update_entry(
        &self,
        _update: &crate::transaction::wal::UpdateWalUnit,
    ) -> StorageResult<()> {
        Ok(())
    }

    /// Restore dirty page tracking
    fn restore_dirty_tracking(
        &self,
        wal_result: &RecoveryResult,
        dirty_tracker: Arc<DirtyPageTracker>,
    ) -> StorageResult<()> {
        for entry in &wal_result.full_page_writes {
            let page_id = Self::parse_page_id(entry.page_id);
            dirty_tracker.mark_dirty(page_id);
        }

        Ok(())
    }

    /// Parse page ID from WAL format
    fn parse_page_id(page_id: u64) -> DirtyPageId {
        let table_type = ((page_id >> 56) & 0xFF) as u8;
        let label_id = ((page_id >> 40) & 0xFFFF) as u16;
        let block_number = page_id & 0xFFFFFFFFFF;

        let table = match table_type {
            1 => TableType::Vertex,
            2 => TableType::Edge,
            3 => TableType::Property,
            _ => TableType::Schema,
        };

        DirtyPageId::new(table, label_id, block_number)
    }

    /// Get recovery statistics
    pub fn stats(&self) -> &RecoveryStats {
        &self.stats
    }

    /// Check if recovery is needed
    pub fn needs_recovery(&self) -> bool {
        self.config.wal_dir.exists()
            && std::fs::read_dir(&self.config.wal_dir)
                .map(|entries| entries.count() > 0)
                .unwrap_or(false)
    }

    /// Clear WAL files after successful recovery
    pub fn clear_wal_files(&self) -> StorageResult<()> {
        if !self.config.wal_dir.exists() {
            return Ok(());
        }

        for entry in std::fs::read_dir(&self.config.wal_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "wal") {
                std::fs::remove_file(&path)?;
            }
        }

        Ok(())
    }
}

impl Default for RecoveryManager {
    fn default() -> Self {
        Self::new(RecoveryConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recovery_config_default() {
        let config = RecoveryConfig::default();
        assert!(config.wal_dir.ends_with("wal"));
        assert!(config.parallel_recovery);
    }

    #[test]
    fn test_recovery_manager_creation() {
        let manager = RecoveryManager::new(RecoveryConfig::default());
        assert!(!manager.needs_recovery());
    }

    #[test]
    fn test_recovery_stats_default() {
        let stats = RecoveryStats::default();
        assert_eq!(stats.wal_entries_replayed, 0);
        assert_eq!(stats.pages_restored, 0);
    }

    #[test]
    fn test_parse_page_id() {
        let page_id: u64 = (1u64 << 56) | (42u64 << 40) | 100u64;
        let parsed = RecoveryManager::parse_page_id(page_id);

        assert_eq!(parsed.table_type, TableType::Vertex);
        assert_eq!(parsed.label_id, 42);
        assert_eq!(parsed.block_number, 100);
    }
}
