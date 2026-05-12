//! Local file-based WAL writer

use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::compression::{self as compression_mod, create_compressor, Compressor};
use super::group_commit::GroupCommitManager;
use super::sync::{elapsed_since, should_sync};
use super::traits::WalWriter;
use crate::transaction::wal::types::{
    ArchiveMode, Lsn, RecordType, WalCompression, WalConfig, WalError, WalFileHeader, WalHeader,
    WalOpType, WalResult, WalStats, WAL_FILE_HEADER_SIZE, WAL_HEADER_SIZE, WAL_MAX_RECORD_SIZE,
};

/// Local file-based WAL writer
pub struct LocalWalWriter {
    wal_uri: String,
    thread_id: u32,
    file: Option<File>,
    file_path: Option<PathBuf>,
    file_size: usize,
    file_used: usize,
    version: u32,
    checkpoint_seq: u64,
    current_lsn: AtomicU64,
    last_synced_lsn: AtomicU64,
    file_start_lsn: Lsn,
    lsn_since_checkpoint: u64,
    last_cleanup_time: Option<Instant>,
    writes_since_cleanup: u64,
    stats: WalStats,
    config: WalConfig,
    is_open: AtomicBool,
    file_header: Option<WalFileHeader>,
    compressor: Box<dyn Compressor>,
    group_commit: Option<Arc<GroupCommitManager>>,
    write_count: AtomicU64,
    last_sync_time: Mutex<Option<Instant>>,
}

impl LocalWalWriter {
    /// Create a new local WAL writer
    pub fn new(wal_uri: &str, thread_id: u32) -> Self {
        let config = WalConfig::default();
        let compressor = create_compressor(&config);
        Self {
            wal_uri: wal_uri.to_string(),
            thread_id,
            file: None,
            file_path: None,
            file_size: 0,
            file_used: 0,
            version: 0,
            checkpoint_seq: 0,
            current_lsn: AtomicU64::new(0),
            last_synced_lsn: AtomicU64::new(0),
            file_start_lsn: Lsn::ZERO,
            lsn_since_checkpoint: 0,
            last_cleanup_time: None,
            writes_since_cleanup: 0,
            stats: WalStats::new(),
            config,
            is_open: AtomicBool::new(false),
            file_header: None,
            compressor,
            group_commit: None,
            write_count: AtomicU64::new(0),
            last_sync_time: Mutex::new(None),
        }
    }

    /// Create with custom configuration
    pub fn with_config(wal_uri: &str, thread_id: u32, config: WalConfig) -> Self {
        let compressor = create_compressor(&config);
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
            current_lsn: AtomicU64::new(0),
            last_synced_lsn: AtomicU64::new(0),
            file_start_lsn: Lsn::ZERO,
            lsn_since_checkpoint: 0,
            last_cleanup_time: None,
            writes_since_cleanup: 0,
            stats: WalStats::new(),
            config,
            is_open: AtomicBool::new(false),
            file_header: None,
            compressor,
            group_commit,
            write_count: AtomicU64::new(0),
            last_sync_time: Mutex::new(None),
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
            std::fs::create_dir_all(&wal_dir).map_err(|e| WalError::IoError(e.to_string()))?;
        }

        for version in self.version..65536 {
            let path = self.get_wal_file_path(version);
            if !path.exists() {
                return Ok(path);
            }
        }

