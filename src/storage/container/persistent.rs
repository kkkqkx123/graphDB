//! Persistent Container
//!
//! Memory-mapped file container for persistent storage.
//! This is the default storage backend for database operations.

use std::fs::{File, OpenOptions};
use std::io::Write as IoWrite;
use std::path::{Path, PathBuf};

use super::types::{
    ContainerConfig, ContainerError, ContainerResult, ContainerStats, FileHeader, StorageBackend,
};

/// Persistent container backed by memory-mapped file
///
/// Data is automatically synced to disk via mmap.
/// This is the default and recommended storage backend for database operations.
pub struct PersistentContainer {
    mmap: Option<memmap2::MmapMut>,
    file: Option<File>,
    path: PathBuf,
    size: usize,
    capacity: usize,
    config: ContainerConfig,
    allocation_count: u64,
}

impl PersistentContainer {
    pub fn create<P: AsRef<Path>>(path: P, capacity: usize) -> ContainerResult<Self> {
        Self::with_config(
            path,
            ContainerConfig {
                initial_capacity: capacity,
                storage_backend: StorageBackend::Persistent,
                ..Default::default()
            },
        )
    }

    pub fn with_config<P: AsRef<Path>>(path: P, config: ContainerConfig) -> ContainerResult<Self> {
        let path = path.as_ref().to_path_buf();
        let total_size = FileHeader::SIZE + config.initial_capacity;

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)?;

        file.set_len(total_size as u64)?;

        let mmap = unsafe {
            memmap2::MmapMut::map_mut(&file)
                .map_err(|e| ContainerError::MappingFailed(e.to_string()))?
        };

        let header = FileHeader::new(0);
        let mut mmap = mmap;
        mmap[..FileHeader::SIZE].copy_from_slice(header.as_bytes());

        Ok(Self {
            mmap: Some(mmap),
            file: Some(file),
            path,
            size: 0,
            capacity: total_size,
            config,
            allocation_count: 1,
        })
    }

    pub fn open<P: AsRef<Path>>(path: P) -> ContainerResult<Self> {
        let path = path.as_ref().to_path_buf();

        if !path.exists() {
            return Err(ContainerError::FileNotFound(path.display().to_string()));
        }

        let file = OpenOptions::new().read(true).write(true).open(&path)?;

        let metadata = file.metadata()?;
        let file_size = metadata.len() as usize;

        let mmap = unsafe {
            memmap2::MmapMut::map_mut(&file)
                .map_err(|e| ContainerError::MappingFailed(e.to_string()))?
        };

        let size = if file_size >= FileHeader::SIZE {
            let header = FileHeader::from_bytes(&mmap[..FileHeader::SIZE]).ok_or_else(|| {
                ContainerError::InvalidOperation("Invalid file header".to_string())
            })?;
            header.data_size as usize
        } else {
            0
        };

        Ok(Self {
            mmap: Some(mmap),
            file: Some(file),
            path,
            size,
            capacity: file_size,
            config: ContainerConfig::default(),
            allocation_count: 1,
        })
    }

    pub fn as_slice(&self) -> &[u8] {
        if let Some(ref mmap) = self.mmap {
            if mmap.len() > FileHeader::SIZE {
                return &mmap[FileHeader::SIZE..FileHeader::SIZE + self.size];
            }
        }
        &[]
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        if let Some(ref mut mmap) = self.mmap {
            if mmap.len() > FileHeader::SIZE {
                return &mut mmap[FileHeader::SIZE..FileHeader::SIZE + self.size];
            }
        }
        &mut []
    }

    pub fn write_at(&mut self, offset: usize, data: &[u8]) -> ContainerResult<()> {
        if data.is_empty() {
            return Ok(());
        }

        let start = FileHeader::SIZE + offset;
        let end = start + data.len();

        if end > self.capacity {
            self.do_resize(end - FileHeader::SIZE)?;
        }

        if let Some(ref mut mmap) = self.mmap {
            if end > mmap.len() {
                return Err(ContainerError::InvalidSize(format!(
                    "Write of {} bytes at offset {} exceeds capacity {}",
                    data.len(),
                    offset,
                    self.capacity.saturating_sub(FileHeader::SIZE)
                )));
            }
            mmap[start..end].copy_from_slice(data);
            if offset + data.len() > self.size {
                self.size = offset + data.len();
                self.update_header()?;
            }
            return Ok(());
        }

        Err(ContainerError::NotInitialized)
    }

    pub fn read_at(&self, offset: usize, len: usize) -> ContainerResult<Vec<u8>> {
        let start = FileHeader::SIZE + offset;
        let end = start + len;

        if let Some(ref mmap) = self.mmap {
            if end > mmap.len() {
                return Err(ContainerError::InvalidSize(format!(
                    "Read of {} bytes at offset {} exceeds capacity {}",
                    len,
                    offset,
                    self.capacity.saturating_sub(FileHeader::SIZE)
                )));
            }
            return Ok(mmap[start..end].to_vec());
        }

        Err(ContainerError::NotInitialized)
    }

    fn update_header(&mut self) -> ContainerResult<()> {
        if let Some(ref mut mmap) = self.mmap {
            let header = FileHeader::new(self.size as u64);
            mmap[..FileHeader::SIZE].copy_from_slice(header.as_bytes());
        }
        Ok(())
    }

    pub fn dump<P: AsRef<Path>>(&mut self, path: P) -> ContainerResult<()> {
        let path = path.as_ref();

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;

        let header = FileHeader::new(self.size as u64);
        file.write_all(header.as_bytes())?;

        if let Some(ref mmap) = self.mmap {
            if self.size > 0 {
                file.write_all(&mmap[FileHeader::SIZE..FileHeader::SIZE + self.size])?;
            }
        }

        file.sync_all()?;
        Ok(())
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn do_resize(&mut self, new_size: usize) -> ContainerResult<()> {
        let new_capacity = FileHeader::SIZE + new_size;

        if new_capacity <= self.capacity {
            self.size = new_size;
            self.update_header()?;
            return Ok(());
        }

        if self.config.max_capacity > 0 && new_size > self.config.max_capacity {
            return Err(ContainerError::InvalidSize(
                "Exceeds maximum capacity".to_string(),
            ));
        }

        let growth_capacity =
            ((self.capacity as f64 * self.config.growth_factor) as usize).max(new_capacity);

        if let Some(ref file) = self.file {
            file.set_len(growth_capacity as u64)?;

            let mmap = unsafe {
                memmap2::MmapMut::map_mut(file)
                    .map_err(|e| ContainerError::MappingFailed(e.to_string()))?
            };

            self.mmap = Some(mmap);
            self.capacity = growth_capacity;
        }

        self.size = new_size;
        self.update_header()?;
        self.allocation_count += 1;
        Ok(())
    }

    fn do_close(&mut self) {
        if let Err(e) = self.do_sync() {
            log::warn!("Failed to sync before close: {}", e);
        }
        if let Some(mmap) = self.mmap.take() {
            drop(mmap);
        }
        self.file = None;
        self.size = 0;
        self.capacity = 0;
    }

    fn do_sync(&self) -> ContainerResult<()> {
        if let Some(ref mmap) = self.mmap {
            mmap.flush()?;
        }
        if let Some(ref file) = self.file {
            file.sync_all()?;
        }
        Ok(())
    }
}

