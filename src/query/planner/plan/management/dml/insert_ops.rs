//! 数据插入操作相关的计划节点
//! 包括插入顶点和边的操作

use crate::query::planner::plan::core::{PlanNode as BasePlanNode, PlanNodeKind, PlanNodeVisitor, PlanNodeVisitError};
use crate::query::validator::Variable;

/// 插入顶点计划节点
#[derive(Debug)]
pub struct InsertVertices {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Box<dyn BasePlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub space_id: i32,
    pub tag_id: i32,
    pub props: Vec<(String, String)>, // 属性名和值
    pub insertable: bool,            // 是否可插入
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

impl BasePlanNode for InsertVertices {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }

    fn dependencies(&self) -> &Vec<Box<dyn BasePlanNode>> {
        &self.deps
    }

    fn output_var(&self) -> &Option<Variable> {
        &self.output_var
    }

    fn col_names(&self) -> &Vec<String> {
        &self.col_names
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn clone_plan_node(&self) -> Box<dyn BasePlanNode> {
        Box::new(self.clone())
    }

    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_insert_vertices(self)?;
        visitor.post_visit()?;
        Ok(())
    }

    fn set_dependencies(&mut self, deps: Vec<Box<dyn BasePlanNode>>) {
        self.deps = deps;
    }

    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }

    fn set_cost(&mut self, cost: f64) {
        self.cost = cost;
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 插入边计划节点
#[derive(Debug)]
pub struct InsertEdges {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Box<dyn BasePlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub space_id: i32,
    pub edge_type_id: i32,
    pub props: Vec<(String, String)>, // 属性名和值
    pub insertable: bool,            // 是否可插入
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

impl BasePlanNode for InsertEdges {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }

    fn dependencies(&self) -> &Vec<Box<dyn BasePlanNode>> {
        &self.deps
    }

    fn output_var(&self) -> &Option<Variable> {
        &self.output_var
    }

    fn col_names(&self) -> &Vec<String> {
        &self.col_names
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn clone_plan_node(&self) -> Box<dyn BasePlanNode> {
        Box::new(self.clone())
    }

    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_insert_edges(self)?;
        visitor.post_visit()?;
        Ok(())
    }

    fn set_dependencies(&mut self, deps: Vec<Box<dyn BasePlanNode>>) {
        self.deps = deps;
    }

    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }

    fn set_cost(&mut self, cost: f64) {
        self.cost = cost;
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}