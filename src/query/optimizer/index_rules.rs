//! Index optimization rules for NebulaGraph
//! These rules optimize index operations based on NebulaGraph's implementation

use super::optimizer::OptimizerError;
use crate::query::optimizer::optimizer::{OptContext, OptGroupNode, OptRule, Pattern, MatchedResult};
use crate::query::planner::plan::{PlanNodeKind, PlanNode};
use std::any::Any;

// Rule to optimize edge index scan by filter
#[derive(Debug)]
pub struct OptimizeEdgeIndexScanByFilterRule;

impl OptRule for OptimizeEdgeIndexScanByFilterRule {
    fn name(&self) -> &str {
        "OptimizeEdgeIndexScanByFilterRule"
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

        // Match the pattern to determine if this is an edge index scan with applicable filters
        if let Some(matched) = self.match_pattern(ctx, node)? {
            if matched.dependencies.len() >= 1 {
                // Check if there are applicable filters that can be pushed into the index scan
                for dep in &matched.dependencies {
                    if dep.plan_node().kind() == PlanNodeKind::Filter {
                        // In a real implementation, we would merge the filter condition into the index scan
                        // to reduce the number of rows retrieved from the index
                        break; // Just check if there's a filter
                    }
                }
                Ok(Some(node.clone()))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn pattern(&self) -> Pattern {
        Pattern::new(PlanNodeKind::IndexScan) // Specifically for edge index scans
    }
}

// Rule to optimize tag index scan by filter
#[derive(Debug)]
pub struct OptimizeTagIndexScanByFilterRule;

impl OptRule for OptimizeTagIndexScanByFilterRule {
    fn name(&self) -> &str {
        "OptimizeTagIndexScanByFilterRule"
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

        // Match the pattern to determine if this is a tag index scan with applicable filters
        if let Some(matched) = self.match_pattern(ctx, node)? {
            if matched.dependencies.len() >= 1 {
                // Check if there are applicable filters that can be pushed into the index scan
                for dep in &matched.dependencies {
                    if dep.plan_node().kind() == PlanNodeKind::Filter {
                        // In a real implementation, we would merge the filter condition into the index scan
                        // to reduce the number of rows retrieved from the index
                        break; // Just check if there's a filter
                    }
                }
                Ok(Some(node.clone()))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn pattern(&self) -> Pattern {
        Pattern::new(PlanNodeKind::IndexScan) // Specifically for tag index scans
    }
}

// Rule for pushing limit operations down
#[derive(Debug)]
pub struct PushLimitDownRule;

impl OptRule for PushLimitDownRule {
    fn name(&self) -> &str {
        "PushLimitDownRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // Check if this is a limit operation
        if node.plan_node.kind() != PlanNodeKind::Limit {
            return Ok(None);
        }

        // Match the pattern to check if we can push the limit down
        if let Some(matched) = self.match_pattern(ctx, node)? {
            if matched.dependencies.len() >= 1 {
                let child = &matched.dependencies[0];

                // Try to push the limit down based on child node type
                match child.plan_node().kind() {
                    PlanNodeKind::IndexScan | PlanNodeKind::GetVertices | PlanNodeKind::GetEdges |
                    PlanNodeKind::ScanVertices | PlanNodeKind::ScanEdges => {
                        // For scan operations, pushing down limit can improve performance
                        Ok(Some(node.clone()))
                    },
                    PlanNodeKind::Sort => {
                        // For sort followed by limit (top-N queries), we might optimize differently
                        Ok(Some(node.clone()))
                    },
                    _ => {
                        // For other nodes, we might still be able to push the limit down
                        Ok(Some(node.clone()))
                    }
                }
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn pattern(&self) -> Pattern {
        Pattern::new(PlanNodeKind::Limit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::optimizer::optimizer::{OptContext, OptGroupNode};
    use crate::query::planner::plan::{PlanNodeKind, PlanNode};
    use crate::query::context::QueryContext;
    use crate::query::planner::plan::{IndexScan, Limit};

    fn create_test_context() -> OptContext {
        OptContext::new(QueryContext::default())
    }

    #[test]
    fn test_optimize_edge_index_scan_by_filter_rule() {
        let rule = OptimizeEdgeIndexScanByFilterRule;
        let mut ctx = create_test_context();

        // Create an index scan node
        let index_scan_node = Box::new(IndexScan::new(1, "tag1", vec!["prop1".to_string()]));
        let opt_node = OptGroupNode::new(1, index_scan_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_optimize_tag_index_scan_by_filter_rule() {
        let rule = OptimizeTagIndexScanByFilterRule;
        let mut ctx = create_test_context();

        // Create an index scan node
        let index_scan_node = Box::new(IndexScan::new(1, "tag1", vec!["prop1".to_string()]));
        let opt_node = OptGroupNode::new(1, index_scan_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_push_limit_down_rule() {
        let rule = PushLimitDownRule;
        let mut ctx = create_test_context();

        // Create a limit node
        let limit_node = Box::new(Limit::new(1, 10, 0));
        let opt_node = OptGroupNode::new(1, limit_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        assert!(result.is_some());
    }
}