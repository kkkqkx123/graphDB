//! Fixed-Size Record Definitions
//!
//! Provides fixed-size record types for page-based storage.
//! These records are designed for efficient memory layout and cache locality.

use std::mem::size_of;

pub const VERTEX_RECORD_SIZE: usize = 16;
pub const EDGE_RECORD_SIZE: usize = 34;

pub type InternalId = u64;
pub type Timestamp = u32;

pub const INVALID_TIMESTAMP: Timestamp = u32::MAX;
pub const DELETED_TIMESTAMP: Timestamp = u32::MAX - 1;

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VertexRecord {
    pub internal_id: InternalId,
    pub timestamp: Timestamp,
    pub flags: u32,
}

impl VertexRecord {
    pub fn new(internal_id: InternalId, timestamp: Timestamp) -> Self {
        Self {
            internal_id,
            timestamp,
            flags: 0,
        }
    }

    pub fn to_bytes(&self) -> [u8; VERTEX_RECORD_SIZE] {
        let mut bytes = [0u8; VERTEX_RECORD_SIZE];
        bytes[0..8].copy_from_slice(&self.internal_id.to_le_bytes());
        bytes[8..12].copy_from_slice(&self.timestamp.to_le_bytes());
        bytes[12..16].copy_from_slice(&self.flags.to_le_bytes());
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < VERTEX_RECORD_SIZE {
            return None;
        }

        let internal_id = u64::from_le_bytes(bytes[0..8].try_into().ok()?);
        let timestamp = u32::from_le_bytes(bytes[8..12].try_into().ok()?);
        let flags = u32::from_le_bytes(bytes[12..16].try_into().ok()?);

        Some(Self {
            internal_id,
            timestamp,
            flags,
        })
    }

    pub fn is_valid(&self, ts: Timestamp) -> bool {
        self.timestamp != INVALID_TIMESTAMP
            && self.timestamp != DELETED_TIMESTAMP
            && self.timestamp <= ts
    }

    pub fn is_deleted(&self) -> bool {
        self.timestamp == INVALID_TIMESTAMP || self.timestamp == DELETED_TIMESTAMP
    }

    pub fn mark_deleted(&mut self) {
        self.timestamp = DELETED_TIMESTAMP;
    }

    pub fn restore(&mut self, ts: Timestamp) {
        self.timestamp = ts;
    }
}

impl Default for VertexRecord {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EdgeRecord {
    pub src_id: u64,
    pub dst_id: u64,
    pub edge_id: u64,
    pub prop_offset: u32,
    pub timestamp: Timestamp,
    pub flags: u16,
}

impl EdgeRecord {
    pub fn new(src_id: u64, dst_id: u64, edge_id: u64, prop_offset: u32, timestamp: Timestamp) -> Self {
        Self {
            src_id,
            dst_id,
            edge_id,
            prop_offset,
            timestamp,
            flags: 0,
        }
    }

    pub fn to_bytes(&self) -> [u8; EDGE_RECORD_SIZE] {
        let mut bytes = [0u8; EDGE_RECORD_SIZE];
        let mut offset = 0;

        bytes[offset..offset + 8].copy_from_slice(&self.src_id.to_le_bytes());
        offset += 8;

        bytes[offset..offset + 8].copy_from_slice(&self.dst_id.to_le_bytes());
        offset += 8;

        bytes[offset..offset + 8].copy_from_slice(&self.edge_id.to_le_bytes());
        offset += 8;

        bytes[offset..offset + 4].copy_from_slice(&self.prop_offset.to_le_bytes());
        offset += 4;

        bytes[offset..offset + 4].copy_from_slice(&self.timestamp.to_le_bytes());
        offset += 4;

        bytes[offset..offset + 2].copy_from_slice(&self.flags.to_le_bytes());

        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < EDGE_RECORD_SIZE {
            return None;
        }

        let mut offset = 0;

        let src_id = u64::from_le_bytes(bytes[offset..offset + 8].try_into().ok()?);
        offset += 8;

        let dst_id = u64::from_le_bytes(bytes[offset..offset + 8].try_into().ok()?);
        offset += 8;

        let edge_id = u64::from_le_bytes(bytes[offset..offset + 8].try_into().ok()?);
        offset += 8;

        let prop_offset = u32::from_le_bytes(bytes[offset..offset + 4].try_into().ok()?);
        offset += 4;

        let timestamp = u32::from_le_bytes(bytes[offset..offset + 4].try_into().ok()?);
        offset += 4;

        let flags = u16::from_le_bytes(bytes[offset..offset + 2].try_into().ok()?);

        Some(Self {
            src_id,
            dst_id,
            edge_id,
            prop_offset,
            timestamp,
            flags,
        })
    }

    pub fn is_valid(&self, ts: Timestamp) -> bool {
        self.timestamp != INVALID_TIMESTAMP
            && self.timestamp != DELETED_TIMESTAMP
            && self.timestamp <= ts
    }

    pub fn is_deleted(&self) -> bool {
        self.timestamp == INVALID_TIMESTAMP || self.timestamp == DELETED_TIMESTAMP
    }

    pub fn mark_deleted(&mut self) {
        self.timestamp = DELETED_TIMESTAMP;
    }

    pub fn restore(&mut self, ts: Timestamp) {
        self.timestamp = ts;
    }
}

impl Default for EdgeRecord {
    fn default() -> Self {
        Self::new(0, 0, 0, 0, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex_record_size() {
        assert_eq!(size_of::<VertexRecord>(), VERTEX_RECORD_SIZE);
    }

    #[test]
    fn test_edge_record_size() {
        assert_eq!(size_of::<EdgeRecord>(), EDGE_RECORD_SIZE);
    }

    #[test]
    fn test_vertex_record_serialization() {
        let record = VertexRecord::new(12345, 100);
        let bytes = record.to_bytes();
        let decoded = VertexRecord::from_bytes(&bytes).unwrap();

        let internal_id = decoded.internal_id;
        let timestamp = decoded.timestamp;
        assert_eq!(internal_id, 12345);
        assert_eq!(timestamp, 100);
    }

    #[test]
    fn test_edge_record_serialization() {
        let record = EdgeRecord::new(1, 2, 100, 50, 200);
        let bytes = record.to_bytes();
        let decoded = EdgeRecord::from_bytes(&bytes).unwrap();

        let src_id = decoded.src_id;
        let dst_id = decoded.dst_id;
        let edge_id = decoded.edge_id;
        let prop_offset = decoded.prop_offset;
        let timestamp = decoded.timestamp;
        assert_eq!(src_id, 1);
        assert_eq!(dst_id, 2);
        assert_eq!(edge_id, 100);
        assert_eq!(prop_offset, 50);
        assert_eq!(timestamp, 200);
    }

    #[test]
    fn test_vertex_record_validity() {
        let mut record = VertexRecord::new(1, 100);

        assert!(record.is_valid(150));
        assert!(!record.is_valid(50));

        record.mark_deleted();
        assert!(record.is_deleted());
        assert!(!record.is_valid(150));

        record.restore(200);
        assert!(!record.is_deleted());
        assert!(record.is_valid(250));
    }

    #[test]
    fn test_edge_record_validity() {
        let mut record = EdgeRecord::new(1, 2, 0, 0, 100);

        assert!(record.is_valid(150));
        assert!(!record.is_valid(50));

        record.mark_deleted();
        assert!(record.is_deleted());

        record.restore(200);
        assert!(!record.is_deleted());
        assert!(record.is_valid(250));
    }
}
