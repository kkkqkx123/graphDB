//! 边查找策略
//!
//! 从边模式开始的查找策略，用于 MATCH 从边开始的情况
//!
//! 适用场景:
//! - MATCH ()-[e:KNOWS]->() WHERE e.since > 2020
//! - MATCH (a)-[e]->(b) WHERE e.weight > 5
//! - 从边索引开始查找

use super::seek_strategy::SeekStrategy;
use super::seek_strategy_base::{SeekResult, SeekStrategyContext, SeekStrategyType};
use crate::core::{StorageError, Value};
use crate::storage::StorageClient;

/// 边模式信息
#[derive(Debug, Clone, PartialEq)]
pub struct EdgePattern {
    pub edge_types: Vec<String>,
    pub direction: EdgeDirection,
    pub src_vid: Option<Value>,
    pub dst_vid: Option<Value>,
    pub properties: Vec<(String, Value)>,
}

/// 边方向
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeDirection {
    Outgoing,  // ->
    Incoming,  // <-
    Both,      // -
}

impl EdgeDirection {
    /// 从字符串解析方向
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "OUT" | "OUTGOING" | "->" => Some(EdgeDirection::Outgoing),
            "IN" | "INCOMING" | "<-" => Some(EdgeDirection::Incoming),
            "BOTH" | "-" => Some(EdgeDirection::Both),
            _ => None,
        }
    }

    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            EdgeDirection::Outgoing => "OUT",
            EdgeDirection::Incoming => "IN",
            EdgeDirection::Both => "BOTH",
        }
    }
}

/// 边查找策略
#[derive(Debug, Clone)]
pub struct EdgeSeek {
    pub edge_pattern: EdgePattern,
}

impl EdgeSeek {
    pub fn new(edge_pattern: EdgePattern) -> Self {
        Self { edge_pattern }
    }

    /// 评估边是否匹配模式
    fn edge_matches_pattern(&self, edge: &crate::core::Edge) -> bool {
        // 检查边类型
        if !self.edge_pattern.edge_types.is_empty() {
            if !self.edge_pattern.edge_types.contains(&edge.edge_type) {
                return false;
            }
        }

        // 检查源顶点ID
        if let Some(ref src_vid) = self.edge_pattern.src_vid {
            if *edge.src != *src_vid {
                return false;
            }
        }

        // 检查目标顶点ID
        if let Some(ref dst_vid) = self.edge_pattern.dst_vid {
            if *edge.dst != *dst_vid {
                return false;
            }
        }

        // 检查属性
        for (prop_name, prop_value) in &self.edge_pattern.properties {
            let found = edge.get_property(prop_name).map(|v| v == prop_value).unwrap_or(false);
            if !found {
                return false;
            }
        }

        true
    }
}

impl SeekStrategy for EdgeSeek {
    fn execute<S: StorageClient>(
        &self,
        storage: &S,
        _context: &SeekStrategyContext,
    ) -> Result<SeekResult, StorageError> {
        let mut edge_ids = Vec::new();
        let mut rows_scanned = 0;

        let space_name = "default"; // 实际应从 context 获取

        // 策略1: 如果有明确的边类型，使用边类型扫描
        if !self.edge_pattern.edge_types.is_empty() {
            for edge_type in &self.edge_pattern.edge_types {
                let edges = storage.scan_edges_by_type(space_name, edge_type)?;
                rows_scanned += edges.len();

                for edge in edges {
                    if self.edge_matches_pattern(&edge) {
                        // 使用边的唯一标识: src->dst@edge_type
                        let edge_id = format!("{}->{}@{}", 
                            edge.src, edge.dst, edge.edge_type);
                        edge_ids.push(Value::String(edge_id));
                    }
                }
            }
        } else {
            // 策略2: 全表扫描所有边
            let edges = storage.scan_all_edges(space_name)?;
            rows_scanned = edges.len();

            for edge in edges {
                if self.edge_matches_pattern(&edge) {
                    let edge_id = format!("{}->{}@{}", 
                        edge.src, edge.dst, edge.edge_type);
                    edge_ids.push(Value::String(edge_id));
                }
            }
        }

        // 去重
        edge_ids.sort();
        edge_ids.dedup();

        Ok(SeekResult {
            vertex_ids: edge_ids, // 这里存储的是边ID
            strategy_used: SeekStrategyType::EdgeSeek,
            rows_scanned,
        })
    }

    fn estimated_cost(&self, _context: &SeekStrategyContext) -> f64 {
        if !self.edge_pattern.edge_types.is_empty() {
            if self.edge_pattern.src_vid.is_some() || self.edge_pattern.dst_vid.is_some() {
                10.0 // 有明确顶点ID，成本较低
            } else {
                40.0 // 只有边类型，成本中等
            }
        } else {
            100.0 // 全表扫描，成本较高
        }
    }

    fn supports(&self, _context: &SeekStrategyContext) -> bool {
        // 只要有边模式就支持
        true
    }
}

/// 边查找结果扩展
#[derive(Debug)]
pub struct EdgeSeekResult {
    pub base: SeekResult,
    pub start_vids: Vec<Value>,
    pub end_vids: Vec<Value>,
}

impl EdgeSeekResult {
    pub fn from_seek_result(result: SeekResult, start_vids: Vec<Value>, end_vids: Vec<Value>) -> Self {
        Self {
            base: result,
            start_vids,
            end_vids,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_direction_from_str() {
        assert_eq!(EdgeDirection::from_str("OUT"), Some(EdgeDirection::Outgoing));
        assert_eq!(EdgeDirection::from_str("->"), Some(EdgeDirection::Outgoing));
        assert_eq!(EdgeDirection::from_str("IN"), Some(EdgeDirection::Incoming));
        assert_eq!(EdgeDirection::from_str("<-"), Some(EdgeDirection::Incoming));
        assert_eq!(EdgeDirection::from_str("BOTH"), Some(EdgeDirection::Both));
        assert_eq!(EdgeDirection::from_str("-"), Some(EdgeDirection::Both));
        assert_eq!(EdgeDirection::from_str("unknown"), None);
    }

    #[test]
    fn test_edge_pattern_matching() {
        let seek = EdgeSeek::new(EdgePattern {
            edge_types: vec!["KNOWS".to_string()],
            direction: EdgeDirection::Outgoing,
            src_vid: None,
            dst_vid: None,
            properties: vec![],
        });

        // 测试边类型匹配
        let edge = crate::core::Edge::new_empty(
            Value::Int(1),
            Value::Int(2),
            "KNOWS".to_string(),
            0,
        );
        assert!(seek.edge_matches_pattern(&edge));

        // 测试边类型不匹配
        let edge2 = crate::core::Edge::new_empty(
            Value::Int(1),
            Value::Int(2),
            "FOLLOWS".to_string(),
            0,
        );
        assert!(!seek.edge_matches_pattern(&edge2));
    }

    #[test]
    fn test_edge_pattern_with_src_vid() {
        let seek = EdgeSeek::new(EdgePattern {
            edge_types: vec![],
            direction: EdgeDirection::Outgoing,
            src_vid: Some(Value::Int(1)),
            dst_vid: None,
            properties: vec![],
        });

        let edge = crate::core::Edge::new_empty(
            Value::Int(1),
            Value::Int(2),
            "KNOWS".to_string(),
            0,
        );
        assert!(seek.edge_matches_pattern(&edge));

        let edge2 = crate::core::Edge::new_empty(
            Value::Int(3),
            Value::Int(2),
            "KNOWS".to_string(),
            0,
        );
        assert!(!seek.edge_matches_pattern(&edge2));
    }
}
