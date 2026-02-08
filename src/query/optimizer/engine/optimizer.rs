//! 优化器引擎核心实现
//! 提供 Optimizer 结构体及其优化逻辑实现
//!
//! Optimizer 是优化器的主类，负责协调整个优化过程：
//! 1. 将执行计划转换为 OptGroup 结构
//! 2. 按阶段执行优化规则
//! 3. 生成最终的执行计划

use std::cell::RefCell;
use std::rc::Rc;

use crate::query::context::execution::QueryContext;
use crate::query::optimizer::core::config::{OptimizationConfig, OptimizationStats};
use crate::query::optimizer::core::OptimizationPhase;
use crate::query::optimizer::plan::{
    OptContext, OptGroup, OptGroupNode, OptRule,
};
use crate::query::planner::plan::{ExecutionPlan, PlanNodeEnum};
use crate::query::optimizer::{OptimizationRule, OptimizerError};

#[derive(Debug)]
pub struct RuleSet {
    pub name: String,
    pub rules: Vec<Rc<dyn OptRule>>,
}

impl RuleSet {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            rules: Vec::new(),
        }
    }

    pub fn add_rule(&mut self, rule: Rc<dyn OptRule>) {
        self.rules.push(rule);
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }
}

#[derive(Debug)]
pub struct Optimizer {
    pub config: OptimizationConfig,
    pub rule_sets: Vec<RuleSet>,
    pub enable_cost_model: bool,
    pub enable_rule_based: bool,
}

impl Default for Optimizer {
    fn default() -> Self {
        Self::new(OptimizationConfig::default())
    }
}

impl Optimizer {
    pub fn new(config: OptimizationConfig) -> Self {
        let mut optimizer = Self {
            config,
            rule_sets: Vec::new(),
            enable_cost_model: true,
            enable_rule_based: true,
        };

        optimizer.setup_default_rule_sets();
        optimizer
    }

    pub fn from_registry() -> Self {
        let config = OptimizationConfig::default();
        Self::new(config)
    }

    pub fn with_config(rule_sets: Vec<RuleSet>, config: OptimizationConfig) -> Self {
        let mut optimizer = Self {
            config,
            rule_sets,
            enable_cost_model: true,
            enable_rule_based: true,
        };

        if optimizer.rule_sets.is_empty() {
            optimizer.setup_default_rule_sets();
        }

        optimizer
    }

    pub fn find_best_plan(
        &mut self,
        query_context: &mut QueryContext,
        plan: ExecutionPlan,
    ) -> Result<ExecutionPlan, OptimizerError> {
        self.optimize(plan, query_context)
    }

    fn setup_default_rule_sets(&mut self) {
        let mut rewrite_rules = RuleSet::new("rewrite");
        if let Some(rule) = OptimizationRule::PushFilterDownAggregate.create_instance() {
            rewrite_rules.add_rule(rule);
        }
        self.rule_sets.push(rewrite_rules);

        let mut logical_rules = RuleSet::new("logical");
        if let Some(rule) = OptimizationRule::CollapseProject.create_instance() {
            logical_rules.add_rule(rule);
        }
        if let Some(rule) = OptimizationRule::CombineFilter.create_instance() {
            logical_rules.add_rule(rule);
        }
        if let Some(rule) = OptimizationRule::DedupElimination.create_instance() {
            logical_rules.add_rule(rule);
        }
        if let Some(rule) = OptimizationRule::RemoveNoopProject.create_instance() {
            logical_rules.add_rule(rule);
        }
        self.rule_sets.push(logical_rules);

        let mut physical_rules = RuleSet::new("physical");
        if let Some(rule) = OptimizationRule::IndexScan.create_instance() {
            physical_rules.add_rule(rule);
        }
        if let Some(rule) = OptimizationRule::JoinOptimization.create_instance() {
            physical_rules.add_rule(rule);
        }
        if let Some(rule) = OptimizationRule::PushLimitDownGetVertices.create_instance() {
            physical_rules.add_rule(rule);
        }
        self.rule_sets.push(physical_rules);
    }

