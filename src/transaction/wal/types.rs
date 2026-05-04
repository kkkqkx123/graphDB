//! WAL Types
//!
//! Type definitions for Write-Ahead Log

use std::fmt;

use serde::{Deserialize, Serialize};
use oxicode::{Encode, Decode};
use crc32fast::Hasher;

/// WAL magic number for file identification
pub const WAL_MAGIC: u32 = 0x47524150; // "GRAP" in hex

/// WAL format version
pub const WAL_VERSION: u32 = 1;

/// WAL file header size
pub const WAL_FILE_HEADER_SIZE: usize = 64;

/// Timestamp type for MVCC
pub type Timestamp = u32;

/// Label ID type
pub type LabelId = u16;

/// Vertex ID type
pub type VertexId = u64;

/// Edge ID type
pub type EdgeId = u64;

/// Column ID type
pub type ColumnId = i32;

/// WAL operation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum WalOpType {
    InsertVertex = 0,
    InsertEdge = 1,
    CreateVertexType = 2,
    CreateEdgeType = 3,
    AddVertexProp = 4,
    AddEdgeProp = 5,
    UpdateVertexProp = 6,
    UpdateEdgeProp = 7,
    DeleteVertex = 8,
    DeleteEdge = 9,
    DeleteVertexType = 10,
    DeleteEdgeType = 11,
    DeleteVertexProp = 12,
    DeleteEdgeProp = 13,
    RenameVertexProp = 14,
    RenameEdgeProp = 15,
}

impl TryFrom<u8> for WalOpType {
    type Error = WalError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(WalOpType::InsertVertex),
            1 => Ok(WalOpType::InsertEdge),
            2 => Ok(WalOpType::CreateVertexType),
            3 => Ok(WalOpType::CreateEdgeType),
            4 => Ok(WalOpType::AddVertexProp),
            5 => Ok(WalOpType::AddEdgeProp),
            6 => Ok(WalOpType::UpdateVertexProp),
            7 => Ok(WalOpType::UpdateEdgeProp),
            8 => Ok(WalOpType::DeleteVertex),
            9 => Ok(WalOpType::DeleteEdge),
            10 => Ok(WalOpType::DeleteVertexType),
            11 => Ok(WalOpType::DeleteEdgeType),
            12 => Ok(WalOpType::DeleteVertexProp),
            13 => Ok(WalOpType::DeleteEdgeProp),
            14 => Ok(WalOpType::RenameVertexProp),
            15 => Ok(WalOpType::RenameEdgeProp),
            _ => Err(WalError::InvalidOpType(value)),
        }
    }
}

impl fmt::Display for WalOpType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WalOpType::InsertVertex => write!(f, "InsertVertex"),
            WalOpType::InsertEdge => write!(f, "InsertEdge"),
            WalOpType::CreateVertexType => write!(f, "CreateVertexType"),
            WalOpType::CreateEdgeType => write!(f, "CreateEdgeType"),
            WalOpType::AddVertexProp => write!(f, "AddVertexProp"),
            WalOpType::AddEdgeProp => write!(f, "AddEdgeProp"),
            WalOpType::UpdateVertexProp => write!(f, "UpdateVertexProp"),
            WalOpType::UpdateEdgeProp => write!(f, "UpdateEdgeProp"),
            WalOpType::DeleteVertex => write!(f, "DeleteVertex"),
            WalOpType::DeleteEdge => write!(f, "DeleteEdge"),
            WalOpType::DeleteVertexType => write!(f, "DeleteVertexType"),
            WalOpType::DeleteEdgeType => write!(f, "DeleteEdgeType"),
            WalOpType::DeleteVertexProp => write!(f, "DeleteVertexProp"),
            WalOpType::DeleteEdgeProp => write!(f, "DeleteEdgeProp"),
            WalOpType::RenameVertexProp => write!(f, "RenameVertexProp"),
            WalOpType::RenameEdgeProp => write!(f, "RenameEdgeProp"),
        }
    }
}

/// WAL file header (written at the beginning of each WAL file)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct WalFileHeader {
    /// Magic number for file identification
    pub magic: u32,
    /// Format version
    pub version: u32,
    /// Checkpoint sequence number
    pub checkpoint_seq: u64,
    /// Random salt-1 for validation
    pub salt1: u32,
    /// Random salt-2 for validation
    pub salt2: u32,
    /// Creation timestamp (Unix epoch)
    pub created_at: u64,
    /// Thread ID that created this file
    pub thread_id: u32,
    /// Reserved for future use
    pub reserved: [u8; 28],
}

