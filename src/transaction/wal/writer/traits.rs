//! WAL Writer trait

use crate::transaction::wal::types::WalResult;

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
