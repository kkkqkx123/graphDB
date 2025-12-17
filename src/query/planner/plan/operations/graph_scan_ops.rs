//! 图扫描操作节点
//! 包含获取顶点、边和邻居节点的计划节点

use crate::query::context::validate::types::Variable;
use crate::query::planner::plan::core::common::{EdgeProp, TagProp};
use crate::query::planner::plan::core::{
    plan_node_traits::{
        PlanNode, PlanNodeClonable, PlanNodeDependencies, PlanNodeIdentifiable, PlanNodeMutable,
        PlanNodeProperties, PlanNodeVisitable,
    },
    PlanNodeKind, PlanNodeVisitError, PlanNodeVisitor,
};
use std::sync::Arc;

// 获取顶点的计划节点
#[derive(Debug)]
pub struct GetVertices {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub space_id: i32,
    pub src_ref: crate::graph::expression::Expression, // 源引用
    pub src_vids: String,                              // 源顶点表达式
    pub tag_props: Vec<TagProp>,                       // 标签属性
    pub expr: Option<String>,                          // 过滤表达式
    pub dedup: bool,                                   // 是否去重
    pub limit: Option<i64>,                            // 限制返回的顶点数量
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
            src_ref: crate::graph::expression::Expression::Variable(src_vids.to_string()),
            src_vids: src_vids.to_string(),
            tag_props: Vec::new(),
            expr: None,
            dedup: false,
            limit: None,
        }
    }

    pub fn set_limit(&mut self, limit: i64) {
        self.limit = Some(limit);
    }

    pub fn has_effective_filter(&self) -> bool {
        self.expr.is_some()
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
            src_ref: self.src_ref.clone(),
            src_vids: self.src_vids.clone(),
            tag_props: self.tag_props.clone(),
            expr: self.expr.clone(),
            dedup: self.dedup,
            limit: self.limit,
        }
    }
}

impl PlanNodeIdentifiable for GetVertices {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for GetVertices {
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

impl PlanNodeDependencies for GetVertices {
    fn dependencies(&self) -> &[Arc<dyn PlanNode>] {
        &self.deps
    }

    fn replace_dependencies(&mut self, deps: Vec<Arc<dyn PlanNode>>) {
        self.deps = deps;
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

    fn clear_dependencies(&mut self) {
        self.deps.clear();
    }
}

impl PlanNodeMutable for GetVertices {
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

impl PlanNodeClonable for GetVertices {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }
}

impl PlanNodeVisitable for GetVertices {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_get_vertices(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for GetVertices {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// 获取边的计划节点
#[derive(Debug)]
pub struct GetEdges {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub space_id: i32,
    pub edge_ref: crate::graph::expression::Expression, // 边引用
    pub src: String,                                    // 源顶点
    pub edge_type: String,                              // 边类型
    pub rank: String,                                   // 排名
    pub dst: String,                                    // 目标顶点
    pub edge_props: Vec<EdgeProp>,                      // 边属性
    pub expr: Option<String>,                           // 过滤表达式
    pub dedup: bool,                                    // 是否去重
    pub limit: Option<i64>,                             // 限制返回的边数量
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
            edge_ref: crate::graph::expression::Expression::Variable(format!(
                "{}->{}@{}",
                src, dst, edge_type
            )),
            src: src.to_string(),
            edge_type: edge_type.to_string(),
            rank: rank.to_string(),
            dst: dst.to_string(),
            edge_props: Vec::new(),
            expr: None,
            dedup: false,
            limit: None,
        }
    }

    pub fn set_limit(&mut self, limit: i64) {
        self.limit = Some(limit);
    }

    pub fn has_effective_filter(&self) -> bool {
        self.expr.is_some()
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
            edge_ref: self.edge_ref.clone(),
            src: self.src.clone(),
            edge_type: self.edge_type.clone(),
            rank: self.rank.clone(),
            dst: self.dst.clone(),
            edge_props: self.edge_props.clone(),
            expr: self.expr.clone(),
            dedup: self.dedup,
            limit: self.limit,
        }
    }
}

impl PlanNodeIdentifiable for GetEdges {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for GetEdges {
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

impl PlanNodeDependencies for GetEdges {
    fn dependencies(&self) -> &[Arc<dyn PlanNode>] {
        &self.deps
    }

