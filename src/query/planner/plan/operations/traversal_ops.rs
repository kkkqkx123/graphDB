//! 图遍历操作节点
//! 包含Expand、ExpandAll、Traverse等图遍历相关的计划节点

use crate::query::planner::plan::core::{PlanNode as BasePlanNode, PlanNodeKind, PlanNodeVisitor, PlanNodeVisitError};
use crate::query::planner::plan::core::common::{TagProp, EdgeProp};
use crate::query::validator::Variable;

// 扩展节点
#[derive(Debug)]
pub struct Expand {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Box<dyn BasePlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub space_id: i32,
    pub edge_types: Vec<String>,  // 边类型
    pub direction: String,        // 方向 (IN/OUT/BOTH)
    pub step_limit: Option<u32>,  // 步数限制
}

impl Expand {
    pub fn new(id: i64, space_id: i32, edge_types: Vec<String>, direction: &str) -> Self {
        Self {
            id,
            kind: PlanNodeKind::Expand,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            space_id,
            edge_types,
            direction: direction.to_string(),
            step_limit: None,
        }
    }
}

impl Clone for Expand {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            space_id: self.space_id,
            edge_types: self.edge_types.clone(),
            direction: self.direction.clone(),
            step_limit: self.step_limit,
        }
    }
}

impl BasePlanNode for Expand {
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
        visitor.visit_expand(self)?;
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

// 扩展全部节点
#[derive(Debug)]
pub struct ExpandAll {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Box<dyn BasePlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub space_id: i32,
    pub edge_types: Vec<String>,  // 边类型
    pub direction: String,        // 方向 (IN/OUT/BOTH)
    pub step_limit: Option<u32>,  // 步数限制
    pub edge_props: Vec<EdgeProp>, // 边属性
    pub vertex_props: Vec<TagProp>, // 顶点属性
}

impl ExpandAll {
    pub fn new(id: i64, space_id: i32, edge_types: Vec<String>, direction: &str) -> Self {
        Self {
            id,
            kind: PlanNodeKind::ExpandAll,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            space_id,
            edge_types,
            direction: direction.to_string(),
            step_limit: None,
            edge_props: Vec::new(),
            vertex_props: Vec::new(),
        }
    }
}

impl Clone for ExpandAll {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            space_id: self.space_id,
            edge_types: self.edge_types.clone(),
            direction: self.direction.clone(),
            step_limit: self.step_limit,
            edge_props: self.edge_props.clone(),
            vertex_props: self.vertex_props.clone(),
        }
    }
}

impl BasePlanNode for ExpandAll {
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
        visitor.visit_expand_all(self)?;
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

// 遍历节点
#[derive(Debug)]
pub struct Traverse {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Box<dyn BasePlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub space_id: i32,
    pub edge_types: Vec<String>,  // 边类型
    pub direction: String,        // 方向 (IN/OUT/BOTH)
    pub step_limit: Option<u32>,  // 步数限制
    pub filter: Option<String>,   // 过滤条件
}

impl Traverse {
    pub fn new(id: i64, space_id: i32, edge_types: Vec<String>, direction: &str) -> Self {
        Self {
            id,
            kind: PlanNodeKind::Traverse,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            space_id,
            edge_types,
            direction: direction.to_string(),
            step_limit: None,
            filter: None,
        }
    }
}

impl Clone for Traverse {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            space_id: self.space_id,
            edge_types: self.edge_types.clone(),
            direction: self.direction.clone(),
            step_limit: self.step_limit,
            filter: self.filter.clone(),
        }
    }
}

impl BasePlanNode for Traverse {
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
        visitor.visit_traverse(self)?;
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

// 追加顶点节点
#[derive(Debug)]
pub struct AppendVertices {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Box<dyn BasePlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub space_id: i32,
    pub tag_ids: Vec<i32>,  // 标签ID列表
    pub filter: Option<String>,  // 过滤条件
}

impl AppendVertices {
    pub fn new(id: i64, space_id: i32, tag_ids: Vec<i32>) -> Self {
        Self {
            id,
            kind: PlanNodeKind::AppendVertices,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            space_id,
            tag_ids,
            filter: None,
        }
    }
}

impl Clone for AppendVertices {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            space_id: self.space_id,
            tag_ids: self.tag_ids.clone(),
            filter: self.filter.clone(),
        }
    }
}

impl BasePlanNode for AppendVertices {
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
        visitor.visit_append_vertices(self)?;
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

// 扫描边节点
#[derive(Debug)]
pub struct ScanEdges {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Box<dyn BasePlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub space_id: i32,
    pub edge_type: String,
    pub limit: Option<i64>,
    pub filter: Option<String>,
    pub props: Vec<String>,  // 边属性
}

impl ScanEdges {
    pub fn new(id: i64, space_id: i32, edge_type: &str) -> Self {
        Self {
            id,
            kind: PlanNodeKind::ScanEdges,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            space_id,
            edge_type: edge_type.to_string(),
            limit: None,
            filter: None,
            props: Vec::new(),
        }
    }

    pub fn set_limit(&mut self, limit: i64) {
        self.limit = Some(limit);
    }

    pub fn has_effective_filter(&self) -> bool {
        self.filter.is_some()
    }
}

impl Clone for ScanEdges {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            space_id: self.space_id,
            edge_type: self.edge_type.clone(),
            limit: self.limit,
            filter: self.filter.clone(),
            props: self.props.clone(),
        }
    }
}

impl BasePlanNode for ScanEdges {
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
        visitor.visit_scan_edges(self)?;
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