//! Persistent Container
//!
//! Memory-mapped file container for persistent storage.
//! This is the default storage backend for database operations.
//!
//! # Platform-Specific Behavior
//!
//! Memory map resizing is handled differently on each platform:
//!
//! - **Linux**: Uses `mremap(2)` for efficient in-place expansion
//! - **Windows**: Recreates the entire mapping with pre-allocation optimization
//! - **macOS**: Recreates the entire mapping with pre-allocation optimization
//!
//! # Data Integrity
//!
//! - File header stores magic number, version, data size, and checksum
//! - Checksum is computed using MD5 for data integrity verification
//! - Checksum computation is deferred to sync time for performance

use std::fs::{File, OpenOptions};
use std::io::Write as IoWrite;
use std::path::{Path, PathBuf};

use super::types::{
    ContainerConfig, ContainerError, ContainerResult, ContainerStats, FileHeader, StorageBackend,
};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "linux")]
use linux::resize_mmap;
#[cfg(target_os = "windows")]
use windows::resize_mmap;
#[cfg(target_os = "macos")]
use macos::resize_mmap;

/// Persistent container backed by memory-mapped file
///
/// Data is automatically synced to disk via mmap.
/// This is the default and recommended storage backend for database operations.
pub struct PersistentContainer {
    mmap: memmap2::MmapMut,
    file: File,
    path: PathBuf,
    size: usize,
    /// Total mmap capacity including FileHeader::SIZE
    mmap_capacity: usize,
    config: ContainerConfig,
    allocation_count: u64,
    /// Tracks whether data has been modified since last checksum update.
    /// Checksum is only recomputed on sync, verify, or close.
    dirty: bool,
}

impl PersistentContainer {
    /// Returns the data-only capacity (excluding header)
    fn data_capacity(&self) -> usize {
        self.mmap_capacity.saturating_sub(FileHeader::SIZE)
    }

    pub fn create<P: AsRef<Path>>(path: P, capacity: usize) -> ContainerResult<Self> {
        let result = Self::with_config(
            path,
            ContainerConfig {
                initial_capacity: capacity,
                storage_backend: StorageBackend::Persistent,
                ..Default::default()
            },
        );
        if result.is_ok() {
            log::info!("Created persistent container with capacity {} bytes", capacity);
        }
        result
    }

