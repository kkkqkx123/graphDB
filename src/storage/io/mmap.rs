//! Memory-Mapped File Implementation
//!
//! Provides efficient memory-mapped I/O for storage operations.
//! This is a simpler, focused implementation for direct file I/O.

use std::fs::{File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use memmap2::MmapMut;

/// Default alignment for mmap operations
pub const DEFAULT_ALIGNMENT: usize = 4096;

/// Errors that can occur during mmap operations
#[derive(Debug)]
pub enum MmapFileError {
    /// I/O error
    IoError(io::Error),
    /// File not found
    FileNotFound(String),
    /// Invalid offset
    InvalidOffset { offset: usize, size: usize },
    /// Mapping failed
    MappingFailed(String),
    /// File already open
    AlreadyOpen,
    /// File not open
    NotOpen,
}

impl std::fmt::Display for MmapFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MmapFileError::IoError(e) => write!(f, "I/O error: {}", e),
            MmapFileError::FileNotFound(path) => write!(f, "File not found: {}", path),
            MmapFileError::InvalidOffset { offset, size } => {
                write!(f, "Invalid offset {} for size {}", offset, size)
            }
            MmapFileError::MappingFailed(msg) => write!(f, "Mapping failed: {}", msg),
            MmapFileError::AlreadyOpen => write!(f, "File is already open"),
            MmapFileError::NotOpen => write!(f, "File is not open"),
        }
    }
}

impl std::error::Error for MmapFileError {}

impl From<io::Error> for MmapFileError {
    fn from(e: io::Error) -> Self {
        MmapFileError::IoError(e)
    }
}

/// Options for creating a memory-mapped file
#[derive(Debug, Clone)]
pub struct MmapOptions {
    /// Initial file size in bytes
    pub initial_size: usize,
    /// Whether to create the file if it doesn't exist
    pub create: bool,
    /// Whether to truncate the file on create
    pub truncate: bool,
    /// Whether to use huge pages (Linux only)
    pub huge_pages: bool,
    /// Read access
    pub read: bool,
    /// Write access
    pub write: bool,
}

impl Default for MmapOptions {
    fn default() -> Self {
        Self {
            initial_size: 0,
            create: true,
            truncate: false,
            huge_pages: false,
            read: true,
            write: true,
        }
    }
}

impl MmapOptions {
    /// Create new mmap options with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the initial file size
    pub fn with_initial_size(mut self, size: usize) -> Self {
        self.initial_size = size;
        self
    }

    /// Set whether to create the file if it doesn't exist
    pub fn create(mut self, create: bool) -> Self {
        self.create = create;
        self
    }

    /// Set whether to truncate the file
    pub fn truncate(mut self, truncate: bool) -> Self {
        self.truncate = truncate;
        self
    }

    /// Set whether to use huge pages (Linux only)
    pub fn huge_pages(mut self, huge_pages: bool) -> Self {
        self.huge_pages = huge_pages;
        self
    }

    /// Set read access
    pub fn read(mut self, read: bool) -> Self {
        self.read = read;
        self
    }

    /// Set write access
    pub fn write(mut self, write: bool) -> Self {
        self.write = write;
        self
    }
}

/// Memory-mapped file wrapper
///
/// Provides a simple interface for memory-mapped file I/O operations.
/// Supports both file-backed and anonymous mappings.
pub struct MmapFile {
    /// File path (None for anonymous mapping)
    path: Option<PathBuf>,
    /// File handle
    file: Option<File>,
    /// Memory-mapped data
    mmap: Option<MmapMut>,
    /// Current size
    size: usize,
    /// Whether the file is open
    is_open: AtomicBool,
    /// Options used to create this mapping
    options: MmapOptions,
}

impl MmapFile {
    /// Open an existing file as a memory-mapped file
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, MmapFileError> {
        Self::open_with_options(path, MmapOptions::default().create(false))
    }

