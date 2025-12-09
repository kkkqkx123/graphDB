//! Optimizer implementation for optimizing execution plans
use crate::query::planner::plan::{PlanNode, PlanNodeKind, ExecutionPlan};
use crate::query::context::QueryContext;

#[derive(Debug, Clone)]
pub struct OptContext {
    // Optimization context that holds state during optimization
    pub query_context: QueryContext,
    pub stats: OptimizationStats,
}

impl OptContext {
    pub fn new(query_context: QueryContext) -> Self {
        Self {
            query_context,
            stats: OptimizationStats::default(),
        }
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
    pub logical: bool, // Whether this is a logical or physical group
}

impl OptGroup {
    pub fn new(id: usize, logical: bool) -> Self {
        Self {
            id,
            nodes: Vec::new(),
            logical,
        }
    }
}

// Represents an individual plan node in the optimization process
#[derive(Debug)]
pub struct OptGroupNode {
    pub id: usize,
    pub plan_node: Box<dyn PlanNode>,
    pub dependencies: Vec<usize>, // IDs of dependency groups
    pub cost: f64,
    pub properties: PlanNodeProperties,
}

impl OptGroupNode {
    pub fn new(id: usize, plan_node: Box<dyn PlanNode>) -> Self {
        Self {
            id,
            plan_node,
            dependencies: Vec::new(),
            cost: 0.0,
            properties: PlanNodeProperties::default(),
        }
    }
}

impl Clone for OptGroupNode {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            plan_node: self.plan_node.clone_plan_node(),
            dependencies: self.dependencies.clone(),
            cost: self.cost,
            properties: self.properties.clone(),
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

// Base trait for optimization rules
pub trait OptRule: std::fmt::Debug {
    fn name(&self) -> &str;
    fn apply(&self, ctx: &mut OptContext, node: &OptGroupNode) -> Result<Option<OptGroupNode>, OptimizerError>;
    fn pattern(&self) -> Box<Pattern>; // Define what plan nodes this rule matches
}

#[derive(Debug, Clone)]
pub struct Pattern {
    pub kind: PlanNodeKind,
    pub dependencies: Vec<Pattern>,
}

impl Pattern {
    pub fn new(kind: PlanNodeKind) -> Self {
        Self {
            kind,
            dependencies: Vec::new(),
        }
    }

    pub fn with_dependency(mut self, dependency: Pattern) -> Self {
        self.dependencies.push(dependency);
        self
    }
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
        logical_rules.add_rule(Box::new(super::rule::FilterPushDownRule));
        logical_rules.add_rule(Box::new(super::rule::DedupEliminationRule));
        logical_rules.add_rule(Box::new(super::rule::ProjectionPushDownRule));
        logical_rules.add_rule(Box::new(super::rule::PredicatePushDownRule));
        logical_rules.add_rule(Box::new(super::advanced_rules::CombineFilterRule));
        logical_rules.add_rule(Box::new(super::advanced_rules::EliminateFilterRule));
        logical_rules.add_rule(Box::new(super::advanced_rules::CollapseProjectRule));

        let mut physical_rules = RuleSet::new("physical");
        physical_rules.add_rule(Box::new(super::rule::JoinOptimizationRule));
        physical_rules.add_rule(Box::new(super::rule::LimitOptimizationRule));
        physical_rules.add_rule(Box::new(super::advanced_rules::PushFilterDownTraverseRule));
        physical_rules.add_rule(Box::new(super::advanced_rules::PushLimitDownRule));
        physical_rules.add_rule(Box::new(super::index_scan_rules::IndexScanRule));
        physical_rules.add_rule(Box::new(super::index_scan_rules::EdgeIndexFullScanRule));
        physical_rules.add_rule(Box::new(super::index_scan_rules::TagIndexFullScanRule));
        physical_rules.add_rule(Box::new(super::join_rules::PushFilterDownInnerJoinRule));
        physical_rules.add_rule(Box::new(super::join_rules::PushFilterDownHashInnerJoinRule));
        physical_rules.add_rule(Box::new(super::join_rules::PushFilterDownHashLeftJoinRule));
        physical_rules.add_rule(Box::new(super::join_rules::MergeGetVerticesAndDedupRule));
        physical_rules.add_rule(Box::new(super::join_rules::MergeGetVerticesAndProjectRule));
        physical_rules.add_rule(Box::new(super::join_rules::MergeGetNbrsAndDedupRule));
        physical_rules.add_rule(Box::new(super::join_rules::MergeGetNbrsAndProjectRule));
        physical_rules.add_rule(Box::new(super::limit_rules::PushLimitDownGetVerticesRule));
        physical_rules.add_rule(Box::new(super::limit_rules::PushLimitDownGetNeighborsRule));
        physical_rules.add_rule(Box::new(super::limit_rules::PushLimitDownGetEdgesRule));
        physical_rules.add_rule(Box::new(super::limit_rules::PushLimitDownScanVerticesRule));
        physical_rules.add_rule(Box::new(super::limit_rules::PushLimitDownScanEdgesRule));
        physical_rules.add_rule(Box::new(super::limit_rules::PushLimitDownIndexScanRule));
        physical_rules.add_rule(Box::new(super::limit_rules::PushLimitDownProjectRule));

