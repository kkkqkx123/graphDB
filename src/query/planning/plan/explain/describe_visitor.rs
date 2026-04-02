//! DescribeVisitor - Description of visitors to the planned node
//!
//! Use the Visitor pattern with zero-cost abstraction for distribution at compile time.
//! Collects node descriptions along with their dependencies for building complete plan graphs.

use crate::query::planning::plan::core::nodes::base::plan_node_traits::{
    MultipleInputNode, PlanNode, SingleInputNode,
};
use crate::query::planning::plan::core::nodes::base::plan_node_visitor::PlanNodeVisitor;
use crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum;
use crate::query::planning::plan::explain::description::PlanNodeDescription;

// Import all node types
use crate::query::planning::plan::core::nodes::access::graph_scan_node::{
    EdgeIndexScanNode, GetEdgesNode, GetNeighborsNode, GetVerticesNode, ScanEdgesNode,
    ScanVerticesNode,
};
use crate::query::planning::plan::core::nodes::access::index_scan::IndexScanNode;
use crate::query::planning::plan::core::nodes::control_flow::control_flow_node::{
    ArgumentNode, LoopNode, PassThroughNode, SelectNode,
};
use crate::query::planning::plan::core::nodes::control_flow::start_node::StartNode;
use crate::query::planning::plan::core::nodes::data_processing::aggregate_node::AggregateNode;
use crate::query::planning::plan::core::nodes::data_processing::data_processing_node::{
    AssignNode, DataCollectNode, DedupNode, PatternApplyNode, RollUpApplyNode, UnionNode,
    UnwindNode,
};
use crate::query::planning::plan::core::nodes::data_processing::set_operations_node::{
    IntersectNode, MinusNode,
};
use crate::query::planning::plan::core::nodes::join::join_node::{
    CrossJoinNode, FullOuterJoinNode, HashInnerJoinNode, HashLeftJoinNode, InnerJoinNode,
    LeftJoinNode,
};
use crate::query::planning::plan::core::nodes::management::edge_nodes::{
    AlterEdgeNode, CreateEdgeNode, DescEdgeNode, DropEdgeNode, ShowEdgesNode,
};
use crate::query::planning::plan::core::nodes::management::index_nodes::{
    CreateEdgeIndexNode, CreateTagIndexNode, DescEdgeIndexNode, DescTagIndexNode,
    DropEdgeIndexNode, DropTagIndexNode, RebuildEdgeIndexNode, RebuildTagIndexNode,
    ShowEdgeIndexesNode, ShowTagIndexesNode,
};
use crate::query::planning::plan::core::nodes::management::space_nodes::{
    CreateSpaceNode, DescSpaceNode, DropSpaceNode, ShowSpacesNode,
};
use crate::query::planning::plan::core::nodes::management::tag_nodes::{
    AlterTagNode, CreateTagNode, DescTagNode, DropTagNode, ShowTagsNode,
};
use crate::query::planning::plan::core::nodes::management::user_nodes::{
    AlterUserNode, ChangePasswordNode, CreateUserNode, DropUserNode,
};
use crate::query::planning::plan::core::nodes::operation::filter_node::FilterNode;
use crate::query::planning::plan::core::nodes::operation::project_node::ProjectNode;
use crate::query::planning::plan::core::nodes::operation::sample_node::SampleNode;
use crate::query::planning::plan::core::nodes::operation::sort_node::{
    LimitNode, SortNode, TopNNode,
};
use crate::query::planning::plan::core::nodes::traversal::path_algorithms::{
    AllPathsNode, BFSShortestNode, MultiShortestPathNode, ShortestPathNode,
};
use crate::query::planning::plan::core::nodes::traversal::traversal_node::{
    AppendVerticesNode, ExpandAllNode, ExpandNode, TraverseNode,
};

/// DescribeVisitor – Description of visitors to the planned node
pub struct DescribeVisitor {
    descriptions: Vec<PlanNodeDescription>,
    visited_ids: std::collections::HashSet<i64>,
}

impl DescribeVisitor {
    pub fn new() -> Self {
        Self {
            descriptions: Vec::new(),
            visited_ids: std::collections::HashSet::new(),
        }
    }

    pub fn into_descriptions(self) -> Vec<PlanNodeDescription> {
        self.descriptions
    }