    pub fn optimize(
        &mut self,
        plan: ExecutionPlan,
        query_context: &mut QueryContext,
    ) -> Result<ExecutionPlan, OptimizerError> {
        let mut opt_ctx = OptContext::new(query_context.clone());

        let root_opt_node = self.build_initial_opt_group(&plan, &mut opt_ctx)?;

        let mut root_group = opt_ctx
            .find_group_by_id(root_opt_node.borrow().id)
            .ok_or(OptimizerError::group_not_found(root_opt_node.borrow().id))?
            .clone();

        self.execute_optimization(&mut opt_ctx, &mut root_group)?;

        let optimized_plan = self.extract_execution_plan(&root_group, &mut opt_ctx)?;

        Ok(optimized_plan)
    }

    pub fn optimize_with_stats(
        &mut self,
        plan: ExecutionPlan,
        query_context: &mut QueryContext,
    ) -> Result<(ExecutionPlan, OptimizationStats), OptimizerError> {
        let mut opt_ctx = OptContext::new(query_context.clone());
        let mut stats = OptimizationStats::default();
        
        stats.plan_nodes_before = self.count_nodes(&plan);

        let root_opt_node = self.build_initial_opt_group(&plan, &mut opt_ctx)?;

        let mut root_group = opt_ctx
            .find_group_by_id(root_opt_node.borrow().id)
            .ok_or(OptimizerError::group_not_found(root_opt_node.borrow().id))?
            .clone();

        self.execute_optimization(&mut opt_ctx, &mut root_group)?;

        let optimized_plan = self.extract_execution_plan(&root_group, &mut opt_ctx)?;

        stats.plan_nodes_after = self.count_nodes(&optimized_plan);

        Ok((optimized_plan, stats))
    }

    fn count_nodes(&self, plan: &ExecutionPlan) -> usize {
        fn count_recursive(node: &PlanNodeEnum) -> usize {
            let mut count = 1;
            count_recursive_node(node, &mut count);
            count
        }

        fn count_recursive_node(node: &PlanNodeEnum, count: &mut usize) {
            use crate::query::planner::plan::core::nodes::{SingleInputNode, BinaryInputNode};

            match node {
                PlanNodeEnum::Project(node) => {
                    count_recursive_node(node.input(), count);
                }
                PlanNodeEnum::Filter(node) => {
                    count_recursive_node(node.input(), count);
                }
                PlanNodeEnum::Sort(node) => {
                    count_recursive_node(node.input(), count);
                }
                PlanNodeEnum::Limit(node) => {
                    count_recursive_node(node.input(), count);
                }
                PlanNodeEnum::TopN(node) => {
                    count_recursive_node(node.input(), count);
                }
                PlanNodeEnum::Aggregate(node) => {
                    count_recursive_node(node.input(), count);
                }
                PlanNodeEnum::InnerJoin(node) => {
                    count_recursive_node(node.left_input(), count);
                    count_recursive_node(node.right_input(), count);
                }
                PlanNodeEnum::LeftJoin(node) => {
                    count_recursive_node(node.left_input(), count);
                    count_recursive_node(node.right_input(), count);
                }
                PlanNodeEnum::HashInnerJoin(node) => {
                    count_recursive_node(node.left_input(), count);
                    count_recursive_node(node.right_input(), count);
                }
                PlanNodeEnum::HashLeftJoin(node) => {
                    count_recursive_node(node.left_input(), count);
                    count_recursive_node(node.right_input(), count);
                }
                PlanNodeEnum::GetNeighbors(_) | PlanNodeEnum::GetVertices(_) | PlanNodeEnum::GetEdges(_) => {
                }
                PlanNodeEnum::Sample(node) => {
                    count_recursive_node(node.input(), count);
                }
                PlanNodeEnum::Dedup(node) => {
                    count_recursive_node(node.input(), count);
                }
                PlanNodeEnum::Unwind(node) => {
                    count_recursive_node(node.input(), count);
                }
                PlanNodeEnum::Expand(node) => {
                    for dep in node.dependencies().iter() {
                        count_recursive_node(dep, count);
                    }
                }
                PlanNodeEnum::ExpandAll(node) => {
                    for dep in node.dependencies().iter() {
                        count_recursive_node(dep, count);
                    }
                }
                PlanNodeEnum::Traverse(node) => {
                    for dep in node.dependencies().iter() {
                        count_recursive_node(dep, count);
                    }
                }
                PlanNodeEnum::AppendVertices(node) => {
                    for dep in node.dependencies().iter() {
                        count_recursive_node(dep, count);
                    }
                }
                PlanNodeEnum::Union(node) => {
                    for input in node.dependencies().iter() {
                        count_recursive_node(input, count);
                    }
                }
                PlanNodeEnum::CrossJoin(node) => {
                    count_recursive_node(node.left_input(), count);
                    count_recursive_node(node.right_input(), count);
                }
                PlanNodeEnum::ScanVertices(_) | PlanNodeEnum::ScanEdges(_) | PlanNodeEnum::Start(_) => {
                }
                _ => {}
            }
        }

        if let Some(root) = plan.root() {
            count_recursive(root)
        } else {
            0
        }
    }

