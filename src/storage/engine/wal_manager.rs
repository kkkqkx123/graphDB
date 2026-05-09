//! WAL Manager
//!
//! Simple wrapper for WAL (Write-Ahead Log) operations.
//! The actual WAL functionality (rotation, cleanup, compression, etc.)
//! is implemented in `crate::transaction::wal::LocalWalWriter`.

use crate::core::{StorageError, StorageResult};
use crate::transaction::wal::writer::WalWriter;
use parking_lot::RwLock;
use std::sync::Arc;

/// Simple WAL manager that wraps a WalWriter
pub struct WalManager {
    wal_writer: Option<Arc<RwLock<Box<dyn WalWriter>>>>,
}

impl WalManager {
    pub fn new() -> Self {
        Self { wal_writer: None }
    }

    pub fn set_wal_writer(&mut self, wal_writer: Arc<RwLock<Box<dyn WalWriter>>>) {
        self.wal_writer = Some(wal_writer);
    }

    pub fn is_enabled(&self) -> bool {
        self.wal_writer.is_some()
    }

    pub fn write(&self, data: &[u8]) -> StorageResult<()> {
        if let Some(ref wal_writer) = self.wal_writer {
            let mut writer = wal_writer.write();
            writer
                .append(data)
                .map_err(|e| StorageError::WalError(format!("Failed to write WAL: {:?}", e)))?;
        }
        Ok(())
    }

    pub fn sync(&self) -> StorageResult<()> {
        if let Some(ref wal_writer) = self.wal_writer {
            let writer = wal_writer.write();
            writer
                .sync()
                .map_err(|e| StorageError::WalError(format!("Failed to sync WAL: {:?}", e)))?;
        }
        Ok(())
    }
}

impl Default for WalManager {
    fn default() -> Self {
        Self::new()
    }
}
