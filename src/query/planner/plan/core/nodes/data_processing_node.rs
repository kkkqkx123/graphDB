//! 数据处理节点实现
//!
//! 包含Union、Unwind、Dedup等数据处理相关的计划节点

use super::super::plan_node_kind::PlanNodeKind;
use super::traits::{PlanNode, PlanNodeClonable, PlanNodeDependencies, PlanNodeDependenciesExt, PlanNodeIdentifiable, PlanNodeMutable, PlanNodeProperties, PlanNodeVisitable};
use super::super::visitor::{PlanNodeVisitError, PlanNodeVisitor};
use crate::query::context::validate::types::Variable;
use std::sync::Arc;

/// Union节点
#[derive(Debug, Clone)]
pub struct UnionNode {
    id: i64,
    input: Arc<dyn PlanNode>,
    deps: Vec<Arc<dyn PlanNode>>,
    distinct: bool,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
}

impl UnionNode {
    pub fn new(input: Arc<dyn PlanNode>, distinct: bool) -> Result<Self, crate::query::planner::planner::PlannerError> {
        let col_names = input.col_names().to_vec();
        let mut deps = Vec::new();
        deps.push(input.clone());

        Ok(Self {
            id: -1,
            input,
            deps,
            distinct,
            output_var: None,
            col_names,
            cost: 0.0,
        })
    }

    pub fn distinct(&self) -> bool {
        self.distinct
    }
}

impl PlanNodeIdentifiable for UnionNode {
    fn id(&self) -> i64 { self.id }
    fn kind(&self) -> PlanNodeKind { PlanNodeKind::Union }
}

impl PlanNodeProperties for UnionNode {
    fn output_var(&self) -> Option<&Variable> { self.output_var.as_ref() }
    fn col_names(&self) -> &[String] { &self.col_names }
    fn cost(&self) -> f64 { self.cost }
}

impl PlanNodeDependencies for UnionNode {
    fn dependencies(&self) -> Vec<Arc<dyn PlanNode>> {
        self.deps.clone()
    }

    fn add_dependency(&mut self, dep: Arc<dyn PlanNode>) {
        self.input = dep.clone();
        self.deps.clear();
        self.deps.push(dep);
    }

    fn remove_dependency(&mut self, _id: i64) -> bool {
         false
     }
}

impl PlanNodeDependenciesExt for UnionNode {
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Arc<dyn PlanNode>]) -> R
    {
        f(&self.deps)
    }
}
    
impl PlanNodeMutable for UnionNode {
    fn set_output_var(&mut self, var: Variable) { self.output_var = Some(var); }
    fn set_col_names(&mut self, names: Vec<String>) { self.col_names = names; }
}

impl PlanNodeClonable for UnionNode {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(Self {
            id: self.id,
            input: self.input.clone_plan_node(),
            deps: self.deps.clone(),
            distinct: self.distinct,
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
        })
    }

    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for UnionNode {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_union(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for UnionNode {
    fn as_any(&self) -> &dyn std::any::Any { self }
}

/// Unwind节点
#[derive(Debug, Clone)]
pub struct UnwindNode {
    id: i64,
    input: Arc<dyn PlanNode>,
    deps: Vec<Arc<dyn PlanNode>>,
    alias: String,
    list_expr: String,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
}

impl UnwindNode {
    pub fn new(
        input: Arc<dyn PlanNode>,
        alias: &str,
        list_expr: &str,
    ) -> Result<Self, crate::query::planner::planner::PlannerError> {
        let mut col_names = input.col_names().to_vec();
        col_names.push(alias.to_string());
        
        let mut deps = Vec::new();
        deps.push(input.clone());

        Ok(Self {
            id: -1,
            input,
            deps,
            alias: alias.to_string(),
            list_expr: list_expr.to_string(),
            output_var: None,
            col_names,
            cost: 0.0,
        })
    }

    pub fn alias(&self) -> &str {
        &self.alias
    }

    pub fn list_expr(&self) -> &str {
        &self.list_expr
    }
}

impl PlanNodeIdentifiable for UnwindNode {
    fn id(&self) -> i64 { self.id }
    fn kind(&self) -> PlanNodeKind { PlanNodeKind::Unwind }
}

impl PlanNodeProperties for UnwindNode {
    fn output_var(&self) -> Option<&Variable> { self.output_var.as_ref() }
    fn col_names(&self) -> &[String] { &self.col_names }
    fn cost(&self) -> f64 { self.cost }
}

impl PlanNodeDependencies for UnwindNode {
    fn dependencies(&self) -> Vec<Arc<dyn PlanNode>> {
        self.deps.clone()
    }

    fn add_dependency(&mut self, dep: Arc<dyn PlanNode>) {
        self.input = dep.clone();
        self.deps.clear();
        self.deps.push(dep);
    }

    fn remove_dependency(&mut self, _id: i64) -> bool {
         false
     }
}

impl PlanNodeDependenciesExt for UnwindNode {
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Arc<dyn PlanNode>]) -> R
    {
        f(&self.deps)
    }
}
    
