//! Page Manager
//!
//! Manages page allocation, deallocation, and I/O operations.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use moka::sync::Cache;
use parking_lot::RwLock;

use super::{Page, PageType, PAGE_SIZE};
use crate::core::StorageResult;
use crate::storage::persistence::{DirtyPageId, TableType};

const MAX_PAGES_IN_MEMORY: u64 = 1024;

/// Page ID type alias for compatibility.
/// Uses DirtyPageId as the unified page identifier.
pub type PageId = DirtyPageId;

#[derive(Debug, Clone)]
pub struct PageManagerConfig {
    pub max_pages: u64,
    pub base_path: PathBuf,
}

impl Default for PageManagerConfig {
    fn default() -> Self {
        Self {
            max_pages: MAX_PAGES_IN_MEMORY,
            base_path: PathBuf::from("./data/pages"),
        }
    }
}

#[derive(Debug)]
pub struct PageManager {
    pages: Cache<PageId, Page>,
    base_path: PathBuf,
    next_block_number: AtomicU64,
    stats: RwLock<PageManagerStats>,
}

#[derive(Debug, Default, Clone)]
pub struct PageManagerStats {
    pub total_pages: usize,
    pub pages_loaded: u64,
    pub pages_flushed: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

impl PageManager {
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        Self::with_config(PageManagerConfig {
            base_path: base_path.as_ref().to_path_buf(),
            ..Default::default()
        })
    }

    pub fn with_config(config: PageManagerConfig) -> Self {
        let base_path_for_listener = config.base_path.clone();
        let pages = Cache::builder()
            .max_capacity(config.max_pages)
            .weigher(|_key: &PageId, _value: &Page| 1u32)
            .eviction_listener(move |page_id, page, _cause| {
                if page.is_dirty() {
                    if let Err(e) = Self::flush_page_to_disk_static(&base_path_for_listener, &page_id, &page) {
                        eprintln!("Failed to flush page during eviction: {:?}", e);
                    }
                }
            })
            .build();

        Self {
            pages,
            base_path: config.base_path.clone(),
            next_block_number: AtomicU64::new(0),
            stats: RwLock::new(PageManagerStats::default()),
        }
    }

    pub fn allocate_page(&self, table_type: TableType, label_id: u16) -> StorageResult<PageId> {
        let block_number = self.next_block_number.fetch_add(1, Ordering::SeqCst);
        let page_id = PageId::new(table_type, label_id, block_number);
        let page_type = Self::table_type_to_page_type(table_type);
        let page = Page::new(page_id.to_u64(), page_type);

        self.pages.insert(page_id, page);
        self.stats.write().total_pages += 1;

        Ok(page_id)
    }

    fn table_type_to_page_type(table_type: TableType) -> PageType {
        match table_type {
            TableType::Vertex => PageType::VertexData,
            TableType::Edge => PageType::EdgeData,
            TableType::Property => PageType::Property,
            TableType::Schema => PageType::Schema,
        }
    }

    pub fn get_page(&self, page_id: &PageId) -> StorageResult<Option<Page>> {
        if let Some(page) = self.pages.get(page_id) {
            self.stats.write().cache_hits += 1;
            return Ok(Some(page));
        }

        self.stats.write().cache_misses += 1;

        if let Some(page) = self.load_page_from_disk(page_id)? {
            self.pages.insert(*page_id, page.clone());
            return Ok(Some(page));
        }

        Ok(None)
    }

    pub fn get_page_mut(&self, page_id: &PageId) -> StorageResult<Option<Page>> {
        self.get_page(page_id)
    }

    pub fn put_page(&self, page: Page) -> StorageResult<()> {
        let page_id = PageId::from_u64(page.page_id());
        self.pages.insert(page_id, page);
        Ok(())
    }

    pub fn mark_dirty(&self, page_id: &PageId) -> StorageResult<()> {
        if let Some(mut page) = self.pages.get(page_id) {
            page.mark_dirty();
            self.pages.insert(*page_id, page);
        }
        Ok(())
    }

    pub fn clear_dirty(&self, page_id: &PageId) -> StorageResult<()> {
        if let Some(mut page) = self.pages.get(page_id) {
            page.clear_dirty();
            self.pages.insert(*page_id, page);
        }
        Ok(())
    }

    fn flush_page_to_disk_static(
        base_path: &Path,
        page_id: &PageId,
        page: &Page,
    ) -> StorageResult<()> {
        use std::fs::{self, File};
        use std::io::Write;

        let file_path = page_id.file_path(base_path);

        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file = File::create(&file_path)?;
        file.write_all(&page.to_bytes())?;

        Ok(())
    }

