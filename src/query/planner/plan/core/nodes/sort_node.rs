//! 排序节点实现
//!
//! SortNode 用于对输入数据进行排序操作

use super::super::plan_node_kind::PlanNodeKind;
use super::super::visitor::{PlanNodeVisitError, PlanNodeVisitor};
use super::traits::{
    PlanNode, PlanNodeClonable, PlanNodeDependencies, PlanNodeDependenciesExt,
    PlanNodeIdentifiable, PlanNodeMutable, PlanNodeProperties, PlanNodeVisitable,
};
use crate::query::context::validate::types::Variable;
use std::sync::Arc;

/// 排序节点
///
/// 根据指定的排序字段对输入数据进行排序
#[derive(Debug, Clone)]
pub struct SortNode {
    id: i64,
    input: Arc<dyn PlanNode>,
    deps: Vec<Arc<dyn PlanNode>>,
    sort_items: Vec<String>,
    limit: Option<i64>,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
}

impl SortNode {
    /// 创建新的排序节点
    pub fn new(
        input: Arc<dyn PlanNode>,
        sort_items: Vec<String>,
    ) -> Result<Self, crate::query::planner::planner::PlannerError> {
        let col_names = input.col_names().to_vec();
        let mut deps = Vec::new();
        deps.push(input.clone());

        Ok(Self {
            id: -1,
            input,
            deps,
            sort_items,
            limit: None,
            output_var: None,
            col_names,
            cost: 0.0,
        })
    }

    /// 获取排序字段
    pub fn sort_items(&self) -> &[String] {
        &self.sort_items
    }

    /// 获取限制数量
    pub fn limit(&self) -> Option<i64> {
        self.limit
    }

    /// 设置限制数量
    pub fn set_limit(&mut self, limit: i64) {
        self.limit = Some(limit);
    }
}

impl PlanNodeIdentifiable for SortNode {
    fn id(&self) -> i64 {
        self.id
    }
    fn kind(&self) -> PlanNodeKind {
        PlanNodeKind::Sort
    }
}

impl PlanNodeProperties for SortNode {
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

impl PlanNodeDependencies for SortNode {
    fn dependencies(&self) -> Vec<Arc<dyn PlanNode>> {
        self.deps.clone()
    }

    fn add_dependency(&mut self, dep: Arc<dyn PlanNode>) {
        self.input = dep.clone();
        self.deps.clear();
        self.deps.push(dep);
    }

    fn remove_dependency(&mut self, _id: i64) -> bool {
        false
    }
}

impl PlanNodeDependenciesExt for SortNode {
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Arc<dyn PlanNode>]) -> R,
    {
        f(&self.deps)
    }
}

impl PlanNodeMutable for SortNode {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }
    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for SortNode {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(Self {
            id: self.id,
            input: self.input.clone_plan_node(),
            deps: self.deps.clone(),
            sort_items: self.sort_items.clone(),
            limit: self.limit,
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
        })
    }

    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        Arc::new(Self {
            id: new_id,
            input: self.input.clone_plan_node(),
            deps: self.deps.clone(),
            sort_items: self.sort_items.clone(),
            limit: self.limit,
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
        })
    }
}

impl PlanNodeVisitable for SortNode {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_sort(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for SortNode {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 限制节点
///
/// 对输入数据进行分页限制
#[derive(Debug, Clone)]
pub struct LimitNode {
    id: i64,
    input: Arc<dyn PlanNode>,
    deps: Vec<Arc<dyn PlanNode>>,
    offset: i64,
    count: i64,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
}

impl LimitNode {
    /// 创建新的限制节点
    pub fn new(
        input: Arc<dyn PlanNode>,
        offset: i64,
        count: i64,
    ) -> Result<Self, crate::query::planner::planner::PlannerError> {
        let col_names = input.col_names().to_vec();
        let mut deps = Vec::new();
        deps.push(input.clone());

        Ok(Self {
            id: -1,
            input,
            deps,
            offset,
            count,
            output_var: None,
            col_names,
            cost: 0.0,
        })
    }

    /// 获取偏移量
    pub fn offset(&self) -> i64 {
        self.offset
    }

    /// 获取计数
    pub fn count(&self) -> i64 {
        self.count
    }
}

impl PlanNodeIdentifiable for LimitNode {
    fn id(&self) -> i64 {
        self.id
    }
    fn kind(&self) -> PlanNodeKind {
        PlanNodeKind::Limit
    }
}

impl PlanNodeProperties for LimitNode {
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

impl PlanNodeDependencies for LimitNode {
    fn dependencies(&self) -> Vec<Arc<dyn PlanNode>> {
        self.deps.clone()
    }

    fn add_dependency(&mut self, dep: Arc<dyn PlanNode>) {
        self.input = dep.clone();
        self.deps.clear();
        self.deps.push(dep);
    }

    fn remove_dependency(&mut self, _id: i64) -> bool {
        false
    }
}

impl PlanNodeDependenciesExt for LimitNode {
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Arc<dyn PlanNode>]) -> R,
    {
        f(&self.deps)
    }
}

impl PlanNodeMutable for LimitNode {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }
    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for LimitNode {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(Self {
            id: self.id,
            input: self.input.clone_plan_node(),
            deps: self.deps.clone(),
            offset: self.offset,
            count: self.count,
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
        })
    }

    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        Arc::new(Self {
            id: new_id,
            input: self.input.clone_plan_node(),
            deps: self.deps.clone(),
            offset: self.offset,
            count: self.count,
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
        })
    }
}

