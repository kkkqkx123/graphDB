//! Persistence Operations
//!
//! Provides persistence, checkpoint, and compaction operations.
//! This module delegates to PropertyGraph's flush operations for data persistence.

use std::path::PathBuf;

use crate::core::{StorageError, StorageResult};
use crate::storage::engine::persistence_coordinator::{CheckpointData, CheckpointInfo, CheckpointStats};
use crate::storage::vertex::Timestamp;

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

    pub fn save_data_to_dir(&self, dir: &PathBuf) -> StorageResult<()> {
        use std::fs::{self, File};
        use std::io::Write;

        let data_dir = dir.join("data");
        fs::create_dir_all(&data_dir)?;

        let version_file = data_dir.join("version");
        let mut file = File::create(&version_file)?;
        writeln!(file, "1")?;

        let graph = self.ctx.graph.read();
        graph.flush_tables_to_dir(&data_dir)?;

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
                let graph = graph.read();
                let mut vertex_count = 0u64;
                let mut edge_count = 0u64;

                let data_dir = checkpoint_dir.join("data");
                std::fs::create_dir_all(&data_dir)?;

                graph.flush_tables_to_dir(&data_dir)?;

                for (_, table) in graph.vertex_tables() {
                    vertex_count += table.total_count() as u64;
                }

                for (_, table) in graph.edge_tables() {
                    edge_count += table.edge_count() as u64;
                }

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
            let mut graph = graph.write();
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
        let mut graph = self.ctx.graph.write();

        let label_ids = graph.vertex_label_ids();

        for label_id in label_ids {
            let removed = graph.compact_vertex_table_with_ts(label_id, ts);
            if !removed.is_empty() {
                log::info!(
                    "Compacted label {}: removed {} vertices",
                    label_id,
                    removed.len()
                );
            }
        }

        let stats = graph.gc_index_tombstones(ts)?;
        if stats.total_removed() > 0 {
            log::info!(
                "Index GC: removed {} vertex entries, {} edge entries",
                stats.vertex_entries_removed,
                stats.edge_entries_removed
            );
        }

        Ok(())
    }

    pub fn load_from_disk(&self) -> StorageResult<()> {
        if let Some(ref path) = self.ctx.work_dir {
            let schema_path = path.join("schema");
            self.ctx.schema_manager.load_schema(&schema_path)?;

            {
                let mut graph = self.ctx.graph.write();
                graph.load()?;

                let index_path = path.join("indexes");
                graph.index_data_manager_mut().load(&index_path)?;
            }
        }
        Ok(())
    }

    pub fn save_to_disk(&self) -> StorageResult<()> {
        if let Some(ref path) = self.ctx.work_dir {
            std::fs::create_dir_all(path).map_err(|e| StorageError::io_error(e.to_string()))?;

            let schema_path = path.join("schema");
            std::fs::create_dir_all(&schema_path).map_err(|e| StorageError::io_error(e.to_string()))?;
            self.ctx.schema_manager.save_schema(&schema_path)?;

            {
                let graph = self.ctx.graph.read();
                graph.flush()?;

                let index_path = path.join("indexes");
                std::fs::create_dir_all(&index_path)
                    .map_err(|e| StorageError::io_error(e.to_string()))?;
                graph.index_data_manager().flush(&index_path)?;
            }
        }
        Ok(())
    }
}
