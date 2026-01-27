//! 执行计划分析访问者
//!
//! 使用访问者模式分析执行计划，生成执行器调度信息。

use crate::query::planner::plan::core::nodes::PlanNodeEnum;
use crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode;
use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;
use crate::query::visitor::PlanNodeVisitor;
use crate::query::context::validate::types::Variable;
use std::collections::HashMap;

/// 执行计划分析结果
#[derive(Debug, Clone)]
pub struct ExecutionPlanAnalysis {
    pub executor_ids: Vec<i64>,
    pub dependencies: HashMap<i64, Vec<i64>>,
    pub successors: HashMap<i64, Vec<i64>>,
    pub executor_types: HashMap<i64, crate::query::scheduler::types::ExecutorType>,
    pub output_variables: HashMap<i64, Variable>,
}

impl ExecutionPlanAnalysis {
    pub fn new() -> Self {
        Self {
            executor_ids: Vec::new(),
            dependencies: HashMap::new(),
            successors: HashMap::new(),
            executor_types: HashMap::new(),
            output_variables: HashMap::new(),
        }
    }

    pub fn add_executor(&mut self, id: i64, exec_type: crate::query::scheduler::types::ExecutorType) {
        self.executor_ids.push(id);
        self.executor_types.insert(id, exec_type);
        self.dependencies.entry(id).or_insert_with(Vec::new);
        self.successors.entry(id).or_insert_with(Vec::new);
    }

    pub fn add_dependency(&mut self, from: i64, to: i64) {
        self.dependencies.entry(to).or_insert_with(Vec::new).push(from);
        self.successors.entry(from).or_insert_with(Vec::new).push(to);
    }

    pub fn set_output_variable(&mut self, executor_id: i64, var: Variable) {
        self.output_variables.insert(executor_id, var);
    }
}

/// 执行计划分析访问者
///
/// 使用访问者模式遍历执行计划，分析执行器依赖关系和类型。
pub struct ExecutionPlanAnalyzer {
    analysis: ExecutionPlanAnalysis,
    next_id: i64,
    current_id: Option<i64>,
}

impl ExecutionPlanAnalyzer {
    pub fn new() -> Self {
        Self {
            analysis: ExecutionPlanAnalysis::new(),
            next_id: 1,
            current_id: None,
        }
    }

    pub fn analyze(&mut self, root: &PlanNodeEnum) -> ExecutionPlanAnalysis {
        self.visit(root);
        self.analysis.clone()
    }

    fn allocate_id(&mut self) -> i64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn set_current_id(&mut self, id: i64) {
        self.current_id = Some(id);
    }

    fn get_current_id(&self) -> Option<i64> {
        self.current_id
    }
}

impl Default for ExecutionPlanAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl PlanNodeVisitor for ExecutionPlanAnalyzer {
    type Result = ();

