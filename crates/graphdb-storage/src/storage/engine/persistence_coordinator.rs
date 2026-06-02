//! Persistence Coordinator
//!
//! Unified coordinator for the persistence responsibility chain:
//!
//! ```text
//! Write Operations
//!     ↓
//! WAL (Write-Ahead Log) - Guarantees durability
//!     ↓
//! Memory (RAM) - Provides fast access
//!     ↓
//! Flush (Periodic) - Writes memory data to disk
//!     ↓
//! Checkpoint (Periodic) - Creates consistent snapshots
//!     ↓
//! Snapshot (Manual) - User-triggered full backup
//! ```
//!
//! Responsibilities:
//! - WalManager: WAL log management, ensures write-ahead logging
//! - PropertyGraph::flush_to_disk(): Memory-to-disk flushing (triggered by coordinator)
//! - CheckpointManager: Checkpoint creation and recovery
//! - SnapshotManager: Full backup management
//!
//! Usage:
//! 1. Write operations go through WAL first
//! 2. Periodic flush is triggered by the coordinator based on thresholds
//! 3. Checkpoints are created periodically or on demand
//! 4. Snapshots are user-triggered for full backups

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

use parking_lot::RwLock;

use crate::core::types::Timestamp;
use crate::core::{StorageError, StorageResult};
use crate::storage::engine::snapshot_manager::{SnapshotManager, SnapshotOptions};
use crate::storage::engine::WalManager;
use crate::transaction::wal::{CheckpointManager, Lsn};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersistenceState {
    Idle,
    Checkpointing,
    Snapshotting,
}

#[derive(Debug, Clone)]
pub struct CheckpointInfo {
    pub lsn: Lsn,
}

#[derive(Debug, Clone)]
pub struct CheckpointStats {
    pub checkpoint_id: u64,
    pub data_flushed: u64,
    pub wal_truncated: u64,
    pub duration: Duration,
    pub snapshot_created: bool,
}

#[derive(Debug, Clone)]
pub struct PersistenceConfig {
    pub data_dir: PathBuf,
    pub wal_dir: PathBuf,
    pub checkpoint_dir: PathBuf,
    pub snapshot_dir: PathBuf,
    pub auto_flush_interval: Duration,
    pub auto_checkpoint_interval: Duration,
    pub checkpoint_threshold: u64,
    pub max_wal_size: u64,
    pub enable_snapshots: bool,
    pub snapshot_interval: Duration,
}

impl Default for PersistenceConfig {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from("data"),
            wal_dir: PathBuf::from("wal"),
            checkpoint_dir: PathBuf::from("checkpoint"),
            snapshot_dir: PathBuf::from("snapshots"),
            auto_flush_interval: Duration::from_secs(60),
            auto_checkpoint_interval: Duration::from_secs(300),
            checkpoint_threshold: 10000,
            max_wal_size: 100 * 1024 * 1024,
            enable_snapshots: true,
            snapshot_interval: Duration::from_secs(3600),
        }
    }
}

pub struct PersistenceCoordinator {
    config: PersistenceConfig,
    wal_manager: Arc<RwLock<WalManager>>,
    checkpoint_manager: RwLock<CheckpointManager>,
    snapshot_manager: Option<Arc<SnapshotManager>>,
    last_checkpoint_time: RwLock<Instant>,
    last_flush_time: RwLock<Instant>,
    last_snapshot_time: RwLock<Option<SystemTime>>,
    pending_wal_entries: RwLock<u64>,
    state: RwLock<PersistenceState>,
}

impl PersistenceCoordinator {
    pub fn new(config: PersistenceConfig) -> StorageResult<Self> {
        std::fs::create_dir_all(&config.data_dir)?;
        std::fs::create_dir_all(&config.wal_dir)?;
        std::fs::create_dir_all(&config.checkpoint_dir)?;

        let mut wal_manager = WalManager::new();
        wal_manager.open(&config.wal_dir, 0)?;

        let mut checkpoint_manager =
            CheckpointManager::new(&config.wal_dir, &config.checkpoint_dir, None);
        checkpoint_manager.init().map_err(|e| {
            crate::core::StorageError::db_error(format!("Failed to init checkpoint manager: {}", e))
        })?;

        let snapshot_manager = if config.enable_snapshots {
            std::fs::create_dir_all(&config.snapshot_dir)?;
            Some(Arc::new(SnapshotManager::new(
                config.snapshot_dir.clone(),
                config.data_dir.join("snapshot_work"),
            )?))
        } else {
            None
        };

        Ok(Self {
            config,
            wal_manager: Arc::new(RwLock::new(wal_manager)),
            checkpoint_manager: RwLock::new(checkpoint_manager),
            snapshot_manager,
            last_checkpoint_time: RwLock::new(Instant::now()),
            last_flush_time: RwLock::new(Instant::now()),
            last_snapshot_time: RwLock::new(None),
            pending_wal_entries: RwLock::new(0),
            state: RwLock::new(PersistenceState::Idle),
        })
    }