    fn create_description<T: PlanNode>(&mut self, name: &'static str, node: &T) {
        let mut desc = PlanNodeDescription::new(name, node.id());
        if let Some(var) = node.output_var() {
            desc = desc.with_output_var(var.to_string());
        }
        self.descriptions.push(desc);
        self.visited_ids.insert(node.id());
    }

    fn create_description_with_deps<T: PlanNode>(
        &mut self,
        name: &'static str,
        node: &T,
        deps: Vec<i64>,
    ) {
        let mut desc = PlanNodeDescription::new(name, node.id());
        if let Some(var) = node.output_var() {
            desc = desc.with_output_var(var.to_string());
        }
        if !deps.is_empty() {
            desc.set_dependencies(deps);
        }
        self.descriptions.push(desc);
        self.visited_ids.insert(node.id());
    }

    fn collect_single_input_deps(&self, input: &PlanNodeEnum) -> Vec<i64> {
        vec![input.id()]
    }

    fn collect_binary_input_deps(&self, left: &PlanNodeEnum, right: &PlanNodeEnum) -> Vec<i64> {
        vec![left.id(), right.id()]
    }

    fn collect_multiple_input_deps(&self, inputs: &[PlanNodeEnum]) -> Vec<i64> {
        inputs.iter().map(|input| input.id()).collect()
    }
}

impl Default for DescribeVisitor {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================
// Macros for generating visitor methods
// ============================================

/// Generate simple visit methods that only create a description with name
macro_rules! impl_simple_visit {
    ($($method:ident => $name:expr, $type:ty),* $(,)?) => {
        $(
            fn $method(&mut self, node: &$type) {
                self.create_description($name, node);
            }
        )*
    };
}

/// Generate visit methods for single-input nodes
macro_rules! impl_single_input_visit {
    ($($method:ident => $name:expr, $type:ty),* $(,)?) => {
        $(
            fn $method(&mut self, node: &$type) {
                let deps = self.collect_single_input_deps(node.input());
                self.create_description_with_deps($name, node, deps);
            }
        )*
    };
}

/// Generate visit methods for binary-input nodes
macro_rules! impl_binary_input_visit {
    ($($method:ident => $name:expr, $type:ty),* $(,)?) => {
        $(
            fn $method(&mut self, node: &$type) {
                let deps = self.collect_binary_input_deps(node.left_input(), node.right_input());
                self.create_description_with_deps($name, node, deps);
            }
        )*
    };
}

/// Generate visit methods for multiple-input nodes (using inputs() method)
macro_rules! impl_multi_input_visit {
    ($($method:ident => $name:expr, $type:ty),* $(,)?) => {
        $(
            fn $method(&mut self, node: &$type) {
                let deps = self.collect_multiple_input_deps(node.inputs());
                self.create_description_with_deps($name, node, deps);
            }
        )*
    };
}

/// Generate visit methods for multiple-input nodes (using dependencies() method)
macro_rules! impl_deps_visit {
    ($($method:ident => $name:expr, $type:ty),* $(,)?) => {
        $(
            fn $method(&mut self, node: &$type) {
                let deps: Vec<i64> = node.dependencies().iter().map(|d| d.id()).collect();
                self.create_description_with_deps($name, node, deps);
            }
        )*
    };
}

impl PlanNodeVisitor for DescribeVisitor {
    type Result = ();

    fn visit_default(&mut self) {}

    // Simple nodes (no inputs)
    impl_simple_visit!(
        visit_start => "Start", StartNode,
        visit_argument => "Argument", ArgumentNode,
        visit_pass_through => "PassThrough", PassThroughNode,
        visit_create_space => "CreateSpace", CreateSpaceNode,
        visit_drop_space => "DropSpace", DropSpaceNode,
        visit_desc_space => "DescSpace", DescSpaceNode,
        visit_show_spaces => "ShowSpaces", ShowSpacesNode,
        visit_create_tag => "CreateTag", CreateTagNode,
        visit_alter_tag => "AlterTag", AlterTagNode,
        visit_desc_tag => "DescTag", DescTagNode,
        visit_drop_tag => "DropTag", DropTagNode,
        visit_show_tags => "ShowTags", ShowTagsNode,
        visit_create_edge => "CreateEdge", CreateEdgeNode,
        visit_alter_edge => "AlterEdge", AlterEdgeNode,
        visit_desc_edge => "DescEdge", DescEdgeNode,
        visit_drop_edge => "DropEdge", DropEdgeNode,
        visit_show_edges => "ShowEdges", ShowEdgesNode,
        visit_create_tag_index => "CreateTagIndex", CreateTagIndexNode,
        visit_drop_tag_index => "DropTagIndex", DropTagIndexNode,
        visit_desc_tag_index => "DescTagIndex", DescTagIndexNode,
        visit_show_tag_indexes => "ShowTagIndexes", ShowTagIndexesNode,
        visit_create_edge_index => "CreateEdgeIndex", CreateEdgeIndexNode,
        visit_drop_edge_index => "DropEdgeIndex", DropEdgeIndexNode,
        visit_desc_edge_index => "DescEdgeIndex", DescEdgeIndexNode,
        visit_show_edge_indexes => "ShowEdgeIndexes", ShowEdgeIndexesNode,
        visit_rebuild_tag_index => "RebuildTagIndex", RebuildTagIndexNode,
        visit_rebuild_edge_index => "RebuildEdgeIndex", RebuildEdgeIndexNode,
        visit_create_user => "CreateUser", CreateUserNode,
        visit_alter_user => "AlterUser", AlterUserNode,
        visit_drop_user => "DropUser", DropUserNode,
        visit_change_password => "ChangePassword", ChangePasswordNode,
    );