    /// Open a file with custom options
    pub fn open_with_options<P: AsRef<Path>>(
        path: P,
        options: MmapOptions,
    ) -> Result<Self, MmapFileError> {
        let path = path.as_ref().to_path_buf();

        if !path.exists() && !options.create {
            return Err(MmapFileError::FileNotFound(path.display().to_string()));
        }

        let mut open_options = OpenOptions::new();
        open_options.read(options.read).write(options.write);

        if options.create {
            open_options.create(true);
        }
        if options.truncate {
            open_options.truncate(true);
        }

        let file = open_options.open(&path)?;

        let metadata = file.metadata()?;
        let file_size = metadata.len() as usize;

        if file_size == 0 && options.initial_size > 0 {
            file.set_len(options.initial_size as u64)?;
        }

        let mmap = unsafe {
            MmapMut::map_mut(&file).map_err(|e| MmapFileError::MappingFailed(e.to_string()))?
        };

        let size = if file_size > 0 { file_size } else { options.initial_size };

        Ok(Self {
            path: Some(path),
            file: Some(file),
            mmap: Some(mmap),
            size,
            is_open: AtomicBool::new(true),
            options,
        })
    }

    /// Create a new file-backed memory-mapped file
    pub fn create<P: AsRef<Path>>(path: P, size: usize) -> Result<Self, MmapFileError> {
        Self::open_with_options(
            path,
            MmapOptions::new()
                .with_initial_size(size)
                .create(true)
                .truncate(true),
        )
    }

    /// Create an anonymous memory mapping (no file backing)
    pub fn create_anonymous(size: usize) -> Result<Self, MmapFileError> {
        let mmap = unsafe {
            memmap2::MmapOptions::new()
                .len(size)
                .map_anon()
                .map_err(|e| MmapFileError::MappingFailed(e.to_string()))?
        };

        Ok(Self {
            path: None,
            file: None,
            mmap: Some(mmap),
            size,
            is_open: AtomicBool::new(true),
            options: MmapOptions::default(),
        })
    }

    /// Get the file path
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Get the current size
    pub fn size(&self) -> usize {
        self.size
    }

    /// Check if the file is open
    pub fn is_open(&self) -> bool {
        self.is_open.load(Ordering::SeqCst)
    }

    /// Read data from the specified offset
    pub fn read(&self, offset: usize, len: usize) -> Result<&[u8], MmapFileError> {
        if !self.is_open() {
            return Err(MmapFileError::NotOpen);
        }

        if let Some(ref mmap) = self.mmap {
            let end = offset.saturating_add(len);
            if end > mmap.len() {
                return Err(MmapFileError::InvalidOffset {
                    offset,
                    size: mmap.len(),
                });
            }
            return Ok(&mmap[offset..end]);
        }

        Err(MmapFileError::NotOpen)
    }

    /// Write data at the specified offset
    pub fn write(&mut self, offset: usize, data: &[u8]) -> Result<(), MmapFileError> {
        if !self.is_open() {
            return Err(MmapFileError::NotOpen);
        }

        if let Some(ref mut mmap) = self.mmap {
            let end = offset.saturating_add(data.len());
            if end > mmap.len() {
                return Err(MmapFileError::InvalidOffset {
                    offset,
                    size: mmap.len(),
                });
            }
            mmap[offset..end].copy_from_slice(data);
            return Ok(());
        }

        Err(MmapFileError::NotOpen)
    }

    /// Get the entire data as a slice
    pub fn as_slice(&self) -> Result<&[u8], MmapFileError> {
        if !self.is_open() {
            return Err(MmapFileError::NotOpen);
        }

        if let Some(ref mmap) = self.mmap {
            return Ok(&mmap[..self.size]);
        }

        Err(MmapFileError::NotOpen)
    }

    /// Get the entire data as a mutable slice
    pub fn as_mut_slice(&mut self) -> Result<&mut [u8], MmapFileError> {
        if !self.is_open() {
            return Err(MmapFileError::NotOpen);
        }

        if let Some(ref mut mmap) = self.mmap {
            return Ok(&mut mmap[..self.size]);
        }

        Err(MmapFileError::NotOpen)
    }

    /// Flush data to disk
    pub fn flush(&self) -> Result<(), MmapFileError> {
        if let Some(ref mmap) = self.mmap {
            mmap.flush()?;
        }
        if let Some(ref file) = self.file {
            file.sync_all()?;
        }
        Ok(())
    }