impl WalFileHeader {
    pub const SIZE: usize = WAL_FILE_HEADER_SIZE;

    pub fn new(thread_id: u32, checkpoint_seq: u64) -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        Self {
            magic: WAL_MAGIC,
            version: WAL_VERSION,
            checkpoint_seq,
            salt1: rng.gen(),
            salt2: rng.gen(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            thread_id,
            reserved: [0; 28],
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self as *const WalFileHeader as *const u8,
                std::mem::size_of::<WalFileHeader>(),
            )
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < Self::SIZE {
            return None;
        }
        let header: WalFileHeader = unsafe { std::ptr::read(bytes.as_ptr() as *const WalFileHeader) };
        Some(header)
    }

    pub fn is_valid(&self) -> bool {
        self.magic == WAL_MAGIC
    }

    pub fn salts(&self) -> (u32, u32) {
        (self.salt1, self.salt2)
    }
}

/// WAL header (for each WAL entry)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct WalHeader {
    /// Length of the payload
    pub length: u32,
    /// Operation type
    pub op_type: u8,
    /// Is this an update operation (vs insert)
    pub is_update: bool,
    /// Flags for future use
    pub flags: u16,
    /// Transaction timestamp
    pub timestamp: Timestamp,
    /// CRC32 checksum of header (excluding this field) + payload
    pub checksum: u32,
}

impl WalHeader {
    pub const SIZE: usize = 16;

    pub fn new(op_type: WalOpType, timestamp: Timestamp, length: u32) -> Self {
        let is_update = matches!(
            op_type,
            WalOpType::UpdateVertexProp
                | WalOpType::UpdateEdgeProp
                | WalOpType::DeleteVertex
                | WalOpType::DeleteEdge
                | WalOpType::DeleteVertexType
                | WalOpType::DeleteEdgeType
                | WalOpType::DeleteVertexProp
                | WalOpType::DeleteEdgeProp
                | WalOpType::RenameVertexProp
                | WalOpType::RenameEdgeProp
        );

        Self {
            length,
            op_type: op_type as u8,
            is_update,
            flags: 0,
            timestamp,
            checksum: 0,
        }
    }

    pub fn with_checksum(mut self, payload: &[u8]) -> Self {
        let mut hasher = Hasher::new();
        hasher.update(&self.length.to_le_bytes());
        hasher.update(&[self.op_type, self.is_update as u8]);
        hasher.update(&self.flags.to_le_bytes());
        hasher.update(&self.timestamp.to_le_bytes());
        hasher.update(payload);
        self.checksum = hasher.finalize();
        self
    }

    pub fn verify_checksum(&self, payload: &[u8]) -> bool {
        let mut hasher = Hasher::new();
        hasher.update(&self.length.to_le_bytes());
        hasher.update(&[self.op_type, self.is_update as u8]);
        hasher.update(&self.flags.to_le_bytes());
        hasher.update(&self.timestamp.to_le_bytes());
        hasher.update(payload);
        hasher.finalize() == self.checksum
    }

    pub fn with_compression(mut self, compression: WalCompression) -> Self {
        self.flags = (self.flags & !wal_flags::COMPRESSION_MASK) 
            | (compression.flag_byte() as u16);
        self
    }

    pub fn compression(&self) -> WalCompression {
        WalCompression::from_flag_byte((self.flags & wal_flags::COMPRESSION_MASK) as u8)
    }

    pub fn is_compressed(&self) -> bool {
        self.compression() != WalCompression::None
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self as *const WalHeader as *const u8,
                std::mem::size_of::<WalHeader>(),
            )
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < Self::SIZE {
            return None;
        }
        let header: WalHeader = unsafe { std::ptr::read(bytes.as_ptr() as *const WalHeader) };
        Some(header)
    }
}

/// WAL error type
#[derive(Debug, Clone, thiserror::Error)]
pub enum WalError {
    #[error("IO error: {0}")]
    IoError(String),

    #[error("Invalid operation type: {0}")]
    InvalidOpType(u8),

    #[error("Invalid header")]
    InvalidHeader,

    #[error("Invalid file header")]
    InvalidFileHeader,

    #[error("Checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: u32, actual: u32 },

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Corrupted WAL: {0}")]
    Corrupted(String),

    #[error("WAL is closed")]
    Closed,

    #[error("Unsupported WAL version: {0}")]
    UnsupportedVersion(u32),