impl PlanNodeVisitable for LimitNode {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_limit(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for LimitNode {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// TopN节点
///
/// 对输入数据进行排序并返回前N个结果
#[derive(Debug, Clone)]
pub struct TopNNode {
    id: i64,
    input: Arc<dyn PlanNode>,
    deps: Vec<Arc<dyn PlanNode>>,
    sort_items: Vec<String>,
    limit: i64,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
}

impl TopNNode {
    /// 创建新的TopN节点
    pub fn new(
        input: Arc<dyn PlanNode>,
        sort_items: Vec<String>,
        limit: i64,
    ) -> Result<Self, crate::query::planner::planner::PlannerError> {
        let col_names = input.col_names().to_vec();
        let mut deps = Vec::new();
        deps.push(input.clone());

        Ok(Self {
            id: -1,
            input,
            deps,
            sort_items,
            limit,
            output_var: None,
            col_names,
            cost: 0.0,
        })
    }

    /// 获取排序字段
    pub fn sort_items(&self) -> &[String] {
        &self.sort_items
    }

    /// 获取限制数量
    pub fn limit(&self) -> i64 {
        self.limit
    }
}

impl PlanNodeIdentifiable for TopNNode {
    fn id(&self) -> i64 {
        self.id
    }
    fn kind(&self) -> PlanNodeKind {
        PlanNodeKind::TopN
    }
}

impl PlanNodeProperties for TopNNode {
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

impl PlanNodeDependencies for TopNNode {
    fn dependencies(&self) -> Vec<Arc<dyn PlanNode>> {
        self.deps.clone()
    }

    fn add_dependency(&mut self, dep: Arc<dyn PlanNode>) {
        self.input = dep.clone();
        self.deps.clear();
        self.deps.push(dep);
    }

    fn remove_dependency(&mut self, _id: i64) -> bool {
        false
    }
}

impl PlanNodeDependenciesExt for TopNNode {
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Arc<dyn PlanNode>]) -> R,
    {
        f(&self.deps)
    }
}

impl PlanNodeMutable for TopNNode {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }
    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for TopNNode {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(Self {
            id: self.id,
            input: self.input.clone_plan_node(),
            deps: self.deps.clone(),
            sort_items: self.sort_items.clone(),
            limit: self.limit,
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
        })
    }

    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        Arc::new(Self {
            id: new_id,
            input: self.input.clone_plan_node(),
            deps: self.deps.clone(),
            sort_items: self.sort_items.clone(),
            limit: self.limit,
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
        })
    }
}

impl PlanNodeVisitable for TopNNode {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_topn(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for TopNNode {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::planner::plan::core::nodes::start_node::StartNode;

    #[test]
    fn test_sort_node_creation() {
        let start_node = StartNode::new();
        let start_node = Arc::new(start_node);

        let sort_items = vec!["name".to_string(), "age".to_string()];

        let sort_node =
            SortNode::new(start_node, sort_items).expect("SortNode creation should succeed");

        assert_eq!(sort_node.kind(), PlanNodeKind::Sort);
        assert_eq!(sort_node.dependencies().len(), 1);
        assert_eq!(sort_node.sort_items().len(), 2);
    }

    #[test]
    fn test_limit_node_creation() {
        let start_node = StartNode::new();
        let start_node = Arc::new(start_node);

        let limit_node = LimitNode::new(start_node, 10, 100).unwrap();

        assert_eq!(limit_node.kind(), PlanNodeKind::Limit);
        assert_eq!(limit_node.dependencies().len(), 1);
        assert_eq!(limit_node.offset(), 10);
        assert_eq!(limit_node.count(), 100);
    }

    #[test]
    fn test_topn_node_creation() {
        let start_node = StartNode::new();
        let start_node = Arc::new(start_node);

        let sort_items = vec!["name".to_string(), "age".to_string()];
        let topn_node = TopNNode::new(start_node, sort_items, 10).unwrap();

        assert_eq!(topn_node.kind(), PlanNodeKind::TopN);
        assert_eq!(topn_node.dependencies().len(), 1);
        assert_eq!(topn_node.sort_items().len(), 2);
        assert_eq!(topn_node.limit(), 10);
    }
}
