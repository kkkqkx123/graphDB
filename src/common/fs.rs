use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

/// File attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAttributes {
    pub size: u64,
    pub created: SystemTime,
    pub modified: SystemTime,
    pub accessed: SystemTime,
    pub is_directory: bool,
    pub is_file: bool,
    pub is_symlink: bool,
    pub permissions: u32, // Simplified permissions representation
}

/// File system operations result
pub type FsResult<T> = Result<T, FsError>;

/// File system errors
#[derive(Debug, thiserror::Error)]
pub enum FsError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Path not found: {0}")]
    PathNotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("File already exists: {0}")]
    AlreadyExists(String),

    #[error("Not a directory: {0}")]
    NotADirectory(String),

    #[error("Not a file: {0}")]
    NotAFile(String),
}

/// File system utilities
pub struct FileSystemUtils;

impl FileSystemUtils {
    /// Check if a path exists
    pub fn exists<P: AsRef<Path>>(path: P) -> bool {
        Path::new(path.as_ref()).exists()
    }

    /// Check if a path is a file
    pub fn is_file<P: AsRef<Path>>(path: P) -> bool {
        Path::new(path.as_ref()).is_file()
    }

    /// Check if a path is a directory
    pub fn is_directory<P: AsRef<Path>>(path: P) -> bool {
        Path::new(path.as_ref()).is_dir()
    }

    /// Get file attributes
    pub fn get_attributes<P: AsRef<Path>>(path: P) -> FsResult<FileAttributes> {
        let metadata = fs::metadata(path.as_ref()).map_err(|e| FsError::IoError(e))?;

        Ok(FileAttributes {
            size: metadata.len(),
            created: match metadata.created() {
                Ok(time) => time,
                Err(_) => SystemTime::now(),
            },
            modified: match metadata.modified() {
                Ok(time) => time,
                Err(_) => SystemTime::now(),
            },
            accessed: match metadata.accessed() {
                Ok(time) => time,
                Err(_) => SystemTime::now(),
            },
            is_directory: metadata.is_dir(),
            is_file: metadata.is_file(),
            is_symlink: metadata.file_type().is_symlink(),
            permissions: metadata.permissions().readonly() as u32,
        })
    }

    /// Create a directory and all its parent directories if they don't exist
    pub fn create_dir_all<P: AsRef<Path>>(path: P) -> FsResult<()> {
        fs::create_dir_all(path.as_ref()).map_err(|e| FsError::IoError(e))
    }

    /// Create an empty file
    pub fn create_file<P: AsRef<Path>>(path: P) -> FsResult<File> {
        File::create(path.as_ref()).map_err(|e| FsError::IoError(e))
    }

    /// Read the entire contents of a file into a string
    pub fn read_to_string<P: AsRef<Path>>(path: P) -> FsResult<String> {
        fs::read_to_string(path.as_ref()).map_err(|e| FsError::IoError(e))
    }

    /// Read the entire contents of a file into a vector of bytes
    pub fn read<P: AsRef<Path>>(path: P) -> FsResult<Vec<u8>> {
        fs::read(path.as_ref()).map_err(|e| FsError::IoError(e))
    }

    /// Write a string to a file
    pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> FsResult<()> {
        fs::write(path.as_ref(), contents).map_err(|e| FsError::IoError(e))
    }

    /// Append a string to a file
    pub fn append<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> FsResult<()> {
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(path.as_ref())
            .map_err(|e| FsError::IoError(e))?;

        file.write_all(contents.as_ref())
            .map_err(|e| FsError::IoError(e))
    }

    /// Copy a file to a new location
    pub fn copy<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> FsResult<u64> {
        fs::copy(from.as_ref(), to.as_ref()).map_err(|e| FsError::IoError(e))
    }

    /// Remove a file
    pub fn remove_file<P: AsRef<Path>>(path: P) -> FsResult<()> {
        fs::remove_file(path.as_ref()).map_err(|e| FsError::IoError(e))
    }

    /// Remove a directory (must be empty)
    pub fn remove_dir<P: AsRef<Path>>(path: P) -> FsResult<()> {
        fs::remove_dir(path.as_ref()).map_err(|e| FsError::IoError(e))
    }

    /// Remove a directory and all its contents
    pub fn remove_dir_all<P: AsRef<Path>>(path: P) -> FsResult<()> {
        fs::remove_dir_all(path.as_ref()).map_err(|e| FsError::IoError(e))
    }

    /// Rename or move a file or directory
    pub fn rename<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> FsResult<()> {
        fs::rename(from.as_ref(), to.as_ref()).map_err(|e| FsError::IoError(e))
    }