impl PlanNodeMutable for UnwindNode {
    fn set_output_var(&mut self, var: Variable) { self.output_var = Some(var); }
    fn set_col_names(&mut self, names: Vec<String>) { self.col_names = names; }
}

impl PlanNodeClonable for UnwindNode {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(Self {
            id: self.id,
            input: self.input.clone_plan_node(),
            deps: self.deps.clone(),
            alias: self.alias.clone(),
            list_expr: self.list_expr.clone(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
        })
    }

    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for UnwindNode {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_unwind(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for UnwindNode {
    fn as_any(&self) -> &dyn std::any::Any { self }
}

/// 去重节点
#[derive(Debug, Clone)]
pub struct DedupNode {
    id: i64,
    input: Arc<dyn PlanNode>,
    deps: Vec<Arc<dyn PlanNode>>,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
}

impl DedupNode {
    pub fn new(input: Arc<dyn PlanNode>) -> Result<Self, crate::query::planner::planner::PlannerError> {
        let col_names = input.col_names().to_vec();
        let mut deps = Vec::new();
        deps.push(input.clone());

        Ok(Self {
            id: -1,
            input,
            deps,
            output_var: None,
            col_names,
            cost: 0.0,
        })
    }
}

impl PlanNodeIdentifiable for DedupNode {
    fn id(&self) -> i64 { self.id }
    fn kind(&self) -> PlanNodeKind { PlanNodeKind::Dedup }
}

impl PlanNodeProperties for DedupNode {
    fn output_var(&self) -> Option<&Variable> { self.output_var.as_ref() }
    fn col_names(&self) -> &[String] { &self.col_names }
    fn cost(&self) -> f64 { self.cost }
}

impl PlanNodeDependencies for DedupNode {
    fn dependencies(&self) -> Vec<Arc<dyn PlanNode>> {
        self.deps.clone()
    }

    fn add_dependency(&mut self, dep: Arc<dyn PlanNode>) {
        self.input = dep.clone();
        self.deps.clear();
        self.deps.push(dep);
    }

    fn remove_dependency(&mut self, _id: i64) -> bool {
         false
     }
}

impl PlanNodeDependenciesExt for DedupNode {
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Arc<dyn PlanNode>]) -> R
    {
        f(&self.deps)
    }
}
    
impl PlanNodeMutable for DedupNode {
    fn set_output_var(&mut self, var: Variable) { self.output_var = Some(var); }
    fn set_col_names(&mut self, names: Vec<String>) { self.col_names = names; }
}

impl PlanNodeClonable for DedupNode {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(Self {
            id: self.id,
            input: self.input.clone_plan_node(),
            deps: self.deps.clone(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
        })
    }

    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for DedupNode {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_dedup(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for DedupNode {
    fn as_any(&self) -> &dyn std::any::Any { self }
}

/// RollUpApply节点
#[derive(Debug, Clone)]
pub struct RollUpApplyNode {
    id: i64,
    input: Arc<dyn PlanNode>,
    deps: Vec<Arc<dyn PlanNode>>,
    collect_exprs: Vec<String>,
    lambda_vars: Vec<String>,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
}

impl RollUpApplyNode {
    pub fn new(
        input: Arc<dyn PlanNode>,
        collect_exprs: Vec<String>,
        lambda_vars: Vec<String>,
    ) -> Result<Self, crate::query::planner::planner::PlannerError> {
        let col_names = input.col_names().to_vec();
        let mut deps = Vec::new();
        deps.push(input.clone());

        Ok(Self {
            id: -1,
            input,
            deps,
            collect_exprs,
            lambda_vars,
            output_var: None,
            col_names,
            cost: 0.0,
        })
    }

    pub fn collect_exprs(&self) -> &[String] {
        &self.collect_exprs
    }

    pub fn lambda_vars(&self) -> &[String] {
        &self.lambda_vars
    }
}

impl PlanNodeIdentifiable for RollUpApplyNode {
    fn id(&self) -> i64 { self.id }
    fn kind(&self) -> PlanNodeKind { PlanNodeKind::RollUpApply }
}

impl PlanNodeProperties for RollUpApplyNode {
    fn output_var(&self) -> Option<&Variable> { self.output_var.as_ref() }
    fn col_names(&self) -> &[String] { &self.col_names }
    fn cost(&self) -> f64 { self.cost }
}

impl PlanNodeDependencies for RollUpApplyNode {
    fn dependencies(&self) -> Vec<Arc<dyn PlanNode>> {
        self.deps.clone()
    }

    fn add_dependency(&mut self, dep: Arc<dyn PlanNode>) {
        self.input = dep.clone();
        self.deps.clear();
        self.deps.push(dep);
    }

    fn remove_dependency(&mut self, _id: i64) -> bool {
         false
     }
}

impl PlanNodeDependenciesExt for RollUpApplyNode {
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Arc<dyn PlanNode>]) -> R
    {
        f(&self.deps)
    }
}
    
impl PlanNodeMutable for RollUpApplyNode {
    fn set_output_var(&mut self, var: Variable) { self.output_var = Some(var); }
    fn set_col_names(&mut self, names: Vec<String>) { self.col_names = names; }
}

impl PlanNodeClonable for RollUpApplyNode {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(Self {
            id: self.id,
            input: self.input.clone_plan_node(),
            deps: self.deps.clone(),
            collect_exprs: self.collect_exprs.clone(),
            lambda_vars: self.lambda_vars.clone(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
        })
    }

    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for RollUpApplyNode {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_roll_up_apply(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for RollUpApplyNode {
    fn as_any(&self) -> &dyn std::any::Any { self }
}

/// PatternApply节点
#[derive(Debug, Clone)]
pub struct PatternApplyNode {
    id: i64,
    input: Arc<dyn PlanNode>,
    deps: Vec<Arc<dyn PlanNode>>,
    pattern: String,
    join_type: String,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
}

impl PatternApplyNode {
    pub fn new(
        input: Arc<dyn PlanNode>,
        pattern: &str,
        join_type: &str,
    ) -> Result<Self, crate::query::planner::planner::PlannerError> {
        let col_names = input.col_names().to_vec();
        let mut deps = Vec::new();
        deps.push(input.clone());

        Ok(Self {
            id: -1,
            input,
            deps,
            pattern: pattern.to_string(),
            join_type: join_type.to_string(),
            output_var: None,
            col_names,
            cost: 0.0,
        })
    }

    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    pub fn join_type(&self) -> &str {
        &self.join_type
    }
}

impl PlanNodeIdentifiable for PatternApplyNode {
    fn id(&self) -> i64 { self.id }
    fn kind(&self) -> PlanNodeKind { PlanNodeKind::PatternApply }
}

impl PlanNodeProperties for PatternApplyNode {
    fn output_var(&self) -> Option<&Variable> { self.output_var.as_ref() }
    fn col_names(&self) -> &[String] { &self.col_names }
    fn cost(&self) -> f64 { self.cost }
}

impl PlanNodeDependencies for PatternApplyNode {
    fn dependencies(&self) -> Vec<Arc<dyn PlanNode>> {
        self.deps.clone()
    }

    fn add_dependency(&mut self, dep: Arc<dyn PlanNode>) {
        self.input = dep.clone();
        self.deps.clear();
        self.deps.push(dep);
    }

    fn remove_dependency(&mut self, _id: i64) -> bool {
         false
     }
}

impl PlanNodeDependenciesExt for PatternApplyNode {
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Arc<dyn PlanNode>]) -> R
    {
        f(&self.deps)
    }
}
    
impl PlanNodeMutable for PatternApplyNode {
    fn set_output_var(&mut self, var: Variable) { self.output_var = Some(var); }
    fn set_col_names(&mut self, names: Vec<String>) { self.col_names = names; }
}

impl PlanNodeClonable for PatternApplyNode {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(Self {
            id: self.id,
            input: self.input.clone_plan_node(),
            deps: self.deps.clone(),
            pattern: self.pattern.clone(),
            join_type: self.join_type.clone(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
        })
    }

    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for PatternApplyNode {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_pattern_apply(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for PatternApplyNode {
    fn as_any(&self) -> &dyn std::any::Any { self }
}

/// 数据收集节点
#[derive(Debug, Clone)]
pub struct DataCollectNode {
    id: i64,
    input: Arc<dyn PlanNode>,
    deps: Vec<Arc<dyn PlanNode>>,
    collect_kind: String,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
}

impl DataCollectNode {
    pub fn new(input: Arc<dyn PlanNode>, collect_kind: &str) -> Result<Self, crate::query::planner::planner::PlannerError> {
        let col_names = input.col_names().to_vec();
        let mut deps = Vec::new();
        deps.push(input.clone());

        Ok(Self {
            id: -1,
            input,
            deps,
            collect_kind: collect_kind.to_string(),
            output_var: None,
            col_names,
            cost: 0.0,
        })
    }

    pub fn collect_kind(&self) -> &str {
        &self.collect_kind
    }
}

impl PlanNodeIdentifiable for DataCollectNode {
    fn id(&self) -> i64 { self.id }
    fn kind(&self) -> PlanNodeKind { PlanNodeKind::DataCollect }
}

impl PlanNodeProperties for DataCollectNode {
    fn output_var(&self) -> Option<&Variable> { self.output_var.as_ref() }
    fn col_names(&self) -> &[String] { &self.col_names }
    fn cost(&self) -> f64 { self.cost }
}

impl PlanNodeDependencies for DataCollectNode {
    fn dependencies(&self) -> Vec<Arc<dyn PlanNode>> {
        self.deps.clone()
    }

    fn add_dependency(&mut self, dep: Arc<dyn PlanNode>) {
        self.input = dep.clone();
        self.deps.clear();
        self.deps.push(dep);
    }

    fn remove_dependency(&mut self, _id: i64) -> bool {
         false
     }
}

impl PlanNodeDependenciesExt for DataCollectNode {
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Arc<dyn PlanNode>]) -> R
    {
        f(&self.deps)
    }
}
    
impl PlanNodeMutable for DataCollectNode {
    fn set_output_var(&mut self, var: Variable) { self.output_var = Some(var); }
    fn set_col_names(&mut self, names: Vec<String>) { self.col_names = names; }
}

impl PlanNodeClonable for DataCollectNode {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(Self {
            id: self.id,
            input: self.input.clone_plan_node(),
            deps: self.deps.clone(),
            collect_kind: self.collect_kind.clone(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
        })
    }

    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for DataCollectNode {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_data_collect(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for DataCollectNode {
    fn as_any(&self) -> &dyn std::any::Any { self }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::planner::plan::core::nodes::start_node::StartNode;
    
    #[test]
    fn test_union_node_creation() {
        let start_node = StartNode::new();
        let start_node = Arc::new(start_node);
        
        let union_node = UnionNode::new(start_node, true).unwrap();
        
        assert_eq!(union_node.kind(), PlanNodeKind::Union);
        assert_eq!(union_node.dependencies().len(), 1);
        assert!(union_node.distinct());
    }
    
    #[test]
    fn test_unwind_node_creation() {
        let start_node = StartNode::new();
        let start_node = Arc::new(start_node);
        
        let unwind_node = UnwindNode::new(start_node, "item", "list").unwrap();
        
        assert_eq!(unwind_node.kind(), PlanNodeKind::Unwind);
        assert_eq!(unwind_node.dependencies().len(), 1);
        assert_eq!(unwind_node.alias(), "item");
        assert_eq!(unwind_node.list_expr(), "list");
    }
    
    #[test]
    fn test_dedup_node_creation() {
        let start_node = StartNode::new();
        let start_node = Arc::new(start_node);
        
        let dedup_node = DedupNode::new(start_node).unwrap();
        
        assert_eq!(dedup_node.kind(), PlanNodeKind::Dedup);
        assert_eq!(dedup_node.dependencies().len(), 1);
    }
}