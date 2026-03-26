//! Integration Testing Shared Tool Module
//!
//! Provide test infrastructure and helper functions for all integration tests

#![allow(dead_code)]

pub mod assertions;
pub mod c_api_helpers;
pub mod data_fixtures;
pub mod storage_helpers;

use graphdb::core::error::DBResult;
use graphdb::storage::redb_storage::RedbStorage;
use parking_lot::Mutex;
use std::path::PathBuf;
use std::sync::Arc;

/// Test Storage Instance Wrapper
///
/// Ensure that each test has a separate storage environment using a temporary folder in the project directory.
/// Automatic cleanup of temporary directories after testing
pub struct TestStorage {
    storage: Arc<Mutex<RedbStorage>>,
    temp_path: PathBuf,
}

impl TestStorage {
    /// Creating a New Test Storage Instance
    pub fn new() -> DBResult<Self> {
        let temp_dir = tempfile::tempdir()?;
        let db_path = temp_dir.path().join("test.db");

        let storage = Arc::new(Mutex::new(RedbStorage::new_with_path(db_path)?));
        Ok(Self {
            storage,
            temp_path: temp_dir.path().to_path_buf(),
        })
    }

    /// Getting a Reference to a Storage Instance
    pub fn storage(&self) -> Arc<Mutex<RedbStorage>> {
        self.storage.clone()
    }
}

impl Drop for TestStorage {
    fn drop(&mut self) {
        // Try to clean up the temporary directory and ignore the error
        let _ = std::fs::remove_dir_all(&self.temp_path);
    }
}