    fn build_initial_opt_group(
        &mut self,
        plan: &ExecutionPlan,
        ctx: &mut OptContext,
    ) -> Result<Rc<RefCell<OptGroupNode>>, OptimizerError> {
        if let Some(root) = plan.root() {
            self.build_opt_group_recursive(root, ctx, vec![])
        } else {
            Err(OptimizerError::no_viable_plan())
        }
    }

    fn build_opt_group_recursive(
        &mut self,
        plan_node: &PlanNodeEnum,
        ctx: &mut OptContext,
        _current_dependencies: Vec<usize>,
    ) -> Result<Rc<RefCell<OptGroupNode>>, OptimizerError> {
        let node_id = ctx.allocate_node_id();
        let group_node = ctx.get_group_node_from_pool(node_id, plan_node.clone());

        let group_node_rc = Rc::new(RefCell::new(group_node));
        ctx.add_group_node(group_node_rc.clone())?;

        self.build_inputs_recursive(plan_node, ctx, node_id)?;

        Ok(group_node_rc)
    }

    fn build_inputs_recursive(
        &mut self,
        plan_node: &PlanNodeEnum,
        ctx: &mut OptContext,
        parent_id: usize,
    ) -> Result<(), OptimizerError> {
        use crate::query::planner::plan::core::nodes::{SingleInputNode, BinaryInputNode};

        match plan_node {
            PlanNodeEnum::Project(node) => {
                let input_id = self.build_single_input(node.input(), ctx)?;
                if let Some(group_node) = ctx.find_group_node_by_id(parent_id) {
                    group_node.borrow_mut().add_dependency(input_id);
                }
                self.build_inputs_recursive(node.input(), ctx, input_id)?;
            }
            PlanNodeEnum::Filter(node) => {
                let input_id = self.build_single_input(node.input(), ctx)?;
                if let Some(group_node) = ctx.find_group_node_by_id(parent_id) {
                    group_node.borrow_mut().add_dependency(input_id);
                }
                self.build_inputs_recursive(node.input(), ctx, input_id)?;
            }
            PlanNodeEnum::Sort(node) => {
                let input_id = self.build_single_input(node.input(), ctx)?;
                if let Some(group_node) = ctx.find_group_node_by_id(parent_id) {
                    group_node.borrow_mut().add_dependency(input_id);
                }
                self.build_inputs_recursive(node.input(), ctx, input_id)?;
            }
            PlanNodeEnum::Limit(node) => {
                let input_id = self.build_single_input(node.input(), ctx)?;
                if let Some(group_node) = ctx.find_group_node_by_id(parent_id) {
                    group_node.borrow_mut().add_dependency(input_id);
                }
                self.build_inputs_recursive(node.input(), ctx, input_id)?;
            }
            PlanNodeEnum::Aggregate(node) => {
                let input_id = self.build_single_input(node.input(), ctx)?;
                if let Some(group_node) = ctx.find_group_node_by_id(parent_id) {
                    group_node.borrow_mut().add_dependency(input_id);
                }
                self.build_inputs_recursive(node.input(), ctx, input_id)?;
            }
            PlanNodeEnum::InnerJoin(node) => {
                let left_id = self.build_single_input(node.left_input(), ctx)?;
                let right_id = self.build_single_input(node.right_input(), ctx)?;
                if let Some(group_node) = ctx.find_group_node_by_id(parent_id) {
                    group_node.borrow_mut().add_dependency(left_id);
                    group_node.borrow_mut().add_dependency(right_id);
                }
                self.build_inputs_recursive(node.left_input(), ctx, left_id)?;
                self.build_inputs_recursive(node.right_input(), ctx, right_id)?;
            }
            PlanNodeEnum::LeftJoin(node) => {
                let left_id = self.build_single_input(node.left_input(), ctx)?;
                let right_id = self.build_single_input(node.right_input(), ctx)?;
                if let Some(group_node) = ctx.find_group_node_by_id(parent_id) {
                    group_node.borrow_mut().add_dependency(left_id);
                    group_node.borrow_mut().add_dependency(right_id);
                }
                self.build_inputs_recursive(node.left_input(), ctx, left_id)?;
                self.build_inputs_recursive(node.right_input(), ctx, right_id)?;
            }
            PlanNodeEnum::HashInnerJoin(node) => {
                let left_id = self.build_single_input(node.left_input(), ctx)?;
                let right_id = self.build_single_input(node.right_input(), ctx)?;
                if let Some(group_node) = ctx.find_group_node_by_id(parent_id) {
                    group_node.borrow_mut().add_dependency(left_id);
                    group_node.borrow_mut().add_dependency(right_id);
                }
                self.build_inputs_recursive(node.left_input(), ctx, left_id)?;
                self.build_inputs_recursive(node.right_input(), ctx, right_id)?;
            }
            PlanNodeEnum::HashLeftJoin(node) => {
                let left_id = self.build_single_input(node.left_input(), ctx)?;
                let right_id = self.build_single_input(node.right_input(), ctx)?;
                if let Some(group_node) = ctx.find_group_node_by_id(parent_id) {
                    group_node.borrow_mut().add_dependency(left_id);
                    group_node.borrow_mut().add_dependency(right_id);
                }
                self.build_inputs_recursive(node.left_input(), ctx, left_id)?;
                self.build_inputs_recursive(node.right_input(), ctx, right_id)?;
            }
            PlanNodeEnum::GetNeighbors(_) => {}
            PlanNodeEnum::GetVertices(_) => {}
            PlanNodeEnum::GetEdges(_) => {}
            PlanNodeEnum::Union(node) => {
                for dep in node.dependencies().iter() {
                    let dep_id = self.build_single_input(dep, ctx)?;
                    if let Some(group_node) = ctx.find_group_node_by_id(parent_id) {
                        group_node.borrow_mut().add_dependency(dep_id);
                    }
                    self.build_inputs_recursive(dep, ctx, dep_id)?;
                }
            }
            PlanNodeEnum::CrossJoin(node) => {
                let left_id = self.build_single_input(node.left_input(), ctx)?;
                let right_id = self.build_single_input(node.right_input(), ctx)?;
                if let Some(group_node) = ctx.find_group_node_by_id(parent_id) {
                    group_node.borrow_mut().add_dependency(left_id);
                    group_node.borrow_mut().add_dependency(right_id);
                }
                self.build_inputs_recursive(node.left_input(), ctx, left_id)?;
                self.build_inputs_recursive(node.right_input(), ctx, right_id)?;
            }
            PlanNodeEnum::TopN(node) => {
                let input_id = self.build_single_input(node.input(), ctx)?;
                if let Some(group_node) = ctx.find_group_node_by_id(parent_id) {
                    group_node.borrow_mut().add_dependency(input_id);
                }
                self.build_inputs_recursive(node.input(), ctx, input_id)?;
            }
            PlanNodeEnum::Sample(node) => {
                let input_id = self.build_single_input(node.input(), ctx)?;
                if let Some(group_node) = ctx.find_group_node_by_id(parent_id) {
                    group_node.borrow_mut().add_dependency(input_id);
                }
                self.build_inputs_recursive(node.input(), ctx, input_id)?;
            }
            PlanNodeEnum::Dedup(node) => {
                let input_id = self.build_single_input(node.input(), ctx)?;
                if let Some(group_node) = ctx.find_group_node_by_id(parent_id) {
                    group_node.borrow_mut().add_dependency(input_id);
                }
                self.build_inputs_recursive(node.input(), ctx, input_id)?;
            }
            PlanNodeEnum::Unwind(node) => {
                let input_id = self.build_single_input(node.input(), ctx)?;
                if let Some(group_node) = ctx.find_group_node_by_id(parent_id) {
                    group_node.borrow_mut().add_dependency(input_id);
                }
                self.build_inputs_recursive(node.input(), ctx, input_id)?;
            }
            PlanNodeEnum::Expand(node) => {
                for dep in node.dependencies().iter() {
                    let dep_id = self.build_single_input(dep, ctx)?;
                    if let Some(group_node) = ctx.find_group_node_by_id(parent_id) {
                        group_node.borrow_mut().add_dependency(dep_id);
                    }
                    self.build_inputs_recursive(dep, ctx, dep_id)?;
                }
            }
            PlanNodeEnum::ExpandAll(node) => {
                for dep in node.dependencies().iter() {
                    let dep_id = self.build_single_input(dep, ctx)?;
                    if let Some(group_node) = ctx.find_group_node_by_id(parent_id) {
                        group_node.borrow_mut().add_dependency(dep_id);
                    }
                    self.build_inputs_recursive(dep, ctx, dep_id)?;
                }
            }
            PlanNodeEnum::Traverse(node) => {
                for dep in node.dependencies().iter() {
                    let dep_id = self.build_single_input(dep, ctx)?;
                    if let Some(group_node) = ctx.find_group_node_by_id(parent_id) {
                        group_node.borrow_mut().add_dependency(dep_id);
                    }
                    self.build_inputs_recursive(dep, ctx, dep_id)?;
                }
            }
            PlanNodeEnum::AppendVertices(node) => {
                for dep in node.dependencies().iter() {
                    let dep_id = self.build_single_input(dep, ctx)?;
                    if let Some(group_node) = ctx.find_group_node_by_id(parent_id) {
                        group_node.borrow_mut().add_dependency(dep_id);
                    }
                    self.build_inputs_recursive(dep, ctx, dep_id)?;
                }
            }
            PlanNodeEnum::Start(_) | PlanNodeEnum::IndexScan(_) | PlanNodeEnum::FulltextIndexScan(_)
            | PlanNodeEnum::ScanVertices(_) | PlanNodeEnum::ScanEdges(_)
            | PlanNodeEnum::Argument(_) | PlanNodeEnum::Loop(_) | PlanNodeEnum::PassThrough(_)
            | PlanNodeEnum::Select(_) | PlanNodeEnum::DataCollect(_) | PlanNodeEnum::PatternApply(_)
            | PlanNodeEnum::RollUpApply(_) | PlanNodeEnum::Assign(_)
            | PlanNodeEnum::MultiShortestPath(_) | PlanNodeEnum::BFSShortest(_) 
            | PlanNodeEnum::AllPaths(_) | PlanNodeEnum::ShortestPath(_)
            | PlanNodeEnum::CreateSpace(_) | PlanNodeEnum::DropSpace(_) | PlanNodeEnum::DescSpace(_)
            | PlanNodeEnum::ShowSpaces(_) | PlanNodeEnum::CreateTag(_) | PlanNodeEnum::AlterTag(_)
            | PlanNodeEnum::DescTag(_) | PlanNodeEnum::DropTag(_) | PlanNodeEnum::ShowTags(_)
            | PlanNodeEnum::CreateEdge(_) | PlanNodeEnum::AlterEdge(_) | PlanNodeEnum::DescEdge(_)
            | PlanNodeEnum::DropEdge(_) | PlanNodeEnum::ShowEdges(_)
            | PlanNodeEnum::CreateTagIndex(_) | PlanNodeEnum::DropTagIndex(_) | PlanNodeEnum::DescTagIndex(_)
            | PlanNodeEnum::ShowTagIndexes(_) | PlanNodeEnum::CreateEdgeIndex(_) | PlanNodeEnum::DropEdgeIndex(_)
            | PlanNodeEnum::DescEdgeIndex(_) | PlanNodeEnum::ShowEdgeIndexes(_)
            | PlanNodeEnum::RebuildTagIndex(_) | PlanNodeEnum::RebuildEdgeIndex(_)
            | PlanNodeEnum::CreateUser(_) | PlanNodeEnum::AlterUser(_) | PlanNodeEnum::DropUser(_)
            | PlanNodeEnum::ChangePassword(_) => {
                // These nodes don't have inputs to process in the current context
            }
        }

        Ok(())
    }

