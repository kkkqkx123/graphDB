//! WAL Manager
//!
//! Unified WAL (Write-Ahead Log) manager that properly integrates with LocalWalWriter.
//! This module provides a single source of truth for LSN management and WAL operations.

use crate::core::{StorageError, StorageResult};
use crate::transaction::wal::writer::WalWriter;
use crate::transaction::wal::{LocalWalWriter, Lsn, WalConfig, WalOpType};
use parking_lot::RwLock;
use std::path::Path;
use std::sync::Arc;

/// Unified WAL manager that wraps LocalWalWriter
///
/// This manager ensures LSN consistency by delegating all LSN operations
/// to the underlying LocalWalWriter, avoiding the dual LSN tracking issue.
pub struct WalManager {
    local_writer: Option<Arc<RwLock<LocalWalWriter>>>,
    dyn_writer: Option<Arc<RwLock<Box<dyn WalWriter>>>>,
    config: WalConfig,
}

impl WalManager {
    pub fn new() -> Self {
        Self {
            local_writer: None,
            dyn_writer: None,
            config: WalConfig::default(),
        }
    }

    pub fn open(&mut self, wal_dir: &Path, thread_id: u32) -> StorageResult<()> {
        let wal_uri = wal_dir.to_string_lossy().to_string();
        let mut writer = LocalWalWriter::with_config(&wal_uri, thread_id, self.config.clone());
        writer
            .open()
            .map_err(|e| StorageError::wal_error(format!("Failed to open WAL: {:?}", e)))?;
        self.local_writer = Some(Arc::new(RwLock::new(writer)));
        Ok(())
    }

    pub fn set_wal_writer(&mut self, writer: Arc<RwLock<Box<dyn WalWriter>>>) {
        self.dyn_writer = Some(writer);
    }

    pub fn writer(&self) -> Option<Arc<RwLock<LocalWalWriter>>> {
        self.local_writer.clone()
    }

    pub fn is_enabled(&self) -> bool {
        self.local_writer.is_some() || self.dyn_writer.is_some()
    }

    pub fn current_lsn(&self) -> Lsn {
        if let Some(ref writer) = self.local_writer {
            writer.read().current_lsn()
        } else {
            Lsn::ZERO
        }
    }

    pub fn sync(&self) -> StorageResult<()> {
        if let Some(ref writer) = self.local_writer {
            writer
                .write()
                .sync()
                .map_err(|e| StorageError::wal_error(format!("Failed to sync WAL: {:?}", e)))?;
        } else if let Some(ref writer) = self.dyn_writer {
            writer
                .write()
                .sync()
                .map_err(|e| StorageError::wal_error(format!("Failed to sync WAL: {:?}", e)))?;
        }
        Ok(())
    }

    pub fn append_entry(
        &self,
        op_type: WalOpType,
        timestamp: u32,
        data: &[u8],
    ) -> StorageResult<Lsn> {
        if let Some(ref writer) = self.local_writer {
            let mut w = writer.write();
            w.append_entry(op_type, timestamp, data).map_err(|e| {
                StorageError::wal_error(format!("Failed to append WAL entry: {:?}", e))
            })?;
            Ok(w.current_lsn())
        } else if let Some(ref writer) = self.dyn_writer {
            let mut w = writer.write();
            w.append(data).map_err(|e| {
                StorageError::wal_error(format!("Failed to append WAL entry: {:?}", e))
            })?;
            Ok(Lsn::ZERO)
        } else {
            Err(StorageError::wal_error("WAL writer not initialized"))
        }
    }

    pub fn truncate(&self, lsn: Lsn) -> StorageResult<()> {
        if let Some(ref writer) = self.local_writer {
            writer.write().set_current_lsn(lsn);
        }
        Ok(())
    }

    pub fn close(&mut self) -> StorageResult<()> {
        if let Some(ref writer) = self.local_writer {
            writer.write().close();
        }
        self.local_writer = None;
        self.dyn_writer = None;
        Ok(())
    }
}

impl Default for WalManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_wal_manager_lsn_consistency() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let mut manager = WalManager::new();

        manager
            .open(temp_dir.path(), 0)
            .expect("Failed to open WAL");

        let lsn1 = manager
            .append_entry(WalOpType::InsertVertex, 1, b"test_data")
            .expect("Failed to append");

        assert!(lsn1.as_u64() > 0);
        assert_eq!(manager.current_lsn(), lsn1);

        let lsn2 = manager
            .append_entry(WalOpType::InsertEdge, 2, b"more_data")
            .expect("Failed to append");

        assert!(lsn2.as_u64() > lsn1.as_u64());
        assert_eq!(manager.current_lsn(), lsn2);

        manager.close().expect("Failed to close");
    }
}
