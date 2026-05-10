//! Overflow Page Handling
//!
//! Provides overflow page management for large records that exceed page size.
//! Follows SQLite's overflow page design where large records are split across
//! multiple pages linked together.

use super::{Page, PageType, PAGE_DATA_SIZE, PAGE_SIZE};
use crate::core::{StorageError, StorageResult};
use std::collections::HashMap;

pub const OVERFLOW_HEADER_SIZE: usize = 22;
pub const OVERFLOW_DATA_SIZE: usize = PAGE_DATA_SIZE - OVERFLOW_HEADER_SIZE;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct OverflowHeader {
    pub record_id: u64,
    pub sequence: u32,
    pub next_page_id: u64,
    pub data_size: u16,
}

impl OverflowHeader {
    pub fn new(record_id: u64, sequence: u32, next_page_id: u64, data_size: u16) -> Self {
        Self {
            record_id,
            sequence,
            next_page_id,
            data_size,
        }
    }

    pub fn to_bytes(&self) -> [u8; OVERFLOW_HEADER_SIZE] {
        let mut bytes = [0u8; OVERFLOW_HEADER_SIZE];
        bytes[0..8].copy_from_slice(&self.record_id.to_le_bytes());
        bytes[8..12].copy_from_slice(&self.sequence.to_le_bytes());
        bytes[12..20].copy_from_slice(&self.next_page_id.to_le_bytes());
        bytes[20..22].copy_from_slice(&self.data_size.to_le_bytes());
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < OVERFLOW_HEADER_SIZE {
            return None;
        }

        let record_id = u64::from_le_bytes(bytes[0..8].try_into().ok()?);
        let sequence = u32::from_le_bytes(bytes[8..12].try_into().ok()?);
        let next_page_id = u64::from_le_bytes(bytes[12..20].try_into().ok()?);
        let data_size = u16::from_le_bytes(bytes[20..22].try_into().ok()?);

        Some(Self {
            record_id,
            sequence,
            next_page_id,
            data_size,
        })
    }

    pub fn is_last(&self) -> bool {
        self.next_page_id == 0
    }
}

#[derive(Debug, Clone)]
pub struct OverflowPage {
    page: Page,
    header: OverflowHeader,
}

impl OverflowPage {
    pub fn new(page_id: u64, record_id: u64, sequence: u32) -> Self {
        let page = Page::new(page_id, PageType::Property);
        let header = OverflowHeader::new(record_id, sequence, 0, 0);

        Self { page, header }
    }

    pub fn from_page(page: Page) -> StorageResult<Self> {
        let header_bytes = page.read_record(0, OVERFLOW_HEADER_SIZE)
            .ok_or_else(|| StorageError::deserialize_error("Failed to read overflow header".to_string()))?;

        let header = OverflowHeader::from_bytes(header_bytes)
            .ok_or_else(|| StorageError::deserialize_error("Invalid overflow header".to_string()))?;

        Ok(Self { page, header })
    }

    pub fn write_data(&mut self, data: &[u8]) -> StorageResult<()> {
        if data.len() > OVERFLOW_DATA_SIZE {
            return Err(StorageError::invalid_operation(format!(
                "Data size {} exceeds overflow page capacity {}",
                data.len(),
                OVERFLOW_DATA_SIZE
            )));
        }

        self.header.data_size = data.len() as u16;
        let header_bytes = self.header.to_bytes();
        self.page.write_record(0, &header_bytes)?;
        self.page.write_record(OVERFLOW_HEADER_SIZE, data)?;

        self.page.header_mut().record_count = 1;

        Ok(())
    }

    pub fn read_data(&self) -> Option<&[u8]> {
        let data_size = self.header.data_size as usize;
        if data_size == 0 {
            return None;
        }

        self.page.read_record(OVERFLOW_HEADER_SIZE, data_size)
    }

    pub fn set_next_page(&mut self, page_id: u64) {
        self.header.next_page_id = page_id;
        let header_bytes = self.header.to_bytes();
        let _ = self.page.write_record(0, &header_bytes);
    }

    pub fn next_page_id(&self) -> u64 {
        self.header.next_page_id
    }

    pub fn record_id(&self) -> u64 {
        self.header.record_id
    }

    pub fn sequence(&self) -> u32 {
        self.header.sequence
    }

    pub fn data_size(&self) -> usize {
        self.header.data_size as usize
    }

    pub fn page_id(&self) -> u64 {
        self.page.page_id()
    }

    pub fn into_page(self) -> Page {
        self.page
    }

    pub fn page(&self) -> &Page {
        &self.page
    }

    pub fn page_mut(&mut self) -> &mut Page {
        &mut self.page
    }
}

pub struct OverflowManager {
    pages: HashMap<u64, Page>,
    free_pages: Vec<u64>,
    next_page_id: u64,
}

impl OverflowManager {
    pub fn new() -> Self {
        Self {
            pages: HashMap::new(),
            free_pages: Vec::new(),
            next_page_id: 1,
        }
    }