    /// Flush a specific range to disk
    pub fn flush_range(&self, offset: usize, len: usize) -> Result<(), MmapFileError> {
        if let Some(ref mmap) = self.mmap {
            let end = offset.saturating_add(len).min(mmap.len());
            mmap.flush_async_range(offset, end - offset)?;
        }
        Ok(())
    }

    /// Resize the file
    pub fn resize(&mut self, new_size: usize) -> Result<(), MmapFileError> {
        if !self.is_open() {
            return Err(MmapFileError::NotOpen);
        }

        if let Some(ref file) = self.file {
            file.set_len(new_size as u64)?;

            let mmap = unsafe {
                MmapMut::map_mut(file).map_err(|e| MmapFileError::MappingFailed(e.to_string()))?
            };

            self.mmap = Some(mmap);
            self.size = new_size;
        } else {
            let mmap = unsafe {
                memmap2::MmapOptions::new()
                    .len(new_size)
                    .map_anon()
                    .map_err(|e| MmapFileError::MappingFailed(e.to_string()))?
            };

            if let Some(ref old_mmap) = self.mmap {
                let copy_len = old_mmap.len().min(new_size);
                let mut new_mmap = mmap;
                new_mmap[..copy_len].copy_from_slice(&old_mmap[..copy_len]);
                self.mmap = Some(new_mmap);
            } else {
                self.mmap = Some(mmap);
            }
            self.size = new_size;
        }

        Ok(())
    }

    /// Close the file
    pub fn close(&mut self) {
        if self.is_open.swap(false, Ordering::SeqCst) {
            if let Some(mmap) = self.mmap.take() {
                drop(mmap);
            }
            self.file = None;
            self.size = 0;
        }
    }
}

impl Drop for MmapFile {
    fn drop(&mut self) {
        self.close();
    }
}

impl std::fmt::Debug for MmapFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MmapFile")
            .field("path", &self.path)
            .field("size", &self.size)
            .field("is_open", &self.is_open.load(Ordering::SeqCst))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_and_write() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path().join("test.mmap");

        let mut mmap = MmapFile::create(&path, 1024).expect("Failed to create mmap");
        assert!(mmap.is_open());
        assert_eq!(mmap.size(), 1024);

        mmap.write(0, b"hello").expect("Failed to write");
        let data = mmap.read(0, 5).expect("Failed to read");
        assert_eq!(data, b"hello");
    }

    #[test]
    fn test_open_existing() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path().join("test.mmap");

        {
            let mut mmap = MmapFile::create(&path, 1024).expect("Failed to create mmap");
            mmap.write(0, b"world").expect("Failed to write");
            mmap.flush().expect("Failed to flush");
        }

        let mmap = MmapFile::open(&path).expect("Failed to open mmap");
        let data = mmap.read(0, 5).expect("Failed to read");
        assert_eq!(data, b"world");
    }

    #[test]
    fn test_anonymous_mmap() {
        let mut mmap = MmapFile::create_anonymous(1024).expect("Failed to create anonymous mmap");
        assert!(mmap.path().is_none());
        assert_eq!(mmap.size(), 1024);

        mmap.write(0, b"test").expect("Failed to write");
        let data = mmap.read(0, 4).expect("Failed to read");
        assert_eq!(data, b"test");
    }

    #[test]
    fn test_resize() {
        let mut mmap = MmapFile::create_anonymous(100).expect("Failed to create mmap");
        mmap.write(0, b"original").expect("Failed to write");

        mmap.resize(200).expect("Failed to resize");
        assert_eq!(mmap.size(), 200);

        let data = mmap.read(0, 8).expect("Failed to read");
        assert_eq!(data, b"original");
    }

    #[test]
    fn test_invalid_offset() {
        let mmap = MmapFile::create_anonymous(100).expect("Failed to create mmap");
        let result = mmap.read(90, 20);
        assert!(result.is_err());
    }

    #[test]
    fn test_options() {
        let options = MmapOptions::new()
            .with_initial_size(2048)
            .create(true)
            .huge_pages(true);

        assert_eq!(options.initial_size, 2048);
        assert!(options.create);
        assert!(options.huge_pages);
    }
}
