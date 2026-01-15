//! Optimizer implementation for optimizing execution plans
use crate::core::context::QueryContext;
use crate::core::types::operators::Operator;
use crate::query::context::validate;
use crate::query::optimizer::property_tracker::PropertyTracker;
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

    /// 获取节点的边别名
    ///
    /// # 参数
    /// * `node_id` - 节点ID
    ///
    /// # 返回值
    /// 如果找到边别名，返回 Some(别名)，否则返回 None
    pub fn get_edge_alias_for_node(&self, node_id: usize) -> Option<String> {
        // 查找对应的 OptGroupNode
        if let Some(group_node) = self.find_group_node_by_plan_node_id(node_id) {
            // 根据节点类型获取边别名
            match group_node.plan_node.name() {
                "Traverse" => {
                    // 对于 Traverse 节点，从列名中提取边别名
                    let col_names = group_node.plan_node.col_names();
                    // 假设边别名是第一个列名
                    if !col_names.is_empty() {
                        return Some(col_names[0].clone());
                    }
                }
                "Expand" => {
                    // 对于 Expand 节点，从列名中提取边别名
                    let col_names = group_node.plan_node.col_names();
                    // 假设边别名是第一个列名
                    if !col_names.is_empty() {
                        return Some(col_names[0].clone());
                    }
                }
                _ => {}
            }
        }

        None
    }

    /// 获取节点的标签别名
    ///
    /// # 参数
    /// * `node_id` - 节点ID
    ///
    /// # 返回值
    /// 如果找到标签别名，返回 Some(别名)，否则返回 None
    pub fn get_tag_alias_for_node(&self, node_id: usize) -> Option<String> {
        // 查找对应的 OptGroupNode
        if let Some(group_node) = self.find_group_node_by_plan_node_id(node_id) {
            // 根据节点类型获取标签别名
            match group_node.plan_node.name() {
                "ScanVertices" => {
                    // 对于 ScanVertices 节点，从列名中提取标签别名
                    let col_names = group_node.plan_node.col_names();
                    // 假设标签别名是第一个列名
                    if !col_names.is_empty() {
                        return Some(col_names[0].clone());
                    }
                }
                "IndexScan" => {
                    // 对于 IndexScan 节点，从列名中提取标签别名
                    let col_names = group_node.plan_node.col_names();
                    // 假设标签别名是第一个列名
                    if !col_names.is_empty() {
                        return Some(col_names[0].clone());
                    }
                }
                _ => {}
            }
        }

        None
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
    pub output_var: Option<String>,  // The output variable should be same across the whole group
    pub bodies: Vec<OptGroup>,      // For control flow nodes (Select, Loop)
    pub group_nodes_referenced: HashSet<usize>, // Save the OptGroupNode which references this OptGroup
}