    /// List all entries in a directory
    pub fn read_dir<P: AsRef<Path>>(path: P) -> FsResult<Vec<PathBuf>> {
        let mut entries = Vec::new();

        for entry in fs::read_dir(path.as_ref()).map_err(|e| FsError::IoError(e))? {
            let entry = entry.map_err(|e| FsError::IoError(e))?;
            entries.push(entry.path());
        }

        Ok(entries)
    }

    /// Walk a directory recursively and return all file paths
    pub fn walk_dir<P: AsRef<Path>>(path: P) -> FsResult<Vec<PathBuf>> {
        let mut paths = Vec::new();
        Self::walk_dir_impl(path.as_ref(), &mut paths)?;
        Ok(paths)
    }

    /// Internal implementation for recursive directory walk
    fn walk_dir_impl(dir: &Path, paths: &mut Vec<PathBuf>) -> FsResult<()> {
        for entry in fs::read_dir(dir).map_err(|e| FsError::IoError(e))? {
            let entry = entry.map_err(|e| FsError::IoError(e))?;
            let path = entry.path();

            if path.is_dir() {
                Self::walk_dir_impl(&path, paths)?;
            } else {
                paths.push(path);
            }
        }
        Ok(())
    }
}

/// A file handle with additional functionality
pub struct FileHandle {
    file: Arc<Mutex<File>>,
    path: PathBuf,
}

impl FileHandle {
    /// Open a file handle for reading and writing
    pub fn open<P: AsRef<Path>>(path: P) -> FsResult<Self> {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path.as_ref())
            .map_err(|e| FsError::IoError(e))?;

        Ok(Self {
            file: Arc::new(Mutex::new(file)),
            path: path.as_ref().to_path_buf(),
        })
    }

    /// Create a new file handle
    pub fn create<P: AsRef<Path>>(path: P) -> FsResult<Self> {
        let file = File::create(path.as_ref()).map_err(|e| FsError::IoError(e))?;

        Ok(Self {
            file: Arc::new(Mutex::new(file)),
            path: path.as_ref().to_path_buf(),
        })
    }

    /// Read from the file
    pub fn read(&self, buf: &mut [u8]) -> FsResult<usize> {
        let mut file = self
            .file
            .lock()
            .expect("File handle lock should not be poisoned");
        file.read(buf).map_err(|e| FsError::IoError(e))
    }

    /// Write to the file
    pub fn write(&self, buf: &[u8]) -> FsResult<usize> {
        let mut file = self
            .file
            .lock()
            .expect("File handle lock should not be poisoned");
        file.write(buf).map_err(|e| FsError::IoError(e))
    }

    /// Seek to a position in the file
    pub fn seek(&self, pos: SeekFrom) -> FsResult<u64> {
        let mut file = self
            .file
            .lock()
            .expect("File handle lock should not be poisoned");
        file.seek(pos).map_err(|e| FsError::IoError(e))
    }

    /// Get file attributes
    pub fn attributes(&self) -> FsResult<FileAttributes> {
        let metadata = self
            .file
            .lock()
            .expect("File handle lock should not be poisoned")
            .metadata()
            .map_err(|e| FsError::IoError(e))?;

        Ok(FileAttributes {
            size: metadata.len(),
            created: match metadata.created() {
                Ok(time) => time,
                Err(_) => SystemTime::now(),
            },
            modified: match metadata.modified() {
                Ok(time) => time,
                Err(_) => SystemTime::now(),
            },
            accessed: match metadata.accessed() {
                Ok(time) => time,
                Err(_) => SystemTime::now(),
            },
            is_directory: metadata.is_dir(),
            is_file: metadata.is_file(),
            is_symlink: metadata.file_type().is_symlink(),
            permissions: metadata.permissions().readonly() as u32,
        })
    }

    /// Synchronize the file to disk
    pub fn sync_all(&self) -> FsResult<()> {
        let file = self
            .file
            .lock()
            .expect("File handle lock should not be poisoned");
        file.sync_all().map_err(|e| FsError::IoError(e))
    }

    /// Get the file path
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// File lock for coordinating access to files
pub struct FileLock {
    file: File,
    path: PathBuf,
    is_exclusive: bool,
}

impl FileLock {
    /// Acquire an exclusive lock on a file
    pub fn acquire_exclusive<P: AsRef<Path>>(path: P) -> FsResult<Self> {
        Self::acquire_lock(path, true)
    }

    /// Acquire a shared lock on a file
    pub fn acquire_shared<P: AsRef<Path>>(path: P) -> FsResult<Self> {
        Self::acquire_lock(path, false)
    }

