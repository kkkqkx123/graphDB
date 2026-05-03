//! WAL Types
//!
//! Type definitions for Write-Ahead Log

use std::fmt;

use serde::{Deserialize, Serialize};
use oxicode::{Encode, Decode};

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

/// WAL header
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct WalHeader {
    /// Length of the payload
    pub length: u32,
    /// Operation type
    pub op_type: u8,
    /// Is this an update operation (vs insert)
    pub is_update: bool,
    /// Reserved
    pub reserved: [u8; 2],
    /// Transaction timestamp
    pub timestamp: Timestamp,
}

impl WalHeader {
    pub const SIZE: usize = 12;

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
            reserved: [0; 2],
            timestamp,
        }
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
}

impl Default for WalConfig {
    fn default() -> Self {
        Self {
            truncate_size: 4 * 1024 * 1024, // 4MB
            max_file_size: 64 * 1024 * 1024, // 64MB
            sync_on_write: true,
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
}