    // Single-input nodes
    impl_single_input_visit!(
        visit_filter => "Filter", FilterNode,
        visit_aggregate => "Aggregate", AggregateNode,
        visit_dedup => "Dedup", DedupNode,
        visit_data_collect => "DataCollect", DataCollectNode,
        visit_unwind => "Unwind", UnwindNode,
        visit_assign => "Assign", AssignNode,
    );

    // Binary-input nodes (joins)
    impl_binary_input_visit!(
        visit_cross_join => "CrossJoin", CrossJoinNode,
        visit_full_outer_join => "FullOuterJoin", FullOuterJoinNode,
        visit_pattern_apply => "PatternApply", PatternApplyNode,
        visit_roll_up_apply => "RollUpApply", RollUpApplyNode,
    );

    // Multi-input nodes (using inputs())
    impl_multi_input_visit!(
        visit_expand => "Expand", ExpandNode,
        visit_expand_all => "ExpandAll", ExpandAllNode,
    );

    // Multi-input nodes (using dependencies())
    impl_deps_visit!(
        visit_minus => "Minus", MinusNode,
        visit_intersect => "Intersect", IntersectNode,
    );

    // Custom implementations for nodes with special handling
    fn visit_project(&mut self, node: &ProjectNode) {
        let deps = self.collect_single_input_deps(node.input());
        let mut desc = PlanNodeDescription::new("Project", node.id());
        if let Some(var) = node.output_var() {
            desc = desc.with_output_var(var.to_string());
        }
        desc.set_dependencies(deps);

        let columns: Vec<String> = node.columns().iter().map(|col| col.alias.clone()).collect();
        if !columns.is_empty() {
            desc.add_description("columns", columns.join(", "));
        }

        self.descriptions.push(desc);
        self.visited_ids.insert(node.id());
    }

    fn visit_sort(&mut self, node: &SortNode) {
        let deps = self.collect_single_input_deps(node.input());
        let mut desc = PlanNodeDescription::new("Sort", node.id());
        if let Some(var) = node.output_var() {
            desc = desc.with_output_var(var.to_string());
        }
        desc.set_dependencies(deps);

        let sort_items = node.sort_items();
        let key_strs: Vec<String> = sort_items
            .iter()
            .map(|item| format!("{} {:?}", item.column, item.direction))
            .collect();
        if !key_strs.is_empty() {
            desc.add_description("sort_keys", key_strs.join(", "));
        }

        self.descriptions.push(desc);
        self.visited_ids.insert(node.id());
    }

    fn visit_limit(&mut self, node: &LimitNode) {
        let deps = self.collect_single_input_deps(node.input());
        let mut desc = PlanNodeDescription::new("Limit", node.id());
        if let Some(var) = node.output_var() {
            desc = desc.with_output_var(var.to_string());
        }
        desc.set_dependencies(deps);
        desc.add_description("count", node.count().to_string());
        desc.add_description("offset", node.offset().to_string());

        self.descriptions.push(desc);
        self.visited_ids.insert(node.id());
    }

    fn visit_topn(&mut self, node: &TopNNode) {
        let deps = self.collect_single_input_deps(node.input());
        let mut desc = PlanNodeDescription::new("TopN", node.id());
        if let Some(var) = node.output_var() {
            desc = desc.with_output_var(var.to_string());
        }
        desc.set_dependencies(deps);
        desc.add_description("limit", node.limit().to_string());

        self.descriptions.push(desc);
        self.visited_ids.insert(node.id());
    }

