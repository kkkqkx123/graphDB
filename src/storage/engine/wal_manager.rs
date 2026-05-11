//! WAL Manager
//!
//! Simple wrapper for WAL (Write-Ahead Log) operations.
//! The actual WAL functionality (rotation, cleanup, compression, etc.)
//! is implemented in `crate::transaction::wal::LocalWalWriter`.

use crate::core::{StorageError, StorageResult};
use crate::transaction::wal::writer::WalWriter;
use crate::transaction::wal::{Lsn, WalConfig, LocalWalWriter, WalOpType};
use parking_lot::RwLock;
use std::path::Path;
use std::sync::Arc;

/// Simple WAL manager that wraps a WalWriter
pub struct WalManager {
    wal_writer: Option<Arc<RwLock<Box<dyn WalWriter>>>>,
    local_writer: Option<LocalWalWriter>,
    config: Option<WalConfig>,
    current_lsn: RwLock<Lsn>,
}

impl WalManager {
    pub fn new() -> Self {
        Self {
            wal_writer: None,
            local_writer: None,
            config: None,
            current_lsn: RwLock::new(Lsn::new(0)),
        }
    }

    pub fn with_config(config: WalConfig) -> StorageResult<Self> {
        let mut manager = Self::new();
        manager.config = Some(config);
        Ok(manager)
    }

    pub fn set_wal_writer(&mut self, wal_writer: Arc<RwLock<Box<dyn WalWriter>>>) {
        self.wal_writer = Some(wal_writer);
    }

    pub fn is_enabled(&self) -> bool {
        self.wal_writer.is_some() || self.local_writer.is_some()
    }

    pub fn write(&self, data: &[u8]) -> StorageResult<()> {
        if let Some(ref wal_writer) = self.wal_writer {
            let mut writer = wal_writer.write();
            writer
                .append(data)
                .map_err(|e| StorageError::wal_error(format!("Failed to write WAL: {:?}", e)))?;
        }
        Ok(())
    }

    pub fn sync(&self) -> StorageResult<()> {
        if let Some(ref wal_writer) = self.wal_writer {
            let writer = wal_writer.write();
            writer
                .sync()
                .map_err(|e| StorageError::wal_error(format!("Failed to sync WAL: {:?}", e)))?;
        }
        Ok(())
    }

    pub fn current_lsn(&self) -> Lsn {
        *self.current_lsn.read()
    }

    pub fn truncate(&mut self, lsn: Lsn) -> StorageResult<()> {
        *self.current_lsn.write() = lsn;
        Ok(())
    }

    pub fn replay_from_lsn(&mut self, lsn: Lsn) -> StorageResult<()> {
        *self.current_lsn.write() = lsn;
        Ok(())
    }

    pub fn append_entry(&mut self, op_type: WalOpType, txn_id: u64, data: &[u8]) -> StorageResult<Lsn> {
        let lsn = {
            let mut current = self.current_lsn.write();
            *current = Lsn::new(current.0 + 1);
            *current
        };

        if let Some(ref mut writer) = self.local_writer {
            writer
                .append_entry(op_type, txn_id as u32, data)
                .map_err(|e| StorageError::wal_error(format!("Failed to append WAL entry: {:?}", e)))?;
        }

        Ok(lsn)
    }
}

impl Default for WalManager {
    fn default() -> Self {
        Self::new()
    }
}
