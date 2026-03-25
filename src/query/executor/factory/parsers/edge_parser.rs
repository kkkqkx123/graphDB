//! Edge Parser
//!
//! Responsible for parsing edge direction strings

use crate::core::EdgeDirection;

/// Parse a string representing an edge direction into the EdgeDirection enumeration.
pub fn parse_edge_direction(direction_str: &str) -> EdgeDirection {
    match direction_str.to_uppercase().as_str() {
        "OUT" => EdgeDirection::Out,
        "IN" => EdgeDirection::In,
        _ => EdgeDirection::Both,
    }
}