    fn visit_sample(&mut self, node: &SampleNode) {
        let deps = self.collect_single_input_deps(node.input());
        let mut desc = PlanNodeDescription::new("Sample", node.id());
        if let Some(var) = node.output_var() {
            desc = desc.with_output_var(var.to_string());
        }
        desc.set_dependencies(deps);
        desc.add_description("count", node.count().to_string());

        self.descriptions.push(desc);
        self.visited_ids.insert(node.id());
    }

    fn visit_inner_join(&mut self, node: &InnerJoinNode) {
        let deps = self.collect_binary_input_deps(node.left_input(), node.right_input());
        let mut desc = PlanNodeDescription::new("InnerJoin", node.id());
        if let Some(var) = node.output_var() {
            desc = desc.with_output_var(var.to_string());
        }
        desc.set_dependencies(deps);

        let hash_keys: Vec<String> = node
            .hash_keys()
            .iter()
            .map(|k| format!("{:?}", k))
            .collect();
        let probe_keys: Vec<String> = node
            .probe_keys()
            .iter()
            .map(|k| format!("{:?}", k))
            .collect();
        if !hash_keys.is_empty() {
            desc.add_description("hash_keys", hash_keys.join(", "));
        }
        if !probe_keys.is_empty() {
            desc.add_description("probe_keys", probe_keys.join(", "));
        }

        self.descriptions.push(desc);
        self.visited_ids.insert(node.id());
    }

    fn visit_left_join(&mut self, node: &LeftJoinNode) {
        let deps = self.collect_binary_input_deps(node.left_input(), node.right_input());
        let mut desc = PlanNodeDescription::new("LeftJoin", node.id());
        if let Some(var) = node.output_var() {
            desc = desc.with_output_var(var.to_string());
        }
        desc.set_dependencies(deps);

        let hash_keys: Vec<String> = node
            .hash_keys()
            .iter()
            .map(|k| format!("{:?}", k))
            .collect();
        let probe_keys: Vec<String> = node
            .probe_keys()
            .iter()
            .map(|k| format!("{:?}", k))
            .collect();
        if !hash_keys.is_empty() {
            desc.add_description("hash_keys", hash_keys.join(", "));
        }
        if !probe_keys.is_empty() {
            desc.add_description("probe_keys", probe_keys.join(", "));
        }

        self.descriptions.push(desc);
        self.visited_ids.insert(node.id());
    }

    fn visit_hash_inner_join(&mut self, node: &HashInnerJoinNode) {
        let deps = self.collect_binary_input_deps(node.left_input(), node.right_input());
        let mut desc = PlanNodeDescription::new("HashInnerJoin", node.id());
        if let Some(var) = node.output_var() {
            desc = desc.with_output_var(var.to_string());
        }
        desc.set_dependencies(deps);

        let hash_keys: Vec<String> = node
            .hash_keys()
            .iter()
            .map(|k| format!("{:?}", k))
            .collect();
        let probe_keys: Vec<String> = node
            .probe_keys()
            .iter()
            .map(|k| format!("{:?}", k))
            .collect();
        if !hash_keys.is_empty() {
            desc.add_description("hash_keys", hash_keys.join(", "));
        }
        if !probe_keys.is_empty() {
            desc.add_description("probe_keys", probe_keys.join(", "));
        }

        self.descriptions.push(desc);
        self.visited_ids.insert(node.id());
    }

    fn visit_hash_left_join(&mut self, node: &HashLeftJoinNode) {
        let deps = self.collect_binary_input_deps(node.left_input(), node.right_input());
        let mut desc = PlanNodeDescription::new("HashLeftJoin", node.id());
        if let Some(var) = node.output_var() {
            desc = desc.with_output_var(var.to_string());
        }
        desc.set_dependencies(deps);

        let hash_keys: Vec<String> = node
            .hash_keys()
            .iter()
            .map(|k| format!("{:?}", k))
            .collect();
        let probe_keys: Vec<String> = node
            .probe_keys()
            .iter()
            .map(|k| format!("{:?}", k))
            .collect();
        if !hash_keys.is_empty() {
            desc.add_description("hash_keys", hash_keys.join(", "));
        }
        if !probe_keys.is_empty() {
            desc.add_description("probe_keys", probe_keys.join(", "));
        }

        self.descriptions.push(desc);
        self.visited_ids.insert(node.id());
    }

