//! 图扫描节点实现
//!
//! 包含获取顶点、边和邻居节点的计划节点

use super::super::plan_node_kind::PlanNodeKind;
use super::super::common::{EdgeProp, TagProp};
use super::traits::{
    PlanNode, PlanNodeClonable, PlanNodeDependencies, PlanNodeIdentifiable,
    PlanNodeMutable, PlanNodeProperties, PlanNodeVisitable
};
use super::super::visitor::{PlanNodeVisitError, PlanNodeVisitor};
use crate::query::context::validate::types::Variable;
use crate::graph::expression::Expression;
use std::sync::Arc;

/// 获取顶点节点
#[derive(Debug, Clone)]
pub struct GetVerticesNode {
    id: i64,
    space_id: i32,
    src_ref: Expression,
    src_vids: String,
    tag_props: Vec<TagProp>,
    expr: Option<String>,
    dedup: bool,
    limit: Option<i64>,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
}

impl GetVerticesNode {
    pub fn new(space_id: i32, src_vids: &str) -> Self {
        Self {
            id: -1,
            space_id,
            src_ref: Expression::Variable(src_vids.to_string()),
            src_vids: src_vids.to_string(),
            tag_props: Vec::new(),
            expr: None,
            dedup: false,
            limit: None,
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
        }
    }

    pub fn set_limit(&mut self, limit: i64) {
        self.limit = Some(limit);
    }

    pub fn has_effective_filter(&self) -> bool {
        self.expr.is_some()
    }

    /// 获取空间ID
    pub fn space_id(&self) -> i32 {
        self.space_id
    }

    /// 获取源顶点ID
    pub fn src_vids(&self) -> &str {
        &self.src_vids
    }

    /// 设置标签属性
    pub fn set_tag_props(&mut self, tag_props: Vec<TagProp>) {
        self.tag_props = tag_props;
    }
}

impl PlanNodeIdentifiable for GetVerticesNode {
    fn id(&self) -> i64 { self.id }
    fn kind(&self) -> PlanNodeKind { PlanNodeKind::GetVertices }
}

impl PlanNodeProperties for GetVerticesNode {
    fn output_var(&self) -> Option<&Variable> { self.output_var.as_ref() }
    fn col_names(&self) -> &[String] { &self.col_names }
    fn cost(&self) -> f64 { self.cost }
}

impl PlanNodeDependencies for GetVerticesNode {
     fn dependencies(&self) -> &[Arc<dyn PlanNode>] { &[] }
     fn dependencies_mut(&mut self) -> &mut Vec<Arc<dyn PlanNode>> {
         static mut EMPTY: Vec<Arc<dyn PlanNode>> = Vec::new();
         unsafe { &mut EMPTY }
     }
     fn add_dependency(&mut self, _dep: Arc<dyn PlanNode>) {}
     fn remove_dependency(&mut self, _id: i64) -> bool { false }
 }

impl PlanNodeMutable for GetVerticesNode {
    fn set_output_var(&mut self, var: Variable) { self.output_var = Some(var); }
    fn set_col_names(&mut self, names: Vec<String>) { self.col_names = names; }
}

