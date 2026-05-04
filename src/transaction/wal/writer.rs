//! WAL Writer
//!
//! Provides Write-Ahead Log writing functionality

use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::collections::VecDeque;

use super::types::{
    WalCompression, WalConfig, WalError, WalFileHeader, WalHeader, WalOpType, WalResult,
    WAL_FILE_HEADER_SIZE,
};

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

/// Pending write for group commit
struct PendingWrite {
    data: Vec<u8>,
    result: Arc<Mutex<WalResult<bool>>>,
    notified: Arc<Condvar>,
}

/// Group commit manager for batching multiple writes
pub struct GroupCommitManager {
    pending_writes: Mutex<VecDeque<PendingWrite>>,
    batch_size_limit: usize,
    commit_delay_us: u64,
    is_leader: AtomicBool,
}

impl GroupCommitManager {
    pub fn new(batch_size_limit: usize, commit_delay_us: u64) -> Self {
        Self {
            pending_writes: Mutex::new(VecDeque::new()),
            batch_size_limit,
            commit_delay_us,
            is_leader: AtomicBool::new(false),
        }
    }

    pub fn submit(&self, data: &[u8]) -> WalResult<bool> {
        let result = Arc::new(Mutex::new(Ok(false)));
        let notified = Arc::new(Condvar::new());
        
        let pending = PendingWrite {
            data: data.to_vec(),
            result: result.clone(),
            notified: notified.clone(),
        };

        {
            let mut queue = self.pending_writes.lock().map_err(|_| {
                WalError::IoError("Failed to lock pending writes".to_string())
            })?;
            queue.push_back(pending);
        }

        let mut result_guard = result.lock().map_err(|_| {
            WalError::IoError("Failed to lock result".to_string())
        })?;
        
        loop {
            if let Ok(true) = *result_guard {
                return Ok(true);
            }
            if let Err(ref e) = *result_guard {
                return Err(e.clone());
            }
            
            result_guard = notified
                .wait_timeout(result_guard, std::time::Duration::from_micros(self.commit_delay_us))
                .map_err(|_| WalError::IoError("Wait timeout error".to_string()))?
                .0;
        }
    }

    pub fn collect_batch(&self) -> Option<Vec<PendingWrite>> {
        let mut queue = self.pending_writes.lock().ok()?;
        
        if queue.is_empty() {
            return None;
        }

        let batch_size = queue.len().min(self.batch_size_limit);
        let batch: Vec<PendingWrite> = queue.drain(..batch_size).collect();
        Some(batch)
    }

    pub fn has_pending(&self) -> bool {
        self.pending_writes.lock().map(|q| !q.is_empty()).unwrap_or(false)
    }

    pub fn notify_results(batch: Vec<PendingWrite>, success: bool) {
        for pending in batch {
            if let Ok(mut result) = pending.result.lock() {
                *result = Ok(success);
            }
            pending.notified.notify_all();
        }
    }

    pub fn notify_error(batch: Vec<PendingWrite>, error: WalError) {
        for pending in batch {
            if let Ok(mut result) = pending.result.lock() {
                *result = Err(error.clone());
            }
            pending.notified.notify_all();
        }
    }
}