    fn visit_get_vertices(&mut self, node: &GetVerticesNode) {
        let mut desc = PlanNodeDescription::new("GetVertices", node.id());
        if let Some(var) = node.output_var() {
            desc = desc.with_output_var(var.to_string());
        }

        desc.add_description("space", node.space_name().to_string());
        desc.add_description("src_vids", node.src_vids().to_string());
        if node.dedup() {
            desc.add_description("dedup", "true".to_string());
        }
        if let Some(limit) = node.limit() {
            desc.add_description("limit", limit.to_string());
        }

        self.descriptions.push(desc);
        self.visited_ids.insert(node.id());
    }

    fn visit_get_edges(&mut self, node: &GetEdgesNode) {
        let mut desc = PlanNodeDescription::new("GetEdges", node.id());
        if let Some(var) = node.output_var() {
            desc = desc.with_output_var(var.to_string());
        }

        desc.add_description("src", node.src().to_string());
        desc.add_description("edge_type", node.edge_type().to_string());
        desc.add_description("dst", node.dst().to_string());
        if let Some(limit) = node.limit() {
            desc.add_description("limit", limit.to_string());
        }

        self.descriptions.push(desc);
        self.visited_ids.insert(node.id());
    }

    fn visit_get_neighbors(&mut self, node: &GetNeighborsNode) {
        let mut desc = PlanNodeDescription::new("GetNeighbors", node.id());
        if let Some(var) = node.output_var() {
            desc = desc.with_output_var(var.to_string());
        }

        desc.add_description("src_vids", node.src_vids().to_string());
        let edge_types = node.edge_types();
        if !edge_types.is_empty() {
            desc.add_description("edge_types", edge_types.join(", "));
        }
        desc.add_description("direction", node.direction().to_string());

        self.descriptions.push(desc);
        self.visited_ids.insert(node.id());
    }

    fn visit_scan_vertices(&mut self, node: &ScanVerticesNode) {
        let mut desc = PlanNodeDescription::new("ScanVertices", node.id());
        if let Some(var) = node.output_var() {
            desc = desc.with_output_var(var.to_string());
        }

        desc.add_description("space", node.space_name().to_string());
        if let Some(tag) = node.tag() {
            desc.add_description("tag", tag.to_string());
        }
        if let Some(limit) = node.limit() {
            desc.add_description("limit", limit.to_string());
        }

        self.descriptions.push(desc);
        self.visited_ids.insert(node.id());
    }

    fn visit_scan_edges(&mut self, node: &ScanEdgesNode) {
        let mut desc = PlanNodeDescription::new("ScanEdges", node.id());
        if let Some(var) = node.output_var() {
            desc = desc.with_output_var(var.to_string());
        }

        if let Some(edge_type) = node.edge_type() {
            desc.add_description("edge_type", edge_type);
        }
        if let Some(limit) = node.limit() {
            desc.add_description("limit", limit.to_string());
        }

        self.descriptions.push(desc);
        self.visited_ids.insert(node.id());
    }

    fn visit_edge_index_scan(&mut self, node: &EdgeIndexScanNode) {
        let mut desc = PlanNodeDescription::new("EdgeIndexScan", node.id());
        if let Some(var) = node.output_var() {
            desc = desc.with_output_var(var.to_string());
        }

        desc.add_description("edge_type", node.edge_type().to_string());
        desc.add_description("index", node.index_name().to_string());
        desc.add_description("scan_type", format!("{:?}", node.scan_type()));
        if let Some(limit) = node.limit() {
            desc.add_description("limit", limit.to_string());
        }

        self.descriptions.push(desc);
        self.visited_ids.insert(node.id());
    }

    fn visit_traverse(&mut self, node: &TraverseNode) {
        let deps = self.collect_single_input_deps(node.input());
        let mut desc = PlanNodeDescription::new("Traverse", node.id());
        if let Some(var) = node.output_var() {
            desc = desc.with_output_var(var.to_string());
        }
        desc.set_dependencies(deps);

        desc.add_description("min_steps", node.min_steps().to_string());
        desc.add_description("max_steps", node.max_steps().to_string());
        let edge_types = node.edge_types();
        if !edge_types.is_empty() {
            desc.add_description("edge_types", edge_types.join(", "));
        }

        self.descriptions.push(desc);
        self.visited_ids.insert(node.id());
    }

