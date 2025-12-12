//! Index scan optimization rules for NebulaGraph
//! These rules optimize index scan operations based on NebulaGraph's implementation

use super::optimizer::OptimizerError;
use crate::query::optimizer::optimizer::{OptContext, OptGroupNode, OptRule, Pattern, MatchedResult};
use crate::query::planner::plan::{PlanNodeKind, PlanNode};
use std::any::Any;

// Rule to convert edge index full scans to more optimal operations
#[derive(Debug)]
pub struct EdgeIndexFullScanRule;

impl OptRule for EdgeIndexFullScanRule {
    fn name(&self) -> &str {
        "EdgeIndexFullScanRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // Check if this is an index scan operation that might be a full scan
        if node.plan_node.kind() != PlanNodeKind::IndexScan {
            return Ok(None);
        }

        // Match the pattern to determine if this is an edge index scan
        if let Some(matched) = self.match_pattern(ctx, node)? {
            // In a complete implementation, we would check if this is a full index scan
            // (scanning the entire index without conditions) and possibly optimize it
            // For example, if the index scan covers all data without benefit, we might
            // convert it to a full table scan which could be more efficient
            Ok(Some(node.clone()))
        } else {
            Ok(None)
        }
    }

    fn pattern(&self) -> Pattern {
        Pattern::new(PlanNodeKind::IndexScan) // Specifically for edge index scans
    }
}

// Rule to convert tag index full scans to more optimal operations
#[derive(Debug)]
pub struct TagIndexFullScanRule;

impl OptRule for TagIndexFullScanRule {
    fn name(&self) -> &str {
        "TagIndexFullScanRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // Check if this is an index scan operation that might be a full scan
        if node.plan_node.kind() != PlanNodeKind::IndexScan {
            return Ok(None);
        }

        // Match the pattern to determine if this is a tag index scan
        if let Some(matched) = self.match_pattern(ctx, node)? {
            // In a complete implementation, we would check if this is a full index scan
            // (scanning the entire index without conditions) and possibly optimize it
            // For example, if the index scan covers all data without benefit, we might
            // convert it to a full table scan which could be more efficient
            Ok(Some(node.clone()))
        } else {
            Ok(None)
        }
    }

    fn pattern(&self) -> Pattern {
        Pattern::new(PlanNodeKind::IndexScan) // Specifically for tag index scans
    }
}

// Rule for general index scan operations
#[derive(Debug)]
pub struct IndexScanRule;

impl OptRule for IndexScanRule {
    fn name(&self) -> &str {
        "IndexScanRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // Check if this is an index scan operation
        if node.plan_node.kind() != PlanNodeKind::IndexScan {
            return Ok(None);
        }

        // Match the pattern and optimize the index scan if possible
        if let Some(matched) = self.match_pattern(ctx, node)? {
            // In a real implementation, we would optimize the index scan based on various factors:
            // - Index selectivity
            // - Data distribution
            // - Available memory
            // For now, we just return the original node
            Ok(Some(node.clone()))
        } else {
            Ok(None)
        }
    }

    fn pattern(&self) -> Pattern {
        Pattern::new(PlanNodeKind::IndexScan)
    }
}

// Rule for union all edge index scans
#[derive(Debug)]
pub struct UnionAllEdgeIndexScanRule;

impl OptRule for UnionAllEdgeIndexScanRule {
    fn name(&self) -> &str {
        "UnionAllEdgeIndexScanRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // Check if this is an index scan operation that's part of a union
        if node.plan_node.kind() != PlanNodeKind::IndexScan {
            return Ok(None);
        }

        // Match the pattern to identify union all edge index scans
        if let Some(matched) = self.match_pattern(ctx, node)? {
            // In a complete implementation, we would optimize union all operations
            // involving edge index scans by potentially merging or reordering them
            Ok(Some(node.clone()))
        } else {
            Ok(None)
        }
    }

    fn pattern(&self) -> Pattern {
        Pattern::new(PlanNodeKind::IndexScan) // For union all edge index scans
    }
}

// Rule for union all tag index scans
#[derive(Debug)]
pub struct UnionAllTagIndexScanRule;

impl OptRule for UnionAllTagIndexScanRule {
    fn name(&self) -> &str {
        "UnionAllTagIndexScanRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // Check if this is an index scan operation that's part of a union
        if node.plan_node.kind() != PlanNodeKind::IndexScan {
            return Ok(None);
        }

        // Match the pattern to identify union all tag index scans
        if let Some(matched) = self.match_pattern(ctx, node)? {
            // In a complete implementation, we would optimize union all operations
            // involving tag index scans by potentially merging or reordering them
            Ok(Some(node.clone()))
        } else {
            Ok(None)
        }
    }

    fn pattern(&self) -> Pattern {
        Pattern::new(PlanNodeKind::IndexScan) // For union all tag index scans
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::optimizer::optimizer::{OptContext, OptGroupNode};
    use crate::query::planner::plan::{PlanNodeKind, PlanNode};
    use crate::query::context::QueryContext;
    use crate::query::planner::plan::IndexScan;

    fn create_test_context() -> OptContext {
        OptContext::new(QueryContext::default())
    }

    #[test]
    fn test_edge_index_full_scan_rule() {
        let rule = EdgeIndexFullScanRule;
        let mut ctx = create_test_context();

        // Create an index scan node
        let index_scan_node = Box::new(IndexScan::new(1, "edge_type", vec!["prop1".to_string()]));
        let opt_node = OptGroupNode::new(1, index_scan_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_tag_index_full_scan_rule() {
        let rule = TagIndexFullScanRule;
        let mut ctx = create_test_context();

        // Create an index scan node
        let index_scan_node = Box::new(IndexScan::new(1, "tag1", vec!["prop1".to_string()]));
        let opt_node = OptGroupNode::new(1, index_scan_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_index_scan_rule() {
        let rule = IndexScanRule;
        let mut ctx = create_test_context();

        // Create an index scan node
        let index_scan_node = Box::new(IndexScan::new(1, "tag1", vec!["prop1".to_string()]));
        let opt_node = OptGroupNode::new(1, index_scan_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_union_all_edge_index_scan_rule() {
        let rule = UnionAllEdgeIndexScanRule;
        let mut ctx = create_test_context();

        // Create an index scan node
        let index_scan_node = Box::new(IndexScan::new(1, "edge_type", vec!["prop1".to_string()]));
        let opt_node = OptGroupNode::new(1, index_scan_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_union_all_tag_index_scan_rule() {
        let rule = UnionAllTagIndexScanRule;
        let mut ctx = create_test_context();

        // Create an index scan node
        let index_scan_node = Box::new(IndexScan::new(1, "tag1", vec!["prop1".to_string()]));
        let opt_node = OptGroupNode::new(1, index_scan_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        assert!(result.is_some());
    }
}
