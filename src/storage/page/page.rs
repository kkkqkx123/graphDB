//! Page Implementation
//!
//! Fixed-size page with header and data area.

use super::{PageHeader, PageType, PAGE_SIZE, PAGE_DATA_SIZE};
use crate::core::StorageResult;

#[derive(Debug)]
pub struct Page {
    header: PageHeader,
    data: [u8; PAGE_DATA_SIZE],
}

impl Page {
    pub fn new(page_id: u64, page_type: PageType) -> Self {
        Self {
            header: PageHeader::new(page_id, page_type),
            data: [0u8; PAGE_DATA_SIZE],
        }
    }

    pub fn from_bytes(bytes: [u8; PAGE_SIZE]) -> StorageResult<Self> {
        let header = PageHeader::from_bytes(&bytes)
            .map_err(|e| crate::core::StorageError::DeserializeError(e))?;

        let mut data = [0u8; PAGE_DATA_SIZE];
        data.copy_from_slice(&bytes[PAGE_SIZE - PAGE_DATA_SIZE..]);

        if !header.verify_checksum(&data) {
            return Err(crate::core::StorageError::DbError(format!(
                "Checksum mismatch for page {}",
                header.page_id
            )));
        }

        Ok(Self { header, data })
    }

    pub fn to_bytes(&self) -> [u8; PAGE_SIZE] {
        let mut bytes = [0u8; PAGE_SIZE];
        let header_bytes = self.header.to_bytes();
        bytes[..header_bytes.len()].copy_from_slice(&header_bytes);
        bytes[PAGE_SIZE - PAGE_DATA_SIZE..].copy_from_slice(&self.data);
        bytes
    }

    pub fn header(&self) -> &PageHeader {
        &self.header
    }

    pub fn header_mut(&mut self) -> &mut PageHeader {
        &mut self.header
    }

    pub fn data(&self) -> &[u8; PAGE_DATA_SIZE] {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut [u8; PAGE_DATA_SIZE] {
        &mut self.data
    }

    pub fn page_id(&self) -> u64 {
        self.header.page_id
    }

    pub fn page_type(&self) -> PageType {
        self.header.page_type
    }

    pub fn record_count(&self) -> u16 {
        self.header.record_count
    }

    pub fn free_space(&self) -> u16 {
        self.header.free_space
    }

    pub fn write_record(&mut self, offset: usize, record_data: &[u8]) -> StorageResult<()> {
        if offset + record_data.len() > PAGE_DATA_SIZE {
            return Err(crate::core::StorageError::InvalidOperation(
                "Record exceeds page bounds".to_string(),
            ));
        }

        self.data[offset..offset + record_data.len()].copy_from_slice(record_data);
        self.header.update_checksum(&self.data);

        Ok(())
    }

    pub fn read_record(&self, offset: usize, len: usize) -> Option<&[u8]> {
        if offset + len > PAGE_DATA_SIZE {
            return None;
        }
        Some(&self.data[offset..offset + len])
    }

    pub fn clear(&mut self) {
        self.data.fill(0);
        self.header.record_count = 0;
        self.header.free_space = PAGE_DATA_SIZE as u16;
        self.header.free_space_offset = 0;
        self.header.update_checksum(&self.data);
    }

    pub fn update_checksum(&mut self) {
        self.header.update_checksum(&self.data);
    }

    pub fn verify_checksum(&self) -> bool {
        self.header.verify_checksum(&self.data)
    }
}

impl Clone for Page {
    fn clone(&self) -> Self {
        Self {
            header: self.header,
            data: self.data,
        }
    }
}

impl Default for Page {
    fn default() -> Self {
        Self::new(0, PageType::Free)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_new() {
        let page = Page::new(1, PageType::VertexData);
        assert_eq!(page.page_id(), 1);
        assert_eq!(page.page_type(), PageType::VertexData);
        assert_eq!(page.record_count(), 0);
    }

    #[test]
    fn test_page_write_read_record() {
        let mut page = Page::new(1, PageType::VertexData);
        let record = b"test record data";

        page.write_record(0, record).unwrap();

        let read = page.read_record(0, record.len()).unwrap();
        assert_eq!(read, record);
    }

    #[test]
    fn test_page_serialization() {
        let mut page = Page::new(42, PageType::EdgeData);
        page.write_record(0, b"some data").unwrap();

        let bytes = page.to_bytes();
        let decoded = Page::from_bytes(bytes).unwrap();

        assert_eq!(decoded.page_id(), 42);
        assert_eq!(decoded.page_type(), PageType::EdgeData);
    }

    #[test]
    fn test_checksum_verification() {
        let mut page = Page::new(1, PageType::VertexData);
        page.write_record(0, b"data").unwrap();

        assert!(page.verify_checksum());

        let mut corrupted = page.clone();
        corrupted.data[0] ^= 0xFF;

        assert!(!corrupted.verify_checksum());
    }
}
