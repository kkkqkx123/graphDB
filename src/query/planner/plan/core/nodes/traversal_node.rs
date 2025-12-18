//! 图遍历节点实现
//!
//! 包含Expand、ExpandAll、Traverse等图遍历相关的计划节点

use super::super::common::{EdgeProp, TagProp};
use super::super::plan_node_kind::PlanNodeKind;
use super::super::visitor::{PlanNodeVisitError, PlanNodeVisitor};
use super::traits::{
    PlanNode, PlanNodeClonable, PlanNodeDependencies, PlanNodeIdentifiable, PlanNodeMutable,
    PlanNodeProperties, PlanNodeVisitable,
};
use crate::core::Value;
use crate::query::context::validate::types::Variable;
use std::sync::Arc;

/// 扩展节点
#[derive(Debug, Clone)]
pub struct ExpandNode {
    id: i64,
    #[allow(dead_code)]
    space_id: i32,
    edge_types: Vec<String>,
    direction: String,
    step_limit: Option<u32>,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
}

impl ExpandNode {
    pub fn new(space_id: i32, edge_types: Vec<String>, direction: &str) -> Self {
        Self {
            id: -1,
            space_id,
            edge_types,
            direction: direction.to_string(),
            step_limit: None,
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
        }
    }
}

impl ExpandNode {
    /// 获取方向
    pub fn direction(&self) -> &str {
        &self.direction
    }

    /// 获取边类型
    pub fn edge_types(&self) -> &[String] {
        &self.edge_types
    }

    /// 获取步数限制
    pub fn step_limit(&self) -> Option<u32> {
        self.step_limit
    }
}

impl PlanNodeIdentifiable for ExpandNode {
    fn id(&self) -> i64 {
        self.id
    }
    fn kind(&self) -> PlanNodeKind {
        PlanNodeKind::Expand
    }
}

