use crate::core::{StorageError, StorageResult};
use crate::storage::persistence::{
    DirtyPageTracker, FlushConfig, FlushManager, PageId,
    TableType as PersistenceTableType,
};
use crate::storage::vertex::LabelId;
use crate::transaction::wal::writer::WalWriter;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct FlushManagerWrapper {
    pub dirty_tracker: Option<Arc<DirtyPageTracker>>,
    pub flush_manager: Option<Arc<FlushManager>>,
    pub wal_writer: Option<Arc<RwLock<Box<dyn WalWriter>>>>,
    pub wal_enabled: bool,
}

impl FlushManagerWrapper {
    pub fn new(
        enable_incremental_flush: bool,
        flush_threshold: usize,
        flush_interval_secs: u64,
        compression: crate::storage::persistence::CompressionType,
        work_dir: std::path::PathBuf,
    ) -> Self {
        let dirty_tracker = if enable_incremental_flush {
            Some(Arc::new(DirtyPageTracker::new(
                flush_threshold,
                std::time::Duration::from_secs(flush_interval_secs),
            )))
        } else {
            None
        };

        let flush_manager = if enable_incremental_flush {
            let flush_config = FlushConfig {
                flush_threshold,
                flush_interval: std::time::Duration::from_secs(flush_interval_secs),
                compression,
                background_flush_enabled: true,
                work_dir,
            };
            Some(Arc::new(FlushManager::new(flush_config)))
        } else {
            None
        };

        Self {
            dirty_tracker,
            flush_manager,
            wal_writer: None,
            wal_enabled: false,
        }
    }

    pub fn set_wal_writer(&mut self, wal_writer: Arc<RwLock<Box<dyn WalWriter>>>) {
        self.wal_writer = Some(wal_writer);
        self.wal_enabled = true;
    }

    pub fn wal_enabled(&self) -> bool {
        self.wal_enabled
    }

    pub fn dirty_tracker(&self) -> Option<&Arc<DirtyPageTracker>> {
        self.dirty_tracker.as_ref()
    }

    pub fn flush_manager(&self) -> Option<&Arc<FlushManager>> {
        self.flush_manager.as_ref()
    }

    pub fn get_dirty_page_count(&self) -> usize {
        self.dirty_tracker
            .as_ref()
            .map(|t| t.get_dirty_page_count())
            .unwrap_or(0)
    }

    pub fn should_flush(&self) -> bool {
        self.dirty_tracker
            .as_ref()
            .map(|t| t.should_flush())
            .unwrap_or(false)
    }

    pub fn mark_vertex_dirty(&self, label_id: LabelId) {
        if let Some(ref tracker) = self.dirty_tracker {
            let page_id = PageId {
                table_type: PersistenceTableType::Vertex,
                label_id,
                block_number: 0,
            };
            tracker.mark_dirty(page_id);
        }
    }

    pub fn mark_edge_dirty(&self, src_label: LabelId, dst_label: LabelId, edge_label: LabelId) {
        if let Some(ref tracker) = self.dirty_tracker {
            let page_id = PageId {
                table_type: PersistenceTableType::Edge,
                label_id: edge_label,
                block_number: ((src_label as u64) << 32) | (dst_label as u64),
            };
            tracker.mark_dirty(page_id);
        }
    }

    pub fn flush_dirty_pages(&self) -> StorageResult<Vec<PageId>> {
        if let Some(ref tracker) = self.dirty_tracker {
            let pages = tracker.flush_and_reset();
            Ok(pages)
        } else {
            Ok(Vec::new())
        }
    }

    pub fn write_wal(&self, data: &[u8]) -> StorageResult<()> {
        if !self.wal_enabled {
            return Ok(());
        }

        if let Some(ref wal_writer) = self.wal_writer {
            let mut writer = wal_writer.write();
            writer
                .append(data)
                .map_err(|e| StorageError::WalError(format!("Failed to write WAL: {:?}", e)))?;
        }
        Ok(())
    }

    pub fn sync_wal(&self) -> StorageResult<()> {
        if let Some(ref wal_writer) = self.wal_writer {
            let writer = wal_writer.write();
            writer
                .sync()
                .map_err(|e| StorageError::WalError(format!("Failed to sync WAL: {:?}", e)))?;
        }
        Ok(())
    }

    pub fn start_background_flush(&self) -> StorageResult<()> {
        if let Some(ref flush_manager) = self.flush_manager {
            flush_manager.start_background_flush()?;
        }
        Ok(())
    }

    pub fn stop_background_flush(&self) {
        if let Some(ref flush_manager) = self.flush_manager {
            flush_manager.stop_background_flush();
        }
    }
}