    fn build_single_input(
        &mut self,
        input: &PlanNodeEnum,
        ctx: &mut OptContext,
    ) -> Result<usize, OptimizerError> {
        let node_id = ctx.allocate_node_id();
        let group_node = ctx.get_group_node_from_pool(node_id, input.clone());
        let group_node_rc = Rc::new(RefCell::new(group_node));
        ctx.add_group_node(group_node_rc.clone())?;
        Ok(node_id)
    }

    fn execute_optimization(
        &mut self,
        ctx: &mut OptContext,
        root_group: &mut OptGroup,
    ) -> Result<(), OptimizerError> {
        self.execute_phase_optimization(ctx, root_group, OptimizationPhase::Rewrite)?;
        self.execute_phase_optimization(ctx, root_group, OptimizationPhase::Logical)?;
        self.execute_phase_optimization(ctx, root_group, OptimizationPhase::Physical)?;

        Ok(())
    }

    fn execute_phase_optimization(
        &mut self,
        ctx: &mut OptContext,
        root_group: &mut OptGroup,
        phase: OptimizationPhase,
    ) -> Result<(), OptimizerError> {
        let phase_rule_names = self.get_rule_names_for_phase(&phase);

        let max_rounds = self.config.max_iteration_rounds;
        let min_rounds = self.config.min_iteration_rounds;
        let stable_threshold = self.config.stable_threshold;
        let enable_adaptive = self.config.enable_adaptive_iteration;

        let mut round = 0;
        let mut stable_count = 0;

        while round < max_rounds {
            let before_nodes = root_group.nodes.len();

            ctx.set_changed(false);

            for rule_name in &phase_rule_names {
                let rule = self.find_rule(rule_name);
                if let Some(rule) = rule {
                    self.apply_rule(ctx, &*root_group, &*rule)?;
                    self.clear_visited(ctx);
                }
            }

            round += 1;

            if ctx.changed() {
                stable_count = 0;
            } else {
                stable_count += 1;
            }

            let _last_changes = root_group.nodes.len() - before_nodes;

            if enable_adaptive
                && round >= min_rounds
                && stable_count >= stable_threshold
                && _last_changes == 0
            {
                break;
            }
        }

        Ok(())
    }

