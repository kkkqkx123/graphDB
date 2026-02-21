//! 图空间基础类型

use crate::core::types::{DataType, TagInfo, EdgeTypeInfo, MetadataVersion};
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};

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
}

impl Default for SpaceInfo {
    fn default() -> Self {
        Self::new("default".to_string())
    }
}
