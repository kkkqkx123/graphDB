use std::collections::{HashMap, HashSet};
use crate::core::{Value, NullType, Vertex, Edge};
use super::base::EvaluationError;
use crate::expressions::value::Expression;

/// Evaluate property access operation
pub fn eval_property_access(entity: Value, property: &str) -> Result<Value, EvaluationError> {
    // This is a simplified implementation. In a full NebulaGraph implementation,
    // this would access properties of complex types like Vertex, Edge, etc.
    match entity {
        Value::Vertex(vertex) => {
            match property {
                "id" => Ok(Value::String(vertex.vid.as_ref().to_string())),
                "tags" => Ok(Value::List(
                    vertex.tags
                        .iter()
                        .map(|tag| Value::String(tag.name.clone()))
                        .collect()
                )),
                _ => Err(EvaluationError::Other(
                    format!("Property '{}' not found on vertex", property)
                )),
            }
        },
        Value::Edge(edge) => {
            match property {
                "src" => Ok(Value::String(edge.src.to_string())),
                "dst" => Ok(Value::String(edge.dst.to_string())),
                "type" => Ok(Value::String(edge.edge_type)),
                "rank" => Ok(Value::Int(edge.ranking)),
                _ => Err(EvaluationError::Other(
                    format!("Property '{}' not found on edge", property)
                )),
            }
        },
        Value::Map(map) => {
            map.get(property)
                .cloned()
                .ok_or_else(|| EvaluationError::Other(
                    format!("Property '{}' not found in map", property)
                ))
        },
        _ => Err(EvaluationError::TypeError(
            format!("Property access not supported on {:?}", entity)
        )),
    }
}