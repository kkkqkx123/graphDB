//! 优化节点定义
//! 定义 OptGroupNode 结构体，表示优化过程中的执行计划节点
//!
//! OptGroupNode 是优化器中的核心数据结构之一：
//! - 封装一个执行计划节点（PlanNode）
//! - 管理节点的成本信息
//! - 追踪节点的依赖关系
//! - 支持规则探索和转换

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use crate::query::optimizer::plan::context::OptContext;
use crate::query::optimizer::core::{Cost, PlanNodeProperties};
use crate::query::optimizer::plan::group::OptGroup;
use crate::query::planner::plan::PlanNodeEnum;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PlanCandidate {
    pub node_id: usize,
    pub cost: Cost,
    pub property: PlanNodeProperties,
}

#[derive(Debug)]
pub struct OptGroupNode {
    pub id: usize,
    pub plan_node: PlanNodeEnum,
    pub cost: Cost,
    pub properties: PlanNodeProperties,
    pub dependencies: Vec<usize>,
    pub explored_rules: HashMap<String, bool>,
    pub bodies: Vec<usize>,
}

impl Default for OptGroupNode {
    fn default() -> Self {
        Self {
            id: 0,
            plan_node: PlanNodeEnum::default(),
            cost: Cost::default(),
            properties: PlanNodeProperties::default(),
            dependencies: Vec::new(),
            explored_rules: HashMap::new(),
            bodies: Vec::new(),
        }
    }
}

