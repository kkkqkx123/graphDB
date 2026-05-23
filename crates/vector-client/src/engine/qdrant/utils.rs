use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
pub struct QdrantSearchResult {
    pub id: PointIdValue,
    pub score: f32,
    #[serde(default)]
    pub payload: Option<Value>,
    #[serde(default)]
    pub vector: Option<VectorValue>,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum PointIdValue {
    Uuid(String),
    Num(u64),
}

impl std::fmt::Display for PointIdValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PointIdValue::Uuid(s) => write!(f, "{}", s),
            PointIdValue::Num(n) => write!(f, "{}", n),
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum VectorValue {
    Single(Vec<f32>),
    Multi { data: Vec<f32> },
    #[allow(dead_code)]
    Named(std::collections::HashMap<String, Value>),
}

impl VectorValue {
    pub fn into_vec(self) -> Option<Vec<f32>> {
        match self {
            VectorValue::Single(v) => Some(v),
            VectorValue::Multi { data } => Some(data),
            VectorValue::Named(_) => None,
        }
    }
}

#[derive(Deserialize)]
pub struct QdrantUpsertResult {
    pub operation_id: Option<u64>,
    #[allow(dead_code)]
    pub status: Option<String>,
}

pub fn point_id_json(id: &str) -> Value {
    if let Ok(num) = id.parse::<u64>() {
        serde_json::json!(num)
    } else {
        serde_json::json!(id)
    }
}

pub fn parse_payload(payload: Option<Value>) -> Option<std::collections::HashMap<String, Value>> {
    payload.and_then(|v| match v {
        Value::Object(map) => {
            let result: std::collections::HashMap<String, Value> = map.into_iter().collect();
            Some(result)
        }
        _ => None,
    })
}
