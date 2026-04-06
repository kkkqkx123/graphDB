use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{Payload, PointId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorPoint {
    pub id: PointId,
    pub vector: Vec<f32>,
    pub payload: Option<Payload>,
}

impl VectorPoint {
    pub fn new(id: impl Into<PointId>, vector: Vec<f32>) -> Self {
        Self {
            id: id.into(),
            vector,
            payload: None,
        }
    }

    pub fn with_payload(mut self, payload: Payload) -> Self {
        self.payload = Some(payload);
        self
    }

    pub fn with_payload_kv(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        let payload = self.payload.get_or_insert_with(HashMap::new);
        payload.insert(key.into(), value);
        self
    }

    pub fn dimension(&self) -> usize {
        self.vector.len()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorPoints {
    pub points: Vec<VectorPoint>,
}

impl VectorPoints {
    pub fn new(points: Vec<VectorPoint>) -> Self {
        Self { points }
    }

    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    pub fn len(&self) -> usize {
        self.points.len()
    }
}

impl From<Vec<VectorPoint>> for VectorPoints {
    fn from(points: Vec<VectorPoint>) -> Self {
        Self::new(points)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertResult {
    pub operation_id: Option<u64>,
    pub status: UpsertStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpsertStatus {
    Completed,
    Acknowledged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteResult {
    pub operation_id: Option<u64>,
    pub deleted_count: u64,
}
