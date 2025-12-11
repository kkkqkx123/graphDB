//! 数据构造操作相关的计划节点
//! 包括创建新顶点、标签和属性的操作

use crate::query::planner::plan::core::{
    PlanNode as BasePlanNode, PlanNodeKind, PlanNodeVisitError, PlanNodeVisitor,
};
use crate::query::validator::Variable;

/// 创建新顶点计划节点
#[derive(Debug)]
pub struct NewVertex {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Box<dyn BasePlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub tag_id: i32,
    pub props: Vec<(String, String)>, // 属性名和值
}

impl NewVertex {
    pub fn new(id: i64, tag_id: i32, props: Vec<(String, String)>) -> Self {
        Self {
            id,
            kind: PlanNodeKind::NewVertex,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            tag_id,
            props,
        }
    }
}

impl Clone for NewVertex {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            tag_id: self.tag_id,
            props: self.props.clone(),
        }
    }
}

impl BasePlanNode for NewVertex {
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
        visitor.visit_new_vertex(self)?;
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
}

/// 创建新标签计划节点
#[derive(Debug)]
pub struct NewTag {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Box<dyn BasePlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub tag_id: i32,
    pub props: Vec<(String, String)>, // 属性名和值
}

impl NewTag {
    pub fn new(id: i64, tag_id: i32, props: Vec<(String, String)>) -> Self {
        Self {
            id,
            kind: PlanNodeKind::NewTag,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            tag_id,
            props,
        }
    }
}

impl Clone for NewTag {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            tag_id: self.tag_id,
            props: self.props.clone(),
        }
    }
}

impl BasePlanNode for NewTag {
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
        visitor.visit_new_tag(self)?;
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
}

/// 创建新属性计划节点
#[derive(Debug)]
pub struct NewProp {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Box<dyn BasePlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub prop_name: String,
    pub prop_value: String,
}

impl NewProp {
    pub fn new(id: i64, prop_name: &str, prop_value: &str) -> Self {
        Self {
            id,
            kind: PlanNodeKind::NewProp,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            prop_name: prop_name.to_string(),
            prop_value: prop_value.to_string(),
        }
    }
}

impl Clone for NewProp {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            prop_name: self.prop_name.clone(),
            prop_value: self.prop_value.clone(),
        }
    }
}

impl BasePlanNode for NewProp {
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
        visitor.visit_new_prop(self)?;
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
}

/// 创建新边计划节点
#[derive(Debug)]
pub struct NewEdge {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Box<dyn BasePlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub edge_type_id: i32,
    pub props: Vec<(String, String)>, // 属性名和值
}

impl NewEdge {
    pub fn new(id: i64, edge_type_id: i32, props: Vec<(String, String)>) -> Self {
        Self {
            id,
            kind: PlanNodeKind::NewEdge,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            edge_type_id,
            props,
        }
    }
}

impl Clone for NewEdge {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            edge_type_id: self.edge_type_id,
            props: self.props.clone(),
        }
    }
}

impl BasePlanNode for NewEdge {
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
        visitor.visit_new_edge(self)?;
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
}