    pub fn with_config<P: AsRef<Path>>(path: P, config: ContainerConfig) -> ContainerResult<Self> {
        let path = path.as_ref().to_path_buf();
        
        let growth_factor = config.growth_factor.max(1.0);
        let preallocated_size = ((config.initial_capacity as f64) * growth_factor) as usize;
        let total_size = FileHeader::SIZE + preallocated_size;

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

        // Initialize header with zero checksum (deferred computation)
        let header = FileHeader::new(0);
        let mut mmap = mmap;
        mmap[..FileHeader::SIZE].copy_from_slice(header.as_bytes());

        Ok(Self {
            mmap,
            file,
            path,
            size: 0,
            mmap_capacity: total_size,
            config,
            allocation_count: 1,
            dirty: false,
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

        log::info!("Opened persistent container from {} ({} bytes data)", path.display(), size);

        Ok(Self {
            mmap,
            file,
            path,
            size,
            mmap_capacity: file_size,
            config: ContainerConfig::default(),
            allocation_count: 1,
            dirty: false,
        })
    }

    /// Write data at offset (internal method, offset is data-relative)
    pub fn write_at(&mut self, offset: usize, data: &[u8]) -> ContainerResult<()> {
        if data.is_empty() {
            return Ok(());
        }

        let start = FileHeader::SIZE + offset;
        let end = start + data.len();

        if end > self.mmap_capacity {
            self.do_resize(end - FileHeader::SIZE)?;
        }

        self.mmap[start..end].copy_from_slice(data);
        if offset + data.len() > self.size {
            self.size = offset + data.len();
        }
        self.dirty = true;
        Ok(())
    }

    /// Read data at offset (internal method, offset is data-relative)
    pub fn read_at(&self, offset: usize, len: usize) -> ContainerResult<Vec<u8>> {
        if offset + len > self.size {
            return Err(ContainerError::InvalidSize(format!(
                "Read of {} bytes at offset {} exceeds size {}",
                len, offset, self.size
            )));
        }

        let start = FileHeader::SIZE + offset;
        let end = start + len;
        Ok(self.mmap[start..end].to_vec())
    }

    /// Update checksum in header if data is dirty, then clear dirty flag
    fn sync_checksum(&mut self) -> ContainerResult<()> {
        if !self.dirty {
            return Ok(());
        }
        if self.size > 0 {
            let data = &self.mmap[FileHeader::SIZE..FileHeader::SIZE + self.size];
            let header = FileHeader::with_checksum(self.size as u64, data);
            self.mmap[..FileHeader::SIZE].copy_from_slice(header.as_bytes());
        }
        self.dirty = false;
        Ok(())
    }

    /// Flush mmap to disk (read-only, no checksum update)
    fn do_sync(&self) -> ContainerResult<()> {
        self.mmap.flush()?;
        self.file.sync_all()?;
        Ok(())
    }

    /// Sync checksum and flush to disk (mutating)
    fn do_sync_with_checksum(&mut self) -> ContainerResult<()> {
        self.sync_checksum()?;
        self.mmap.flush()?;
        self.file.sync_all()?;
        Ok(())
    }

    /// Verify data integrity using checksum
    pub fn verify_integrity(&self) -> ContainerResult<()> {
        if self.size == 0 {
            return Ok(());
        }

        let stored_header = FileHeader::from_bytes(&self.mmap[..FileHeader::SIZE])
            .ok_or_else(|| ContainerError::InvalidHeader("Failed to parse header".to_string()))?;

        if stored_header.has_valid_checksum() {
            let data = &self.mmap[FileHeader::SIZE..FileHeader::SIZE + self.size];
            if !stored_header.verify_checksum(data) {
                return Err(ContainerError::ChecksumMismatch);
            }
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

        let header = if self.dirty {
            // Compute fresh checksum for export
            let data = &self.mmap[FileHeader::SIZE..FileHeader::SIZE + self.size];
            FileHeader::with_checksum(self.size as u64, data)
        } else {
            // Use existing header
            FileHeader::from_bytes(&self.mmap[..FileHeader::SIZE])
                .unwrap_or_else(|| FileHeader::new(self.size as u64))
        };
        file.write_all(header.as_bytes())?;

        if self.size > 0 {
            file.write_all(&self.mmap[FileHeader::SIZE..FileHeader::SIZE + self.size])?;
        }

        file.sync_all()?;
        Ok(())
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn do_resize(&mut self, new_size: usize) -> ContainerResult<()> {
        let new_capacity = FileHeader::SIZE + new_size;

        if new_capacity <= self.mmap_capacity {
            self.size = new_size;
            self.dirty = true;
            return Ok(());
        }

        if self.config.max_capacity > 0 && new_size > self.config.max_capacity {
            return Err(ContainerError::InvalidSize(
                "Exceeds maximum capacity".to_string(),
            ));
        }

        let growth_capacity =
            ((self.mmap_capacity as f64 * self.config.growth_factor) as usize).max(new_capacity);

        log::info!(
            "Resizing mmap from {} to {} bytes",
            self.mmap_capacity, growth_capacity
        );

        if let Err(e) = self.file.set_len(growth_capacity as u64) {
            return match e.kind() {
                std::io::ErrorKind::StorageFull => {
                    log::error!("Disk full when resizing to {} bytes", growth_capacity);
                    Err(ContainerError::DiskFull)
                }
                std::io::ErrorKind::PermissionDenied => {
                    log::error!("Permission denied when resizing: {}", e);
                    Err(ContainerError::PermissionDenied(e.to_string()))
                }
                _ => {
                    log::error!("IO error when resizing: {}", e);
                    Err(ContainerError::IoError(e))
                }
            };
        }

        resize_mmap(&mut self.mmap, &self.file, growth_capacity)
            .map_err(|e| {
                log::error!("Failed to remap mmap to {} bytes: {}", growth_capacity, e);
                e
            })?;

        self.mmap_capacity = growth_capacity;
        self.size = new_size;
        self.allocation_count += 1;
        self.dirty = true;
        Ok(())
    }

    fn do_close(&mut self) {
        if let Err(e) = self.do_sync_with_checksum() {
            log::warn!("Failed to sync before close: {}", e);
        }
        self.size = 0;
        self.mmap_capacity = 0;
    }
}

impl super::IDataContainer for PersistentContainer {
    fn data(&self) -> *const u8 {
        if self.mmap.len() > FileHeader::SIZE {
            return self.mmap[FileHeader::SIZE..].as_ptr();
        }
        std::ptr::null()
    }

    fn data_mut(&mut self) -> *mut u8 {
        if self.mmap.len() > FileHeader::SIZE {
            return self.mmap[FileHeader::SIZE..].as_mut_ptr();
        }
        std::ptr::null_mut()
    }

    fn size(&self) -> usize {
        self.size
    }

    fn capacity(&self) -> usize {
        // Return data-only capacity (excluding header) for consistency with VolatileContainer
        self.data_capacity()
    }

    fn is_open(&self) -> bool {
        self.mmap_capacity > 0
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
            capacity: self.data_capacity(),
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

    fn verify_integrity(&self) -> ContainerResult<()> {
        PersistentContainer::verify_integrity(self)
    }

    fn write_at(&mut self, offset: usize, buf: &[u8]) -> ContainerResult<()> {
        // Delegate to the direct method which handles FileHeader offset internally
        self.write_at(offset, buf)
    }

    fn read_at(&self, offset: usize, len: usize) -> ContainerResult<Vec<u8>> {
        // Delegate to the direct method which handles FileHeader offset internally
        self.read_at(offset, len)
    }

    fn write_batch(&mut self, operations: &[(usize, &[u8])]) -> ContainerResult<usize> {
        if operations.is_empty() {
            return Ok(0);
        }

        // Find the maximum offset to determine if resize is needed
        let max_end = operations
            .iter()
            .map(|(offset, data)| offset + data.len())
            .max()
            .unwrap_or(0);

        // Resize if needed (only once)
        if max_end > self.size {
            self.do_resize(max_end)?;
        }

        // Perform all writes
        let mut total_written = 0;
        for (offset, data) in operations {
            if !data.is_empty() {
                let start = FileHeader::SIZE + offset;
                let end = start + data.len();
                self.mmap[start..end].copy_from_slice(data);
                total_written += data.len();
            }
        }

        // Update size and mark dirty (checksum deferred)
        self.size = max_end;
        self.dirty = true;

        Ok(total_written)
    }

    fn read_batch(&self, operations: &[(usize, usize)]) -> ContainerResult<Vec<Vec<u8>>> {
        let mut results = Vec::with_capacity(operations.len());
        for (offset, len) in operations {
            let start = FileHeader::SIZE + offset;
            let end = start + len;
            if end > self.size + FileHeader::SIZE {
                return Err(ContainerError::InvalidSize(format!(
                    "Read at offset {} with len {} exceeds size {}",
                    offset,
                    len,
                    self.size
                )));
            }
            results.push(self.mmap[start..end].to_vec());
        }
        Ok(results)
    }
}

impl Drop for PersistentContainer {
    fn drop(&mut self) {
        self.do_close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::storage::container::container_trait::IDataContainer;

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
        // capacity() now returns data-only capacity (without header)
        assert!(container.capacity() >= 1000);
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
    fn test_persistent_container_checksum_verification() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path().join("test_checksum.mmap");

        {
            let mut container =
                PersistentContainer::create(&path, 1024).expect("Failed to create container");
            container.write_at(0, b"test data for checksum").expect("Failed to write");
            container.sync().expect("Failed to sync");
        }

        let container = PersistentContainer::open(&path).expect("Failed to open container");
        container.verify_integrity().expect("Checksum verification failed");
    }

    #[test]
    fn test_persistent_container_preallocation() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path().join("test_prealloc.mmap");

        let container =
            PersistentContainer::with_config(
                &path,
                ContainerConfig {
                    initial_capacity: 1024,
                    growth_factor: 2.0,
                    storage_backend: StorageBackend::Persistent,
                    ..Default::default()
                },
            ).expect("Failed to create container");

        // capacity() returns data-only capacity
        assert!(container.capacity() >= 1024);
    }

    #[test]
    fn test_persistent_container_send_sync() {
        fn assert_send<T: Send>() {}
        assert_send::<PersistentContainer>();
    }

    #[test]
    fn test_persistent_container_batch_write() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path().join("test_batch.mmap");

        let mut container =
            PersistentContainer::create(&path, 1024).expect("Failed to create container");

        let operations = vec![
            (0, b"first".as_slice()),
            (10, b"second".as_slice()),
            (20, b"third".as_slice()),
        ];

        let written = container.write_batch(&operations).expect("Batch write failed");
        assert_eq!(written, 15); // 5 + 6 + 4

        let results = container
            .read_batch(&[(0, 5), (10, 6), (20, 5)])
            .expect("Batch read failed");
        assert_eq!(&results[0], b"first");
        assert_eq!(&results[1], b"second");
        assert_eq!(&results[2], b"third");
    }

    #[test]
    fn test_persistent_container_deferred_checksum() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path().join("test_deferred.mmap");

        let mut container =
            PersistentContainer::create(&path, 1024).expect("Failed to create container");
        assert!(!container.dirty, "Fresh container should not be dirty");

        container.write_at(0, b"data").expect("Failed to write");
        assert!(container.dirty, "After write, container should be dirty");

        container.sync().expect("Failed to sync");
        assert!(!container.dirty, "After sync, container should not be dirty");

        let reopened = PersistentContainer::open(&path).expect("Failed to reopen");
        reopened.verify_integrity().expect("Checksum should be valid");
    }
}