    pub fn flush_page(&self, page_id: &PageId) -> StorageResult<()> {
        if let Some(page) = self.pages.get(page_id) {
            if page.is_dirty() {
                Self::flush_page_to_disk_static(&self.base_path, page_id, &page)?;
                self.stats.write().pages_flushed += 1;
            }
        }
        Ok(())
    }

    fn load_page_from_disk(&self, page_id: &PageId) -> StorageResult<Option<Page>> {
        use std::fs::File;
        use std::io::Read;

        let file_path = page_id.file_path(&self.base_path);

        if !file_path.exists() {
            return Ok(None);
        }

        let mut file = File::open(&file_path)?;
        let mut buffer = [0u8; PAGE_SIZE];
        file.read_exact(&mut buffer)?;

        let page = Page::from_bytes(buffer)?;
        self.stats.write().pages_loaded += 1;

        Ok(Some(page))
    }

    pub fn delete_page(&self, page_id: &PageId) -> StorageResult<bool> {
        if self.pages.remove(page_id).is_some() {
            self.stats.write().total_pages -= 1;

            let file_path = page_id.file_path(&self.base_path);
            if file_path.exists() {
                std::fs::remove_file(&file_path)?;
            }

            return Ok(true);
        }

        Ok(false)
    }

    pub fn flush_all(&self) -> StorageResult<()> {
        for entry in self.pages.iter() {
            let page_id = entry.0;
            let page = &entry.1;
            if page.is_dirty() {
                Self::flush_page_to_disk_static(&self.base_path, &page_id, page)?;
                self.stats.write().pages_flushed += 1;
            }
        }

        Ok(())
    }

    pub fn stats(&self) -> PageManagerStats {
        let mut stats = self.stats.read().clone();
        stats.total_pages = self.pages.entry_count() as usize;
        stats
    }

    pub fn page_count(&self) -> usize {
        self.pages.entry_count() as usize
    }

    pub fn clear(&self) {
        self.pages.invalidate_all();

        let mut stats = self.stats.write();
        stats.total_pages = 0;
        stats.pages_loaded = 0;
        stats.pages_flushed = 0;
        stats.cache_hits = 0;
        stats.cache_misses = 0;
    }

    pub fn get_dirty_pages(&self) -> Vec<PageId> {
        self.pages
            .iter()
            .filter(|(_, page)| page.is_dirty())
            .map(|(page_id, _)| *page_id)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_allocate_page() {
        let dir = tempdir().unwrap();
        let manager = PageManager::new(dir.path());

        let page_id = manager.allocate_page(TableType::Vertex, 1).unwrap();
        assert!(manager.get_page(&page_id).unwrap().is_some());
    }

    #[test]
    fn test_page_id_conversion() {
        let page_id = PageId::new(TableType::Vertex, 42, 100);
        let value = page_id.to_u64();
        let decoded = PageId::from_u64(value);

        assert_eq!(decoded.table_type, TableType::Vertex);
        assert_eq!(decoded.label_id, 42);
        assert_eq!(decoded.block_number, 100);
    }

    #[test]
    fn test_put_and_get_page() {
        let dir = tempdir().unwrap();
        let manager = PageManager::new(dir.path());

        let page_id = manager.allocate_page(TableType::Edge, 1).unwrap();
        let mut page = manager.get_page(&page_id).unwrap().unwrap();

        page.write_record(0, b"test data").unwrap();
        manager.put_page(page).unwrap();

        let retrieved = manager.get_page(&page_id).unwrap().unwrap();
        let data = retrieved.read_record(0, 9).unwrap();
        assert_eq!(data, b"test data");
    }

    #[test]
    fn test_stats() {
        let dir = tempdir().unwrap();
        let manager = PageManager::new(dir.path());

        manager.allocate_page(TableType::Vertex, 1).unwrap();
        manager.allocate_page(TableType::Edge, 2).unwrap();

        manager.pages.run_pending_tasks();

        let stats = manager.stats();
        assert_eq!(stats.total_pages, 2);
    }

    #[test]
    fn test_file_path_generation() {
        let page_id = PageId::new(TableType::Vertex, 5, 123);
        let path = page_id.file_path(Path::new("/data"));

        assert!(path.ends_with("block_00000123.page"));
        assert!(path.to_str().unwrap().contains("vertex"));
        assert!(path.to_str().unwrap().contains("label_5"));
    }
}
