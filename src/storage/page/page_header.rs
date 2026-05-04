//! Page Header Definition
//!
//! Contains page metadata including type, checksum, and record information.

use std::mem::size_of;

pub const PAGE_SIZE: usize = 4096;
pub const PAGE_HEADER_SIZE: usize = size_of::<PageHeader>();
pub const PAGE_DATA_SIZE: usize = PAGE_SIZE - PAGE_HEADER_SIZE;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PageType {
    VertexHeader = 1,
    VertexData = 2,
    EdgeHeader = 3,
    EdgeData = 4,
    Property = 5,
    Schema = 6,
    Free = 255,
}

impl TryFrom<u8> for PageType {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(PageType::VertexHeader),
            2 => Ok(PageType::VertexData),
            3 => Ok(PageType::EdgeHeader),
            4 => Ok(PageType::EdgeData),
            5 => Ok(PageType::Property),
            6 => Ok(PageType::Schema),
            255 => Ok(PageType::Free),
            _ => Err(format!("Invalid page type: {}", value)),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PageHeader {
    pub page_id: u64,
    pub page_type: PageType,
    pub flags: u8,
    pub checksum: u32,
    pub record_count: u16,
    pub free_space_offset: u16,
    pub free_space: u16,
    pub reserved: [u8; 8],
}

impl PageHeader {
    pub fn new(page_id: u64, page_type: PageType) -> Self {
        Self {
            page_id,
            page_type,
            flags: 0,
            checksum: 0,
            record_count: 0,
            free_space_offset: PAGE_HEADER_SIZE as u16,
            free_space: PAGE_DATA_SIZE as u16,
            reserved: [0; 8],
        }
    }

    pub fn to_bytes(&self) -> [u8; PAGE_HEADER_SIZE] {
        let mut bytes = [0u8; PAGE_HEADER_SIZE];
        let mut offset = 0;

        bytes[offset..offset + 8].copy_from_slice(&self.page_id.to_le_bytes());
        offset += 8;

        bytes[offset] = self.page_type as u8;
        offset += 1;

        bytes[offset] = self.flags;
        offset += 1;

        bytes[offset..offset + 4].copy_from_slice(&self.checksum.to_le_bytes());
        offset += 4;

        bytes[offset..offset + 2].copy_from_slice(&self.record_count.to_le_bytes());
        offset += 2;

        bytes[offset..offset + 2].copy_from_slice(&self.free_space_offset.to_le_bytes());
        offset += 2;

        bytes[offset..offset + 2].copy_from_slice(&self.free_space.to_le_bytes());
        offset += 2;

        bytes[offset..offset + 8].copy_from_slice(&self.reserved);

        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < PAGE_HEADER_SIZE {
            return Err("Insufficient bytes for page header".to_string());
        }

        let mut offset = 0;

        let page_id = u64::from_le_bytes(bytes[offset..offset + 8].try_into().map_err(|_| "Invalid page_id")?);
        offset += 8;

        let page_type = PageType::try_from(bytes[offset])?;
        offset += 1;

        let flags = bytes[offset];
        offset += 1;

        let checksum = u32::from_le_bytes(bytes[offset..offset + 4].try_into().map_err(|_| "Invalid checksum")?);
        offset += 4;

        let record_count = u16::from_le_bytes(bytes[offset..offset + 2].try_into().map_err(|_| "Invalid record_count")?);
        offset += 2;

        let free_space_offset = u16::from_le_bytes(bytes[offset..offset + 2].try_into().map_err(|_| "Invalid free_space_offset")?);
        offset += 2;

        let free_space = u16::from_le_bytes(bytes[offset..offset + 2].try_into().map_err(|_| "Invalid free_space")?);
        offset += 2;

        let mut reserved = [0u8; 8];
        reserved.copy_from_slice(&bytes[offset..offset + 8]);

        Ok(Self {
            page_id,
            page_type,
            flags,
            checksum,
            record_count,
            free_space_offset,
            free_space,
            reserved,
        })
    }

    pub fn compute_checksum(data: &[u8]) -> u32 {
        let mut hash: u32 = 0x811c9dc5;
        for &byte in data {
            hash ^= byte as u32;
            hash = hash.wrapping_mul(0x01000193);
        }
        hash
    }

    pub fn update_checksum(&mut self, data: &[u8]) {
        self.checksum = Self::compute_checksum(data);
    }

    pub fn verify_checksum(&self, data: &[u8]) -> bool {
        self.checksum == Self::compute_checksum(data)
    }

    pub fn can_fit_record(&self, record_size: usize) -> bool {
        self.free_space as usize >= record_size
    }

    pub fn allocate_space(&mut self, size: usize) -> Option<u16> {
        if size > self.free_space as usize {
            return None;
        }

        let offset = self.free_space_offset;
        self.free_space_offset += size as u16;
        self.free_space -= size as u16;
        self.record_count += 1;

        Some(offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_header_new() {
        let header = PageHeader::new(1, PageType::VertexData);
        assert_eq!(header.page_id, 1);
        assert_eq!(header.page_type, PageType::VertexData);
        assert_eq!(header.record_count, 0);
        assert_eq!(header.free_space, PAGE_DATA_SIZE as u16);
    }

    #[test]
    fn test_page_header_serialization() {
        let header = PageHeader::new(42, PageType::EdgeData);
        let bytes = header.to_bytes();
        let decoded = PageHeader::from_bytes(&bytes).unwrap();

        assert_eq!(decoded.page_id, 42);
        assert_eq!(decoded.page_type, PageType::EdgeData);
    }

    #[test]
    fn test_checksum() {
        let data = b"test data for checksum";
        let checksum1 = PageHeader::compute_checksum(data);
        let checksum2 = PageHeader::compute_checksum(data);
        assert_eq!(checksum1, checksum2);

        let different_data = b"different data";
        let checksum3 = PageHeader::compute_checksum(different_data);
        assert_ne!(checksum1, checksum3);
    }

    #[test]
    fn test_allocate_space() {
        let mut header = PageHeader::new(1, PageType::VertexData);
        let record_size = 32;

        let offset = header.allocate_space(record_size);
        assert!(offset.is_some());
        assert_eq!(header.record_count, 1);
        assert_eq!(header.free_space, PAGE_DATA_SIZE as u16 - record_size as u16);
    }

    #[test]
    fn test_allocate_space_insufficient() {
        let mut header = PageHeader::new(1, PageType::VertexData);
        let huge_size = PAGE_DATA_SIZE + 100;

        let offset = header.allocate_space(huge_size);
        assert!(offset.is_none());
        assert_eq!(header.record_count, 0);
    }
}
