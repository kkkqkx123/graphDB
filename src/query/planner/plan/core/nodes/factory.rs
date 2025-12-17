//! 节点工厂实现
//! 
//! 提供统一的节点创建接口

use super::traits::PlanNode;
use super::filter_node::FilterNode;
use super::project_node::ProjectNode;
use super::join_node::InnerJoinNode;
use super::start_node::StartNode;
use super::placeholder_node::PlaceholderNode;
use crate::query::validator::YieldColumn;
use crate::query::parser::ast::expr::Expr;
use std::sync::Arc;

/// 节点工厂
/// 
/// 提供统一的节点创建接口，简化节点创建过程
pub struct PlanNodeFactory;

impl PlanNodeFactory {
    /// 创建过滤节点
    pub fn create_filter(
        input: Arc<dyn PlanNode>,
        condition: Expr,
    ) -> Result<Arc<dyn PlanNode>, crate::query::planner::planner::PlannerError> {
        Ok(Arc::new(FilterNode::new(input, condition)?))
    }
    
    /// 创建投影节点
    pub fn create_project(
        input: Arc<dyn PlanNode>,
        columns: Vec<YieldColumn>,
    ) -> Result<Arc<dyn PlanNode>, crate::query::planner::planner::PlannerError> {
        Ok(Arc::new(ProjectNode::new(input, columns)?))
    }
    
    /// 创建内连接节点
    pub fn create_inner_join(
        left: Arc<dyn PlanNode>,
        right: Arc<dyn PlanNode>,
        hash_keys: Vec<Expr>,
        probe_keys: Vec<Expr>,
    ) -> Result<Arc<dyn PlanNode>, crate::query::planner::planner::PlannerError> {
        Ok(Arc::new(InnerJoinNode::new(left, right, hash_keys, probe_keys)?))
    }
    
    /// 创建起始节点
    pub fn create_start_node() -> Result<Arc<dyn PlanNode>, crate::query::planner::planner::PlannerError> {
        Ok(Arc::new(StartNode::new()))
    }
    
    /// 创建占位符节点
    pub fn create_placeholder_node() -> Result<Arc<dyn PlanNode>, crate::query::planner::planner::PlannerError> {
        Ok(Arc::new(PlaceholderNode::new()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::parser::ast::expr::{Expr, VariableExpr};
    use crate::query::parser::ast::types::Span;
    
    #[test]
    fn test_create_filter_node() {
        let start_node = PlanNodeFactory::create_start_node().unwrap();
        let condition = Expr::Variable(VariableExpr::new("test".to_string(), Span::default()));
        let filter_node = PlanNodeFactory::create_filter(start_node, condition).unwrap();
        
        assert_eq!(filter_node.kind(), crate::query::planner::plan::core::plan_node_kind::PlanNodeKind::Filter);
        assert_eq!(filter_node.dependencies().len(), 1);
    }
    
    #[test]
    fn test_create_project_node() {
        let start_node = PlanNodeFactory::create_start_node().unwrap();
        let columns = vec![YieldColumn {
            expr: Expr::Variable(VariableExpr::new("test".to_string(), Span::default())),
            alias: "test".to_string(),
        }];
        let project_node = PlanNodeFactory::create_project(start_node, columns).unwrap();
        
        assert_eq!(project_node.kind(), crate::query::planner::plan::core::plan_node_kind::PlanNodeKind::Project);
        assert_eq!(project_node.dependencies().len(), 1);
        assert_eq!(project_node.col_names().len(), 1);
        assert_eq!(project_node.col_names()[0], "test");
    }
    
    #[test]
    fn test_create_inner_join_node() {
        let left_node = PlanNodeFactory::create_start_node().unwrap();
        let right_node = PlanNodeFactory::create_start_node().unwrap();
        let hash_keys = vec![Expr::Variable(VariableExpr::new("key".to_string(), Span::default()))];
        let probe_keys = vec![Expr::Variable(VariableExpr::new("key".to_string(), Span::default()))];
        
        let join_node = PlanNodeFactory::create_inner_join(
            left_node,
            right_node,
            hash_keys,
            probe_keys,
        ).unwrap();
        
        assert_eq!(join_node.kind(), crate::query::planner::plan::core::plan_node_kind::PlanNodeKind::HashInnerJoin);
        assert_eq!(join_node.dependencies().len(), 2);
    }
    
    #[test]
    fn test_create_start_node() {
        let start_node = PlanNodeFactory::create_start_node().unwrap();
        
        assert_eq!(start_node.kind(), crate::query::planner::plan::core::plan_node_kind::PlanNodeKind::Start);
        assert_eq!(start_node.dependencies().len(), 0);
        assert_eq!(start_node.col_names().len(), 0);
    }
    
    #[test]
    fn test_create_placeholder_node() {
        let placeholder_node = PlanNodeFactory::create_placeholder_node().unwrap();
        
        assert_eq!(placeholder_node.kind(), crate::query::planner::plan::core::plan_node_kind::PlanNodeKind::Argument);
        assert_eq!(placeholder_node.dependencies().len(), 0);
        assert_eq!(placeholder_node.col_names().len(), 0);
    }
}