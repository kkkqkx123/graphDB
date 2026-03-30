//! Vertex Parser
//!
//! Responsible for parsing the vertex ID string and extracting the vertex IDs from the planning nodes.

use crate::core::Value;
use crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum;

/// Extract the list of vertex IDs from PlanNode.
/// Used to obtain the starting and target vertices in algorithms such as the shortest path algorithm for multiple sources.
pub fn extract_vertex_ids_from_node(node: &PlanNodeEnum) -> Vec<Value> {
    match node {
        PlanNodeEnum::GetVertices(n) => {
            vec![Value::from(format!("vertex_{}", n.id()))]
        }
        PlanNodeEnum::ScanVertices(n) => {
            vec![Value::from(format!("scan_{}", n.id()))]
        }
        PlanNodeEnum::Project(n) => {
            vec![Value::from(format!("project_{}", n.id()))]
        }
        PlanNodeEnum::Start(_) => {
            vec![Value::from("__start__")]
        }
        _ => {
            vec![Value::from(format!("node_{}", node.id()))]
        }
    }
}

/// Parse the vertex ID string into a list of Values.
/// Supports multiple IDs separated by commas.
/// Tries to parse as integer first, then falls back to string.
pub fn parse_vertex_ids(src_vids: &str) -> Vec<Value> {
    src_vids
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| {
            // Try to parse as integer first
            if let Ok(i) = s.parse::<i64>() {
                Value::Int(i)
            } else {
                Value::String(s.to_string())
            }
        })
        .collect()
}
