//! Page Writer Implementation
//!
//! Provides concrete implementation of the PageWriter trait for persisting
//! dirty pages to disk with compression and checksum support.
//!
//! # Features
//!
//! - Page serialization and deserialization
//! - Compression support
//! - Checksum validation
//! - Atomic page writes with temporary files
//! - Page recovery on corruption

use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use parking_lot::RwLock;

use super::dirty_tracker::{DirtyPageTracker, PageId, TableType};
use super::flush_manager::PageWriter;
use super::compression::{CompressionType, Compressor};
use crate::core::{StorageError, StorageResult};

pub const PAGE_FILE_MAGIC: u32 = 0x50414745;
pub const PAGE_FILE_VERSION: u32 = 1;

#[derive(Debug, Clone)]
pub struct PageHeader {
    pub magic: u32,
    pub version: u32,
    pub page_id: PageId,
    pub data_size: u32,
    pub compressed_size: u32,
    pub checksum: u64,
    pub timestamp: u64,
}

impl PageHeader {
    pub fn new(page_id: PageId, data_size: u32, compressed_size: u32, timestamp: u64) -> Self {
        Self {
            magic: PAGE_FILE_MAGIC,
            version: PAGE_FILE_VERSION,
            page_id,
            data_size,
            compressed_size,
            checksum: 0,
            timestamp,
        }
    }

    pub fn compute_checksum(&self) -> u64 {
        let mut hash: u64 = 0;
        hash = hash.wrapping_mul(31).wrapping_add(self.magic as u64);
        hash = hash.wrapping_mul(31).wrapping_add(self.version as u64);
        hash = hash.wrapping_mul(31).wrapping_add(self.page_id.table_type as u64);
        hash = hash.wrapping_mul(31).wrapping_add(self.page_id.label_id as u64);
        hash = hash.wrapping_mul(31).wrapping_add(self.page_id.block_number);
        hash = hash.wrapping_mul(31).wrapping_add(self.data_size as u64);
        hash = hash.wrapping_mul(31).wrapping_add(self.compressed_size as u64);
        hash = hash.wrapping_mul(31).wrapping_add(self.timestamp);
        hash
    }

    pub fn update_checksum(&mut self) {
        self.checksum = self.compute_checksum();
    }

    pub fn verify_checksum(&self) -> bool {
        self.checksum == self.compute_checksum()
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(48);
        data.extend_from_slice(&self.magic.to_le_bytes());
        data.extend_from_slice(&self.version.to_le_bytes());
        data.extend_from_slice(&(self.page_id.table_type as u8).to_le_bytes());
        data.extend_from_slice(&self.page_id.label_id.to_le_bytes());
        data.extend_from_slice(&self.page_id.block_number.to_le_bytes());
        data.extend_from_slice(&self.data_size.to_le_bytes());
        data.extend_from_slice(&self.compressed_size.to_le_bytes());
        data.extend_from_slice(&self.checksum.to_le_bytes());
        data.extend_from_slice(&self.timestamp.to_le_bytes());
        data
    }

    pub fn deserialize(data: &[u8]) -> StorageResult<Self> {
        if data.len() < 48 {
            return Err(StorageError::DeserializeError("PageHeader too small".to_string()));
        }

        let magic = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        if magic != PAGE_FILE_MAGIC {
            return Err(StorageError::DeserializeError("Invalid page magic".to_string()));
        }

        let version = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        let table_type_byte = data[8];
        let table_type = match table_type_byte {
            1 => TableType::Vertex,
            2 => TableType::Edge,
            3 => TableType::Property,
            4 => TableType::Schema,
            _ => return Err(StorageError::DeserializeError("Invalid table type".to_string())),
        };
        let label_id = u16::from_le_bytes([data[9], data[10]]);
        let block_number = u64::from_le_bytes([
            data[11], data[12], data[13], data[14],
            data[15], data[16], data[17], data[18],
        ]);
        let data_size = u32::from_le_bytes([data[19], data[20], data[21], data[22]]);
        let compressed_size = u32::from_le_bytes([data[23], data[24], data[25], data[26]]);
        let checksum = u64::from_le_bytes([
            data[27], data[28], data[29], data[30],
            data[31], data[32], data[33], data[34],
        ]);
        let timestamp = u64::from_le_bytes([
            data[35], data[36], data[37], data[38],
            data[39], data[40], data[41], data[42],
        ]);

        Ok(Self {
            magic,
            version,
            page_id: PageId {
                table_type,
                label_id,
                block_number,
            },
            data_size,
            compressed_size,
            checksum,
            timestamp,
        })
    }

