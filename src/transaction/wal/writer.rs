//! WAL Writer
//!
//! Provides Write-Ahead Log writing functionality

use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

use super::types::{WalConfig, WalError, WalHeader, WalOpType, WalResult};

/// WAL writer trait
pub trait WalWriter: Send + Sync {
    /// Open the WAL
    fn open(&mut self) -> WalResult<()>;

    /// Close the WAL
    fn close(&mut self);

    /// Append data to the WAL
    fn append(&mut self, data: &[u8]) -> WalResult<bool>;

    /// Sync the WAL to disk
    fn sync(&self) -> WalResult<()>;
}

/// Local file-based WAL writer
pub struct LocalWalWriter {
    /// WAL URI/path
    wal_uri: String,
    /// Thread ID for this writer
    thread_id: u32,
    /// File handle
    file: Option<File>,
    /// File path
    file_path: Option<PathBuf>,
    /// Current file size
    file_size: usize,
    /// Current file used bytes
    file_used: usize,
    /// WAL version counter
    version: u32,
    /// Configuration
    config: WalConfig,
    /// Is open flag
    is_open: AtomicBool,
}

impl LocalWalWriter {
    /// Create a new local WAL writer
    pub fn new(wal_uri: &str, thread_id: u32) -> Self {
        Self {
            wal_uri: wal_uri.to_string(),
            thread_id,
            file: None,
            file_path: None,
            file_size: 0,
            file_used: 0,
            version: 0,
            config: WalConfig::default(),
            is_open: AtomicBool::new(false),
        }
    }

    /// Create with custom configuration
    pub fn with_config(wal_uri: &str, thread_id: u32, config: WalConfig) -> Self {
        Self {
            wal_uri: wal_uri.to_string(),
            thread_id,
            file: None,
            file_path: None,
            file_size: 0,
            file_used: 0,
            version: 0,
            config,
            is_open: AtomicBool::new(false),
        }
    }

    /// Get the WAL directory path
    fn get_wal_dir(&self) -> PathBuf {
        PathBuf::from(&self.wal_uri)
    }

    /// Find next available file path
    fn find_available_path(&self) -> WalResult<PathBuf> {
        let wal_dir = self.get_wal_dir();

        if !wal_dir.exists() {
            std::fs::create_dir_all(&wal_dir)
                .map_err(|e| WalError::IoError(e.to_string()))?;
        }

        for version in 0..65536 {
            let path = wal_dir.join(format!(
                "thread_{}_{}.wal",
                self.thread_id, version
            ));
            if !path.exists() {
                return Ok(path);
            }
        }

        Err(WalError::IoError("No available WAL file version".to_string()))
    }

    /// Append a WAL entry
    pub fn append_entry(
        &mut self,
        op_type: WalOpType,
        timestamp: u32,
        payload: &[u8],
    ) -> WalResult<bool> {
        if !self.is_open.load(Ordering::SeqCst) {
            return Err(WalError::Closed);
        }

        let header = WalHeader::new(op_type, timestamp, payload.len() as u32);
        let header_bytes = header.as_bytes();

        let file = self.file.as_mut().ok_or(WalError::Closed)?;
        let total_len = header_bytes.len() + payload.len();

        let expected_size = self.file_used + total_len;
        if expected_size > self.file_size {
            let new_size = ((expected_size / self.config.truncate_size) + 1)
                * self.config.truncate_size;
            file.set_len(new_size as u64)?;
            self.file_size = new_size;
        }

        use std::io::{Seek, SeekFrom, Write};
        file.seek(SeekFrom::End(0))?;
        file.write_all(header_bytes)?;
        file.write_all(payload)?;
        self.file_used += total_len;

        if self.config.sync_on_write {
            file.sync_data()?;
        }

        Ok(true)
    }

    /// Get current file size
    pub fn file_size(&self) -> usize {
        self.file_size
    }

    /// Get current file used bytes
    pub fn file_used(&self) -> usize {
        self.file_used
    }

    /// Rotate to a new file if needed
    fn rotate_if_needed(&mut self) -> WalResult<()> {
        if self.file_used >= self.config.max_file_size {
            self.rotate()?;
        }
        Ok(())
    }

    /// Rotate to a new file
    fn rotate(&mut self) -> WalResult<()> {
        if let Some(ref file) = self.file {
            file.sync_all()?;
        }

        self.file = None;
        self.file_path = None;
        self.file_size = 0;
        self.file_used = 0;

        let path = self.find_available_path()?;
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)?;

        file.set_len(self.config.truncate_size as u64)?;

        self.file = Some(file);
        self.file_path = Some(path);
        self.file_size = self.config.truncate_size;
        self.version += 1;

        Ok(())
    }
}

