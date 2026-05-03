//! Memory-Mapped Container
//!
//! Provides memory-mapped file and anonymous memory container implementations
//! for efficient I/O operations.

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use super::types::{ContainerConfig, ContainerError, ContainerResult, FileHeader};

/// Trait for data containers
pub trait IDataContainer: Send + Sync {
    /// Get the data pointer
    fn data(&self) -> *const u8;

    /// Get the mutable data pointer
    fn data_mut(&mut self) -> *mut u8;

    /// Get the size of the data
    fn size(&self) -> usize;

    /// Check if the container is open
    fn is_open(&self) -> bool;

    /// Sync data to disk
    fn sync(&self) -> ContainerResult<()>;

    /// Resize the container
    fn resize(&mut self, new_size: usize) -> ContainerResult<()>;

    /// Close the container
    fn close(&mut self);
}

/// Memory-mapped container base implementation
pub struct MmapContainer {
    /// File path (None for anonymous mapping)
    path: Option<PathBuf>,
    /// File handle
    file: Option<File>,
    /// Memory-mapped data
    mmap_data: Option<memmap2::MmapMut>,
    /// Data size (excluding header)
    size: usize,
    /// Capacity (total mapped size)
    capacity: usize,
    /// Whether the container is open
    is_open: AtomicBool,
    /// Configuration
    config: ContainerConfig,
}

impl MmapContainer {
    /// Create a new anonymous mmap container
    pub fn create_anonymous(capacity: usize) -> ContainerResult<Self> {
        let config = ContainerConfig::default().with_initial_capacity(capacity);
        Self::create_anonymous_with_config(config)
    }

    /// Create a new anonymous mmap container with configuration
    pub fn create_anonymous_with_config(config: ContainerConfig) -> ContainerResult<Self> {
        let capacity = config.initial_capacity;

        let container = Self {
            path: None,
            file: None,
            mmap_data: None,
            size: 0,
            capacity: 0,
            is_open: AtomicBool::new(false),
            config,
        };

        let mut container = container;
        container.resize(capacity)?;
        container.is_open.store(true, Ordering::SeqCst);
        Ok(container)
    }

    /// Open an existing file-backed mmap container
    pub fn open<P: AsRef<Path>>(path: P) -> ContainerResult<Self> {
        let path = path.as_ref().to_path_buf();

        if !path.exists() {
            return Err(ContainerError::FileNotFound(path.display().to_string()));
        }

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)?;

        let metadata = file.metadata()?;
        let file_size = metadata.len() as usize;

        let mmap = unsafe {
            memmap2::MmapMut::map_mut(&file)
                .map_err(|e| ContainerError::MappingFailed(e.to_string()))?
        };

        let (size, capacity) = if file_size >= FileHeader::SIZE {
            let header = FileHeader::from_bytes(&mmap[..FileHeader::SIZE])
                .ok_or_else(|| ContainerError::InvalidOperation("Invalid file header".to_string()))?;
            (header.data_size as usize, file_size)
        } else {
            (0, file_size)
        };

