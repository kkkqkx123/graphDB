use std::collections::HashSet;
use std::time::{Duration, Instant};

use parking_lot::RwLock;

use crate::core::StorageResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PageId {
    pub table_type: TableType,
    pub label_id: u16,
    pub block_number: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TableType {
    Vertex = 1,
    Edge = 2,
    Property = 3,
    Schema = 4,
}

pub struct DirtyPageTracker {
    dirty_pages: RwLock<HashSet<PageId>>,
    last_flush: RwLock<Instant>,
    flush_threshold: usize,
    flush_interval: Duration,
}

impl DirtyPageTracker {
    pub fn new(flush_threshold: usize, flush_interval: Duration) -> Self {
        Self {
            dirty_pages: RwLock::new(HashSet::new()),
            last_flush: RwLock::new(Instant::now()),
            flush_interval,
            flush_threshold,
        }
    }

    pub fn mark_dirty(&self, page_id: PageId) {
        self.dirty_pages.write().insert(page_id);
    }

    pub fn mark_dirty_batch(&self, page_ids: &[PageId]) {
        let mut dirty = self.dirty_pages.write();
        for page_id in page_ids {
            dirty.insert(*page_id);
        }
    }

    pub fn unmark_dirty(&self, page_id: &PageId) {
        self.dirty_pages.write().remove(page_id);
    }

    pub fn is_dirty(&self, page_id: &PageId) -> bool {
        self.dirty_pages.read().contains(page_id)
    }

    pub fn should_flush(&self) -> bool {
        let dirty = self.dirty_pages.read();
        let threshold_reached = dirty.len() >= self.flush_threshold;
        drop(dirty);

        let time_reached = self.last_flush.read().elapsed() >= self.flush_interval;

        threshold_reached || time_reached
    }

    pub fn get_dirty_pages(&self) -> Vec<PageId> {
        self.dirty_pages.write().drain().collect()
    }

    pub fn get_dirty_page_count(&self) -> usize {
        self.dirty_pages.read().len()
    }

    pub fn clear(&self) {
        self.dirty_pages.write().clear();
    }

    pub fn update_flush_time(&self) {
        *self.last_flush.write() = Instant::now();
    }

    pub fn flush_and_reset(&self) -> Vec<PageId> {
        let pages = self.get_dirty_pages();
        self.update_flush_time();
        pages
    }
}

impl Default for DirtyPageTracker {
    fn default() -> Self {
        Self::new(1000, Duration::from_secs(60))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mark_dirty() {
        let tracker = DirtyPageTracker::new(10, Duration::from_secs(60));
        let page_id = PageId {
            table_type: TableType::Vertex,
            label_id: 1,
            block_number: 0,
        };

        assert!(!tracker.is_dirty(&page_id));

        tracker.mark_dirty(page_id);
        assert!(tracker.is_dirty(&page_id));
        assert_eq!(tracker.get_dirty_page_count(), 1);
    }

    #[test]
    fn test_mark_dirty_batch() {
        let tracker = DirtyPageTracker::new(10, Duration::from_secs(60));
        let pages = vec![
            PageId {
                table_type: TableType::Vertex,
                label_id: 1,
                block_number: 0,
            },
            PageId {
                table_type: TableType::Vertex,
                label_id: 1,
                block_number: 1,
            },
            PageId {
                table_type: TableType::Edge,
                label_id: 2,
                block_number: 0,
            },
        ];

        tracker.mark_dirty_batch(&pages);
        assert_eq!(tracker.get_dirty_page_count(), 3);
    }

    #[test]
    fn test_unmark_dirty() {
        let tracker = DirtyPageTracker::new(10, Duration::from_secs(60));
        let page_id = PageId {
            table_type: TableType::Vertex,
            label_id: 1,
            block_number: 0,
        };

        tracker.mark_dirty(page_id);
        assert!(tracker.is_dirty(&page_id));

        tracker.unmark_dirty(&page_id);
        assert!(!tracker.is_dirty(&page_id));
    }

    #[test]
    fn test_should_flush_threshold() {
        let tracker = DirtyPageTracker::new(3, Duration::from_secs(60));

        assert!(!tracker.should_flush());

        tracker.mark_dirty(PageId {
            table_type: TableType::Vertex,
            label_id: 1,
            block_number: 0,
        });
        tracker.mark_dirty(PageId {
            table_type: TableType::Vertex,
            label_id: 1,
            block_number: 1,
        });
        assert!(!tracker.should_flush());

        tracker.mark_dirty(PageId {
            table_type: TableType::Vertex,
            label_id: 1,
            block_number: 2,
        });
        assert!(tracker.should_flush());
    }

    #[test]
    fn test_get_dirty_pages() {
        let tracker = DirtyPageTracker::new(10, Duration::from_secs(60));
        let page1 = PageId {
            table_type: TableType::Vertex,
            label_id: 1,
            block_number: 0,
        };
        let page2 = PageId {
            table_type: TableType::Edge,
            label_id: 2,
            block_number: 1,
        };

        tracker.mark_dirty(page1);
        tracker.mark_dirty(page2);

        let dirty_pages = tracker.get_dirty_pages();
        assert_eq!(dirty_pages.len(), 2);
        assert!(dirty_pages.contains(&page1));
        assert!(dirty_pages.contains(&page2));

        assert_eq!(tracker.get_dirty_page_count(), 0);
    }

    #[test]
    fn test_clear() {
        let tracker = DirtyPageTracker::new(10, Duration::from_secs(60));
        tracker.mark_dirty(PageId {
            table_type: TableType::Vertex,
            label_id: 1,
            block_number: 0,
        });

        assert_eq!(tracker.get_dirty_page_count(), 1);
        tracker.clear();
        assert_eq!(tracker.get_dirty_page_count(), 0);
    }
}