    pub fn size() -> usize {
        48
    }
}

pub struct FilePageWriter {
    work_dir: PathBuf,
    compressor: Compressor,
    page_index: RwLock<HashMap<PageId, PageIndexEntry>>,
    write_count: AtomicU64,
    read_count: AtomicU64,
}

#[derive(Debug, Clone)]
struct PageIndexEntry {
    file_path: PathBuf,
    offset: u64,
    size: u64,
    timestamp: u64,
}

impl FilePageWriter {
    pub fn new(work_dir: PathBuf, compression: CompressionType) -> StorageResult<Self> {
        fs::create_dir_all(&work_dir)?;

        let index_path = work_dir.join("page_index.bin");
        let page_index = if index_path.exists() {
            Self::load_index(&index_path)?
        } else {
            HashMap::new()
        };

        Ok(Self {
            work_dir,
            compressor: Compressor::new(compression),
            page_index: RwLock::new(page_index),
            write_count: AtomicU64::new(0),
            read_count: AtomicU64::new(0),
        })
    }

    fn get_page_file_path(&self, page_id: &PageId) -> PathBuf {
        let table_dir = match page_id.table_type {
            TableType::Vertex => self.work_dir.join("vertex"),
            TableType::Edge => self.work_dir.join("edge"),
            TableType::Property => self.work_dir.join("property"),
            TableType::Schema => self.work_dir.join("schema"),
        };

        fs::create_dir_all(&table_dir).ok();

        table_dir.join(format!("{}_{}.page", page_id.label_id, page_id.block_number))
    }

