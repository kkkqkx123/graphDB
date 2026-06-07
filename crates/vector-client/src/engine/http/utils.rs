use std::collections::HashMap;

use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
pub struct QdrantSearchResult {
    pub id: crate::types::PointId,
    pub score: f32,
    #[serde(default)]
    pub payload: Option<Value>,
    #[serde(default)]
    pub vector: Option<VectorValue>,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum VectorValue {
    Single(Vec<f32>),
    Multi { data: Vec<f32> },
    Named(HashMap<String, Value>),
}

impl VectorValue {
    pub fn into_vec(self) -> Option<Vec<f32>> {
        match self {
            VectorValue::Single(v) => Some(v),
            VectorValue::Multi { data } => Some(data),
            VectorValue::Named(named) => {
                if named.len() == 1 {
                    named.into_values().next().and_then(value_to_vec)
                } else {
                    None
                }
            }
        }
    }
}

#[derive(Deserialize)]
pub struct QdrantUpsertResult {
    pub operation_id: Option<u64>,
    pub status: Option<String>,
}

pub fn parse_payload(payload: Option<Value>) -> Option<HashMap<String, Value>> {
    payload.and_then(|v| match v {
        Value::Object(map) => {
            let result: HashMap<String, Value> = map.into_iter().collect();
            Some(result)
        }
        _ => None,
    })
}

fn value_to_vec(value: Value) -> Option<Vec<f32>> {
    match value {
        Value::Array(values) => values
            .into_iter()
            .map(|entry| match entry {
                Value::Number(number) => number.as_f64().map(|v| v as f32),
                _ => None,
            })
            .collect(),
        _ => None,
    }
}