    #[error("Recovery aborted: {0}")]
    RecoveryAborted(String),
}

impl From<std::io::Error> for WalError {
    fn from(e: std::io::Error) -> Self {
        WalError::IoError(e.to_string())
    }
}

impl From<oxicode::Error> for WalError {
    fn from(e: oxicode::Error) -> Self {
        WalError::SerializationError(e.to_string())
    }
}

/// WAL result type
pub type WalResult<T> = Result<T, WalError>;

/// WAL recovery mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WalRecoveryMode {
    /// Abort recovery if any corruption is found
    AbortOnCorruption,
    /// Skip corrupted entries and continue recovery
    #[default]
    SkipCorruption,
    /// Only use WAL for recovery (ignore other state)
    WalOnly,
    /// Error if WAL file is missing
    ErrorIfMissing,
}

/// WAL compression type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WalCompression {
    #[default]
    None,
    Snappy,
    Zstd,
}

impl WalCompression {
    pub fn flag_byte(&self) -> u8 {
        match self {
            WalCompression::None => 0,
            WalCompression::Snappy => 1,
            WalCompression::Zstd => 2,
        }
    }

    pub fn from_flag_byte(byte: u8) -> Self {
        match byte & 0x0F {
            1 => WalCompression::Snappy,
            2 => WalCompression::Zstd,
            _ => WalCompression::None,
        }
    }
}

/// WAL header flags
pub mod wal_flags {
    pub const COMPRESSION_MASK: u16 = 0x000F;
    pub const COMPRESSED: u16 = 0x0001;
}

/// WAL content unit (parsed WAL entry)
#[derive(Debug, Clone)]
pub struct WalContentUnit {
    /// Pointer to the content (for mmap'd data)
    pub data: Vec<u8>,
    /// Size of the content
    pub size: usize,
}

impl WalContentUnit {
    pub fn new(data: Vec<u8>) -> Self {
        let size = data.len();
        Self { data, size }
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }
}

/// Update WAL unit (with timestamp)
#[derive(Debug, Clone)]
pub struct UpdateWalUnit {
    /// Transaction timestamp
    pub timestamp: Timestamp,
    /// Content
    pub content: WalContentUnit,
}

impl UpdateWalUnit {
    pub fn new(timestamp: Timestamp, data: Vec<u8>) -> Self {
        Self {
            timestamp,
            content: WalContentUnit::new(data),
        }
    }
}

/// Insert vertex redo log
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct InsertVertexRedo {
    pub label: LabelId,
    pub oid: Vec<u8>,
    pub properties: Vec<(String, Vec<u8>)>,
}

/// Insert edge redo log
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct InsertEdgeRedo {
    pub src_label: LabelId,
    pub src_oid: Vec<u8>,
    pub dst_label: LabelId,
    pub dst_oid: Vec<u8>,
    pub edge_label: LabelId,
    pub properties: Vec<(String, Vec<u8>)>,
}

/// Update vertex property redo log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateVertexPropRedo {
    pub label: LabelId,
    pub oid: Vec<u8>,
    pub prop_name: String,
    pub value: Vec<u8>,
}

/// Update edge property redo log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateEdgePropRedo {
    pub src_label: LabelId,
    pub src_oid: Vec<u8>,
    pub dst_label: LabelId,
    pub dst_oid: Vec<u8>,
    pub edge_label: LabelId,
    pub prop_name: String,
    pub value: Vec<u8>,
}

/// Create vertex type redo log
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct CreateVertexTypeRedo {
    pub label_name: String,
    pub schema: Vec<(String, String)>,
}

/// Create edge type redo log
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct CreateEdgeTypeRedo {
    pub src_label: String,
    pub dst_label: String,
    pub edge_label: String,
    pub schema: Vec<(String, String)>,
}

/// Delete vertex redo log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteVertexRedo {
    pub label: LabelId,
    pub oid: Vec<u8>,
}

/// Delete edge redo log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteEdgeRedo {
    pub src_label: LabelId,
    pub src_oid: Vec<u8>,
    pub dst_label: LabelId,
    pub dst_oid: Vec<u8>,
    pub edge_label: LabelId,
}