    fn load_index(index_path: &Path) -> StorageResult<HashMap<PageId, PageIndexEntry>> {
        if !index_path.exists() {
            return Ok(HashMap::new());
        }

        let mut file = File::open(index_path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        let mut index = HashMap::new();
        let mut offset = 0;

        while offset + 4 <= data.len() {
            let key_len = u32::from_le_bytes([
                data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
            ]) as usize;
            offset += 4;

            if offset + key_len > data.len() {
                break;
            }

            let table_type_byte = data[offset];
            let table_type = match table_type_byte {
                1 => TableType::Vertex,
                2 => TableType::Edge,
                3 => TableType::Property,
                4 => TableType::Schema,
                _ => break,
            };
            offset += 1;

            let label_id = u16::from_le_bytes([data[offset], data[offset + 1]]);
            offset += 2;

            let block_number = u64::from_le_bytes([
                data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
                data[offset + 4], data[offset + 5], data[offset + 6], data[offset + 7],
            ]);
            offset += 8;

            let path_len = u32::from_le_bytes([
                data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
            ]) as usize;
            offset += 4;

            if offset + path_len > data.len() {
                break;
            }

            let file_path = String::from_utf8_lossy(&data[offset..offset + path_len]).to_string();
            offset += path_len;

            let file_offset = u64::from_le_bytes([
                data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
                data[offset + 4], data[offset + 5], data[offset + 6], data[offset + 7],
            ]);
            offset += 8;

            let size = u64::from_le_bytes([
                data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
                data[offset + 4], data[offset + 5], data[offset + 6], data[offset + 7],
            ]);
            offset += 8;

            let timestamp = u64::from_le_bytes([
                data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
                data[offset + 4], data[offset + 5], data[offset + 6], data[offset + 7],
            ]);
            offset += 8;

            let page_id = PageId {
                table_type,
                label_id,
                block_number,
            };

            index.insert(
                page_id,
                PageIndexEntry {
                    file_path: PathBuf::from(file_path),
                    offset: file_offset,
                    size,
                    timestamp,
                },
            );
        }

        Ok(index)
    }

    fn save_index(&self) -> StorageResult<()> {
        let index_path = self.work_dir.join("page_index.bin");
        let index = self.page_index.read();

        let mut data = Vec::new();

        for (page_id, entry) in index.iter() {
            data.extend_from_slice(&1u32.to_le_bytes());
            data.extend_from_slice(&(page_id.table_type as u8).to_le_bytes());
            data.extend_from_slice(&page_id.label_id.to_le_bytes());
            data.extend_from_slice(&page_id.block_number.to_le_bytes());

            let path_cow = entry.file_path.to_string_lossy();
            let path_bytes = path_cow.as_bytes();
            data.extend_from_slice(&(path_bytes.len() as u32).to_le_bytes());
            data.extend_from_slice(path_bytes);

            data.extend_from_slice(&entry.offset.to_le_bytes());
            data.extend_from_slice(&entry.size.to_le_bytes());
            data.extend_from_slice(&entry.timestamp.to_le_bytes());
        }

        let temp_path = index_path.with_extension("tmp");
        let mut file = File::create(&temp_path)?;
        file.write_all(&data)?;
        file.sync_all()?;
        drop(file);

        fs::rename(&temp_path, &index_path)?;

        Ok(())
    }

    pub fn write_count(&self) -> u64 {
        self.write_count.load(Ordering::Relaxed)
    }

    pub fn read_count(&self) -> u64 {
        self.read_count.load(Ordering::Relaxed)
    }

    pub fn page_count(&self) -> usize {
        self.page_index.read().len()
    }
}

impl PageWriter for FilePageWriter {
    fn write_page(&self, page_id: &PageId, data: &[u8]) -> StorageResult<()> {
        let compressed = self.compressor.compress(data)?;

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut header = PageHeader::new(
            *page_id,
            data.len() as u32,
            compressed.len() as u32,
            timestamp,
        );
        header.update_checksum();

        let file_path = self.get_page_file_path(page_id);
        let temp_path = file_path.with_extension("tmp");

        {
            let mut file = File::create(&temp_path)?;
            file.write_all(&header.serialize())?;
            file.write_all(&compressed)?;
            file.sync_all()?;
        }

        fs::rename(&temp_path, &file_path)?;

        let entry = PageIndexEntry {
            file_path: file_path.clone(),
            offset: 0,
            size: (PageHeader::size() + compressed.len()) as u64,
            timestamp,
        };

        self.page_index.write().insert(*page_id, entry);
        self.write_count.fetch_add(1, Ordering::Relaxed);

        if self.write_count.load(Ordering::Relaxed) % 100 == 0 {
            self.save_index()?;
        }

        Ok(())
    }

