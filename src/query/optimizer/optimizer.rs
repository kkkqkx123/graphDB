//! Optimizer implementation for optimizing execution plans
use crate::core::context::QueryContext;
use crate::query::context::validate;
use crate::query::planner::plan::{ExecutionPlan, PlanNodeEnum};

use std::collections::{HashMap, HashSet, VecDeque};

// A simple object pool for reusing objects and reducing allocations
#[derive(Debug)]
pub struct ObjectPool<T> {
    objects: VecDeque<T>,
}

impl<T: Default> ObjectPool<T> {
    pub fn new() -> Self {
        Self {
            objects: VecDeque::new(),
        }
    }

    pub fn acquire(&mut self) -> T {
        self.objects.pop_front().unwrap_or_else(T::default)
    }

    pub fn release(&mut self, obj: T) {
        self.objects.push_back(obj);
    }

    pub fn size(&self) -> usize {
        self.objects.len()
    }
}

#[derive(Debug)]
pub struct OptContext {
    // Optimization context that holds state during optimization
    pub query_context: QueryContext,
    pub stats: OptimizationStats,
    pub changed: bool, // Whether this iteration caused a change to the plan
    pub visited_groups: HashSet<usize>, // Track visited groups during exploration
    pub plan_node_to_group_node: HashMap<usize, OptGroupNode>, // Map plan node IDs to optimization group nodes
    object_pool: ObjectPool<OptGroupNode>, // Pool for reusing OptGroupNode objects
}

impl OptContext {
    pub fn new(query_context: QueryContext) -> Self {
        Self {
            query_context,
            stats: OptimizationStats::default(),
            changed: true,
            visited_groups: HashSet::new(),
            plan_node_to_group_node: HashMap::new(),
            object_pool: ObjectPool::new(),
        }
    }

    pub fn set_changed(&mut self, changed: bool) {
        self.changed = changed;
    }

    pub fn add_plan_node_and_group_node(&mut self, plan_node_id: usize, group_node: &OptGroupNode) {
        self.plan_node_to_group_node
            .insert(plan_node_id, group_node.clone());
    }

    pub fn find_group_node_by_plan_node_id(&self, plan_node_id: usize) -> Option<&OptGroupNode> {
        self.plan_node_to_group_node.get(&plan_node_id)
    }

    pub fn get_group_node_from_pool(&mut self, id: usize, plan_node: PlanNodeEnum) -> OptGroupNode {
        let mut node = self.object_pool.acquire();
        node.id = id;
        node.plan_node = plan_node;
        node.group_id = 0; // Will be set when added to a group
                           // Reset other fields as needed
        node.dependencies.clear();
        node.explored_rules.clear();
        node.cost = 0.0;
        node.properties = PlanNodeProperties::default();
        node
    }

    pub fn return_group_node_to_pool(&mut self, mut node: OptGroupNode) {
        // Clear references to prevent holding onto memory unnecessarily
        node.dependencies.clear();
        node.explored_rules.clear();
        node.properties = PlanNodeProperties::default();
        self.object_pool.release(node);
    }

    pub fn validate_data_flow(&self, group_node: &OptGroupNode, boundary: &[&OptGroup]) -> bool {
        // Check if dependencies are within boundary
        let all_deps_in_boundary = group_node
            .dependencies
            .iter()
            .all(|&dep_id| boundary.iter().any(|&group| group.id == dep_id));

        if all_deps_in_boundary {
            return true;
        }

        // Check data flow between input variables and dependencies
        let input_vars_count = group_node.properties.input_vars.len();
        let deps_count = group_node.dependencies.len();

        if input_vars_count == deps_count {
            for (i, &_dep_id) in group_node.dependencies.iter().enumerate() {
                // In a complete implementation, we would check if the input var matches the output var of the dependency
                // For now, we just check the basic structure
                if i >= input_vars_count {
                    return false;
                }
            }
        }

        true
    }
}

#[derive(Debug, Default, Clone)]
pub struct OptimizationStats {
    pub rules_applied: usize,
    pub plan_nodes_before: usize,
    pub plan_nodes_after: usize,
    pub cost_before: f64,
    pub cost_after: f64,
}

