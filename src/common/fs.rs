use std::fs::File;
use std::path::{Path, PathBuf};

/// File system errors
#[derive(Debug, thiserror::Error)]
pub enum FsError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}

/// File lock for coordinating access to files
pub struct FileLock {
    file: File,
    path: PathBuf,
    is_exclusive: bool,
}

impl FileLock {
    pub fn acquire_exclusive<P: AsRef<Path>>(path: P) -> Result<Self, FsError> {
        Self::acquire_lock_nonblocking(path, true)
    }

    pub fn acquire_shared<P: AsRef<Path>>(path: P) -> Result<Self, FsError> {
        Self::acquire_lock_nonblocking(path, false)
    }

    fn acquire_lock_nonblocking<P: AsRef<Path>>(path: P, exclusive: bool) -> Result<Self, FsError> {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path.as_ref())
            .map_err(|e| FsError::IoError(e))?;

        let locked = Self::try_lock_nonblocking(&file, exclusive)?;

        if !locked {
            return Err(FsError::PermissionDenied(format!(
                "Failed to acquire lock on file: {}",
                path.as_ref().display()
            )));
        }

        Ok(Self {
            file,
            path: path.as_ref().to_path_buf(),
            is_exclusive: exclusive,
        })
    }

    fn try_lock_nonblocking(file: &File, exclusive: bool) -> Result<bool, FsError> {
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

            return if result == 0 {
                Ok(true)
            } else {
                Ok(false)
            };
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

            return Ok(result != 0);
        }

        #[cfg(not(any(unix, windows)))]
        {
            return Ok(true);
        }
    }

    pub fn try_lock(&self) -> Result<bool, FsError> {
        Self::try_lock_nonblocking(&self.file, self.is_exclusive)
    }

    pub fn try_lock_exclusive(&self) -> Result<bool, FsError> {
        Self::try_lock_nonblocking(&self.file, true)
    }

    pub fn try_lock_shared(&self) -> Result<bool, FsError> {
        Self::try_lock_nonblocking(&self.file, false)
    }

    /// Release the lock
    pub fn release(&self) -> Result<(), FsError> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_lock_exclusive() {
        let dir = tempfile::tempdir().expect("Failed to create temp directory");
        let file_path = dir.path().join("test_lock.txt");

        let lock = FileLock::acquire_exclusive(&file_path);
        assert!(lock.is_ok());
        assert!(lock.unwrap().is_exclusive());
    }

    #[test]
    fn test_file_lock_shared() {
        let dir = tempfile::tempdir().expect("Failed to create temp directory");
        let file_path = dir.path().join("test_lock_shared.txt");

        let lock = FileLock::acquire_shared(&file_path);
        assert!(lock.is_ok());
        assert!(!lock.unwrap().is_exclusive());
    }

    #[test]
    fn test_file_lock_try_lock() {
        let dir = tempfile::tempdir().expect("Failed to create temp directory");
        let file_path = dir.path().join("test_try_lock.txt");

        let lock = FileLock::acquire_exclusive(&file_path).expect("Failed to acquire lock");
        let try_result = lock.try_lock();
        assert!(try_result.is_ok());
        assert!(!try_result.unwrap());
    }

    #[test]
    fn test_file_lock_drop() {
        let dir = tempfile::tempdir().expect("Failed to create temp directory");
        let file_path = dir.path().join("test_lock_drop.txt");

        {
            let lock = FileLock::acquire_exclusive(&file_path);
            assert!(lock.is_ok());
        }
        let lock2 = FileLock::acquire_exclusive(&file_path);
        assert!(lock2.is_ok());
    }
}
