//! 聚合节点实现
//!
//! AggregateNode 用于对输入数据进行聚合操作

use super::super::plan_node_kind::PlanNodeKind;
use super::traits::{
    PlanNode, PlanNodeClonable, PlanNodeDependencies, PlanNodeIdentifiable,
    PlanNodeMutable, PlanNodeProperties, PlanNodeVisitable
};
use super::super::visitor::{PlanNodeVisitError, PlanNodeVisitor};
use crate::query::context::validate::types::Variable;
use std::sync::Arc;

/// 聚合节点
///
/// 根据指定的分组键和聚合表达式对输入数据进行聚合
#[derive(Debug, Clone)]
pub struct AggregateNode {
    id: i64,
    input: Arc<dyn PlanNode>,
    deps: Vec<Arc<dyn PlanNode>>,
    group_keys: Vec<String>,
    agg_exprs: Vec<String>,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
}

impl AggregateNode {
    /// 创建新的聚合节点
    pub fn new(
        input: Arc<dyn PlanNode>,
        group_keys: Vec<String>,
        agg_exprs: Vec<String>,
    ) -> Result<Self, crate::query::planner::planner::PlannerError> {
        let col_names: Vec<String> = group_keys.iter()
            .chain(agg_exprs.iter())
            .cloned()
            .collect();
        
        let mut deps = Vec::new();
        deps.push(input.clone());

        Ok(Self {
            id: -1,
            input,
            deps,
            group_keys,
            agg_exprs,
            output_var: None,
            col_names,
            cost: 0.0,
        })
    }

    /// 获取分组键
    pub fn group_keys(&self) -> &[String] {
        &self.group_keys
    }

    /// 获取聚合表达式
    pub fn agg_exprs(&self) -> &[String] {
        &self.agg_exprs
    }

}

impl PlanNodeIdentifiable for AggregateNode {
    fn id(&self) -> i64 { self.id }
    fn kind(&self) -> PlanNodeKind { PlanNodeKind::Aggregate }
}

impl PlanNodeProperties for AggregateNode {
    fn output_var(&self) -> Option<&Variable> { self.output_var.as_ref() }
    fn col_names(&self) -> &[String] { &self.col_names }
    fn cost(&self) -> f64 { self.cost }
}

impl PlanNodeDependencies for AggregateNode {
    fn dependencies(&self) -> &[Arc<dyn PlanNode>] {
        &self.deps
    }

    fn dependencies_mut(&mut self) -> &mut Vec<Arc<dyn PlanNode>> {
        &mut self.deps
    }

    fn add_dependency(&mut self, dep: Arc<dyn PlanNode>) {
        self.input = dep.clone();
        self.deps.clear();
        self.deps.push(dep);
    }

    fn remove_dependency(&mut self, _id: i64) -> bool {
        // 聚合节点只支持单个输入，这个方法在当前设计中不太适用
        false
    }
}

impl PlanNodeMutable for AggregateNode {
    fn set_output_var(&mut self, var: Variable) { self.output_var = Some(var); }
    fn set_col_names(&mut self, names: Vec<String>) { self.col_names = names; }
}

impl PlanNodeClonable for AggregateNode {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(Self {
            id: self.id,
            input: self.input.clone_plan_node(),
            deps: self.deps.clone(),
            group_keys: self.group_keys.clone(),
            agg_exprs: self.agg_exprs.clone(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
        })
    }

    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        Arc::new(Self {
            id: new_id,
            input: self.input.clone_plan_node(),
            deps: self.deps.clone(),
            group_keys: self.group_keys.clone(),
            agg_exprs: self.agg_exprs.clone(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
        })
    }
}

impl PlanNodeVisitable for AggregateNode {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_aggregate(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for AggregateNode {
    fn as_any(&self) -> &dyn std::any::Any { self }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::planner::plan::core::nodes::start_node::StartNode;
    
    #[test]
    fn test_aggregate_node_creation() {
        let start_node = StartNode::new();
        let start_node = Arc::new(start_node);
        
        let group_keys = vec!["category".to_string()];
        let agg_exprs = vec!["COUNT(*)".to_string()];
        
        let aggregate_node = AggregateNode::new(start_node, group_keys, agg_exprs).unwrap();
        
        assert_eq!(aggregate_node.kind(), PlanNodeKind::Aggregate);
        assert_eq!(aggregate_node.dependencies().len(), 1);
        assert_eq!(aggregate_node.group_keys().len(), 1);
        assert_eq!(aggregate_node.agg_exprs().len(), 1);
    }
}