impl Default for GroupCommitManager {
    fn default() -> Self {
        Self::new(1024, 100)
    }
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
    /// Checkpoint sequence number
    checkpoint_seq: u64,
    /// Configuration
    config: WalConfig,
    /// Is open flag
    is_open: AtomicBool,
    /// WAL file header
    file_header: Option<WalFileHeader>,
    /// Group commit manager
    group_commit: Option<Arc<GroupCommitManager>>,
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
            checkpoint_seq: 0,
            config: WalConfig::default(),
            is_open: AtomicBool::new(false),
            file_header: None,
            group_commit: None,
        }
    }

    /// Create with custom configuration
    pub fn with_config(wal_uri: &str, thread_id: u32, config: WalConfig) -> Self {
        let group_commit = if config.group_commit_enabled {
            Some(Arc::new(GroupCommitManager::new(
                config.group_commit_batch_size,
                config.group_commit_delay_us,
            )))
        } else {
            None
        };
        
        Self {
            wal_uri: wal_uri.to_string(),
            thread_id,
            file: None,
            file_path: None,
            file_size: 0,
            file_used: 0,
            version: 0,
            checkpoint_seq: 0,
            config,
            is_open: AtomicBool::new(false),
            file_header: None,
            group_commit,
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

    /// Write WAL file header
    fn write_file_header(&mut self) -> WalResult<()> {
        let header = WalFileHeader::new(self.thread_id, self.checkpoint_seq);
        let header_bytes = header.as_bytes();
        
        let file = self.file.as_mut().ok_or(WalError::Closed)?;
        file.seek(SeekFrom::Start(0))?;
        file.write_all(header_bytes)?;
        file.sync_all()?;
        
        self.file_header = Some(header);
        self.file_used = WAL_FILE_HEADER_SIZE;
        
        Ok(())
    }

    /// Append a WAL entry with checksum
    pub fn append_entry(
        &mut self,
        op_type: WalOpType,
        timestamp: u32,
        payload: &[u8],
    ) -> WalResult<bool> {
        if !self.is_open.load(Ordering::SeqCst) {
            return Err(WalError::Closed);
        }

        let (final_payload, compression) = self.compress_payload(payload)?;
        let header = if self.config.checksum_enabled {
            WalHeader::new(op_type, timestamp, final_payload.len() as u32)
                .with_checksum(&final_payload)
                .with_compression(compression)
        } else {
            WalHeader::new(op_type, timestamp, final_payload.len() as u32)
                .with_compression(compression)
        };
        let header_bytes = header.as_bytes();

        let file = self.file.as_mut().ok_or(WalError::Closed)?;
        let total_len = header_bytes.len() + final_payload.len();

        let expected_size = self.file_used + total_len;
        if expected_size > self.file_size {
            let new_size = ((expected_size / self.config.truncate_size) + 1)
                * self.config.truncate_size;
            file.set_len(new_size as u64)?;
            self.file_size = new_size;
        }

        file.seek(SeekFrom::Start(self.file_used as u64))?;
        file.write_all(header_bytes)?;
        file.write_all(&final_payload)?;
        self.file_used += total_len;

        if self.config.sync_on_write {
            file.sync_data()?;
        }

        Ok(true)
    }

    /// Compress payload if compression is enabled
    fn compress_payload(&self, payload: &[u8]) -> WalResult<(Vec<u8>, WalCompression)> {
        if payload.len() < 64 {
            return Ok((payload.to_vec(), WalCompression::None));
        }

        match self.config.compression {
            WalCompression::Snappy => {
                #[cfg(feature = "compression-snappy")]
                {
                    let compressed = snap::raw::Encoder::new()
                        .compress_vec(payload)
                        .map_err(|e| WalError::SerializationError(e.to_string()))?;
                    
                    if compressed.len() < payload.len() {
                        return Ok((compressed, WalCompression::Snappy));
                    }
                }
                Ok((payload.to_vec(), WalCompression::None))
            }
            WalCompression::Zstd => {
                #[cfg(feature = "compression-zstd")]
                {
                    let compressed = zstd::encode_all(payload, 0)
                        .map_err(|e| WalError::SerializationError(e.to_string()))?;
                    
                    if compressed.len() < payload.len() {
                        return Ok((compressed, WalCompression::Zstd));
                    }
                }
                Ok((payload.to_vec(), WalCompression::None))
            }
            WalCompression::None => Ok((payload.to_vec(), WalCompression::None)),
        }
    }

    /// Decompress payload
    pub fn decompress_payload(payload: &[u8], compression: WalCompression) -> WalResult<Vec<u8>> {
        match compression {
            WalCompression::Snappy => {
                #[cfg(feature = "compression-snappy")]
                {
                    snap::raw::Decoder::new()
                        .decompress_vec(payload)
                        .map_err(|e| WalError::DeserializationError(e.to_string()))
                }
                #[cfg(not(feature = "compression-snappy"))]
                {
                    Err(WalError::DeserializationError(
                        "Snappy compression not enabled".to_string(),
                    ))
                }
            }
            WalCompression::Zstd => {
                #[cfg(feature = "compression-zstd")]
                {
                    zstd::decode_all(payload)
                        .map_err(|e| WalError::DeserializationError(e.to_string()))
                }
                #[cfg(not(feature = "compression-zstd"))]
                {
                    Err(WalError::DeserializationError(
                        "Zstd compression not enabled".to_string(),
                    ))
                }
            }
            WalCompression::None => Ok(payload.to_vec()),
        }
    }

    /// Append multiple entries as a batch (for group commit)
    pub fn append_batch(&mut self, entries: &[(WalOpType, u32, &[u8])]) -> WalResult<bool> {
        if !self.is_open.load(Ordering::SeqCst) {
            return Err(WalError::Closed);
        }

        let mut total_len = 0;
        let mut compressed_entries = Vec::with_capacity(entries.len());
        
        for (op_type, timestamp, payload) in entries {
            let (final_payload, compression) = self.compress_payload(payload)?;
            let header = if self.config.checksum_enabled {
                WalHeader::new(*op_type, *timestamp, final_payload.len() as u32)
                    .with_checksum(&final_payload)
                    .with_compression(compression)
            } else {
                WalHeader::new(*op_type, *timestamp, final_payload.len() as u32)
                    .with_compression(compression)
            };
            
            total_len += WalHeader::SIZE + final_payload.len();
            compressed_entries.push((header, final_payload));
        }

        let file = self.file.as_mut().ok_or(WalError::Closed)?;

        let expected_size = self.file_used + total_len;
        if expected_size > self.file_size {
            let new_size = ((expected_size / self.config.truncate_size) + 1)
                * self.config.truncate_size;
            file.set_len(new_size as u64)?;
            self.file_size = new_size;
        }

        file.seek(SeekFrom::Start(self.file_used as u64))?;

        for (header, payload) in compressed_entries {
            file.write_all(header.as_bytes())?;
            file.write_all(&payload)?;
        }
        
        self.file_used += total_len;
        file.sync_data()?;

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

    /// Get checkpoint sequence number
    pub fn checkpoint_seq(&self) -> u64 {
        self.checkpoint_seq
    }

    /// Set checkpoint sequence number
    pub fn set_checkpoint_seq(&mut self, seq: u64) {
        self.checkpoint_seq = seq;
    }

    /// Get the file header
    pub fn file_header(&self) -> Option<&WalFileHeader> {
        self.file_header.as_ref()
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
        self.file_header = None;

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
        
        self.write_file_header()?;

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
        
        self.write_file_header()?;

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
        self.file_header = None;
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

        file.seek(SeekFrom::Start(self.file_used as u64))?;
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
        
        assert!(writer.file_header().is_some());
        let header = writer.file_header().unwrap();
        assert!(header.is_valid());

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
    fn test_append_entry_with_checksum() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let wal_path = temp_dir.path().to_string_lossy().to_string();

        let config = WalConfig::new().with_checksum(true);
        let mut writer = LocalWalWriter::with_config(&wal_path, 0, config);
        writer.open().expect("Failed to open WAL");

        writer
            .append_entry(WalOpType::InsertVertex, 1, b"payload")
            .expect("Failed to append entry");

        assert!(writer.file_used() > WAL_FILE_HEADER_SIZE);
        writer.close();
    }

    #[test]
    fn test_append_batch() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let wal_path = temp_dir.path().to_string_lossy().to_string();

        let mut writer = LocalWalWriter::new(&wal_path, 0);
        writer.open().expect("Failed to open WAL");

        let entries: Vec<(WalOpType, u32, &[u8])> = vec![
            (WalOpType::InsertVertex, 1, b"vertex1"),
            (WalOpType::InsertVertex, 2, b"vertex2"),
            (WalOpType::InsertEdge, 3, b"edge1"),
        ];

        writer.append_batch(&entries).expect("Failed to append batch");
        writer.close();
    }

    #[test]
    fn test_group_commit_manager() {
        let manager = GroupCommitManager::new(10, 100);
        assert!(!manager.has_pending());
    }

    #[test]
    fn test_wal_file_header() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let wal_path = temp_dir.path().to_string_lossy().to_string();

        let mut writer = LocalWalWriter::new(&wal_path, 42);
        writer.open().expect("Failed to open WAL");

        let header = writer.file_header().expect("No file header");
        assert!(header.is_valid());
        assert_eq!(header.thread_id, 42);
        assert_eq!(header.checkpoint_seq, 0);

        writer.close();
    }
}
