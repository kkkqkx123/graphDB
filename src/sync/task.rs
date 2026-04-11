//! Sync Task Definition

use crate::coordinator::ChangeType;
use crate::core::Value;
use crate::sync::vector_sync::VectorChangeType;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncTask {
    VertexChange {
        task_id: String,
        space_id: u64,
        tag_name: String,
        vertex_id: Value,
        properties: Vec<(String, Value)>,
        change_type: ChangeType,
        created_at: DateTime<Utc>,
    },
    BatchIndex {
        task_id: String,
        space_id: u64,
        tag_name: String,
        field_name: String,
        documents: Vec<(String, String)>,
        created_at: DateTime<Utc>,
    },
    CommitIndex {
        task_id: String,
        space_id: u64,
        tag_name: String,
        field_name: String,
        created_at: DateTime<Utc>,
    },
    RebuildIndex {
        task_id: String,
        space_id: u64,
        tag_name: String,
        field_name: String,
        created_at: DateTime<Utc>,
    },
    BatchDelete {
        task_id: String,
        space_id: u64,
        tag_name: String,
        field_name: String,
        doc_ids: Vec<String>,
        created_at: DateTime<Utc>,
    },

    VectorChange {
        task_id: String,
        space_id: u64,
        tag_name: String,
        field_name: String,
        vertex_id: Value,
        vector: Option<Vec<f32>>,
        payload: HashMap<String, Value>,
        change_type: VectorChangeType,
        created_at: DateTime<Utc>,
    },
    VectorBatchUpsert {
        task_id: String,
        space_id: u64,
        tag_name: String,
        field_name: String,
        points: Vec<VectorPointData>,
        created_at: DateTime<Utc>,
    },
    VectorBatchDelete {
        task_id: String,
        space_id: u64,
        tag_name: String,
        field_name: String,
        point_ids: Vec<String>,
        created_at: DateTime<Utc>,
    },
    VectorRebuildIndex {
        task_id: String,
        space_id: u64,
        tag_name: String,
        field_name: String,
        created_at: DateTime<Utc>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorPointData {
    pub id: String,
    pub vector: Vec<f32>,
    pub payload: HashMap<String, Value>,
}

impl SyncTask {
    pub fn vertex_change(
        space_id: u64,
        tag_name: &str,
        vertex_id: &Value,
        properties: Vec<(String, Value)>,
        change_type: ChangeType,
    ) -> Self {
        Self::VertexChange {
            task_id: generate_task_id(),
            space_id,
            tag_name: tag_name.to_string(),
            vertex_id: vertex_id.clone(),
            properties,
            change_type,
            created_at: Utc::now(),
        }
    }

    pub fn batch_index(
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        documents: Vec<(String, String)>,
    ) -> Self {
        Self::BatchIndex {
            task_id: generate_task_id(),
            space_id,
            tag_name: tag_name.to_string(),
            field_name: field_name.to_string(),
            documents,
            created_at: Utc::now(),
        }
    }

    pub fn commit_index(space_id: u64, tag_name: &str, field_name: &str) -> Self {
        Self::CommitIndex {
            task_id: generate_task_id(),
            space_id,
            tag_name: tag_name.to_string(),
            field_name: field_name.to_string(),
            created_at: Utc::now(),
        }
    }

    pub fn rebuild_index(space_id: u64, tag_name: &str, field_name: &str) -> Self {
        Self::RebuildIndex {
            task_id: generate_task_id(),
            space_id,
            tag_name: tag_name.to_string(),
            field_name: field_name.to_string(),
            created_at: Utc::now(),
        }
    }

    pub fn batch_delete(
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        doc_ids: Vec<String>,
    ) -> Self {
        Self::BatchDelete {
            task_id: generate_task_id(),
            space_id,
            tag_name: tag_name.to_string(),
            field_name: field_name.to_string(),
            doc_ids,
            created_at: Utc::now(),
        }
    }

    pub fn vector_change(
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        vertex_id: &Value,
        vector: Option<Vec<f32>>,
        payload: HashMap<String, Value>,
        change_type: VectorChangeType,
    ) -> Self {
        Self::VectorChange {
            task_id: generate_task_id(),
            space_id,
            tag_name: tag_name.to_string(),
            field_name: field_name.to_string(),
            vertex_id: vertex_id.clone(),
            vector,
            payload,
            change_type,
            created_at: Utc::now(),
        }
    }

    pub fn vector_batch_upsert(
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        points: Vec<VectorPointData>,
    ) -> Self {
        Self::VectorBatchUpsert {
            task_id: generate_task_id(),
            space_id,
            tag_name: tag_name.to_string(),
            field_name: field_name.to_string(),
            points,
            created_at: Utc::now(),
        }
    }

    pub fn vector_batch_delete(
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        point_ids: Vec<String>,
    ) -> Self {
        Self::VectorBatchDelete {
            task_id: generate_task_id(),
            space_id,
            tag_name: tag_name.to_string(),
            field_name: field_name.to_string(),
            point_ids,
            created_at: Utc::now(),
        }
    }

    pub fn vector_rebuild_index(space_id: u64, tag_name: &str, field_name: &str) -> Self {
        Self::VectorRebuildIndex {
            task_id: generate_task_id(),
            space_id,
            tag_name: tag_name.to_string(),
            field_name: field_name.to_string(),
            created_at: Utc::now(),
        }
    }

    pub fn task_id(&self) -> &str {
        match self {
            Self::VertexChange { task_id, .. } => task_id,
            Self::BatchIndex { task_id, .. } => task_id,
            Self::CommitIndex { task_id, .. } => task_id,
            Self::RebuildIndex { task_id, .. } => task_id,
            Self::BatchDelete { task_id, .. } => task_id,
            Self::VectorChange { task_id, .. } => task_id,
            Self::VectorBatchUpsert { task_id, .. } => task_id,
            Self::VectorBatchDelete { task_id, .. } => task_id,
            Self::VectorRebuildIndex { task_id, .. } => task_id,
        }
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        match self {
            Self::VertexChange { created_at, .. } => *created_at,
            Self::BatchIndex { created_at, .. } => *created_at,
            Self::CommitIndex { created_at, .. } => *created_at,
            Self::RebuildIndex { created_at, .. } => *created_at,
            Self::BatchDelete { created_at, .. } => *created_at,
            Self::VectorChange { created_at, .. } => *created_at,
            Self::VectorBatchUpsert { created_at, .. } => *created_at,
            Self::VectorBatchDelete { created_at, .. } => *created_at,
            Self::VectorRebuildIndex { created_at, .. } => *created_at,
        }
    }

    pub fn is_vector_task(&self) -> bool {
        matches!(
            self,
            Self::VectorChange { .. }
                | Self::VectorBatchUpsert { .. }
                | Self::VectorBatchDelete { .. }
                | Self::VectorRebuildIndex { .. }
        )
    }

    pub fn priority(&self) -> u8 {
        match self {
            Self::BatchDelete { .. } => 10,
            Self::VectorBatchDelete { .. } => 10,

            Self::VertexChange { .. } => 5,
            Self::VectorChange { .. } => 5,

            Self::BatchIndex { .. } => 3,
            Self::VectorBatchUpsert { .. } => 3,

            Self::RebuildIndex { .. } => 1,
            Self::VectorRebuildIndex { .. } => 1,
            Self::CommitIndex { .. } => 1,
        }
    }
}

fn generate_task_id() -> String {
    use uuid::Uuid;
    Uuid::new_v4().to_string()
}

#[derive(Debug, Clone)]
pub enum TaskResult {
    Success,
    Failed(String),
    Retryable(String),
}
