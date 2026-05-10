//! CSR Persistence Module
//!
//! Provides persistence support for CSR data structures using memory-mapped files.

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::core::{StorageError, StorageResult};

const MAGIC_NUMBER: u32 = 0x43535231; // "CSR1"
const HEADER_SIZE: usize = 32;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct CsrFileHeader {
    magic: u32,
    version: u32,
    vertex_capacity: u64,
    edge_count: u64,
    total_edge_capacity: u64,
    offsets_offset: u64,
    degrees_offset: u64,
    capacities_offset: u64,
    nbr_list_offset: u64,
}

impl CsrFileHeader {
    fn new(vertex_capacity: u64, edge_count: u64, total_edge_capacity: u64) -> Self {
        Self {
            magic: MAGIC_NUMBER,
            version: 1,
            vertex_capacity,
            edge_count,
            total_edge_capacity,
            offsets_offset: HEADER_SIZE as u64,
            degrees_offset: HEADER_SIZE as u64 + vertex_capacity * 8,
            capacities_offset: HEADER_SIZE as u64 + vertex_capacity * 8 + vertex_capacity * 4,
            nbr_list_offset: HEADER_SIZE as u64
                + vertex_capacity * 8
                + vertex_capacity * 4
                + vertex_capacity * 4,
        }
    }

    fn as_bytes(&self) -> [u8; HEADER_SIZE] {
        let mut bytes = [0u8; HEADER_SIZE];
        bytes[0..4].copy_from_slice(&self.magic.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.version.to_le_bytes());
        bytes[8..16].copy_from_slice(&self.vertex_capacity.to_le_bytes());
        bytes[16..24].copy_from_slice(&self.edge_count.to_le_bytes());
        bytes[24..32].copy_from_slice(&self.total_edge_capacity.to_le_bytes());
        bytes
    }

    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < HEADER_SIZE {
            return None;
        }

        let magic = u32::from_le_bytes(bytes[0..4].try_into().ok()?);
        if magic != MAGIC_NUMBER {
            return None;
        }

        Some(Self {
            magic,
            version: u32::from_le_bytes(bytes[4..8].try_into().ok()?),
            vertex_capacity: u64::from_le_bytes(bytes[8..16].try_into().ok()?),
            edge_count: u64::from_le_bytes(bytes[16..24].try_into().ok()?),
            total_edge_capacity: u64::from_le_bytes(bytes[24..32].try_into().ok()?),
            offsets_offset: HEADER_SIZE as u64,
            degrees_offset: 0,
            capacities_offset: 0,
            nbr_list_offset: 0,
        })
    }
}

/// CSR file-based persistence
pub struct CsrPersistence {
    is_open: AtomicBool,
}

impl CsrPersistence {
    pub fn new() -> Self {
        Self {
            is_open: AtomicBool::new(false),
        }
    }