impl OptGroupNode {
    pub fn new(id: usize, plan_node: PlanNodeEnum) -> Self {
        Self {
            id,
            plan_node,
            cost: Cost::default(),
            properties: PlanNodeProperties::default(),
            dependencies: Vec::new(),
            explored_rules: HashMap::new(),
            bodies: Vec::new(),
        }
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn get_plan_node(&self) -> &PlanNodeEnum {
        &self.plan_node
    }

    pub fn get_plan_node_mut(&mut self) -> &mut PlanNodeEnum {
        &mut self.plan_node
    }

    pub fn get_cost(&self) -> Cost {
        self.cost.clone()
    }

    pub fn set_cost(&mut self, cost: Cost) {
        self.cost = cost;
    }

    pub fn get_properties(&self) -> &PlanNodeProperties {
        &self.properties
    }

    pub fn set_properties(&mut self, properties: PlanNodeProperties) {
        self.properties = properties;
    }

    pub fn get_dependencies(&self) -> &[usize] {
        &self.dependencies
    }

    pub fn add_dependency(&mut self, dep_id: usize) {
        if !self.dependencies.contains(&dep_id) {
            self.dependencies.push(dep_id);
        }
    }

    pub fn remove_dependency(&mut self, dep_id: usize) {
        self.dependencies.retain(|&id| id != dep_id);
    }

    pub fn has_explored_rule(&self, rule_name: &str) -> bool {
        self.explored_rules.contains_key(rule_name)
    }

    pub fn set_rule_explored(&mut self, rule_name: String, explored: bool) {
        self.explored_rules.insert(rule_name, explored);
    }

    pub fn add_body(&mut self, body_id: usize) {
        if !self.bodies.contains(&body_id) {
            self.bodies.push(body_id);
        }
    }

    pub fn get_bodies(&self) -> &[usize] {
        &self.bodies
    }
}

impl Clone for OptGroupNode {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            plan_node: self.plan_node.clone(),
            cost: self.cost.clone(),
            properties: self.properties.clone(),
            dependencies: self.dependencies.clone(),
            explored_rules: self.explored_rules.clone(),
            bodies: self.bodies.clone(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct TransformResult {
    pub erase_curr: bool,
    pub erase_all: bool,
    pub new_group_nodes: Vec<Rc<RefCell<OptGroupNode>>>,
    pub candidates: Vec<PlanCandidate>,
    pub new_dependencies: Vec<usize>,
}

impl TransformResult {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_erase_curr(mut self, erase_curr: bool) -> Self {
        self.erase_curr = erase_curr;
        self
    }

    pub fn with_erase_all(mut self, erase_all: bool) -> Self {
        self.erase_all = erase_all;
        self
    }

    pub fn add_new_group_node(&mut self, node: Rc<RefCell<OptGroupNode>>) {
        self.new_group_nodes.push(node);
    }

    pub fn add_candidate(&mut self, candidate: PlanCandidate) {
        self.candidates.push(candidate);
    }

    pub fn add_new_dependency(&mut self, dep_id: usize) {
        self.new_dependencies.push(dep_id);
    }

    pub fn with_replacement(mut self, node: Rc<RefCell<OptGroupNode>>) -> Self {
        self.erase_curr = true;
        self.new_group_nodes.push(node);
        self
    }

    pub fn with_erased(mut self) -> Self {
        self.erase_curr = true;
        self
    }

    pub fn unchanged() -> Self {
        Self::default()
    }
}

pub trait OptRule: fmt::Debug {
    fn name(&self) -> &str;
    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> Result<Option<TransformResult>, OptimizerError>;
    fn pattern(&self) -> Pattern;
    fn match_pattern(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> Result<Option<MatchedResult>, OptimizerError> {
        let node_ref = group_node.borrow();
        if self.pattern().matches(&node_ref.plan_node) {
            let mut result = MatchedResult::new();
            result.add_group_node(group_node.clone());
            for dep_id in &node_ref.dependencies {
                if let Some(dep_node) = ctx.find_group_node_by_id(*dep_id) {
                    result.dependencies.push(dep_node.clone());
                }
            }
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }
    fn transform(&self, _ctx: &mut OptContext, _group_node: &Rc<RefCell<OptGroupNode>>) -> Result<Option<TransformResult>, OptimizerError> {
        Ok(None)
    }
    fn is_rule_expired(&self, _ctx: &OptContext, _group_node: &Rc<RefCell<OptGroupNode>>) -> bool {
        false
    }
    fn get_match_plan(&self, _group_node: &Rc<RefCell<OptGroupNode>>) -> PlanNodeEnum {
        _group_node.borrow().plan_node.clone()
    }
    fn require(&self, _ctx: &OptContext, _group_node: &Rc<RefCell<OptGroupNode>>) -> bool {
        true
    }
}

#[derive(Debug)]
pub struct OptimizerError {
    pub message: String,
    pub code: i32,
}

impl OptimizerError {
    pub fn new(message: String, code: i32) -> Self {
        Self { message, code }
    }

    pub fn group_not_found(group_id: usize) -> Self {
        Self {
            message: format!("Group not found: {}", group_id),
            code: 1001,
        }
    }

    pub fn no_viable_plan() -> Self {
        Self {
            message: "No viable plan found".to_string(),
            code: 1002,
        }
    }

    pub fn rule_application_failed(rule_name: String, reason: String) -> Self {
        Self {
            message: format!("Rule application failed: {} - {}", rule_name, reason),
            code: 1003,
        }
    }

    pub fn cycle_detected(node_id: usize) -> Self {
        Self {
            message: format!("Cycle detected at node: {}", node_id),
            code: 1004,
        }
    }

    pub fn invalid_plan_structure(message: String) -> Self {
        Self {
            message: format!("Invalid plan structure: {}", message),
            code: 1005,
        }
    }

    pub fn unsupported_operation(operation: String) -> Self {
        Self {
            message: format!("Unsupported operation: {}", operation),
            code: 1006,
        }
    }

    pub fn validation(message: String) -> Self {
        Self {
            message,
            code: 2000,
        }
    }
}

impl fmt::Display for OptimizerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[Error {}] {}", self.code, self.message)
    }
}

impl std::error::Error for OptimizerError {}

pub type Result<T> = std::result::Result<T, OptimizerError>;

#[derive(Debug, Clone)]
pub struct Pattern {
    pub node: Option<MatchNode>,
    pub dependencies: Vec<Pattern>,
}

impl Pattern {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_node(node: MatchNode) -> Self {
        Self {
            node: Some(node),
            dependencies: Vec::new(),
        }
    }

    pub fn new_with_name(name: &'static str) -> Self {
        Self::with_node(MatchNode::Single(name))
    }

    pub fn multi(node_names: Vec<&'static str>) -> Self {
        Self::with_node(MatchNode::Multi(node_names))
    }

    pub fn with_dependency(mut self, dependency: Pattern) -> Self {
        self.dependencies.push(dependency);
        self
    }

    pub fn add_dependency(&mut self, dependency: Pattern) {
        self.dependencies.push(dependency);
    }

    pub fn matches(&self, plan_node: &PlanNodeEnum) -> bool {
        if let Some(ref node) = self.node {
            if !node.matches(plan_node.name()) {
                return false;
            }
        }

        if self.dependencies.is_empty() {
            return true;
        }

        let dep_names: Vec<&str> = self.dependencies
            .iter()
            .filter_map(|d| d.node.as_ref())
            .filter_map(|n| n.as_single())
            .collect();

        if dep_names.is_empty() {
            return true;
        }

        for dep_pattern in &self.dependencies {
            let dep_matches = plan_node.dependencies().iter().any(|input| {
                if let Some(ref node) = dep_pattern.node {
                    node.matches(input.name())
                } else {
                    true
                }
            });

            if !dep_matches && !dep_pattern.dependencies.is_empty() {
                return false;
            }
        }

        true
    }

    pub fn with_project_matcher() -> Self {
        Self::new_with_name("Project")
    }

    pub fn with_filter_matcher() -> Self {
        Self::new_with_name("Filter")
    }

    pub fn with_scan_vertices_matcher() -> Self {
        Self::new_with_name("ScanVertices")
    }

    pub fn with_get_vertices_matcher() -> Self {
        Self::new_with_name("GetVertices")
    }

    pub fn with_limit_matcher() -> Self {
        Self::new_with_name("Limit")
    }

    pub fn with_sort_matcher() -> Self {
        Self::new_with_name("Sort")
    }

    pub fn with_aggregate_matcher() -> Self {
        Self::new_with_name("Aggregate")
    }
}

impl Default for Pattern {
    fn default() -> Self {
        Self {
            node: None,
            dependencies: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum PlanNodeMatcher {
    MatchNode(&'static str),
    Not(Box<PlanNodeMatcher>),
    And(Vec<PlanNodeMatcher>),
    Or(Vec<PlanNodeMatcher>),
}

impl PlanNodeMatcher {
    pub fn matches(&self, plan_node: &PlanNodeEnum) -> bool {
        match self {
            PlanNodeMatcher::MatchNode(name) => plan_node.name() == *name,
            PlanNodeMatcher::Not(matcher) => !matcher.matches(plan_node),
            PlanNodeMatcher::And(matchers) => matchers.iter().all(|m| m.matches(plan_node)),
            PlanNodeMatcher::Or(matchers) => matchers.iter().any(|m| m.matches(plan_node)),
        }
    }

    pub fn and(self, other: PlanNodeMatcher) -> Self {
        PlanNodeMatcher::And(vec![self, other])
    }

    pub fn or(self, other: PlanNodeMatcher) -> Self {
        PlanNodeMatcher::Or(vec![self, other])
    }
}

pub trait PatternBuilder {
    fn build(&self) -> Pattern;
}

pub trait NodeVisitor {
    fn visit(&mut self, node: &PlanNodeEnum) -> bool;
}

#[derive(Debug, Default)]
pub struct NodeVisitorRecorder {
    pub nodes: Vec<PlanNodeEnum>,
}

impl NodeVisitorRecorder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(&mut self, node: &PlanNodeEnum) {
        self.nodes.push(node.clone());
    }
}

impl NodeVisitor for NodeVisitorRecorder {
    fn visit(&mut self, node: &PlanNodeEnum) -> bool {
        self.record(node);
        true
    }
}

#[derive(Debug)]
pub struct NodeVisitorFinder {
    pub target_name: String,
    pub found_node: Option<PlanNodeEnum>,
}

impl NodeVisitorFinder {
    pub fn new(target_name: &str) -> Self {
        Self {
            target_name: target_name.to_string(),
            found_node: None,
        }
    }
}

impl NodeVisitor for NodeVisitorFinder {
    fn visit(&mut self, node: &PlanNodeEnum) -> bool {
        if node.name() == self.target_name {
            self.found_node = Some(node.clone());
            return false;
        }
        true
    }
}

#[derive(Debug)]
pub struct Context {}

impl Context {
    pub fn new() -> Self {
        Context {}
    }
}

#[derive(Debug, Clone)]
pub enum MatchNode {
    Single(&'static str),
    Multi(Vec<&'static str>),
    Any,
}

impl MatchNode {
    pub fn matches(&self, node_name: &str) -> bool {
        match self {
            MatchNode::Single(name) => *name == node_name,
            MatchNode::Multi(names) => names.contains(&node_name),
            MatchNode::Any => true,
        }
    }

    pub fn as_single(&self) -> Option<&'static str> {
        match self {
            MatchNode::Single(name) => Some(name),
            _ => None,
        }
    }

    pub fn as_multi(&self) -> Option<&Vec<&'static str>> {
        match self {
            MatchNode::Multi(names) => Some(names),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MatchedResult {
    pub group_nodes: Vec<Rc<RefCell<OptGroupNode>>>,
    pub groups: Vec<OptGroup>,
    pub root_group: OptGroup,
    pub dependencies: Vec<Rc<RefCell<OptGroupNode>>>,
}

impl MatchedResult {
    pub fn new() -> Self {
        Self {
            group_nodes: Vec::new(),
            groups: Vec::new(),
            root_group: OptGroup::new(0),
            dependencies: Vec::new(),
        }
    }

    pub fn add_group_node(&mut self, node: Rc<RefCell<OptGroupNode>>) {
        self.group_nodes.push(node);
    }

    pub fn add_group(&mut self, group: OptGroup) {
        self.groups.push(group);
    }

    pub fn set_root_group(&mut self, group: OptGroup) {
        self.root_group = group;
    }
}

impl Default for MatchedResult {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opt_group_node_creation() {
        let plan_node = PlanNodeEnum::default();
        let group_node = OptGroupNode::new(1, plan_node);
        assert_eq!(group_node.id, 1);
    }

    #[test]
    fn test_opt_group_node_cost() {
        let plan_node = PlanNodeEnum::default();
        let mut group_node = OptGroupNode::new(1, plan_node);
        let cost = Cost::new(10.0, 100.0);
        group_node.set_cost(cost.clone());
        assert_eq!(group_node.get_cost(), cost);
    }

    #[test]
    fn test_opt_group_node_dependencies() {
        let plan_node = PlanNodeEnum::default();
        let mut group_node = OptGroupNode::new(1, plan_node);
        group_node.add_dependency(2);
        group_node.add_dependency(3);
        assert_eq!(group_node.get_dependencies().len(), 2);
        group_node.remove_dependency(2);
        assert_eq!(group_node.get_dependencies().len(), 1);
    }

    #[test]
    fn test_pattern_matches() {
        let mut pattern = Pattern::new();
        pattern.add_matcher(PlanNodeMatcher::MatchNode("Project"));
        let project_node = PlanNodeEnum::Project(
            crate::query::planner::plan::core::nodes::project::Project::default(),
        );
        let filter_node = PlanNodeEnum::Filter(
            crate::query::planner::plan::core::nodes::filter::Filter::default(),
        );
        assert!(pattern.matches(&project_node));
        assert!(!pattern.matches(&filter_node));
    }

    #[test]
    fn test_match_node_single() {
        let matcher = MatchNode::Single("Project");
        assert!(matcher.matches("Project"));
        assert!(!matcher.matches("Filter"));
    }

    #[test]
    fn test_match_node_multi() {
        let matcher = MatchNode::Multi(vec!["Project", "Filter"]);
        assert!(matcher.matches("Project"));
        assert!(matcher.matches("Filter"));
        assert!(!matcher.matches("ScanVertices"));
    }

    #[test]
    fn test_match_node_any() {
        let matcher = MatchNode::Any;
        assert!(matcher.matches("Project"));
        assert!(matcher.matches("Filter"));
        assert!(matcher.matches("ScanVertices"));
    }
}