    /// Internal method to acquire a lock
    fn acquire_lock<P: AsRef<Path>>(path: P, exclusive: bool) -> FsResult<Self> {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path.as_ref())
            .map_err(|e| FsError::IoError(e))?;

        // Try to acquire the lock with a timeout
        let timeout = std::time::Duration::from_secs(30);
        let start = std::time::Instant::now();

        loop {
            match Self::try_lock(&file, exclusive) {
                Ok(_) => break,
                Err(_) => {
                    if start.elapsed() >= timeout {
                        return Err(FsError::PermissionDenied(format!(
                            "Failed to acquire lock on file: {}",
                            path.as_ref().display()
                        )));
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }
        }

        Ok(Self {
            file,
            path: path.as_ref().to_path_buf(),
            is_exclusive: exclusive,
        })
    }

    /// Try to acquire a lock without blocking
    fn try_lock(file: &File, exclusive: bool) -> FsResult<()> {
        #[cfg(unix)]
        {
            use std::os::unix::io::AsRawFd;
            let fd = file.as_raw_fd();

            let operation = if exclusive {
                libc::LOCK_EX | libc::LOCK_NB
            } else {
                libc::LOCK_SH | libc::LOCK_NB
            };

            let result = unsafe { libc::flock(fd, operation) };

            if result == 0 {
                Ok(())
            } else {
                Err(FsError::PermissionDenied(
                    "File is locked by another process".to_string(),
                ))
            }
        }

        #[cfg(windows)]
        {
            use std::os::windows::io::AsRawHandle;
            use winapi::um::fileapi::LockFileEx;
            use winapi::um::minwinbase::{LOCKFILE_EXCLUSIVE_LOCK, LOCKFILE_FAIL_IMMEDIATELY};
            use winapi::um::minwinbase::OVERLAPPED;

            let handle = file.as_raw_handle() as *mut winapi::ctypes::c_void;

            let mut overlapped: OVERLAPPED = unsafe { std::mem::zeroed() };
            let flags = if exclusive {
                LOCKFILE_EXCLUSIVE_LOCK | LOCKFILE_FAIL_IMMEDIATELY
            } else {
                LOCKFILE_FAIL_IMMEDIATELY
            };

            let result = unsafe {
                LockFileEx(
                    handle,
                    flags,
                    0,
                    0xFFFFFFFF,
                    0xFFFFFFFF,
                    &mut overlapped,
                )
            };

            if result != 0 {
                Ok(())
            } else {
                Err(FsError::PermissionDenied(
                    "File is locked by another process".to_string(),
                ))
            }
        }

        #[cfg(not(any(unix, windows)))]
        {
            Ok(())
        }
    }

    /// Release the lock
    pub fn release(&self) -> FsResult<()> {
        #[cfg(unix)]
        {
            use std::os::unix::io::AsRawFd;
            let fd = self.file.as_raw_fd();
            let result = unsafe { libc::flock(fd, libc::LOCK_UN) };

            if result == 0 {
                Ok(())
            } else {
                Err(FsError::IoError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Failed to release file lock",
                )))
            }
        }

        #[cfg(windows)]
        {
            use std::os::windows::io::AsRawHandle;
            use winapi::um::fileapi::UnlockFileEx;
            use winapi::um::minwinbase::OVERLAPPED;

            let handle = self.file.as_raw_handle() as *mut winapi::ctypes::c_void;
            let mut overlapped: OVERLAPPED = unsafe { std::mem::zeroed() };

            let result = unsafe {
                UnlockFileEx(
                    handle,
                    0,
                    0xFFFFFFFF,
                    0xFFFFFFFF,
                    &mut overlapped,
                )
            };

            if result != 0 {
                Ok(())
            } else {
                Err(FsError::IoError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Failed to release file lock",
                )))
            }
        }

        #[cfg(not(any(unix, windows)))]
        {
            Ok(())
        }
    }

    /// Get the file path
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Check if the lock is exclusive
    pub fn is_exclusive(&self) -> bool {
        self.is_exclusive
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        let _ = self.release();
    }
}

/// A simple file cache to keep frequently accessed files in memory
pub struct FileCache {
    cache: Arc<Mutex<HashMap<PathBuf, Vec<u8>>>>,
    max_size: usize,
    current_size: Arc<Mutex<usize>>,
}