    /// Save MutableCsr to file
    pub fn save_mutable_csr<P: AsRef<Path>>(
        &self,
        csr: &super::MutableCsr,
        path: P,
    ) -> StorageResult<()> {
        use std::fs::File;
        use std::io::{BufWriter, Write};

        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| StorageError::io_error(e.to_string()))?;
        }

        let file = File::create(path).map_err(|e| StorageError::io_error(e.to_string()))?;
        let mut writer = BufWriter::new(file);

        let vertex_capacity = csr.vertex_capacity() as u64;
        let edge_count = csr.edge_count();
        let total_edge_capacity = csr.adj_offsets().len() as u64;

        let header = CsrFileHeader::new(vertex_capacity, edge_count, total_edge_capacity);
        writer
            .write_all(&header.as_bytes())
            .map_err(|e| StorageError::io_error(e.to_string()))?;

        for &offset in csr.adj_offsets() {
            writer
                .write_all(&(offset as u64).to_le_bytes())
                .map_err(|e| StorageError::io_error(e.to_string()))?;
        }

        for &degree in csr.degrees() {
            writer
                .write_all(&degree.to_le_bytes())
                .map_err(|e| StorageError::io_error(e.to_string()))?;
        }

        for &capacity in csr.capacities() {
            writer
                .write_all(&capacity.to_le_bytes())
                .map_err(|e| StorageError::io_error(e.to_string()))?;
        }

        for nbr in csr.nbr_slice() {
            writer
                .write_all(&nbr.neighbor.to_le_bytes())
                .map_err(|e| StorageError::io_error(e.to_string()))?;
            writer
                .write_all(&nbr.edge_id.to_le_bytes())
                .map_err(|e| StorageError::io_error(e.to_string()))?;
            writer
                .write_all(&nbr.prop_offset.to_le_bytes())
                .map_err(|e| StorageError::io_error(e.to_string()))?;
            writer
                .write_all(&nbr.timestamp.to_le_bytes())
                .map_err(|e| StorageError::io_error(e.to_string()))?;
        }

        writer
            .flush()
            .map_err(|e| StorageError::io_error(e.to_string()))?;

        Ok(())
    }

    /// Load MutableCsr from file
    pub fn load_mutable_csr<P: AsRef<Path>>(&self, path: P) -> StorageResult<super::MutableCsr> {
        use std::fs::File;
        use std::io::{BufReader, Read};

        let path = path.as_ref();
        let file = File::open(path).map_err(|e| StorageError::io_error(e.to_string()))?;
        let mut reader = BufReader::new(file);

        let mut header_bytes = [0u8; HEADER_SIZE];
        reader
            .read_exact(&mut header_bytes)
            .map_err(|e| StorageError::io_error(e.to_string()))?;

        let header = CsrFileHeader::from_bytes(&header_bytes)
            .ok_or_else(|| StorageError::deserialize_error("Invalid CSR file header".to_string()))?;

        let vertex_capacity = header.vertex_capacity as usize;
        let edge_count = header.edge_count;
        let total_edge_capacity = header.total_edge_capacity as usize;

        let mut adj_offsets = Vec::with_capacity(vertex_capacity);
        for _ in 0..vertex_capacity {
            let mut bytes = [0u8; 8];
            reader
                .read_exact(&mut bytes)
                .map_err(|e| StorageError::io_error(e.to_string()))?;
            adj_offsets.push(u64::from_le_bytes(bytes) as usize);
        }

        let mut degrees = Vec::with_capacity(vertex_capacity);
        for _ in 0..vertex_capacity {
            let mut bytes = [0u8; 4];
            reader
                .read_exact(&mut bytes)
                .map_err(|e| StorageError::io_error(e.to_string()))?;
            degrees.push(u32::from_le_bytes(bytes));
        }

        let mut capacities = Vec::with_capacity(vertex_capacity);
        for _ in 0..vertex_capacity {
            let mut bytes = [0u8; 4];
            reader
                .read_exact(&mut bytes)
                .map_err(|e| StorageError::io_error(e.to_string()))?;
            capacities.push(u32::from_le_bytes(bytes));
        }

        let mut nbr_list = Vec::with_capacity(total_edge_capacity);
        for _ in 0..total_edge_capacity {
            let mut neighbor_bytes = [0u8; 8];
            let mut edge_id_bytes = [0u8; 8];
            let mut prop_offset_bytes = [0u8; 4];
            let mut timestamp_bytes = [0u8; 4];

            reader
                .read_exact(&mut neighbor_bytes)
                .map_err(|e| StorageError::io_error(e.to_string()))?;
            reader
                .read_exact(&mut edge_id_bytes)
                .map_err(|e| StorageError::io_error(e.to_string()))?;
            reader
                .read_exact(&mut prop_offset_bytes)
                .map_err(|e| StorageError::io_error(e.to_string()))?;
            reader
                .read_exact(&mut timestamp_bytes)
                .map_err(|e| StorageError::io_error(e.to_string()))?;

            nbr_list.push(super::Nbr {
                neighbor: u64::from_le_bytes(neighbor_bytes),
                edge_id: u64::from_le_bytes(edge_id_bytes),
                prop_offset: u32::from_le_bytes(prop_offset_bytes),
                timestamp: u32::from_le_bytes(timestamp_bytes),
            });
        }

        let mut csr = super::MutableCsr::new();
        csr.load_from_parts(
            nbr_list,
            adj_offsets,
            degrees,
            capacities,
            vertex_capacity,
            total_edge_capacity,
            edge_count,
        );

        Ok(csr)
    }

    /// Save immutable Csr to file
    pub fn save_csr<P: AsRef<Path>>(&self, csr: &super::Csr, path: P) -> StorageResult<()> {
        use std::fs::File;
        use std::io::{BufWriter, Write};

        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| StorageError::io_error(e.to_string()))?;
        }

        let file = File::create(path).map_err(|e| StorageError::io_error(e.to_string()))?;
        let mut writer = BufWriter::new(file);

        let data = csr.dump();
        writer
            .write_all(&data)
            .map_err(|e| StorageError::io_error(e.to_string()))?;

        writer
            .flush()
            .map_err(|e| StorageError::io_error(e.to_string()))?;

        Ok(())
    }

    /// Load immutable Csr from file
    pub fn load_csr<P: AsRef<Path>>(&self, path: P) -> StorageResult<super::Csr> {
        use std::fs::File;
        use std::io::Read;

        let path = path.as_ref();
        let mut file = File::open(path).map_err(|e| StorageError::io_error(e.to_string()))?;

        let mut data = Vec::new();
        file.read_to_end(&mut data)
            .map_err(|e| StorageError::io_error(e.to_string()))?;

        let mut csr = super::Csr::new();
        csr.load(&data);

        Ok(csr)
    }

    pub fn is_open(&self) -> bool {
        self.is_open.load(Ordering::SeqCst)
    }
}

impl Default for CsrPersistence {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_save_and_load_mutable_csr() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path().join("test_csr.bin");

        let mut csr = super::super::MutableCsr::with_capacity(10, 100);
        csr.insert_edge(0, 1, 100, 0, 1);
        csr.insert_edge(0, 2, 101, 0, 1);
        csr.insert_edge(1, 3, 102, 0, 1);

        let persistence = CsrPersistence::new();
        persistence
            .save_mutable_csr(&csr, &path)
            .expect("Failed to save");

        let loaded_csr = persistence.load_mutable_csr(&path).expect("Failed to load");

        assert_eq!(loaded_csr.vertex_capacity(), csr.vertex_capacity());
        assert_eq!(loaded_csr.edge_count(), csr.edge_count());
        assert!(loaded_csr.has_edge(0, 1, 1));
        assert!(loaded_csr.has_edge(0, 2, 1));
        assert!(loaded_csr.has_edge(1, 3, 1));
    }

    #[test]
    fn test_save_and_load_csr() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path().join("test_csr_immutable.bin");

        let mut csr = super::super::Csr::with_capacity(10, 100);
        csr.batch_put_edges(&[0, 0, 1], &[1, 2, 3], &[100, 101, 102], &[0, 0, 0], 1);

        let persistence = CsrPersistence::new();
        persistence.save_csr(&csr, &path).expect("Failed to save");

        let loaded_csr = persistence.load_csr(&path).expect("Failed to load");

        assert_eq!(loaded_csr.vertex_capacity(), csr.vertex_capacity());
        assert_eq!(loaded_csr.edge_count(), csr.edge_count());
        assert!(loaded_csr.has_edge(0, 1));
        assert!(loaded_csr.has_edge(0, 2));
        assert!(loaded_csr.has_edge(1, 3));
    }
}
