//! Page Manager
//!
//! Manages page allocation, deallocation, and I/O operations.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use parking_lot::RwLock;

use super::{Page, PageHeader, PageType, PAGE_SIZE};
use crate::core::{StorageError, StorageResult};

const MAX_PAGES_IN_MEMORY: usize = 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StoragePageId {
    pub file_id: u32,
    pub page_number: u32,
}

impl StoragePageId {
    pub fn new(file_id: u32, page_number: u32) -> Self {
        Self { file_id, page_number }
    }

    pub fn to_u64(&self) -> u64 {
        ((self.file_id as u64) << 32) | (self.page_number as u64)
    }

    pub fn from_u64(value: u64) -> Self {
        Self {
            file_id: (value >> 32) as u32,
            page_number: value as u32,
        }
    }
}

#[derive(Debug)]
pub struct PageManager {
    pages: RwLock<HashMap<StoragePageId, Page>>,
    page_directory: RwLock<HashMap<u64, StoragePageId>>,
    base_path: PathBuf,
    next_page_id: RwLock<u64>,
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
        Self {
            pages: RwLock::new(HashMap::new()),
            page_directory: RwLock::new(HashMap::new()),
            base_path: base_path.as_ref().to_path_buf(),
            next_page_id: RwLock::new(1),
            stats: RwLock::new(PageManagerStats::default()),
        }
    }

    pub fn allocate_page(&self, page_type: PageType) -> StorageResult<StoragePageId> {
        let mut next_id = self.next_page_id.write();
        let page_number = *next_id;
        *next_id += 1;

        let page_id = StoragePageId::new(0, page_number as u32);
        let page = Page::new(page_id.to_u64(), page_type);

        self.pages.write().insert(page_id, page);
        self.stats.write().total_pages += 1;

        Ok(page_id)
    }

    pub fn get_page(&self, page_id: &StoragePageId) -> StorageResult<Option<Page>> {
        let pages = self.pages.read();

        if let Some(page) = pages.get(page_id) {
            self.stats.write().cache_hits += 1;
            return Ok(Some(page.clone()));
        }

        self.stats.write().cache_misses += 1;
        Ok(None)
    }

    pub fn get_page_mut(&self, page_id: &StoragePageId) -> StorageResult<Option<Page>> {
        let pages = self.pages.read();

        if let Some(page) = pages.get(page_id) {
            self.stats.write().cache_hits += 1;
            return Ok(Some(page.clone()));
        }

        self.stats.write().cache_misses += 1;
        Ok(None)
    }

    pub fn put_page(&self, page: Page) -> StorageResult<()> {
        let page_id = StoragePageId::from_u64(page.page_id());

        let mut pages = self.pages.write();
        if pages.len() >= MAX_PAGES_IN_MEMORY {
            self.evict_pages(&mut pages)?;
        }

        pages.insert(page_id, page);

        Ok(())
    }

    fn evict_pages(&self, pages: &mut HashMap<StoragePageId, Page>) -> StorageResult<()> {
        let pages_to_evict: Vec<StoragePageId> = pages
            .iter()
            .take(pages.len() / 4)
            .map(|(id, _)| *id)
            .collect();

        for page_id in pages_to_evict {
            if let Some(page) = pages.remove(&page_id) {
                self.flush_page_to_disk(&page)?;
            }
        }

        Ok(())
    }

    fn flush_page_to_disk(&self, page: &Page) -> StorageResult<()> {
        use std::fs::{self, File};
        use std::io::{Seek, SeekFrom, Write};

        let page_id = StoragePageId::from_u64(page.page_id());
        let file_path = self.get_page_path(&page_id);

        fs::create_dir_all(&self.base_path)?;

        let mut file = File::create(&file_path)?;
        file.write_all(&page.to_bytes())?;

        self.stats.write().pages_flushed += 1;

        Ok(())
    }

    fn load_page_from_disk(&self, page_id: &StoragePageId) -> StorageResult<Option<Page>> {
        use std::fs::File;
        use std::io::Read;

        let file_path = self.get_page_path(page_id);

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

    fn get_page_path(&self, page_id: &StoragePageId) -> PathBuf {
        self.base_path.join(format!("page_{:08}.bin", page_id.page_number))
    }

    pub fn delete_page(&self, page_id: &StoragePageId) -> StorageResult<bool> {
        let mut pages = self.pages.write();

        if pages.remove(page_id).is_some() {
            self.stats.write().total_pages -= 1;

            let file_path = self.get_page_path(page_id);
            if file_path.exists() {
                std::fs::remove_file(&file_path)?;
            }

            return Ok(true);
        }

        Ok(false)
    }

    pub fn flush_all(&self) -> StorageResult<()> {
        let pages = self.pages.read();

        for page in pages.values() {
            self.flush_page_to_disk(page)?;
        }

        Ok(())
    }

    pub fn stats(&self) -> PageManagerStats {
        self.stats.read().clone()
    }

    pub fn page_count(&self) -> usize {
        self.pages.read().len()
    }

    pub fn clear(&self) {
        let mut pages = self.pages.write();
        pages.clear();

        let mut stats = self.stats.write();
        stats.total_pages = 0;
        stats.pages_loaded = 0;
        stats.pages_flushed = 0;
        stats.cache_hits = 0;
        stats.cache_misses = 0;
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

        let page_id = manager.allocate_page(PageType::VertexData).unwrap();
        assert!(manager.get_page(&page_id).unwrap().is_some());
    }

    #[test]
    fn test_page_id_conversion() {
        let page_id = StoragePageId::new(1, 42);
        let value = page_id.to_u64();
        let decoded = StoragePageId::from_u64(value);

        assert_eq!(decoded.file_id, 1);
        assert_eq!(decoded.page_number, 42);
    }

    #[test]
    fn test_put_and_get_page() {
        let dir = tempdir().unwrap();
        let manager = PageManager::new(dir.path());

        let page_id = manager.allocate_page(PageType::EdgeData).unwrap();
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

        manager.allocate_page(PageType::VertexData).unwrap();
        manager.allocate_page(PageType::EdgeData).unwrap();

        let stats = manager.stats();
        assert_eq!(stats.total_pages, 2);
    }
}