    pub fn wal_manager(&self) -> Arc<RwLock<WalManager>> {
        self.wal_manager.clone()
    }

    fn set_state(&self, state: PersistenceState) {
        *self.state.write() = state;
    }

    pub fn should_flush(&self) -> bool {
        let pending = *self.pending_wal_entries.read();
        let last_flush = *self.last_flush_time.read();

        pending >= self.config.checkpoint_threshold
            || last_flush.elapsed() >= self.config.auto_flush_interval
    }

    pub fn should_checkpoint(&self) -> bool {
        let pending = *self.pending_wal_entries.read();
        let last_checkpoint = *self.last_checkpoint_time.read();

        pending >= self.config.checkpoint_threshold
            || last_checkpoint.elapsed() >= self.config.auto_checkpoint_interval
    }

    pub fn should_snapshot(&self) -> bool {
        if !self.config.enable_snapshots {
            return false;
        }

        if let Some(last_snapshot) = *self.last_snapshot_time.read() {
            if let Ok(elapsed) = last_snapshot.elapsed() {
                return elapsed >= self.config.snapshot_interval;
            }
        }

        true
    }

    pub fn create_checkpoint(
        &self,
        flush_data: impl FnOnce(&Path, Timestamp) -> StorageResult<CheckpointData>,
        timestamp: Timestamp,
    ) -> StorageResult<CheckpointStats> {
        let start = Instant::now();

        self.set_state(PersistenceState::Checkpointing);

        let wal_lsn = {
            let wal = self.wal_manager.read();
            wal.current_lsn()
        };

        log::info!(
            "Creating checkpoint at timestamp {}, LSN {}",
            timestamp,
            wal_lsn
        );

        let checkpoint = {
            let mut cm = self.checkpoint_manager.write();
            cm.create_checkpoint(timestamp, wal_lsn).map_err(|e| {
                crate::core::StorageError::db_error(format!("Failed to create checkpoint: {}", e))
            })?
        };

        let checkpoint_dir = self
            .config
            .checkpoint_dir
            .join(format!("checkpoint_{}", checkpoint.seq));
        std::fs::create_dir_all(&checkpoint_dir)?;

        let data = flush_data(&checkpoint_dir, timestamp)?;

        self.save_checkpoint_metadata(&checkpoint_dir, &checkpoint, &data)?;

        {
            let wal = self.wal_manager.read();
            wal.truncate(wal_lsn)?;
        }

        *self.pending_wal_entries.write() = 0;
        *self.last_checkpoint_time.write() = Instant::now();

        let snapshot_created = if self.should_snapshot() {
            self.set_state(PersistenceState::Snapshotting);
            if let Some(ref snapshot_manager) = self.snapshot_manager {
                let snapshot_options = SnapshotOptions::default();
                match snapshot_manager.create_snapshot(
                    crate::storage::engine::snapshot_manager::CreateSnapshotParams {
                        data_dir: self.config.data_dir.clone(),
                        snapshot_id: checkpoint.seq,
                        vertex_count: data.vertex_count,
                        edge_count: data.edge_count,
                        checkpoint_seq: checkpoint.seq,
                        wal_lsn: wal_lsn.into(),
                        options: snapshot_options,
                    },
                ) {
                    Ok(_) => {
                        *self.last_snapshot_time.write() = Some(SystemTime::now());
                        true
                    }
                    Err(e) => {
                        log::error!("Failed to create snapshot: {}", e);
                        false
                    }
                }
            } else {
                false
            }
        } else {
            false
        };

        self.set_state(PersistenceState::Idle);

        let stats = CheckpointStats {
            checkpoint_id: checkpoint.seq,
            data_flushed: data.data_size,
            wal_truncated: wal_lsn.into(),
            duration: start.elapsed(),
            snapshot_created,
        };

        log::info!(
            "Checkpoint {} completed in {:?}",
            checkpoint.seq,
            stats.duration
        );

        Ok(stats)
    }

