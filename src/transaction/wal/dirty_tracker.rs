//! Dirty Page Tracking
//!
//! Tracks modified pages for efficient checkpointing and full page writes.

use std::collections::HashSet;
use std::time::{Duration, Instant};

use parking_lot::RwLock;

/// Table type identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TableType {
    Vertex = 1,
    Edge = 2,
    Schema = 4,
}

/// Dirty page identifier with table context
/// This is more detailed than the basic PageId (u64) in types.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DirtyPageId {
    /// Table type (Vertex, Edge, Schema)
    pub table_type: TableType,
    /// Label ID for the table
    pub label_id: u16,
    /// Block number within the table
    pub block_number: u64,
}

impl DirtyPageId {
    pub fn new(table_type: TableType, label_id: u16, block_number: u64) -> Self {
        Self {
            table_type,
            label_id,
            block_number,
        }
    }

    pub fn vertex(label_id: u16, block_number: u64) -> Self {
        Self::new(TableType::Vertex, label_id, block_number)
    }

    pub fn edge(label_id: u16, block_number: u64) -> Self {
        Self::new(TableType::Edge, label_id, block_number)
    }

    pub fn schema(block_number: u64) -> Self {
        Self::new(TableType::Schema, 0, block_number)
    }
}

/// Configuration for dirty page tracker
#[derive(Debug, Clone)]
pub struct DirtyTrackerConfig {
    /// Number of dirty pages to trigger flush
    pub flush_threshold: usize,
    /// Time interval to trigger flush
    pub flush_interval: Duration,
}

impl Default for DirtyTrackerConfig {
    fn default() -> Self {
        Self {
            flush_threshold: 1000,
            flush_interval: Duration::from_secs(60),
        }
    }
}

/// Thread-safe dirty page tracker
pub struct DirtyPageTracker {
    /// Set of dirty page IDs
    dirty_pages: RwLock<HashSet<DirtyPageId>>,
    /// Last flush timestamp
    last_flush: RwLock<Instant>,
    /// Flush threshold (number of pages)
    flush_threshold: usize,
    /// Flush interval (time-based)
    flush_interval: Duration,
    /// Pages modified since last checkpoint (for full page writes)
    pages_since_checkpoint: RwLock<HashSet<DirtyPageId>>,
}

impl DirtyPageTracker {
    pub fn new(flush_threshold: usize, flush_interval: Duration) -> Self {
        Self {
            dirty_pages: RwLock::new(HashSet::new()),
            last_flush: RwLock::new(Instant::now()),
            flush_threshold,
            flush_interval,
            pages_since_checkpoint: RwLock::new(HashSet::new()),
        }
    }

    pub fn with_config(config: DirtyTrackerConfig) -> Self {
        Self::new(config.flush_threshold, config.flush_interval)
    }

    /// Mark a page as dirty
    pub fn mark_dirty(&self, page_id: DirtyPageId) {
        self.dirty_pages.write().insert(page_id);
    }

    /// Mark multiple pages as dirty
    pub fn mark_dirty_batch(&self, page_ids: &[DirtyPageId]) {
        let mut dirty = self.dirty_pages.write();
        for page_id in page_ids {
            dirty.insert(*page_id);
        }
    }

    /// Mark a page as modified since checkpoint (for full page writes)
    pub fn mark_modified_since_checkpoint(&self, page_id: DirtyPageId) {
        self.pages_since_checkpoint.write().insert(page_id);
    }

    /// Check if a page was modified since last checkpoint
    pub fn is_modified_since_checkpoint(&self, page_id: &DirtyPageId) -> bool {
        self.pages_since_checkpoint.read().contains(page_id)
    }

    /// Clear checkpoint tracking (call after checkpoint completes)
    pub fn clear_checkpoint_tracking(&self) {
        self.pages_since_checkpoint.write().clear();
    }

    /// Unmark a page as dirty
    pub fn unmark_dirty(&self, page_id: &DirtyPageId) {
        self.dirty_pages.write().remove(page_id);
    }

    /// Check if a page is dirty
    pub fn is_dirty(&self, page_id: &DirtyPageId) -> bool {
        self.dirty_pages.read().contains(page_id)
    }

    /// Check if flush should be triggered
    pub fn should_flush(&self) -> bool {
        let dirty = self.dirty_pages.read();
        let threshold_reached = dirty.len() >= self.flush_threshold;
        drop(dirty);

        let time_reached = self.last_flush.read().elapsed() >= self.flush_interval;

        threshold_reached || time_reached
    }

