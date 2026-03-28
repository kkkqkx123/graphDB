//! Implementation of the PlanNode operation
//!
//! Implementing various operation methods for PlanNodeEnum

use std::borrow::Cow;

use super::plan_node_enum::PlanNodeEnum;
use super::plan_node_traits::{MultipleInputNode, SingleInputNode};

/// Generate a macro for the `match` branch of `PlanNodeEnum` (including default values)
///
/// This macro generates matches for all node types, automatically calls the specified methods, and returns default values for management nodes.
macro_rules! match_all_nodes_with_default {
    ($self:expr, $method:ident, $default:expr) => {
        match $self {
            PlanNodeEnum::Start(node) => node.$method(),
            PlanNodeEnum::Project(node) => node.$method(),
            PlanNodeEnum::Sort(node) => node.$method(),
            PlanNodeEnum::Limit(node) => node.$method(),
            PlanNodeEnum::TopN(node) => node.$method(),
            PlanNodeEnum::Sample(node) => node.$method(),
            PlanNodeEnum::InnerJoin(node) => node.$method(),
            PlanNodeEnum::LeftJoin(node) => node.$method(),
            PlanNodeEnum::CrossJoin(node) => node.$method(),
            PlanNodeEnum::HashInnerJoin(node) => node.$method(),
            PlanNodeEnum::HashLeftJoin(node) => node.$method(),
            PlanNodeEnum::FullOuterJoin(node) => node.$method(),
            PlanNodeEnum::IndexScan(node) => node.$method(),
            PlanNodeEnum::EdgeIndexScan(node) => node.$method(),
            PlanNodeEnum::GetVertices(node) => node.$method(),
            PlanNodeEnum::GetEdges(node) => node.$method(),
            PlanNodeEnum::GetNeighbors(node) => node.$method(),
            PlanNodeEnum::ScanVertices(node) => node.$method(),
            PlanNodeEnum::ScanEdges(node) => node.$method(),
            PlanNodeEnum::Expand(node) => node.$method(),
            PlanNodeEnum::ExpandAll(node) => node.$method(),
            PlanNodeEnum::Traverse(node) => node.$method(),
            PlanNodeEnum::AppendVertices(node) => node.$method(),
            PlanNodeEnum::Filter(node) => node.$method(),
            PlanNodeEnum::Aggregate(node) => node.$method(),
            PlanNodeEnum::Argument(node) => node.$method(),
            PlanNodeEnum::Loop(node) => node.$method(),
            PlanNodeEnum::PassThrough(node) => node.$method(),
            PlanNodeEnum::Select(node) => node.$method(),
            PlanNodeEnum::DataCollect(node) => node.$method(),
            PlanNodeEnum::Dedup(node) => node.$method(),
            PlanNodeEnum::PatternApply(node) => node.$method(),
            PlanNodeEnum::RollUpApply(node) => node.$method(),
            PlanNodeEnum::Union(node) => node.$method(),
            PlanNodeEnum::Minus(node) => node.$method(),
            PlanNodeEnum::Intersect(node) => node.$method(),
            PlanNodeEnum::Unwind(node) => node.$method(),
            PlanNodeEnum::Assign(node) => node.$method(),
            PlanNodeEnum::MultiShortestPath(node) => node.$method(),
            PlanNodeEnum::BFSShortest(node) => node.$method(),
            PlanNodeEnum::AllPaths(node) => node.$method(),
            PlanNodeEnum::ShortestPath(node) => node.$method(),
            // The management node returns the default value.
            _ => $default,
        }
    };
}

impl PlanNodeEnum {
    /// Obtaining the unique ID of a node
    pub fn id(&self) -> i64 {
        match_all_nodes_with_default!(self, id, 0)
    }

