//! WAL Parser
//!
//! Provides Write-Ahead Log parsing functionality for recovery

use std::fs::File;
use std::path::{Path, PathBuf};

use super::types::{
    Timestamp, UpdateWalUnit, WalCompression, WalContentUnit, WalError, WalFileHeader, WalHeader, 
    WalOpType, WalRecoveryMode, WalResult, WAL_FILE_HEADER_SIZE,
};

/// WAL parser trait
pub trait WalParser: Send + Sync {
    /// Open and parse WAL files
    fn open(&mut self, wal_uri: &str) -> WalResult<()>;

    /// Close the parser
    fn close(&mut self);

    /// Get the last timestamp
    fn last_timestamp(&self) -> Timestamp;

    /// Get insert WAL content for a timestamp
    fn get_insert_wal(&self, ts: Timestamp) -> Option<&WalContentUnit>;

    /// Get all update WAL units
    fn get_update_wals(&self) -> &[UpdateWalUnit];
}

/// Parse result for a single WAL entry
#[derive(Debug, Clone)]
pub struct ParsedWalEntry {
    pub header: WalHeader,
    pub payload: Vec<u8>,
    pub checksum_valid: bool,
    pub offset: usize,
}

/// Local file-based WAL parser
pub struct LocalWalParser {
    /// WAL directory path
    wal_dir: Option<PathBuf>,
    /// Insert WAL entries indexed by timestamp
    insert_wal_list: Vec<WalContentUnit>,
    /// Update WAL entries sorted by timestamp
    update_wal_list: Vec<UpdateWalUnit>,
    /// Last seen timestamp
    last_timestamp: Timestamp,
    /// Opened files
    files: Vec<File>,
    /// File headers for each parsed file
    file_headers: Vec<WalFileHeader>,
    /// Recovery mode
    recovery_mode: WalRecoveryMode,
    /// Enable checksum verification
    verify_checksum: bool,
    /// Number of corrupted entries found
    corrupted_count: usize,
    /// Number of skipped entries
    skipped_count: usize,
}

impl LocalWalParser {
    /// Create a new local WAL parser
    pub fn new() -> Self {
        Self {
            wal_dir: None,
            insert_wal_list: Vec::new(),
            update_wal_list: Vec::new(),
            last_timestamp: 0,
            files: Vec::new(),
            file_headers: Vec::new(),
            recovery_mode: WalRecoveryMode::default(),
            verify_checksum: true,
            corrupted_count: 0,
            skipped_count: 0,
        }
    }

    /// Create with custom recovery mode
    pub fn with_recovery_mode(recovery_mode: WalRecoveryMode) -> Self {
        Self {
            recovery_mode,
            verify_checksum: true,
            ..Self::new()
        }
    }

    /// Set checksum verification
    pub fn with_verify_checksum(mut self, verify: bool) -> Self {
        self.verify_checksum = verify;
        self
    }

    /// Get number of corrupted entries found
    pub fn corrupted_count(&self) -> usize {
        self.corrupted_count
    }

    /// Get number of skipped entries
    pub fn skipped_count(&self) -> usize {
        self.skipped_count
    }

    /// Get file headers
    pub fn file_headers(&self) -> &[WalFileHeader] {
        &self.file_headers
    }

    /// Parse all WAL files in the directory
    fn parse_wal_files(&mut self, wal_dir: &Path) -> WalResult<()> {
        if !wal_dir.exists() {
            if self.recovery_mode == WalRecoveryMode::ErrorIfMissing {
                return Err(WalError::FileNotFound(wal_dir.to_string_lossy().to_string()));
            }
            std::fs::create_dir_all(wal_dir)
                .map_err(|e| WalError::IoError(e.to_string()))?;
            return Ok(());
        }

        let mut wal_files: Vec<PathBuf> = std::fs::read_dir(wal_dir)
            .map_err(|e| WalError::IoError(e.to_string()))?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| path.extension().map_or(false, |ext| ext == "wal"))
            .collect();

        wal_files.sort();

