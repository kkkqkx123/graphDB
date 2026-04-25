//! Storage Module Sharing Types
//!
//! Define common data structures and types shared by all storage implementations

use serde::{Deserialize, Serialize};

/// Storing Information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageInfo {
    pub name: String,
    pub version: String,
    pub size: u64,
    pub document_count: usize,
    pub index_count: usize,
    pub is_connected: bool,
}

/// File storage data format
///
/// Serialization formats for file storage and cache storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStorageData {
    pub version: String,
    pub timestamp: String,
    pub data: std::collections::HashMap<String, Vec<crate::r#type::DocId>>,
    pub context_data: std::collections::HashMap<
        String,
        std::collections::HashMap<String, Vec<crate::r#type::DocId>>,
    >,
    pub documents: std::collections::HashMap<crate::r#type::DocId, String>,
}
