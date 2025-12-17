//! 投影节点实现
//! 
//! ProjectNode 用于根据指定的列表达式投影输入数据流

use super::super::plan_node_kind::PlanNodeKind;
use super::traits::{
    PlanNode, PlanNodeClonable, PlanNodeDependencies, PlanNodeIdentifiable,
    PlanNodeMutable, PlanNodeProperties, PlanNodeVisitable, SingleInputPlanNode
};
use super::super::visitor::{PlanNodeVisitError, PlanNodeVisitor};
use crate::query::context::validate::types::Variable;
use crate::query::validator::structs::common_structs::YieldColumn;
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
}

impl ProjectNode {
    /// 创建新的投影节点
    pub fn new(
        input: Arc<dyn PlanNode>,
        columns: Vec<YieldColumn>,
    ) -> Result<Self, crate::query::planner::planner::PlannerError> {
        let col_names: Vec<String> = columns.iter()
            .map(|col| col.alias.clone())
            .collect();
        
        Ok(Self {
            id: -1,
            input,
            columns,
            output_var: None,
            col_names,
            cost: 0.0,
        })
    }
    
    /// 获取投影列
    pub fn columns(&self) -> &[YieldColumn] {
        &self.columns
    }
}

impl PlanNodeIdentifiable for ProjectNode {
    fn id(&self) -> i64 { self.id }
    fn kind(&self) -> PlanNodeKind { PlanNodeKind::Project }
}

impl PlanNodeProperties for ProjectNode {
    fn output_var(&self) -> &Option<Variable> { &self.output_var }
    fn col_names(&self) -> &Vec<String> { &self.col_names }
    fn cost(&self) -> f64 { self.cost }
}

impl PlanNodeDependencies for ProjectNode {
    fn dependencies(&self) -> &[Arc<dyn PlanNode>] { 
        std::slice::from_ref(&self.input) 
    }
    
    fn replace_dependencies(&mut self, deps: Vec<Arc<dyn PlanNode>>) {
        // 投影节点只支持单个输入，取第一个
        if let Some(dep) = deps.into_iter().next() {
            self.input = dep;
        }
    }
    
    fn add_dependency(&mut self, dep: Arc<dyn PlanNode>) {
        self.input = dep;
    }
    
    fn remove_dependency(&mut self, id: i64) -> bool {
        if self.input.id() == id {
            false
        } else {
            false
        }
    }
    
    fn clear_dependencies(&mut self) {
        // 投影节点必须有输入，不能清空
    }
}

impl PlanNodeMutable for ProjectNode {
    fn set_output_var(&mut self, var: Variable) { self.output_var = Some(var); }
    fn set_col_names(&mut self, names: Vec<String>) { self.col_names = names; }
    fn set_cost(&mut self, cost: f64) { self.cost = cost; }
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
        })
    }
}

impl PlanNodeVisitable for ProjectNode {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_project_node(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for ProjectNode {
    fn as_any(&self) -> &dyn std::any::Any { self }
}

impl SingleInputPlanNode for ProjectNode {
    fn input(&self) -> &Arc<dyn PlanNode> {
        &self.input
    }
    
    fn set_input(&mut self, input: Arc<dyn PlanNode>) {
        self.input = input;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::parser::ast::expr::{Expr, VariableExpr};
    use crate::query::parser::ast::types::Span;
    
    #[test]
    fn test_project_node_creation() {
        // 创建一个起始节点作为输入
        let start_node = crate::query::planner::plan::core::nodes::start_node::StartNode::new();
        let start_node = Arc::new(start_node);
        
        let columns = vec![YieldColumn {
            expr: Expr::Variable(VariableExpr::new("test".to_string(), Span::default())),
            alias: "test".to_string(),
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
                expr: Expr::Variable(VariableExpr::new("name".to_string(), Span::default())),
                alias: "name".to_string(),
            },
            YieldColumn {
                expr: Expr::Variable(VariableExpr::new("age".to_string(), Span::default())),
                alias: "age".to_string(),
            },
        ];
        
        let project_node = ProjectNode::new(start_node, columns).unwrap();
        
        assert_eq!(project_node.columns().len(), 2);
        assert_eq!(project_node.columns()[0].alias, "name");
        assert_eq!(project_node.columns()[1].alias, "age");
    }
}