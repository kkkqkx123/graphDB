//! Recovery Manager
//!
//! Provides crash recovery functionality using WAL replay.

use std::path::PathBuf;

use postcard::from_bytes;

use crate::core::types::{LabelId, Timestamp};
use crate::core::{StorageError, StorageResult};
use crate::transaction::wal::{
    DeleteEdgeRedo, DeleteVertexRedo, InsertEdgeRedo, InsertVertexRedo, LocalWalParser,
    ParallelWalParser, ParsedWalEntry, RecoveryResult, UpdateEdgePropRedo, UpdateVertexPropRedo,
    WalOpType, WalParser, WalRecoveryMode,
};

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

/// Trait for applying recovered operations to the storage engine.
/// Implementors handle the actual data modifications during WAL replay.
pub trait RecoveryApplier {
    fn replay_insert_vertex(
        &mut self,
        label: LabelId,
        oid: &[u8],
        properties: &[(String, Vec<u8>)],
        ts: Timestamp,
    ) -> StorageResult<()>;

    fn replay_insert_edge(
        &mut self,
        redo: &InsertEdgeRedo,
        ts: Timestamp,
    ) -> StorageResult<()>;

    fn replay_update_vertex_prop(
        &mut self,
        label: LabelId,
        oid: &[u8],
        prop_name: &str,
        value: &[u8],
        ts: Timestamp,
    ) -> StorageResult<()>;

    fn replay_update_edge_prop(
        &mut self,
        redo: &UpdateEdgePropRedo,
        ts: Timestamp,
    ) -> StorageResult<()>;

    fn replay_delete_vertex(&mut self, label: LabelId, oid: &[u8], ts: Timestamp) -> StorageResult<()>;

    fn replay_delete_edge(
        &mut self,
        src_label: LabelId,
        src_oid: &[u8],
        dst_label: LabelId,
        dst_oid: &[u8],
        edge_label: LabelId,
        ts: Timestamp,
    ) -> StorageResult<()>;
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

    /// Perform crash recovery with a RecoveryApplier for WAL replay
    pub fn recover_with_applier(
        &mut self,
        applier: &mut dyn RecoveryApplier,
    ) -> StorageResult<RecoveryStats> {
        let start = std::time::Instant::now();

        self.stats = RecoveryStats::default();

        let wal_result = self.parse_wal_files()?;

        self.restore_from_checkpoint(&wal_result)?;

        self.replay_wal_entries(&wal_result, applier)?;

        self.stats.recovery_time_ms = start.elapsed().as_millis() as u64;

        Ok(self.stats.clone())
    }