        Err(WalError::IoError(
            "No available WAL file version".to_string(),
        ))
    }

    /// Write WAL file header
    fn write_file_header(&mut self) -> WalResult<()> {
        let current_lsn = Lsn::new(self.current_lsn.load(Ordering::SeqCst));
        let header = WalFileHeader::new(self.thread_id, self.checkpoint_seq, current_lsn);
        let header_bytes = header.as_bytes();

        let file = self.file.as_mut().ok_or(WalError::Closed)?;
        file.seek(SeekFrom::Start(0))?;
        file.write_all(header_bytes)?;
        file.sync_all()?;

        self.file_header = Some(header);
        self.file_start_lsn = current_lsn;
        self.file_used = WAL_FILE_HEADER_SIZE;

        Ok(())
    }

    /// Generate WAL file path for a given version
    fn get_wal_file_path(&self, version: u32) -> PathBuf {
        PathBuf::from(&self.wal_uri)
            .join(format!("thread_{}_wal_{:08X}", self.thread_id, version))
    }

    /// List all WAL files in the directory
    fn list_wal_files(&self) -> WalResult<Vec<PathBuf>> {
        let wal_dir = self.get_wal_dir();

        if !wal_dir.exists() {
            return Ok(Vec::new());
        }

        let mut files = Vec::new();
        for entry in std::fs::read_dir(&wal_dir)? {
            let entry = entry?;
            let path = entry.path();

            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("wal_") && name.len() == 12 {
                    files.push(path);
                }
            }
        }

        Ok(files)
    }

    /// Get total size of all WAL files
    fn get_total_wal_size(&self) -> WalResult<usize> {
        let mut total = 0;
        for file in self.list_wal_files()? {
            if let Ok(metadata) = std::fs::metadata(&file) {
                total += metadata.len() as usize;
            }
        }
        Ok(total)
    }

    /// Check if rotation is needed
    fn rotate_if_needed(&mut self) -> WalResult<()> {
        if self.file_used >= self.config.max_file_size {
            self.rotate()?;
        }
        Ok(())
    }

    /// Rotate to a new WAL file
    fn rotate(&mut self) -> WalResult<()> {
        log::info!(
            "Rotating WAL file: used={}, max_size={}, version={}",
            self.file_used,
            self.config.max_file_size,
            self.version
        );

        if let Some(ref file) = self.file {
            file.sync_all()?;
        }

        self.version += 1;

        let new_path = self.get_wal_file_path(self.version);

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&new_path)?;

        file.set_len(self.config.truncate_size as u64)?;

        self.file = Some(file);
        self.file_path = Some(new_path);
        self.file_size = self.config.truncate_size;
        self.file_used = 0;
        self.file_start_lsn = Lsn::new(self.current_lsn.load(Ordering::SeqCst));

        self.write_file_header()?;

        // Record rotation statistics
        self.stats.record_rotation();

        log::info!(
            "WAL rotated to version {}, file: {:?}, start_lsn={}",
            self.version,
            self.file_path,
            self.file_start_lsn
        );

        Ok(())
    }

    /// Delete or archive a WAL file based on configuration
    fn delete_or_archive_file(&mut self, file: &Path) -> WalResult<()> {
        if let Some(ref archive_dir) = self.config.archive_dir {
            match self.config.archive_mode {
                ArchiveMode::None => {
                    std::fs::remove_file(file)?;
                    self.stats.record_file_deleted();
                }
                ArchiveMode::Move => {
                    self.archive_wal_file(file, archive_dir)?;
                    self.stats.record_file_archived();
                }
                ArchiveMode::Copy => {
                    self.copy_and_delete(file, archive_dir)?;
                    self.stats.record_file_archived();
                }
            }
        } else {
            std::fs::remove_file(file)?;
            self.stats.record_file_deleted();
        }
        Ok(())
    }

    /// Archive a WAL file to the archive directory
    fn archive_wal_file(&self, file: &Path, archive_dir: &str) -> WalResult<()> {
        std::fs::create_dir_all(archive_dir)
            .map_err(|e| WalError::IoError(e.to_string()))?;

        let file_name = file
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let archive_name = format!("{}_{}", file_name, timestamp);
        let archive_path = PathBuf::from(archive_dir).join(archive_name);

        std::fs::rename(file, &archive_path)
            .map_err(|e| WalError::IoError(e.to_string()))?;

        log::debug!("Archived WAL file: {:?} -> {:?}", file, archive_path);

        Ok(())
    }

    /// Copy a file and delete the original
    fn copy_and_delete(&self, file: &Path, archive_dir: &str) -> WalResult<()> {
        std::fs::create_dir_all(archive_dir)
            .map_err(|e| WalError::IoError(e.to_string()))?;

        let file_name = file
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let archive_path = PathBuf::from(archive_dir).join(file_name);

        std::fs::copy(file, &archive_path)
            .map_err(|e| WalError::IoError(e.to_string()))?;

        std::fs::remove_file(file)?;

        log::debug!("Copied and deleted WAL file: {:?} -> {:?}", file, archive_path);

        Ok(())
    }

    /// Clean up old WAL files based on size and TTL
    fn cleanup_old_wal_files(&mut self) -> WalResult<usize> {
        let now = Instant::now();
        if let Some(last_time) = self.last_cleanup_time {
            if now.duration_since(last_time) < Duration::from_secs(1) {
                return Ok(0);
            }
        }

        if self.writes_since_cleanup < 100 {
            return Ok(0);
        }

        let mut deleted_count = 0;

        let mut wal_files = self.list_wal_files()?;

        if wal_files.is_empty() {
            self.writes_since_cleanup = 0;
            return Ok(0);
        }

        wal_files.sort();

        if self.config.max_total_size > 0 {
            let total_size = self.get_total_wal_size()?;

            if total_size > self.config.max_total_size {
                let mut current_size = total_size;

                for file in &wal_files {
                    if current_size <= self.config.max_total_size {
                        break;
                    }

                    let file_size = std::fs::metadata(file)?.len() as usize;

                    self.delete_or_archive_file(file)?;

                    current_size -= file_size;
                    deleted_count += 1;
                }
            }
        }

        if self.config.ttl_seconds > 0 {
            let ttl = Duration::from_secs(self.config.ttl_seconds);

            for file in &wal_files {
                if let Ok(metadata) = std::fs::metadata(file) {
                    if let Ok(modified) = metadata.modified() {
                        if modified.elapsed().unwrap_or(Duration::from_secs(0)) > ttl {
                            self.delete_or_archive_file(file)?;
                            deleted_count += 1;
                        }
                    }
                }
            }
        }

        if deleted_count > 0 {
            log::info!("Cleaned up {} old WAL files", deleted_count);
        }

        self.last_cleanup_time = Some(Instant::now());
        self.writes_since_cleanup = 0;

        Ok(deleted_count)
    }

    /// Check if auto-checkpoint should be triggered
    fn maybe_trigger_checkpoint(&mut self) -> WalResult<()> {
        if !self.config.auto_checkpoint {
            return Ok(());
        }

        self.lsn_since_checkpoint += 1;

        if self.lsn_since_checkpoint >= self.config.checkpoint_interval {
            log::debug!(
                "Triggering auto-checkpoint at LSN {}",
                self.current_lsn.load(Ordering::SeqCst)
            );

            self.lsn_since_checkpoint = 0;
        }

        Ok(())
    }

    /// Append a WAL entry with checksum and LSN
    pub fn append_entry(
        &mut self,
        op_type: WalOpType,
        timestamp: u32,
        payload: &[u8],
    ) -> WalResult<bool> {
        if !self.is_open.load(Ordering::SeqCst) {
            return Err(WalError::Closed);
        }

        let (final_payload, compression) = self.compressor.compress(payload)?;

        if final_payload.len() > WAL_MAX_RECORD_SIZE {
            return self.append_fragmented_entry(op_type, timestamp, &final_payload, compression);
        }

        self.append_single_entry(op_type, timestamp, &final_payload, compression)
    }

    /// Append a single (non-fragmented) WAL entry
    fn append_single_entry(
        &mut self,
        op_type: WalOpType,
        timestamp: u32,
        payload: &[u8],
        compression: WalCompression,
    ) -> WalResult<bool> {
        let prev_lsn = Lsn::new(self.current_lsn.load(Ordering::SeqCst));
        let entry_size = WAL_HEADER_SIZE + payload.len();
        let new_lsn = Lsn::new(prev_lsn.as_u64() + entry_size as u64);

        let header = if self.config.checksum_enabled {
            WalHeader::new(op_type, timestamp, payload.len() as u32)
                .with_lsn(new_lsn, prev_lsn)
                .with_record_type(RecordType::Full)
                .with_checksum(payload)
                .with_compression(compression)
        } else {
            WalHeader::new(op_type, timestamp, payload.len() as u32)
                .with_lsn(new_lsn, prev_lsn)
                .with_record_type(RecordType::Full)
                .with_compression(compression)
        };

        self.write_entry(&header, payload, new_lsn)
    }

    /// Append a fragmented WAL entry (for large payloads)
    fn append_fragmented_entry(
        &mut self,
        op_type: WalOpType,
        timestamp: u32,
        payload: &[u8],
        compression: WalCompression,
    ) -> WalResult<bool> {
        let total_chunks = payload.len().div_ceil(WAL_MAX_RECORD_SIZE);
        let mut offset = 0;
        let mut chunk_index = 0;
        let mut first_lsn = Lsn::ZERO;
        let mut chunks_written = 0;

        while offset < payload.len() {
            let chunk_end = (offset + WAL_MAX_RECORD_SIZE).min(payload.len());
            let chunk_data = &payload[offset..chunk_end];
            let chunk_size = chunk_data.len();

            let prev_lsn = Lsn::new(self.current_lsn.load(Ordering::SeqCst));
            let entry_size = WAL_HEADER_SIZE + chunk_size;
            let new_lsn = Lsn::new(prev_lsn.as_u64() + entry_size as u64);

            if chunk_index == 0 {
                first_lsn = new_lsn;
            }

            let record_type = if total_chunks == 1 {
                RecordType::Full
            } else if chunk_index == 0 {
                RecordType::First
            } else if chunk_index == total_chunks - 1 {
                RecordType::Last
            } else {
                RecordType::Middle
            };

            let header = if self.config.checksum_enabled {
                WalHeader::new(op_type, timestamp, chunk_size as u32)
                    .with_lsn(new_lsn, prev_lsn)
                    .with_record_type(record_type)
                    .with_checksum(chunk_data)
                    .with_compression(compression)
            } else {
                WalHeader::new(op_type, timestamp, chunk_size as u32)
                    .with_lsn(new_lsn, prev_lsn)
                    .with_record_type(record_type)
                    .with_compression(compression)
            };

            if let Err(e) = self.write_entry(&header, chunk_data, new_lsn) {
                log::error!(
                    "Failed to write chunk {}/{} of fragmented WAL entry (first_lsn: {}, written: {}): {}",
                    chunk_index + 1,
                    total_chunks,
                    first_lsn.as_u64(),
                    chunks_written,
                    e
                );
                return Err(e);
            }

            offset = chunk_end;
            chunk_index += 1;
            chunks_written += 1;
        }

        Ok(true)
    }

    /// Write a single entry to the file
    fn write_entry(&mut self, header: &WalHeader, payload: &[u8], new_lsn: Lsn) -> WalResult<bool> {
        let header_bytes = header.as_bytes();

        let file = self.file.as_mut().ok_or(WalError::Closed)?;
        let total_len = header_bytes.len() + payload.len();

        let expected_size = self.file_used + total_len;
        if expected_size > self.file_size {
            let new_size =
                ((expected_size / self.config.truncate_size) + 1) * self.config.truncate_size;
            file.set_len(new_size as u64)?;
            self.file_size = new_size;
        }

        file.seek(SeekFrom::Start(self.file_used as u64))?;
        file.write_all(header_bytes)?;
        file.write_all(payload)?;
        self.file_used += total_len;

        self.current_lsn.store(new_lsn.as_u64(), Ordering::SeqCst);

        let write_count = self.write_count.fetch_add(1, Ordering::SeqCst) + 1;
        let elapsed = elapsed_since(*self.last_sync_time.lock().unwrap());
        let should_sync = should_sync(&self.config.sync_policy, write_count, elapsed);

        if should_sync {
            file.sync_data()?;
            let lsn = self.current_lsn.load(Ordering::SeqCst);
            self.last_synced_lsn.store(lsn, Ordering::SeqCst);
            self.write_count.store(0, Ordering::SeqCst);
            if let Ok(mut guard) = self.last_sync_time.lock() {
                *guard = Some(Instant::now());
            }
        }

        Ok(true)
    }

    /// Append multiple entries as a batch (for group commit)
    pub fn append_batch(&mut self, entries: &[(WalOpType, u32, &[u8])]) -> WalResult<bool> {
        if !self.is_open.load(Ordering::SeqCst) {
            return Err(WalError::Closed);
        }

        let mut total_len = 0;
        let mut compressed_entries = Vec::with_capacity(entries.len());

        for (op_type, timestamp, payload) in entries {
            let (final_payload, compression) = self.compressor.compress(payload)?;

            let prev_lsn = Lsn::new(self.current_lsn.load(Ordering::SeqCst) + total_len as u64);
            let entry_size = WAL_HEADER_SIZE + final_payload.len();
            let new_lsn = Lsn::new(prev_lsn.as_u64() + entry_size as u64);

            let header = if self.config.checksum_enabled {
                WalHeader::new(*op_type, *timestamp, final_payload.len() as u32)
                    .with_lsn(new_lsn, prev_lsn)
                    .with_checksum(&final_payload)
                    .with_compression(compression)
            } else {
                WalHeader::new(*op_type, *timestamp, final_payload.len() as u32)
                    .with_lsn(new_lsn, prev_lsn)
                    .with_compression(compression)
            };

            total_len += WAL_HEADER_SIZE + final_payload.len();
            compressed_entries.push((header, final_payload));
        }

        let file = self.file.as_mut().ok_or(WalError::Closed)?;

        let expected_size = self.file_used + total_len;
        if expected_size > self.file_size {
            let new_size =
                ((expected_size / self.config.truncate_size) + 1) * self.config.truncate_size;
            file.set_len(new_size as u64)?;
            self.file_size = new_size;
        }

        file.seek(SeekFrom::Start(self.file_used as u64))?;

        for (header, payload) in compressed_entries {
            file.write_all(header.as_bytes())?;
            file.write_all(&payload)?;
        }

        self.file_used += total_len;

        let new_lsn = self.current_lsn.load(Ordering::SeqCst) + total_len as u64;
        self.current_lsn.store(new_lsn, Ordering::SeqCst);

        file.sync_data()?;
        self.last_synced_lsn.store(new_lsn, Ordering::SeqCst);

        Ok(true)
    }

    /// Decompress payload (public helper)
    pub fn decompress_payload(payload: &[u8], compression: WalCompression) -> WalResult<Vec<u8>> {
        compression_mod::decompress_payload(payload, compression)
    }

    // ── Getters and Setters ──

    pub fn current_lsn(&self) -> Lsn {
        Lsn::new(self.current_lsn.load(Ordering::SeqCst))
    }

    pub fn last_synced_lsn(&self) -> Lsn {
        Lsn::new(self.last_synced_lsn.load(Ordering::SeqCst))
    }

    pub fn file_start_lsn(&self) -> Lsn {
        self.file_start_lsn
    }

    pub fn set_current_lsn(&self, lsn: Lsn) {
        self.current_lsn.store(lsn.as_u64(), Ordering::SeqCst);
    }

    pub fn file_size(&self) -> usize {
        self.file_size
    }

    pub fn file_used(&self) -> usize {
        self.file_used
    }

    pub fn checkpoint_seq(&self) -> u64 {
        self.checkpoint_seq
    }

    pub fn set_checkpoint_seq(&mut self, seq: u64) {
        self.checkpoint_seq = seq;
    }

    pub fn file_header(&self) -> Option<&WalFileHeader> {
        self.file_header.as_ref()
    }

    pub fn get_stats(&self) -> &WalStats {
        &self.stats
    }

    pub fn reset_stats(&mut self) {
        self.stats = WalStats::new();
    }

    pub fn group_commit_manager(&self) -> Option<&Arc<GroupCommitManager>> {
        self.group_commit.as_ref()
    }

    /// Process pending group commit batch
    pub fn process_group_commit(&mut self) -> WalResult<()> {
        if let Some(manager) = self.group_commit.clone() {
            manager.process_batch(|entries| self.append_batch(entries))?;
        }
        Ok(())
    }
}

