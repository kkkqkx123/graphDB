//! WAL writer factory

use super::dummy::DummyWalWriter;
use super::local::LocalWalWriter;
use super::traits::WalWriter;
use crate::transaction::wal::types::{WalError, WalResult};

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