        if wal_files.is_empty() && self.recovery_mode == WalRecoveryMode::ErrorIfMissing {
            return Err(WalError::FileNotFound("No WAL files found".to_string()));
        }

        for path in wal_files {
            if let Err(e) = self.parse_wal_file(&path) {
                match self.recovery_mode {
                    WalRecoveryMode::AbortOnCorruption => {
                        return Err(WalError::RecoveryAborted(format!(
                            "Failed to parse {}: {}",
                            path.display(),
                            e
                        )));
                    }
                    _ => {
                        self.corrupted_count += 1;
                        continue;
                    }
                }
            }
        }

        self.update_wal_list.sort_by_key(|u| u.timestamp);

        Ok(())
    }

    /// Parse a single WAL file
    fn parse_wal_file(&mut self, path: &Path) -> WalResult<()> {
        use std::io::Read;

        let metadata = std::fs::metadata(path)
            .map_err(|e| WalError::IoError(e.to_string()))?;

        if metadata.len() == 0 {
            return Ok(());
        }

        let mut file = File::open(path)
            .map_err(|e| WalError::IoError(e.to_string()))?;

        let file_size = metadata.len() as usize;
        let mut buffer = Vec::with_capacity(file_size);
        file.read_to_end(&mut buffer)
            .map_err(|e| WalError::IoError(e.to_string()))?;

        self.files.push(file);

        if buffer.len() < WAL_FILE_HEADER_SIZE {
            return Err(WalError::InvalidFileHeader);
        }

        let file_header = WalFileHeader::from_bytes(&buffer[..WAL_FILE_HEADER_SIZE])
            .ok_or(WalError::InvalidFileHeader)?;

        if !file_header.is_valid() {
            return Err(WalError::InvalidFileHeader);
        }

        self.file_headers.push(file_header);

        let mut offset = WAL_FILE_HEADER_SIZE;
        while offset + WalHeader::SIZE <= buffer.len() {
            let header = match WalHeader::from_bytes(&buffer[offset..offset + WalHeader::SIZE]) {
                Some(h) => h,
                None => {
                    self.corrupted_count += 1;
                    offset += 1;
                    continue;
                }
            };

            if header.timestamp == 0 && header.length == 0 {
                break;
            }

            let payload_start = offset + WalHeader::SIZE;
            let payload_end = payload_start + header.length as usize;

            if payload_end > buffer.len() {
                match self.recovery_mode {
                    WalRecoveryMode::AbortOnCorruption => {
                        return Err(WalError::Corrupted(format!(
                            "Truncated entry at offset {}",
                            offset
                        )));
                    }
                    _ => {
                        self.corrupted_count += 1;
                        break;
                    }
                }
            }

            let payload = buffer[payload_start..payload_end].to_vec();

            if self.verify_checksum && header.checksum != 0 {
                if !header.verify_checksum(&payload) {
                    match self.recovery_mode {
                        WalRecoveryMode::AbortOnCorruption => {
                            return Err(WalError::ChecksumMismatch {
                                expected: header.checksum,
                                actual: self.compute_checksum(&header, &payload),
                            });
                        }
                        _ => {
                            self.corrupted_count += 1;
                            offset = payload_end;
                            continue;
                        }
                    }
                }
            }

            let final_payload = if header.is_compressed() {
                match Self::decompress_payload(&payload, header.compression()) {
                    Ok(decompressed) => decompressed,
                    Err(e) => {
                        match self.recovery_mode {
                            WalRecoveryMode::AbortOnCorruption => {
                                return Err(e);
                            }
                            _ => {
                                self.corrupted_count += 1;
                                offset = payload_end;
                                continue;
                            }
                        }
                    }
                }
            } else {
                payload
            };

            let content = WalContentUnit::new(final_payload);

            if header.is_update {
                self.update_wal_list.push(UpdateWalUnit::new(
                    header.timestamp,
                    content.data,
                ));
            } else {
                let ts = header.timestamp as usize;
                if ts >= self.insert_wal_list.len() {
                    self.insert_wal_list.resize(ts + 1, WalContentUnit::new(Vec::new()));
                }
                self.insert_wal_list[ts] = content;
            }

            self.last_timestamp = self.last_timestamp.max(header.timestamp);
            offset = payload_end;
        }

        Ok(())
    }

    /// Compute checksum for verification
    fn compute_checksum(&self, header: &WalHeader, payload: &[u8]) -> u32 {
        use crc32fast::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(&header.length.to_le_bytes());
        hasher.update(&[header.op_type, header.is_update as u8]);
        hasher.update(&header.flags.to_le_bytes());
        hasher.update(&header.timestamp.to_le_bytes());
        hasher.update(payload);
        hasher.finalize()
    }

    /// Decompress payload
    fn decompress_payload(payload: &[u8], compression: WalCompression) -> WalResult<Vec<u8>> {
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

    /// Get all WAL entries as an iterator
    pub fn iter_entries(&self) -> WalEntryIter {
        WalEntryIter {
            parser: self,
            insert_index: 0,
            update_index: 0,
        }
    }

    /// Parse and return all entries with metadata
    pub fn parse_all_entries(&self) -> Vec<ParsedWalEntry> {
        let mut entries = Vec::new();
        
        for (ts, content) in self.insert_wal_list.iter().enumerate() {
            if content.size > 0 {
                let header = WalHeader::new(WalOpType::InsertVertex, ts as u32, content.size as u32);
                entries.push(ParsedWalEntry {
                    header,
                    payload: content.data.clone(),
                    checksum_valid: true,
                    offset: 0,
                });
            }
        }

        entries
    }
}