    fn clear_visited(&mut self, ctx: &mut OptContext) {
        let group_ids: Vec<usize> = {
            let group_map = ctx.group_map_mut();
            group_map.keys().cloned().collect()
        };

        for group_id in group_ids {
            if let Some(mut group) = ctx.find_group_by_id_mut(group_id) {
                group.set_visited(false);
                ctx.register_group(group);
            }
        }
    }

    fn get_rule_names_for_phase(&self, phase: &OptimizationPhase) -> Vec<&'static str> {
        match phase {
            OptimizationPhase::Rewrite => vec![
                "ExpandGetNeighborsRule",
                "AddVertexIdRule",
                "PushFilterDownAggregateRule",
                "LimitPushDownRule",
                "PredicatePushDownRule",
            ],
            OptimizationPhase::Logical => vec![
                "UnionEdgeTypeGroupRule",
                "GetNodeRule",
                "GetEdgeRule",
                "DedupNodeRule",
                "SortRule",
                "CollapseProjectRule",
                "CollapseFilterRule",
                "BinaryJoinRule",
            ],
            OptimizationPhase::Physical => vec![
                "IndexScanRule",
                "VertexIndexScanRule",
                "EdgeIndexScanRule",
                "HashJoinRule",
                "SortRule",
                "LimitRule",
            ],
            _ => Vec::new(),
        }
    }

    fn find_rule(&self, name: &str) -> Option<Rc<dyn OptRule>> {
        for rs in &self.rule_sets {
            for rule in &rs.rules {
                if rule.name() == name {
                    return Some(Rc::clone(rule));
                }
            }
        }
        None
    }

    fn apply_rule(
        &mut self,
        ctx: &mut OptContext,
        root_group: &OptGroup,
        rule: &dyn OptRule,
    ) -> Result<(), OptimizerError> {
        let root_group_id = root_group.id;
        self.explore_until_max_round(ctx, root_group_id, rule)
    }

    fn explore_until_max_round(
        &mut self,
        ctx: &mut OptContext,
        group_id: usize,
        rule: &dyn OptRule,
    ) -> Result<(), OptimizerError> {
        const MAX_EXPLORATION_ROUND: i32 = 128;
        let mut max_round = MAX_EXPLORATION_ROUND;
        
        while !self.is_group_explored(ctx, group_id, rule) {
            if max_round <= 0 {
                break;
            }
            max_round -= 1;
            self.explore_group(ctx, group_id, rule)?;
        }
        
        Ok(())
    }

    fn is_group_explored(&self, ctx: &OptContext, group_id: usize, rule: &dyn OptRule) -> bool {
        if let Some(group) = ctx.find_group_by_id(group_id) {
            group.is_explored(rule.name())
        } else {
            true
        }
    }

    fn explore_group(
        &mut self,
        ctx: &mut OptContext,
        group_id: usize,
        rule: &dyn OptRule,
    ) -> Result<(), OptimizerError> {
        if self.is_group_explored(ctx, group_id, rule) {
            return Ok(());
        }

        if let Some(mut group) = ctx.find_group_by_id_mut(group_id) {
            group.set_explored(rule.name());
            ctx.register_group(group);
        }

        if let Some(mut group) = ctx.find_group_by_id_mut(group_id) {
            if group.is_visited() {
                return Ok(());
            }
            group.set_visited(true);
            ctx.register_group(group);
        }

        let nodes: Vec<Rc<RefCell<OptGroupNode>>> = {
            if let Some(group) = ctx.find_group_by_id(group_id) {
                group.nodes.iter().cloned().collect()
            } else {
                return Ok(());
            }
        };

        for node_rc in nodes {
            if self.is_node_explored(&node_rc.borrow(), rule) {
                continue;
            }

            self.explore_group_node(ctx, &node_rc, rule)?;

            if !rule.pattern().matches(&node_rc.borrow().plan_node) {
                continue;
            }

            match rule.apply(ctx, &node_rc) {
                Ok(Some(result)) => {
                    let node_id = node_rc.borrow().id;
                    let node_dependencies = node_rc.borrow().dependencies.clone();
                    let has_new_nodes = !result.new_group_nodes.is_empty();

                    if result.erase_curr || result.erase_all {
                        if let Some(mut group) = ctx.find_group_by_id_mut(node_id) {
                            group.nodes.retain(|n| n.borrow().id != node_id);
                            ctx.register_group(group);
                        }
                    }

                    for new_node in result.new_group_nodes {
                        let new_node_id = new_node.borrow().id;
                        if let Some(mut group) = ctx.find_group_by_id_mut(new_node_id) {
                            if !group.nodes.iter().any(|n| n.borrow().id == new_node_id) {
                                group.add_node(new_node);
                            }
                            ctx.register_group(group);
                        } else {
                            let mut new_group = OptGroup::new(new_node_id);
                            new_group.add_node(new_node);
                            ctx.register_group(new_group);
                        }
                    }

                    for &new_dep in &result.new_dependencies {
                        if !node_dependencies.contains(&new_dep) {
                            let mut node_mut = node_rc.borrow_mut();
                            node_mut.dependencies.push(new_dep);
                        }
                    }

                    ctx.set_changed(true);

                    if has_new_nodes {
                        self.set_group_unexplored(ctx, group_id, rule);
                    }
                }
                Ok(None) => {
                    let mut node_mut = node_rc.borrow_mut();
                    node_mut.set_explored(rule.name());
                }
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }

    fn explore_group_node(
        &mut self,
        ctx: &mut OptContext,
        node: &Rc<RefCell<OptGroupNode>>,
        rule: &dyn OptRule,
    ) -> Result<(), OptimizerError> {
        if self.is_node_explored(&node.borrow(), rule) {
            return Ok(());
        }

        let mut node_mut = node.borrow_mut();
        node_mut.set_explored(rule.name());
        drop(node_mut);

        let dependencies: Vec<usize> = node.borrow().dependencies.clone();
        let bodies: Vec<usize> = node.borrow().bodies.clone();

        for dep_id in dependencies {
            self.explore_until_max_round(ctx, dep_id, rule)?;
        }

        for body_id in bodies {
            self.explore_until_max_round(ctx, body_id, rule)?;
        }

        Ok(())
    }

    fn is_node_explored(&self, node: &OptGroupNode, rule: &dyn OptRule) -> bool {
        node.is_explored(rule.name())
    }

    fn set_group_unexplored(&mut self, ctx: &mut OptContext, group_id: usize, rule: &dyn OptRule) {
        if let Some(mut group) = ctx.find_group_by_id_mut(group_id) {
            if group.is_visited() {
                return;
            }
            group.set_visited(true);
            group.set_unexplored(rule.name());

            let node_ids: Vec<usize> = group.nodes.iter().map(|n| n.borrow().id).collect();
            let dependencies = group.get_all_dependencies();
            let bodies = group.get_all_bodies();

            ctx.register_group(group);

            for node_id in node_ids {
                if let Some(node_group) = ctx.find_group_by_id(node_id) {
                    if let Some(node_rc) = node_group.get_node_by_id(node_id) {
                        let mut node_mut = node_rc.borrow_mut();
                        node_mut.set_unexplored(rule.name());
                    }
                }
            }

            for dep_id in dependencies {
                self.set_group_unexplored(ctx, dep_id, rule);
            }

            for body_id in bodies {
                self.set_group_unexplored(ctx, body_id, rule);
            }
        }
    }

    fn explore_node(
        &mut self,
        ctx: &mut OptContext,
        node: &Rc<RefCell<OptGroupNode>>,
        rule: &dyn OptRule,
    ) -> Result<(), OptimizerError> {
        let node_borrowed = node.borrow();
        if !rule.pattern().matches(&node_borrowed.plan_node) {
            return Ok(());
        }

        let node_id = node.borrow().id;
        let node_dependencies = node.borrow().dependencies.clone();

        drop(node_borrowed);

        match rule.apply(ctx, node) {
            Ok(Some(result)) => {
                if result.erase_curr || result.erase_all {
                    if let Some(mut group) = ctx.find_group_by_id_mut(node_id) {
                        group.nodes.retain(|n| n.borrow().id != node_id);
                        ctx.register_group(group);
                    }
                }

                for new_node in result.new_group_nodes {
                    let new_node_id = new_node.borrow().id;
                    if let Some(mut group) = ctx.find_group_by_id_mut(new_node_id) {
                        if !group.nodes.iter().any(|n| n.borrow().id == new_node_id) {
                            group.add_node(new_node);
                        }
                        ctx.register_group(group);
                    } else {
                        let mut new_group = OptGroup::new(new_node_id);
                        new_group.add_node(new_node);
                        ctx.register_group(new_group);
                    }
                }

                for &new_dep in &result.new_dependencies {
                    if !node_dependencies.contains(&new_dep) {
                        let mut node_mut = node.borrow_mut();
                        node_mut.dependencies.push(new_dep);
                    }
                }

                ctx.set_changed(true);
            }
            Ok(None) => {
                let mut node_mut = node.borrow_mut();
                node_mut.explored_rules.insert(rule.name().to_string(), true);
            }
            Err(e) => return Err(e),
        }

        Ok(())
    }

    fn extract_execution_plan(
        &self,
        root_group: &OptGroup,
        ctx: &mut OptContext,
    ) -> Result<ExecutionPlan, OptimizerError> {
        let root_node = root_group
            .get_min_cost_group_node()
            .ok_or(OptimizerError::no_viable_plan())?;

        self.build_execution_plan_recursive(&root_node, ctx)
    }

    fn build_execution_plan_recursive(
        &self,
        opt_node: &Rc<RefCell<OptGroupNode>>,
        ctx: &mut OptContext,
    ) -> Result<ExecutionPlan, OptimizerError> {
        let opt_node_borrowed = opt_node.borrow();

        let mut inputs: Vec<ExecutionPlan> = Vec::new();

        for &dep_id in &opt_node_borrowed.dependencies {
            if let Some(dep_group) = ctx.find_group_by_id(dep_id) {
                if let Some(dep_node) = dep_group.get_min_cost_group_node() {
                    let input_plan = self.build_execution_plan_recursive(&dep_node, ctx)?;
                    inputs.push(input_plan);
                }
            }
        }

        let plan_node = opt_node_borrowed.plan_node.clone();

        drop(opt_node_borrowed);

        let root_plan_node = plan_node;

        let plan = ExecutionPlan::new(Some(root_plan_node));

        Ok(plan)
    }

    pub fn add_rule_set(&mut self, rule_set: RuleSet) {
        self.rule_sets.push(rule_set);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimizer_creation() {
        let optimizer = Optimizer::default();
        assert!(!optimizer.rule_sets.is_empty());
    }

    #[test]
    fn test_rule_set_creation() {
        let rule_set = RuleSet::new("test");
        assert_eq!(rule_set.name, "test");
        assert!(rule_set.is_empty());
    }
}
