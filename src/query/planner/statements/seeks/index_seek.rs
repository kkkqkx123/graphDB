//! 索引查找策略
//!
//! 使用标签或属性索引进行高效查找

use super::seek_strategy::SeekStrategy;
use super::seek_strategy_base::{SeekResult, SeekStrategyContext, SeekStrategyType, NodePattern};
use crate::core::{StorageError, Vertex};
use crate::storage::StorageClient;

#[derive(Debug, Clone)]
pub struct IndexSeek;

impl IndexSeek {
    pub fn new() -> Self {
        Self
    }
}

impl SeekStrategy for IndexSeek {
    fn execute<S: StorageClient>(
        &self,
        storage: &S,
        context: &SeekStrategyContext,
    ) -> Result<SeekResult, StorageError> {
        let mut vertex_ids = Vec::new();
        let mut rows_scanned = 0;

        if let Some(index_info) = context.get_index_for_labels(&context.node_pattern.labels) {
            let vertices = storage.scan_vertices_by_tag("default", &index_info.target_name)?;
            rows_scanned = vertices.len();
            for vertex in vertices {
                if self.vertex_matches_pattern(&vertex, &context.node_pattern) {
                    vertex_ids.push(vertex.vid().clone());
                }
            }
        }

        if vertex_ids.is_empty() {
            rows_scanned = 0;
        }

        Ok(SeekResult {
            vertex_ids,
            strategy_used: SeekStrategyType::IndexSeek,
            rows_scanned,
        })
    }

    fn estimated_cost(&self, context: &SeekStrategyContext) -> f64 {
        if context.has_explicit_vid() {
            1.0
        } else if context.has_predicates() {
            10.0
        } else {
            50.0
        }
    }

    fn supports(&self, context: &SeekStrategyContext) -> bool {
        context.get_index_for_labels(&context.node_pattern.labels).is_some()
    }
}

impl IndexSeek {
    fn vertex_matches_pattern(&self, vertex: &Vertex, pattern: &NodePattern) -> bool {
        if !pattern.labels.is_empty() {
            let has_all_labels = pattern.labels.iter().all(|label| {
                vertex.tags.iter().any(|tag| tag.name == *label)
            });
            if !has_all_labels {
                return false;
            }
        }

        for (prop_name, prop_value) in &pattern.properties {
            let found = vertex.get_all_properties().iter().any(|(name, value)| {
                name == prop_name && **value == *prop_value
            });
            if !found {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::seek_strategy_base::IndexInfo;

    #[test]
    fn test_index_seek_new() {
        let _ = IndexSeek::new();
        assert!(true);
    }

    #[test]
    fn test_index_seek_supports_with_index() {
        let seek = IndexSeek::new();
        let context = SeekStrategyContext::new(
            1,
            NodePattern {
                vid: None,
                labels: vec!["person".to_string()],
                properties: vec![],
            },
            vec![],
        )
        .with_indexes(vec![IndexInfo {
            name: "idx_person_name".to_string(),
            target_type: "tag".to_string(),
            target_name: "person".to_string(),
            properties: vec!["name".to_string()],
        }]);
        assert!(seek.supports(&context));
    }

    #[test]
    fn test_index_seek_supports_without_index() {
        let seek = IndexSeek::new();
        let context = SeekStrategyContext::new(
            1,
            NodePattern {
                vid: None,
                labels: vec!["person".to_string()],
                properties: vec![],
            },
            vec![],
        );
        assert!(!seek.supports(&context));
    }
}