        Ok(Self {
            path: Some(path),
            file: Some(file),
            mmap_data: Some(mmap),
            size,
            capacity,
            is_open: AtomicBool::new(true),
            config: ContainerConfig::default(),
        })
    }

    /// Create a new file-backed mmap container
    pub fn create<P: AsRef<Path>>(path: P, capacity: usize) -> ContainerResult<Self> {
        let path = path.as_ref().to_path_buf();

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)?;

        let total_size = FileHeader::SIZE + capacity;
        file.set_len(total_size as u64)?;

        let mmap = unsafe {
            memmap2::MmapMut::map_mut(&file)
                .map_err(|e| ContainerError::MappingFailed(e.to_string()))?
        };

        let header = FileHeader::new(0);
        let mut mmap = mmap;
        mmap[..FileHeader::SIZE].copy_from_slice(header.as_bytes());

        Ok(Self {
            path: Some(path),
            file: Some(file),
            mmap_data: Some(mmap),
            size: 0,
            capacity: total_size,
            is_open: AtomicBool::new(true),
            config: ContainerConfig::default(),
        })
    }

    /// Get the file path
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Get the data slice (excluding header)
    pub fn as_slice(&self) -> &[u8] {
        if let Some(ref mmap) = self.mmap_data {
            if mmap.len() > FileHeader::SIZE {
                return &mmap[FileHeader::SIZE..FileHeader::SIZE + self.size];
            }
        }
        &[]
    }

    /// Get the mutable data slice (excluding header)
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        if let Some(ref mut mmap) = self.mmap_data {
            if mmap.len() > FileHeader::SIZE {
                return &mut mmap[FileHeader::SIZE..FileHeader::SIZE + self.size];
            }
        }
        &mut []
    }

    /// Write data at the specified offset
    pub fn write_at(&mut self, offset: usize, data: &[u8]) -> ContainerResult<()> {
        let start = FileHeader::SIZE + offset;
        let end = start + data.len();

        if end > self.capacity {
            self.resize(end - FileHeader::SIZE)?;
        }

        if let Some(ref mut mmap) = self.mmap_data {
            if end <= mmap.len() {
                mmap[start..end].copy_from_slice(data);
                if offset + data.len() > self.size {
                    self.size = offset + data.len();
                    self.update_header()?;
                }
                return Ok(());
            }
        }

        Err(ContainerError::NotInitialized)
    }

    /// Read data from the specified offset
    pub fn read_at(&self, offset: usize, len: usize) -> ContainerResult<Vec<u8>> {
        let start = FileHeader::SIZE + offset;
        let end = start + len;

        if let Some(ref mmap) = self.mmap_data {
            if end <= mmap.len() {
                return Ok(mmap[start..end].to_vec());
            }
        }

        Err(ContainerError::NotInitialized)
    }

    /// Update the file header
    fn update_header(&mut self) -> ContainerResult<()> {
        if let Some(ref mut mmap) = self.mmap_data {
            let header = FileHeader::new(self.size as u64);
            mmap[..FileHeader::SIZE].copy_from_slice(header.as_bytes());
        }
        Ok(())
    }

    /// Dump the container to a file
    pub fn dump<P: AsRef<Path>>(&mut self, path: P) -> ContainerResult<()> {
        let path = path.as_ref();

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;

        let header = FileHeader::new(self.size as u64);
        file.write_all(header.as_bytes())?;

        if let Some(ref mmap) = self.mmap_data {
            if self.size > 0 {
                file.write_all(&mmap[FileHeader::SIZE..FileHeader::SIZE + self.size])?;
            }
        }

        file.sync_all()?;
        Ok(())
    }
}

impl IDataContainer for MmapContainer {
    fn data(&self) -> *const u8 {
        if let Some(ref mmap) = self.mmap_data {
            if mmap.len() > FileHeader::SIZE {
                return mmap[FileHeader::SIZE..].as_ptr();
            }
        }
        std::ptr::null()
    }

    fn data_mut(&mut self) -> *mut u8 {
        if let Some(ref mut mmap) = self.mmap_data {
            if mmap.len() > FileHeader::SIZE {
                return mmap[FileHeader::SIZE..].as_mut_ptr();
            }
        }
        std::ptr::null_mut()
    }

    fn size(&self) -> usize {
        self.size
    }

    fn is_open(&self) -> bool {
        self.is_open.load(Ordering::SeqCst)
    }

    fn sync(&self) -> ContainerResult<()> {
        if let Some(ref mmap) = self.mmap_data {
            mmap.flush()
                .map_err(|e| ContainerError::IoError(e.to_string()))?;
        }
        if let Some(ref file) = self.file {
            file.sync_all()?;
        }
        Ok(())
    }

    fn resize(&mut self, new_size: usize) -> ContainerResult<()> {
        let new_capacity = FileHeader::SIZE + new_size;

        if new_capacity <= self.capacity {
            self.size = new_size;
            self.update_header()?;
            return Ok(());
        }

        let growth_capacity = ((self.capacity as f64 * self.config.growth_factor) as usize)
            .max(new_capacity);

        if self.config.max_capacity > 0 && growth_capacity > self.config.max_capacity {
            return Err(ContainerError::InvalidSize(
                "Exceeds maximum capacity".to_string(),
            ));
        }

        if let Some(ref file) = self.file {
            file.set_len(growth_capacity as u64)?;

            let mmap = unsafe {
                memmap2::MmapMut::map_mut(file)
                    .map_err(|e| ContainerError::MappingFailed(e.to_string()))?
            };

            self.mmap_data = Some(mmap);
            self.capacity = growth_capacity;
        } else {
            let mmap = unsafe {
                memmap2::MmapOptions::new()
                    .len(growth_capacity)
                    .map_anon()
                    .map_err(|e| ContainerError::MappingFailed(e.to_string()))?
            };

            if let Some(ref old_mmap) = self.mmap_data {
                let copy_len = old_mmap.len().min(growth_capacity);
                let mut new_mmap = mmap;
                new_mmap[..copy_len].copy_from_slice(&old_mmap[..copy_len]);
                self.mmap_data = Some(new_mmap);
            } else {
                self.mmap_data = Some(mmap);
            }
            self.capacity = growth_capacity;
        }

        self.size = new_size;
        self.update_header()?;
        Ok(())
    }