impl super::IDataContainer for PersistentContainer {
    fn data(&self) -> *const u8 {
        if let Some(ref mmap) = self.mmap {
            if mmap.len() > FileHeader::SIZE {
                return mmap[FileHeader::SIZE..].as_ptr();
            }
        }
        std::ptr::null()
    }

    fn data_mut(&mut self) -> *mut u8 {
        if let Some(ref mut mmap) = self.mmap {
            if mmap.len() > FileHeader::SIZE {
                return mmap[FileHeader::SIZE..].as_mut_ptr();
            }
        }
        std::ptr::null_mut()
    }

    fn size(&self) -> usize {
        self.size
    }

    fn capacity(&self) -> usize {
        self.capacity
    }

    fn is_open(&self) -> bool {
        self.mmap.is_some()
    }

    fn sync(&self) -> ContainerResult<()> {
        self.do_sync()
    }

    fn resize(&mut self, new_size: usize) -> ContainerResult<()> {
        self.do_resize(new_size)
    }

    fn close(&mut self) {
        self.do_close();
    }

    fn stats(&self) -> ContainerStats {
        ContainerStats {
            capacity: self.capacity,
            used: self.size,
            is_huge_page: false,
            allocation_count: self.allocation_count,
        }
    }

    fn storage_backend(&self) -> StorageBackend {
        StorageBackend::Persistent
    }

    fn file_path(&self) -> Option<&Path> {
        Some(&self.path)
    }
}

impl Drop for PersistentContainer {
    fn drop(&mut self) {
        self.do_close();
    }
}

#[deprecated(since = "0.2.0", note = "Use PersistentContainer instead")]
pub type FileMmap = PersistentContainer;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::storage::container::mmap::IDataContainer;

    #[test]
    fn test_persistent_container_create() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path().join("test.mmap");

        let mut container =
            PersistentContainer::create(&path, 1024).expect("Failed to create container");
        assert!(container.is_open());

        container.write_at(0, b"hello").expect("Failed to write");
        let data = container.read_at(0, 5).expect("Failed to read");
        assert_eq!(&data, b"hello");
    }

    #[test]
    fn test_persistent_container_open() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path().join("test.mmap");

        {
            let mut container =
                PersistentContainer::create(&path, 1024).expect("Failed to create container");
            container.write_at(0, b"world").expect("Failed to write");
            container.sync().expect("Failed to sync");
        }

        let container = PersistentContainer::open(&path).expect("Failed to open container");
        let data = container.read_at(0, 5).expect("Failed to read");
        assert_eq!(&data, b"world");
    }

    #[test]
    fn test_persistent_container_resize() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path().join("test.mmap");

        let mut container =
            PersistentContainer::create(&path, 100).expect("Failed to create container");
        assert!(container.resize(1000).is_ok());
        assert!(container.capacity() >= 1000 + FileHeader::SIZE);
    }

    #[test]
    fn test_persistent_container_stats() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path().join("test.mmap");

        let mut container =
            PersistentContainer::create(&path, 1024).expect("Failed to create container");
        let stats = container.stats();
        assert_eq!(stats.allocation_count, 1);

        container.resize(4096).expect("Failed to resize");
        let stats = container.stats();
        assert_eq!(stats.allocation_count, 2);
    }

    #[test]
    fn test_persistent_container_sync() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path().join("test.mmap");

        let mut container =
            PersistentContainer::create(&path, 1024).expect("Failed to create container");
        container.write_at(0, b"persist").expect("Failed to write");
        container.close();

        let container = PersistentContainer::open(&path).expect("Failed to reopen");
        let data = container.read_at(0, 7).expect("Failed to read");
        assert_eq!(&data, b"persist");
    }

    #[test]
    fn test_persistent_container_storage_backend() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path().join("test.mmap");

        let container =
            PersistentContainer::create(&path, 1024).expect("Failed to create container");
        assert!(container.storage_backend().is_persistent());
    }

    #[test]
    fn test_persistent_container_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<PersistentContainer>();
        assert_sync::<PersistentContainer>();
    }
}
