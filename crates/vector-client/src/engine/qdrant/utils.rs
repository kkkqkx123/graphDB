use qdrant_client::qdrant::{PointId, PointStruct, vectors_output::VectorsOptions};
use qdrant_client::Payload as QdrantPayload;
use std::collections::HashMap;

use crate::error::{Result, VectorClientError};
use crate::types::{VectorPoint, Payload, SearchResult};

pub fn point_id_from_str(id: &str) -> PointId {
    if let Ok(num) = id.parse::<u64>() {
        num.into()
    } else {
        id.into()
    }
}

pub fn point_struct_from_vector_point(point: VectorPoint) -> Result<PointStruct> {
    let id = point_id_from_str(&point.id);
    let payload = payload_to_qdrant_payload(&point.payload)?;
    Ok(PointStruct::new(id, point.vector, payload))
}

pub fn payload_to_qdrant_payload(payload: &Option<Payload>) -> Result<QdrantPayload> {
    let json = match payload {
        Some(p) => serde_json::to_value(p),
        None => Ok(serde_json::Value::Object(Default::default())),
    }
    .map_err(|e| VectorClientError::PayloadError(e.to_string()))?;

    QdrantPayload::try_from(json)
        .map_err(|e| VectorClientError::PayloadError(e.to_string()))
}

pub fn qdrant_payload_to_payload(payload: HashMap<String, qdrant_client::qdrant::Value>) -> Payload {
    let json = serde_json::to_value(payload).unwrap_or(serde_json::Value::Object(Default::default()));
    serde_json::from_value(json).unwrap_or_default()
}

pub fn search_result_from_scored_point(
    point: qdrant_client::qdrant::ScoredPoint,
) -> Result<SearchResult> {
    let id = point.id
        .map(|id| format!("{:?}", id))
        .ok_or_else(|| VectorClientError::InvalidPointId("empty id".to_string()))?;

    let payload = if point.payload.is_empty() {
        None
    } else {
        Some(qdrant_payload_to_payload(point.payload))
    };

    Ok(SearchResult {
        id,
        score: point.score,
        payload,
        vector: None,
    })
}

pub fn vector_point_from_retrieved_point(
    point: qdrant_client::qdrant::RetrievedPoint,
) -> Result<VectorPoint> {
    let id = point.id
        .map(|id| format!("{:?}", id))
        .ok_or_else(|| VectorClientError::InvalidPointId("empty id".to_string()))?;

    let payload = if point.payload.is_empty() {
        None
    } else {
        Some(qdrant_payload_to_payload(point.payload))
    };

    #[allow(deprecated)]
    let vector = point.vectors.and_then(|v| {
        match v.vectors_options {
            Some(VectorsOptions::Vector(vec)) => Some(vec.data),
            _ => None,
        }
    });

    Ok(VectorPoint {
        id,
        vector: vector.unwrap_or_default(),
        payload,
    })
}