    /// Obtain the name of the node type.
    pub fn name(&self) -> &'static str {
        match self {
            // Basic node types
            PlanNodeEnum::Start(_) => "Start",
            PlanNodeEnum::Project(_) => "Project",
            PlanNodeEnum::Sort(_) => "Sort",
            PlanNodeEnum::Limit(_) => "Limit",
            PlanNodeEnum::TopN(_) => "TopN",
            PlanNodeEnum::Sample(_) => "Sample",
            PlanNodeEnum::InnerJoin(_) => "InnerJoin",
            PlanNodeEnum::LeftJoin(_) => "LeftJoin",
            PlanNodeEnum::CrossJoin(_) => "CrossJoin",
            PlanNodeEnum::HashInnerJoin(_) => "HashInnerJoin",
            PlanNodeEnum::HashLeftJoin(_) => "HashLeftJoin",
            PlanNodeEnum::IndexScan(_) => "IndexScan",
            PlanNodeEnum::GetVertices(_) => "GetVertices",
            PlanNodeEnum::GetEdges(_) => "GetEdges",
            PlanNodeEnum::GetNeighbors(_) => "GetNeighbors",
            PlanNodeEnum::ScanVertices(_) => "ScanVertices",
            PlanNodeEnum::ScanEdges(_) => "ScanEdges",
            PlanNodeEnum::Expand(_) => "Expand",
            PlanNodeEnum::ExpandAll(_) => "ExpandAll",
            PlanNodeEnum::Traverse(_) => "Traverse",
            PlanNodeEnum::AppendVertices(_) => "AppendVertices",
            PlanNodeEnum::Filter(_) => "Filter",
            PlanNodeEnum::Aggregate(_) => "Aggregate",
            PlanNodeEnum::Argument(_) => "Argument",
            PlanNodeEnum::Loop(_) => "Loop",
            PlanNodeEnum::PassThrough(_) => "PassThrough",
            PlanNodeEnum::Select(_) => "Select",
            PlanNodeEnum::DataCollect(_) => "DataCollect",
            PlanNodeEnum::Dedup(_) => "Dedup",
            PlanNodeEnum::PatternApply(_) => "PatternApply",
            PlanNodeEnum::RollUpApply(_) => "RollUpApply",
            PlanNodeEnum::Union(_) => "Union",
            PlanNodeEnum::Unwind(_) => "Unwind",
            PlanNodeEnum::Assign(_) => "Assign",
            PlanNodeEnum::MultiShortestPath(_) => "MultiShortestPath",
            PlanNodeEnum::BFSShortest(_) => "BFSShortest",
            PlanNodeEnum::AllPaths(_) => "AllPaths",
            PlanNodeEnum::ShortestPath(_) => "ShortestPath",

            // Management node
            _ => "AdminNode",
        }
    }

    /// Obtain the output variables of the node.
    pub fn output_var(&self) -> Option<&str> {
        match self {
            // Basic node types – These nodes implement the PlanNode trait.
            PlanNodeEnum::Start(node) => node.output_var(),
            PlanNodeEnum::Project(node) => node.output_var(),
            PlanNodeEnum::Sort(node) => node.output_var(),
            PlanNodeEnum::Limit(node) => node.output_var(),
            PlanNodeEnum::TopN(node) => node.output_var(),
            PlanNodeEnum::Sample(node) => node.output_var(),
            PlanNodeEnum::InnerJoin(node) => node.output_var(),
            PlanNodeEnum::LeftJoin(node) => node.output_var(),
            PlanNodeEnum::CrossJoin(node) => node.output_var(),
            PlanNodeEnum::HashInnerJoin(node) => node.output_var(),
            PlanNodeEnum::HashLeftJoin(node) => node.output_var(),
            PlanNodeEnum::IndexScan(node) => node.output_var(),
            PlanNodeEnum::GetVertices(node) => node.output_var(),
            PlanNodeEnum::GetEdges(node) => node.output_var(),
            PlanNodeEnum::GetNeighbors(node) => node.output_var(),
            PlanNodeEnum::ScanVertices(node) => node.output_var(),
            PlanNodeEnum::ScanEdges(node) => node.output_var(),
            PlanNodeEnum::Expand(node) => node.output_var(),
            PlanNodeEnum::ExpandAll(node) => node.output_var(),
            PlanNodeEnum::Traverse(node) => node.output_var(),
            PlanNodeEnum::AppendVertices(node) => node.output_var(),
            PlanNodeEnum::Filter(node) => node.output_var(),
            PlanNodeEnum::Aggregate(node) => node.output_var(),
            PlanNodeEnum::Argument(node) => node.output_var(),
            PlanNodeEnum::Loop(node) => node.output_var(),
            PlanNodeEnum::PassThrough(node) => node.output_var(),
            PlanNodeEnum::Select(node) => node.output_var(),
            PlanNodeEnum::DataCollect(node) => node.output_var(),
            PlanNodeEnum::Dedup(node) => node.output_var(),
            PlanNodeEnum::PatternApply(node) => node.output_var(),
            PlanNodeEnum::RollUpApply(node) => node.output_var(),
            PlanNodeEnum::Union(node) => node.output_var(),
            PlanNodeEnum::Unwind(node) => node.output_var(),
            PlanNodeEnum::Assign(node) => node.output_var(),
            PlanNodeEnum::MultiShortestPath(node) => node.output_var(),
            PlanNodeEnum::BFSShortest(node) => node.output_var(),
            PlanNodeEnum::AllPaths(node) => node.output_var(),
            PlanNodeEnum::ShortestPath(node) => node.output_var(),

            // Management Node – No output variables
            _ => None,
        }
    }

    /// Obtain a list of column names.
    pub fn col_names(&self) -> &[String] {
        match_all_nodes_with_default!(self, col_names, &[])
    }

    /// Obtain the list of dependent nodes of a node.
    pub fn dependencies(&self) -> Vec<PlanNodeEnum> {
        self.dependencies_ref().iter().map(|&n| n.clone()).collect()
    }

    /// Get references to dependent nodes without cloning
    ///
    /// Uses Cow to avoid allocation for common cases (0, 1, or 2 dependencies)
    pub fn dependencies_ref(&self) -> Cow<'_, [&PlanNodeEnum]> {
        match self {
            // ========== Zero-input nodes ==========
            PlanNodeEnum::Start(_)
            | PlanNodeEnum::GetVertices(_)
            | PlanNodeEnum::GetEdges(_)
            | PlanNodeEnum::GetNeighbors(_)
            | PlanNodeEnum::ScanVertices(_)
            | PlanNodeEnum::ScanEdges(_)
            | PlanNodeEnum::IndexScan(_)
            | PlanNodeEnum::EdgeIndexScan(_)
            | PlanNodeEnum::MultiShortestPath(_)
            | PlanNodeEnum::BFSShortest(_)
            | PlanNodeEnum::AllPaths(_)
            | PlanNodeEnum::ShortestPath(_)
            | PlanNodeEnum::Argument(_)
            | PlanNodeEnum::PassThrough(_)
            | PlanNodeEnum::Select(_) => Cow::Borrowed(&[]),

            // ========== Single input nodes ==========
            PlanNodeEnum::Project(node) => Cow::Owned(vec![node.input()]),
            PlanNodeEnum::Sort(node) => Cow::Owned(vec![node.input()]),
            PlanNodeEnum::Limit(node) => Cow::Owned(vec![node.input()]),
            PlanNodeEnum::TopN(node) => Cow::Owned(vec![node.input()]),
            PlanNodeEnum::Sample(node) => Cow::Owned(vec![node.input()]),
            PlanNodeEnum::Filter(node) => Cow::Owned(vec![node.input()]),
            PlanNodeEnum::Aggregate(node) => Cow::Owned(vec![node.input()]),
            PlanNodeEnum::DataCollect(node) => Cow::Owned(vec![node.input()]),
            PlanNodeEnum::Dedup(node) => Cow::Owned(vec![node.input()]),
            PlanNodeEnum::PatternApply(node) => Cow::Owned(vec![node.input()]),
            PlanNodeEnum::RollUpApply(node) => Cow::Owned(vec![node.input()]),
            PlanNodeEnum::Union(node) => Cow::Owned(vec![node.input()]),
            PlanNodeEnum::Unwind(node) => Cow::Owned(vec![node.input()]),
            PlanNodeEnum::Assign(node) => Cow::Owned(vec![node.input()]),
            PlanNodeEnum::Traverse(node) => Cow::Owned(vec![node.input()]),

            // ========== Dual-input nodes (Joins) ==========
            PlanNodeEnum::InnerJoin(node) => {
                Cow::Owned(vec![node.left_input(), node.right_input()])
            }
            PlanNodeEnum::LeftJoin(node) => Cow::Owned(vec![node.left_input(), node.right_input()]),
            PlanNodeEnum::CrossJoin(node) => {
                Cow::Owned(vec![node.left_input(), node.right_input()])
            }
            PlanNodeEnum::HashInnerJoin(node) => {
                Cow::Owned(vec![node.left_input(), node.right_input()])
            }
            PlanNodeEnum::HashLeftJoin(node) => {
                Cow::Owned(vec![node.left_input(), node.right_input()])
            }
            PlanNodeEnum::FullOuterJoin(node) => {
                Cow::Owned(vec![node.left_input(), node.right_input()])
            }

            // ========== Multi-input nodes ==========
            // These nodes store inputs in a Vec, so we need to convert to Vec<&PlanNodeEnum>
            PlanNodeEnum::Expand(node) => Cow::Owned(node.inputs().iter().collect::<Vec<_>>()),
            PlanNodeEnum::ExpandAll(node) => Cow::Owned(node.inputs().iter().collect::<Vec<_>>()),
            PlanNodeEnum::AppendVertices(node) => {
                Cow::Owned(node.inputs().iter().collect::<Vec<_>>())
            }

            // ========== Other nodes ==========
            PlanNodeEnum::Loop(_) => Cow::Borrowed(&[]),

            // Management nodes: No input dependencies
            _ => Cow::Borrowed(&[]),
        }
    }

    /// Retrieve the first dependent node (if it exists).
    pub fn first_dependency(&self) -> Option<PlanNodeEnum> {
        let deps = self.dependencies();
        if deps.is_empty() {
            None
        } else {
            Some(deps[0].clone())
        }
    }

    /// Setting the output variables of the node
    pub fn set_output_var(&mut self, var: String) {
        match self {
            // Basic node types – These nodes implement the PlanNode trait.
            PlanNodeEnum::Start(node) => node.set_output_var(var),
            PlanNodeEnum::Project(node) => node.set_output_var(var),
            PlanNodeEnum::Sort(node) => node.set_output_var(var),
            PlanNodeEnum::Limit(node) => node.set_output_var(var),
            PlanNodeEnum::TopN(node) => node.set_output_var(var),
            PlanNodeEnum::Sample(node) => node.set_output_var(var),
            PlanNodeEnum::InnerJoin(node) => node.set_output_var(var),
            PlanNodeEnum::LeftJoin(node) => node.set_output_var(var),
            PlanNodeEnum::CrossJoin(node) => node.set_output_var(var),
            PlanNodeEnum::HashInnerJoin(node) => node.set_output_var(var),
            PlanNodeEnum::HashLeftJoin(node) => node.set_output_var(var),
            PlanNodeEnum::IndexScan(node) => node.set_output_var(var),
            PlanNodeEnum::GetVertices(node) => node.set_output_var(var),
            PlanNodeEnum::GetEdges(node) => node.set_output_var(var),
            PlanNodeEnum::GetNeighbors(node) => node.set_output_var(var),
            PlanNodeEnum::ScanVertices(node) => node.set_output_var(var),
            PlanNodeEnum::ScanEdges(node) => node.set_output_var(var),
            PlanNodeEnum::Expand(node) => node.set_output_var(var),
            PlanNodeEnum::ExpandAll(node) => node.set_output_var(var),
            PlanNodeEnum::Traverse(node) => node.set_output_var(var),
            PlanNodeEnum::AppendVertices(node) => node.set_output_var(var),
            PlanNodeEnum::Filter(node) => node.set_output_var(var),
            PlanNodeEnum::Aggregate(node) => node.set_output_var(var),
            PlanNodeEnum::Argument(node) => node.set_output_var(var),
            PlanNodeEnum::Loop(node) => node.set_output_var(var),
            PlanNodeEnum::PassThrough(node) => node.set_output_var(var),
            PlanNodeEnum::Select(node) => node.set_output_var(var),
            PlanNodeEnum::DataCollect(node) => node.set_output_var(var),
            PlanNodeEnum::Dedup(node) => node.set_output_var(var),
            PlanNodeEnum::PatternApply(node) => node.set_output_var(var),
            PlanNodeEnum::RollUpApply(node) => node.set_output_var(var),
            PlanNodeEnum::Union(node) => node.set_output_var(var),
            PlanNodeEnum::Unwind(node) => node.set_output_var(var),
            PlanNodeEnum::Assign(node) => node.set_output_var(var),
            PlanNodeEnum::MultiShortestPath(node) => node.set_output_var(var),
            PlanNodeEnum::BFSShortest(node) => node.set_output_var(var),
            PlanNodeEnum::AllPaths(node) => node.set_output_var(var),
            PlanNodeEnum::ShortestPath(node) => node.set_output_var(var),

            // Management node: There is no need to set any output variables.
            _ => {}
        }
    }

    /// Set column names
    pub fn set_col_names(&mut self, names: Vec<String>) {
        match self {
            // Basic node types – These nodes implement the PlanNode trait.
            PlanNodeEnum::Start(node) => node.set_col_names(names),
            PlanNodeEnum::Project(node) => node.set_col_names(names),
            PlanNodeEnum::Sort(node) => node.set_col_names(names),
            PlanNodeEnum::Limit(node) => node.set_col_names(names),
            PlanNodeEnum::TopN(node) => node.set_col_names(names),
            PlanNodeEnum::Sample(node) => node.set_col_names(names),
            PlanNodeEnum::InnerJoin(node) => node.set_col_names(names),
            PlanNodeEnum::LeftJoin(node) => node.set_col_names(names),
            PlanNodeEnum::CrossJoin(node) => node.set_col_names(names),
            PlanNodeEnum::HashInnerJoin(node) => node.set_col_names(names),
            PlanNodeEnum::HashLeftJoin(node) => node.set_col_names(names),
            PlanNodeEnum::IndexScan(node) => node.set_col_names(names),
            PlanNodeEnum::GetVertices(node) => node.set_col_names(names),
            PlanNodeEnum::GetEdges(node) => node.set_col_names(names),
            PlanNodeEnum::GetNeighbors(node) => node.set_col_names(names),
            PlanNodeEnum::ScanVertices(node) => node.set_col_names(names),
            PlanNodeEnum::ScanEdges(node) => node.set_col_names(names),
            PlanNodeEnum::Expand(node) => node.set_col_names(names),
            PlanNodeEnum::ExpandAll(node) => node.set_col_names(names),
            PlanNodeEnum::Traverse(node) => node.set_col_names(names),
            PlanNodeEnum::AppendVertices(node) => node.set_col_names(names),
            PlanNodeEnum::Filter(node) => node.set_col_names(names),
            PlanNodeEnum::Aggregate(node) => node.set_col_names(names),
            PlanNodeEnum::Argument(node) => node.set_col_names(names),
            PlanNodeEnum::Loop(node) => node.set_col_names(names),
            PlanNodeEnum::PassThrough(node) => node.set_col_names(names),
            PlanNodeEnum::Select(node) => node.set_col_names(names),
            PlanNodeEnum::DataCollect(node) => node.set_col_names(names),
            PlanNodeEnum::Dedup(node) => node.set_col_names(names),
            PlanNodeEnum::PatternApply(node) => node.set_col_names(names),
            PlanNodeEnum::RollUpApply(node) => node.set_col_names(names),
            PlanNodeEnum::Union(node) => node.set_col_names(names),
            PlanNodeEnum::Unwind(node) => node.set_col_names(names),
            PlanNodeEnum::Assign(node) => node.set_col_names(names),
            PlanNodeEnum::MultiShortestPath(node) => node.set_col_names(names),
            PlanNodeEnum::BFSShortest(node) => node.set_col_names(names),
            PlanNodeEnum::AllPaths(node) => node.set_col_names(names),
            PlanNodeEnum::ShortestPath(node) => node.set_col_names(names),

            // Management Node: There is no need to set column names.
            _ => {}
        }
    }
}