    fn read_page(&self, page_id: &PageId) -> StorageResult<Option<Vec<u8>>> {
        let index = self.page_index.read();
        let entry = match index.get(page_id) {
            Some(e) => e.clone(),
            None => {
                let file_path = self.get_page_file_path(page_id);
                if !file_path.exists() {
                    return Ok(None);
                }

                let mut file = File::open(&file_path)?;
                let mut data = Vec::new();
                file.read_to_end(&mut data)?;

                if data.len() < PageHeader::size() {
                    return Err(StorageError::DeserializeError("Page file too small".to_string()));
                }

                let header = PageHeader::deserialize(&data[..PageHeader::size()])?;
                if !header.verify_checksum() {
                    return Err(StorageError::DataCorruption(
                        "Page checksum verification failed".to_string(),
                    ));
                }

                let compressed = &data[PageHeader::size()..];
                let decompressed = self.compressor.decompress(compressed)?;

                self.read_count.fetch_add(1, Ordering::Relaxed);
                return Ok(Some(decompressed));
            }
        };
        drop(index);

        if !entry.file_path.exists() {
            return Ok(None);
        }

        let mut file = File::open(&entry.file_path)?;
        let mut data = vec![0u8; entry.size as usize];
        file.read_exact(&mut data)?;

        if data.len() < PageHeader::size() {
            return Err(StorageError::DeserializeError("Page file too small".to_string()));
        }

        let header = PageHeader::deserialize(&data[..PageHeader::size()])?;
        if !header.verify_checksum() {
            return Err(StorageError::DataCorruption(
                "Page checksum verification failed".to_string(),
            ));
        }

        let compressed = &data[PageHeader::size()..];
        let decompressed = self.compressor.decompress(compressed)?;

        self.read_count.fetch_add(1, Ordering::Relaxed);
        Ok(Some(decompressed))
    }
}

pub struct CheckpointManager {
    work_dir: PathBuf,
    dirty_tracker: Arc<DirtyPageTracker>,
    page_writer: Arc<FilePageWriter>,
    last_checkpoint: RwLock<CheckpointInfo>,
    checkpoint_count: AtomicU64,
}

#[derive(Debug, Clone)]
pub struct CheckpointInfo {
    pub timestamp: u64,
    pub dirty_page_count: usize,
    pub checkpoint_id: u64,
}

impl CheckpointManager {
    pub fn new(
        work_dir: PathBuf,
        dirty_tracker: Arc<DirtyPageTracker>,
        page_writer: Arc<FilePageWriter>,
    ) -> Self {
        let last_checkpoint = Self::load_last_checkpoint(&work_dir);

        Self {
            work_dir,
            dirty_tracker,
            page_writer,
            last_checkpoint: RwLock::new(last_checkpoint),
            checkpoint_count: AtomicU64::new(0),
        }
    }

    fn load_last_checkpoint(work_dir: &Path) -> CheckpointInfo {
        let checkpoint_path = work_dir.join("last_checkpoint.bin");
        if checkpoint_path.exists() {
            if let Ok(mut file) = File::open(&checkpoint_path) {
                let mut data = [0u8; 24];
                if file.read_exact(&mut data).is_ok() {
                    let timestamp = u64::from_le_bytes([
                        data[0], data[1], data[2], data[3],
                        data[4], data[5], data[6], data[7],
                    ]);
                    let dirty_page_count = u64::from_le_bytes([
                        data[8], data[9], data[10], data[11],
                        data[12], data[13], data[14], data[15],
                    ]) as usize;
                    let checkpoint_id = u64::from_le_bytes([
                        data[16], data[17], data[18], data[19],
                        data[20], data[21], data[22], data[23],
                    ]);

                    return CheckpointInfo {
                        timestamp,
                        dirty_page_count,
                        checkpoint_id,
                    };
                }
            }
        }

        CheckpointInfo {
            timestamp: 0,
            dirty_page_count: 0,
            checkpoint_id: 0,
        }
    }

    fn save_checkpoint_info(&self, info: &CheckpointInfo) -> StorageResult<()> {
        let checkpoint_path = self.work_dir.join("last_checkpoint.bin");
        let temp_path = checkpoint_path.with_extension("tmp");

        let mut data = Vec::with_capacity(24);
        data.extend_from_slice(&info.timestamp.to_le_bytes());
        data.extend_from_slice(&(info.dirty_page_count as u64).to_le_bytes());
        data.extend_from_slice(&info.checkpoint_id.to_le_bytes());

        let mut file = File::create(&temp_path)?;
        file.write_all(&data)?;
        file.sync_all()?;
        drop(file);

        fs::rename(&temp_path, &checkpoint_path)?;

        Ok(())
    }

    pub fn create_checkpoint(&self) -> StorageResult<CheckpointInfo> {
        let dirty_pages = self.dirty_tracker.get_dirty_pages();
        let dirty_count = dirty_pages.len();

        let last_checkpoint = self.last_checkpoint.read();
        let checkpoint_id = last_checkpoint.checkpoint_id + 1;
        drop(last_checkpoint);

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let info = CheckpointInfo {
            timestamp,
            dirty_page_count: dirty_count,
            checkpoint_id,
        };

        self.save_checkpoint_info(&info)?;

        *self.last_checkpoint.write() = info.clone();
        self.checkpoint_count.fetch_add(1, Ordering::Relaxed);

        Ok(info)
    }

