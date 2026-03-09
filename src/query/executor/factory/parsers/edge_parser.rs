//! 边解析器
//!
//! 负责解析边方向字符串

use crate::core::EdgeDirection;

/// 解析边方向字符串为 EdgeDirection 枚举
pub fn parse_edge_direction(direction_str: &str) -> EdgeDirection {
    match direction_str.to_uppercase().as_str() {
        "OUT" => EdgeDirection::Out,
        "IN" => EdgeDirection::In,
        _ => EdgeDirection::Both,
    }
}
