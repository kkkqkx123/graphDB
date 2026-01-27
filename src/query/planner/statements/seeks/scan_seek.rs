//! 扫描查找策略
//!
//! 全表扫描策略，作为无法使用索引时的回退方案

use super::seek_strategy::{SeekStrategy, SeekStrategyTraitObject};
use super::seek_strategy_base::{SeekResult, SeekStrategyContext, SeekStrategyType, NodePattern};
use crate::core::{StorageError, Vertex};
use crate::storage::StorageEngine;

#[derive(Debug, Clone)]
pub struct ScanSeek;

impl ScanSeek {
    pub fn new() -> Self {
        Self
    }
}

impl SeekStrategy for ScanSeek {
    fn execute(
        &self,
        storage: &dyn StorageEngine,
        context: &SeekStrategyContext,
    ) -> Result<SeekResult, StorageError> {
        let vertices = storage.scan_all_vertices()?;
        let mut vertex_ids = Vec::new();
        let mut rows_scanned = 0;

        for vertex in vertices {
            rows_scanned += 1;
            if self.vertex_matches_pattern(&vertex, &context.node_pattern) {
                vertex_ids.push(vertex.vid().clone());
            }
        }

        Ok(SeekResult {
            vertex_ids,
            strategy_used: SeekStrategyType::ScanSeek,
            rows_scanned,
        })
    }

    fn estimated_cost(&self, context: &SeekStrategyContext) -> f64 {
        context.estimated_rows as f64
    }

    fn supports(&self, _context: &SeekStrategyContext) -> bool {
        true
    }
}

impl ScanSeek {
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

    #[test]
    fn test_scan_seek_new() {
        let seek = ScanSeek::new();
        assert!(true);
    }

    #[test]
    fn test_scan_seek_supports_always() {
        let seek = ScanSeek::new();
        let context = SeekStrategyContext::new(
            1,
            NodePattern {
                vid: None,
                labels: vec![],
                properties: vec![],
            },
            vec![],
        );
        assert!(seek.supports(&context));
    }

    #[test]
    fn test_scan_seek_cost() {
        let seek = ScanSeek::new();
        let context = SeekStrategyContext::new(
            1,
            NodePattern {
                vid: None,
                labels: vec![],
                properties: vec![],
            },
            vec![],
        )
        .with_estimated_rows(10000);
        assert_eq!(seek.estimated_cost(&context), 10000.0);
    }
}
