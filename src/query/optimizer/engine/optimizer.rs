//! 优化器引擎核心实现
//! 提供 Optimizer 结构体及其优化逻辑实现

use crate::query::context::execution::QueryContext;
use crate::query::optimizer::core::{
    Cost, OptimizationConfig, OptimizationPhase, OptimizationStats,
};
use crate::query::optimizer::plan::{
    OptContext, OptGroup, OptGroupNode, OptRule,
};
use crate::query::optimizer::property_tracker::PropertyTracker;
use crate::query::planner::plan::{ExecutionPlan, PlanNodeEnum};
use crate::query::optimizer::rule_traits::BaseOptRule;
use crate::query::optimizer::PlanValidator;

#[derive(Debug)]
pub struct RuleSet {
    pub name: String,
    pub rules: Vec<Box<dyn OptRule>>,
}

impl RuleSet {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            rules: Vec::new(),
        }
    }

    pub fn add_rule(&mut self, rule: Box<dyn OptRule>) {
        self.rules.push(rule);
    }

    pub fn rules(&self) -> &Vec<Box<dyn OptRule>> {
        &self.rules
    }
}

#[derive(Debug)]
pub struct Optimizer {
    rule_sets: Vec<RuleSet>,
    pub config: OptimizationConfig,
}

impl Optimizer {
    pub fn new(rule_sets: Vec<RuleSet>) -> Self {
        Self {
            rule_sets,
            config: OptimizationConfig::default(),
        }
    }

    pub fn with_config(rule_sets: Vec<RuleSet>, config: OptimizationConfig) -> Self {
        Self {
            rule_sets,
            config,
        }
    }

    pub fn default() -> Self {
        let mut logical_rules = RuleSet::new("logical");
        logical_rules.add_rule(Box::new(crate::query::optimizer::FilterPushDownRule));
        logical_rules.add_rule(Box::new(crate::query::optimizer::PredicatePushDownRule));
        logical_rules.add_rule(Box::new(crate::query::optimizer::PushFilterDownTraverseRule));
        logical_rules.add_rule(Box::new(crate::query::optimizer::PushFilterDownExpandRule));
        logical_rules.add_rule(Box::new(crate::query::optimizer::PushFilterDownInnerJoinRule));
        logical_rules.add_rule(Box::new(crate::query::optimizer::PushFilterDownHashInnerJoinRule));
        logical_rules.add_rule(Box::new(crate::query::optimizer::PushFilterDownHashLeftJoinRule));

        logical_rules.add_rule(Box::new(crate::query::optimizer::ProjectionPushDownRule));
        logical_rules.add_rule(Box::new(crate::query::optimizer::PushProjectDownRule));

        logical_rules.add_rule(Box::new(crate::query::optimizer::CombineFilterRule));
        logical_rules.add_rule(Box::new(crate::query::optimizer::CollapseProjectRule));
        logical_rules.add_rule(Box::new(crate::query::optimizer::MergeGetVerticesAndProjectRule));
        logical_rules.add_rule(Box::new(crate::query::optimizer::MergeGetVerticesAndDedupRule));
        logical_rules.add_rule(Box::new(crate::query::optimizer::MergeGetNbrsAndDedupRule));
        logical_rules.add_rule(Box::new(crate::query::optimizer::MergeGetNbrsAndProjectRule));

        logical_rules.add_rule(Box::new(crate::query::optimizer::DedupEliminationRule));
        logical_rules.add_rule(Box::new(crate::query::optimizer::EliminateFilterRule));
        logical_rules.add_rule(Box::new(crate::query::optimizer::RemoveNoopProjectRule));
        logical_rules.add_rule(Box::new(crate::query::optimizer::EliminateAppendVerticesRule));
        logical_rules.add_rule(Box::new(crate::query::optimizer::RemoveAppendVerticesBelowJoinRule));

        logical_rules.add_rule(Box::new(crate::query::optimizer::TopNRule));

        let mut physical_rules = RuleSet::new("physical");
        physical_rules.add_rule(Box::new(crate::query::optimizer::JoinOptimizationRule));

        physical_rules.add_rule(Box::new(crate::query::optimizer::PushLimitDownRule));
        physical_rules.add_rule(Box::new(crate::query::optimizer::PushLimitDownGetVerticesRule));
        physical_rules.add_rule(Box::new(crate::query::optimizer::PushLimitDownGetNeighborsRule));
        physical_rules.add_rule(Box::new(crate::query::optimizer::PushLimitDownGetEdgesRule));
        physical_rules.add_rule(Box::new(crate::query::optimizer::PushLimitDownScanVerticesRule));
        physical_rules.add_rule(Box::new(crate::query::optimizer::PushLimitDownScanEdgesRule));
        physical_rules.add_rule(Box::new(crate::query::optimizer::PushLimitDownIndexScanRule));
        physical_rules.add_rule(Box::new(crate::query::optimizer::PushLimitDownProjectRule));

        physical_rules.add_rule(Box::new(crate::query::optimizer::ScanWithFilterOptimizationRule));
        physical_rules.add_rule(Box::new(crate::query::optimizer::IndexFullScanRule));

        physical_rules.add_rule(Box::new(crate::query::optimizer::IndexScanRule));
        physical_rules.add_rule(Box::new(crate::query::optimizer::EdgeIndexFullScanRule));
        physical_rules.add_rule(Box::new(crate::query::optimizer::TagIndexFullScanRule));
        physical_rules.add_rule(Box::new(crate::query::optimizer::UnionAllEdgeIndexScanRule));
        physical_rules.add_rule(Box::new(crate::query::optimizer::UnionAllTagIndexScanRule));
        physical_rules.add_rule(Box::new(crate::query::optimizer::OptimizeEdgeIndexScanByFilterRule));
        physical_rules.add_rule(Box::new(crate::query::optimizer::OptimizeTagIndexScanByFilterRule));

        Self::new(vec![logical_rules, physical_rules])
    }