impl WalWriter for LocalWalWriter {
    fn open(&mut self) -> WalResult<()> {
        if self.is_open.load(Ordering::SeqCst) {
            return Ok(());
        }

        self.version += 1;
        let path = self.find_available_path()?;

        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            if let Some(version_str) = file_name
                .strip_prefix(&format!("thread_{}_wal_", self.thread_id))
            {
                if let Ok(version) = u32::from_str_radix(version_str, 16) {
                    self.version = version;
                }
            }
        }

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

        self.rotate_if_needed()?;

        let file = self.file.as_mut().ok_or(WalError::Closed)?;

        let expected_size = self.file_used + data.len();
        if expected_size > self.file_size {
            let new_size =
                ((expected_size / self.config.truncate_size) + 1) * self.config.truncate_size;
            file.set_len(new_size as u64)?;
            self.file_size = new_size;
        }

        file.seek(SeekFrom::Start(self.file_used as u64))?;
        file.write_all(data)?;
        self.file_used += data.len();

        let new_lsn = self.current_lsn.load(Ordering::SeqCst) + data.len() as u64;
        self.current_lsn.store(new_lsn, Ordering::SeqCst);

        let write_count = self.write_count.fetch_add(1, Ordering::SeqCst) + 1;
        let elapsed = elapsed_since(*self.last_sync_time.lock().unwrap());
        let should_sync = should_sync(&self.config.sync_policy, write_count, elapsed);