    fn visit_start(&mut self, node: &crate::query::planner::plan::core::nodes::StartNode) {
        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);
        
        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }
    }

    fn visit_project(&mut self, node: &crate::query::planner::plan::core::nodes::ProjectNode) {
        use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;

        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }

        if let Some(input_id) = self.current_id {
            self.analysis.add_dependency(id, input_id);
        }

        let input = node.input();
        self.visit(input);
    }

    fn visit_filter(&mut self, node: &crate::query::planner::plan::core::nodes::FilterNode) {
        use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;

        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }

        if let Some(input_id) = self.current_id {
            self.analysis.add_dependency(id, input_id);
        }

        let input = node.input();
        self.visit(input);
    }

    fn visit_sort(&mut self, node: &crate::query::planner::plan::core::nodes::SortNode) {
        use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;

        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }

        if let Some(input_id) = self.current_id {
            self.analysis.add_dependency(id, input_id);
        }

        let input = node.input();
        self.visit(input);
    }

    fn visit_limit(&mut self, node: &crate::query::planner::plan::core::nodes::LimitNode) {
        use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;

        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }

        if let Some(input_id) = self.current_id {
            self.analysis.add_dependency(id, input_id);
        }

        let input = node.input();
        self.visit(input);
    }

    fn visit_topn(&mut self, node: &crate::query::planner::plan::core::nodes::TopNNode) {
        use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;

        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }

        if let Some(input_id) = self.current_id {
            self.analysis.add_dependency(id, input_id);
        }

        let input = node.input();
        self.visit(input);
    }

    fn visit_aggregate(&mut self, node: &crate::query::planner::plan::core::nodes::AggregateNode) {
        use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;

        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }

        if let Some(input_id) = self.current_id {
            self.analysis.add_dependency(id, input_id);
        }

        let input = node.input();
        self.visit(input);
    }

    fn visit_get_vertices(&mut self, node: &crate::query::planner::plan::core::nodes::GetVerticesNode) {
        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Leaf);

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }
    }

    fn visit_get_edges(&mut self, node: &crate::query::planner::plan::core::nodes::GetEdgesNode) {
        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Leaf);

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }
    }

    fn visit_get_neighbors(&mut self, node: &crate::query::planner::plan::core::nodes::GetNeighborsNode) {
        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Leaf);

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }
    }

    fn visit_scan_vertices(&mut self, node: &crate::query::planner::plan::core::nodes::ScanVerticesNode) {
        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Leaf);

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }
    }

    fn visit_scan_edges(&mut self, node: &crate::query::planner::plan::core::nodes::ScanEdgesNode) {
        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Leaf);

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }
    }

    fn visit_index_scan(&mut self, node: &crate::query::planner::plan::algorithms::IndexScan) {
        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Leaf);

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }
    }

    fn visit_expand(&mut self, node: &crate::query::planner::plan::core::nodes::ExpandNode) {
        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }

        if !node.dependencies().is_empty() {
            let input = node.dependencies()[0].clone();
            self.visit(&input);
            if let Some(input_id) = self.get_current_id() {
                self.analysis.add_dependency(input_id, id);
            }
        }
    }

    fn visit_traverse(&mut self, node: &crate::query::planner::plan::core::nodes::TraverseNode) {
        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }

        if !node.dependencies().is_empty() {
            let input = node.dependencies()[0].clone();
            self.visit(&input);
            if let Some(input_id) = self.get_current_id() {
                self.analysis.add_dependency(input_id, id);
            }
        }
    }

    fn visit_inner_join(&mut self, node: &crate::query::planner::plan::core::nodes::InnerJoinNode) {
        use crate::query::planner::plan::core::nodes::plan_node_traits::BinaryInputNode;

        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }

        let left_input = node.left_input();
        self.visit(left_input);
        if let Some(left_id) = self.get_current_id() {
            self.analysis.add_dependency(left_id, id);
        }

        let right_input = node.right_input();
        self.visit(right_input);
        if let Some(right_id) = self.get_current_id() {
            self.analysis.add_dependency(right_id, id);
        }
    }

    fn visit_left_join(&mut self, node: &crate::query::planner::plan::core::nodes::LeftJoinNode) {
        use crate::query::planner::plan::core::nodes::plan_node_traits::BinaryInputNode;

        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }

        let left_input = node.left_input();
        self.visit(left_input);
        if let Some(left_id) = self.get_current_id() {
            self.analysis.add_dependency(left_id, id);
        }

        let right_input = node.right_input();
        self.visit(right_input);
        if let Some(right_id) = self.get_current_id() {
            self.analysis.add_dependency(right_id, id);
        }
    }

    fn visit_hash_inner_join(&mut self, node: &crate::query::planner::plan::core::nodes::HashInnerJoinNode) {
        use crate::query::planner::plan::core::nodes::plan_node_traits::BinaryInputNode;

        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }

        let left_input = node.left_input();
        self.visit(left_input);
        if let Some(left_id) = self.get_current_id() {
            self.analysis.add_dependency(left_id, id);
        }

        let right_input = node.right_input();
        self.visit(right_input);
        if let Some(right_id) = self.get_current_id() {
            self.analysis.add_dependency(right_id, id);
        }
    }

    fn visit_hash_left_join(&mut self, node: &crate::query::planner::plan::core::nodes::HashLeftJoinNode) {
        use crate::query::planner::plan::core::nodes::plan_node_traits::BinaryInputNode;

        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }

        let left_input = node.left_input();
        self.visit(left_input);
        if let Some(left_id) = self.get_current_id() {
            self.analysis.add_dependency(left_id, id);
        }

        let right_input = node.right_input();
        self.visit(right_input);
        if let Some(right_id) = self.get_current_id() {
            self.analysis.add_dependency(right_id, id);
        }
    }

    fn visit_cross_join(&mut self, node: &crate::query::planner::plan::core::nodes::CrossJoinNode) {
        use crate::query::planner::plan::core::nodes::plan_node_traits::BinaryInputNode;

        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }

        let left_input = node.left_input();
        self.visit(left_input);
        if let Some(left_id) = self.get_current_id() {
            self.analysis.add_dependency(left_id, id);
        }

        let right_input = node.right_input();
        self.visit(right_input);
        if let Some(right_id) = self.get_current_id() {
            self.analysis.add_dependency(right_id, id);
        }
    }

    fn visit_loop(&mut self, node: &crate::query::planner::plan::core::nodes::control_flow_node::LoopNode) {
        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Loop);

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }

        if let Some(body) = node.body() {
            self.visit(body);
            if let Some(body_id) = self.get_current_id() {
                self.analysis.add_dependency(body_id, id);
            }
        }
    }

    fn visit_argument(&mut self, node: &crate::query::planner::plan::core::nodes::ArgumentNode) {
        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Argument);

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }
    }

    fn visit_select(&mut self, node: &crate::query::planner::plan::core::nodes::control_flow_node::SelectNode) {
        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Select);

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }

        if let Some(if_branch) = node.if_branch() {
            self.visit(if_branch);
            if let Some(if_id) = self.get_current_id() {
                self.analysis.add_dependency(if_id, id);
            }
        }

        if let Some(else_branch) = node.else_branch() {
            self.visit(else_branch);
            if let Some(else_id) = self.get_current_id() {
                self.analysis.add_dependency(else_id, id);
            }
        }
    }

    fn visit_pass_through(&mut self, node: &crate::query::planner::plan::core::nodes::control_flow_node::PassThroughNode) {
        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }
    }

    fn visit_data_collect(&mut self, node: &crate::query::planner::plan::core::nodes::DataCollectNode) {
        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }

        if !node.dependencies().is_empty() {
            let input = node.dependencies()[0].clone();
            self.visit(&input);
            if let Some(input_id) = self.get_current_id() {
                self.analysis.add_dependency(input_id, id);
            }
        }
    }

    fn visit_dedup(&mut self, node: &crate::query::planner::plan::core::nodes::DedupNode) {
        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }

        if !node.dependencies().is_empty() {
            let input = node.dependencies()[0].clone();
            self.visit(&input);
            if let Some(input_id) = self.get_current_id() {
                self.analysis.add_dependency(input_id, id);
            }
        }
    }

    fn visit_pattern_apply(&mut self, node: &crate::query::planner::plan::core::nodes::PatternApplyNode) {
        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);

        let input = node.input();
        self.visit(input);
        if let Some(input_id) = self.get_current_id() {
            self.analysis.add_dependency(input_id, id);
        }

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }
    }

    fn visit_rollup_apply(&mut self, node: &crate::query::planner::plan::core::nodes::RollUpApplyNode) {
        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);

        let input = node.input();
        self.visit(input);
        if let Some(input_id) = self.get_current_id() {
            self.analysis.add_dependency(input_id, id);
        }

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }
    }

    fn visit_union(&mut self, node: &crate::query::planner::plan::core::nodes::UnionNode) {
        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);

        let input = node.input();
        self.visit(input);
        if let Some(input_id) = self.get_current_id() {
            self.analysis.add_dependency(input_id, id);
        }

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }
    }

    fn visit_unwind(&mut self, node: &crate::query::planner::plan::core::nodes::UnwindNode) {
        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);

        let input = node.input();
        self.visit(input);
        if let Some(input_id) = self.get_current_id() {
            self.analysis.add_dependency(input_id, id);
        }

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }
    }

    fn visit_assign(&mut self, node: &crate::query::planner::plan::core::nodes::AssignNode) {
        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);

        let input = node.input();
        self.visit(input);
        if let Some(input_id) = self.get_current_id() {
            self.analysis.add_dependency(input_id, id);
        }

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }
    }

    fn visit_sample(&mut self, node: &crate::query::planner::plan::core::nodes::SampleNode) {
        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);

        let input = node.input();
        self.visit(input);
        if let Some(input_id) = self.get_current_id() {
            self.analysis.add_dependency(input_id, id);
        }

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }
    }

    fn visit_expand_all(&mut self, node: &crate::query::planner::plan::core::nodes::ExpandAllNode) {
        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);

        for dep in node.dependencies() {
            self.visit(dep.as_ref());
            if let Some(input_id) = self.get_current_id() {
                self.analysis.add_dependency(input_id, id);
            }
        }

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }
    }

    fn visit_append_vertices(&mut self, node: &crate::query::planner::plan::core::nodes::AppendVerticesNode) {
        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);

        for dep in node.dependencies() {
            self.visit(dep.as_ref());
            if let Some(input_id) = self.get_current_id() {
                self.analysis.add_dependency(input_id, id);
            }
        }

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }
    }

    fn visit_multi_shortest_path(&mut self, node: &crate::query::planner::plan::algorithms::MultiShortestPath) {
        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);

        for dep in &node.deps {
            self.visit(dep);
            if let Some(input_id) = self.get_current_id() {
                self.analysis.add_dependency(input_id, id);
            }
        }

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }
    }

    fn visit_bfs_shortest(&mut self, node: &crate::query::planner::plan::algorithms::BFSShortest) {
        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);

        for dep in &node.deps {
            self.visit(dep);
            if let Some(input_id) = self.get_current_id() {
                self.analysis.add_dependency(input_id, id);
            }
        }

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }
    }

    fn visit_all_paths(&mut self, node: &crate::query::planner::plan::algorithms::AllPaths) {
        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);

        for dep in &node.deps {
            self.visit(dep);
            if let Some(input_id) = self.get_current_id() {
                self.analysis.add_dependency(input_id, id);
            }
        }

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }
    }

    fn visit_shortest_path(&mut self, node: &crate::query::planner::plan::algorithms::ShortestPath) {
        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);

        for dep in &node.deps {
            self.visit(dep);
            if let Some(input_id) = self.get_current_id() {
                self.analysis.add_dependency(input_id, id);
            }
        }

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }
    }

    fn visit_fulltext_index_scan(&mut self, node: &crate::query::planner::plan::algorithms::FulltextIndexScan) {
        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Leaf);

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }
    }

    fn visit_create_space(&mut self, node: &crate::query::planner::plan::core::nodes::CreateSpaceNode) {
        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }
    }

    fn visit_drop_space(&mut self, node: &crate::query::planner::plan::core::nodes::DropSpaceNode) {
        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }
    }

    fn visit_desc_space(&mut self, node: &crate::query::planner::plan::core::nodes::DescSpaceNode) {
        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }
    }

    fn visit_show_spaces(&mut self, node: &crate::query::planner::plan::core::nodes::ShowSpacesNode) {
        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }
    }

    fn visit_create_tag(&mut self, node: &crate::query::planner::plan::core::nodes::CreateTagNode) {
        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }
    }

    fn visit_alter_tag(&mut self, node: &crate::query::planner::plan::core::nodes::AlterTagNode) {
        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }
    }

    fn visit_desc_tag(&mut self, node: &crate::query::planner::plan::core::nodes::DescTagNode) {
        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }
    }

    fn visit_drop_tag(&mut self, node: &crate::query::planner::plan::core::nodes::DropTagNode) {
        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }
    }

    fn visit_show_tags(&mut self, node: &crate::query::planner::plan::core::nodes::ShowTagsNode) {
        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }
    }

    fn visit_create_edge(&mut self, node: &crate::query::planner::plan::core::nodes::CreateEdgeNode) {
        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }
    }

    fn visit_alter_edge(&mut self, node: &crate::query::planner::plan::core::nodes::AlterEdgeNode) {
        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }
    }

    fn visit_desc_edge(&mut self, node: &crate::query::planner::plan::core::nodes::DescEdgeNode) {
        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }
    }

    fn visit_drop_edge(&mut self, node: &crate::query::planner::plan::core::nodes::DropEdgeNode) {
        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }
    }

    fn visit_show_edges(&mut self, node: &crate::query::planner::plan::core::nodes::ShowEdgesNode) {
        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }
    }

    fn visit_create_tag_index(&mut self, node: &crate::query::planner::plan::core::nodes::CreateTagIndexNode) {
        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }
    }

    fn visit_drop_tag_index(&mut self, node: &crate::query::planner::plan::core::nodes::DropTagIndexNode) {
        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }
    }

    fn visit_desc_tag_index(&mut self, node: &crate::query::planner::plan::core::nodes::DescTagIndexNode) {
        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }
    }

    fn visit_show_tag_indexes(&mut self, node: &crate::query::planner::plan::core::nodes::ShowTagIndexesNode) {
        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }
    }

    fn visit_create_edge_index(&mut self, node: &crate::query::planner::plan::core::nodes::CreateEdgeIndexNode) {
        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }
    }

    fn visit_drop_edge_index(&mut self, node: &crate::query::planner::plan::core::nodes::DropEdgeIndexNode) {
        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }
    }

    fn visit_desc_edge_index(&mut self, node: &crate::query::planner::plan::core::nodes::DescEdgeIndexNode) {
        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }
    }

    fn visit_show_edge_indexes(&mut self, node: &crate::query::planner::plan::core::nodes::ShowEdgeIndexesNode) {
        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }
    }

    fn visit_rebuild_tag_index(&mut self, node: &crate::query::planner::plan::core::nodes::RebuildTagIndexNode) {
        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }
    }

    fn visit_rebuild_edge_index(&mut self, node: &crate::query::planner::plan::core::nodes::RebuildEdgeIndexNode) {
        let id = self.allocate_id();
        self.set_current_id(id);
        self.analysis.add_executor(id, crate::query::scheduler::types::ExecutorType::Normal);

        if let Some(output_var) = node.output_var() {
            self.analysis.set_output_variable(id, output_var.clone());
        }
    }
}
