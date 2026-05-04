//! Integration Testing Shared Tool Module
//!
//! Provide test infrastructure and helper functions for all integration tests

#![allow(dead_code)]

pub mod assertions;
pub mod data_fixtures;
pub mod debug_helpers;
pub mod fulltext_helpers;
pub mod query_helpers;
pub mod storage_helpers;
pub mod sync_helpers;
pub mod test_scenario;
pub mod transaction_helpers;
pub mod validation_helpers;

// C API helpers only compiled when c-api feature is enabled
#[cfg(feature = "c-api")]
pub mod c_api_helpers;

use graphdb::core::error::DBResult;
use graphdb::storage::metadata::InMemorySchemaManager;
use graphdb::storage::GraphStorage;
use parking_lot::Mutex;
use std::path::PathBuf;
use std::sync::Arc;

/// Test Storage Instance Wrapper
///
/// Ensure that each test has a separate storage environment using a temporary folder in the project directory.
/// Automatic cleanup of temporary directories after testing
pub struct TestStorage {
    storage: Arc<Mutex<GraphStorage>>,
    temp_path: PathBuf,
}

impl TestStorage {
    /// Creating a New Test Storage Instance
    pub fn new() -> DBResult<Self> {
        let temp_dir = tempfile::tempdir()?;
        let db_path = temp_dir.path().join("test.db");

        let storage = Arc::new(Mutex::new(GraphStorage::new_with_path(db_path)?));
        Ok(Self {
            storage,
            temp_path: temp_dir.path().to_path_buf(),
        })
    }

    /// Getting a Reference to a Storage Instance
    pub fn storage(&self) -> Arc<Mutex<GraphStorage>> {
        self.storage.clone()
    }

    /// Getting the Schema Manager from Storage
    pub fn schema_manager(&self) -> Arc<InMemorySchemaManager> {
        let storage = self.storage.lock();
        storage.get_schema_manager()
    }
}

impl Drop for TestStorage {
    fn drop(&mut self) {
        // Try to clean up the temporary directory and ignore the error
        let _ = std::fs::remove_dir_all(&self.temp_path);
    }
}
