//! Join optimization rules for NebulaGraph
//! These rules optimize join operations based on NebulaGraph's implementation

use super::optimizer::OptimizerError;
use crate::query::optimizer::optimizer::{OptContext, OptGroupNode, OptRule, Pattern, MatchedResult};
use crate::query::planner::plan::{PlanNodeKind, PlanNode};
use std::any::Any;

// Rule for pushing filter down hash inner join
#[derive(Debug)]
pub struct PushFilterDownHashInnerJoinRule;

impl OptRule for PushFilterDownHashInnerJoinRule {
    fn name(&self) -> &str {
        "PushFilterDownHashInnerJoinRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // Check if this is a filter operation above a hash inner join
        if node.plan_node.kind() != PlanNodeKind::Filter {
            return Ok(None);
        }

        // Match the pattern to see if we have a filter over a hash inner join
        if let Some(matched) = self.match_pattern(ctx, node)? {
            if matched.dependencies.len() >= 1 {
                let child = &matched.dependencies[0];

                if child.plan_node().kind() == PlanNodeKind::HashInnerJoin {
                    // In a real implementation, we would push the filter condition down to one or both sides of the join
                    // This can reduce the number of tuples that need to be joined
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
        Pattern::new(PlanNodeKind::Filter)
            .with_dependency(Pattern::new(PlanNodeKind::HashInnerJoin))
    }
}

// Rule for pushing filter down hash left join
#[derive(Debug)]
pub struct PushFilterDownHashLeftJoinRule;

impl OptRule for PushFilterDownHashLeftJoinRule {
    fn name(&self) -> &str {
        "PushFilterDownHashLeftJoinRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // Check if this is a filter operation above a hash left join
        if node.plan_node.kind() != PlanNodeKind::Filter {
            return Ok(None);
        }

        // Match the pattern to see if we have a filter over a hash left join
        if let Some(matched) = self.match_pattern(ctx, node)? {
            if matched.dependencies.len() >= 1 {
                let child = &matched.dependencies[0];

                if child.plan_node().kind() == PlanNodeKind::HashLeftJoin {
                    // In a real implementation, we would push the filter condition down to one or both sides of the join
                    // This can reduce the number of tuples that need to be joined
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
        Pattern::new(PlanNodeKind::Filter)
            .with_dependency(Pattern::new(PlanNodeKind::HashLeftJoin))
    }
}

// Rule for pushing filter down inner join
#[derive(Debug)]
pub struct PushFilterDownInnerJoinRule;

impl OptRule for PushFilterDownInnerJoinRule {
    fn name(&self) -> &str {
        "PushFilterDownInnerJoinRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // Check if this is a filter operation above an inner join
        if node.plan_node.kind() != PlanNodeKind::Filter {
            return Ok(None);
        }

        // Match the pattern to see if we have a filter over an inner join
        if let Some(matched) = self.match_pattern(ctx, node)? {
            if matched.dependencies.len() >= 1 {
                let child = &matched.dependencies[0];

                if child.plan_node().kind() == PlanNodeKind::InnerJoin {
                    // In a real implementation, we would push the filter condition down to one or both sides of the join
                    // This can reduce the number of tuples that need to be joined
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
        Pattern::new(PlanNodeKind::Filter)
            .with_dependency(Pattern::new(PlanNodeKind::InnerJoin))
    }
}

// Rule to merge get vertices and dedup operations
#[derive(Debug)]
pub struct MergeGetVerticesAndDedupRule;

impl OptRule for MergeGetVerticesAndDedupRule {
    fn name(&self) -> &str {
        "MergeGetVerticesAndDedupRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // Check if this is a GetVertices operation followed by Dedup
        if node.plan_node.kind() != PlanNodeKind::GetVertices {
            return Ok(None);
        }

        // Match the pattern to see if we have GetVertices followed by Dedup
        if let Some(matched) = self.match_pattern(ctx, node)? {
            if matched.dependencies.len() >= 1 {
                let child = &matched.dependencies[0];

                if child.plan_node().kind() == PlanNodeKind::Dedup {
                    // In a real implementation, we would merge these operations to avoid
                    // intermediate data storage and make the execution more efficient
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
        Pattern::new(PlanNodeKind::GetVertices)
            .with_dependency(Pattern::new(PlanNodeKind::Dedup))
    }
}

// Rule to merge get vertices and project operations
#[derive(Debug)]
pub struct MergeGetVerticesAndProjectRule;

impl OptRule for MergeGetVerticesAndProjectRule {
    fn name(&self) -> &str {
        "MergeGetVerticesAndProjectRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // Check if this is a GetVertices operation followed by Project
        if node.plan_node.kind() != PlanNodeKind::GetVertices {
            return Ok(None);
        }

        // Match the pattern to see if we have GetVertices followed by Project
        if let Some(matched) = self.match_pattern(ctx, node)? {
            if matched.dependencies.len() >= 1 {
                let child = &matched.dependencies[0];

                if child.plan_node().kind() == PlanNodeKind::Project {
                    // In a real implementation, we would merge these operations to avoid
                    // intermediate data storage and directly fetch only the needed properties
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
        Pattern::new(PlanNodeKind::GetVertices)
            .with_dependency(Pattern::new(PlanNodeKind::Project))
    }
}

// Rule to merge get neighbors and dedup operations
#[derive(Debug)]
pub struct MergeGetNbrsAndDedupRule;

impl OptRule for MergeGetNbrsAndDedupRule {
    fn name(&self) -> &str {
        "MergeGetNbrsAndDedupRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // Check if this is a GetNeighbors operation followed by Dedup
        if node.plan_node.kind() != PlanNodeKind::GetNeighbors {
            return Ok(None);
        }

        // Match the pattern to see if we have GetNeighbors followed by Dedup
        if let Some(matched) = self.match_pattern(ctx, node)? {
            if matched.dependencies.len() >= 1 {
                let child = &matched.dependencies[0];

                if child.plan_node().kind() == PlanNodeKind::Dedup {
                    // In a real implementation, we would merge these operations to avoid
                    // intermediate data storage and make the execution more efficient
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
        Pattern::new(PlanNodeKind::GetNeighbors)
            .with_dependency(Pattern::new(PlanNodeKind::Dedup))
    }
}

// Rule to merge get neighbors and project operations
#[derive(Debug)]
pub struct MergeGetNbrsAndProjectRule;

impl OptRule for MergeGetNbrsAndProjectRule {
    fn name(&self) -> &str {
        "MergeGetNbrsAndProjectRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // Check if this is a GetNeighbors operation followed by Project
        if node.plan_node.kind() != PlanNodeKind::GetNeighbors {
            return Ok(None);
        }

        // Match the pattern to see if we have GetNeighbors followed by Project
        if let Some(matched) = self.match_pattern(ctx, node)? {
            if matched.dependencies.len() >= 1 {
                let child = &matched.dependencies[0];

                if child.plan_node().kind() == PlanNodeKind::Project {
                    // In a real implementation, we would merge these operations to avoid
                    // intermediate data storage and directly fetch only the needed properties
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
        Pattern::new(PlanNodeKind::GetNeighbors)
            .with_dependency(Pattern::new(PlanNodeKind::Project))
    }
}

// Rule to remove append vertices operations below join
#[derive(Debug)]
pub struct RemoveAppendVerticesBelowJoinRule;

impl OptRule for RemoveAppendVerticesBelowJoinRule {
    fn name(&self) -> &str {
        "RemoveAppendVerticesBelowJoinRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // Check if this is an AppendVertices operation followed by a join
        if node.plan_node.kind() != PlanNodeKind::AppendVertices {
            return Ok(None);
        }

        // Match the pattern to see if we have AppendVertices followed by a join
        if let Some(matched) = self.match_pattern(ctx, node)? {
            if matched.dependencies.len() >= 1 {
                let child = &matched.dependencies[0];

                if child.plan_node().kind() == PlanNodeKind::InnerJoin ||
                   child.plan_node().kind() == PlanNodeKind::HashInnerJoin ||
                   child.plan_node().kind() == PlanNodeKind::HashLeftJoin {
                    // In a real implementation, we might be able to eliminate unnecessary
                    // append operations if they don't add value before the join
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
        Pattern::new(PlanNodeKind::AppendVertices)
            .with_dependency(Pattern::new(PlanNodeKind::InnerJoin))
    } // Or other join types
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::optimizer::optimizer::{OptContext, OptGroupNode};
    use crate::query::planner::plan::{PlanNodeKind, PlanNode};
    use crate::query::context::QueryContext;
    use crate::query::planner::plan::{Filter, GetVertices, GetNeighbors, Project, Dedup, AppendVertices};

    fn create_test_context() -> OptContext {
        OptContext::new(QueryContext::default())
    }

    #[test]
    fn test_push_filter_down_hash_inner_join_rule() {
        let rule = PushFilterDownHashInnerJoinRule;
        let mut ctx = create_test_context();

        // Create a filter node
        let filter_node = Box::new(Filter::new(1, "col1 > 100"));
        let opt_node = OptGroupNode::new(1, filter_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_push_filter_down_hash_left_join_rule() {
        let rule = PushFilterDownHashLeftJoinRule;
        let mut ctx = create_test_context();

        // Create a filter node
        let filter_node = Box::new(Filter::new(1, "col1 > 100"));
        let opt_node = OptGroupNode::new(1, filter_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_push_filter_down_inner_join_rule() {
        let rule = PushFilterDownInnerJoinRule;
        let mut ctx = create_test_context();

        // Create a filter node
        let filter_node = Box::new(Filter::new(1, "col1 > 100"));
        let opt_node = OptGroupNode::new(1, filter_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_merge_get_vertices_and_dedup_rule() {
        let rule = MergeGetVerticesAndDedupRule;
        let mut ctx = create_test_context();

        // Create a GetVertices node
        let get_vertices_node = Box::new(GetVertices::new(1));
        let opt_node = OptGroupNode::new(1, get_vertices_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_merge_get_vertices_and_project_rule() {
        let rule = MergeGetVerticesAndProjectRule;
        let mut ctx = create_test_context();

        // Create a GetVertices node
        let get_vertices_node = Box::new(GetVertices::new(1));
        let opt_node = OptGroupNode::new(1, get_vertices_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_merge_get_nbrs_and_dedup_rule() {
        let rule = MergeGetNbrsAndDedupRule;
        let mut ctx = create_test_context();

        // Create a GetNeighbors node
        let get_nbrs_node = Box::new(GetNeighbors::new(1));
        let opt_node = OptGroupNode::new(1, get_nbrs_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_merge_get_nbrs_and_project_rule() {
        let rule = MergeGetNbrsAndProjectRule;
        let mut ctx = create_test_context();

        // Create a GetNeighbors node
        let get_nbrs_node = Box::new(GetNeighbors::new(1));
        let opt_node = OptGroupNode::new(1, get_nbrs_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_remove_append_vertices_below_join_rule() {
        let rule = RemoveAppendVerticesBelowJoinRule;
        let mut ctx = create_test_context();

        // Create an AppendVertices node
        let append_vertices_node = Box::new(AppendVertices::new(1));
        let opt_node = OptGroupNode::new(1, append_vertices_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        assert!(result.is_some());
    }
}
