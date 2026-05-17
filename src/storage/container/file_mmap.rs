//! File-Backed MMap Container
//!
//! Memory-mapped file container for persistent storage.

use std::fs::{File, OpenOptions};
use std::io::Write as IoWrite;
use std::path::Path;

use super::mmap::{IDataContainer, MmapBase};
use super::types::{
    ContainerConfig, ContainerError, ContainerResult, ContainerStats, FileHeader, MemoryLevel,
};

/// File-backed mmap container
pub struct FileMmap {
    base: MmapBase,
    file: Option<File>,
    mmap: Option<memmap2::MmapMut>,
    config: ContainerConfig,
    allocation_count: u64,
}

impl FileMmap {
    pub fn create<P: AsRef<Path>>(path: P, capacity: usize) -> ContainerResult<Self> {
        Self::with_config(
            path,
            ContainerConfig {
                initial_capacity: capacity,
                memory_level: MemoryLevel::SyncToFile,
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
            memmap2::MmapMut::map_mut(&file).map_err(|e| ContainerError::MappingFailed(e.to_string()))?
        };

        let header = FileHeader::new(0);
        let mut mmap = mmap;
        mmap[..FileHeader::SIZE].copy_from_slice(header.as_bytes());

        let mut base = MmapBase::new();
        base.path = Some(path);
        base.capacity = total_size;
        base.size = 0;

        Ok(Self {
            base,
            file: Some(file),
            mmap: Some(mmap),
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
            memmap2::MmapMut::map_mut(&file).map_err(|e| ContainerError::MappingFailed(e.to_string()))?
        };

        let size = if file_size >= FileHeader::SIZE {
            let header = FileHeader::from_bytes(&mmap[..FileHeader::SIZE]).ok_or_else(|| {
                ContainerError::InvalidOperation("Invalid file header".to_string())
            })?;
            header.data_size as usize
        } else {
            0
        };

        let mut base = MmapBase::new();
        base.path = Some(path);
        base.capacity = file_size;
        base.size = size;

        Ok(Self {
            base,
            file: Some(file),
            mmap: Some(mmap),
            config: ContainerConfig::default(),
            allocation_count: 1,
        })
    }

    pub fn as_slice(&self) -> &[u8] {
        if let Some(ref mmap) = self.mmap {
            if mmap.len() > FileHeader::SIZE {
                return &mmap[FileHeader::SIZE..FileHeader::SIZE + self.base.size];
            }
        }
        &[]
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        if let Some(ref mut mmap) = self.mmap {
            if mmap.len() > FileHeader::SIZE {
                return &mut mmap[FileHeader::SIZE..FileHeader::SIZE + self.base.size];
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

        if end > self.base.capacity {
            self.resize(end - FileHeader::SIZE)?;
        }

        if let Some(ref mut mmap) = self.mmap {
            if end > mmap.len() {
                return Err(ContainerError::InvalidSize(format!(
                    "Write of {} bytes at offset {} exceeds capacity {}",
                    data.len(),
                    offset,
                    self.base.capacity.saturating_sub(FileHeader::SIZE)
                )));
            }
            mmap[start..end].copy_from_slice(data);
            if offset + data.len() > self.base.size {
                self.base.size = offset + data.len();
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
                    self.base.capacity.saturating_sub(FileHeader::SIZE)
                )));
            }
            return Ok(mmap[start..end].to_vec());
        }

        Err(ContainerError::NotInitialized)
    }

    fn update_header(&mut self) -> ContainerResult<()> {
        if let Some(ref mut mmap) = self.mmap {
            let header = FileHeader::new(self.base.size as u64);
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

        let header = FileHeader::new(self.base.size as u64);
        file.write_all(header.as_bytes())?;

        if let Some(ref mmap) = self.mmap {
            if self.base.size > 0 {
                file.write_all(&mmap[FileHeader::SIZE..FileHeader::SIZE + self.base.size])?;
            }
        }

        file.sync_all()?;
        Ok(())
    }
}

impl IDataContainer for FileMmap {
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
        self.base.size
    }

    fn capacity(&self) -> usize {
        self.base.capacity
    }

    fn is_open(&self) -> bool {
        self.mmap.is_some()
    }

    fn sync(&self) -> ContainerResult<()> {
        if let Some(ref mmap) = self.mmap {
            mmap.flush()?;
        }
        if let Some(ref file) = self.file {
            file.sync_all()?;
        }
        Ok(())
    }

    fn resize(&mut self, new_size: usize) -> ContainerResult<()> {
        let new_capacity = FileHeader::SIZE + new_size;

        if new_capacity <= self.base.capacity {
            self.base.size = new_size;
            self.update_header()?;
            return Ok(());
        }

        if self.config.max_capacity > 0 && new_size > self.config.max_capacity {
            return Err(ContainerError::InvalidSize(
                "Exceeds maximum capacity".to_string(),
            ));
        }

        let growth_capacity =
            ((self.base.capacity as f64 * self.config.growth_factor) as usize).max(new_capacity);

        if let Some(ref file) = self.file {
            file.set_len(growth_capacity as u64)?;

            let mmap = unsafe {
                memmap2::MmapMut::map_mut(file)
                    .map_err(|e| ContainerError::MappingFailed(e.to_string()))?
            };

            self.mmap = Some(mmap);
            self.base.capacity = growth_capacity;
        }

        self.base.size = new_size;
        self.update_header()?;
        self.allocation_count += 1;
        Ok(())
    }

    fn close(&mut self) {
        if let Err(e) = self.sync() {
            log::warn!("Failed to sync before close: {}", e);
        }
        if let Some(mmap) = self.mmap.take() {
            drop(mmap);
        }
        self.file = None;
        self.base.size = 0;
        self.base.capacity = 0;
    }

    fn stats(&self) -> ContainerStats {
        ContainerStats {
            capacity: self.base.capacity,
            used: self.base.size,
            is_huge_page: false,
            allocation_count: self.allocation_count,
        }
    }

    fn memory_level(&self) -> MemoryLevel {
        MemoryLevel::SyncToFile
    }

    fn path(&self) -> Option<&Path> {
        self.base.path.as_deref()
    }
}

impl Drop for FileMmap {
    fn drop(&mut self) {
        self.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_file_mmap_create() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path().join("test.mmap");

        let mut container = FileMmap::create(&path, 1024).expect("Failed to create container");
        assert!(container.is_open());

        container.write_at(0, b"hello").expect("Failed to write");
        let data = container.read_at(0, 5).expect("Failed to read");
        assert_eq!(&data, b"hello");
    }

