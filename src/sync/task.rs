use crate::coordinator::ChangeType;
use crate::core::Value;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
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

    pub fn task_id(&self) -> &str {
        match self {
            Self::VertexChange { task_id, .. } => task_id,
            Self::BatchIndex { task_id, .. } => task_id,
            Self::CommitIndex { task_id, .. } => task_id,
            Self::RebuildIndex { task_id, .. } => task_id,
        }
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        match self {
            Self::VertexChange { created_at, .. } => *created_at,
            Self::BatchIndex { created_at, .. } => *created_at,
            Self::CommitIndex { created_at, .. } => *created_at,
            Self::RebuildIndex { created_at, .. } => *created_at,
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