// Represents a group of equivalent plan nodes during optimization
#[derive(Debug)]
pub struct OptGroup {
    pub id: usize,
    pub nodes: Vec<OptGroupNode>,
    pub logical: bool,               // Whether this is a logical or physical group
    pub explored_rules: Vec<String>, // Track which rules have been applied to this group
    pub root_group: bool,            // Whether this is the root optimization group
}

impl OptGroup {
    pub fn new(id: usize, logical: bool) -> Self {
        Self {
            id,
            nodes: Vec::new(),
            logical,
            explored_rules: Vec::new(),
            root_group: false,
        }
    }

    pub fn is_explored(&self, rule_name: &str) -> bool {
        self.explored_rules.contains(&rule_name.to_string())
    }

    pub fn set_explored(&mut self, rule: &dyn OptRule) {
        if !self.explored_rules.contains(&rule.name().to_string()) {
            self.explored_rules.push(rule.name().to_string());
        }
    }

    pub fn set_unexplored(&mut self, rule: &dyn OptRule) {
        let rule_name = rule.name();
        self.explored_rules.retain(|r| r != rule_name);
    }
}

// Represents an individual plan node in the optimization process
#[derive(Debug)]
pub struct OptGroupNode {
    pub id: usize,
    pub plan_node: PlanNodeEnum,
    pub dependencies: Vec<usize>, // IDs of dependency groups
    pub cost: f64,
    pub properties: PlanNodeProperties,
    pub explored_rules: Vec<String>, // Track which rules have been applied to this node
    pub group_id: usize,             // ID of the group this node belongs to
}

use crate::query::context::validate::types::Variable;

// A dummy plan node for default implementation
#[derive(Debug, Default)]
struct DummyPlanNode {
    id: i64,
    dependencies: Vec<PlanNodeEnum>,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
}

impl DummyPlanNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn type_name(&self) -> &'static str {
        "Dummy"
    }

    fn dependencies(&self) -> &[PlanNodeEnum] {
        &self.dependencies
    }

    fn output_var(&self) -> Option<&validate::types::Variable> {
        self.output_var.as_ref()
    }

    fn col_names(&self) -> &[String] {
        &self.col_names
    }

    fn cost(&self) -> f64 {
        self.cost
    }
}

impl Default for OptGroupNode {
    fn default() -> Self {
        Self {
            id: 0,
            plan_node: PlanNodeEnum::Start(
                crate::query::planner::plan::core::nodes::StartNode::new(),
            ),
            dependencies: Vec::new(),
            cost: 0.0,
            properties: PlanNodeProperties::default(),
            explored_rules: Vec::new(),
            group_id: 0,
        }
    }
}

impl OptGroupNode {
    pub fn new(id: usize, plan_node: PlanNodeEnum) -> Self {
        Self {
            id,
            plan_node,
            dependencies: Vec::new(),
            cost: 0.0,
            properties: PlanNodeProperties::default(),
            explored_rules: Vec::new(),
            group_id: 0, // Will be set when added to a group
        }
    }

    pub fn is_explored(&self, rule_name: &str) -> bool {
        self.explored_rules.contains(&rule_name.to_string())
    }

    pub fn set_explored(&mut self, rule: &dyn OptRule) {
        if !self.explored_rules.contains(&rule.name().to_string()) {
            self.explored_rules.push(rule.name().to_string());
        }
    }

    pub fn set_unexplored(&mut self, rule: &dyn OptRule) {
        let rule_name = rule.name();
        self.explored_rules.retain(|r| r != rule_name);
    }
}

impl Clone for OptGroupNode {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            plan_node: self.plan_node.clone(),
            dependencies: self.dependencies.clone(),
            cost: self.cost,
            properties: self.properties.clone(),
            explored_rules: self.explored_rules.clone(),
            group_id: self.group_id,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct PlanNodeProperties {
    // Properties that describe the plan node for optimization purposes
    pub output_vars: Vec<String>,
    pub input_vars: Vec<String>,
    pub estimated_rows: Option<u64>,
    pub has_side_effects: bool,
}

// Match result that represents a matched node and its dependencies
#[derive(Debug, Clone)]
pub struct MatchedResult {
    pub node: OptGroupNode,
    pub dependencies: Vec<MatchedResult>,
}

impl MatchedResult {
    pub fn plan_node(&self) -> &PlanNodeEnum {
        &self.node.plan_node
    }

