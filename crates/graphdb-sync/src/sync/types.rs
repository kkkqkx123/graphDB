use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    Insert,
    Update,
    Delete,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IndexOpKey {
    pub space_id: u64,
    pub tag_name: String,
    pub field_name: String,
}

impl IndexOpKey {
    pub fn new(space_id: u64, tag_name: impl Into<String>, field_name: impl Into<String>) -> Self {
        Self {
            space_id,
            tag_name: tag_name.into(),
            field_name: field_name.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndexOperation {
    Insert {
        key: IndexOpKey,
        id: String,
        text: String,
    },
    Delete {
        key: IndexOpKey,
        id: String,
    },
    Update {
        key: IndexOpKey,
        id: String,
        text: String,
    },
}

impl IndexOperation {
    pub fn extract_index_key(&self) -> Option<(u64, String, String)> {
        match self {
            IndexOperation::Insert { key, .. }
            | IndexOperation::Update { key, .. }
            | IndexOperation::Delete { key, .. } => {
                Some((key.space_id, key.tag_name.clone(), key.field_name.clone()))
            }
        }
    }
}