    /// Perform crash recovery (legacy, without applier)
    pub fn recover(&mut self) -> StorageResult<RecoveryStats> {
        let start = std::time::Instant::now();

        self.stats = RecoveryStats::default();

        let wal_result = self.parse_wal_files()?;

        self.restore_from_checkpoint(&wal_result)?;

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

            Ok(RecoveryResult {
                all_entries: parser.parse_all_entries(),
                last_timestamp: parser.last_timestamp(),
                last_lsn: parser.last_lsn(),
                corrupted_count: parser.corrupted_count(),
                skipped_count: parser.skipped_count(),
            })
        }
    }

    /// Restore from checkpoint
    fn restore_from_checkpoint(&mut self, _wal_result: &RecoveryResult) -> StorageResult<()> {
        if !self.config.data_dir.exists() {
            std::fs::create_dir_all(&self.config.data_dir)?;
            return Ok(());
        }

        self.stats.checkpoints_processed = 1;

        Ok(())
    }

    /// Replay WAL entries using a RecoveryApplier
    fn replay_wal_entries(
        &mut self,
        wal_result: &RecoveryResult,
        applier: &mut dyn RecoveryApplier,
    ) -> StorageResult<()> {
        self.replay_parsed_entries(&wal_result.all_entries, applier)
    }

    /// Replay parsed WAL entries (new format)
    fn replay_parsed_entries(
        &mut self,
        entries: &[ParsedWalEntry],
        applier: &mut dyn RecoveryApplier,
    ) -> StorageResult<()> {
        for entry in entries {
            let op_type = match WalOpType::try_from(entry.header.op_type) {
                Ok(t) => t,
                Err(_) => {
                    self.stats.errors_encountered += 1;
                    continue;
                }
            };

            let ts = entry.header.timestamp;
            let payload = &entry.payload;

            match op_type {
                WalOpType::InsertVertex => {
                    match self.deserialize_insert_vertex(payload) {
                        Ok(redo) => {
                            applier.replay_insert_vertex(
                                redo.label,
                                &redo.oid,
                                &redo.properties,
                                ts,
                            )?;
                            self.stats.wal_entries_replayed += 1;
                        }
                        Err(e) => {
                            log::warn!("Failed to deserialize InsertVertex redo: {}", e);
                            self.stats.errors_encountered += 1;
                        }
                    }
                }
                WalOpType::InsertEdge => {
                    match self.deserialize_insert_edge(payload) {
                        Ok(redo) => {
                            applier.replay_insert_edge(&redo, ts)?;
                            self.stats.wal_entries_replayed += 1;
                        }
                        Err(e) => {
                            log::warn!("Failed to deserialize InsertEdge redo: {}", e);
                            self.stats.errors_encountered += 1;
                        }
                    }
                }
                WalOpType::UpdateVertexProp => {
                    match self.deserialize_update_vertex_prop(payload) {
                        Ok(redo) => {
                            applier.replay_update_vertex_prop(
                                redo.label,
                                &redo.oid,
                                &redo.prop_name,
                                &redo.value,
                                ts,
                            )?;
                            self.stats.wal_entries_replayed += 1;
                        }
                        Err(e) => {
                            log::warn!("Failed to deserialize UpdateVertexProp redo: {}", e);
                            self.stats.errors_encountered += 1;
                        }
                    }
                }
                WalOpType::UpdateEdgeProp => {
                    match self.deserialize_update_edge_prop(payload) {
                        Ok(redo) => {
                            applier.replay_update_edge_prop(&redo, ts)?;
                            self.stats.wal_entries_replayed += 1;
                        }
                        Err(e) => {
                            log::warn!("Failed to deserialize UpdateEdgeProp redo: {}", e);
                            self.stats.errors_encountered += 1;
                        }
                    }
                }
                WalOpType::DeleteVertex => {
                    match self.deserialize_delete_vertex(payload) {
                        Ok(redo) => {
                            applier.replay_delete_vertex(redo.label, &redo.oid, ts)?;
                            self.stats.wal_entries_replayed += 1;
                        }
                        Err(e) => {
                            log::warn!("Failed to deserialize DeleteVertex redo: {}", e);
                            self.stats.errors_encountered += 1;
                        }
                    }
                }
                WalOpType::DeleteEdge => {
                    match self.deserialize_delete_edge(payload) {
                        Ok(redo) => {
                            applier.replay_delete_edge(
                                redo.src_label,
                                &redo.src_oid,
                                redo.dst_label,
                                &redo.dst_oid,
                                redo.edge_label,
                                ts,
                            )?;
                            self.stats.wal_entries_replayed += 1;
                        }
                        Err(e) => {
                            log::warn!("Failed to deserialize DeleteEdge redo: {}", e);
                            self.stats.errors_encountered += 1;
                        }
                    }
                }
                _ => {
                    // Schema operations (CreateVertexType, CreateEdgeType, etc.)
                    // These are typically handled separately during schema recovery
                }
            }
        }

        Ok(())
    }

    fn deserialize_insert_vertex(&self, payload: &[u8]) -> StorageResult<InsertVertexRedo> {
        from_bytes(payload)
            .map_err(|e| StorageError::deserialize_error(e.to_string()))
    }

    fn deserialize_insert_edge(&self, payload: &[u8]) -> StorageResult<InsertEdgeRedo> {
        from_bytes(payload)
            .map_err(|e| StorageError::deserialize_error(e.to_string()))
    }

    fn deserialize_update_vertex_prop(&self, payload: &[u8]) -> StorageResult<UpdateVertexPropRedo> {
        from_bytes(payload)
            .map_err(|e| StorageError::deserialize_error(e.to_string()))
    }

    fn deserialize_update_edge_prop(&self, payload: &[u8]) -> StorageResult<UpdateEdgePropRedo> {
        from_bytes(payload)
            .map_err(|e| StorageError::deserialize_error(e.to_string()))
    }

    fn deserialize_delete_vertex(&self, payload: &[u8]) -> StorageResult<DeleteVertexRedo> {
        from_bytes(payload)
            .map_err(|e| StorageError::deserialize_error(e.to_string()))
    }

    fn deserialize_delete_edge(&self, payload: &[u8]) -> StorageResult<DeleteEdgeRedo> {
        from_bytes(payload)
            .map_err(|e| StorageError::deserialize_error(e.to_string()))
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
        assert_eq!(config.wal_dir, PathBuf::from("./data/wal"));
        assert_eq!(config.data_dir, PathBuf::from("./data"));
        assert!(config.parallel_recovery);
        assert!(config.verify_checksum);
    }

    #[test]
    fn test_recovery_stats_default() {
        let stats = RecoveryStats::default();
        assert_eq!(stats.wal_entries_replayed, 0);
        assert_eq!(stats.pages_restored, 0);
        assert_eq!(stats.checkpoints_processed, 0);
        assert_eq!(stats.recovery_time_ms, 0);
        assert_eq!(stats.errors_encountered, 0);
    }

    #[test]
    fn test_recovery_manager_new() {
        let config = RecoveryConfig::default();
        let manager = RecoveryManager::new(config);
        assert_eq!(manager.stats().wal_entries_replayed, 0);
    }
}
