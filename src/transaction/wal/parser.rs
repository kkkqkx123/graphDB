//! WAL Parser
//!
//! Provides Write-Ahead Log parsing functionality for recovery

use std::fs::File;
use std::path::{Path, PathBuf};

use super::types::{
    Timestamp, UpdateWalUnit, WalContentUnit, WalError, WalHeader, WalOpType, WalResult,
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
        }
    }

    /// Parse all WAL files in the directory
    fn parse_wal_files(&mut self, wal_dir: &Path) -> WalResult<()> {
        if !wal_dir.exists() {
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

        for path in wal_files {
            self.parse_wal_file(&path)?;
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

        let mut offset = 0;
        while offset + WalHeader::SIZE <= buffer.len() {
            let header = WalHeader::from_bytes(&buffer[offset..offset + WalHeader::SIZE])
                .ok_or(WalError::InvalidHeader)?;

            if header.timestamp == 0 {
                break;
            }

            let payload_start = offset + WalHeader::SIZE;
            let payload_end = payload_start + header.length as usize;

            if payload_end > buffer.len() {
                break;
            }

            let payload = buffer[payload_start..payload_end].to_vec();
            let content = WalContentUnit::new(payload);

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

    /// Get all WAL entries as an iterator
    pub fn iter_entries(&self) -> WalEntryIter {
        WalEntryIter {
            parser: self,
            insert_index: 0,
            update_index: 0,
        }
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
        self.last_timestamp = 0;
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
    use tempfile::TempDir;

    #[test]
    fn test_wal_parser() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let wal_path = temp_dir.path().to_string_lossy().to_string();

        {
            let mut writer = LocalWalWriter::new(&wal_path, 0);
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

        let insert_wal = parser.get_insert_wal(1);
        assert!(insert_wal.is_some());

        let update_wals = parser.get_update_wals();
        assert_eq!(update_wals.len(), 1);

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
}
