//! 数据插入操作相关的计划节点
//! 包括插入顶点和边的操作

use crate::query::context::validate::types::Variable;
use crate::query::planner::plan::core::{
    plan_node_traits::{
        PlanNode, PlanNodeClonable, PlanNodeDependencies, PlanNodeIdentifiable, PlanNodeMutable,
        PlanNodeProperties, PlanNodeVisitable,
    },
    PlanNodeKind, PlanNodeVisitError, PlanNodeVisitor,
};
use std::sync::Arc;

/// 插入顶点计划节点
#[derive(Debug)]
pub struct InsertVertices {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub space_id: i32,
    pub tag_id: i32,
    pub props: Vec<(String, String)>, // 属性名和值
    pub insertable: bool,             // 是否可插入
}

impl InsertVertices {
    pub fn new(
        id: i64,
        space_id: i32,
        tag_id: i32,
        props: Vec<(String, String)>,
        insertable: bool,
    ) -> Self {
        Self {
            id,
            kind: PlanNodeKind::InsertVertices,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            space_id,
            tag_id,
            props,
            insertable,
        }
    }
}

impl Clone for InsertVertices {
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
            props: self.props.clone(),
            insertable: self.insertable,
        }
    }
}

impl PlanNodeIdentifiable for InsertVertices {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for InsertVertices {
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

impl PlanNodeDependencies for InsertVertices {
    fn dependencies(&self) -> &[Arc<dyn PlanNode>] {
        &self.deps
    }

    fn dependencies_mut(&mut self) -> &mut Vec<Arc<dyn PlanNode>> {
        &mut self.deps
    }

    fn add_dependency(&mut self, dep: Arc<dyn PlanNode>) {
        self.deps.push(dep);
    }

    fn remove_dependency(&mut self, id: i64) -> bool {
        if let Some(pos) = self.deps.iter().position(|dep| dep.id() == id) {
            self.deps.remove(pos);
            true
        } else {
            false
        }
    }
}

impl PlanNodeMutable for InsertVertices {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for InsertVertices {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }
    
    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for InsertVertices {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_insert_vertices(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for InsertVertices {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 插入边计划节点
#[derive(Debug)]
pub struct InsertEdges {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub space_id: i32,
    pub edge_type_id: i32,
    pub props: Vec<(String, String)>, // 属性名和值
    pub insertable: bool,             // 是否可插入
}

impl InsertEdges {
    pub fn new(
        id: i64,
        space_id: i32,
        edge_type_id: i32,
        props: Vec<(String, String)>,
        insertable: bool,
    ) -> Self {
        Self {
            id,
            kind: PlanNodeKind::InsertEdges,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            space_id,
            edge_type_id,
            props,
            insertable,
        }
    }
}

impl Clone for InsertEdges {
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
            props: self.props.clone(),
            insertable: self.insertable,
        }
    }
}

impl PlanNodeIdentifiable for InsertEdges {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for InsertEdges {
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

impl PlanNodeDependencies for InsertEdges {
    fn dependencies(&self) -> &[Arc<dyn PlanNode>] {
        &self.deps
    }

    fn dependencies_mut(&mut self) -> &mut Vec<Arc<dyn PlanNode>> {
        &mut self.deps
    }

    fn add_dependency(&mut self, dep: Arc<dyn PlanNode>) {
        self.deps.push(dep);
    }

    fn remove_dependency(&mut self, id: i64) -> bool {
        if let Some(pos) = self.deps.iter().position(|dep| dep.id() == id) {
            self.deps.remove(pos);
            true
        } else {
            false
        }
    }
}

impl PlanNodeMutable for InsertEdges {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for InsertEdges {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }
    
    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for InsertEdges {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_insert_edges(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for InsertEdges {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