impl PlanNodeProperties for ExpandNode {
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

impl PlanNodeDependencies for ExpandNode {
    fn dependencies(&self) -> &[Arc<dyn PlanNode>] {
        &[]
    }
    fn dependencies_mut(&mut self) -> &mut Vec<Arc<dyn PlanNode>> {
        // 返回一个空的可变向量引用
        static mut EMPTY: Vec<Arc<dyn PlanNode>> = Vec::new();
        unsafe { &mut EMPTY }
    }
    fn add_dependency(&mut self, _dep: Arc<dyn PlanNode>) {}
    fn remove_dependency(&mut self, _id: i64) -> bool {
        false
    }
}

impl PlanNodeMutable for ExpandNode {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }
    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for ExpandNode {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for ExpandNode {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_expand(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for ExpandNode {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 扩展全部节点
#[derive(Debug, Clone)]
pub struct ExpandAllNode {
    id: i64,
    #[allow(dead_code)]
    space_id: i32,
    edge_types: Vec<String>,
    direction: String,
    step_limit: Option<u32>,
    #[allow(dead_code)]
    edge_props: Vec<EdgeProp>,
    #[allow(dead_code)]
    vertex_props: Vec<TagProp>,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
}

impl ExpandAllNode {
    pub fn new(space_id: i32, edge_types: Vec<String>, direction: &str) -> Self {
        Self {
            id: -1,
            space_id,
            edge_types,
            direction: direction.to_string(),
            step_limit: None,
            edge_props: Vec::new(),
            vertex_props: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
        }
    }

    /// 获取边类型
    pub fn edge_types(&self) -> &[String] {
        &self.edge_types
    }

    /// 获取方向
    pub fn direction(&self) -> &str {
        &self.direction
    }

    /// 获取步数限制
    pub fn step_limit(&self) -> Option<u32> {
        self.step_limit
    }
}

impl PlanNodeIdentifiable for ExpandAllNode {
    fn id(&self) -> i64 {
        self.id
    }
    fn kind(&self) -> PlanNodeKind {
        PlanNodeKind::ExpandAll
    }
}

impl PlanNodeProperties for ExpandAllNode {
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

impl PlanNodeDependencies for ExpandAllNode {
    fn dependencies(&self) -> &[Arc<dyn PlanNode>] {
        &[]
    }
    fn dependencies_mut(&mut self) -> &mut Vec<Arc<dyn PlanNode>> {
        // 返回一个空的可变向量引用
        static mut EMPTY: Vec<Arc<dyn PlanNode>> = Vec::new();
        unsafe { &mut EMPTY }
    }
    fn add_dependency(&mut self, _dep: Arc<dyn PlanNode>) {}
    fn remove_dependency(&mut self, _id: i64) -> bool {
        false
    }
}

impl PlanNodeMutable for ExpandAllNode {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }
    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for ExpandAllNode {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for ExpandAllNode {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_expand_all(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for ExpandAllNode {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 遍历节点
#[derive(Debug, Clone)]
pub struct TraverseNode {
    id: i64,
    #[allow(dead_code)]
    space_id: i32,
    edge_types: Vec<String>,
    direction: String,
    step_limit: Option<u32>,
    filter: Option<String>,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
}

impl TraverseNode {
    pub fn new(space_id: i32, edge_types: Vec<String>, direction: &str) -> Self {
        Self {
            id: -1,
            space_id,
            edge_types,
            direction: direction.to_string(),
            step_limit: None,
            filter: None,
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
        }
    }

    /// 获取边类型
    pub fn edge_types(&self) -> &[String] {
        &self.edge_types
    }

    /// 获取方向
    pub fn direction(&self) -> &str {
        &self.direction
    }

    /// 获取步数限制
    pub fn step_limit(&self) -> Option<u32> {
        self.step_limit
    }

    /// 获取过滤条件
    pub fn filter(&self) -> Option<&String> {
        self.filter.as_ref()
    }
}

impl PlanNodeIdentifiable for TraverseNode {
    fn id(&self) -> i64 {
        self.id
    }
    fn kind(&self) -> PlanNodeKind {
        PlanNodeKind::Traverse
    }
}

impl PlanNodeProperties for TraverseNode {
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

impl PlanNodeDependencies for TraverseNode {
    fn dependencies(&self) -> &[Arc<dyn PlanNode>] {
        &[]
    }
    fn dependencies_mut(&mut self) -> &mut Vec<Arc<dyn PlanNode>> {
        // 返回一个空的可变向量引用
        static mut EMPTY: Vec<Arc<dyn PlanNode>> = Vec::new();
        unsafe { &mut EMPTY }
    }
    fn add_dependency(&mut self, _dep: Arc<dyn PlanNode>) {}
    fn remove_dependency(&mut self, _id: i64) -> bool {
        false
    }
}

impl PlanNodeMutable for TraverseNode {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }
    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for TraverseNode {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for TraverseNode {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_traverse(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for TraverseNode {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 追加顶点节点
#[derive(Debug, Clone)]
pub struct AppendVerticesNode {
    id: i64,
    space_id: i32,
    vids: Vec<Value>,
    tag_ids: Vec<i32>,
    filter: Option<String>,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
}

impl AppendVerticesNode {
    pub fn new(space_id: i32, vids: Vec<Value>, tag_ids: Vec<i32>) -> Self {
        Self {
            id: -1,
            space_id,
            vids,
            tag_ids,
            filter: None,
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
        }
    }

    /// 获取空间ID
    pub fn space_id(&self) -> i32 {
        self.space_id
    }

    /// 获取顶点ID列表
    pub fn vids(&self) -> &[Value] {
        &self.vids
    }

    /// 获取标签ID列表
    pub fn tag_ids(&self) -> &[i32] {
        &self.tag_ids
    }

    /// 获取过滤条件
    pub fn filter(&self) -> Option<&String> {
        self.filter.as_ref()
    }
}

impl PlanNodeIdentifiable for AppendVerticesNode {
    fn id(&self) -> i64 {
        self.id
    }
    fn kind(&self) -> PlanNodeKind {
        PlanNodeKind::AppendVertices
    }
}

impl PlanNodeProperties for AppendVerticesNode {
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

impl PlanNodeDependencies for AppendVerticesNode {
    fn dependencies(&self) -> &[Arc<dyn PlanNode>] {
        &[]
    }
    fn dependencies_mut(&mut self) -> &mut Vec<Arc<dyn PlanNode>> {
        // 返回一个空的可变向量引用
        static mut EMPTY: Vec<Arc<dyn PlanNode>> = Vec::new();
        unsafe { &mut EMPTY }
    }
    fn add_dependency(&mut self, _dep: Arc<dyn PlanNode>) {}
    fn remove_dependency(&mut self, _id: i64) -> bool {
        false
    }
}

impl PlanNodeMutable for AppendVerticesNode {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }
    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for AppendVerticesNode {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for AppendVerticesNode {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_append_vertices(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for AppendVerticesNode {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_node_creation() {
        let edge_types = vec!["edge1".to_string(), "edge2".to_string()];
        let node = ExpandNode::new(1, edge_types, "OUT");
        assert_eq!(node.kind(), PlanNodeKind::Expand);
        assert_eq!(node.space_id, 1);
        assert_eq!(node.direction, "OUT");
        assert_eq!(node.edge_types.len(), 2);
    }

    #[test]
    fn test_traverse_node_creation() {
        let edge_types = vec!["edge1".to_string()];
        let node = TraverseNode::new(1, edge_types, "BOTH");
        assert_eq!(node.kind(), PlanNodeKind::Traverse);
        assert_eq!(node.space_id, 1);
        assert_eq!(node.direction, "BOTH");
    }

    #[test]
    fn test_append_vertices_node_creation() {
        let vids = vec![Value::String("vid1".to_string())];
        let tag_ids = vec![1, 2];
        let node = AppendVerticesNode::new(1, vids, tag_ids);
        assert_eq!(node.kind(), PlanNodeKind::AppendVertices);
        assert_eq!(node.space_id, 1);
        assert_eq!(node.vids.len(), 1);
        assert_eq!(node.tag_ids.len(), 2);
    }
}