impl FileCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            max_size,
            current_size: Arc::new(Mutex::new(0)),
        }
    }

    /// Get a file from the cache or load it if not present
    pub fn get_or_load<P: AsRef<Path>>(&self, path: P) -> FsResult<Vec<u8>> {
        let path_buf = path.as_ref().to_path_buf();

        // Check if it's in cache first
        {
            let cache = self
                .cache
                .lock()
                .expect("File cache lock should not be poisoned");
            if let Some(data) = cache.get(&path_buf) {
                return Ok(data.clone());
            }
        }

        // Not in cache, load from disk
        let data = FileSystemUtils::read(&path_buf)?;

        // Add to cache if it fits
        {
            let mut current_size = self
                .current_size
                .lock()
                .expect("File cache size lock should not be poisoned");
            if *current_size + data.len() <= self.max_size {
                let mut cache = self
                    .cache
                    .lock()
                    .expect("File cache lock should not be poisoned");
                cache.insert(path_buf, data.clone());
                *current_size += data.len();
            }
        }

        Ok(data)
    }

    /// Add a file to the cache
    pub fn put<P: AsRef<Path>>(&self, path: P, data: Vec<u8>) {
        let path_buf = path.as_ref().to_path_buf();

        let mut current_size = self
            .current_size
            .lock()
            .expect("File cache size lock should not be poisoned");
        if *current_size + data.len() <= self.max_size {
            let mut cache = self
                .cache
                .lock()
                .expect("File cache lock should not be poisoned");
            cache.insert(path_buf, data.clone());
            *current_size += data.len();
        }
    }

    /// Remove a file from the cache
    pub fn remove<P: AsRef<Path>>(&self, path: P) {
        let path_buf = path.as_ref().to_path_buf();

        let mut cache = self
            .cache
            .lock()
            .expect("File cache lock should not be poisoned");
        if let Some(data) = cache.remove(&path_buf) {
            let mut current_size = self
                .current_size
                .lock()
                .expect("File cache size lock should not be poisoned");
            *current_size -= data.len();
        }
    }

    /// Get the current cache size
    pub fn size(&self) -> usize {
        *self
            .current_size
            .lock()
            .expect("File cache size lock should not be poisoned")
    }

    /// Clear the cache
    pub fn clear(&self) {
        let mut cache = self
            .cache
            .lock()
            .expect("File cache lock should not be poisoned");
        cache.clear();
        *self
            .current_size
            .lock()
            .expect("File cache size lock should not be poisoned") = 0;
    }
}

/// File system watcher (simplified implementation)
pub struct FileSystemWatcher {
    watched_paths: Arc<Mutex<HashMap<PathBuf, SystemTime>>>,
    callback: Option<Arc<dyn Fn(&Path, FileEvent) + Send + Sync>>,
}

#[derive(Debug, Clone)]
pub enum FileEvent {
    Created,
    Modified,
    Deleted,
    Renamed,
}

impl FileSystemWatcher {
    pub fn new() -> Self {
        Self {
            watched_paths: Arc::new(Mutex::new(HashMap::new())),
            callback: None,
        }
    }

    /// Add a path to watch
    pub fn watch<P: AsRef<Path>>(&self, path: P) -> FsResult<()> {
        let path_buf = path.as_ref().to_path_buf();

        // Get the current modification time
        let metadata = fs::metadata(path.as_ref()).map_err(|e| FsError::IoError(e))?;
        let modified_time = metadata.modified().map_err(|_| {
            FsError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Could not get modification time",
            ))
        })?;

        // Store the path and its current modification time
        self.watched_paths
            .lock()
            .expect("File system watcher paths lock should not be poisoned")
            .insert(path_buf, modified_time);
        Ok(())
    }

    /// Remove a path from watching
    pub fn unwatch<P: AsRef<Path>>(&self, path: P) {
        let path_buf = path.as_ref().to_path_buf();
        self.watched_paths
            .lock()
            .expect("File system watcher paths lock should not be poisoned")
            .remove(&path_buf);
    }

    /// Check for changes (in a real implementation, this would run continuously)
    pub fn check_for_changes(&self) -> FsResult<Vec<(PathBuf, FileEvent)>> {
        let watched_paths = self
            .watched_paths
            .lock()
            .expect("File system watcher paths lock should not be poisoned");
        let mut changes = Vec::new();

        for (path, last_modified) in watched_paths.iter() {
            if let Ok(metadata) = fs::metadata(path) {
                if let Ok(current_modified) = metadata.modified() {
                    if &current_modified != last_modified {
                        changes.push((path.clone(), FileEvent::Modified));
                    }
                }
            } else {
                // File was deleted
                changes.push((path.clone(), FileEvent::Deleted));
            }
        }

        Ok(changes)
    }

    /// Set a callback function to be called when changes occur
    pub fn set_callback<F>(&mut self, callback: F)
    where
        F: Fn(&Path, FileEvent) + Send + Sync + 'static,
    {
        self.callback = Some(Arc::new(callback));
    }
}

