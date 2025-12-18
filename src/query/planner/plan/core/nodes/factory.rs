//! 节点工厂实现
//!
//! 提供统一的节点创建接口

use super::aggregate_node::AggregateNode;
use super::control_flow_node::{ArgumentNode, LoopNode, PassThroughNode, SelectNode};
use super::data_processing_node::{
    DataCollectNode, DedupNode, PatternApplyNode, RollUpApplyNode, UnionNode, UnwindNode,
};
use super::filter_node::FilterNode;
use super::graph_scan_node::{
    GetEdgesNode, GetNeighborsNode, GetVerticesNode, ScanEdgesNode, ScanVerticesNode,
};
use super::join_node::InnerJoinNode;
use super::placeholder_node::PlaceholderNode;
use super::project_node::ProjectNode;
use super::sort_node::{LimitNode, SortNode};
use super::start_node::StartNode;
use super::traits::PlanNode;
use super::traversal_node::{AppendVerticesNode, ExpandAllNode, ExpandNode, TraverseNode};
use crate::core::Value;
use crate::query::parser::ast::expr::Expr;
use crate::query::parser::expressions::convert_ast_to_graph_expression;
use crate::query::planner::plan::PlanNodeKind;
use crate::query::validator::YieldColumn;
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
        // 将 Expr 转换为 Expression
        let expr = convert_ast_to_graph_expression(&condition).map_err(|e| {
            crate::query::planner::planner::PlannerError::InvalidOperation(e.to_string())
        })?;
        Ok(Arc::new(FilterNode::new(input, expr)?))
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
        // 将 Expr 转换为 Expression
        let hash_keys_expr: Result<Vec<_>, _> = hash_keys
            .iter()
            .map(|e| convert_ast_to_graph_expression(e))
            .collect();
        let hash_keys_expr = hash_keys_expr.map_err(|e| {
            crate::query::planner::planner::PlannerError::InvalidOperation(e.to_string())
        })?;

        let probe_keys_expr: Result<Vec<_>, _> = probe_keys
            .iter()
            .map(|e| convert_ast_to_graph_expression(e))
            .collect();
        let probe_keys_expr = probe_keys_expr.map_err(|e| {
            crate::query::planner::planner::PlannerError::InvalidOperation(e.to_string())
        })?;

        Ok(Arc::new(InnerJoinNode::new(
            left,
            right,
            hash_keys_expr,
            probe_keys_expr,
        )?))
    }

    /// 创建起始节点
    pub fn create_start_node(
    ) -> Result<Arc<dyn PlanNode>, crate::query::planner::planner::PlannerError> {
        Ok(Arc::new(StartNode::new()))
    }

    /// 创建占位符节点
    pub fn create_placeholder_node(
    ) -> Result<Arc<dyn PlanNode>, crate::query::planner::planner::PlannerError> {
        Ok(Arc::new(PlaceholderNode::new()))
    }

    /// 创建聚合节点
    pub fn create_aggregate(
        input: Arc<dyn PlanNode>,
        group_keys: Vec<String>,
        agg_exprs: Vec<String>,
    ) -> Result<Arc<dyn PlanNode>, crate::query::planner::planner::PlannerError> {
        Ok(Arc::new(AggregateNode::new(input, group_keys, agg_exprs)?))
    }

    /// 创建排序节点
    pub fn create_sort(
        input: Arc<dyn PlanNode>,
        sort_items: Vec<String>,
    ) -> Result<Arc<dyn PlanNode>, crate::query::planner::planner::PlannerError> {
        Ok(Arc::new(SortNode::new(input, sort_items)?))
    }

    /// 创建限制节点
    pub fn create_limit(
        input: Arc<dyn PlanNode>,
        offset: i64,
        count: i64,
    ) -> Result<Arc<dyn PlanNode>, crate::query::planner::planner::PlannerError> {
        Ok(Arc::new(LimitNode::new(input, offset, count)?))
    }

    /// 创建获取顶点节点
    pub fn create_get_vertices(
        space_id: i32,
        src_vids: &str,
    ) -> Result<Arc<dyn PlanNode>, crate::query::planner::planner::PlannerError> {
        Ok(Arc::new(GetVerticesNode::new(space_id, src_vids)))
    }

    /// 创建获取边节点
    pub fn create_get_edges(
        space_id: i32,
        src: &str,
        edge_type: &str,
        rank: &str,
        dst: &str,
    ) -> Result<Arc<dyn PlanNode>, crate::query::planner::planner::PlannerError> {
        Ok(Arc::new(GetEdgesNode::new(
            space_id, src, edge_type, rank, dst,
        )))
    }

    /// 创建获取邻居节点
    pub fn create_get_neighbors(
        space_id: i32,
        src_vids: &str,
    ) -> Result<Arc<dyn PlanNode>, crate::query::planner::planner::PlannerError> {
        Ok(Arc::new(GetNeighborsNode::new(space_id, src_vids)))
    }

    /// 创建扫描顶点节点
    pub fn create_scan_vertices(
        space_id: i32,
    ) -> Result<Arc<dyn PlanNode>, crate::query::planner::planner::PlannerError> {
        Ok(Arc::new(ScanVerticesNode::new(space_id)))
    }

    /// 创建扫描边节点
    pub fn create_scan_edges(
        space_id: i32,
        edge_type: &str,
    ) -> Result<Arc<dyn PlanNode>, crate::query::planner::planner::PlannerError> {
        Ok(Arc::new(ScanEdgesNode::new(space_id, edge_type)))
    }

    /// 创建扩展节点
    pub fn create_expand(
        space_id: i32,
        edge_types: Vec<String>,
        direction: &str,
    ) -> Result<Arc<dyn PlanNode>, crate::query::planner::planner::PlannerError> {
        Ok(Arc::new(ExpandNode::new(space_id, edge_types, direction)))
    }

    /// 创建扩展全部节点
    pub fn create_expand_all(
        space_id: i32,
        edge_types: Vec<String>,
        direction: &str,
    ) -> Result<Arc<dyn PlanNode>, crate::query::planner::planner::PlannerError> {
        Ok(Arc::new(ExpandAllNode::new(
            space_id, edge_types, direction,
        )))
    }

    /// 创建遍历节点
    pub fn create_traverse(
        space_id: i32,
        edge_types: Vec<String>,
        direction: &str,
    ) -> Result<Arc<dyn PlanNode>, crate::query::planner::planner::PlannerError> {
        Ok(Arc::new(TraverseNode::new(space_id, edge_types, direction)))
    }

    /// 创建追加顶点节点
    pub fn create_append_vertices(
        space_id: i32,
        vids: Vec<Value>,
        tag_ids: Vec<i32>,
    ) -> Result<Arc<dyn PlanNode>, crate::query::planner::planner::PlannerError> {
        Ok(Arc::new(AppendVerticesNode::new(space_id, vids, tag_ids)))
    }

    /// 创建参数节点
    pub fn create_argument(
        id: i64,
        var: &str,
    ) -> Result<Arc<dyn PlanNode>, crate::query::planner::planner::PlannerError> {
        Ok(Arc::new(ArgumentNode::new(id, var)))
    }

    /// 创建选择节点
    pub fn create_select(
        id: i64,
        condition: &str,
    ) -> Result<Arc<dyn PlanNode>, crate::query::planner::planner::PlannerError> {
        Ok(Arc::new(SelectNode::new(id, condition)))
    }

    /// 创建循环节点
    pub fn create_loop(
        id: i64,
        condition: &str,
    ) -> Result<Arc<dyn PlanNode>, crate::query::planner::planner::PlannerError> {
        Ok(Arc::new(LoopNode::new(id, condition)))
    }

    /// 创建透传节点
    pub fn create_pass_through(
        id: i64,
    ) -> Result<Arc<dyn PlanNode>, crate::query::planner::planner::PlannerError> {
        Ok(Arc::new(PassThroughNode::new(id)))
    }

    /// 创建联合节点
    pub fn create_union(
        input: Arc<dyn PlanNode>,
        distinct: bool,
    ) -> Result<Arc<dyn PlanNode>, crate::query::planner::planner::PlannerError> {
        Ok(Arc::new(UnionNode::new(input, distinct)?))
    }

    /// 创建展开节点
    pub fn create_unwind(
        input: Arc<dyn PlanNode>,
        alias: &str,
        list_expr: &str,
    ) -> Result<Arc<dyn PlanNode>, crate::query::planner::planner::PlannerError> {
        Ok(Arc::new(UnwindNode::new(input, alias, list_expr)?))
    }

    /// 创建去重节点
    pub fn create_dedup(
        input: Arc<dyn PlanNode>,
    ) -> Result<Arc<dyn PlanNode>, crate::query::planner::planner::PlannerError> {
        Ok(Arc::new(DedupNode::new(input)?))
    }

    /// 创建RollUp应用节点
    pub fn create_roll_up_apply(
        input: Arc<dyn PlanNode>,
        collect_exprs: Vec<String>,
        lambda_vars: Vec<String>,
    ) -> Result<Arc<dyn PlanNode>, crate::query::planner::planner::PlannerError> {
        Ok(Arc::new(RollUpApplyNode::new(
            input,
            collect_exprs,
            lambda_vars,
        )?))
    }

    /// 创建模式应用节点
    pub fn create_pattern_apply(
        input: Arc<dyn PlanNode>,
        pattern: &str,
        join_type: &str,
    ) -> Result<Arc<dyn PlanNode>, crate::query::planner::planner::PlannerError> {
        Ok(Arc::new(PatternApplyNode::new(input, pattern, join_type)?))
    }

    /// 创建数据收集节点
    pub fn create_data_collect(
        input: Arc<dyn PlanNode>,
        collect_kind: &str,
    ) -> Result<Arc<dyn PlanNode>, crate::query::planner::planner::PlannerError> {
        Ok(Arc::new(DataCollectNode::new(input, collect_kind)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::expression::Expression;
    use crate::query::parser::ast::expr::{Expr, VariableExpr};
    use crate::query::parser::ast::types::Span;

    #[test]
    fn test_create_filter_node() {
        let start_node = PlanNodeFactory::create_start_node().unwrap();
        let condition = Expr::Variable(VariableExpr::new("test".to_string(), Span::default()));
        let filter_node = PlanNodeFactory::create_filter(start_node, condition).unwrap();

        assert_eq!(filter_node.kind(), PlanNodeKind::Filter);
        assert_eq!(filter_node.dependencies().len(), 1);
    }

    #[test]
    fn test_create_project_node() {
        let start_node = PlanNodeFactory::create_start_node().unwrap();
        let columns = vec![YieldColumn {
            expr: Expression::Variable("test".to_string()),
            alias: "test".to_string(),
            is_matched: false,
        }];
        let project_node = PlanNodeFactory::create_project(start_node, columns).unwrap();

        assert_eq!(project_node.kind(), PlanNodeKind::Project);
        assert_eq!(project_node.dependencies().len(), 1);
        assert_eq!(project_node.col_names().len(), 1);
        assert_eq!(project_node.col_names()[0], "test");
    }

    #[test]
    fn test_create_inner_join_node() {
        let left_node = PlanNodeFactory::create_start_node().unwrap();
        let right_node = PlanNodeFactory::create_start_node().unwrap();
        let hash_keys = vec![Expr::Variable(VariableExpr::new(
            "key".to_string(),
            Span::default(),
        ))];
        let probe_keys = vec![Expr::Variable(VariableExpr::new(
            "key".to_string(),
            Span::default(),
        ))];

        let join_node =
            PlanNodeFactory::create_inner_join(left_node, right_node, hash_keys, probe_keys)
                .unwrap();

        assert_eq!(join_node.kind(), PlanNodeKind::HashInnerJoin);
        assert_eq!(join_node.dependencies().len(), 2);
    }

    #[test]
    fn test_create_start_node() {
        let start_node = PlanNodeFactory::create_start_node().unwrap();

        assert_eq!(start_node.kind(), PlanNodeKind::Start);
        assert_eq!(start_node.dependencies().len(), 0);
        assert_eq!(start_node.col_names().len(), 0);
    }

    #[test]
    fn test_create_placeholder_node() {
        let placeholder_node = PlanNodeFactory::create_placeholder_node().unwrap();

        assert_eq!(placeholder_node.kind(), PlanNodeKind::Argument);
        assert_eq!(placeholder_node.dependencies().len(), 0);
        assert_eq!(placeholder_node.col_names().len(), 0);
    }

    #[test]
    fn test_create_aggregate_node() {
        let start_node = PlanNodeFactory::create_start_node().unwrap();
        let group_keys = vec!["category".to_string()];
        let agg_exprs = vec!["COUNT(*)".to_string()];

        let aggregate_node =
            PlanNodeFactory::create_aggregate(start_node, group_keys, agg_exprs).unwrap();

        assert_eq!(aggregate_node.kind(), PlanNodeKind::Aggregate);
        assert_eq!(aggregate_node.dependencies().len(), 1);
        // Note: group_keys and agg_exprs methods are not available in the PlanNode trait
        // These would need to be accessed through downcasting if needed
    }

    #[test]
    fn test_create_sort_node() {
        let start_node = PlanNodeFactory::create_start_node().unwrap();
        let sort_items = vec!["name".to_string(), "age".to_string()];

        let sort_node = PlanNodeFactory::create_sort(start_node, sort_items).unwrap();

        assert_eq!(sort_node.kind(), PlanNodeKind::Sort);
        assert_eq!(sort_node.dependencies().len(), 1);
        // Note: sort_items method is not available in the PlanNode trait
        // This would need to be accessed through downcasting if needed
    }

    #[test]
    fn test_create_limit_node() {
        let start_node = PlanNodeFactory::create_start_node().unwrap();

        let limit_node = PlanNodeFactory::create_limit(start_node, 10, 100).unwrap();

        assert_eq!(limit_node.kind(), PlanNodeKind::Limit);
        assert_eq!(limit_node.dependencies().len(), 1);
        // Note: offset and count methods are not available in the PlanNode trait
        // These would need to be accessed through downcasting if needed
    }
}