    pub fn find_best_plan(
        &mut self,
        qctx: &mut QueryContext,
        plan: ExecutionPlan,
    ) -> Result<ExecutionPlan, OptimizerError> {
        let mut opt_ctx = OptContext::new(qctx.clone());

        opt_ctx.stats.plan_nodes_before = self.count_nodes(&plan);

        let mut root_group = self.plan_to_group(&plan)?;
        root_group.root_group = true;

        self.execute_phase_optimization(&mut opt_ctx, &mut root_group, OptimizationPhase::LogicalOptimization)?;
        self.execute_phase_optimization(&mut opt_ctx, &mut root_group, OptimizationPhase::PhysicalOptimization)?;
        self.execute_phase_optimization(&mut opt_ctx, &mut root_group, OptimizationPhase::PostOptimization)?;

        self.post_process(&mut opt_ctx, &mut root_group)?;

        let optimized_plan = self.group_to_plan(&root_group)?;

        opt_ctx.stats.plan_nodes_after = self.count_nodes(&optimized_plan);

        if let Some(best_node) = root_group.get_min_cost_group_node() {
            opt_ctx.stats.finalize_phase(best_node.cost.total());
        }

        Ok(optimized_plan)
    }

    pub fn find_best_plan_with_stats(
        &mut self,
        qctx: &mut QueryContext,
        plan: ExecutionPlan,
    ) -> Result<(ExecutionPlan, OptimizationStats), OptimizerError> {
        let mut opt_ctx = OptContext::new(qctx.clone());
        opt_ctx.stats.plan_nodes_before = self.count_nodes(&plan);

        let mut root_group = self.plan_to_group(&plan)?;
        root_group.root_group = true;

        self.execute_phase_optimization(&mut opt_ctx, &mut root_group, OptimizationPhase::LogicalOptimization)?;
        self.execute_phase_optimization(&mut opt_ctx, &mut root_group, OptimizationPhase::PhysicalOptimization)?;
        self.execute_phase_optimization(&mut opt_ctx, &mut root_group, OptimizationPhase::PostOptimization)?;

        self.post_process(&mut opt_ctx, &mut root_group)?;

        let optimized_plan = self.group_to_plan(&root_group)?;
        opt_ctx.stats.plan_nodes_after = self.count_nodes(&optimized_plan);

        if let Some(best_node) = root_group.get_min_cost_group_node() {
            opt_ctx.stats.finalize_phase(best_node.cost.total());
        }

        Ok((optimized_plan, opt_ctx.stats))
    }

