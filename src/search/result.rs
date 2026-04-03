use crate::core::Value;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub doc_id: Value,
    pub score: f32,
    pub highlights: Option<Vec<String>>,
    pub matched_fields: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct IndexStats {
    pub doc_count: usize,
    pub index_size: usize,
    pub last_updated: Option<DateTime<Utc>>,
    pub engine_info: Option<serde_json::Value>,
}