    pub fn get_last_checkpoint(&self) -> CheckpointInfo {
        self.last_checkpoint.read().clone()
    }

    pub fn checkpoint_count(&self) -> u64 {
        self.checkpoint_count.load(Ordering::Relaxed)
    }

    pub fn needs_checkpoint(&self, max_dirty_pages: usize, max_interval_secs: u64) -> bool {
        let dirty_count = self.dirty_tracker.get_dirty_page_count();
        if dirty_count >= max_dirty_pages {
            return true;
        }

        let last_checkpoint = self.last_checkpoint.read();
        if last_checkpoint.timestamp == 0 {
            return true;
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        (now - last_checkpoint.timestamp) >= max_interval_secs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_page_header_serialization() {
        let page_id = PageId {
            table_type: TableType::Vertex,
            label_id: 1,
            block_number: 0,
        };

        let mut header = PageHeader::new(page_id, 100, 50, 12345);
        header.update_checksum();

        let serialized = header.serialize();
        assert_eq!(serialized.len(), PageHeader::size());

        let deserialized = PageHeader::deserialize(&serialized).expect("Deserialize failed");
        assert_eq!(deserialized.magic, header.magic);
        assert_eq!(deserialized.version, header.version);
        assert_eq!(deserialized.page_id.table_type, header.page_id.table_type);
        assert_eq!(deserialized.page_id.label_id, header.page_id.label_id);
        assert_eq!(deserialized.page_id.block_number, header.page_id.block_number);
        assert_eq!(deserialized.data_size, header.data_size);
        assert_eq!(deserialized.compressed_size, header.compressed_size);
        assert_eq!(deserialized.checksum, header.checksum);
        assert_eq!(deserialized.timestamp, header.timestamp);
    }

    #[test]
    fn test_page_header_checksum() {
        let page_id = PageId {
            table_type: TableType::Vertex,
            label_id: 1,
            block_number: 0,
        };

        let mut header = PageHeader::new(page_id, 100, 50, 12345);
        header.update_checksum();

        assert!(header.verify_checksum());

        let mut corrupted = header.clone();
        corrupted.data_size = 200;
        assert!(!corrupted.verify_checksum());
    }

    #[test]
    fn test_file_page_writer() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let work_dir = temp_dir.path().to_path_buf();

        let writer = FilePageWriter::new(work_dir.clone(), CompressionType::None)
            .expect("Failed to create writer");

        let page_id = PageId {
            table_type: TableType::Vertex,
            label_id: 1,
            block_number: 0,
        };

        let data = b"test page data";
        writer.write_page(&page_id, data).expect("Write failed");

        let read_data = writer.read_page(&page_id).expect("Read failed");
        assert!(read_data.is_some());
        assert_eq!(read_data.unwrap(), data);
    }

    #[test]
    fn test_checkpoint_manager() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let work_dir = temp_dir.path().to_path_buf();

        let dirty_tracker = Arc::new(DirtyPageTracker::new(1000, std::time::Duration::from_secs(60)));
        let writer = Arc::new(
            FilePageWriter::new(work_dir.clone(), CompressionType::None)
                .expect("Failed to create writer"),
        );

        let manager = CheckpointManager::new(work_dir, dirty_tracker.clone(), writer);

        let page_id = PageId {
            table_type: TableType::Vertex,
            label_id: 1,
            block_number: 0,
        };
        dirty_tracker.mark_dirty(page_id);

        let info = manager.create_checkpoint().expect("Checkpoint failed");
        assert_eq!(info.checkpoint_id, 1);
        assert_eq!(info.dirty_page_count, 1);

        let last = manager.get_last_checkpoint();
        assert_eq!(last.checkpoint_id, 1);
    }
}