impl Default for LocalWalParser {
    fn default() -> Self {
        Self::new()
    }
}

impl WalParser for LocalWalParser {
    fn open(&mut self, wal_uri: &str) -> WalResult<()> {
        let wal_dir = PathBuf::from(wal_uri);
        self.wal_dir = Some(wal_dir.clone());
        self.parse_wal_files(&wal_dir)
    }

    fn close(&mut self) {
        self.insert_wal_list.clear();
        self.update_wal_list.clear();
        self.files.clear();
        self.file_headers.clear();
        self.last_timestamp = 0;
        self.corrupted_count = 0;
        self.skipped_count = 0;
    }

    fn last_timestamp(&self) -> Timestamp {
        self.last_timestamp
    }

    fn get_insert_wal(&self, ts: Timestamp) -> Option<&WalContentUnit> {
        self.insert_wal_list.get(ts as usize)
    }

    fn get_update_wals(&self) -> &[UpdateWalUnit] {
        &self.update_wal_list
    }
}

/// Iterator over WAL entries
pub struct WalEntryIter<'a> {
    parser: &'a LocalWalParser,
    insert_index: usize,
    update_index: usize,
}

impl<'a> Iterator for WalEntryIter<'a> {
    type Item = WalEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.insert_index < self.parser.insert_wal_list.len() {
            let ts = self.insert_index as Timestamp;
            let content = &self.parser.insert_wal_list[self.insert_index];
            self.insert_index += 1;

            if content.size > 0 {
                return Some(WalEntry::Insert(ts, content.clone()));
            }
        }

        if self.update_index < self.parser.update_wal_list.len() {
            let unit = &self.parser.update_wal_list[self.update_index];
            self.update_index += 1;
            return Some(WalEntry::Update(unit.clone()));
        }

        None
    }
}

/// WAL entry
#[derive(Debug, Clone)]
pub enum WalEntry {
    Insert(Timestamp, WalContentUnit),
    Update(UpdateWalUnit),
}

/// WAL parser factory
pub struct WalParserFactory;

