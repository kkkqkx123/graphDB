//! 图遍历操作节点
//! 包含Expand、ExpandAll、Traverse等图遍历相关的计划节点

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

// 扩展节点
#[derive(Debug)]
pub struct Expand {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub space_id: i32,
    pub edge_types: Vec<String>, // 边类型
    pub direction: String,       // 方向 (IN/OUT/BOTH)
    pub step_limit: Option<u32>, // 步数限制
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

impl PlanNodeIdentifiable for Expand {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for Expand {
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

impl PlanNodeDependencies for Expand {
    fn dependencies(&self) -> &[Arc<dyn PlanNode>] {
        &self.deps
    }
    
    fn dependency_count(&self) -> usize {
        self.deps.len()
    }
    
    fn has_dependency(&self, id: i64) -> bool {
        self.deps.iter().any(|dep| dep.id() == id)
    }
    
    fn add_dependency(&mut self, dep: Arc<dyn PlanNode>) {
        self.deps.push(dep);
    }
}

impl PlanNodeMutable for Expand {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for Expand {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }
    
    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for Expand {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_expand(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for Expand {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// 扩展全部节点
#[derive(Debug)]
pub struct ExpandAll {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub space_id: i32,
    pub edge_types: Vec<String>,    // 边类型
    pub direction: String,          // 方向 (IN/OUT/BOTH)
    pub step_limit: Option<u32>,    // 步数限制
    pub edge_props: Vec<EdgeProp>,  // 边属性
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

impl PlanNodeIdentifiable for ExpandAll {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for ExpandAll {
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

impl PlanNodeDependencies for ExpandAll {
    fn dependencies(&self) -> &[Arc<dyn PlanNode>] {
        &self.deps
    }
    
    fn dependency_count(&self) -> usize {
        self.deps.len()
    }
    
    fn has_dependency(&self, id: i64) -> bool {
        self.deps.iter().any(|dep| dep.id() == id)
    }
    
    fn add_dependency(&mut self, dep: Arc<dyn PlanNode>) {
        self.deps.push(dep);
    }
}

impl PlanNodeMutable for ExpandAll {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for ExpandAll {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }
    
    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for ExpandAll {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_expand_all(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for ExpandAll {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// 遍历节点
#[derive(Debug)]
pub struct Traverse {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub space_id: i32,
    pub edge_types: Vec<String>, // 边类型
    pub direction: String,       // 方向 (IN/OUT/BOTH)
    pub step_limit: Option<u32>, // 步数限制
    pub filter: Option<String>,  // 过滤条件
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

impl PlanNodeIdentifiable for Traverse {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for Traverse {
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

impl PlanNodeDependencies for Traverse {
    fn dependencies(&self) -> &[Arc<dyn PlanNode>] {
        &self.deps
    }
    
    fn dependency_count(&self) -> usize {
        self.deps.len()
    }
    
    fn has_dependency(&self, id: i64) -> bool {
        self.deps.iter().any(|dep| dep.id() == id)
    }
    
    fn add_dependency(&mut self, dep: Arc<dyn PlanNode>) {
        self.deps.push(dep);
    }
}

impl PlanNodeMutable for Traverse {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for Traverse {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }
    
    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for Traverse {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_traverse(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for Traverse {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// 追加顶点节点
#[derive(Debug)]
pub struct AppendVertices {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub space_id: i32,
    pub vids: Vec<crate::core::Value>, // 顶点ID列表
    pub tag_ids: Vec<i32>,             // 标签ID列表
    pub filter: Option<String>,        // 过滤条件
}

impl AppendVertices {
    pub fn new(id: i64, space_id: i32, vids: Vec<crate::core::Value>, tag_ids: Vec<i32>) -> Self {
        Self {
            id,
            kind: PlanNodeKind::AppendVertices,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            space_id,
            vids,
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
            vids: self.vids.clone(),
            tag_ids: self.tag_ids.clone(),
            filter: self.filter.clone(),
        }
    }
}

impl PlanNodeIdentifiable for AppendVertices {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for AppendVertices {
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

impl PlanNodeDependencies for AppendVertices {
    fn dependencies(&self) -> &[Arc<dyn PlanNode>] {
        &self.deps
    }
    
    fn dependency_count(&self) -> usize {
        self.deps.len()
    }
    
    fn has_dependency(&self, id: i64) -> bool {
        self.deps.iter().any(|dep| dep.id() == id)
    }
    
    fn add_dependency(&mut self, dep: Arc<dyn PlanNode>) {
        self.deps.push(dep);
    }
}

impl PlanNodeMutable for AppendVertices {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for AppendVertices {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }
    
    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for AppendVertices {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_append_vertices(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for AppendVertices {
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
    pub props: Vec<String>, // 边属性
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
    
    fn dependency_count(&self) -> usize {
        self.deps.len()
    }
    
    fn has_dependency(&self, id: i64) -> bool {
        self.deps.iter().any(|dep| dep.id() == id)
    }
    
    fn add_dependency(&mut self, dep: Arc<dyn PlanNode>) {
        self.deps.push(dep);
    }
}

impl PlanNodeMutable for ScanEdges {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for ScanEdges {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }
    
    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for ScanEdges {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_scan_edges(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for ScanEdges {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