    pub fn result(&self, pos: &[usize]) -> &MatchedResult {
        if pos.is_empty() {
            return self;
        }

        assert_eq!(pos[0], 0);

        let mut result = self;
        for &i in pos.iter().skip(1) {
            result = &result.dependencies[i];
        }
        result
    }
}

// Match node by type name or set of type names of plan node
#[derive(Debug, Clone)]
pub enum MatchNode {
    Single(&'static str),
    Multi(Vec<&'static str>),
}

impl MatchNode {
    pub fn matches(&self, node: &PlanNodeEnum) -> bool {
        match self {
            MatchNode::Single(type_name) => node.type_name() == *type_name,
            MatchNode::Multi(type_names) => type_names.iter().any(|t| node.type_name() == *t),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Pattern {
    pub node: MatchNode,
    pub dependencies: Vec<Pattern>,
}

impl Pattern {
    pub fn new(type_name: &'static str) -> Self {
        Self {
            node: MatchNode::Single(type_name),
            dependencies: Vec::new(),
        }
    }

    pub fn multi(type_names: Vec<&'static str>) -> Self {
        Self {
            node: MatchNode::Multi(type_names),
            dependencies: Vec::new(),
        }
    }

    pub fn with_dependency(mut self, dependency: Pattern) -> Self {
        self.dependencies.push(dependency);
        self
    }

    pub fn matches(&self, node: &OptGroupNode) -> bool {
        if !self.node.matches(&node.plan_node) {
            return false;
        }

        if self.dependencies.is_empty() {
            return true;
        }

        if node.dependencies.len() != self.dependencies.len() {
            return false;
        }

        // We don't match dependencies here, we match them via the matching algorithm
        true
    }
}

// Result of a rule transformation
#[derive(Debug)]
pub struct TransformResult {
    pub erase_curr: bool,
    pub erase_all: bool,
    pub new_group_nodes: Vec<OptGroupNode>,
}

impl TransformResult {
    pub fn no_transform() -> Self {
        Self {
            erase_curr: false,
            erase_all: false,
            new_group_nodes: Vec::new(),
        }
    }
}

// Base trait for optimization rules
pub trait OptRule: std::fmt::Debug {
    fn name(&self) -> &str;
    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError>;

    // Match method with detailed matching
    fn match_pattern(
        &self,
        ctx: &mut OptContext,
        group_node: &OptGroupNode,
    ) -> Result<Option<MatchedResult>, OptimizerError> {
        let pattern = self.pattern();
        self.match_pattern_with_result(ctx, group_node, &pattern)
    }

    fn match_pattern_with_result(
        &self,
        _ctx: &mut OptContext,
        group_node: &OptGroupNode,
        pattern: &Pattern,
    ) -> Result<Option<MatchedResult>, OptimizerError> {
        if !pattern.matches(group_node) {
            return Ok(None);
        }

        if pattern.dependencies.is_empty() {
            return Ok(Some(MatchedResult {
                node: group_node.clone(),
                dependencies: Vec::new(),
            }));
        }

        if group_node.dependencies.len() != pattern.dependencies.len() {
            return Ok(None);
        }

        let mut dependencies = Vec::new();
        for (_i, _dep_pattern) in pattern.dependencies.iter().enumerate() {
            // In a complete implementation, we would look up the actual dependency OptGroupNode
            // For now, this is a simplified version that doesn't implement full dependency matching
            // This would need a more complex structure to properly match dependencies
            dependencies.push(MatchedResult {
                node: OptGroupNode::new(0, group_node.plan_node.clone()), // Placeholder
                dependencies: Vec::new(),
            });
        }

        Ok(Some(MatchedResult {
            node: group_node.clone(),
            dependencies,
        }))
    }

    fn pattern(&self) -> Pattern;
}

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
}

impl Optimizer {
    pub fn new(rule_sets: Vec<RuleSet>) -> Self {
        Self { rule_sets }
    }

    // Create a default optimizer with commonly used rule sets
    pub fn default() -> Self {
        let mut logical_rules = RuleSet::new("logical");
        // 谓词下推规则
        logical_rules.add_rule(Box::new(super::FilterPushDownRule));
        logical_rules.add_rule(Box::new(super::PredicatePushDownRule));
        logical_rules.add_rule(Box::new(super::PushFilterDownTraverseRule));
        logical_rules.add_rule(Box::new(super::PushFilterDownExpandRule));
        logical_rules.add_rule(Box::new(super::PushFilterDownInnerJoinRule));
        logical_rules.add_rule(Box::new(super::PushFilterDownHashInnerJoinRule));
        logical_rules.add_rule(Box::new(super::PushFilterDownHashLeftJoinRule));

        // 投影下推规则
        logical_rules.add_rule(Box::new(super::ProjectionPushDownRule));
        logical_rules.add_rule(Box::new(super::PushProjectDownRule));

        // 操作合并规则
        logical_rules.add_rule(Box::new(super::CombineFilterRule));
        logical_rules.add_rule(Box::new(super::CollapseProjectRule));
        logical_rules.add_rule(Box::new(super::MergeGetVerticesAndProjectRule));
        logical_rules.add_rule(Box::new(super::MergeGetVerticesAndDedupRule));
        logical_rules.add_rule(Box::new(super::MergeGetNbrsAndDedupRule));
        logical_rules.add_rule(Box::new(super::MergeGetNbrsAndProjectRule));

        // 消除规则
        logical_rules.add_rule(Box::new(super::DedupEliminationRule));
        logical_rules.add_rule(Box::new(super::EliminateFilterRule));
        logical_rules.add_rule(Box::new(super::RemoveNoopProjectRule));
        logical_rules.add_rule(Box::new(super::EliminateAppendVerticesRule));
        logical_rules.add_rule(Box::new(super::RemoveAppendVerticesBelowJoinRule));

        // 转换规则
        logical_rules.add_rule(Box::new(super::TopNRule));

        let mut physical_rules = RuleSet::new("physical");
        // 连接优化规则
        physical_rules.add_rule(Box::new(super::JoinOptimizationRule));

        // LIMIT下推规则
        physical_rules.add_rule(Box::new(super::PushLimitDownRule));
        physical_rules.add_rule(Box::new(super::PushLimitDownGetVerticesRule));
        physical_rules.add_rule(Box::new(super::PushLimitDownGetNeighborsRule));
        physical_rules.add_rule(Box::new(super::PushLimitDownGetEdgesRule));
        physical_rules.add_rule(Box::new(super::PushLimitDownScanVerticesRule));
        physical_rules.add_rule(Box::new(super::PushLimitDownScanEdgesRule));
        physical_rules.add_rule(Box::new(super::PushLimitDownIndexScanRule));
        physical_rules.add_rule(Box::new(super::PushLimitDownProjectRule));
        // 注释掉使用不存在的 AllPaths 和 ExpandAll 类型的规则
        // physical_rules.add_rule(Box::new(super::PushLimitDownAllPathsRule));
        // physical_rules.add_rule(Box::new(super::PushLimitDownExpandAllRule));

        // 扫描优化规则
        physical_rules.add_rule(Box::new(super::ScanWithFilterOptimizationRule));
        physical_rules.add_rule(Box::new(super::IndexFullScanRule));

        // 索引优化规则
        physical_rules.add_rule(Box::new(super::IndexScanRule));
        physical_rules.add_rule(Box::new(super::EdgeIndexFullScanRule));
        physical_rules.add_rule(Box::new(super::TagIndexFullScanRule));
        physical_rules.add_rule(Box::new(super::UnionAllEdgeIndexScanRule));
        physical_rules.add_rule(Box::new(super::UnionAllTagIndexScanRule));
        physical_rules.add_rule(Box::new(super::OptimizeEdgeIndexScanByFilterRule));
        physical_rules.add_rule(Box::new(super::OptimizeTagIndexScanByFilterRule));

        Self::new(vec![logical_rules, physical_rules])
    }

    pub fn find_best_plan(
        &mut self,
        qctx: &mut QueryContext,
        plan: ExecutionPlan,
    ) -> Result<ExecutionPlan, OptimizerError> {
        // Create an optimization context
        let mut opt_ctx = OptContext::new(qctx.clone());

        // Convert the execution plan to an optimization graph
        let mut root_group = self.plan_to_group(&plan)?;
        root_group.root_group = true;

        // Iterative optimization with multiple rounds
        const MAX_ITERATION_ROUNDS: usize = 5;
        let mut round = 0;

        while opt_ctx.changed && round < MAX_ITERATION_ROUNDS {
            opt_ctx.changed = false;
            opt_ctx.visited_groups.clear();

            for rule_set in &self.rule_sets {
                for rule in &rule_set.rules {
                    // Apply the rule to the group
                    self.apply_rule(&mut opt_ctx, &mut root_group, rule.as_ref())?;
                }
            }

            round += 1;
        }

        // Perform post-processing validation
        self.post_process(&mut opt_ctx, &mut root_group)?;

        // Convert the optimized group back to an execution plan
        let optimized_plan = self.group_to_plan(&root_group)?;

        Ok(optimized_plan)
    }

    fn post_process(
        &self,
        ctx: &mut OptContext,
        root_group: &mut OptGroup,
    ) -> Result<(), OptimizerError> {
        // In a complete implementation, we would perform post-processing operations
        // such as property pruning, plan validation, etc.

        // Validate data flow in the optimized plan
        for node in &root_group.nodes {
            // Simple validation that all dependencies exist
            for &dep_id in &node.dependencies {
                if !root_group.nodes.iter().any(|n| n.id == dep_id) {
                    return Err(OptimizerError::OptimizationFailed(format!(
                        "Invalid dependency: node {} depends on non-existent node {}",
                        node.id, dep_id
                    )));
                }
            }

            // Validate data flow for this node against the root group as boundary
            let boundary = vec![&*root_group];
            if !ctx.validate_data_flow(node, &boundary) {
                return Err(OptimizerError::OptimizationFailed(format!(
                    "Data flow validation failed for node {}",
                    node.id
                )));
            }
        }

        Ok(())
    }

    fn plan_to_group(&self, plan: &ExecutionPlan) -> Result<OptGroup, OptimizerError> {
        // Convert an execution plan to an optimization group structure
        if let Some(root_node) = &plan.root {
            let mut group = OptGroup::new(0, false); // Physical group for execution plan
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
        // Create an OptGroupNode from the PlanNode
        let opt_node = OptGroupNode::new(node_id, node.clone());
        group.nodes.push(opt_node);

        // Process dependencies
        for (i, dep) in node.dependencies().iter().enumerate() {
            // In a complete implementation, we would recursively process the dependencies
            // For now, we just call this function recursively
            self.convert_node_to_group(dep, group, node_id + i + 1)?;
        }

        Ok(())
    }

    fn group_to_plan(&self, group: &OptGroup) -> Result<ExecutionPlan, OptimizerError> {
        // Convert an optimization group back to an execution plan
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
        // Check if this rule has already been applied to this group
        if group.is_explored(rule.name()) {
            return Ok(());
        }

        // Apply the rule using the exploration algorithm
        self.explore_rule(ctx, group, rule)?;

        // Mark the rule as explored for this group
        group.set_explored(rule);

        Ok(())
    }

    fn explore_rule(
        &self,
        ctx: &mut OptContext,
        group: &mut OptGroup,
        rule: &dyn OptRule,
    ) -> Result<(), OptimizerError> {
        // Explore the rule with multiple rounds up to maximum rounds
        const MAX_EXPLORATION_ROUNDS: usize = 128;
        let mut round = 0;

        while round < MAX_EXPLORATION_ROUNDS {
            let mut changed = false;

            // Process each node in the group
            for node_idx in 0..group.nodes.len() {
                // Skip if this node has already been explored with this rule
                if group.nodes[node_idx].is_explored(rule.name()) {
                    continue;
                }

                // Try to match the rule pattern on this node
                if let Ok(Some(_matched)) = rule.match_pattern(ctx, &group.nodes[node_idx]) {
                    // Apply the transformation
                    if let Some(new_node) = rule.apply(ctx, &group.nodes[node_idx])? {
                        // Add the new node to the group if not already present
                        if !self.node_exists_in_group(&new_node, group) {
                            group.nodes.push(new_node);
                            changed = true;
                            ctx.stats.rules_applied += 1;

                            // Mark the new node as explored to prevent immediate reprocessing
                            if let Some(node) = group.nodes.last_mut() {
                                node.set_explored(rule);
                            }
                        }
                    }
                }

                // Mark this node as explored with this rule
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
}