    fn execute_phase_optimization(
        &mut self,
        ctx: &mut OptContext,
        root_group: &mut OptGroup,
        phase: OptimizationPhase,
    ) -> Result<(), OptimizerError> {
        ctx.stats.start_phase(phase.clone());
        root_group.set_phase(phase.clone());

        let phase_rules = self.get_rules_for_phase(&phase);

        let max_rounds = self.config.max_iteration_rounds;
        let mut round = 0;

        while ctx.changed && round < max_rounds {
            ctx.changed = false;

            for rule in &phase_rules {
                self.apply_rule(ctx, root_group, *rule)?;
            }

            round += 1;
            ctx.stats.total_iterations += 1;
        }

        Ok(())
    }

    fn get_rules_for_phase(&self, phase: &OptimizationPhase) -> Vec<&dyn OptRule> {
        let mut rules = Vec::new();

        for rule_set in &self.rule_sets {
            for rule in &rule_set.rules {
                let rule_name = rule.name();
                let matches_phase = match phase {
                    OptimizationPhase::LogicalOptimization => {
                        matches!(rule_name,
                            "FilterPushDownRule" | "PredicatePushDownRule" | "PushFilterDownTraverseRule"
                            | "ProjectionPushDownRule" | "PushProjectDownRule" | "CombineFilterRule"
                            | "CollapseProjectRule" | "EliminateFilterRule" | "RemoveNoopProjectRule"
                            | "TopNRule")
                    }
                    OptimizationPhase::PhysicalOptimization => {
                        matches!(rule_name,
                            "JoinOptimizationRule" | "PushLimitDownRule" | "PushLimitDownGetVerticesRule"
                            | "PushLimitDownGetNeighborsRule" | "PushLimitDownGetEdgesRule"
                            | "ScanWithFilterOptimizationRule" | "IndexFullScanRule" | "IndexScanRule"
                            | "EdgeIndexFullScanRule" | "TagIndexFullScanRule")
                    }
                    OptimizationPhase::PostOptimization => {
                        matches!(rule_name, "TopNRule")
                    }
                };

                if matches_phase {
                    rules.push(rule.as_ref());
                }
            }
        }

        rules
    }

    fn post_process(
        &self,
        ctx: &mut OptContext,
        root_group: &mut OptGroup,
    ) -> Result<(), OptimizerError> {
        self.prune_properties(ctx, root_group)?;
        self.rewrite_arguments(ctx, root_group)?;
        PlanValidator::validate_plan(ctx, root_group)?;

        for node in &root_group.nodes {
            for &dep_id in &node.dependencies {
                if !root_group.nodes.iter().any(|n| n.id == dep_id) {
                    return Err(OptimizerError::OptimizationFailed(format!(
                        "无效的依赖：节点 {} 依赖于不存在的节点 {}",
                        node.id, dep_id
                    )));
                }
            }

            let boundary = vec![&*root_group];
            if !ctx.validate_data_flow(node, &boundary) {
                return Err(OptimizerError::OptimizationFailed(format!(
                    "节点 {} 的数据流验证失败",
                    node.id
                )));
            }
        }

        Ok(())
    }

    fn prune_properties(
        &self,
        ctx: &mut OptContext,
        root_group: &mut OptGroup,
    ) -> Result<(), OptimizerError> {
        let mut property_tracker = PropertyTracker::new();
        self.collect_required_properties(ctx, root_group, &mut property_tracker)?;
        self.apply_property_pruning(ctx, root_group, &property_tracker)?;
        Ok(())
    }

