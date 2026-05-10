//! WAL Checkpoint Mechanism
//!
//! Provides checkpoint functionality for faster recovery and WAL file management.

use std::fs;
use std::path::{Path, PathBuf};

use super::types::{
    Lsn, PageId, Timestamp, TransactionId, WalError, WalFileHeader, WalResult, WAL_FILE_HEADER_SIZE,
};
use crate::storage::persistence::{DirtyPageId, TableType};

/// Checkpoint information
#[derive(Debug, Clone)]
pub struct Checkpoint {
    /// Checkpoint sequence number
    pub seq: u64,
    /// Timestamp of the checkpoint
    pub timestamp: Timestamp,
    /// LSN (Log Sequence Number) at checkpoint
    pub lsn: Lsn,
    /// WAL files that can be safely deleted after this checkpoint
    pub wal_files: Vec<PathBuf>,
    /// Active transactions at checkpoint time
    pub active_transactions: Vec<TransactionId>,
    /// Dirty pages that need to be flushed
    pub dirty_pages: Vec<PageId>,
    /// Redo LSN (where recovery should start)
    pub redo_lsn: Lsn,
}

/// Checkpoint mode (similar to SQLite's checkpoint modes)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CheckpointMode {
    /// Passive: checkpoint as many frames as possible without blocking writers
    Passive,
    /// Full: block until no writers, checkpoint all frames
    #[default]
    Full,
    /// Restart: same as Full, but ensures next writer restarts log
    Restart,
    /// Truncate: same as Restart, but also truncates WAL file
    Truncate,
}

/// Result of a checkpoint operation
#[derive(Debug, Clone, Default)]
pub struct CheckpointResult {
    /// Number of pages checkpointed
    pub pages_written: usize,
    /// Number of WAL files processed
    pub wal_files_processed: usize,
    /// Duration of checkpoint in microseconds
    pub duration_us: u64,
    /// Checkpoint mode used
    pub mode: CheckpointMode,
    /// Whether the checkpoint was successful
    pub success: bool,
}

/// Checkpoint manager for WAL
pub struct CheckpointManager {
    /// WAL directory path
    wal_dir: PathBuf,
    /// Current checkpoint sequence
    current_seq: u64,
    /// Last checkpoint timestamp
    last_checkpoint_ts: Timestamp,
    /// Last checkpoint LSN
    last_checkpoint_lsn: Lsn,
    /// Checkpoint file path
    checkpoint_file: PathBuf,
    /// Active transactions
    active_transactions: Vec<TransactionId>,
    /// Dirty pages
    dirty_pages: Vec<PageId>,
}

impl CheckpointManager {
    /// Create a new checkpoint manager
    pub fn new(wal_dir: &Path) -> Self {
        let checkpoint_file = wal_dir.join("checkpoint.meta");
        Self {
            wal_dir: wal_dir.to_path_buf(),
            current_seq: 0,
            last_checkpoint_ts: 0,
            last_checkpoint_lsn: Lsn::ZERO,
            checkpoint_file,
            active_transactions: Vec::new(),
            dirty_pages: Vec::new(),
        }
    }