/// File system configuration
#[derive(Debug, Clone)]
pub struct FileSystemConfig {
    pub default_permissions: u32,
    pub max_file_size: u64, // in bytes
    pub enable_caching: bool,
    pub cache_size: usize, // in bytes
    pub temp_dir: PathBuf,
    pub file_lock_timeout: std::time::Duration,
}

impl Default for FileSystemConfig {
    fn default() -> Self {
        Self {
            default_permissions: 0o644,       // rw-r--r--
            max_file_size: 100 * 1024 * 1024, // 100MB
            enable_caching: true,
            cache_size: 10 * 1024 * 1024, // 10MB
            temp_dir: std::env::temp_dir(),
            file_lock_timeout: std::time::Duration::from_secs(30),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_file_system_utils_exists() {
        // Create a temporary directory
        let dir = tempdir().expect("Failed to create temp directory");
        let file_path = dir.path().join("test_file.txt");

        // File shouldn't exist yet
        assert!(!FileSystemUtils::exists(&file_path));

        // Create the file
        FileSystemUtils::write(&file_path, "test content").expect("Failed to write file");

        // Now it should exist
        assert!(FileSystemUtils::exists(&file_path));
    }

    #[test]
    fn test_file_system_utils_read_write() {
        let dir = tempdir().expect("Failed to create temp directory");
        let file_path = dir.path().join("test_read_write.txt");

        // Write some content
        let content = "Hello, file system!";
        FileSystemUtils::write(&file_path, content).expect("Failed to write file");

        // Read it back
        let read_content =
            FileSystemUtils::read_to_string(&file_path).expect("Failed to read file");
        assert_eq!(content, read_content);
    }

    #[test]
    fn test_file_attributes() {
        let dir = tempdir().expect("Failed to create temp directory");
        let file_path = dir.path().join("test_attrs.txt");

        // Create a file
        FileSystemUtils::write(&file_path, "test").expect("Failed to write file");

        // Get its attributes
        let attrs =
            FileSystemUtils::get_attributes(&file_path).expect("Failed to get file attributes");

        assert!(attrs.is_file);
        assert!(!attrs.is_directory);
        assert_eq!(attrs.size, 4); // "test" is 4 bytes
    }

    #[test]
    fn test_file_handle() {
        let dir = tempdir().expect("Failed to create temp directory");
        let file_path = dir.path().join("test_handle.txt");

        // Create a file handle
        let handle = FileHandle::create(&file_path).expect("Failed to create file handle");

        // Write to the file
        let content = b"Hello from file handle!";
        handle
            .write(content)
            .expect("Failed to write to file handle");
        handle.sync_all().expect("Failed to sync file handle");

        // Read it back with standard fs
        let read_content = FileSystemUtils::read(&file_path).expect("Failed to read file");
        assert_eq!(content.to_vec(), read_content);
    }

    #[test]
    fn test_file_cache() {
        let dir = tempdir().expect("Failed to create temp directory");
        let file_path = dir.path().join("test_cache.txt");

        // Create and write to a file
        let content = b"Cache test content";
        FileSystemUtils::write(&file_path, content).expect("Failed to write file");

        // Create a cache
        let cache = FileCache::new(1024); // 1KB cache

        // Load the file into cache
        let cached_content = cache
            .get_or_load(&file_path)
            .expect("Failed to load file into cache");
        assert_eq!(content.to_vec(), cached_content);

        assert_eq!(cache.size(), content.len());
    }

    #[test]
    fn test_directory_operations() {
        let dir = tempdir().expect("Failed to create temp directory");
        let sub_dir = dir.path().join("subdir");

        // Create a subdirectory
        FileSystemUtils::create_dir_all(&sub_dir).expect("Failed to create subdirectory");
        assert!(FileSystemUtils::is_directory(&sub_dir));

        // Create a file in the subdirectory
        let file_path = sub_dir.join("nested_file.txt");
        FileSystemUtils::write(&file_path, "nested content").expect("Failed to write file");

        // List directory contents
        let entries = FileSystemUtils::read_dir(&sub_dir).expect("Failed to read directory");
        assert_eq!(entries.len(), 1);
        assert!(entries[0].ends_with("nested_file.txt"));

        // Walk directory recursively
        let all_paths = FileSystemUtils::walk_dir(&dir.path()).expect("Failed to walk directory");
        assert_eq!(all_paths.len(), 1);
        assert!(all_paths[0].ends_with("nested_file.txt"));
    }
}
