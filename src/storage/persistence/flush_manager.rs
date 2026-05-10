//! Flush Manager
//!
//! Manages background flushing of dirty pages to persistent storage.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use parking_lot::RwLock;

use super::dirty_tracker::{DirtyPageId, DirtyPageTracker};
use super::page_writer::FilePageWriter;
use crate::core::StorageResult;
use crate::storage::compression::{CompressionType, Compressor};
use crate::storage::page::PageManager;

/// Type alias for compatibility
pub type PageId = DirtyPageId;

/// Maximum number of retry attempts for failed flushes
const MAX_FLUSH_RETRIES: usize = 3;

/// Delay between retry attempts
const RETRY_DELAY_MS: u64 = 100;

#[derive(Debug, Clone)]
pub struct FlushConfig {
    pub flush_threshold: usize,
    pub flush_interval: Duration,
    pub compression: CompressionType,
    pub background_flush_enabled: bool,
    pub work_dir: PathBuf,
    pub max_retries: usize,
    pub retry_delay_ms: u64,
}

impl Default for FlushConfig {
    fn default() -> Self {
        Self {
            flush_threshold: 1000,
            flush_interval: Duration::from_secs(60),
            compression: CompressionType::Zstd { level: 3 },
            background_flush_enabled: true,
            work_dir: PathBuf::from("./data"),
            max_retries: MAX_FLUSH_RETRIES,
            retry_delay_ms: RETRY_DELAY_MS,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FlushTask {
    pub pages: Vec<PageId>,
}

#[derive(Debug, Default, Clone)]
pub struct FlushStats {
    pub total_flushes: u64,
    pub pages_flushed: u64,
    pub failed_flushes: u64,
    pub retries: u64,
}

pub trait PageWriter: Send + Sync {
    fn write_page(&self, page_id: &PageId, data: &[u8]) -> StorageResult<()>;
    fn read_page(&self, page_id: &PageId) -> StorageResult<Option<Vec<u8>>>;
}

pub struct FlushManager {
    dirty_tracker: Arc<DirtyPageTracker>,
    page_manager: Option<Arc<PageManager>>,
    page_writer: Option<Arc<dyn PageWriter>>,
    compressor: Compressor,
    config: FlushConfig,
    running: Arc<AtomicBool>,
    background_thread: RwLock<Option<JoinHandle<()>>>,
    stats: RwLock<FlushStats>,
}

impl FlushManager {
    pub fn new(config: FlushConfig) -> Self {
        let dirty_tracker = Arc::new(DirtyPageTracker::new(
            config.flush_threshold,
            config.flush_interval,
        ));

        Self {
            dirty_tracker,
            page_manager: None,
            page_writer: None,
            compressor: Compressor::new(config.compression),
            config,
            running: Arc::new(AtomicBool::new(false)),
            background_thread: RwLock::new(None),
            stats: RwLock::new(FlushStats::default()),
        }
    }

    pub fn with_page_manager(mut self, page_manager: Arc<PageManager>) -> Self {
        self.page_manager = Some(page_manager);
        self
    }

    pub fn with_page_writer(mut self, writer: Arc<dyn PageWriter>) -> Self {
        self.page_writer = Some(writer);
        self
    }

    pub fn with_file_writer(mut self) -> Self {
        let writer = Arc::new(
            FilePageWriter::new(self.config.work_dir.clone(), self.config.compression)
                .expect("Failed to create file page writer"),
        );
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
        let page_manager = self.page_manager.clone();
        let page_writer = self.page_writer.clone();
        let interval = self.config.flush_interval;
        let running = self.running.clone();
        let max_retries = self.config.max_retries;
        let retry_delay = Duration::from_millis(self.config.retry_delay_ms);

        let handle = thread::spawn(move || {
            while running.load(Ordering::Relaxed) {
                thread::sleep(interval);

                if dirty_tracker.should_flush() {
                    let pages = dirty_tracker.flush_and_reset();
                    if !pages.is_empty() {
                        if let Some(ref pm) = page_manager {
                            if let Some(ref pw) = page_writer {
                                let mut flushed = 0usize;
                                for page_id in &pages {
                                    for attempt in 0..=max_retries {
                                        if let Some(page) = pm.get_page(page_id).ok().flatten() {
                                            if page.is_dirty() {
                                                let data = page.to_bytes();
                                                match pw.write_page(page_id, &data) {
                                                    Ok(()) => {
                                                        let _ = pm.clear_dirty(page_id);
                                                        flushed += 1;
                                                        break;
                                                    }
                                                    Err(e) => {
                                                        if attempt == max_retries {
                                                            log::error!(
                                                                "Failed to flush page {:?} after {} attempts: {}",
                                                                page_id,
                                                                max_retries + 1,
                                                                e
                                                            );
                                                            dirty_tracker.mark_dirty(*page_id);
                                                        } else {
                                                            log::warn!(
                                                                "Flush attempt {} failed for page {:?}: {}, retrying...",
                                                                attempt + 1,
                                                                page_id,
                                                                e
                                                            );
                                                            thread::sleep(retry_delay);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                log::info!(
                                    "Background flush completed: {}/{} pages flushed",
                                    flushed,
                                    pages.len()
                                );
                            } else {
                                log::warn!("No page writer configured, re-queueing {} dirty pages", pages.len());
                                dirty_tracker.mark_dirty_batch(&pages);
                            }
                        } else {
                            log::warn!("No page manager configured, re-queueing {} dirty pages", pages.len());
                            dirty_tracker.mark_dirty_batch(&pages);
                        }
                    }
                }
            }
        });

        *self.background_thread.write() = Some(handle);

        Ok(())
    }

    pub fn stop_background_flush(&self) {
        self.running.store(false, Ordering::SeqCst);

        if let Some(handle) = self.background_thread.write().take() {
            let _ = handle.join();
        }
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

    pub fn do_flush(&self) -> StorageResult<usize> {
        let pages = self.dirty_tracker.flush_and_reset();
        if pages.is_empty() {
            return Ok(0);
        }

        let mut flushed = 0usize;
        let mut failed = Vec::new();

        if let Some(ref pm) = self.page_manager {
            if let Some(ref pw) = self.page_writer {
                for page_id in &pages {
                    for attempt in 0..=self.config.max_retries {
                        if let Some(page) = pm.get_page(page_id)? {
                            if page.is_dirty() {
                                let data = page.to_bytes();
                                match pw.write_page(page_id, &data) {
                                    Ok(()) => {
                                        let _ = pm.clear_dirty(page_id);
                                        flushed += 1;
                                        break;
                                    }
                                    Err(e) => {
                                        if attempt == self.config.max_retries {
                                            log::error!(
                                                "Failed to flush page {:?}: {}",
                                                page_id,
                                                e
                                            );
                                            failed.push(*page_id);
                                        } else {
                                            thread::sleep(Duration::from_millis(
                                                self.config.retry_delay_ms,
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if !failed.is_empty() {
            self.dirty_tracker.mark_dirty_batch(&failed);
        }

        self.stats.write().total_flushes += 1;
        self.stats.write().pages_flushed += flushed as u64;
        self.stats.write().failed_flushes += failed.len() as u64;

        Ok(flushed)
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

    pub fn stats(&self) -> FlushStats {
        self.stats.read().clone()
    }

    pub fn page_manager(&self) -> Option<&Arc<PageManager>> {
        self.page_manager.as_ref()
    }

    pub fn page_writer(&self) -> Option<&Arc<dyn PageWriter>> {
        self.page_writer.as_ref()
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
    use crate::storage::persistence::dirty_tracker::TableType;
    use std::collections::HashMap;
    use std::sync::Mutex;

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

        let page_id = PageId::new(TableType::Vertex, 1, 0);

        manager.mark_dirty(page_id);
        assert_eq!(manager.get_dirty_page_count(), 1);
    }

    #[test]
    fn test_flush_dirty_pages() {
        let config = FlushConfig::default();
        let manager = FlushManager::new(config);

        let page_id = PageId::new(TableType::Vertex, 1, 0);

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
        let decompressed = manager
            .decompress_data(&compressed)
            .expect("Decompress failed");

        assert_eq!(decompressed, data);
    }

    struct MockPageWriter {
        pages: Mutex<HashMap<PageId, Vec<u8>>>,
    }

    impl MockPageWriter {
        fn new() -> Self {
            Self {
                pages: Mutex::new(HashMap::new()),
            }
        }
    }

    impl PageWriter for MockPageWriter {
        fn write_page(&self, page_id: &PageId, data: &[u8]) -> StorageResult<()> {
            self.pages.lock().expect("lock poisoned").insert(*page_id, data.to_vec());
            Ok(())
        }

        fn read_page(&self, page_id: &PageId) -> StorageResult<Option<Vec<u8>>> {
            Ok(self.pages.lock().expect("lock poisoned").get(page_id).cloned())
        }
    }

    #[test]
    fn test_with_page_writer() {
        let config = FlushConfig::default();
        let mock_writer = Arc::new(MockPageWriter::new());

        let manager = FlushManager::new(config).with_page_writer(mock_writer.clone());

        assert!(manager.page_writer().is_some());
    }
}
