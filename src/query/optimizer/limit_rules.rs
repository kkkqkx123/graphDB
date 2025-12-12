//! Limit and pagination optimization rules for NebulaGraph
//! These rules optimize limit and pagination operations based on NebulaGraph's implementation

use crate::query::optimizer::optimizer::{OptRule, Pattern, OptGroupNode, OptContext, MatchedResult};
use crate::query::planner::plan::{PlanNodeKind, PlanNode};
use super::optimizer::OptimizerError;
use std::any::Any;

// Rule to push limit down get vertices operations
#[derive(Debug)]
pub struct PushLimitDownGetVerticesRule;

impl OptRule for PushLimitDownGetVerticesRule {
    fn name(&self) -> &str {
        "PushLimitDownGetVerticesRule"
    }

    fn apply(&self, ctx: &mut OptContext, node: &OptGroupNode) -> Result<Option<OptGroupNode>, OptimizerError> {
        // Check if this is a limit operation
        if node.plan_node.kind() != PlanNodeKind::Limit {
            return Ok(None);
        }

        // Match the pattern to see if we have a limit over get vertices
        if let Some(matched) = self.match_pattern(ctx, node)? {
            if matched.dependencies.len() >= 1 {
                let child = &matched.dependencies[0];

                if child.plan_node().kind() == PlanNodeKind::GetVertices {
                    // In a real implementation, we would push the limit down to the get vertices operation
                    // to reduce the number of vertices fetched from storage
                    Ok(Some(node.clone()))
                } else {
                    Ok(None)
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
            .with_dependency(Pattern::new(PlanNodeKind::GetVertices))
    }
}

// Rule to push limit down get neighbors operations
#[derive(Debug)]
pub struct PushLimitDownGetNeighborsRule;

impl OptRule for PushLimitDownGetNeighborsRule {
    fn name(&self) -> &str {
        "PushLimitDownGetNeighborsRule"
    }

    fn apply(&self, ctx: &mut OptContext, node: &OptGroupNode) -> Result<Option<OptGroupNode>, OptimizerError> {
        // Check if this is a limit operation
        if node.plan_node.kind() != PlanNodeKind::Limit {
            return Ok(None);
        }

        // Match the pattern to see if we have a limit over get neighbors
        if let Some(matched) = self.match_pattern(ctx, node)? {
            if matched.dependencies.len() >= 1 {
                let child = &matched.dependencies[0];

                if child.plan_node().kind() == PlanNodeKind::GetNeighbors {
                    // In a real implementation, we would push the limit down to the get neighbors operation
                    // to reduce the number of neighbors fetched from storage
                    Ok(Some(node.clone()))
                } else {
                    Ok(None)
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
            .with_dependency(Pattern::new(PlanNodeKind::GetNeighbors))
    }
}

// Rule to push limit down get edges operations
#[derive(Debug)]
pub struct PushLimitDownGetEdgesRule;

impl OptRule for PushLimitDownGetEdgesRule {
    fn name(&self) -> &str {
        "PushLimitDownGetEdgesRule"
    }

    fn apply(&self, ctx: &mut OptContext, node: &OptGroupNode) -> Result<Option<OptGroupNode>, OptimizerError> {
        // Check if this is a limit operation
        if node.plan_node.kind() != PlanNodeKind::Limit {
            return Ok(None);
        }

        // Match the pattern to see if we have a limit over get edges
        if let Some(matched) = self.match_pattern(ctx, node)? {
            if matched.dependencies.len() >= 1 {
                let child = &matched.dependencies[0];

                if child.plan_node().kind() == PlanNodeKind::GetEdges {
                    // In a real implementation, we would push the limit down to the get edges operation
                    // to reduce the number of edges fetched from storage
                    Ok(Some(node.clone()))
                } else {
                    Ok(None)
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
            .with_dependency(Pattern::new(PlanNodeKind::GetEdges))
    }
}

// Rule to push limit down scan vertices operations
#[derive(Debug)]
pub struct PushLimitDownScanVerticesRule;

impl OptRule for PushLimitDownScanVerticesRule {
    fn name(&self) -> &str {
        "PushLimitDownScanVerticesRule"
    }

    fn apply(&self, ctx: &mut OptContext, node: &OptGroupNode) -> Result<Option<OptGroupNode>, OptimizerError> {
        // Check if this is a limit operation
        if node.plan_node.kind() != PlanNodeKind::Limit {
            return Ok(None);
        }

        // Match the pattern to see if we have a limit over scan vertices
        if let Some(matched) = self.match_pattern(ctx, node)? {
            if matched.dependencies.len() >= 1 {
                let child = &matched.dependencies[0];

                if child.plan_node().kind() == PlanNodeKind::ScanVertices {
                    // In a real implementation, we would push the limit down to the scan vertices operation
                    // to reduce the number of vertices scanned from storage
                    Ok(Some(node.clone()))
                } else {
                    Ok(None)
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
            .with_dependency(Pattern::new(PlanNodeKind::ScanVertices))
    }
}

// Rule to push limit down scan edges operations
#[derive(Debug)]
pub struct PushLimitDownScanEdgesRule;

impl OptRule for PushLimitDownScanEdgesRule {
    fn name(&self) -> &str {
        "PushLimitDownScanEdgesRule"
    }

    fn apply(&self, ctx: &mut OptContext, node: &OptGroupNode) -> Result<Option<OptGroupNode>, OptimizerError> {
        // Check if this is a limit operation
        if node.plan_node.kind() != PlanNodeKind::Limit {
            return Ok(None);
        }

        // Match the pattern to see if we have a limit over scan edges
        if let Some(matched) = self.match_pattern(ctx, node)? {
            if matched.dependencies.len() >= 1 {
                let child = &matched.dependencies[0];

                if child.plan_node().kind() == PlanNodeKind::ScanEdges {
                    // In a real implementation, we would push the limit down to the scan edges operation
                    // to reduce the number of edges scanned from storage
                    Ok(Some(node.clone()))
                } else {
                    Ok(None)
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
            .with_dependency(Pattern::new(PlanNodeKind::ScanEdges))
    }
}

// Rule to push limit down index scan operations
#[derive(Debug)]
pub struct PushLimitDownIndexScanRule;

impl OptRule for PushLimitDownIndexScanRule {
    fn name(&self) -> &str {
        "PushLimitDownIndexScanRule"
    }

    fn apply(&self, ctx: &mut OptContext, node: &OptGroupNode) -> Result<Option<OptGroupNode>, OptimizerError> {
        // Check if this is a limit operation
        if node.plan_node.kind() != PlanNodeKind::Limit {
            return Ok(None);
        }

        // Match the pattern to see if we have a limit over index scan
        if let Some(matched) = self.match_pattern(ctx, node)? {
            if matched.dependencies.len() >= 1 {
                let child = &matched.dependencies[0];

                if child.plan_node().kind() == PlanNodeKind::IndexScan {
                    // In a real implementation, we would push the limit down to the index scan operation
                    // to reduce the number of index entries scanned
                    Ok(Some(node.clone()))
                } else {
                    Ok(None)
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
            .with_dependency(Pattern::new(PlanNodeKind::IndexScan))
    }
}

// Rule to push limit down project operations
#[derive(Debug)]
pub struct PushLimitDownProjectRule;

impl OptRule for PushLimitDownProjectRule {
    fn name(&self) -> &str {
        "PushLimitDownProjectRule"
    }

    fn apply(&self, ctx: &mut OptContext, node: &OptGroupNode) -> Result<Option<OptGroupNode>, OptimizerError> {
        // Check if this is a limit operation
        if node.plan_node.kind() != PlanNodeKind::Limit {
            return Ok(None);
        }

        // Match the pattern to see if we have a limit over project
        if let Some(matched) = self.match_pattern(ctx, node)? {
            if matched.dependencies.len() >= 1 {
                let child = &matched.dependencies[0];

                if child.plan_node().kind() == PlanNodeKind::Project {
                    // In a real implementation, we would push the limit down to the project operation
                    // to limit the number of projected results
                    Ok(Some(node.clone()))
                } else {
                    Ok(None)
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
            .with_dependency(Pattern::new(PlanNodeKind::Project))
    }
}

// Rule to push limit down all paths operations
#[derive(Debug)]
pub struct PushLimitDownAllPathsRule;

impl OptRule for PushLimitDownAllPathsRule {
    fn name(&self) -> &str {
        "PushLimitDownAllPathsRule"
    }

    fn apply(&self, ctx: &mut OptContext, node: &OptGroupNode) -> Result<Option<OptGroupNode>, OptimizerError> {
        // Check if this is a limit operation
        if node.plan_node.kind() != PlanNodeKind::Limit {
            return Ok(None);
        }

        // Match the pattern to see if we have a limit over all paths
        if let Some(matched) = self.match_pattern(ctx, node)? {
            if matched.dependencies.len() >= 1 {
                let child = &matched.dependencies[0];

                if child.plan_node().kind() == PlanNodeKind::AllPaths {
                    // In a real implementation, we would push the limit down to the all paths operation
                    // to limit the number of paths computed
                    Ok(Some(node.clone()))
                } else {
                    Ok(None)
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
            .with_dependency(Pattern::new(PlanNodeKind::AllPaths))
    }
}

// Rule to push limit down expand all operations
#[derive(Debug)]
pub struct PushLimitDownExpandAllRule;

impl OptRule for PushLimitDownExpandAllRule {
    fn name(&self) -> &str {
        "PushLimitDownExpandAllRule"
    }

    fn apply(&self, ctx: &mut OptContext, node: &OptGroupNode) -> Result<Option<OptGroupNode>, OptimizerError> {
        // Check if this is a limit operation
        if node.plan_node.kind() != PlanNodeKind::Limit {
            return Ok(None);
        }

        // Match the pattern to see if we have a limit over expand all
        if let Some(matched) = self.match_pattern(ctx, node)? {
            if matched.dependencies.len() >= 1 {
                let child = &matched.dependencies[0];

                if child.plan_node().kind() == PlanNodeKind::ExpandAll {
                    // In a real implementation, we would push the limit down to the expand all operation
                    // to limit the number of expansions
                    Ok(Some(node.clone()))
                } else {
                    Ok(None)
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
            .with_dependency(Pattern::new(PlanNodeKind::ExpandAll))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::optimizer::optimizer::{OptContext, OptGroupNode};
    use crate::query::planner::plan::{PlanNodeKind, PlanNode};
    use crate::query::context::QueryContext;
    use crate::query::planner::plan::{Limit, GetVertices, GetNeighbors, GetEdges, Project, IndexScan, ScanVertices, ScanEdges};

    fn create_test_context() -> OptContext {
        OptContext::new(QueryContext::default())
    }

    #[test]
    fn test_push_limit_down_get_vertices_rule() {
        let rule = PushLimitDownGetVerticesRule;
        let mut ctx = create_test_context();

        // Create a limit node
        let limit_node = Box::new(Limit::new(1, 10, 0));
        let opt_node = OptGroupNode::new(1, limit_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_push_limit_down_get_neighbors_rule() {
        let rule = PushLimitDownGetNeighborsRule;
        let mut ctx = create_test_context();

        // Create a limit node
        let limit_node = Box::new(Limit::new(1, 10, 0));
        let opt_node = OptGroupNode::new(1, limit_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_push_limit_down_get_edges_rule() {
        let rule = PushLimitDownGetEdgesRule;
        let mut ctx = create_test_context();

        // Create a limit node
        let limit_node = Box::new(Limit::new(1, 10, 0));
        let opt_node = OptGroupNode::new(1, limit_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_push_limit_down_scan_vertices_rule() {
        let rule = PushLimitDownScanVerticesRule;
        let mut ctx = create_test_context();

        // Create a limit node
        let limit_node = Box::new(Limit::new(1, 10, 0));
        let opt_node = OptGroupNode::new(1, limit_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_push_limit_down_scan_edges_rule() {
        let rule = PushLimitDownScanEdgesRule;
        let mut ctx = create_test_context();

        // Create a limit node
        let limit_node = Box::new(Limit::new(1, 10, 0));
        let opt_node = OptGroupNode::new(1, limit_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_push_limit_down_index_scan_rule() {
        let rule = PushLimitDownIndexScanRule;
        let mut ctx = create_test_context();

        // Create a limit node
        let limit_node = Box::new(Limit::new(1, 10, 0));
        let opt_node = OptGroupNode::new(1, limit_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_push_limit_down_project_rule() {
        let rule = PushLimitDownProjectRule;
        let mut ctx = create_test_context();

        // Create a limit node
        let limit_node = Box::new(Limit::new(1, 10, 0));
        let opt_node = OptGroupNode::new(1, limit_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_push_limit_down_all_paths_rule() {
        let rule = PushLimitDownAllPathsRule;
        let mut ctx = create_test_context();

        // Create a limit node
        let limit_node = Box::new(Limit::new(1, 10, 0));
        let opt_node = OptGroupNode::new(1, limit_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_push_limit_down_expand_all_rule() {
        let rule = PushLimitDownExpandAllRule;
        let mut ctx = create_test_context();

        // Create a limit node
        let limit_node = Box::new(Limit::new(1, 10, 0));
        let opt_node = OptGroupNode::new(1, limit_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        assert!(result.is_some());
    }
}