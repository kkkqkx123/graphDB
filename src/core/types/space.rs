//! Basic types in the graph space

use crate::core::types::{DataType, EdgeTypeInfo, MetadataVersion, TagInfo};
use oxicode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

/// Charset and collation information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct CharsetInfo {
    pub charset: String,
    pub collation: String,
}

/// Isolation level for space storage
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize, Encode, Decode)]
pub enum IsolationLevel {
    /// Shared storage (default) - all spaces share the same base path
    #[default]
    Shared,
    /// Independent subdirectory - each space has its own subdirectory
    Directory,
    /// Independent storage device - each space can have a custom storage path
    Device,
}

static SPACE_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

pub fn generate_space_id() -> u64 {
    SPACE_ID_COUNTER.fetch_add(1, Ordering::SeqCst)
}

pub fn reset_space_id_counter() {
    SPACE_ID_COUNTER.store(1, Ordering::SeqCst);
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub struct SpaceInfo {
    pub space_id: u64,
    pub space_name: String,
    pub vid_type: DataType,
    pub tags: Vec<TagInfo>,
    pub edge_types: Vec<EdgeTypeInfo>,
    pub version: MetadataVersion,
    pub comment: Option<String>,
    /// Custom storage path for this space (optional)
    pub storage_path: Option<PathBuf>,
    /// Isolation level for storage
    pub isolation_level: IsolationLevel,
}

impl SpaceInfo {
    pub fn new(space_name: String) -> Self {
        Self {
            space_id: generate_space_id(),
            space_name,
            vid_type: DataType::String,
            tags: Vec::new(),
            edge_types: Vec::new(),
            version: MetadataVersion::default(),
            comment: None,
            storage_path: None,
            isolation_level: IsolationLevel::default(),
        }
    }

    pub fn with_vid_type(mut self, vid_type: DataType) -> Self {
        self.vid_type = vid_type;
        self
    }

    pub fn with_comment(mut self, comment: Option<String>) -> Self {
        self.comment = comment;
        self
    }

    pub fn with_storage_path(mut self, storage_path: Option<PathBuf>) -> Self {
        self.storage_path = storage_path;
        if self.storage_path.is_some() {
            self.isolation_level = IsolationLevel::Device;
        }
        self
    }

    pub fn with_isolation_level(mut self, isolation_level: IsolationLevel) -> Self {
        self.isolation_level = isolation_level;
        self
    }
}

impl Default for SpaceInfo {
    fn default() -> Self {
        Self::new("default".to_string())
    }
}
