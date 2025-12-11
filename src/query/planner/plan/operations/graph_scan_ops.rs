//! 图扫描操作节点
//! 包含获取顶点、边和邻居节点的计划节点

use crate::query::planner::plan::core::{PlanNode as BasePlanNode, PlanNodeKind, PlanNodeVisitor, PlanNodeVisitError};
use crate::query::planner::plan::core::common::{TagProp, EdgeProp};
use crate::query::validator::Variable;

// 获取顶点的计划节点
#[derive(Debug)]
pub struct GetVertices {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Box<dyn BasePlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub space_id: i32,
    pub src_vids: String,        // 源顶点表达式
    pub tag_props: Vec<TagProp>, // 标签属性
    pub expr: Option<String>,    // 过滤表达式
    pub dedup: bool,             // 是否去重
}

impl GetVertices {
    pub fn new(id: i64, space_id: i32, src_vids: &str) -> Self {
        Self {
            id,
            kind: PlanNodeKind::GetVertices,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            space_id,
            src_vids: src_vids.to_string(),
            tag_props: Vec::new(),
            expr: None,
            dedup: false,
        }
    }
}

impl Clone for GetVertices {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            space_id: self.space_id,
            src_vids: self.src_vids.clone(),
            tag_props: self.tag_props.clone(),
            expr: self.expr.clone(),
            dedup: self.dedup,
        }
    }
}

impl BasePlanNode for GetVertices {
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
        visitor.visit_get_vertices(self)?;
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

// 获取边的计划节点
#[derive(Debug)]
pub struct GetEdges {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Box<dyn BasePlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub space_id: i32,
    pub src: String,               // 源顶点
    pub edge_type: String,         // 边类型
    pub rank: String,              // 排名
    pub dst: String,               // 目标顶点
    pub edge_props: Vec<EdgeProp>, // 边属性
    pub expr: Option<String>,      // 过滤表达式
    pub dedup: bool,               // 是否去重
}

impl GetEdges {
    pub fn new(id: i64, space_id: i32, src: &str, edge_type: &str, rank: &str, dst: &str) -> Self {
        Self {
            id,
            kind: PlanNodeKind::GetEdges,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            space_id,
            src: src.to_string(),
            edge_type: edge_type.to_string(),
            rank: rank.to_string(),
            dst: dst.to_string(),
            edge_props: Vec::new(),
            expr: None,
            dedup: false,
        }
    }
}

impl Clone for GetEdges {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            space_id: self.space_id,
            src: self.src.clone(),
            edge_type: self.edge_type.clone(),
            rank: self.rank.clone(),
            dst: self.dst.clone(),
            edge_props: self.edge_props.clone(),
            expr: self.expr.clone(),
            dedup: self.dedup,
        }
    }
}

impl BasePlanNode for GetEdges {
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
        visitor.visit_get_edges(self)?;
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

// 获取邻居节点
#[derive(Debug)]
pub struct GetNeighbors {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Box<dyn BasePlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub space_id: i32,
    pub src_vids: String,          // 源顶点表达式
    pub edge_types: Vec<String>,   // 边类型
    pub tag_props: Vec<TagProp>,   // 标签属性
    pub edge_props: Vec<EdgeProp>, // 边属性
    pub expr: Option<String>,      // 过滤表达式
    pub dedup: bool,               // 是否去重
}

impl GetNeighbors {
    pub fn new(id: i64, space_id: i32, src_vids: &str) -> Self {
        Self {
            id,
            kind: PlanNodeKind::GetNeighbors,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            space_id,
            src_vids: src_vids.to_string(),
            edge_types: Vec::new(),
            tag_props: Vec::new(),
            edge_props: Vec::new(),
            expr: None,
            dedup: false,
        }
    }
}

impl Clone for GetNeighbors {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            space_id: self.space_id,
            src_vids: self.src_vids.clone(),
            edge_types: self.edge_types.clone(),
            tag_props: self.tag_props.clone(),
            edge_props: self.edge_props.clone(),
            expr: self.expr.clone(),
            dedup: self.dedup,
        }
    }
}

impl BasePlanNode for GetNeighbors {
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
        visitor.visit_get_neighbors(self)?;
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

// 扫描顶点节点
#[derive(Debug)]
pub struct ScanVertices {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Box<dyn BasePlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub space_id: i32,
    pub tag_filter: Option<String>,
    pub vertex_filter: Option<String>,
}

impl ScanVertices {
    pub fn new(id: i64, space_id: i32) -> Self {
        Self {
            id,
            kind: PlanNodeKind::ScanVertices,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            space_id,
            tag_filter: None,
            vertex_filter: None,
        }
    }
}

impl Clone for ScanVertices {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            space_id: self.space_id,
            tag_filter: self.tag_filter.clone(),
            vertex_filter: self.vertex_filter.clone(),
        }
    }
}

impl BasePlanNode for ScanVertices {
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
        visitor.visit_scan_vertices(self)?;
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
    pub props: Vec<EdgeProp>,
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
}