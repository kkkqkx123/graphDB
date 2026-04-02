use serde::{Deserialize, Serialize};

/// Node description key-value pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pair {
    pub key: String,
    pub value: String,
}

impl Pair {
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

/// Branch information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanNodeBranchInfo {
    pub is_do_branch: bool,
    pub condition_node_id: i64,
}

impl PlanNodeBranchInfo {
    pub fn new(is_do_branch: bool, condition_node_id: i64) -> Self {
        Self {
            is_do_branch,
            condition_node_id,
        }
    }
}

/// Performance statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilingStats {
    pub rows: i64,
    pub exec_duration_in_us: i64,
    pub total_duration_in_us: i64,
    pub other_stats: std::collections::HashMap<String, String>,
}

impl ProfilingStats {
    pub fn new() -> Self {
        Self {
            rows: 0,
            exec_duration_in_us: 0,
            total_duration_in_us: 0,
            other_stats: std::collections::HashMap::new(),
        }
    }
}

impl Default for ProfilingStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Plan Node Description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanNodeDescription {
    pub name: String,
    pub id: i64,
    pub output_var: String,
    pub description: Option<Vec<Pair>>,
    pub profiles: Option<Vec<ProfilingStats>>,
    pub branch_info: Option<PlanNodeBranchInfo>,
    pub dependencies: Option<Vec<i64>>,
}

impl PlanNodeDescription {
    pub fn new(name: impl Into<String>, id: i64) -> Self {
        Self {
            name: name.into(),
            id,
            output_var: String::new(),
            description: None,
            profiles: None,
            branch_info: None,
            dependencies: None,
        }
    }

    pub fn with_output_var(mut self, output_var: impl Into<String>) -> Self {
        self.output_var = output_var.into();
        self
    }

    pub fn add_description(&mut self, key: impl Into<String>, value: impl Into<String>) {
        if self.description.is_none() {
            self.description = Some(Vec::new());
        }
        self.description
            .as_mut()
            .expect("description should be Some after initialization")
            .push(Pair::new(key, value));
    }

    pub fn with_description(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.add_description(key, value);
        self
    }

    pub fn set_dependencies(&mut self, deps: Vec<i64>) {
        self.dependencies = Some(deps);
    }

    pub fn with_dependencies(mut self, deps: Vec<i64>) -> Self {
        self.dependencies = Some(deps);
        self
    }

    pub fn set_branch_info(&mut self, branch_info: PlanNodeBranchInfo) {
        self.branch_info = Some(branch_info);
    }

    pub fn with_branch_info(mut self, branch_info: PlanNodeBranchInfo) -> Self {
        self.branch_info = Some(branch_info);
        self
    }

    pub fn add_profile(&mut self, profile: ProfilingStats) {
        if self.profiles.is_none() {
            self.profiles = Some(Vec::new());
        }
        self.profiles
            .as_mut()
            .expect("profiles should be Some after initialization")
            .push(profile);
    }
}

/// Plan Description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanDescription {
    pub plan_node_descs: Vec<PlanNodeDescription>,
    pub node_index_map: std::collections::HashMap<i64, usize>,
    pub format: String,
    pub optimize_time_in_us: i64,
}

impl PlanDescription {
    pub fn new() -> Self {
        Self {
            plan_node_descs: Vec::new(),
            node_index_map: std::collections::HashMap::new(),
            format: String::new(),
            optimize_time_in_us: 0,
        }
    }

    pub fn add_node_desc(&mut self, desc: PlanNodeDescription) -> usize {
        let index = self.plan_node_descs.len();
        let node_id = desc.id;
        self.plan_node_descs.push(desc);
        self.node_index_map.insert(node_id, index);
        index
    }

    pub fn get_node_desc(&self, node_id: i64) -> Option<&PlanNodeDescription> {
        self.node_index_map
            .get(&node_id)
            .and_then(|&index| self.plan_node_descs.get(index))
    }

    pub fn get_node_desc_mut(&mut self, node_id: i64) -> Option<&mut PlanNodeDescription> {
        if let Some(&index) = self.node_index_map.get(&node_id) {
            self.plan_node_descs.get_mut(index)
        } else {
            None
        }
    }
}

