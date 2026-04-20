//! Sync Task Definition

use crate::core::Value;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct VectorPointData {
    pub id: String,
    pub vector: Vec<f32>,
    pub payload: HashMap<String, Value>,
}