    /// Initialize checkpoint manager and load existing checkpoint info
    pub fn init(&mut self) -> WalResult<()> {
        if !self.wal_dir.exists() {
            fs::create_dir_all(&self.wal_dir).map_err(|e| WalError::IoError(e.to_string()))?;
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
                    "lsn" => {
                        self.last_checkpoint_lsn = Lsn::new(value.trim().parse().unwrap_or(0));
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
            "seq={}\ntimestamp={}\nlsn={}\n",
            self.current_seq,
            self.last_checkpoint_ts,
            self.last_checkpoint_lsn.as_u64()
        );

        fs::write(&self.checkpoint_file, content).map_err(|e| WalError::IoError(e.to_string()))?;

        Ok(())
    }

    /// Register an active transaction
    pub fn register_transaction(&mut self, tx_id: TransactionId) {
        if !self.active_transactions.contains(&tx_id) {
            self.active_transactions.push(tx_id);
        }
    }

    /// Unregister a completed transaction
    pub fn unregister_transaction(&mut self, tx_id: TransactionId) {
        self.active_transactions.retain(|&id| id != tx_id);
    }

    /// Mark a page as dirty
    pub fn mark_page_dirty(&mut self, page_id: PageId) {
        if !self.dirty_pages.contains(&page_id) {
            self.dirty_pages.push(page_id);
        }
    }

    /// Mark a page as clean
    pub fn mark_page_clean(&mut self, page_id: PageId) {
        self.dirty_pages.retain(|&id| id != page_id);
    }

    /// Get active transactions
    pub fn active_transactions(&self) -> &[TransactionId] {
        &self.active_transactions
    }

    /// Get dirty pages
    pub fn dirty_pages(&self) -> &[PageId] {
        &self.dirty_pages
    }

    /// Create a new checkpoint
    pub fn create_checkpoint(&mut self, timestamp: Timestamp, lsn: Lsn) -> WalResult<Checkpoint> {
        self.current_seq += 1;
        self.last_checkpoint_ts = timestamp;
        self.last_checkpoint_lsn = lsn;

        let wal_files = self.get_wal_files_before_checkpoint()?;

        let redo_lsn = self.calculate_redo_lsn();

        let checkpoint = Checkpoint {
            seq: self.current_seq,
            timestamp,
            lsn,
            wal_files,
            active_transactions: self.active_transactions.clone(),
            dirty_pages: self.dirty_pages.clone(),
            redo_lsn,
        };

        self.save_checkpoint_meta()?;

        Ok(checkpoint)
    }

    /// Calculate the redo LSN (where recovery should start)
    fn calculate_redo_lsn(&self) -> Lsn {
        if self.active_transactions.is_empty() {
            self.last_checkpoint_lsn
        } else {
            Lsn::ZERO
        }
    }

    /// Create a checkpoint with full page writes
    pub fn create_checkpoint_with_full_pages(
        &mut self,
        timestamp: Timestamp,
        lsn: Lsn,
        dirty_pages: Vec<PageId>,
    ) -> WalResult<Checkpoint> {
        self.dirty_pages = dirty_pages;
        self.create_checkpoint(timestamp, lsn)
    }

    /// Create a checkpoint with specified mode
    pub fn checkpoint(
        &mut self,
        timestamp: Timestamp,
        lsn: Lsn,
        mode: CheckpointMode,
    ) -> WalResult<CheckpointResult> {
        let start_time = std::time::Instant::now();
        
        let checkpoint = match mode {
            CheckpointMode::Passive => self.checkpoint_passive(timestamp, lsn)?,
            CheckpointMode::Full => self.create_checkpoint(timestamp, lsn)?,
            CheckpointMode::Restart => self.checkpoint_restart(timestamp, lsn)?,
            CheckpointMode::Truncate => self.checkpoint_truncate(timestamp, lsn)?,
        };

        let duration_us = start_time.elapsed().as_micros() as u64;
        
        Ok(CheckpointResult {
            pages_written: checkpoint.dirty_pages.len(),
            wal_files_processed: checkpoint.wal_files.len(),
            duration_us,
            mode,
            success: true,
        })
    }

    /// Passive checkpoint: checkpoint without blocking writers
    fn checkpoint_passive(&mut self, timestamp: Timestamp, lsn: Lsn) -> WalResult<Checkpoint> {
        self.create_checkpoint(timestamp, lsn)
    }

    /// Restart checkpoint: full checkpoint and reset log
    fn checkpoint_restart(&mut self, timestamp: Timestamp, lsn: Lsn) -> WalResult<Checkpoint> {
        let checkpoint = self.create_checkpoint(timestamp, lsn)?;
        self.dirty_pages.clear();
        Ok(checkpoint)
    }

    /// Truncate checkpoint: full checkpoint, reset log, and truncate WAL
    fn checkpoint_truncate(&mut self, timestamp: Timestamp, lsn: Lsn) -> WalResult<Checkpoint> {
        let checkpoint = self.checkpoint_restart(timestamp, lsn)?;
        
        for wal_file in &checkpoint.wal_files {
            if wal_file.exists() {
                fs::remove_file(wal_file).map_err(|e| WalError::IoError(e.to_string()))?;
            }
        }
        
        Ok(checkpoint)
    }

    /// Get WAL files that can be deleted before current checkpoint
    fn get_wal_files_before_checkpoint(&self) -> WalResult<Vec<PathBuf>> {
        let mut wal_files = Vec::new();

        if !self.wal_dir.exists() {
            return Ok(wal_files);
        }

        let entries = fs::read_dir(&self.wal_dir).map_err(|e| WalError::IoError(e.to_string()))?;

        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "wal") {
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

        let mut file = File::open(path).map_err(|e| WalError::IoError(e.to_string()))?;

        let mut buffer = [0u8; WAL_FILE_HEADER_SIZE];
        if let Ok(()) = file.read_exact(&mut buffer) {
            if let Some(header) = WalFileHeader::from_bytes(&buffer) {
                return Ok(header.checkpoint_seq < self.current_seq);
            }
        }

        Ok(false)
    }

    /// Clean up WAL files before a checkpoint
    pub fn cleanup_before_checkpoint(&self, checkpoint: &Checkpoint) -> WalResult<usize> {
        let mut deleted_count = 0;

        for path in &checkpoint.wal_files {
            if path.exists() {
                fs::remove_file(path).map_err(|e| WalError::IoError(e.to_string()))?;
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

    /// Get last checkpoint LSN
    pub fn last_checkpoint_lsn(&self) -> Lsn {
        self.last_checkpoint_lsn
    }

    /// Get the latest checkpoint info
    pub fn get_latest_checkpoint(&self) -> Option<Checkpoint> {
        if self.current_seq == 0 {
            return None;
        }

        Some(Checkpoint {
            seq: self.current_seq,
            timestamp: self.last_checkpoint_ts,
            lsn: self.last_checkpoint_lsn,
            wal_files: Vec::new(),
            active_transactions: self.active_transactions.clone(),
            dirty_pages: self.dirty_pages.clone(),
            redo_lsn: self.calculate_redo_lsn(),
        })
    }

    /// Mark a dirty page using DirtyPageId
    pub fn mark_dirty_page(&mut self, page_id: &DirtyPageId) {
        let raw_id = page_id.to_u64();
        self.mark_page_dirty(raw_id);
    }

    /// Mark a dirty page as clean using DirtyPageId
    pub fn mark_clean_page(&mut self, page_id: &DirtyPageId) {
        let raw_id = page_id.to_u64();
        self.mark_page_clean(raw_id);
    }

    /// Get dirty pages as DirtyPageId vector
    pub fn dirty_pages_as_dirty_page_ids(&self) -> Vec<DirtyPageId> {
        self.dirty_pages
            .iter()
            .filter_map(|&raw_id| DirtyPageId::try_from_u64(raw_id))
            .collect()
    }

    /// Create checkpoint with DirtyPageId list
    pub fn create_checkpoint_with_dirty_page_ids(
        &mut self,
        timestamp: Timestamp,
        lsn: Lsn,
        dirty_page_ids: &[DirtyPageId],
    ) -> WalResult<Checkpoint> {
        let raw_ids: Vec<PageId> = dirty_page_ids.iter().map(|id| id.to_u64()).collect();
        self.create_checkpoint_with_full_pages(timestamp, lsn, raw_ids)
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
        assert_eq!(manager.last_checkpoint_lsn(), Lsn::ZERO);
    }

    #[test]
    fn test_create_checkpoint() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let wal_path = temp_dir.path();

        let mut manager = CheckpointManager::new(wal_path);
        manager.init().expect("Failed to init");

        let checkpoint = manager
            .create_checkpoint(100, Lsn::new(1000))
            .expect("Failed to create checkpoint");

        assert_eq!(checkpoint.seq, 1);
        assert_eq!(checkpoint.timestamp, 100);
        assert_eq!(checkpoint.lsn, Lsn::new(1000));
        assert_eq!(manager.current_seq(), 1);
        assert_eq!(manager.last_checkpoint_ts(), 100);
        assert_eq!(manager.last_checkpoint_lsn(), Lsn::new(1000));
    }

    #[test]
    fn test_checkpoint_persistence() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let wal_path = temp_dir.path();

        {
            let mut manager = CheckpointManager::new(wal_path);
            manager.init().expect("Failed to init");
            manager
                .create_checkpoint(100, Lsn::new(1000))
                .expect("Failed to create checkpoint");
        }

        {
            let mut manager = CheckpointManager::new(wal_path);
            manager.init().expect("Failed to init");
            assert_eq!(manager.current_seq(), 1);
            assert_eq!(manager.last_checkpoint_ts(), 100);
            assert_eq!(manager.last_checkpoint_lsn(), Lsn::new(1000));
        }
    }

    #[test]
    fn test_get_latest_checkpoint() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let wal_path = temp_dir.path();

        let mut manager = CheckpointManager::new(wal_path);
        manager.init().expect("Failed to init");

        assert!(manager.get_latest_checkpoint().is_none());

        manager
            .create_checkpoint(100, Lsn::new(1000))
            .expect("Failed to create checkpoint");

        let latest = manager.get_latest_checkpoint().expect("No checkpoint");
        assert_eq!(latest.seq, 1);
        assert_eq!(latest.timestamp, 100);
        assert_eq!(latest.lsn, Lsn::new(1000));
    }

    #[test]
    fn test_active_transactions() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let wal_path = temp_dir.path();

        let mut manager = CheckpointManager::new(wal_path);
        manager.init().expect("Failed to init");

        manager.register_transaction(1);
        manager.register_transaction(2);
        manager.register_transaction(1);

        assert_eq!(manager.active_transactions().len(), 2);
        assert!(manager.active_transactions().contains(&1));
        assert!(manager.active_transactions().contains(&2));

        manager.unregister_transaction(1);
        assert_eq!(manager.active_transactions().len(), 1);
        assert!(!manager.active_transactions().contains(&1));
    }

    #[test]
    fn test_dirty_pages() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let wal_path = temp_dir.path();

        let mut manager = CheckpointManager::new(wal_path);
        manager.init().expect("Failed to init");

        manager.mark_page_dirty(100);
        manager.mark_page_dirty(200);
        manager.mark_page_dirty(100);

        assert_eq!(manager.dirty_pages().len(), 2);
        assert!(manager.dirty_pages().contains(&100));
        assert!(manager.dirty_pages().contains(&200));

        manager.mark_page_clean(100);
        assert_eq!(manager.dirty_pages().len(), 1);
        assert!(!manager.dirty_pages().contains(&100));
    }

    #[test]
    fn test_checkpoint_with_full_pages() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let wal_path = temp_dir.path();

        let mut manager = CheckpointManager::new(wal_path);
        manager.init().expect("Failed to init");

        let dirty_pages = vec![100, 200, 300];
        let checkpoint = manager
            .create_checkpoint_with_full_pages(100, Lsn::new(1000), dirty_pages.clone())
            .expect("Failed to create checkpoint");

        assert_eq!(checkpoint.dirty_pages, dirty_pages);
        assert_eq!(manager.dirty_pages().len(), 3);
    }
}
