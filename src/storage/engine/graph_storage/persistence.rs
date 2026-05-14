//! Persistence Operations
//!
//! Provides persistence, checkpoint, and compaction operations.
//! This module delegates to PropertyGraph's flush operations for data persistence.

use std::path::Path;

use crate::core::types::Timestamp;
use crate::core::{StorageError, StorageResult};
use crate::storage::engine::persistence_coordinator::{CheckpointData, CheckpointInfo, CheckpointStats};
use crate::transaction::compact_transaction::CompactTransaction;
use crate::interfaces::CompactTarget;
use crate::transaction::wal::recovery::{RecoveryConfig, RecoveryManager, RecoveryStats};
use crate::transaction::wal::writer::WalWriter;

use super::context::GraphStorageContext;

pub struct PersistenceOps<'a> {
    ctx: &'a GraphStorageContext,
}

impl<'a> PersistenceOps<'a> {
    pub fn new(ctx: &'a GraphStorageContext) -> Self {
        Self { ctx }
    }

    pub fn save_data(&self) -> StorageResult<()> {
        let work_dir = self
            .ctx
            .work_dir
            .as_ref()
            .ok_or_else(|| StorageError::db_error("No work directory configured".to_string()))?;

        self.save_data_to_dir(work_dir)
    }

    pub fn save_data_to_dir(&self, dir: &Path) -> StorageResult<()> {
        use std::fs::{self, File};
        use std::io::Write;

        let data_dir = dir.join("data");
        fs::create_dir_all(&data_dir)?;

        let version_file = data_dir.join("version");
        let mut file = File::create(&version_file)?;
        writeln!(file, "1")?;

        self.ctx.graph.flush_tables_to_dir(&data_dir)?;

        log::info!("Data saved to {:?}", data_dir);
        Ok(())
    }

    pub fn flush(&self) -> StorageResult<()> {
        self.save_data()
    }

    pub fn create_checkpoint(&self) -> StorageResult<Option<CheckpointStats>> {
        let persistence = match &self.ctx.persistence {
            Some(p) => p,
            None => return Ok(None),
        };

        let ts = self.ctx.get_write_timestamp();
        let graph = self.ctx.graph.clone();

        let stats = persistence.write().create_checkpoint(
            |checkpoint_dir, _timestamp| {
                let data_dir = checkpoint_dir.join("data");
                std::fs::create_dir_all(&data_dir)?;

                graph.flush_tables_to_dir(&data_dir)?;

                let vertex_count = graph.total_vertex_count() as u64;
                let edge_count = graph.total_edge_count() as u64;

                let data_size = std::fs::metadata(&data_dir)
                    .map(|m| m.len())
                    .unwrap_or(0);

                Ok(CheckpointData {
                    vertex_count,
                    edge_count,
                    data_size,
                })
            },
            ts,
        )?;

        Ok(Some(stats))
    }

    pub fn load_latest_checkpoint(&self) -> StorageResult<Option<CheckpointInfo>> {
        let persistence = match &self.ctx.persistence {
            Some(p) => p,
            None => return Ok(None),
        };

        let graph = self.ctx.graph.clone();

        persistence.write().load_latest_checkpoint(|checkpoint_dir| {
            graph.restore_from_checkpoint(checkpoint_dir)
        })
    }

    pub fn should_flush(&self) -> bool {
        if let Some(ref persistence) = self.ctx.persistence {
            persistence.read().should_flush()
        } else {
            false
        }
    }

    pub fn should_checkpoint(&self) -> bool {
        if let Some(ref persistence) = self.ctx.persistence {
            persistence.read().should_checkpoint()
        } else {
            false
        }
    }

    pub fn auto_flush_if_needed(&self) -> StorageResult<bool> {
        if self.should_flush() {
            self.flush()?;
            if let Some(ref persistence) = self.ctx.persistence {
                persistence.write().reset_flush_timer();
            }
            return Ok(true);
        }
        Ok(false)
    }

    pub fn auto_checkpoint_if_needed(&self) -> StorageResult<Option<CheckpointStats>> {
        if self.should_checkpoint() {
            let stats = self.create_checkpoint()?;
            if let Some(ref persistence) = self.ctx.persistence {
                persistence.write().reset_checkpoint_timer();
            }
            return Ok(stats);
        }
        Ok(None)
    }