    fn save_checkpoint_metadata(
        &self,
        dir: &Path,
        checkpoint: &crate::transaction::wal::Checkpoint,
        data: &CheckpointData,
    ) -> StorageResult<()> {
        use std::fs::File;
        use std::io::Write;

        let metadata_path = dir.join("checkpoint.meta");
        let mut file = File::create(metadata_path)?;

        writeln!(file, "checkpoint_id={}", checkpoint.seq)?;
        writeln!(file, "timestamp={}", checkpoint.timestamp)?;
        writeln!(file, "wal_lsn={}", checkpoint.lsn)?;
        writeln!(file, "vertex_count={}", data.vertex_count)?;
        writeln!(file, "edge_count={}", data.edge_count)?;
        writeln!(file, "data_size={}", data.data_size)?;
        writeln!(file, "created_at={:?}", SystemTime::now())?;

        Ok(())
    }

    pub fn load_latest_checkpoint(
        &self,
        load_data: impl FnOnce(&Path) -> StorageResult<()>,
    ) -> StorageResult<Option<CheckpointInfo>> {
        let checkpoints_dir = &self.config.checkpoint_dir;

        if !checkpoints_dir.exists() {
            return Ok(None);
        }

        let mut checkpoints: Vec<(u64, PathBuf)> = std::fs::read_dir(checkpoints_dir)?
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| {
                let path = entry.path();
                if path.is_dir() {
                    let name = path.file_name()?.to_string_lossy();
                    if name.starts_with("checkpoint_") {
                        let id: u64 = name.trim_start_matches("checkpoint_").parse().ok()?;
                        Some((id, path))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        checkpoints.sort_by_key(|(id, _)| std::cmp::Reverse(*id));

        if let Some((_, checkpoint_path)) = checkpoints.first() {
            let info = self.load_checkpoint_metadata(checkpoint_path)?;

            load_data(checkpoint_path)?;

            {
                let wal = self.wal_manager.read();
                wal.truncate(info.lsn)?;
            }

            return Ok(Some(info));
        }

        Ok(None)
    }

    fn load_checkpoint_metadata(&self, dir: &Path) -> StorageResult<CheckpointInfo> {
        use std::fs::File;
        use std::io::{BufRead, BufReader};

        let metadata_path = dir.join("checkpoint.meta");
        let file = File::open(metadata_path)?;
        let reader = BufReader::new(file);

        let mut lsn = 0u64;

        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() == 2 && parts[0] == "wal_lsn" {
                lsn = parts[1].parse().unwrap_or(0);
                break;
            }
        }

        Ok(CheckpointInfo { lsn: Lsn::new(lsn) })
    }

    pub fn verify_snapshot(&self, snapshot_id: u64) -> StorageResult<bool> {
        let snapshot_manager = self
            .snapshot_manager
            .as_ref()
            .ok_or_else(|| StorageError::not_supported("Snapshots are not enabled"))?;

        snapshot_manager.verify_snapshot(snapshot_id)
    }

    pub fn cleanup_old_snapshots(&self) -> StorageResult<usize> {
        let snapshot_manager = self
            .snapshot_manager
            .as_ref()
            .ok_or_else(|| StorageError::not_supported("Snapshots are not enabled"))?;

        snapshot_manager.cleanup_old_snapshots()
    }

    pub fn snapshot_stats(&self) -> SnapshotStats {
        if let Some(snapshot_manager) = self.snapshot_manager.as_ref() {
            SnapshotStats {
                snapshot_count: snapshot_manager.snapshot_count(),
                total_size_bytes: snapshot_manager.total_snapshot_size(),
                latest_snapshot_id: snapshot_manager.get_latest_snapshot().map(|info| info.id),
            }
        } else {
            SnapshotStats::default()
        }
    }

    pub fn reset_flush_timer(&mut self) {
        *self.last_flush_time.write() = Instant::now();
    }

    pub fn reset_checkpoint_timer(&mut self) {
        *self.last_checkpoint_time.write() = Instant::now();
    }

}

#[derive(Debug, Clone)]
pub struct CheckpointData {
    pub vertex_count: u64,
    pub edge_count: u64,
    pub data_size: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SnapshotStats {
    pub snapshot_count: usize,
    pub total_size_bytes: u64,
    pub latest_snapshot_id: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_persistence_config_default() {
        let config = PersistenceConfig::default();
        assert_eq!(config.data_dir, PathBuf::from("data"));
        assert_eq!(config.auto_flush_interval, Duration::from_secs(60));
    }
}
