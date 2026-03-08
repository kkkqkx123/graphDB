//! 导入导出类型定义

use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct SchemaExportConfig {
    pub space_id: Option<u64>,
    pub format: ExportFormat,
    pub include_comments: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub enum ExportFormat {
    JSON,
    YAML,
    Rust,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct SchemaImportResult {
    pub success: bool,
    pub space_name: String,
    pub imported_items: i32,
    pub imported_tags: Vec<String>,
    pub imported_edge_types: Vec<String>,
    pub skipped_items: Vec<String>,
    pub errors: Vec<String>,
}

impl Default for SchemaImportResult {
    fn default() -> Self {
        Self {
            success: false,
            space_name: String::new(),
            imported_items: 0,
            imported_tags: Vec::new(),
            imported_edge_types: Vec::new(),
            skipped_items: Vec::new(),
            errors: Vec::new(),
        }
    }
}

impl SchemaImportResult {
    pub fn new() -> Self {
        Self::default()
    }
}
