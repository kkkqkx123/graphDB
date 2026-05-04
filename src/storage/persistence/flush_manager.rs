use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crossbeam_utils::atomic::AtomicCell;
use parking_lot::RwLock;

use super::compression::{CompressionType, Compressor};
use super::dirty_tracker::{DirtyPageTracker, PageId};
use crate::core::{StorageError, StorageResult};

#[derive(Debug, Clone)]
pub struct FlushConfig {
    pub flush_threshold: usize,
    pub flush_interval: Duration,
    pub compression: CompressionType,
    pub background_flush_enabled: bool,
    pub work_dir: PathBuf,
}

impl Default for FlushConfig {
    fn default() -> Self {
        Self {
            flush_threshold: 1000,
            flush_interval: Duration::from_secs(60),
            compression: CompressionType::Zstd { level: 3 },
            background_flush_enabled: true,
            work_dir: PathBuf::from("./data"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FlushTask {
    pub pages: Vec<PageId>,
}

pub trait PageWriter: Send + Sync {
    fn write_page(&self, page_id: &PageId, data: &[u8]) -> StorageResult<()>;
    fn read_page(&self, page_id: &PageId) -> StorageResult<Option<Vec<u8>>>;
}

pub struct FlushManager {
    dirty_tracker: Arc<DirtyPageTracker>,
    compressor: Compressor,
    config: FlushConfig,
    running: AtomicBool,
    background_thread: RwLock<Option<JoinHandle<()>>>,
    page_writer: Option<Arc<dyn PageWriter>>,
}

impl FlushManager {
    pub fn new(config: FlushConfig) -> Self {
        let dirty_tracker = Arc::new(DirtyPageTracker::new(
            config.flush_threshold,
            config.flush_interval,
        ));

        Self {
            dirty_tracker,
            compressor: Compressor::new(config.compression),
            config,
            running: AtomicBool::new(false),
            background_thread: RwLock::new(None),
            page_writer: None,
        }
    }

    pub fn with_page_writer(mut self, writer: Arc<dyn PageWriter>) -> Self {
        self.page_writer = Some(writer);
        self
    }

    pub fn start_background_flush(&self) -> StorageResult<()> {
        if !self.config.background_flush_enabled {
            return Ok(());
        }

        if self.running.swap(true, Ordering::SeqCst) {
            return Ok(());
        }

        let dirty_tracker = self.dirty_tracker.clone();
        let interval = self.config.flush_interval;
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();

        let handle = thread::spawn(move || {
            while running_clone.load(Ordering::Relaxed) {
                thread::sleep(interval);

                if dirty_tracker.should_flush() {
                    let pages = dirty_tracker.flush_and_reset();
                    if !pages.is_empty() {
                        log::debug!("Background flush triggered for {} pages", pages.len());
                    }
                }
            }
        });

        Ok(())
    }

    pub fn stop_background_flush(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    pub fn mark_dirty(&self, page_id: PageId) {
        self.dirty_tracker.mark_dirty(page_id);
    }

    pub fn mark_dirty_batch(&self, page_ids: &[PageId]) {
        self.dirty_tracker.mark_dirty_batch(page_ids);
    }

    pub fn should_flush(&self) -> bool {
        self.dirty_tracker.should_flush()
    }

    pub fn get_dirty_page_count(&self) -> usize {
        self.dirty_tracker.get_dirty_page_count()
    }

    pub fn flush_dirty_pages(&self) -> StorageResult<Vec<PageId>> {
        let pages = self.dirty_tracker.flush_and_reset();
        Ok(pages)
    }

    pub fn compress_data(&self, data: &[u8]) -> StorageResult<Vec<u8>> {
        self.compressor.compress(data)
    }

    pub fn decompress_data(&self, data: &[u8]) -> StorageResult<Vec<u8>> {
        self.compressor.decompress(data)
    }

    pub fn compressor(&self) -> &Compressor {
        &self.compressor
    }

    pub fn dirty_tracker(&self) -> &Arc<DirtyPageTracker> {
        &self.dirty_tracker
    }

    pub fn config(&self) -> &FlushConfig {
        &self.config
    }
}

impl Drop for FlushManager {
    fn drop(&mut self) {
        self.stop_background_flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flush_config_default() {
        let config = FlushConfig::default();
        assert_eq!(config.flush_threshold, 1000);
        assert_eq!(config.flush_interval, Duration::from_secs(60));
        assert!(config.background_flush_enabled);
    }

    #[test]
    fn test_flush_manager_creation() {
        let config = FlushConfig::default();
        let manager = FlushManager::new(config);

        assert_eq!(manager.get_dirty_page_count(), 0);
        assert!(!manager.should_flush());
    }

    #[test]
    fn test_mark_dirty() {
        let config = FlushConfig::default();
        let manager = FlushManager::new(config);

        let page_id = PageId {
            table_type: super::super::dirty_tracker::TableType::Vertex,
            label_id: 1,
            block_number: 0,
        };

        manager.mark_dirty(page_id);
        assert_eq!(manager.get_dirty_page_count(), 1);
    }

    #[test]
    fn test_flush_dirty_pages() {
        let config = FlushConfig::default();
        let manager = FlushManager::new(config);

        let page_id = PageId {
            table_type: super::super::dirty_tracker::TableType::Vertex,
            label_id: 1,
            block_number: 0,
        };

        manager.mark_dirty(page_id);
        assert_eq!(manager.get_dirty_page_count(), 1);

        let pages = manager.flush_dirty_pages().expect("Flush failed");
        assert_eq!(pages.len(), 1);
        assert_eq!(manager.get_dirty_page_count(), 0);
    }

    #[test]
    fn test_compress_decompress() {
        let config = FlushConfig {
            compression: CompressionType::None,
            ..Default::default()
        };
        let manager = FlushManager::new(config);

        let data = b"hello world";
        let compressed = manager.compress_data(data).expect("Compress failed");
        let decompressed = manager.decompress_data(&compressed).expect("Decompress failed");

        assert_eq!(decompressed, data);
    }
}