        if should_sync {
            file.sync_data()?;
            let lsn = self.current_lsn.load(Ordering::SeqCst);
            self.last_synced_lsn.store(lsn, Ordering::SeqCst);
            self.write_count.store(0, Ordering::SeqCst);
            if let Ok(mut guard) = self.last_sync_time.lock() {
                *guard = Some(Instant::now());
            }
        }

        self.writes_since_cleanup += 1;

        if self.config.max_total_size > 0 || self.config.ttl_seconds > 0 {
            self.cleanup_old_wal_files()?;
        }

        if self.config.auto_checkpoint {
            self.maybe_trigger_checkpoint()?;
        }

        Ok(true)
    }

    fn sync(&self) -> WalResult<()> {
        if let Some(ref file) = self.file {
            file.sync_all()?;
            let current_lsn = self.current_lsn.load(Ordering::SeqCst);
            self.last_synced_lsn.store(current_lsn, Ordering::SeqCst);
            self.write_count.store(0, Ordering::SeqCst);
            if let Ok(mut guard) = self.last_sync_time.lock() {
                *guard = Some(Instant::now());
            }
        }
        Ok(())
    }
}

impl Drop for LocalWalWriter {
    fn drop(&mut self) {
        self.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::wal::SyncPolicy;
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

        writer
            .append_batch(&entries)
            .expect("Failed to append batch");
        writer.close();
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

    #[test]
    fn test_lsn_tracking() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let wal_path = temp_dir.path().to_string_lossy().to_string();

        let config = WalConfig::new()
            .with_checksum(true)
            .with_sync_policy(SyncPolicy::EveryWrite);
        let mut writer = LocalWalWriter::with_config(&wal_path, 0, config);
        writer.open().expect("Failed to open WAL");

        let initial_lsn = writer.current_lsn();

        writer
            .append_entry(WalOpType::InsertVertex, 1, b"payload1")
            .expect("Failed to append entry");

        let lsn_after_first = writer.current_lsn();
        assert!(lsn_after_first > initial_lsn);

        writer
            .append_entry(WalOpType::InsertVertex, 2, b"payload2")
            .expect("Failed to append entry");

        let lsn_after_second = writer.current_lsn();
        assert!(lsn_after_second > lsn_after_first);

        assert_eq!(writer.current_lsn(), writer.last_synced_lsn());

        writer.close();
    }

