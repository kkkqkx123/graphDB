//! WAL Checkpoint Mechanism
//!
//! Provides checkpoint functionality for faster recovery and WAL file management.

use std::fs;
use std::path::{Path, PathBuf};

use super::types::{Timestamp, WalError, WalFileHeader, WalResult, WAL_FILE_HEADER_SIZE};

/// Checkpoint information
#[derive(Debug, Clone)]
pub struct Checkpoint {
    /// Checkpoint sequence number
    pub seq: u64,
    /// Timestamp of the checkpoint
    pub timestamp: Timestamp,
    /// LSN (Log Sequence Number) at checkpoint
    pub lsn: u64,
    /// WAL files that can be safely deleted after this checkpoint
    pub wal_files: Vec<PathBuf>,
}

/// Checkpoint manager for WAL
pub struct CheckpointManager {
    /// WAL directory path
    wal_dir: PathBuf,
    /// Current checkpoint sequence
    current_seq: u64,
    /// Last checkpoint timestamp
    last_checkpoint_ts: Timestamp,
    /// Checkpoint file path
    checkpoint_file: PathBuf,
}

impl CheckpointManager {
    /// Create a new checkpoint manager
    pub fn new(wal_dir: &Path) -> Self {
        let checkpoint_file = wal_dir.join("checkpoint.meta");
        Self {
            wal_dir: wal_dir.to_path_buf(),
            current_seq: 0,
            last_checkpoint_ts: 0,
            checkpoint_file,
        }
    }

    /// Initialize checkpoint manager and load existing checkpoint info
    pub fn init(&mut self) -> WalResult<()> {
        if !self.wal_dir.exists() {
            fs::create_dir_all(&self.wal_dir)
                .map_err(|e| WalError::IoError(e.to_string()))?;
        }

        self.load_checkpoint_meta()?;
        Ok(())
    }

    /// Load checkpoint metadata from file
    fn load_checkpoint_meta(&mut self) -> WalResult<()> {
        if !self.checkpoint_file.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&self.checkpoint_file)
            .map_err(|e| WalError::IoError(e.to_string()))?;

        for line in content.lines() {
            if let Some((key, value)) = line.split_once('=') {
                match key.trim() {
                    "seq" => {
                        self.current_seq = value.trim().parse().unwrap_or(0);
                    }
                    "timestamp" => {
                        self.last_checkpoint_ts = value.trim().parse().unwrap_or(0);
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    /// Save checkpoint metadata to file
    fn save_checkpoint_meta(&self) -> WalResult<()> {
        let content = format!(
            "seq={}\ntimestamp={}\n",
            self.current_seq, self.last_checkpoint_ts
        );

        fs::write(&self.checkpoint_file, content)
            .map_err(|e| WalError::IoError(e.to_string()))?;

        Ok(())
    }

    /// Create a new checkpoint
    pub fn create_checkpoint(
        &mut self,
        timestamp: Timestamp,
        lsn: u64,
    ) -> WalResult<Checkpoint> {
        self.current_seq += 1;
        self.last_checkpoint_ts = timestamp;

        let wal_files = self.get_wal_files_before_checkpoint()?;

        let checkpoint = Checkpoint {
            seq: self.current_seq,
            timestamp,
            lsn,
            wal_files,
        };

        self.save_checkpoint_meta()?;

        Ok(checkpoint)
    }

    /// Get WAL files that can be deleted before current checkpoint
    fn get_wal_files_before_checkpoint(&self) -> WalResult<Vec<PathBuf>> {
        let mut wal_files = Vec::new();

        if !self.wal_dir.exists() {
            return Ok(wal_files);
        }

        let entries = fs::read_dir(&self.wal_dir)
            .map_err(|e| WalError::IoError(e.to_string()))?;

        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "wal") {
                if let Ok(true) = self.is_wal_file_before_checkpoint(&path) {
                    wal_files.push(path);
                }
            }
        }

        Ok(wal_files)
    }

    /// Check if a WAL file is before the current checkpoint
    fn is_wal_file_before_checkpoint(&self, path: &Path) -> WalResult<bool> {
        use std::fs::File;
        use std::io::Read;

        let mut file = File::open(path)
            .map_err(|e| WalError::IoError(e.to_string()))?;

        let mut buffer = [0u8; WAL_FILE_HEADER_SIZE];
        match file.read_exact(&mut buffer) {
            Ok(()) => {
                if let Some(header) = WalFileHeader::from_bytes(&buffer) {
                    return Ok(header.checkpoint_seq < self.current_seq);
                }
            }
            Err(_) => {}
        }

        Ok(false)
    }

    /// Clean up WAL files before a checkpoint
    pub fn cleanup_before_checkpoint(&self, checkpoint: &Checkpoint) -> WalResult<usize> {
        let mut deleted_count = 0;

        for path in &checkpoint.wal_files {
            if path.exists() {
                fs::remove_file(path)
                    .map_err(|e| WalError::IoError(e.to_string()))?;
                deleted_count += 1;
            }
        }

        Ok(deleted_count)
    }

    /// Get current checkpoint sequence
    pub fn current_seq(&self) -> u64 {
        self.current_seq
    }

    /// Get last checkpoint timestamp
    pub fn last_checkpoint_ts(&self) -> Timestamp {
        self.last_checkpoint_ts
    }

    /// Get the latest checkpoint info
    pub fn get_latest_checkpoint(&self) -> Option<Checkpoint> {
        if self.current_seq == 0 {
            return None;
        }

        Some(Checkpoint {
            seq: self.current_seq,
            timestamp: self.last_checkpoint_ts,
            lsn: 0,
            wal_files: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_checkpoint_manager() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let wal_path = temp_dir.path();

        let mut manager = CheckpointManager::new(wal_path);
        manager.init().expect("Failed to init");

        assert_eq!(manager.current_seq(), 0);
        assert_eq!(manager.last_checkpoint_ts(), 0);
    }

    #[test]
    fn test_create_checkpoint() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let wal_path = temp_dir.path();

        let mut manager = CheckpointManager::new(wal_path);
        manager.init().expect("Failed to init");

        let checkpoint = manager.create_checkpoint(100, 1000).expect("Failed to create checkpoint");

        assert_eq!(checkpoint.seq, 1);
        assert_eq!(checkpoint.timestamp, 100);
        assert_eq!(checkpoint.lsn, 1000);
        assert_eq!(manager.current_seq(), 1);
        assert_eq!(manager.last_checkpoint_ts(), 100);
    }

    #[test]
    fn test_checkpoint_persistence() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let wal_path = temp_dir.path();

        {
            let mut manager = CheckpointManager::new(wal_path);
            manager.init().expect("Failed to init");
            manager.create_checkpoint(100, 1000).expect("Failed to create checkpoint");
        }

        {
            let mut manager = CheckpointManager::new(wal_path);
            manager.init().expect("Failed to init");
            assert_eq!(manager.current_seq(), 1);
            assert_eq!(manager.last_checkpoint_ts(), 100);
        }
    }

    #[test]
    fn test_get_latest_checkpoint() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let wal_path = temp_dir.path();

        let mut manager = CheckpointManager::new(wal_path);
        manager.init().expect("Failed to init");

        assert!(manager.get_latest_checkpoint().is_none());

        manager.create_checkpoint(100, 1000).expect("Failed to create checkpoint");

        let latest = manager.get_latest_checkpoint().expect("No checkpoint");
        assert_eq!(latest.seq, 1);
        assert_eq!(latest.timestamp, 100);
    }
}