    fn close(&mut self) {
        if self.is_open.swap(false, Ordering::SeqCst) {
            if let Some(mmap) = self.mmap_data.take() {
                drop(mmap);
            }
            self.file = None;
            self.size = 0;
            self.capacity = 0;
        }
    }
}

impl Drop for MmapContainer {
    fn drop(&mut self) {
        self.close();
    }
}

/// File-backed shared mmap container (changes persist to file)
pub struct FileSharedMmap {
    inner: MmapContainer,
}

impl FileSharedMmap {
    pub fn open<P: AsRef<Path>>(path: P) -> ContainerResult<Self> {
        let inner = MmapContainer::open(path)?;
        Ok(Self { inner })
    }

    pub fn create<P: AsRef<Path>>(path: P, capacity: usize) -> ContainerResult<Self> {
        let inner = MmapContainer::create(path, capacity)?;
        Ok(Self { inner })
    }

    pub fn as_slice(&self) -> &[u8] {
        self.inner.as_slice()
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        self.inner.as_mut_slice()
    }
}

impl IDataContainer for FileSharedMmap {
    fn data(&self) -> *const u8 {
        self.inner.data()
    }

    fn data_mut(&mut self) -> *mut u8 {
        self.inner.data_mut()
    }

    fn size(&self) -> usize {
        self.inner.size()
    }

    fn is_open(&self) -> bool {
        self.inner.is_open()
    }

    fn sync(&self) -> ContainerResult<()> {
        self.inner.sync()
    }

    fn resize(&mut self, new_size: usize) -> ContainerResult<()> {
        self.inner.resize(new_size)
    }

    fn close(&mut self) {
        self.inner.close()
    }
}

/// Anonymous mmap container (memory-only, no file backing)
pub struct AnonMmap {
    inner: MmapContainer,
}

impl AnonMmap {
    pub fn new(capacity: usize) -> ContainerResult<Self> {
        let inner = MmapContainer::create_anonymous(capacity)?;
        Ok(Self { inner })
    }

    pub fn with_config(config: ContainerConfig) -> ContainerResult<Self> {
        let inner = MmapContainer::create_anonymous_with_config(config)?;
        Ok(Self { inner })
    }

    pub fn as_slice(&self) -> &[u8] {
        self.inner.as_slice()
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        self.inner.as_mut_slice()
    }
}

impl IDataContainer for AnonMmap {
    fn data(&self) -> *const u8 {
        self.inner.data()
    }

    fn data_mut(&mut self) -> *mut u8 {
        self.inner.data_mut()
    }

    fn size(&self) -> usize {
        self.inner.size()
    }

    fn is_open(&self) -> bool {
        self.inner.is_open()
    }

    fn sync(&self) -> ContainerResult<()> {
        Ok(())
    }

    fn resize(&mut self, new_size: usize) -> ContainerResult<()> {
        self.inner.resize(new_size)
    }

    fn close(&mut self) {
        self.inner.close()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_anon_mmap() {
        let mut container = AnonMmap::new(1024).expect("Failed to create container");
        assert!(container.is_open());

        container.as_mut_slice()[..5].copy_from_slice(b"hello");
        assert_eq!(&container.as_slice()[..5], b"hello");
    }

    #[test]
    fn test_file_mmap() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path().join("test.mmap");

        {
            let mut container =
                FileSharedMmap::create(&path, 1024).expect("Failed to create container");
            container.as_mut_slice()[..5].copy_from_slice(b"world");
            container.sync().expect("Failed to sync");
        }

        let container = FileSharedMmap::open(&path).expect("Failed to open container");
        assert_eq!(&container.as_slice()[..5], b"world");
    }

    #[test]
    fn test_resize() {
        let mut container = AnonMmap::new(100).expect("Failed to create container");
        assert!(container.resize(1000).is_ok());
        assert!(container.size() >= 1000);
    }
}