impl OptGroup {
    pub fn new(id: usize, logical: bool) -> Self {
        Self {
            id,
            nodes: Vec::new(),
            logical,
            explored_rules: Vec::new(),
            root_group: false,
            output_var: None,
            bodies: Vec::new(),
            group_nodes_referenced: HashSet::new(),
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

    /// Add a reference to this group from a group node
    pub fn add_ref_group_node(&mut self, node_id: usize) {
        self.group_nodes_referenced.insert(node_id);
    }

    /// Delete a reference to this group from a group node
    pub fn delete_ref_group_node(&mut self, node_id: usize) {
        self.group_nodes_referenced.remove(&node_id);
    }

    /// Get the minimum cost group node
    pub fn get_min_cost_group_node(&self) -> Option<&OptGroupNode> {
        self.nodes.iter().min_by(|a, b| {
            a.cost
                .partial_cmp(&b.cost)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// Validate the group
    pub fn validate(&self, _rule: &dyn OptRule) -> Result<(), OptimizerError> {
        // Validate data flow
        for node in &self.nodes {
            self.validate_data_flow(node)?;
        }

        Ok(())
    }

    /// Validate data flow for a node
    fn validate_data_flow(&self, node: &OptGroupNode) -> Result<(), OptimizerError> {
        // Check if dependencies are within boundary
        for &dep_id in &node.dependencies {
            if !self.nodes.iter().any(|n| n.id == dep_id) {
                return Err(OptimizerError::OptimizationFailed(format!(
                    "Invalid dependency: node {} depends on non-existent node {}",
                    node.id, dep_id
                )));
            }
        }

        Ok(())
    }
}

// Represents an individual plan node in the optimization process
#[derive(Debug)]
pub struct OptGroupNode {
    pub id: usize,
    pub plan_node: PlanNodeEnum,
    pub dependencies: Vec<usize>, // IDs of dependency groups
    pub bodies: Vec<usize>,         // IDs of body groups (for control flow nodes)
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
            bodies: Vec::new(),
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
            bodies: Vec::new(),
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

    /// Add a body group (for control flow nodes)
    pub fn add_body(&mut self, body_id: usize) {
        self.bodies.push(body_id);
    }

    /// Get the cost of this node
    pub fn get_cost(&self) -> f64 {
        self.cost
    }

    /// Release the opt group node from its opt group
    pub fn release(&mut self) {
        self.dependencies.clear();
        self.bodies.clear();
        self.explored_rules.clear();
    }

    /// Validate the node
    pub fn validate(&self, _rule: &dyn OptRule) -> Result<(), OptimizerError> {
        // Validate dependencies
        for &dep_id in &self.dependencies {
            if dep_id == 0 {
                return Err(OptimizerError::OptimizationFailed(format!(
                    "Invalid dependency: node {} has zero dependency ID",
                    self.id
                )));
            }
        }

        // Validate bodies
        for &body_id in &self.bodies {
            if body_id == 0 {
                return Err(OptimizerError::OptimizationFailed(format!(
                    "Invalid body: node {} has zero body ID",
                    self.id
                )));
            }
        }

        Ok(())
    }
}

impl Clone for OptGroupNode {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            plan_node: self.plan_node.clone(),
            dependencies: self.dependencies.clone(),
            bodies: self.bodies.clone(),
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

    /// Check data flow of the result
    ///
    /// # 参数
    /// * `boundary` - 边界组列表
    ///
    /// # 返回值
    /// 如果数据流有效，返回 true
    pub fn check_data_flow(&self, boundary: &[&OptGroup]) -> bool {
        for node in &self.new_group_nodes {
            // Check if all dependencies are within boundary
            for &dep_id in &node.dependencies {
                if !boundary.iter().any(|&group| group.id == dep_id) {
                    return false;
                }
            }

            // Check if all bodies are within boundary
            for &body_id in &node.bodies {
                if !boundary.iter().any(|&group| group.id == body_id) {
                    return false;
                }
            }
        }

        true
    }

    /// Check data flow for a single group node
    ///
    /// # 参数
    /// * `group_node` - 要检查的组节点
    /// * `boundary` - 边界组列表
    ///
    /// # 返回值
    /// 如果数据流有效，返回 true
    pub fn check_data_flow_for_node(group_node: &OptGroupNode, boundary: &[&OptGroup]) -> bool {
        // Check if all dependencies are within boundary
        for &dep_id in &group_node.dependencies {
            if !boundary.iter().any(|&group| group.id == dep_id) {
                return false;
            }
        }

        // Check if all bodies are within boundary
        for &body_id in &group_node.bodies {
            if !boundary.iter().any(|&group| group.id == body_id) {
                return false;
            }
        }

        true
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
        // 执行属性剪枝
        self.prune_properties(ctx, root_group)?;

        // 执行参数重写
        self.rewrite_arguments(ctx, root_group)?;

        // 验证优化后的计划
        self::super::PlanValidator::validate_plan(ctx, root_group)?;

        // 验证数据流在优化后的计划中
        for node in &root_group.nodes {
            // 简单验证所有依赖都存在
            for &dep_id in &node.dependencies {
                if !root_group.nodes.iter().any(|n| n.id == dep_id) {
                    return Err(OptimizerError::OptimizationFailed(format!(
                        "无效的依赖：节点 {} 依赖于不存在的节点 {}",
                        node.id, dep_id
                    )));
                }
            }

            // 验证此节点的数据流，以根组为边界
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

    /// 属性剪枝
    ///
    /// 移除计划中不需要的属性，减少数据传输量
    fn prune_properties(
        &self,
        ctx: &mut OptContext,
        root_group: &mut OptGroup,
    ) -> Result<(), OptimizerError> {
        // 创建属性跟踪器
        let mut property_tracker = PropertyTracker::new();

        // 收集所有需要的属性
        self.collect_required_properties(ctx, root_group, &mut property_tracker)?;

        // 应用属性剪枝
        self.apply_property_pruning(ctx, root_group, &property_tracker)?;

        Ok(())
    }

    /// 收集所有需要的属性
    fn collect_required_properties(
        &self,
        ctx: &OptContext,
        group: &OptGroup,
        property_tracker: &mut PropertyTracker,
    ) -> Result<(), OptimizerError> {
        for node in &group.nodes {
            // 收集节点所需的属性
            self.collect_node_properties(ctx, node, property_tracker)?;

            // 递归收集依赖组的属性
            for dep_id in &node.dependencies {
                if let Some(dep_group) = self.find_group_by_id(ctx, *dep_id) {
                    self.collect_required_properties(ctx, dep_group, property_tracker)?;
                }
            }

            // 递归收集主体组的属性
            for body_id in &node.bodies {
                if let Some(body_group) = self.find_group_by_id(ctx, *body_id) {
                    self.collect_required_properties(ctx, body_group, property_tracker)?;
                }
            }
        }

        Ok(())
    }

    /// 收集节点所需的属性
    fn collect_node_properties(
        &self,
        _ctx: &OptContext,
        node: &OptGroupNode,
        property_tracker: &mut PropertyTracker,
    ) -> Result<(), OptimizerError> {
        // 根据节点类型收集属性
        match node.plan_node.name() {
            "Project" => {
                // 收集投影节点中的所有属性
                if let Some(project_node) = node.plan_node.as_project() {
                    for column in project_node.columns() {
                        self.collect_expression_properties(&column.expr, property_tracker);
                    }
                }
            }
            "Filter" => {
                // 收集过滤节点中的所有属性
                if let Some(filter_node) = node.plan_node.as_filter() {
                    self.collect_expression_properties(&filter_node.condition(), property_tracker);
                }
            }
            "Aggregate" => {
                // 收集聚合节点中的所有属性
                if let Some(aggregate_node) = node.plan_node.as_aggregate() {
                    for group_key in aggregate_node.group_keys() {
                        self.collect_expression_properties(&crate::core::Expression::Variable(group_key.clone()), property_tracker);
                    }
                    for item in aggregate_node.aggregation_functions() {
                        self.collect_expression_properties(&crate::core::Expression::Variable(item.name().to_string()), property_tracker);
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// 收集表达式中的属性
    fn collect_expression_properties(
        &self,
        expr: &crate::core::Expression,
        property_tracker: &mut PropertyTracker,
    ) {
        use crate::core::Expression;

        match expr {
            Expression::Property { object, property } => {
                if let Expression::Variable(var_name) = object.as_ref() {
                    property_tracker.track_property(var_name, property);
                }
            }
            Expression::TagProperty { tag, prop } => {
                property_tracker.track_property(tag, prop);
            }
            Expression::EdgeProperty { edge, prop } => {
                property_tracker.track_property(edge, prop);
            }
            Expression::VariableProperty { var, prop } => {
                property_tracker.track_property(var, prop);
            }
            Expression::SourceProperty { tag, prop } => {
                property_tracker.track_property(tag, prop);
            }
            Expression::DestinationProperty { tag, prop } => {
                property_tracker.track_property(tag, prop);
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
                if let Some(default_expr) = default {
                    self.collect_expression_properties(default_expr, property_tracker);
                }
            }
            Expression::TypeCast { expr, .. } => {
                self.collect_expression_properties(expr, property_tracker);
            }
            Expression::Subscript { collection, index } => {
                self.collect_expression_properties(collection, property_tracker);
                self.collect_expression_properties(index, property_tracker);
            }
            Expression::Range { collection, start, end } => {
                self.collect_expression_properties(collection, property_tracker);
                if let Some(start_expr) = start {
                    self.collect_expression_properties(start_expr, property_tracker);
                }
                if let Some(end_expr) = end {
                    self.collect_expression_properties(end_expr, property_tracker);
                }
            }
            Expression::Path(items) => {
                for item in items {
                    self.collect_expression_properties(item, property_tracker);
                }
            }
            Expression::UnaryPlus(operand) => {
                self.collect_expression_properties(operand, property_tracker);
            }
            Expression::UnaryNegate(operand) => {
                self.collect_expression_properties(operand, property_tracker);
            }
            Expression::UnaryNot(operand) => {
                self.collect_expression_properties(operand, property_tracker);
            }
            Expression::UnaryIncr(operand) => {
                self.collect_expression_properties(operand, property_tracker);
            }
            Expression::UnaryDecr(operand) => {
                self.collect_expression_properties(operand, property_tracker);
            }
            Expression::IsNull(operand) => {
                self.collect_expression_properties(operand, property_tracker);
            }
            Expression::IsNotNull(operand) => {
                self.collect_expression_properties(operand, property_tracker);
            }
            Expression::IsEmpty(operand) => {
                self.collect_expression_properties(operand, property_tracker);
            }
            Expression::IsNotEmpty(operand) => {
                self.collect_expression_properties(operand, property_tracker);
            }
            Expression::ListComprehension { generator, condition } => {
                self.collect_expression_properties(generator, property_tracker);
                if let Some(condition_expr) = condition {
                    self.collect_expression_properties(condition_expr, property_tracker);
                }
            }
            Expression::Predicate { list, condition } => {
                self.collect_expression_properties(list, property_tracker);
                self.collect_expression_properties(condition, property_tracker);
            }
            Expression::Reduce { list, initial, expr, .. } => {
                self.collect_expression_properties(list, property_tracker);
                self.collect_expression_properties(initial, property_tracker);
                self.collect_expression_properties(expr, property_tracker);
            }
            _ => {}
        }
    }

    /// 应用属性剪枝
    fn apply_property_pruning(
        &self,
        ctx: &mut OptContext,
        group: &mut OptGroup,
        property_tracker: &PropertyTracker,
    ) -> Result<(), OptimizerError> {
        for node in &mut group.nodes {
            // 应用属性剪枝到节点
            self.apply_node_property_pruning(ctx, node, property_tracker)?;

            // 递归应用属性剪枝到依赖组
            for dep_id in &node.dependencies {
                if let Some(dep_group) = self.find_group_by_id_mut(ctx, *dep_id) {
                    self.apply_property_pruning(ctx, dep_group, property_tracker)?;
                }
            }

            // 递归应用属性剪枝到主体组
            for body_id in &node.bodies {
                if let Some(body_group) = self.find_group_by_id_mut(ctx, *body_id) {
                    self.apply_property_pruning(ctx, body_group, property_tracker)?;
                }
            }
        }

        Ok(())
    }

    /// 应用属性剪枝到节点
    fn apply_node_property_pruning(
        &self,
        _ctx: &mut OptContext,
        _node: &mut OptGroupNode,
        _property_tracker: &PropertyTracker,
    ) -> Result<(), OptimizerError> {
        // 根据节点类型应用属性剪枝
        // 简化实现：暂不实现属性剪枝
        Ok(())
    }

    /// 参数重写
    ///
    /// 重写计划中的参数，确保参数引用正确
    fn rewrite_arguments(
        &self,
        ctx: &mut OptContext,
        group: &mut OptGroup,
    ) -> Result<(), OptimizerError> {
        for node in &mut group.nodes {
            // 重写节点的参数
            self.rewrite_node_arguments(ctx, node)?;

            // 递归重写依赖组的参数
            for dep_id in &node.dependencies {
                if let Some(dep_group) = self.find_group_by_id_mut(ctx, *dep_id) {
                    self.rewrite_arguments(ctx, dep_group)?;
                }
            }

            // 递归重写主体组的参数
            for body_id in &node.bodies {
                if let Some(body_group) = self.find_group_by_id_mut(ctx, *body_id) {
                    self.rewrite_arguments(ctx, body_group)?;
                }
            }
        }

        Ok(())
    }

    /// 重写节点的参数
    fn rewrite_node_arguments(
        &self,
        _ctx: &mut OptContext,
        _node: &mut OptGroupNode,
    ) -> Result<(), OptimizerError> {
        // 在完整实现中，这里会重写节点的参数引用
        // 目前简化实现，不做任何操作
        Ok(())
    }

    /// 根据ID查找优化组
    fn find_group_by_id(&self, _ctx: &OptContext, _group_id: usize) -> Option<&OptGroup> {
        // 这里需要实现查找逻辑
        // 由于 OptContext 没有存储所有组的引用，这里返回 None
        // 在完整实现中，应该在 OptContext 中添加一个存储所有组的字段
        None
    }

    /// 根据ID查找优化组（可变引用）
    fn find_group_by_id_mut(&self, _ctx: &mut OptContext, _group_id: usize) -> Option<&mut OptGroup> {
        // 这里需要实现查找逻辑
        // 由于 OptContext 没有存储所有组的引用，这里返回 None
        // 在完整实现中，应该在 OptContext 中添加一个存储所有组的字段
        None
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

    #[error("Validation error: {message}")]
    Validation {
        message: String,
    },
}