impl Default for PlanDescription {
    fn default() -> Self {
        Self::new()
    }
}

use crate::query::planning::plan::core::nodes::access::graph_scan_node::{
    EdgeIndexScanNode, GetEdgesNode, GetNeighborsNode, GetVerticesNode, ScanEdgesNode,
    ScanVerticesNode,
};
use crate::query::planning::plan::core::nodes::access::index_scan::IndexScanNode;
use crate::query::planning::plan::core::nodes::base::plan_node_enum::*;
use crate::query::planning::plan::core::nodes::base::plan_node_traits::{
    MultipleInputNode, PlanNode, SingleInputNode,
};
use crate::query::planning::plan::core::nodes::base::plan_node_visitor::PlanNodeVisitor;
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
use crate::query::planning::plan::core::nodes::operation::sort_node::{LimitNode, SortNode, TopNNode};
use crate::query::planning::plan::core::nodes::traversal::path_algorithms::{
    AllPathsNode, BFSShortestNode, MultiShortestPathNode, ShortestPathNode,
};
use crate::query::planning::plan::core::nodes::traversal::traversal_node::{
    AppendVerticesNode, ExpandAllNode, ExpandNode, TraverseNode,
};

/// DescribeVisitor – Description of visitors to the planned node
///
/// Use the Visitor pattern with zero-cost abstraction for distribution at compile time.
/// Collects node descriptions along with their dependencies for building complete plan graphs.
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

    fn get_dependency_ids(&self, node_enum: &PlanNodeEnum) -> Vec<i64> {
        vec![node_enum.id()]
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

impl PlanNodeVisitor for DescribeVisitor {
    type Result = ();

    fn visit_default(&mut self) {}

    fn visit_start(&mut self, node: &StartNode) {
        self.create_description("Start", node);
    }

    fn visit_project(&mut self, node: &ProjectNode) {
        let deps = self.collect_single_input_deps(node.input());
        let mut desc = PlanNodeDescription::new("Project", node.id());
        if let Some(var) = node.output_var() {
            desc = desc.with_output_var(var.to_string());
        }
        desc.set_dependencies(deps);

        // Add column information
        let columns: Vec<String> = node
            .columns()
            .iter()
            .map(|col| col.alias.clone())
            .collect();
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

        // Add sort key information
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

        // Add join key information
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

    fn visit_cross_join(&mut self, node: &CrossJoinNode) {
        let deps = self.collect_binary_input_deps(node.left_input(), node.right_input());
        self.create_description_with_deps("CrossJoin", node, deps);
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

    fn visit_full_outer_join(&mut self, node: &FullOuterJoinNode) {
        let deps = self.collect_binary_input_deps(node.left_input(), node.right_input());
        self.create_description_with_deps("FullOuterJoin", node, deps);
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

    fn visit_expand(&mut self, node: &ExpandNode) {
        let deps = self.collect_multiple_input_deps(node.inputs());
        let mut desc = PlanNodeDescription::new("Expand", node.id());
        if let Some(var) = node.output_var() {
            desc = desc.with_output_var(var.to_string());
        }
        desc.set_dependencies(deps);

        let edge_types = node.edge_types();
        if !edge_types.is_empty() {
            desc.add_description("edge_types", edge_types.join(", "));
        }
        desc.add_description("direction", format!("{:?}", node.direction()));

        self.descriptions.push(desc);
        self.visited_ids.insert(node.id());
    }

    fn visit_expand_all(&mut self, node: &ExpandAllNode) {
        let deps = self.collect_multiple_input_deps(node.inputs());
        let mut desc = PlanNodeDescription::new("ExpandAll", node.id());
        if let Some(var) = node.output_var() {
            desc = desc.with_output_var(var.to_string());
        }
        desc.set_dependencies(deps);

        let edge_types = node.edge_types();
        if !edge_types.is_empty() {
            desc.add_description("edge_types", edge_types.join(", "));
        }
        desc.add_description("direction", format!("{:?}", node.direction()));

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
        // AppendVerticesNode has no input, it's a leaf node
        let mut desc = PlanNodeDescription::new("AppendVertices", node.id());
        if let Some(var) = node.output_var() {
            desc = desc.with_output_var(var.to_string());
        }

        self.descriptions.push(desc);
        self.visited_ids.insert(node.id());
    }

    fn visit_filter(&mut self, node: &FilterNode) {
        let deps = self.collect_single_input_deps(node.input());
        let mut desc = PlanNodeDescription::new("Filter", node.id());
        if let Some(var) = node.output_var() {
            desc = desc.with_output_var(var.to_string());
        }
        desc.set_dependencies(deps);

        // Note: Filter condition details would require access to condition_serializable
        // which is not publicly accessible. Consider adding a getter method to FilterNode.

        self.descriptions.push(desc);
        self.visited_ids.insert(node.id());
    }

    fn visit_aggregate(&mut self, node: &AggregateNode) {
        let deps = self.collect_single_input_deps(node.input());
        let mut desc = PlanNodeDescription::new("Aggregate", node.id());
        if let Some(var) = node.output_var() {
            desc = desc.with_output_var(var.to_string());
        }
        desc.set_dependencies(deps);

        // Add group by and aggregate function info
        let group_keys = node.group_keys();
        if !group_keys.is_empty() {
            desc.add_description("group_by", group_keys.join(", "));
        }

        let agg_funcs = node.aggregation_functions();
        if !agg_funcs.is_empty() {
            let func_names: Vec<String> = agg_funcs.iter().map(|f| f.name().to_string()).collect();
            desc.add_description("aggregates", func_names.join(", "));
        }

        self.descriptions.push(desc);
        self.visited_ids.insert(node.id());
    }

    fn visit_argument(&mut self, node: &ArgumentNode) {
        self.create_description("Argument", node);
    }

    fn visit_loop(&mut self, node: &LoopNode) {
        let mut desc = PlanNodeDescription::new("Loop", node.id());
        if let Some(var) = node.output_var() {
            desc = desc.with_output_var(var.to_string());
        }

        // LoopNode has body instead of input
        if let Some(ref body) = node.body() {
            desc.set_dependencies(vec![body.id()]);
        }

        self.descriptions.push(desc);
        self.visited_ids.insert(node.id());
    }

    fn visit_pass_through(&mut self, node: &PassThroughNode) {
        // PassThroughNode is a ZeroInputNode, no dependencies
        self.create_description("PassThrough", node);
    }

    fn visit_select(&mut self, node: &SelectNode) {
        let mut desc = PlanNodeDescription::new("Select", node.id());
        if let Some(var) = node.output_var() {
            desc = desc.with_output_var(var.to_string());
        }

        // SelectNode has if_branch and else_branch
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

    fn visit_data_collect(&mut self, node: &DataCollectNode) {
        let deps = self.collect_single_input_deps(node.input());
        self.create_description_with_deps("DataCollect", node, deps);
    }

    fn visit_dedup(&mut self, node: &DedupNode) {
        let deps = self.collect_single_input_deps(node.input());
        self.create_description_with_deps("Dedup", node, deps);
    }

    fn visit_pattern_apply(&mut self, node: &PatternApplyNode) {
        let deps = self.collect_binary_input_deps(node.left_input(), node.right_input());
        self.create_description_with_deps("PatternApply", node, deps);
    }

    fn visit_roll_up_apply(&mut self, node: &RollUpApplyNode) {
        let deps = self.collect_binary_input_deps(node.left_input(), node.right_input());
        self.create_description_with_deps("RollUpApply", node, deps);
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

    fn visit_minus(&mut self, node: &MinusNode) {
        // MinusNode has main input and minus_input in deps
        let deps: Vec<i64> = node.dependencies().iter().map(|d| d.id()).collect();
        self.create_description_with_deps("Minus", node, deps);
    }

    fn visit_intersect(&mut self, node: &IntersectNode) {
        // IntersectNode has main input and intersect_input in deps
        let deps: Vec<i64> = node.dependencies().iter().map(|d| d.id()).collect();
        self.create_description_with_deps("Intersect", node, deps);
    }

    fn visit_unwind(&mut self, node: &UnwindNode) {
        let deps = self.collect_single_input_deps(node.input());
        self.create_description_with_deps("Unwind", node, deps);
    }

    fn visit_assign(&mut self, node: &AssignNode) {
        let deps = self.collect_single_input_deps(node.input());
        self.create_description_with_deps("Assign", node, deps);
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

    fn visit_create_space(&mut self, node: &CreateSpaceNode) {
        self.create_description("CreateSpace", node);
    }

    fn visit_drop_space(&mut self, node: &DropSpaceNode) {
        self.create_description("DropSpace", node);
    }

    fn visit_desc_space(&mut self, node: &DescSpaceNode) {
        self.create_description("DescSpace", node);
    }

    fn visit_show_spaces(&mut self, node: &ShowSpacesNode) {
        self.create_description("ShowSpaces", node);
    }

    fn visit_create_tag(&mut self, node: &CreateTagNode) {
        self.create_description("CreateTag", node);
    }

    fn visit_alter_tag(&mut self, node: &AlterTagNode) {
        self.create_description("AlterTag", node);
    }

    fn visit_desc_tag(&mut self, node: &DescTagNode) {
        self.create_description("DescTag", node);
    }

    fn visit_drop_tag(&mut self, node: &DropTagNode) {
        self.create_description("DropTag", node);
    }

    fn visit_show_tags(&mut self, node: &ShowTagsNode) {
        self.create_description("ShowTags", node);
    }

    fn visit_create_edge(&mut self, node: &CreateEdgeNode) {
        self.create_description("CreateEdge", node);
    }

    fn visit_alter_edge(&mut self, node: &AlterEdgeNode) {
        self.create_description("AlterEdge", node);
    }

    fn visit_desc_edge(&mut self, node: &DescEdgeNode) {
        self.create_description("DescEdge", node);
    }

    fn visit_drop_edge(&mut self, node: &DropEdgeNode) {
        self.create_description("DropEdge", node);
    }

    fn visit_show_edges(&mut self, node: &ShowEdgesNode) {
        self.create_description("ShowEdges", node);
    }

    fn visit_create_tag_index(&mut self, node: &CreateTagIndexNode) {
        self.create_description("CreateTagIndex", node);
    }

    fn visit_drop_tag_index(&mut self, node: &DropTagIndexNode) {
        self.create_description("DropTagIndex", node);
    }

    fn visit_desc_tag_index(&mut self, node: &DescTagIndexNode) {
        self.create_description("DescTagIndex", node);
    }

    fn visit_show_tag_indexes(&mut self, node: &ShowTagIndexesNode) {
        self.create_description("ShowTagIndexes", node);
    }

    fn visit_create_edge_index(&mut self, node: &CreateEdgeIndexNode) {
        self.create_description("CreateEdgeIndex", node);
    }

    fn visit_drop_edge_index(&mut self, node: &DropEdgeIndexNode) {
        self.create_description("DropEdgeIndex", node);
    }

    fn visit_desc_edge_index(&mut self, node: &DescEdgeIndexNode) {
        self.create_description("DescEdgeIndex", node);
    }

    fn visit_show_edge_indexes(&mut self, node: &ShowEdgeIndexesNode) {
        self.create_description("ShowEdgeIndexes", node);
    }

    fn visit_rebuild_tag_index(&mut self, node: &RebuildTagIndexNode) {
        self.create_description("RebuildTagIndex", node);
    }

    fn visit_rebuild_edge_index(&mut self, node: &RebuildEdgeIndexNode) {
        self.create_description("RebuildEdgeIndex", node);
    }

    fn visit_create_user(&mut self, node: &CreateUserNode) {
        self.create_description("CreateUser", node);
    }

    fn visit_alter_user(&mut self, node: &AlterUserNode) {
        self.create_description("AlterUser", node);
    }

    fn visit_drop_user(&mut self, node: &DropUserNode) {
        self.create_description("DropUser", node);
    }

    fn visit_change_password(&mut self, node: &ChangePasswordNode) {
        self.create_description("ChangePassword", node);
    }
}