    /// Get all dirty pages and reset the tracker
    pub fn flush_and_reset(&self) -> Vec<DirtyPageId> {
        let pages: Vec<DirtyPageId> = self.dirty_pages.write().drain().collect();
        *self.last_flush.write() = Instant::now();
        pages
    }

    /// Get dirty pages without resetting
    pub fn get_dirty_pages(&self) -> Vec<DirtyPageId> {
        self.dirty_pages.read().iter().copied().collect()
    }

    /// Get the number of dirty pages
    pub fn get_dirty_page_count(&self) -> usize {
        self.dirty_pages.read().len()
    }

    /// Clear all dirty pages
    pub fn clear(&self) {
        self.dirty_pages.write().clear();
    }

    /// Update the last flush time
    pub fn update_flush_time(&self) {
        *self.last_flush.write() = Instant::now();
    }

    /// Get time since last flush
    pub fn time_since_last_flush(&self) -> Duration {
        self.last_flush.read().elapsed()
    }

    /// Get pages that need full page writes (modified since checkpoint)
    pub fn get_pages_for_full_write(&self) -> Vec<DirtyPageId> {
        self.pages_since_checkpoint.read().iter().copied().collect()
    }
}

impl Default for DirtyPageTracker {
    fn default() -> Self {
        Self::with_config(DirtyTrackerConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mark_dirty() {
        let tracker = DirtyPageTracker::new(10, Duration::from_secs(60));
        let page_id = DirtyPageId::vertex(1, 0);

        assert!(!tracker.is_dirty(&page_id));

        tracker.mark_dirty(page_id);
        assert!(tracker.is_dirty(&page_id));
        assert_eq!(tracker.get_dirty_page_count(), 1);
    }

    #[test]
    fn test_mark_dirty_batch() {
        let tracker = DirtyPageTracker::new(10, Duration::from_secs(60));
        let pages = vec![
            DirtyPageId::vertex(1, 0),
            DirtyPageId::vertex(1, 1),
            DirtyPageId::edge(2, 0),
        ];

        tracker.mark_dirty_batch(&pages);
        assert_eq!(tracker.get_dirty_page_count(), 3);
    }

    #[test]
    fn test_unmark_dirty() {
        let tracker = DirtyPageTracker::new(10, Duration::from_secs(60));
        let page_id = DirtyPageId::vertex(1, 0);

        tracker.mark_dirty(page_id);
        assert!(tracker.is_dirty(&page_id));

        tracker.unmark_dirty(&page_id);
        assert!(!tracker.is_dirty(&page_id));
    }

    #[test]
    fn test_should_flush_threshold() {
        let tracker = DirtyPageTracker::new(3, Duration::from_secs(60));

        assert!(!tracker.should_flush());

        tracker.mark_dirty(DirtyPageId::vertex(1, 0));
        tracker.mark_dirty(DirtyPageId::vertex(1, 1));
        assert!(!tracker.should_flush());

        tracker.mark_dirty(DirtyPageId::vertex(1, 2));
        assert!(tracker.should_flush());
    }

    #[test]
    fn test_flush_and_reset() {
        let tracker = DirtyPageTracker::new(10, Duration::from_secs(60));
        let page1 = DirtyPageId::vertex(1, 0);
        let page2 = DirtyPageId::edge(2, 1);

        tracker.mark_dirty(page1);
        tracker.mark_dirty(page2);

        let dirty_pages = tracker.flush_and_reset();
        assert_eq!(dirty_pages.len(), 2);
        assert!(dirty_pages.contains(&page1));
        assert!(dirty_pages.contains(&page2));

        assert_eq!(tracker.get_dirty_page_count(), 0);
    }

    #[test]
    fn test_checkpoint_tracking() {
        let tracker = DirtyPageTracker::new(10, Duration::from_secs(60));
        let page_id = DirtyPageId::vertex(1, 0);

        assert!(!tracker.is_modified_since_checkpoint(&page_id));

        tracker.mark_modified_since_checkpoint(page_id);
        assert!(tracker.is_modified_since_checkpoint(&page_id));

        let pages = tracker.get_pages_for_full_write();
        assert_eq!(pages.len(), 1);

        tracker.clear_checkpoint_tracking();
        assert!(!tracker.is_modified_since_checkpoint(&page_id));
    }

    #[test]
    fn test_clear() {
        let tracker = DirtyPageTracker::new(10, Duration::from_secs(60));
        tracker.mark_dirty(DirtyPageId::vertex(1, 0));

        assert_eq!(tracker.get_dirty_page_count(), 1);
        tracker.clear();
        assert_eq!(tracker.get_dirty_page_count(), 0);
    }
}