    fn visit_append_vertices(&mut self, node: &AppendVerticesNode) {
        let mut desc = PlanNodeDescription::new("AppendVertices", node.id());
        if let Some(var) = node.output_var() {
            desc = desc.with_output_var(var.to_string());
        }

        self.descriptions.push(desc);
        self.visited_ids.insert(node.id());
    }

    fn visit_loop(&mut self, node: &LoopNode) {
        let mut desc = PlanNodeDescription::new("Loop", node.id());
        if let Some(var) = node.output_var() {
            desc = desc.with_output_var(var.to_string());
        }

        if let Some(ref body) = node.body() {
            desc.set_dependencies(vec![body.id()]);
        }

        self.descriptions.push(desc);
        self.visited_ids.insert(node.id());
    }

    fn visit_select(&mut self, node: &SelectNode) {
        let mut desc = PlanNodeDescription::new("Select", node.id());
        if let Some(var) = node.output_var() {
            desc = desc.with_output_var(var.to_string());
        }

        let mut deps = Vec::new();
        if let Some(ref if_branch) = node.if_branch() {
            deps.push(if_branch.id());
        }
        if let Some(ref else_branch) = node.else_branch() {
            deps.push(else_branch.id());
        }
        if !deps.is_empty() {
            desc.set_dependencies(deps);
        }

        self.descriptions.push(desc);
        self.visited_ids.insert(node.id());
    }

    fn visit_union(&mut self, node: &UnionNode) {
        let deps = self.collect_single_input_deps(node.input());
        let mut desc = PlanNodeDescription::new("Union", node.id());
        if let Some(var) = node.output_var() {
            desc = desc.with_output_var(var.to_string());
        }
        desc.set_dependencies(deps);
        desc.add_description("distinct", node.distinct().to_string());

        self.descriptions.push(desc);
        self.visited_ids.insert(node.id());
    }

    fn visit_index_scan(&mut self, node: &IndexScanNode) {
        let mut desc = PlanNodeDescription::new("IndexScan", node.id());
        if let Some(var) = node.output_var() {
            desc = desc.with_output_var(var.to_string());
        }

        desc.add_description("schema", node.schema_name().to_string());
        desc.add_description("index", node.index_name().to_string());
        desc.add_description("scan_type", format!("{:?}", node.scan_type()));
        if let Some(limit) = node.limit() {
            desc.add_description("limit", limit.to_string());
        }

        self.descriptions.push(desc);
        self.visited_ids.insert(node.id());
    }

    fn visit_multi_shortest_path(&mut self, node: &MultiShortestPathNode) {
        let mut desc = PlanNodeDescription::new("MultiShortestPath", node.id());
        if let Some(var) = node.output_var() {
            desc = desc.with_output_var(var.to_string());
        }

        desc.add_description("steps", node.steps().to_string());

        self.descriptions.push(desc);
        self.visited_ids.insert(node.id());
    }

    fn visit_bfs_shortest(&mut self, node: &BFSShortestNode) {
        let mut desc = PlanNodeDescription::new("BFSShortest", node.id());
        if let Some(var) = node.output_var() {
            desc = desc.with_output_var(var.to_string());
        }

        desc.add_description("steps", node.steps().to_string());
        let edge_types = node.edge_types();
        if !edge_types.is_empty() {
            desc.add_description("edge_types", edge_types.join(", "));
        }

        self.descriptions.push(desc);
        self.visited_ids.insert(node.id());
    }

    fn visit_all_paths(&mut self, node: &AllPathsNode) {
        let mut desc = PlanNodeDescription::new("AllPaths", node.id());
        if let Some(var) = node.output_var() {
            desc = desc.with_output_var(var.to_string());
        }

        desc.add_description("steps", node.steps().to_string());
        let edge_types = node.edge_types();
        if !edge_types.is_empty() {
            desc.add_description("edge_types", edge_types.join(", "));
        }

        self.descriptions.push(desc);
        self.visited_ids.insert(node.id());
    }

    fn visit_shortest_path(&mut self, node: &ShortestPathNode) {
        let mut desc = PlanNodeDescription::new("ShortestPath", node.id());
        if let Some(var) = node.output_var() {
            desc = desc.with_output_var(var.to_string());
        }

        desc.add_description("max_step", node.max_step().to_string());
        let edge_types = node.edge_types();
        if !edge_types.is_empty() {
            desc.add_description("edge_types", edge_types.join(", "));
        }

        self.descriptions.push(desc);
        self.visited_ids.insert(node.id());
    }
}
