//! 数据更新操作相关的计划节点
//! 包括更新顶点和边的操作

use crate::query::context::validate::types::Variable;
use crate::query::planner::plan::core::{
    plan_node_traits::{
        PlanNode, PlanNodeClonable, PlanNodeDependencies, PlanNodeIdentifiable, PlanNodeMutable,
        PlanNodeProperties, PlanNodeVisitable,
    },
    PlanNodeKind, PlanNodeVisitError, PlanNodeVisitor,
};
use std::sync::Arc;

/// 更新顶点计划节点
#[derive(Debug)]
pub struct UpdateVertex {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub space_id: i32,
    pub tag_id: i32,
    pub filter: Option<String>,              // 过滤条件
    pub update_props: Vec<(String, String)>, // 要更新的属性
    pub insertable: bool,                    // 是否可插入
}

impl UpdateVertex {
    pub fn new(
        id: i64,
        space_id: i32,
        tag_id: i32,
        filter: Option<String>,
        update_props: Vec<(String, String)>,
        insertable: bool,
    ) -> Self {
        Self {
            id,
            kind: PlanNodeKind::UpdateVertex,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            space_id,
            tag_id,
            filter,
            update_props,
            insertable,
        }
    }
}

impl Clone for UpdateVertex {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            space_id: self.space_id,
            tag_id: self.tag_id,
            filter: self.filter.clone(),
            update_props: self.update_props.clone(),
            insertable: self.insertable,
        }
    }
}

impl PlanNodeIdentifiable for UpdateVertex {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for UpdateVertex {
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

impl PlanNodeDependencies for UpdateVertex {
    fn dependencies(&self) -> &[Arc<dyn PlanNode>] {
        &self.deps
    }

    fn add_dependency(&mut self, dep: Arc<dyn PlanNode>) {
        self.deps.push(dep);
    }
}

impl PlanNodeMutable for UpdateVertex {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for UpdateVertex {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }
    
    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for UpdateVertex {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_update_vertex(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for UpdateVertex {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 更新边计划节点
#[derive(Debug)]
pub struct UpdateEdge {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub space_id: i32,
    pub edge_type_id: i32,
    pub filter: Option<String>,              // 过滤条件
    pub update_props: Vec<(String, String)>, // 要更新的属性
    pub insertable: bool,                    // 是否可插入
}

impl UpdateEdge {
    pub fn new(
        id: i64,
        space_id: i32,
        edge_type_id: i32,
        filter: Option<String>,
        update_props: Vec<(String, String)>,
        insertable: bool,
    ) -> Self {
        Self {
            id,
            kind: PlanNodeKind::UpdateEdge,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            space_id,
            edge_type_id,
            filter,
            update_props,
            insertable,
        }
    }
}

impl Clone for UpdateEdge {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            space_id: self.space_id,
            edge_type_id: self.edge_type_id,
            filter: self.filter.clone(),
            update_props: self.update_props.clone(),
            insertable: self.insertable,
        }
    }
}

impl PlanNodeIdentifiable for UpdateEdge {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for UpdateEdge {
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

impl PlanNodeDependencies for UpdateEdge {
    fn dependencies(&self) -> &[Arc<dyn PlanNode>] {
        &self.deps
    }

    fn add_dependency(&mut self, dep: Arc<dyn PlanNode>) {
        self.deps.push(dep);
    }
}

impl PlanNodeMutable for UpdateEdge {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for UpdateEdge {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }
    
    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for UpdateEdge {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_update_edge(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for UpdateEdge {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