impl PlanNodeClonable for GetVerticesNode {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }
    
    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for GetVerticesNode {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_get_vertices(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for GetVerticesNode {
    fn as_any(&self) -> &dyn std::any::Any { self }
}

/// 获取边节点
#[derive(Debug, Clone)]
pub struct GetEdgesNode {
    id: i64,
    space_id: i32,
    edge_ref: Expression,
    src: String,
    edge_type: String,
    rank: String,
    dst: String,
    edge_props: Vec<EdgeProp>,
    expr: Option<String>,
    dedup: bool,
    limit: Option<i64>,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
}

impl GetEdgesNode {
    pub fn new(space_id: i32, src: &str, edge_type: &str, rank: &str, dst: &str) -> Self {
        Self {
            id: -1,
            space_id,
            edge_ref: Expression::Variable(format!("{}->{}@{}", src, dst, edge_type)),
            src: src.to_string(),
            edge_type: edge_type.to_string(),
            rank: rank.to_string(),
            dst: dst.to_string(),
            edge_props: Vec::new(),
            expr: None,
            dedup: false,
            limit: None,
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
        }
    }

    pub fn set_limit(&mut self, limit: i64) {
        self.limit = Some(limit);
    }

    pub fn has_effective_filter(&self) -> bool {
        self.expr.is_some()
    }

    /// 获取空间ID
    pub fn space_id(&self) -> i32 {
        self.space_id
    }

    /// 获取源顶点
    pub fn src(&self) -> &str {
        &self.src
    }

    /// 获取边类型
    pub fn edge_type(&self) -> &str {
        &self.edge_type
    }

    /// 获取排名
    pub fn rank(&self) -> &str {
        &self.rank
    }

    /// 获取目标顶点
    pub fn dst(&self) -> &str {
        &self.dst
    }
}

impl PlanNodeIdentifiable for GetEdgesNode {
    fn id(&self) -> i64 { self.id }
    fn kind(&self) -> PlanNodeKind { PlanNodeKind::GetEdges }
}

impl PlanNodeProperties for GetEdgesNode {
    fn output_var(&self) -> Option<&Variable> { self.output_var.as_ref() }
    fn col_names(&self) -> &[String] { &self.col_names }
    fn cost(&self) -> f64 { self.cost }
}

impl PlanNodeDependencies for GetEdgesNode {
     fn dependencies(&self) -> &[Arc<dyn PlanNode>] { &[] }
     fn dependencies_mut(&mut self) -> &mut Vec<Arc<dyn PlanNode>> {
         static mut EMPTY: Vec<Arc<dyn PlanNode>> = Vec::new();
         unsafe { &mut EMPTY }
     }
     fn add_dependency(&mut self, _dep: Arc<dyn PlanNode>) {}
     fn remove_dependency(&mut self, _id: i64) -> bool { false }
 }

impl PlanNodeMutable for GetEdgesNode {
    fn set_output_var(&mut self, var: Variable) { self.output_var = Some(var); }
    fn set_col_names(&mut self, names: Vec<String>) { self.col_names = names; }
}

impl PlanNodeClonable for GetEdgesNode {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }
    
    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for GetEdgesNode {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_get_edges(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for GetEdgesNode {
    fn as_any(&self) -> &dyn std::any::Any { self }
}

/// 获取邻居节点
#[derive(Debug, Clone)]
pub struct GetNeighborsNode {
    id: i64,
    space_id: i32,
    src_vids: String,
    edge_types: Vec<String>,
    tag_props: Vec<TagProp>,
    edge_props: Vec<EdgeProp>,
    expr: Option<String>,
    dedup: bool,
    limit: Option<i64>,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
}

impl GetNeighborsNode {
    pub fn new(space_id: i32, src_vids: &str) -> Self {
        Self {
            id: -1,
            space_id,
            src_vids: src_vids.to_string(),
            edge_types: Vec::new(),
            tag_props: Vec::new(),
            edge_props: Vec::new(),
            expr: None,
            dedup: false,
            limit: None,
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
        }
    }

    pub fn set_limit(&mut self, limit: i64) {
        self.limit = Some(limit);
    }

    pub fn has_effective_filter(&self) -> bool {
        self.expr.is_some()
    }
}

impl PlanNodeIdentifiable for GetNeighborsNode {
    fn id(&self) -> i64 { self.id }
    fn kind(&self) -> PlanNodeKind { PlanNodeKind::GetNeighbors }
}

impl PlanNodeProperties for GetNeighborsNode {
    fn output_var(&self) -> Option<&Variable> { self.output_var.as_ref() }
    fn col_names(&self) -> &[String] { &self.col_names }
    fn cost(&self) -> f64 { self.cost }
}

impl PlanNodeDependencies for GetNeighborsNode {
     fn dependencies(&self) -> &[Arc<dyn PlanNode>] { &[] }
     fn dependencies_mut(&mut self) -> &mut Vec<Arc<dyn PlanNode>> {
         static mut EMPTY: Vec<Arc<dyn PlanNode>> = Vec::new();
         unsafe { &mut EMPTY }
     }
     fn add_dependency(&mut self, _dep: Arc<dyn PlanNode>) {}
     fn remove_dependency(&mut self, _id: i64) -> bool { false }
 }

impl PlanNodeMutable for GetNeighborsNode {
    fn set_output_var(&mut self, var: Variable) { self.output_var = Some(var); }
    fn set_col_names(&mut self, names: Vec<String>) { self.col_names = names; }
}

impl PlanNodeClonable for GetNeighborsNode {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }
    
    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for GetNeighborsNode {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_get_neighbors(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for GetNeighborsNode {
    fn as_any(&self) -> &dyn std::any::Any { self }
}

/// 扫描顶点节点
#[derive(Debug, Clone)]
pub struct ScanVerticesNode {
    id: i64,
    space_id: i32,
    tag_filter: Option<String>,
    vertex_filter: Option<String>,
    limit: Option<i64>,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
}

impl ScanVerticesNode {
    pub fn new(space_id: i32) -> Self {
        Self {
            id: -1,
            space_id,
            tag_filter: None,
            vertex_filter: None,
            limit: None,
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
        }
    }

    pub fn set_limit(&mut self, limit: i64) {
        self.limit = Some(limit);
    }

    pub fn has_effective_filter(&self) -> bool {
        self.tag_filter.is_some() || self.vertex_filter.is_some()
    }

    /// 获取空间ID
    pub fn space_id(&self) -> i32 {
        self.space_id
    }

    /// 获取标签过滤器
    pub fn tag_filter(&self) -> &Option<String> {
        &self.tag_filter
    }

    /// 获取顶点过滤器
    pub fn vertex_filter(&self) -> &Option<String> {
        &self.vertex_filter
    }

    /// 获取限制
    pub fn limit(&self) -> Option<i64> {
        self.limit
    }
}

impl PlanNodeIdentifiable for ScanVerticesNode {
    fn id(&self) -> i64 { self.id }
    fn kind(&self) -> PlanNodeKind { PlanNodeKind::ScanVertices }
}

impl PlanNodeProperties for ScanVerticesNode {
    fn output_var(&self) -> Option<&Variable> { self.output_var.as_ref() }
    fn col_names(&self) -> &[String] { &self.col_names }
    fn cost(&self) -> f64 { self.cost }
}

impl PlanNodeDependencies for ScanVerticesNode {
     fn dependencies(&self) -> &[Arc<dyn PlanNode>] { &[] }
     fn dependencies_mut(&mut self) -> &mut Vec<Arc<dyn PlanNode>> {
         static mut EMPTY: Vec<Arc<dyn PlanNode>> = Vec::new();
         unsafe { &mut EMPTY }
     }
     fn add_dependency(&mut self, _dep: Arc<dyn PlanNode>) {}
     fn remove_dependency(&mut self, _id: i64) -> bool { false }
 }

impl PlanNodeMutable for ScanVerticesNode {
    fn set_output_var(&mut self, var: Variable) { self.output_var = Some(var); }
    fn set_col_names(&mut self, names: Vec<String>) { self.col_names = names; }
}

impl PlanNodeClonable for ScanVerticesNode {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }
    
    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for ScanVerticesNode {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_scan_vertices(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for ScanVerticesNode {
    fn as_any(&self) -> &dyn std::any::Any { self }
}

/// 扫描边节点
#[derive(Debug, Clone)]
pub struct ScanEdgesNode {
    id: i64,
    space_id: i32,
    edge_type: String,
    limit: Option<i64>,
    filter: Option<String>,
    props: Vec<EdgeProp>,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
}

impl ScanEdgesNode {
    pub fn new(space_id: i32, edge_type: &str) -> Self {
        Self {
            id: -1,
            space_id,
            edge_type: edge_type.to_string(),
            limit: None,
            filter: None,
            props: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
        }
    }

    pub fn set_limit(&mut self, limit: i64) {
        self.limit = Some(limit);
    }

    pub fn has_effective_filter(&self) -> bool {
        self.filter.is_some()
    }

    /// 获取空间ID
    pub fn space_id(&self) -> i32 {
        self.space_id
    }

    /// 获取边类型
    pub fn edge_type(&self) -> &str {
        &self.edge_type
    }

    /// 获取限制
    pub fn limit(&self) -> Option<i64> {
        self.limit
    }

    /// 获取过滤条件
    pub fn filter(&self) -> Option<&String> {
        self.filter.as_ref()
    }
}

impl PlanNodeIdentifiable for ScanEdgesNode {
    fn id(&self) -> i64 { self.id }
    fn kind(&self) -> PlanNodeKind { PlanNodeKind::ScanEdges }
}

impl PlanNodeProperties for ScanEdgesNode {
    fn output_var(&self) -> Option<&Variable> { self.output_var.as_ref() }
    fn col_names(&self) -> &[String] { &self.col_names }
    fn cost(&self) -> f64 { self.cost }
}

impl PlanNodeDependencies for ScanEdgesNode {
     fn dependencies(&self) -> &[Arc<dyn PlanNode>] { &[] }
     fn dependencies_mut(&mut self) -> &mut Vec<Arc<dyn PlanNode>> {
         static mut EMPTY: Vec<Arc<dyn PlanNode>> = Vec::new();
         unsafe { &mut EMPTY }
     }
     fn add_dependency(&mut self, _dep: Arc<dyn PlanNode>) {}
     fn remove_dependency(&mut self, _id: i64) -> bool { false }
 }

impl PlanNodeMutable for ScanEdgesNode {
    fn set_output_var(&mut self, var: Variable) { self.output_var = Some(var); }
    fn set_col_names(&mut self, names: Vec<String>) { self.col_names = names; }
}

impl PlanNodeClonable for ScanEdgesNode {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }
    
    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for ScanEdgesNode {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_scan_edges(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for ScanEdgesNode {
    fn as_any(&self) -> &dyn std::any::Any { self }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_get_vertices_node_creation() {
        let node = GetVerticesNode::new(1, "vids");
        assert_eq!(node.kind(), PlanNodeKind::GetVertices);
        assert_eq!(node.space_id, 1);
        assert_eq!(node.src_vids, "vids");
    }
    
    #[test]
    fn test_get_edges_node_creation() {
        let node = GetEdgesNode::new(1, "src", "edge", "0", "dst");
        assert_eq!(node.kind(), PlanNodeKind::GetEdges);
        assert_eq!(node.space_id, 1);
        assert_eq!(node.src, "src");
        assert_eq!(node.edge_type, "edge");
    }
    
    #[test]
    fn test_scan_vertices_node_creation() {
        let node = ScanVerticesNode::new(1);
        assert_eq!(node.kind(), PlanNodeKind::ScanVertices);
        assert_eq!(node.space_id, 1);
    }
}