    pub fn compact_all(&self, ts: Timestamp) -> StorageResult<()> {
        let label_ids = self.ctx.graph.vertex_label_ids();

        for label_id in label_ids {
            let removed = self.ctx.graph.compact_vertex_table_with_ts(label_id, ts);
            if !removed.is_empty() {
                log::info!(
                    "Compacted label {}: removed {} vertices",
                    label_id,
                    removed.len()
                );
            }
        }

        let stats = self.ctx.graph.gc_index_tombstones(ts)?;
        if stats.total_removed() > 0 {
            log::info!(
                "Index GC: removed {} vertex entries, {} edge entries",
                stats.vertex_entries_removed,
                stats.edge_entries_removed
            );
        }

        Ok(())
    }

    /// Compact using CompactTransaction for transactional compaction
    ///
    /// This method uses CompactTarget trait for transactional storage compaction.
    /// It provides ACID guarantees for the compaction operation.
    pub fn compact_transactional(
        &self,
        compact_csr: bool,
        reserve_ratio: f32,
        wal_writer: &mut dyn WalWriter,
    ) -> StorageResult<()> {
        let version_manager = &self.ctx.version_manager;

        let txn = CompactTransaction::new(
            &*self.ctx.graph,
            version_manager,
            wal_writer,
            compact_csr,
            reserve_ratio,
        ).map_err(|e| StorageError::db_error(format!("Failed to create compact transaction: {}", e)))?;

        let before_stats = txn.storage_stats();
        log::info!(
            "Starting transactional compaction: compact_csr={}, reserve_ratio={:.2}, size={}/{}",
            compact_csr,
            reserve_ratio,
            before_stats.used_size,
            before_stats.total_size
        );

        txn.commit().map_err(|e| StorageError::db_error(format!("Compact transaction failed: {}", e)))?;

        let after_stats = self.ctx.graph.get_compact_stats();
        log::info!(
            "Compaction completed: size={}/{} (freed {} bytes)",
            after_stats.used_size,
            after_stats.total_size,
            before_stats.used_size.saturating_sub(after_stats.used_size)
        );

        Ok(())
    }

    pub fn load_from_disk(&self) -> StorageResult<()> {
        if let Some(ref path) = self.ctx.work_dir {
            let schema_path = path.join("schema");
            self.ctx.schema_manager.load_schema(&schema_path)?;

            self.ctx.graph.load()?;

            let index_path = path.join("indexes");
            self.ctx.graph.index_data_manager().write().load(&index_path)?;
        }
        Ok(())
    }

    pub fn save_to_disk(&self) -> StorageResult<()> {
        if let Some(ref path) = self.ctx.work_dir {
            std::fs::create_dir_all(path).map_err(|e| StorageError::io_error(e.to_string()))?;

            let schema_path = path.join("schema");
            std::fs::create_dir_all(&schema_path).map_err(|e| StorageError::io_error(e.to_string()))?;
            self.ctx.schema_manager.save_schema(&schema_path)?;

            self.ctx.graph.flush()?;

            let index_path = path.join("indexes");
            std::fs::create_dir_all(&index_path)
                .map_err(|e| StorageError::io_error(e.to_string()))?;
            self.ctx.graph.index_data_manager().read().flush(&index_path)?;
        }
        Ok(())
    }

    /// Recover from WAL using RecoveryApplier trait
    ///
    /// This method performs crash recovery by replaying WAL entries
    /// using the RecoveryApplier implementation on PropertyGraph.
    pub fn recover_from_wal(&self) -> StorageResult<RecoveryStats> {
        let work_dir = self
            .ctx
            .work_dir
            .as_ref()
            .ok_or_else(|| StorageError::db_error("No work directory configured".to_string()))?;

        let config = RecoveryConfig {
            wal_dir: work_dir.join("wal"),
            data_dir: work_dir.join("data"),
            ..Default::default()
        };

        let mut manager = RecoveryManager::new(config);

        manager.recover_with_applier(&*self.ctx.graph)
    }

    /// Recover from WAL with custom configuration
    pub fn recover_from_wal_with_config(&self, config: RecoveryConfig) -> StorageResult<RecoveryStats> {
        let mut manager = RecoveryManager::new(config);

        manager.recover_with_applier(&*self.ctx.graph)
    }

    /// Check if WAL recovery is needed
    ///
    /// Returns true if there are unflushed WAL entries that need recovery.
    pub fn needs_recovery(&self) -> bool {
        if let Some(ref work_dir) = self.ctx.work_dir {
            let wal_dir = work_dir.join("wal");
            if wal_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(&wal_dir) {
                    return entries.count() > 0;
                }
            }
        }
        false
    }
}