    pub fn allocate_page_id(&mut self) -> u64 {
        self.free_pages.pop().unwrap_or_else(|| {
            let id = self.next_page_id;
            self.next_page_id += 1;
            id
        })
    }

    pub fn free_page_id(&mut self, page_id: u64) {
        self.free_pages.push(page_id);
    }

    pub fn store(&mut self, record_id: u64, data: &[u8]) -> StorageResult<u64> {
        let chunks = self.split_data(data);
        let mut page_ids = Vec::new();

        for (sequence, chunk) in chunks.into_iter().enumerate() {
            let page_id = self.allocate_page_id();
            let mut overflow_page = OverflowPage::new(page_id, record_id, sequence as u32);
            overflow_page.write_data(&chunk)?;

            self.pages.insert(page_id, overflow_page.into_page());
            page_ids.push(page_id);
        }

        for i in 0..page_ids.len() - 1 {
            let current_id = page_ids[i];
            let next_id = page_ids[i + 1];

            if let Some(page) = self.pages.get_mut(&current_id) {
                let mut overflow_page = OverflowPage::from_page(page.clone())?;
                overflow_page.set_next_page(next_id);
                *page = overflow_page.into_page();
            }
        }

        page_ids.first().copied().ok_or_else(|| {
            StorageError::invalid_operation("Failed to create overflow pages".to_string())
        })
    }

    pub fn read(&self, first_page_id: u64) -> StorageResult<Vec<u8>> {
        let mut data = Vec::new();
        let mut current_page_id = first_page_id;

        loop {
            let page = self.pages.get(&current_page_id)
                .ok_or_else(|| StorageError::invalid_operation(format!(
                    "Overflow page {} not found",
                    current_page_id
                )))?;

            let overflow_page = OverflowPage::from_page(page.clone())?;

            if let Some(chunk) = overflow_page.read_data() {
                data.extend_from_slice(chunk);
            }

            if overflow_page.header.is_last() {
                break;
            }

            current_page_id = overflow_page.next_page_id();
        }

        Ok(data)
    }

    pub fn delete(&mut self, first_page_id: u64) -> StorageResult<()> {
        let mut page_ids = Vec::new();
        let mut current_page_id = first_page_id;

        loop {
            let page = self.pages.get(&current_page_id)
                .ok_or_else(|| StorageError::invalid_operation(format!(
                    "Overflow page {} not found",
                    current_page_id
                )))?;

            let overflow_page = OverflowPage::from_page(page.clone())?;
            page_ids.push(current_page_id);

            if overflow_page.header.is_last() {
                break;
            }

            current_page_id = overflow_page.next_page_id();
        }

        for page_id in page_ids {
            self.pages.remove(&page_id);
            self.free_page_id(page_id);
        }

        Ok(())
    }

    pub fn update(&mut self, first_page_id: u64, data: &[u8]) -> StorageResult<u64> {
        self.delete(first_page_id)?;
        self.store(first_page_id, data)
    }

    fn split_data(&self, data: &[u8]) -> Vec<Vec<u8>> {
        let mut chunks = Vec::new();
        let mut offset = 0;

        while offset < data.len() {
            let end = (offset + OVERFLOW_DATA_SIZE).min(data.len());
            chunks.push(data[offset..end].to_vec());
            offset = end;
        }

        if chunks.is_empty() {
            chunks.push(Vec::new());
        }

        chunks
    }

    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    pub fn free_page_count(&self) -> usize {
        self.free_pages.len()
    }

    pub fn memory_usage(&self) -> usize {
        self.pages.len() * PAGE_SIZE
    }

    pub fn get_page(&self, page_id: u64) -> Option<&Page> {
        self.pages.get(&page_id)
    }

    pub fn get_page_mut(&mut self, page_id: u64) -> Option<&mut Page> {
        self.pages.get_mut(&page_id)
    }

    pub fn get_overflow_page(&self, page_id: u64) -> StorageResult<OverflowPage> {
        let page = self.pages.get(&page_id)
            .ok_or_else(|| StorageError::invalid_operation(format!(
                "Page {} not found",
                page_id
            )))?;

        OverflowPage::from_page(page.clone())
    }
}

impl Default for OverflowManager {
    fn default() -> Self {
        Self::new()
    }
}

pub struct OverflowStats {
    pub total_pages: usize,
    pub free_pages: usize,
    pub memory_usage: usize,
    pub average_chain_length: f64,
}

