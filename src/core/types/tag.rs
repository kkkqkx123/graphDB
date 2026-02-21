//! 标签基础类型

use super::property::PropertyDef;
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub struct TagInfo {
    pub tag_id: i32,
    pub tag_name: String,
    pub properties: Vec<PropertyDef>,
    pub comment: Option<String>,
    pub ttl_duration: Option<i64>,
    pub ttl_col: Option<String>,
}

impl TagInfo {
    pub fn new(tag_name: String) -> Self {
        Self {
            tag_id: 0,
            tag_name,
            properties: Vec::new(),
            comment: None,
            ttl_duration: None,
            ttl_col: None,
        }
    }

    pub fn with_properties(mut self, properties: Vec<PropertyDef>) -> Self {
        self.properties = properties;
        self
    }

    pub fn with_comment(mut self, comment: Option<String>) -> Self {
        self.comment = comment;
        self
    }

    pub fn with_ttl(mut self, duration: Option<i64>, col: Option<String>) -> Self {
        self.ttl_duration = duration;
        self.ttl_col = col;
        self
    }
}

impl Default for TagInfo {
    fn default() -> Self {
        Self::new("default".to_string())
    }
}
