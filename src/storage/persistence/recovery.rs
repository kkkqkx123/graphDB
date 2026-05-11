//! Recovery Manager
//!
//! Provides crash recovery functionality combining WAL replay and page restoration.

use std::path::PathBuf;
use std::sync::Arc;

use oxicode::decode_from_slice;

use super::dirty_tracker::{DirtyPageId, DirtyPageTracker, TableType};
use crate::core::{StorageError, StorageResult};
use crate::transaction::wal::{
    DeleteEdgeRedo, DeleteVertexRedo, InsertEdgeRedo, InsertVertexRedo, LocalWalParser,
    ParallelWalParser, RecoveryResult, UpdateEdgePropRedo, UpdateVertexPropRedo, WalContentUnit,
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
        label: u16,
        oid: &[u8],
        properties: &[(String, Vec<u8>)],
        ts: u32,
    ) -> StorageResult<()>;

    fn replay_insert_edge(
        &mut self,
        src_label: u16,
        src_oid: &[u8],
        dst_label: u16,
        dst_oid: &[u8],
        edge_label: u16,
        properties: &[(String, Vec<u8>)],
        ts: u32,
    ) -> StorageResult<()>;

    fn replay_update_vertex_prop(
        &mut self,
        label: u16,
        oid: &[u8],
        prop_name: &str,
        value: &[u8],
        ts: u32,
    ) -> StorageResult<()>;

    fn replay_update_edge_prop(
        &mut self,
        src_label: u16,
        src_oid: &[u8],
        dst_label: u16,
        dst_oid: &[u8],
        edge_label: u16,
        prop_name: &str,
        value: &[u8],
        ts: u32,
    ) -> StorageResult<()>;

    fn replay_delete_vertex(&mut self, label: u16, oid: &[u8], ts: u32) -> StorageResult<()>;

    fn replay_delete_edge(
        &mut self,
        src_label: u16,
        src_oid: &[u8],
        dst_label: u16,
        dst_oid: &[u8],
        edge_label: u16,
        ts: u32,
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
        dirty_tracker: Option<Arc<DirtyPageTracker>>,
    ) -> StorageResult<RecoveryStats> {
        let start = std::time::Instant::now();

        self.stats = RecoveryStats::default();

        let wal_result = self.parse_wal_files()?;

        self.restore_from_checkpoint(&wal_result)?;

        self.replay_wal_entries(&wal_result, applier)?;

        if let Some(tracker) = dirty_tracker {
            self.restore_dirty_tracking(&wal_result, tracker)?;
        }

        self.stats.recovery_time_ms = start.elapsed().as_millis() as u64;

        Ok(self.stats.clone())
    }

    /// Perform crash recovery (legacy, without applier)
    pub fn recover(
        &mut self,
        dirty_tracker: Option<Arc<DirtyPageTracker>>,
    ) -> StorageResult<RecoveryStats> {
        let start = std::time::Instant::now();

        self.stats = RecoveryStats::default();

        let wal_result = self.parse_wal_files()?;

        self.restore_from_checkpoint(&wal_result)?;

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

    /// Replay WAL entries using a RecoveryApplier
    fn replay_wal_entries(
        &mut self,
        wal_result: &RecoveryResult,
        applier: &mut dyn RecoveryApplier,
    ) -> StorageResult<()> {
        for content in &wal_result.insert_wal_list {
            self.replay_insert_entries(content, applier)?;
        }

        for update in &wal_result.update_wal_list {
            self.replay_update_entry(update, applier)?;
            self.stats.wal_entries_replayed += 1;
        }

        Ok(())
    }

    /// Replay insert WAL entries (may contain multiple operations)
    fn replay_insert_entries(
        &mut self,
        content: &WalContentUnit,
        applier: &mut dyn RecoveryApplier,
    ) -> StorageResult<()> {
        let data = content.as_slice();
        let mut offset = 0;

        while offset < data.len() {
            let op_type = match WalOpType::try_from(data[offset]) {
                Ok(t) => t,
                Err(_) => {
                    self.stats.errors_encountered += 1;
                    break;
                }
            };
            offset += 1;

            if offset + 4 > data.len() {
                self.stats.errors_encountered += 1;
                break;
            }

            let len = u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]) as usize;
            offset += 4;

            if offset + len > data.len() {
                self.stats.errors_encountered += 1;
                break;
            }

            let payload = &data[offset..offset + len];
            offset += len;

            match op_type {
                WalOpType::InsertVertex => {
                    let redo: InsertVertexRedo = decode_from_slice(payload)
                        .map_err(|e| StorageError::deserialize_error(e.to_string()))?
                        .0;
                    applier.replay_insert_vertex(
                        redo.label,
                        &redo.oid,
                        &redo.properties,
                        0,
                    )?;
                    self.stats.wal_entries_replayed += 1;
                }
                WalOpType::InsertEdge => {
                    let redo: InsertEdgeRedo = decode_from_slice(payload)
                        .map_err(|e| StorageError::deserialize_error(e.to_string()))?
                        .0;
                    applier.replay_insert_edge(
                        redo.src_label,
                        &redo.src_oid,
                        redo.dst_label,
                        &redo.dst_oid,
                        redo.edge_label,
                        &redo.properties,
                        0,
                    )?;
                    self.stats.wal_entries_replayed += 1;
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Replay an update WAL entry
    fn replay_update_entry(
        &mut self,
        update: &crate::transaction::wal::UpdateWalUnit,
        applier: &mut dyn RecoveryApplier,
    ) -> StorageResult<()> {
        let data = update.content.as_slice();
        if data.is_empty() {
            return Ok(());
        }

        let mut offset = 0;

        while offset < data.len() {
            let op_type = match WalOpType::try_from(data[offset]) {
                Ok(t) => t,
                Err(_) => {
                    self.stats.errors_encountered += 1;
                    break;
                }
            };
            offset += 1;

            if offset + 4 > data.len() {
                self.stats.errors_encountered += 1;
                break;
            }

            let len = u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]) as usize;
            offset += 4;

            if offset + len > data.len() {
                self.stats.errors_encountered += 1;
                break;
            }

            let payload = &data[offset..offset + len];
            offset += len;

            let ts = update.timestamp;

            match op_type {
                WalOpType::UpdateVertexProp => {
                    let redo: UpdateVertexPropRedo =
                        serde_json::from_slice(payload).map_err(|e| {
                            StorageError::deserialize_error(e.to_string())
                        })?;
                    applier.replay_update_vertex_prop(
                        redo.label,
                        &redo.oid,
                        &redo.prop_name,
                        &redo.value,
                        ts,
                    )?;
                }
                WalOpType::UpdateEdgeProp => {
                    let redo: UpdateEdgePropRedo =
                        serde_json::from_slice(payload).map_err(|e| {
                            StorageError::deserialize_error(e.to_string())
                        })?;
                    applier.replay_update_edge_prop(
                        redo.src_label,
                        &redo.src_oid,
                        redo.dst_label,
                        &redo.dst_oid,
                        redo.edge_label,
                        &redo.prop_name,
                        &redo.value,
                        ts,
                    )?;
                }
                WalOpType::DeleteVertex => {
                    let redo: DeleteVertexRedo =
                        serde_json::from_slice(payload).map_err(|e| {
                            StorageError::deserialize_error(e.to_string())
                        })?;
                    applier.replay_delete_vertex(redo.label, &redo.oid, ts)?;
                }
                WalOpType::DeleteEdge => {
                    let redo: DeleteEdgeRedo =
                        serde_json::from_slice(payload).map_err(|e| {
                            StorageError::deserialize_error(e.to_string())
                        })?;
                    applier.replay_delete_edge(
                        redo.src_label,
                        &redo.src_oid,
                        redo.dst_label,
                        &redo.dst_oid,
                        redo.edge_label,
                        ts,
                    )?;
                }
                _ => {}
            }
        }

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

    struct MockApplier {
        inserted_vertices: std::sync::Mutex<Vec<(u16, Vec<u8>)>>,
        inserted_edges: std::sync::Mutex<Vec<(u16, u16, u16)>>,
    }

    impl MockApplier {
        fn new() -> Self {
            Self {
                inserted_vertices: std::sync::Mutex::new(Vec::new()),
                inserted_edges: std::sync::Mutex::new(Vec::new()),
            }
        }
    }

    impl RecoveryApplier for MockApplier {
        fn replay_insert_vertex(
            &mut self,
            label: u16,
            oid: &[u8],
            _properties: &[(String, Vec<u8>)],
            _ts: u32,
        ) -> StorageResult<()> {
            self.inserted_vertices
                .lock()
                .unwrap()
                .push((label, oid.to_vec()));
            Ok(())
        }

        fn replay_insert_edge(
            &mut self,
            src_label: u16,
            _src_oid: &[u8],
            dst_label: u16,
            _dst_oid: &[u8],
            edge_label: u16,
            _properties: &[(String, Vec<u8>)],
            _ts: u32,
        ) -> StorageResult<()> {
            self.inserted_edges
                .lock()
                .unwrap()
                .push((src_label, dst_label, edge_label));
            Ok(())
        }

        fn replay_update_vertex_prop(
            &mut self,
            _label: u16,
            _oid: &[u8],
            _prop_name: &str,
            _value: &[u8],
            _ts: u32,
        ) -> StorageResult<()> {
            Ok(())
        }

        fn replay_update_edge_prop(
            &mut self,
            _src_label: u16,
            _src_oid: &[u8],
            _dst_label: u16,
            _dst_oid: &[u8],
            _edge_label: u16,
            _prop_name: &str,
            _value: &[u8],
            _ts: u32,
        ) -> StorageResult<()> {
            Ok(())
        }

        fn replay_delete_vertex(
            &mut self,
            _label: u16,
            _oid: &[u8],
            _ts: u32,
        ) -> StorageResult<()> {
            Ok(())
        }

        fn replay_delete_edge(
            &mut self,
            _src_label: u16,
            _src_oid: &[u8],
            _dst_label: u16,
            _dst_oid: &[u8],
            _edge_label: u16,
            _ts: u32,
        ) -> StorageResult<()> {
            Ok(())
        }
    }

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

    #[test]
    fn test_mock_applier_insert_vertex() {
        let mut applier = MockApplier::new();
        let result = applier.replay_insert_vertex(1, b"test_oid", &[], 0);
        assert!(result.is_ok());
        assert_eq!(applier.inserted_vertices.lock().unwrap().len(), 1);
    }

    #[test]
    fn test_mock_applier_insert_edge() {
        let mut applier = MockApplier::new();
        let result = applier.replay_insert_edge(1, b"src", 2, b"dst", 3, &[], 0);
        assert!(result.is_ok());
        assert_eq!(applier.inserted_edges.lock().unwrap().len(), 1);
    }
}