impl WalWriter for LocalWalWriter {
    fn open(&mut self) -> WalResult<()> {
        if self.is_open.load(Ordering::SeqCst) {
            return Ok(());
        }

        let path = self.find_available_path()?;
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)?;

        file.set_len(self.config.truncate_size as u64)?;

        self.file = Some(file);
        self.file_path = Some(path);
        self.file_size = self.config.truncate_size;
        self.file_used = 0;
        self.is_open.store(true, Ordering::SeqCst);

        Ok(())
    }

    fn close(&mut self) {
        if !self.is_open.swap(false, Ordering::SeqCst) {
            return;
        }

        if let Some(ref file) = self.file {
            let _ = file.sync_all();
        }

        self.file = None;
        self.file_path = None;
        self.file_size = 0;
        self.file_used = 0;
    }

    fn append(&mut self, data: &[u8]) -> WalResult<bool> {
        if !self.is_open.load(Ordering::SeqCst) {
            return Err(WalError::Closed);
        }

        let file = self.file.as_mut().ok_or(WalError::Closed)?;

        let expected_size = self.file_used + data.len();
        if expected_size > self.file_size {
            let new_size = ((expected_size / self.config.truncate_size) + 1)
                * self.config.truncate_size;
            file.set_len(new_size as u64)?;
            self.file_size = new_size;
        }

        file.seek(SeekFrom::End(0))?;
        file.write_all(data)?;
        self.file_used += data.len();

        if self.config.sync_on_write {
            file.sync_data()?;
        }

        Ok(true)
    }

    fn sync(&self) -> WalResult<()> {
        if let Some(ref file) = self.file {
            file.sync_all()?;
        }
        Ok(())
    }
}

impl Drop for LocalWalWriter {
    fn drop(&mut self) {
        self.close();
    }
}

/// Dummy WAL writer (no-op, for read-only mode)
pub struct DummyWalWriter {
    is_open: AtomicBool,
}

impl DummyWalWriter {
    pub fn new() -> Self {
        Self {
            is_open: AtomicBool::new(false),
        }
    }
}

impl Default for DummyWalWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl WalWriter for DummyWalWriter {
    fn open(&mut self) -> WalResult<()> {
        self.is_open.store(true, Ordering::SeqCst);
        Ok(())
    }

    fn close(&mut self) {
        self.is_open.store(false, Ordering::SeqCst);
    }

    fn append(&mut self, _data: &[u8]) -> WalResult<bool> {
        Ok(true)
    }

    fn sync(&self) -> WalResult<()> {
        Ok(())
    }
}

/// WAL writer factory
pub struct WalWriterFactory;

impl WalWriterFactory {
    /// Create a WAL writer based on the URI scheme
    pub fn create_wal_writer(wal_uri: &str, thread_id: u32) -> WalResult<Box<dyn WalWriter>> {
        let scheme = Self::get_scheme(wal_uri);

        match scheme.as_str() {
            "file" | "" => Ok(Box::new(LocalWalWriter::new(wal_uri, thread_id))),
            "dummy" => Ok(Box::new(DummyWalWriter::new())),
            _ => Err(WalError::IoError(format!(
                "Unknown WAL writer scheme: {}",
                scheme
            ))),
        }
    }

    /// Create a dummy WAL writer
    pub fn create_dummy_wal_writer() -> Box<dyn WalWriter> {
        Box::new(DummyWalWriter::new())
    }

    fn get_scheme(uri: &str) -> String {
        if let Some(pos) = uri.find("://") {
            uri[..pos].to_string()
        } else {
            "file".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_local_wal_writer() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let wal_path = temp_dir.path().to_string_lossy().to_string();

        let mut writer = LocalWalWriter::new(&wal_path, 0);
        writer.open().expect("Failed to open WAL");

        let header = WalHeader::new(WalOpType::InsertVertex, 1, 5);
        let mut data = header.as_bytes().to_vec();
        data.extend_from_slice(b"hello");

        writer.append(&data).expect("Failed to append");

        writer.sync().expect("Failed to sync");
        writer.close();
    }

    #[test]
    fn test_dummy_wal_writer() {
        let mut writer = DummyWalWriter::new();
        writer.open().expect("Failed to open");
        writer.append(b"test").expect("Failed to append");
        writer.sync().expect("Failed to sync");
        writer.close();
    }

    #[test]
    fn test_append_entry() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let wal_path = temp_dir.path().to_string_lossy().to_string();

        let mut writer = LocalWalWriter::new(&wal_path, 0);
        writer.open().expect("Failed to open WAL");

        writer
            .append_entry(WalOpType::InsertVertex, 1, b"payload")
            .expect("Failed to append entry");

        writer.close();
    }
}
