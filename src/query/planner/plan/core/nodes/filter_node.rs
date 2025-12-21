//! 过滤节点实现
//!
//! FilterNode 用于根据指定的条件过滤输入数据流

use super::super::plan_node_kind::PlanNodeKind;
use super::super::visitor::{PlanNodeVisitError, PlanNodeVisitor};
use super::traits::{
    PlanNode, PlanNodeClonable, PlanNodeDependencies, PlanNodeDependenciesExt,
    PlanNodeIdentifiable, PlanNodeMutable, PlanNodeProperties, PlanNodeVisitable,
};
use crate::expression::Expression;
use crate::query::context::validate::types::Variable;
use std::sync::Arc;

/// 过滤节点
///
/// 根据指定的条件表达式过滤输入数据流
#[derive(Debug, Clone)]
pub struct FilterNode {
    id: i64,
    input: Arc<dyn PlanNode>,
    deps: Vec<Arc<dyn PlanNode>>, // 添加这个字段以满足PlanNodeDependencies trait
    condition: Expression,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
}

impl FilterNode {
    /// 创建新的过滤节点
    pub fn new(
        input: Arc<dyn PlanNode>,
        condition: Expression,
    ) -> Result<Self, crate::query::planner::planner::PlannerError> {
        let col_names = input.col_names().to_vec();
        let mut deps = Vec::new();
        deps.push(input.clone());

        Ok(Self {
            id: -1, // 将在后续分配
            input,
            deps,
            condition,
            output_var: None,
            col_names,
            cost: 0.0,
        })
    }

    /// 获取过滤条件
    pub fn condition(&self) -> &Expression {
        &self.condition
    }
}

impl PlanNodeIdentifiable for FilterNode {
    fn id(&self) -> i64 {
        self.id
    }
    fn kind(&self) -> PlanNodeKind {
        PlanNodeKind::Filter
    }
}

impl PlanNodeProperties for FilterNode {
    fn output_var(&self) -> Option<&Variable> {
        self.output_var.as_ref()
    }
    fn col_names(&self) -> &[String] {
        &self.col_names
    }
    fn cost(&self) -> f64 {
        self.cost
    }
}

impl PlanNodeDependencies for FilterNode {
    fn dependencies(&self) -> Vec<Arc<dyn PlanNode>> {
        self.deps.clone()
    }

    fn add_dependency(&mut self, dep: Arc<dyn PlanNode>) {
        // 过滤节点只支持单个输入，替换现有输入
        self.input = dep.clone();
        self.deps.clear();
        self.deps.push(dep);
    }

    fn remove_dependency(&mut self, id: i64) -> bool {
        if let Some(pos) = self.deps.iter().position(|dep| dep.id() == id) {
            self.deps.remove(pos);
            // 还需要更新input字段，保持一致性
            if self.deps.len() == 1 {
                self.input = self.deps[0].clone();
            } else if self.deps.is_empty() {
                // 这种情况不应该发生，因为FilterNode应该始终有一个输入
                // 但为了安全，我们可以创建一个占位符节点
                self.input = Arc::new(
                    crate::query::planner::plan::core::nodes::start_node::StartNode::new(),
                );
            }
            true
        } else {
            false
        }
    }
}

impl PlanNodeDependenciesExt for FilterNode {
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Arc<dyn PlanNode>]) -> R,
    {
        f(&self.deps)
    }
}

impl PlanNodeMutable for FilterNode {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }
    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for FilterNode {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(Self {
            id: self.id,
            input: self.input.clone_plan_node(),
            deps: self.deps.clone(),
            condition: self.condition.clone(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
        })
    }

    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        Arc::new(Self {
            id: new_id,
            input: self.input.clone_plan_node(),
            deps: self.deps.clone(), // 包含deps字段
            condition: self.condition.clone(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
        })
    }
}

impl PlanNodeVisitable for FilterNode {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_filter(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for FilterNode {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expression::Expression;

    #[test]
    fn test_filter_node_creation() {
        // 创建一个起始节点作为输入
        let start_node = crate::query::planner::plan::core::nodes::start_node::StartNode::new();
        let start_node = Arc::new(start_node);

        let condition = Expression::Variable("test".to_string());
        let filter_node = FilterNode::new(start_node, condition).expect("Filter node should be created successfully");

        assert_eq!(filter_node.kind(), PlanNodeKind::Filter);
        assert_eq!(filter_node.dependencies().len(), 1);
    }

    #[test]
    fn test_filter_node_dependencies() {
        let start_node = crate::query::planner::plan::core::nodes::start_node::StartNode::new();
        let start_node = Arc::new(start_node);
        let start_node_id = start_node.id();

        let condition = Expression::Variable("test".to_string());
        let mut filter_node = FilterNode::new(start_node, condition).expect("Failed to create filter node");

        // 测试依赖管理
        assert_eq!(filter_node.dependency_count(), 1);
        assert!(filter_node.has_dependency(start_node_id));

        // 测试替换依赖
        let new_start_node = crate::query::planner::plan::core::nodes::start_node::StartNode::new();
        let new_start_node = Arc::new(new_start_node);
        filter_node.add_dependency(new_start_node.clone());

        assert_eq!(filter_node.dependency_count(), 1);
        assert!(filter_node.has_dependency(new_start_node.id()));
    }
}