    fn collect_required_properties(
        &self,
        ctx: &OptContext,
        group: &OptGroup,
        property_tracker: &mut PropertyTracker,
    ) -> Result<(), OptimizerError> {
        for node in &group.nodes {
            self.collect_node_properties(ctx, node, property_tracker)?;

            for dep_id in &node.dependencies {
                if let Some(dep_group) = Self::find_group_by_id(ctx, *dep_id) {
                    self.collect_required_properties(ctx, dep_group, property_tracker)?;
                }
            }

            for body_id in &node.bodies {
                if let Some(body_group) = Self::find_group_by_id(ctx, *body_id) {
                    self.collect_required_properties(ctx, body_group, property_tracker)?;
                }
            }
        }

        Ok(())
    }

    fn collect_node_properties(
        &self,
        _ctx: &OptContext,
        node: &OptGroupNode,
        property_tracker: &mut PropertyTracker,
    ) -> Result<(), OptimizerError> {
        match node.plan_node.name() {
            "Project" => {
                if let Some(project_node) = node.plan_node.as_project() {
                    for column in project_node.columns() {
                        self.collect_expression_properties(&column.expression, property_tracker);
                    }
                }
            }
            "Filter" => {
                if let Some(filter_node) = node.plan_node.as_filter() {
                    self.collect_expression_properties(&filter_node.condition(), property_tracker);
                }
            }
            "Aggregate" => {
                if let Some(aggregate_node) = node.plan_node.as_aggregate() {
                    for group_key in aggregate_node.group_keys() {
                        self.collect_expression_properties(
                            &crate::core::Expression::Variable(group_key.clone()),
                            property_tracker,
                        );
                    }
                    for item in aggregate_node.aggregation_functions() {
                        self.collect_expression_properties(
                            &crate::core::Expression::Variable(item.name().to_string()),
                            property_tracker,
                        );
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn collect_expression_properties(
        &self,
        expression: &crate::core::Expression,
        property_tracker: &mut PropertyTracker,
    ) {
        use crate::core::Expression;

        match expression {
            Expression::Property { object, property } => {
                if let Expression::Variable(var_name) = object.as_ref() {
                    property_tracker.track_property(var_name, &property);
                }
            }
            Expression::Binary { left, right, .. } => {
                self.collect_expression_properties(left, property_tracker);
                self.collect_expression_properties(right, property_tracker);
            }
            Expression::Unary { operand, .. } => {
                self.collect_expression_properties(operand, property_tracker);
            }
            Expression::Function { args, .. } => {
                for arg in args {
                    self.collect_expression_properties(arg, property_tracker);
                }
            }
            Expression::Aggregate { arg, .. } => {
                self.collect_expression_properties(arg, property_tracker);
            }
            Expression::List(items) => {
                for item in items {
                    self.collect_expression_properties(item, property_tracker);
                }
            }
            Expression::Map(pairs) => {
                for (_, value) in pairs {
                    self.collect_expression_properties(value, property_tracker);
                }
            }
            Expression::Case { conditions, default } => {
                for (condition, value) in conditions {
                    self.collect_expression_properties(condition, property_tracker);
                    self.collect_expression_properties(value, property_tracker);
                }
                if let Some(default_expression) = default {
                    self.collect_expression_properties(default_expression, property_tracker);
                }
            }
            Expression::TypeCast { expression, .. } => {
                self.collect_expression_properties(expression, property_tracker);
            }
            Expression::Subscript { collection, index } => {
                self.collect_expression_properties(collection, property_tracker);
                self.collect_expression_properties(index, property_tracker);
            }
            Expression::Range { collection, start, end } => {
                self.collect_expression_properties(collection, property_tracker);
                if let Some(start_expression) = start {
                    self.collect_expression_properties(start_expression, property_tracker);
                }
                if let Some(end_expression) = end {
                    self.collect_expression_properties(end_expression, property_tracker);
                }
            }
            Expression::Path(items) => {
                for item in items {
                    self.collect_expression_properties(item, property_tracker);
                }
            }
            _ => {}
        }
    }

    fn apply_property_pruning(
        &self,
        _ctx: &mut OptContext,
        _group: &mut OptGroup,
        _property_tracker: &PropertyTracker,
    ) -> Result<(), OptimizerError> {
        Ok(())
    }

    fn rewrite_arguments(
        &self,
        ctx: &mut OptContext,
        group: &mut OptGroup,
    ) -> Result<(), OptimizerError> {
        let mut groups_to_process: Vec<usize> = Vec::new();

        for node in &mut group.nodes {
            self.rewrite_node_arguments(node)?;
            groups_to_process.extend(node.dependencies.iter());
            groups_to_process.extend(node.bodies.iter());
        }

        while let Some(group_id) = groups_to_process.pop() {
            if let Some(dep_group) = ctx.group_map.get_mut(&group_id) {
                let mut new_groups: Vec<usize> = Vec::new();

                for node in &mut dep_group.nodes {
                    self.rewrite_node_arguments(node)?;
                    new_groups.extend(node.dependencies.iter());
                    new_groups.extend(node.bodies.iter());
                }

                groups_to_process.extend(new_groups);
            }
        }

        Ok(())
    }

    fn rewrite_node_arguments(
        &self,
        _node: &mut OptGroupNode,
    ) -> Result<(), OptimizerError> {
        Ok(())
    }

    fn find_group_by_id(ctx: &OptContext, group_id: usize) -> Option<&OptGroup> {
        ctx.group_map.get(&group_id)
    }

    fn plan_to_group(&self, plan: &ExecutionPlan) -> Result<OptGroup, OptimizerError> {
        if let Some(root_node) = &plan.root {
            let mut group = OptGroup::new(0, false);
            self.convert_node_to_group(root_node, &mut group, 0)?;
            Ok(group)
        } else {
            Err(OptimizerError::PlanConversionError(
                "Cannot convert empty plan to group".to_string(),
            ))
        }
    }

    fn convert_node_to_group(
        &self,
        node: &PlanNodeEnum,
        group: &mut OptGroup,
        node_id: usize,
    ) -> Result<(), OptimizerError> {
        let opt_node = OptGroupNode::new(node_id, node.clone());
        group.nodes.push(opt_node);

        for (i, dep) in node.dependencies().iter().enumerate() {
            self.convert_node_to_group(dep, group, node_id + i + 1)?;
        }

        Ok(())
    }

    fn group_to_plan(&self, group: &OptGroup) -> Result<ExecutionPlan, OptimizerError> {
        if let Some(opt_node) = group.nodes.first() {
            let root = Some(opt_node.plan_node.clone());
            Ok(ExecutionPlan::new(root))
        } else {
            Err(OptimizerError::PlanConversionError(
                "Cannot convert empty group to plan".to_string(),
            ))
        }
    }

    fn apply_rule(
        &self,
        ctx: &mut OptContext,
        group: &mut OptGroup,
        rule: &dyn OptRule,
    ) -> Result<(), OptimizerError> {
        if group.is_explored(rule.name()) {
            return Ok(());
        }

        self.explore_rule(ctx, group, rule)?;
        group.set_explored(rule);

        Ok(())
    }

    fn explore_rule(
        &self,
        ctx: &mut OptContext,
        group: &mut OptGroup,
        rule: &dyn OptRule,
    ) -> Result<(), OptimizerError> {
        const MAX_EXPLORATION_ROUNDS: usize = 128;
        let mut round = 0;

        while round < MAX_EXPLORATION_ROUNDS {
            let mut changed = false;

            for node_idx in 0..group.nodes.len() {
                if group.nodes[node_idx].is_explored(rule.name()) {
                    continue;
                }

                if let Ok(Some(_matched)) = rule.match_pattern(ctx, &group.nodes[node_idx]) {
                    if let Some(new_node) = rule.apply(ctx, &group.nodes[node_idx])? {
                        if !self.node_exists_in_group(&new_node, group) {
                            group.nodes.push(new_node);
                            changed = true;
                            ctx.stats.rules_applied += 1;

                            if let Some(node) = group.nodes.last_mut() {
                                node.set_explored(rule);
                            }
                        }
                    }
                }

                group.nodes[node_idx].set_explored(rule);
            }

            if !changed {
                break;
            }

            round += 1;
        }

        Ok(())
    }

    fn node_exists_in_group(&self, node: &OptGroupNode, group: &OptGroup) -> bool {
        group.nodes.iter().any(|n| n.id == node.id)
    }

    fn count_nodes(&self, plan: &ExecutionPlan) -> usize {
        let mut count = 0;
        self.count_nodes_recursive(plan.root.as_ref(), &mut count);
        count
    }

    fn count_nodes_recursive(&self, node: Option<&PlanNodeEnum>, count: &mut usize) {
        if let Some(n) = node {
            *count += 1;
            for dep in n.dependencies() {
                self.count_nodes_recursive(Some(dep.as_ref()), count);
            }
        }
    }

    pub fn estimate_cost(&self, ctx: &OptContext, node: &OptGroupNode) -> Cost {
        let base_cost = match node.plan_node.name() {
            "ScanVertices" => Cost::new(10.0, 100.0, 50.0, 0.0),
            "ScanEdges" => Cost::new(10.0, 100.0, 50.0, 0.0),
            "IndexScan" => Cost::new(5.0, 20.0, 30.0, 0.0),
            "GetVertices" => Cost::new(5.0, 30.0, 20.0, 0.0),
            "GetEdges" => Cost::new(5.0, 30.0, 20.0, 0.0),
            "GetNeighbors" => Cost::new(20.0, 50.0, 40.0, 0.0),
            "Filter" => Cost::new(15.0, 0.0, 10.0, 0.0),
            "Project" => Cost::new(5.0, 0.0, 5.0, 0.0),
            "Aggregate" => Cost::new(30.0, 0.0, 25.0, 0.0),
            "Sort" => Cost::new(25.0, 0.0, 20.0, 0.0),
            "Limit" => Cost::new(2.0, 0.0, 5.0, 0.0),
            "InnerJoin" | "HashInnerJoin" => Cost::new(50.0, 10.0, 40.0, 0.0),
            "HashLeftJoin" => Cost::new(45.0, 10.0, 35.0, 0.0),
            "Traverse" => Cost::new(100.0, 20.0, 60.0, 0.0),
            "Expand" => Cost::new(80.0, 15.0, 50.0, 0.0),
            "Dedup" => Cost::new(10.0, 0.0, 15.0, 0.0),
            "Start" => Cost::new(1.0, 0.0, 1.0, 0.0),
            _ => Cost::new(10.0, 5.0, 10.0, 0.0),
        };

        let dependency_cost = node.dependencies.iter().fold(Cost::default(), |acc, dep_id| {
            if let Some(dep_node) = ctx.find_group_node_by_plan_node_id(*dep_id) {
                let dep_cost = self.estimate_cost(ctx, dep_node);
                Cost::new(
                    acc.cpu_cost + dep_cost.cpu_cost,
                    acc.io_cost + dep_cost.io_cost,
                    acc.memory_cost + dep_cost.memory_cost,
                    acc.network_cost + dep_cost.network_cost,
                )
            } else {
                acc
            }
        });

        Cost::new(
            base_cost.cpu_cost + dependency_cost.cpu_cost,
            base_cost.io_cost + dependency_cost.io_cost,
            base_cost.memory_cost + dependency_cost.memory_cost,
            base_cost.network_cost + dependency_cost.network_cost,
        )
    }
}

#[derive(Debug, thiserror::Error)]
pub enum OptimizerError {
    #[error("Plan conversion error: {0}")]
    PlanConversionError(String),

    #[error("Rule application error: {0}")]
    RuleApplicationError(String),

    #[error("Optimization failed: {0}")]
    OptimizationFailed(String),

    #[error("Invalid optimization context: {0}")]
    InvalidOptContext(String),

    #[error("Validation error: {message}")]
    Validation { message: String },
}