        Self::new(vec![logical_rules, physical_rules])
    }

    pub fn find_best_plan(&mut self, qctx: &mut QueryContext, plan: ExecutionPlan) -> Result<ExecutionPlan, OptimizerError> {
         // Create an optimization context
         let mut opt_ctx = OptContext::new(qctx.clone());

         // Convert the execution plan to an optimization graph
         let mut root_group = self.plan_to_group(&plan)?;

         // Apply optimization rules
         for rule_set in &self.rule_sets {
             for rule in &rule_set.rules {
                 // Apply the rule to the group
                 self.apply_rule(&mut opt_ctx, &mut root_group, rule.as_ref())?;
             }
         }

         // Convert the optimized group back to an execution plan
         let optimized_plan = self.group_to_plan(&root_group)?;

         Ok(optimized_plan)
     }

    fn plan_to_group(&self, plan: &ExecutionPlan) -> Result<OptGroup, OptimizerError> {
        // Convert an execution plan to an optimization group structure
        if let Some(root_node) = &plan.root {
            let mut group = OptGroup::new(0, false); // Physical group for execution plan
            self.convert_node_to_group(root_node.as_ref(), &mut group, 0)?;
            Ok(group)
        } else {
            Err(OptimizerError::PlanConversionError("Cannot convert empty plan to group".to_string()))
        }
    }

    fn convert_node_to_group(&self, node: &dyn PlanNode, group: &mut OptGroup, node_id: usize) -> Result<(), OptimizerError> {
        // Create an OptGroupNode from the PlanNode
        let opt_node = OptGroupNode::new(node_id, node.clone_plan_node());
        group.nodes.push(opt_node);
        
        // Process dependencies
        for (i, dep) in node.dependencies().iter().enumerate() {
            // In a complete implementation, we would recursively process the dependencies
            // For now, we just call this function recursively
            self.convert_node_to_group(dep.as_ref(), group, node_id + i + 1)?;
        }
        
        Ok(())
    }

    fn group_to_plan(&self, group: &OptGroup) -> Result<ExecutionPlan, OptimizerError> {
         // Convert an optimization group back to an execution plan
         if let Some(opt_node) = group.nodes.first() {
             let root = Some(opt_node.plan_node.clone_plan_node());
             Ok(ExecutionPlan::new(root))
         } else {
             Err(OptimizerError::PlanConversionError("Cannot convert empty group to plan".to_string()))
         }
     }

    fn apply_rule(&self, ctx: &mut OptContext, group: &mut OptGroup, rule: &dyn OptRule) -> Result<(), OptimizerError> {
         // Apply a single optimization rule to the group
         // This implementation applies the rule iteratively until no more changes occur
         let mut changed = true;
         let mut iterations = 0;
         const MAX_ITERATIONS: usize = 10; // Prevent infinite loops

         while changed && iterations < MAX_ITERATIONS {
             changed = false;
             let mut new_nodes = Vec::new();
             let mut old_nodes = Vec::new();
             std::mem::swap(&mut group.nodes, &mut old_nodes);

             for node in old_nodes.into_iter() {
                  if self.matches_pattern(&node, &rule.pattern()) {
                      if let Some(new_node) = rule.apply(ctx, &node)? {
                          new_nodes.push(new_node);
                          ctx.stats.rules_applied += 1;
                          changed = true;
                      } else {
                          new_nodes.push(node);
                      }
                  } else {
                      new_nodes.push(node);
                  }
              }

             group.nodes = new_nodes;
             iterations += 1;
         }

         Ok(())
     }

    fn matches_pattern(&self, node: &OptGroupNode, pattern: &Pattern) -> bool {
         // Check if the node matches the given pattern
         if node.plan_node.kind() != pattern.kind {
             return false;
         }

         // If the pattern has dependencies, check if node's dependencies match
         if !pattern.dependencies.is_empty() {
             // In a real implementation, we would check the actual dependencies
             // For now, we just check if the number of dependencies is sufficient
             // This is a simplified version of the pattern matching
         }

         true
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