/// WAL configuration
#[derive(Debug, Clone)]
pub struct WalConfig {
    /// Truncate size for WAL files
    pub truncate_size: usize,
    /// Maximum WAL file size before rotation
    pub max_file_size: usize,
    /// Whether to sync after each write
    pub sync_on_write: bool,
    /// Enable group commit for better throughput
    pub group_commit_enabled: bool,
    /// Delay in microseconds for group commit batching
    pub group_commit_delay_us: u64,
    /// Maximum batch size for group commit
    pub group_commit_batch_size: usize,
    /// Recovery mode
    pub recovery_mode: WalRecoveryMode,
    /// Compression type
    pub compression: WalCompression,
    /// Enable checksum verification
    pub checksum_enabled: bool,
}

impl Default for WalConfig {
    fn default() -> Self {
        Self {
            truncate_size: 4 * 1024 * 1024, // 4MB
            max_file_size: 64 * 1024 * 1024, // 64MB
            sync_on_write: true,
            group_commit_enabled: true,
            group_commit_delay_us: 100, // 100 microseconds
            group_commit_batch_size: 1024,
            recovery_mode: WalRecoveryMode::default(),
            compression: WalCompression::default(),
            checksum_enabled: true,
        }
    }
}

impl WalConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_truncate_size(mut self, size: usize) -> Self {
        self.truncate_size = size;
        self
    }

    pub fn with_max_file_size(mut self, size: usize) -> Self {
        self.max_file_size = size;
        self
    }

    pub fn with_sync_on_write(mut self, sync: bool) -> Self {
        self.sync_on_write = sync;
        self
    }

    pub fn with_group_commit(mut self, enabled: bool, delay_us: u64, batch_size: usize) -> Self {
        self.group_commit_enabled = enabled;
        self.group_commit_delay_us = delay_us;
        self.group_commit_batch_size = batch_size;
        self
    }

    pub fn with_recovery_mode(mut self, mode: WalRecoveryMode) -> Self {
        self.recovery_mode = mode;
        self
    }

    pub fn with_compression(mut self, compression: WalCompression) -> Self {
        self.compression = compression;
        self
    }

    pub fn with_checksum(mut self, enabled: bool) -> Self {
        self.checksum_enabled = enabled;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wal_header() {
        let header = WalHeader::new(WalOpType::InsertVertex, 12345, 100);
        assert_eq!(header.length, 100);
        assert_eq!(header.timestamp, 12345);
        assert_eq!(header.op_type, WalOpType::InsertVertex as u8);
        assert!(!header.is_update);
    }

    #[test]
    fn test_wal_op_type() {
        assert_eq!(
            WalOpType::try_from(0).unwrap(),
            WalOpType::InsertVertex
        );
        assert_eq!(
            WalOpType::try_from(6).unwrap(),
            WalOpType::UpdateVertexProp
        );
        assert!(WalOpType::try_from(100).is_err());
    }

    #[test]
    fn test_wal_header_serialization() {
        let header = WalHeader::new(WalOpType::InsertEdge, 999, 50);
        let bytes = header.as_bytes();
        assert_eq!(bytes.len(), WalHeader::SIZE);

        let parsed = WalHeader::from_bytes(bytes).unwrap();
        assert_eq!(parsed.length, 50);
        assert_eq!(parsed.timestamp, 999);
    }

    #[test]
    fn test_wal_header_checksum() {
        let payload = b"test_payload_data";
        let header = WalHeader::new(WalOpType::InsertVertex, 12345, payload.len() as u32)
            .with_checksum(payload);
        
        assert!(header.verify_checksum(payload));
        
        let corrupted_payload = b"corrupted_data";
        assert!(!header.verify_checksum(corrupted_payload));
    }

    #[test]
    fn test_wal_file_header() {
        let header = WalFileHeader::new(1, 0);
        assert!(header.is_valid());
        assert_eq!(header.thread_id, 1);
        assert_eq!(header.checkpoint_seq, 0);
        
        let bytes = header.as_bytes();
        assert_eq!(bytes.len(), WalFileHeader::SIZE);
        
        let parsed = WalFileHeader::from_bytes(bytes).unwrap();
        assert!(parsed.is_valid());
        assert_eq!(parsed.thread_id, 1);
    }

    #[test]
    fn test_wal_config() {
        let config = WalConfig::new()
            .with_checksum(true)
            .with_group_commit(true, 200, 512)
            .with_recovery_mode(WalRecoveryMode::AbortOnCorruption);
        
        assert!(config.checksum_enabled);
        assert!(config.group_commit_enabled);
        assert_eq!(config.group_commit_delay_us, 200);
        assert_eq!(config.group_commit_batch_size, 512);
        assert_eq!(config.recovery_mode, WalRecoveryMode::AbortOnCorruption);
    }
}
