//! Index metadata

use serde::{Deserialize, Serialize};

use crate::types::CollectionConfig;

/// Index metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexMetadata {
    pub name: String,
    pub config: CollectionConfig,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub vector_count: u64,
}

impl IndexMetadata {
    pub fn new(name: String, config: CollectionConfig) -> Self {
        Self {
            name,
            config,
            created_at: chrono::Utc::now(),
            vector_count: 0,
        }
    }
}
