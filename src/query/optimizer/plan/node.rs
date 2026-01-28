//! 优化组节点定义
//! 定义 OptGroupNode、PlanCandidate 等核心数据结构

use std::fmt::Debug;

use crate::query::optimizer::core::Cost;
use crate::query::planner::plan::PlanNodeEnum;
use crate::utils::ObjectPool;

use super::context::OptContext;
use super::group::OptGroup;
use crate::query::optimizer::OptimizerError;

#[derive(Debug, Clone)]
pub struct PlanCandidate {
    pub node: OptGroupNode,
    pub cost: Cost,
    pub explanation: String,
}

impl PlanCandidate {
    pub fn new(node: OptGroupNode, cost: Cost, explanation: String) -> Self {
        Self {
            node,
            cost,
            explanation,
        }
    }
}

#[derive(Debug)]
pub struct TransformResult {
    pub erase_curr: bool,
    pub erase_all: bool,
    pub new_group_nodes: Vec<OptGroupNode>,
    pub candidates: Vec<PlanCandidate>,
}

impl TransformResult {
    pub fn no_transform() -> Self {
        Self {
            erase_curr: false,
            erase_all: false,
            new_group_nodes: Vec::new(),
            candidates: Vec::new(),
        }
    }

    pub fn with_candidate(mut self, candidate: PlanCandidate) -> Self {
        self.candidates.push(candidate);
        self
    }

    pub fn has_transform(&self) -> bool {
        self.erase_curr || self.erase_all || !self.new_group_nodes.is_empty() || !self.candidates.is_empty()
    }

    pub fn check_data_flow(&self, boundary: &[&OptGroup]) -> bool {
        for node in &self.new_group_nodes {
            for &dep_id in &node.dependencies {
                if !boundary.iter().any(|&group| group.id == dep_id) {
                    return false;
                }
            }
            for &body_id in &node.bodies {
                if !boundary.iter().any(|&group| group.id == body_id) {
                    return false;
                }
            }
        }
        true
    }

    pub fn check_data_flow_for_node(group_node: &OptGroupNode, boundary: &[&OptGroup]) -> bool {
        for &dep_id in &group_node.dependencies {
            if !boundary.iter().any(|&group| group.id == dep_id) {
                return false;
            }
        }
        for &body_id in &group_node.bodies {
            if !boundary.iter().any(|&group| group.id == body_id) {
                return false;
            }
        }
        true
    }
}

#[derive(Debug, Default, Clone)]
pub struct PlanNodeProperties {
    pub output_vars: Vec<String>,
    pub input_vars: Vec<String>,
    pub estimated_rows: Option<u64>,
    pub has_side_effects: bool,
}

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

        true
    }
}

pub type OptGroupNodeId = usize;

#[derive(Debug)]
pub struct OptGroupNode {
    pub id: usize,
    pub plan_node: PlanNodeEnum,
    pub dependencies: Vec<usize>,
    pub bodies: Vec<usize>,
    pub cost: Cost,
    pub properties: PlanNodeProperties,
    pub explored_rules: Vec<String>,
    pub group_id: usize,
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
            cost: Cost::default(),
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
            cost: Cost::default(),
            properties: PlanNodeProperties::default(),
            explored_rules: Vec::new(),
            group_id: 0,
        }
    }

    pub fn with_cost(id: usize, plan_node: PlanNodeEnum, cost: Cost) -> Self {
        Self {
            id,
            plan_node,
            dependencies: Vec::new(),
            bodies: Vec::new(),
            cost,
            properties: PlanNodeProperties::default(),
            explored_rules: Vec::new(),
            group_id: 0,
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

    pub fn add_body(&mut self, body_id: usize) {
        self.bodies.push(body_id);
    }

    pub fn get_cost(&self) -> f64 {
        self.cost.total()
    }

    pub fn release(&mut self) {
        self.dependencies.clear();
        self.bodies.clear();
        self.explored_rules.clear();
    }

    pub fn validate(&self, _rule: &dyn OptRule) -> Result<(), OptimizerError> {
        for &dep_id in &self.dependencies {
            if dep_id == 0 {
                return Err(OptimizerError::OptimizationFailed(format!(
                    "Invalid dependency: node {} has zero dependency ID",
                    self.id
                )));
            }
        }

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

pub trait OptRule: std::fmt::Debug {
    fn name(&self) -> &str;
    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError>;

    fn pattern(&self) -> Pattern;

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
        ctx: &mut OptContext,
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
        for (i, dep_id) in group_node.dependencies.iter().enumerate() {
            if let Some(dep_node) = ctx.find_group_node_by_plan_node_id(*dep_id) {
                let dep_node_clone = dep_node.clone();
                if let Some(matched_dep) = self.match_pattern_with_result(ctx, &dep_node_clone, &pattern.dependencies[i])? {
                    dependencies.push(matched_dep);
                } else {
                    return Ok(None);
                }
            } else {
                return Ok(None);
            }
        }

        Ok(Some(MatchedResult {
            node: group_node.clone(),
            dependencies,
        }))
    }
}