impl OverflowManager {
    pub fn stats(&self) -> OverflowStats {
        let mut chain_lengths = Vec::new();
        let mut visited = std::collections::HashSet::new();

        for &page_id in self.pages.keys() {
            if visited.contains(&page_id) {
                continue;
            }

            let mut current_id = page_id;
            let mut chain_length = 0;

            loop {
                if visited.contains(&current_id) {
                    break;
                }

                visited.insert(current_id);
                chain_length += 1;

                if let Some(p) = self.pages.get(&current_id) {
                    if let Ok(overflow) = OverflowPage::from_page(p.clone()) {
                        if overflow.header.is_last() {
                            break;
                        }
                        current_id = overflow.next_page_id();
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }

            if chain_length > 0 {
                chain_lengths.push(chain_length);
            }
        }

        let average_chain_length = if chain_lengths.is_empty() {
            0.0
        } else {
            chain_lengths.iter().sum::<usize>() as f64 / chain_lengths.len() as f64
        };

        OverflowStats {
            total_pages: self.pages.len(),
            free_pages: self.free_pages.len(),
            memory_usage: self.memory_usage(),
            average_chain_length,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overflow_header() {
        let header = OverflowHeader::new(123, 1, 456, 100);
        let bytes = header.to_bytes();
        let decoded = OverflowHeader::from_bytes(&bytes).expect("Decode failed");

        assert_eq!(decoded.record_id, 123);
        assert_eq!(decoded.sequence, 1);
        assert_eq!(decoded.next_page_id, 456);
        assert_eq!(decoded.data_size, 100);
        assert!(!decoded.is_last());
    }

    #[test]
    fn test_overflow_header_last() {
        let header = OverflowHeader::new(123, 1, 0, 100);
        assert!(header.is_last());
    }

    #[test]
    fn test_overflow_page_creation() {
        let page = OverflowPage::new(1, 100, 0);
        assert_eq!(page.page_id(), 1);
        assert_eq!(page.record_id(), 100);
        assert_eq!(page.sequence(), 0);
    }

    #[test]
    fn test_overflow_page_write_read() {
        let mut page = OverflowPage::new(1, 100, 0);
        let data = b"test data for overflow page";

        page.write_data(data).expect("Write failed");

        let read_data = page.read_data().expect("Read failed");
        assert_eq!(read_data, data);
    }

    #[test]
    fn test_overflow_page_chain() {
        let mut page1 = OverflowPage::new(1, 100, 0);
        let mut page2 = OverflowPage::new(2, 100, 1);

        page1.write_data(b"data1").expect("Write failed");
        page2.write_data(b"data2").expect("Write failed");

        page1.set_next_page(2);

        assert_eq!(page1.next_page_id(), 2);
        assert!(page2.header.is_last());
    }

    #[test]
    fn test_overflow_manager_store_read() {
        let mut manager = OverflowManager::new();

        let large_data = vec![0xAB; 10_000];
        let first_page_id = manager.store(1, &large_data).expect("Store failed");

        let read_data = manager.read(first_page_id).expect("Read failed");
        assert_eq!(read_data, large_data);
    }

    #[test]
    fn test_overflow_manager_delete() {
        let mut manager = OverflowManager::new();

        let data = vec![0xCD; 8_000];
        let first_page_id = manager.store(1, &data).expect("Store failed");

        let initial_count = manager.page_count();
        assert!(initial_count > 0);

        manager.delete(first_page_id).expect("Delete failed");

        assert_eq!(manager.page_count(), 0);
        assert!(manager.free_page_count() > 0);
    }

    #[test]
    fn test_overflow_manager_update() {
        let mut manager = OverflowManager::new();

        let data1 = vec![0x11; 5_000];
        let first_page_id = manager.store(1, &data1).expect("Store failed");

        let data2 = vec![0x22; 12_000];
        let new_first_page_id = manager.update(first_page_id, &data2).expect("Update failed");

        let read_data = manager.read(new_first_page_id).expect("Read failed");
        assert_eq!(read_data, data2);
    }

    #[test]
    fn test_overflow_manager_small_data() {
        let mut manager = OverflowManager::new();

        let small_data = b"small data";
        let first_page_id = manager.store(1, small_data).expect("Store failed");

        let read_data = manager.read(first_page_id).expect("Read failed");
        assert_eq!(read_data, small_data);
        assert_eq!(manager.page_count(), 1);
    }

    #[test]
    fn test_overflow_manager_large_data() {
        let mut manager = OverflowManager::new();

        let large_data = vec![0xFF; 100_000];
        let first_page_id = manager.store(1, &large_data).expect("Store failed");

        let read_data = manager.read(first_page_id).expect("Read failed");
        assert_eq!(read_data, large_data);

        let expected_pages = large_data.len().div_ceil(OVERFLOW_DATA_SIZE);
        assert_eq!(manager.page_count(), expected_pages);
    }

    #[test]
    fn test_overflow_manager_stats() {
        let mut manager = OverflowManager::new();

        let data1 = vec![0x11; 5_000];
        let data2 = vec![0x22; 15_000];

        manager.store(1, &data1).expect("Store failed");
        manager.store(2, &data2).expect("Store failed");

        let stats = manager.stats();
        assert!(stats.total_pages > 0);
        assert!(stats.memory_usage > 0);
        assert!(stats.average_chain_length > 0.0);
    }

    #[test]
    fn test_overflow_page_size_limit() {
        let mut page = OverflowPage::new(1, 100, 0);

        let valid_data = vec![0u8; OVERFLOW_DATA_SIZE];
        assert!(page.write_data(&valid_data).is_ok());

        let invalid_data = vec![0u8; OVERFLOW_DATA_SIZE + 1];
        assert!(page.write_data(&invalid_data).is_err());
    }
}
