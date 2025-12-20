//! 投影节点实现
//!
//! ProjectNode 用于根据指定的列表达式投影输入数据流

use super::super::plan_node_kind::PlanNodeKind;
use super::super::visitor::{PlanNodeVisitError, PlanNodeVisitor};
use super::traits::{
    PlanNode, PlanNodeClonable, PlanNodeDependencies, PlanNodeDependenciesExt,
    PlanNodeIdentifiable, PlanNodeMutable, PlanNodeProperties, PlanNodeVisitable,
};
use crate::query::context::validate::types::Variable;
use crate::query::validator::YieldColumn;
use std::sync::Arc;

/// 投影节点
///
/// 根据指定的列表达式投影输入数据流
#[derive(Debug, Clone)]
pub struct ProjectNode {
    id: i64,
    input: Arc<dyn PlanNode>,
    columns: Vec<YieldColumn>,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
    dependencies_vec: Vec<Arc<dyn PlanNode>>, // 添加一个 Vec 来满足 trait 要求
}

impl ProjectNode {
    /// 创建新的投影节点
    pub fn new(
        input: Arc<dyn PlanNode>,
        columns: Vec<YieldColumn>,
    ) -> Result<Self, crate::query::planner::planner::PlannerError> {
        let col_names: Vec<String> = columns.iter().map(|col| col.alias.clone()).collect();

        let mut dependencies_vec = Vec::new();
        dependencies_vec.push(input.clone());

        Ok(Self {
            id: -1,
            input,
            columns,
            output_var: None,
            col_names,
            cost: 0.0,
            dependencies_vec,
        })
    }

    /// 获取投影列
    pub fn columns(&self) -> &[YieldColumn] {
        &self.columns
    }
}

impl PlanNodeIdentifiable for ProjectNode {
    fn id(&self) -> i64 {
        self.id
    }
    fn kind(&self) -> PlanNodeKind {
        PlanNodeKind::Project
    }
}

impl PlanNodeProperties for ProjectNode {
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

impl PlanNodeDependencies for ProjectNode {
    fn dependencies(&self) -> Vec<Arc<dyn PlanNode>> {
        self.dependencies_vec.clone()
    }

    fn add_dependency(&mut self, dep: Arc<dyn PlanNode>) {
        self.input = dep.clone();
        self.dependencies_vec.clear();
        self.dependencies_vec.push(dep);
    }

    fn remove_dependency(&mut self, id: i64) -> bool {
        let initial_len = self.dependencies_vec.len();
        self.dependencies_vec.retain(|dep| dep.id() != id);
        let final_len = self.dependencies_vec.len();

        if initial_len != final_len {
            // 更新 input，如果原来的 input 被移除
            if self.input.id() == id {
                // 如果移除了唯一的输入节点，使用 Vec 中的第一个元素作为新的输入
                if let Some(first_dep) = self.dependencies_vec.first() {
                    self.input = first_dep.clone();
                }
            }
            true
        } else {
            false
        }
    }
}

impl PlanNodeDependenciesExt for ProjectNode {
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Arc<dyn PlanNode>]) -> R,
    {
        f(&self.dependencies_vec)
    }
}

impl PlanNodeMutable for ProjectNode {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }
    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for ProjectNode {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(Self {
            id: self.id,
            input: self.input.clone_plan_node(),
            columns: self.columns.clone(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            dependencies_vec: self
                .dependencies_vec
                .iter()
                .map(|dep| dep.clone_plan_node())
                .collect(),
        })
    }

    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        Arc::new(Self {
            id: new_id,
            input: self.input.clone_plan_node(),
            columns: self.columns.clone(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            dependencies_vec: self
                .dependencies_vec
                .iter()
                .map(|dep| dep.clone_plan_node())
                .collect(),
        })
    }
}

impl PlanNodeVisitable for ProjectNode {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_project(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for ProjectNode {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::expression::Expression;

    #[test]
    fn test_project_node_creation() {
        // 创建一个起始节点作为输入
        let start_node = crate::query::planner::plan::core::nodes::start_node::StartNode::new();
        let start_node = Arc::new(start_node);

        let columns = vec![YieldColumn {
            expr: Expression::Variable("test".to_string()),
            alias: "test".to_string(),
            is_matched: false,
        }];

        let project_node = ProjectNode::new(start_node, columns).unwrap();

        assert_eq!(project_node.kind(), PlanNodeKind::Project);
        assert_eq!(project_node.dependencies().len(), 1);
        assert_eq!(project_node.col_names().len(), 1);
        assert_eq!(project_node.col_names()[0], "test");
    }

    #[test]
    fn test_project_node_columns() {
        let start_node = crate::query::planner::plan::core::nodes::start_node::StartNode::new();
        let start_node = Arc::new(start_node);

        let columns = vec![
            YieldColumn {
                expr: Expression::Variable("name".to_string()),
                alias: "name".to_string(),
                is_matched: false,
            },
            YieldColumn {
                expr: Expression::Variable("age".to_string()),
                alias: "age".to_string(),
                is_matched: false,
            },
        ];

        let project_node = ProjectNode::new(start_node, columns).unwrap();

        assert_eq!(project_node.columns().len(), 2);
        assert_eq!(project_node.columns()[0].alias, "name");
        assert_eq!(project_node.columns()[1].alias, "age");
    }
}