    #[test]
    fn test_sync_policy_batch() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let wal_path = temp_dir.path().to_string_lossy().to_string();

        let config = WalConfig::new()
            .with_checksum(true)
            .with_sync_policy(SyncPolicy::Batch { batch_size: 3 });
        let mut writer = LocalWalWriter::with_config(&wal_path, 0, config);
        writer.open().expect("Failed to open WAL");

        writer
            .append_entry(WalOpType::InsertVertex, 1, b"payload1")
            .expect("Failed to append entry");
        assert_ne!(writer.current_lsn(), writer.last_synced_lsn());

        writer
            .append_entry(WalOpType::InsertVertex, 2, b"payload2")
            .expect("Failed to append entry");
        assert_ne!(writer.current_lsn(), writer.last_synced_lsn());

        writer
            .append_entry(WalOpType::InsertVertex, 3, b"payload3")
            .expect("Failed to append entry");
        assert_eq!(writer.current_lsn(), writer.last_synced_lsn());

        writer.close();
    }

    #[test]
    fn test_sync_policy_never() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let wal_path = temp_dir.path().to_string_lossy().to_string();

        let config = WalConfig::new()
            .with_checksum(true)
            .with_sync_policy(SyncPolicy::Never);
        let mut writer = LocalWalWriter::with_config(&wal_path, 0, config);
        writer.open().expect("Failed to open WAL");

        for i in 0..10 {
            writer
                .append_entry(WalOpType::InsertVertex, i, b"payload")
                .expect("Failed to append entry");
        }

        assert_ne!(writer.current_lsn(), writer.last_synced_lsn());

        writer.sync().expect("Failed to sync");
        assert_eq!(writer.current_lsn(), writer.last_synced_lsn());

        writer.close();
    }

    #[test]
    fn test_fragmented_entry() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let wal_path = temp_dir.path().to_string_lossy().to_string();

        let config = WalConfig::new().with_checksum(true);
        let mut writer = LocalWalWriter::with_config(&wal_path, 0, config);
        writer.open().expect("Failed to open WAL");

        let large_payload: Vec<u8> = (0..(WAL_MAX_RECORD_SIZE * 2 + 1000))
            .map(|i| (i % 256) as u8)
            .collect();

        writer
            .append_entry(WalOpType::InsertVertex, 1, &large_payload)
            .expect("Failed to append fragmented entry");

        writer.sync().expect("Failed to sync");
        writer.close();
    }

    #[test]
    fn test_wal_rotation_basic() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let wal_path = temp_dir.path().to_string_lossy().to_string();

        let config = WalConfig::default()
            .with_max_file_size(1024)
            .with_truncate_size(4096);

        let mut writer = LocalWalWriter::with_config(&wal_path, 0, config);
        writer.open().expect("Failed to open WAL");

        let data = vec![0u8; 512];
        for _ in 0..3 {
            writer.append(&data).expect("Failed to append");
        }

        assert!(writer.version >= 2);
        writer.close();
    }

    #[test]
    fn test_wal_cleanup_by_size() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let wal_path = temp_dir.path().to_string_lossy().to_string();

        let config = WalConfig::default()
            .with_max_file_size(1024)
            .with_max_total_size(4096)
            .with_truncate_size(4096);

        let mut writer = LocalWalWriter::with_config(&wal_path, 0, config.clone());
        writer.open().expect("Failed to open WAL");

        let data = vec![0u8; 512];
        for _ in 0..20 {
            writer.append(&data).expect("Failed to append");
        }

        writer.cleanup_old_wal_files().expect("Failed to cleanup");

        let total_size = writer.get_total_wal_size().expect("Failed to get total size");
        assert!(total_size <= config.max_total_size);
        writer.close();
    }

    #[test]
    fn test_wal_file_naming() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let wal_path = temp_dir.path().to_string_lossy().to_string();

        let config = WalConfig::default();
        let writer = LocalWalWriter::with_config(&wal_path, 0, config);

        let path = writer.get_wal_file_path(1);
        assert!(path.to_string_lossy().contains("wal_00000001"));

        let path = writer.get_wal_file_path(100);
        assert!(path.to_string_lossy().contains("wal_00000064"));
    }

    #[test]
    fn test_wal_archive() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let wal_path = temp_dir.path().to_string_lossy().to_string();
        let archive_path = temp_dir.path().join("archive");

        let config = WalConfig::default()
            .with_archive_dir(archive_path.to_string_lossy().to_string())
            .with_archive_mode(ArchiveMode::Move);

        let mut writer = LocalWalWriter::with_config(&wal_path, 0, config);
        writer.open().expect("Failed to open WAL");

        let test_file = temp_dir.path().join("wal_00000001");
        std::fs::write(&test_file, vec![0u8; 100]).expect("Failed to create test file");

        writer
            .archive_wal_file(&test_file, archive_path.to_string_lossy().as_ref())
            .expect("Failed to archive");

        assert!(!test_file.exists());
        assert!(archive_path.exists());
        writer.close();
    }

    #[test]
    fn test_wal_rotation_with_recovery() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let wal_path = temp_dir.path().to_string_lossy().to_string();

        let config = WalConfig::default()
            .with_max_file_size(1024)
            .with_checksum(true);

        {
            let mut writer = LocalWalWriter::with_config(&wal_path, 0, config.clone());
            writer.open().expect("Failed to open WAL");

            for i in 0..10 {
                let data = format!("Entry {}", i).into_bytes();
                writer.append(&data).expect("Failed to append");
            }

            writer.sync().expect("Failed to sync");
        }

        let wal_files = std::fs::read_dir(&wal_path)
            .expect("Failed to read WAL dir")
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_str()
                    .map(|n| n.contains("_wal_"))
                    .unwrap_or(false)
            })
            .count();

        assert!(wal_files >= 1);
    }
}