impl WalParserFactory {
    /// Create a WAL parser based on the URI scheme
    pub fn create_wal_parser(wal_uri: &str) -> WalResult<Box<dyn WalParser>> {
        let scheme = Self::get_scheme(wal_uri);

        match scheme.as_str() {
            "file" | "" => Ok(Box::new(LocalWalParser::new())),
            _ => Err(WalError::IoError(format!(
                "Unknown WAL parser scheme: {}",
                scheme
            ))),
        }
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
    use crate::transaction::wal::writer::{LocalWalWriter, WalWriter};
    use crate::transaction::wal::types::WalConfig;
    use tempfile::TempDir;

    #[test]
    fn test_wal_parser() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let wal_path = temp_dir.path().to_string_lossy().to_string();

        {
            let config = WalConfig::new().with_checksum(true);
            let mut writer = LocalWalWriter::with_config(&wal_path, 0, config);
            writer.open().expect("Failed to open WAL");

            writer
                .append_entry(WalOpType::InsertVertex, 1, b"vertex1")
                .expect("Failed to append");
            writer
                .append_entry(WalOpType::InsertVertex, 2, b"vertex2")
                .expect("Failed to append");
            writer
                .append_entry(WalOpType::UpdateVertexProp, 3, b"update1")
                .expect("Failed to append");

            writer.sync().expect("Failed to sync");
        }

        let mut parser = LocalWalParser::new();
        parser.open(&wal_path).expect("Failed to parse WAL");

        assert_eq!(parser.last_timestamp(), 3);
        assert_eq!(parser.corrupted_count(), 0);

        let insert_wal = parser.get_insert_wal(1);
        assert!(insert_wal.is_some());

        let update_wals = parser.get_update_wals();
        assert_eq!(update_wals.len(), 1);

        assert!(!parser.file_headers().is_empty());
        assert!(parser.file_headers()[0].is_valid());

        parser.close();
    }

    #[test]
    fn test_wal_entry_iter() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let wal_path = temp_dir.path().to_string_lossy().to_string();

        {
            let mut writer = LocalWalWriter::new(&wal_path, 0);
            writer.open().expect("Failed to open WAL");

            writer
                .append_entry(WalOpType::InsertVertex, 1, b"data1")
                .expect("Failed to append");
            writer
                .append_entry(WalOpType::UpdateVertexProp, 2, b"data2")
                .expect("Failed to append");

            writer.sync().expect("Failed to sync");
        }

        let mut parser = LocalWalParser::new();
        parser.open(&wal_path).expect("Failed to parse WAL");

        let entries: Vec<_> = parser.iter_entries().collect();
        assert!(!entries.is_empty());
    }

    #[test]
    fn test_wal_parser_with_recovery_mode() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let wal_path = temp_dir.path().to_string_lossy().to_string();

        {
            let mut writer = LocalWalWriter::new(&wal_path, 0);
            writer.open().expect("Failed to open WAL");
            writer
                .append_entry(WalOpType::InsertVertex, 1, b"data")
                .expect("Failed to append");
            writer.sync().expect("Failed to sync");
        }

        let mut parser = LocalWalParser::with_recovery_mode(WalRecoveryMode::SkipCorruption);
        parser.open(&wal_path).expect("Failed to parse WAL");
        assert_eq!(parser.last_timestamp(), 1);
    }

    #[test]
    fn test_wal_parser_checksum_verification() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let wal_path = temp_dir.path().to_string_lossy().to_string();

        {
            let config = WalConfig::new().with_checksum(true);
            let mut writer = LocalWalWriter::with_config(&wal_path, 0, config);
            writer.open().expect("Failed to open WAL");
            writer
                .append_entry(WalOpType::InsertVertex, 1, b"test_payload")
                .expect("Failed to append");
            writer.sync().expect("Failed to sync");
        }

        let mut parser = LocalWalParser::new().with_verify_checksum(true);
        parser.open(&wal_path).expect("Failed to parse WAL");
        
        assert_eq!(parser.corrupted_count(), 0);
        let insert_wal = parser.get_insert_wal(1);
        assert!(insert_wal.is_some());
    }

    #[test]
    fn test_wal_parser_error_if_missing() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let non_existent_path = temp_dir.path().join("non_existent");
        let wal_path = non_existent_path.to_string_lossy().to_string();

        let mut parser = LocalWalParser::with_recovery_mode(WalRecoveryMode::ErrorIfMissing);
        let result = parser.open(&wal_path);
        assert!(result.is_err());
    }
}