    #[test]
    fn test_file_mmap_open() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path().join("test.mmap");

        {
            let mut container =
                FileMmap::create(&path, 1024).expect("Failed to create container");
            container.write_at(0, b"world").expect("Failed to write");
            container.sync().expect("Failed to sync");
        }

        let container = FileMmap::open(&path).expect("Failed to open container");
        let data = container.read_at(0, 5).expect("Failed to read");
        assert_eq!(&data, b"world");
    }

    #[test]
    fn test_file_mmap_resize() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path().join("test.mmap");

        let mut container = FileMmap::create(&path, 100).expect("Failed to create container");
        assert!(container.resize(1000).is_ok());
        assert!(container.capacity() >= 1000 + FileHeader::SIZE);
    }

    #[test]
    fn test_file_mmap_empty_write() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path().join("test.mmap");

        let mut container = FileMmap::create(&path, 1024).expect("Failed to create container");
        container.write_at(0, b"").expect("Empty write should succeed");
        assert_eq!(container.size(), 0);
    }

    #[test]
    fn test_file_mmap_read_out_of_bounds() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path().join("test.mmap");

        let container = FileMmap::create(&path, 100).expect("Failed to create container");
        let result = container.read_at(0, 200);
        assert!(result.is_err());
        match result {
            Err(ContainerError::InvalidSize(_)) => {}
            _ => panic!("Expected InvalidSize error"),
        }
    }

    #[test]
    fn test_file_mmap_write_exceeds_capacity() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path().join("test.mmap");

        let mut container = FileMmap::create(&path, 100).expect("Failed to create container");
        let large_data = vec![0xABu8; 500];
        container.write_at(0, &large_data).expect("Write should trigger auto-resize");
        let data = container.read_at(0, 500).expect("Failed to read");
        assert_eq!(data, large_data);
    }

    #[test]
    fn test_file_mmap_stats() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path().join("test.mmap");

        let mut container = FileMmap::create(&path, 1024).expect("Failed to create container");
        let stats = container.stats();
        assert_eq!(stats.allocation_count, 1);

        container.resize(4096).expect("Failed to resize");
        let stats = container.stats();
        assert_eq!(stats.allocation_count, 2);
    }

    #[test]
    fn test_file_mmap_close_sync() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path().join("test.mmap");

        let mut container = FileMmap::create(&path, 1024).expect("Failed to create container");
        container.write_at(0, b"persist").expect("Failed to write");
        container.close();

        let container = FileMmap::open(&path).expect("Failed to reopen");
        let data = container.read_at(0, 7).expect("Failed to read");
        assert_eq!(&data, b"persist");
    }

    #[test]
    fn test_file_mmap_dump() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path().join("test.mmap");
        let dump_path = temp_dir.path().join("dump.mmap");

        let mut container = FileMmap::create(&path, 1024).expect("Failed to create container");
        container.write_at(0, b"dump test").expect("Failed to write");
        container.dump(&dump_path).expect("Failed to dump");

        let loaded = FileMmap::open(&dump_path).expect("Failed to open dump");
        let data = loaded.read_at(0, 9).expect("Failed to read");
        assert_eq!(&data, b"dump test");
    }

    #[test]
    fn test_file_mmap_boundary_write() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path().join("test.mmap");

        let mut container = FileMmap::create(&path, 20).expect("Failed to create container");
        container.write_at(10, b"boundary").expect("Failed to write at offset 10");
        assert_eq!(container.size(), 18);

        let data = container.read_at(10, 8).expect("Failed to read");
        assert_eq!(&data, b"boundary");
    }

    #[test]
    fn test_file_mmap_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<FileMmap>();
        assert_sync::<FileMmap>();
    }
}
