//! Write-Ahead Log (WAL) Module
//!
//! Provides durability guarantees through write-ahead logging.
//!
//! ## Components
//!
//! - `WalWriter`: Write WAL entries to persistent storage
//! - `WalParser`: Parse WAL files for recovery
//! - `WalHeader`: WAL entry header format
//!
//! ## Usage
//!
//! ```rust,ignore
//! use graphdb::transaction::wal::{LocalWalWriter, WalWriter, WalOpType};
//!
//! // Create a WAL writer
//! let mut writer = LocalWalWriter::new("/path/to/wal", 0);
//! writer.open()?;
//!
//! // Append an entry
//! writer.append_entry(WalOpType::InsertVertex, 1, b"payload")?;
//!
//! // Sync and close
//! writer.sync()?;
//! writer.close();
//! ```
//!
//! ## Recovery
//!
//! ```rust,ignore
//! use graphdb::transaction::wal::{LocalWalParser, WalParser};
//!
//! let mut parser = LocalWalParser::new();
//! parser.open("/path/to/wal")?;
//!
//! // Get insert WAL entries
//! if let Some(content) = parser.get_insert_wal(1) {
//!     // Process the entry
//! }
//!
//! // Get update WAL entries
//! for update in parser.get_update_wals() {
//!     // Process update entries
//! }
//! ```

pub mod checkpoint;
pub mod parser;
pub mod types;
pub mod writer;

pub use checkpoint::{Checkpoint, CheckpointManager};
pub use parser::{
    LocalWalParser, WalEntry, WalEntryIter, WalParser, WalParserFactory, ParsedWalEntry,
};
pub use types::{
    ColumnId, CreateEdgeTypeRedo, CreateVertexTypeRedo, DeleteEdgeRedo, DeleteVertexRedo,
    EdgeId, InsertEdgeRedo, InsertVertexRedo, LabelId, Timestamp, UpdateEdgePropRedo,
    UpdateVertexPropRedo, UpdateWalUnit, WalCompression, WalConfig, WalContentUnit, WalError,
    WalFileHeader, WalHeader, WalOpType, WalRecoveryMode, WalResult, VertexId,
    WAL_FILE_HEADER_SIZE, WAL_MAGIC, WAL_VERSION,
};
pub use writer::{DummyWalWriter, GroupCommitManager, LocalWalWriter, WalWriter, WalWriterFactory};

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_wal_roundtrip() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let wal_path = temp_dir.path().to_string_lossy().to_string();

        {
            let mut writer = LocalWalWriter::new(&wal_path, 0);
            writer.open().expect("Failed to open WAL");

            writer
                .append_entry(WalOpType::InsertVertex, 1, b"test_data")
                .expect("Failed to append");

            writer.sync().expect("Failed to sync");
        }

        let mut parser = LocalWalParser::new();
        parser.open(&wal_path).expect("Failed to parse WAL");

        let content = parser.get_insert_wal(1).expect("WAL entry not found");
        assert_eq!(content.as_slice(), b"test_data");
    }
}
