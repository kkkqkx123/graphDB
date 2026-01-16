use serde::{Deserialize, Serialize};

/// 节点描述键值对
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

/// 分支信息
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

/// 性能统计
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

/// 计划节点描述
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

/// 计划描述
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

use crate::query::planner::plan::algorithms::{
    AllPaths, BFSShortest, FulltextIndexScan, IndexScan, MultiShortestPath, ShortestPath,
};
use crate::query::planner::plan::core::nodes::plan_node_enum::*;
use crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode;

/// DescribeVisitor - 计划节点描述访问者
///
/// 使用零成本抽象的访问者模式，在编译时进行分发
pub struct DescribeVisitor {
    descriptions: Vec<PlanNodeDescription>,
}

impl DescribeVisitor {
    pub fn new() -> Self {
        Self {
            descriptions: Vec::new(),
        }
    }

    pub fn into_descriptions(self) -> Vec<PlanNodeDescription> {
        self.descriptions
    }

    fn create_description<T: PlanNode>(&mut self, name: &'static str, node: &T) {
        let mut desc = PlanNodeDescription::new(name, node.id());
        if let Some(var) = node.output_var() {
            desc = desc.with_output_var(var.name.clone());
        }
        desc.add_description("cost", format!("{:.2}", node.cost()));
        self.descriptions.push(desc);
    }
}

impl Default for DescribeVisitor {
    fn default() -> Self {
        Self::new()
    }
}

impl PlanNodeVisitor for DescribeVisitor {
    type Result = ();

    fn visit_start(&mut self, node: &StartNode) {
        self.create_description("Start", node);
    }

    fn visit_project(&mut self, node: &ProjectNode) {
        self.create_description("Project", node);
    }

    fn visit_sort(&mut self, node: &SortNode) {
        self.create_description("Sort", node);
    }

    fn visit_limit(&mut self, node: &LimitNode) {
        self.create_description("Limit", node);
    }

    fn visit_topn(&mut self, node: &TopNNode) {
        self.create_description("TopN", node);
    }

    fn visit_inner_join(&mut self, node: &InnerJoinNode) {
        self.create_description("InnerJoin", node);
    }

    fn visit_left_join(&mut self, node: &LeftJoinNode) {
        self.create_description("LeftJoin", node);
    }

    fn visit_cross_join(&mut self, node: &CrossJoinNode) {
        self.create_description("CrossJoin", node);
    }

    fn visit_get_vertices(&mut self, node: &GetVerticesNode) {
        self.create_description("GetVertices", node);
    }

    fn visit_get_edges(&mut self, node: &GetEdgesNode) {
        self.create_description("GetEdges", node);
    }

    fn visit_get_neighbors(&mut self, node: &GetNeighborsNode) {
        self.create_description("GetNeighbors", node);
    }

    fn visit_scan_vertices(&mut self, node: &ScanVerticesNode) {
        self.create_description("ScanVertices", node);
    }

    fn visit_scan_edges(&mut self, node: &ScanEdgesNode) {
        self.create_description("ScanEdges", node);
    }

    fn visit_expand(&mut self, node: &ExpandNode) {
        self.create_description("Expand", node);
    }

    fn visit_expand_all(&mut self, node: &ExpandAllNode) {
        self.create_description("ExpandAll", node);
    }

    fn visit_traverse(&mut self, node: &TraverseNode) {
        self.create_description("Traverse", node);
    }

    fn visit_append_vertices(&mut self, node: &AppendVerticesNode) {
        self.create_description("AppendVertices", node);
    }

    fn visit_filter(&mut self, node: &FilterNode) {
        self.create_description("Filter", node);
    }

    fn visit_aggregate(&mut self, node: &AggregateNode) {
        self.create_description("Aggregate", node);
    }

    fn visit_argument(&mut self, node: &ArgumentNode) {
        self.create_description("Argument", node);
    }

    fn visit_loop(&mut self, node: &LoopNode) {
        self.create_description("Loop", node);
    }

    fn visit_pass_through(&mut self, node: &PassThroughNode) {
        self.create_description("PassThrough", node);
    }

    fn visit_select(&mut self, node: &SelectNode) {
        self.create_description("Select", node);
    }

    fn visit_data_collect(&mut self, node: &DataCollectNode) {
        self.create_description("DataCollect", node);
    }

    fn visit_dedup(&mut self, node: &DedupNode) {
        self.create_description("Dedup", node);
    }

    fn visit_pattern_apply(&mut self, node: &PatternApplyNode) {
        self.create_description("PatternApply", node);
    }

    fn visit_roll_up_apply(&mut self, node: &RollUpApplyNode) {
        self.create_description("RollUpApply", node);
    }

    fn visit_union(&mut self, node: &UnionNode) {
        self.create_description("Union", node);
    }

    fn visit_unwind(&mut self, node: &UnwindNode) {
        self.create_description("Unwind", node);
    }

    fn visit_assign(&mut self, node: &AssignNode) {
        self.create_description("Assign", node);
    }

    fn visit_index_scan(&mut self, node: &IndexScan) {
        self.create_description("IndexScan", node);
    }

    fn visit_fulltext_index_scan(&mut self, node: &FulltextIndexScan) {
        self.create_description("FulltextIndexScan", node);
    }

    fn visit_multi_shortest_path(&mut self, node: &MultiShortestPath) {
        self.create_description("MultiShortestPath", node);
    }

    fn visit_bfs_shortest(&mut self, node: &BFSShortest) {
        self.create_description("BFSShortest", node);
    }

    fn visit_all_paths(&mut self, node: &AllPaths) {
        self.create_description("AllPaths", node);
    }

    fn visit_shortest_path(&mut self, node: &ShortestPath) {
        self.create_description("ShortestPath", node);
    }
}