    fn replace_dependencies(&mut self, deps: Vec<Arc<dyn PlanNode>>) {
        self.deps = deps;
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

    fn clear_dependencies(&mut self) {
        self.deps.clear();
    }
}

impl PlanNodeMutable for GetEdges {
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

impl PlanNodeClonable for GetEdges {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }
}

impl PlanNodeVisitable for GetEdges {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_get_edges(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for GetEdges {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// 获取邻居节点
#[derive(Debug)]
pub struct GetNeighbors {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
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
    pub limit: Option<i64>,        // 限制返回的邻居数量
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
            limit: None,
        }
    }

    pub fn set_limit(&mut self, limit: i64) {
        self.limit = Some(limit);
    }

    pub fn has_effective_filter(&self) -> bool {
        self.expr.is_some()
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
            limit: self.limit,
        }
    }
}

impl PlanNodeIdentifiable for GetNeighbors {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for GetNeighbors {
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

impl PlanNodeDependencies for GetNeighbors {
    fn dependencies(&self) -> &[Arc<dyn PlanNode>] {
        &self.deps
    }

    fn replace_dependencies(&mut self, deps: Vec<Arc<dyn PlanNode>>) {
        self.deps = deps;
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

    fn clear_dependencies(&mut self) {
        self.deps.clear();
    }
}

impl PlanNodeMutable for GetNeighbors {
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

impl PlanNodeClonable for GetNeighbors {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }
}

impl PlanNodeVisitable for GetNeighbors {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_get_neighbors(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for GetNeighbors {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// 扫描顶点节点
#[derive(Debug)]
pub struct ScanVertices {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub space_id: i32,
    pub tag_filter: Option<String>,
    pub vertex_filter: Option<String>,
    pub limit: Option<i64>, // 限制返回的顶点数量
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
            limit: None,
        }
    }

    pub fn set_limit(&mut self, limit: i64) {
        self.limit = Some(limit);
    }

    pub fn has_effective_filter(&self) -> bool {
        self.tag_filter.is_some() || self.vertex_filter.is_some()
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
            limit: self.limit,
        }
    }
}

impl PlanNodeIdentifiable for ScanVertices {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for ScanVertices {
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

impl PlanNodeDependencies for ScanVertices {
    fn dependencies(&self) -> &[Arc<dyn PlanNode>] {
        &self.deps
    }

    fn replace_dependencies(&mut self, deps: Vec<Arc<dyn PlanNode>>) {
        self.deps = deps;
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

    fn clear_dependencies(&mut self) {
        self.deps.clear();
    }
}

impl PlanNodeMutable for ScanVertices {
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

impl PlanNodeClonable for ScanVertices {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }
}

impl PlanNodeVisitable for ScanVertices {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_scan_vertices(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for ScanVertices {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// 扫描边节点
#[derive(Debug)]
pub struct ScanEdges {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
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

impl PlanNodeIdentifiable for ScanEdges {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for ScanEdges {
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

impl PlanNodeDependencies for ScanEdges {
    fn dependencies(&self) -> &[Arc<dyn PlanNode>] {
        &self.deps
    }

    fn replace_dependencies(&mut self, deps: Vec<Arc<dyn PlanNode>>) {
        self.deps = deps;
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

    fn clear_dependencies(&mut self) {
        self.deps.clear();
    }
}

impl PlanNodeMutable for ScanEdges {
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

impl PlanNodeClonable for ScanEdges {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }
}

impl PlanNodeVisitable for ScanEdges {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_plan_node(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for ScanEdges {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
