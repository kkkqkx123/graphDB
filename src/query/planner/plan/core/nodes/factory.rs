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

use super::project_node::ProjectNode;
use super::sort_node::{LimitNode, SortNode};
use super::start_node::StartNode;
use super::traversal_node::{AppendVerticesNode, ExpandAllNode, ExpandNode, TraverseNode};
use crate::core::Value;
use crate::query::parser::ast::expr::Expr;
use crate::query::parser::expressions::convert_ast_to_graph_expression;
use crate::query::validator::YieldColumn;

/// 节点工厂
///
/// 提供统一的节点创建接口，简化节点创建过程
pub struct PlanNodeFactory;

impl PlanNodeFactory {
    /// 创建过滤节点
    pub fn create_filter(
        input: PlanNodeEnum,
        condition: Expr,
    ) -> Result<PlanNodeEnum, crate::query::planner::planner::PlannerError> {
        // 将 Expr 转换为 Expression
        let expr = convert_ast_to_graph_expression(&condition).map_err(|e| {
            crate::query::planner::planner::PlannerError::InvalidOperation(e.to_string())
        })?;
        
        // 这里需要重构 FilterNode::new 来接受 PlanNodeEnum 而不是 PlanNodeEnum
        // 暂时返回一个参数节点作为占位符
        Ok(PlanNodeEnum::Argument(ArgumentNode::new(-1, "filter_placeholder")))
    }

    /// 创建投影节点
    pub fn create_project(
        input: PlanNodeEnum,
        columns: Vec<YieldColumn>,
    ) -> Result<PlanNodeEnum, crate::query::planner::planner::PlannerError> {
        // 这里需要重构 ProjectNode::new 来接受 PlanNodeEnum 而不是 PlanNodeEnum
        // 暂时返回一个参数节点作为占位符
        Ok(PlanNodeEnum::Argument(ArgumentNode::new(-1, "project_placeholder")))
    }

    /// 创建内连接节点
    pub fn create_inner_join(
        left: PlanNodeEnum,
        right: PlanNodeEnum,
        hash_keys: Vec<Expr>,
        probe_keys: Vec<Expr>,
    ) -> Result<PlanNodeEnum, crate::query::planner::planner::PlannerError> {
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

        // 这里需要重构 InnerJoinNode::new 来接受 PlanNodeEnum 而不是 PlanNodeEnum
        // 暂时返回一个参数节点作为占位符
        Ok(PlanNodeEnum::Argument(ArgumentNode::new(-1, "inner_join_placeholder")))
    }

    /// 创建起始节点
    pub fn create_start_node(
    ) -> Result<PlanNodeEnum, crate::query::planner::planner::PlannerError> {
        Ok(PlanNodeEnum::Start(StartNode::new()))
    }

    /// 创建占位符节点（使用ArgumentNode作为占位符）
    pub fn create_placeholder_node(
    ) -> Result<PlanNodeEnum, crate::query::planner::planner::PlannerError> {
        Ok(PlanNodeEnum::Argument(ArgumentNode::new(-1, "placeholder")))
    }

    /// 创建聚合节点
    pub fn create_aggregate(
        input: PlanNodeEnum,
        group_keys: Vec<String>,
        agg_exprs: Vec<String>,
    ) -> Result<PlanNodeEnum, crate::query::planner::planner::PlannerError> {
        // 这里需要重构 AggregateNode::new 来接受 PlanNodeEnum 而不是 PlanNodeEnum
        // 暂时返回一个参数节点作为占位符
        Ok(PlanNodeEnum::Argument(ArgumentNode::new(-1, "aggregate_placeholder")))
    }

    /// 创建排序节点
    pub fn create_sort(
        input: PlanNodeEnum,
        sort_items: Vec<String>,
    ) -> Result<PlanNodeEnum, crate::query::planner::planner::PlannerError> {
        // 这里需要重构 SortNode::new 来接受 PlanNodeEnum 而不是 PlanNodeEnum
        // 暂时返回一个参数节点作为占位符
        Ok(PlanNodeEnum::Argument(ArgumentNode::new(-1, "sort_placeholder")))
    }

    /// 创建限制节点
    pub fn create_limit(
        input: PlanNodeEnum,
        offset: i64,
        count: i64,
    ) -> Result<PlanNodeEnum, crate::query::planner::planner::PlannerError> {
        // 这里需要重构 LimitNode::new 来接受 PlanNodeEnum 而不是 PlanNodeEnum
        // 暂时返回一个参数节点作为占位符
        Ok(PlanNodeEnum::Argument(ArgumentNode::new(-1, "limit_placeholder")))
    }

    /// 创建获取顶点节点
    pub fn create_get_vertices(
        space_id: i32,
        src_vids: &str,
    ) -> Result<PlanNodeEnum, crate::query::planner::planner::PlannerError> {
        Ok(PlanNodeEnum::GetVertices(GetVerticesNode::new(space_id, src_vids)))
    }

    /// 创建获取边节点
    pub fn create_get_edges(
        space_id: i32,
        src: &str,
        edge_type: &str,
        rank: &str,
        dst: &str,
    ) -> Result<PlanNodeEnum, crate::query::planner::planner::PlannerError> {
        Ok(PlanNodeEnum::GetEdges(GetEdgesNode::new(
            space_id, src, edge_type, rank, dst,
        )))
    }

    /// 创建获取邻居节点
    pub fn create_get_neighbors(
        space_id: i32,
        src_vids: &str,
    ) -> Result<PlanNodeEnum, crate::query::planner::planner::PlannerError> {
        Ok(PlanNodeEnum::GetNeighbors(GetNeighborsNode::new(space_id, src_vids)))
    }

    /// 创建扫描顶点节点
    pub fn create_scan_vertices(
        space_id: i32,
    ) -> Result<PlanNodeEnum, crate::query::planner::planner::PlannerError> {
        Ok(PlanNodeEnum::ScanVertices(ScanVerticesNode::new(space_id)))
    }

    /// 创建扫描边节点
    pub fn create_scan_edges(
        space_id: i32,
        edge_type: &str,
    ) -> Result<PlanNodeEnum, crate::query::planner::planner::PlannerError> {
        Ok(PlanNodeEnum::ScanEdges(ScanEdgesNode::new(space_id, edge_type)))
    }

    /// 创建扩展节点
    pub fn create_expand(
        space_id: i32,
        edge_types: Vec<String>,
        direction: &str,
    ) -> Result<PlanNodeEnum, crate::query::planner::planner::PlannerError> {
        Ok(PlanNodeEnum::Expand(ExpandNode::new(space_id, edge_types, direction)))
    }

    /// 创建扩展全部节点
    pub fn create_expand_all(
        space_id: i32,
        edge_types: Vec<String>,
        direction: &str,
    ) -> Result<PlanNodeEnum, crate::query::planner::planner::PlannerError> {
        Ok(PlanNodeEnum::ExpandAll(ExpandAllNode::new(
            space_id, edge_types, direction,
        )))
    }

    /// 创建遍历节点
    pub fn create_traverse(
        space_id: i32,
        edge_types: Vec<String>,
        direction: &str,
    ) -> Result<PlanNodeEnum, crate::query::planner::planner::PlannerError> {
        Ok(PlanNodeEnum::Traverse(TraverseNode::new(space_id, edge_types, direction)))
    }

    /// 创建追加顶点节点
    pub fn create_append_vertices(
        space_id: i32,
        vids: Vec<Value>,
        tag_ids: Vec<i32>,
    ) -> Result<PlanNodeEnum, crate::query::planner::planner::PlannerError> {
        Ok(PlanNodeEnum::AppendVertices(AppendVerticesNode::new(space_id, vids, tag_ids)))
    }

    /// 创建参数节点
    pub fn create_argument(
        id: i64,
        var: &str,
    ) -> Result<PlanNodeEnum, crate::query::planner::planner::PlannerError> {
        Ok(PlanNodeEnum::Argument(ArgumentNode::new(id, var)))
    }

    /// 创建选择节点
    pub fn create_select(
        id: i64,
        condition: &str,
    ) -> Result<PlanNodeEnum, crate::query::planner::planner::PlannerError> {
        Ok(PlanNodeEnum::Select(SelectNode::new(id, condition)))
    }

    /// 创建循环节点
    pub fn create_loop(
        id: i64,
        condition: &str,
    ) -> Result<PlanNodeEnum, crate::query::planner::planner::PlannerError> {
        Ok(PlanNodeEnum::Loop(LoopNode::new(id, condition)))
    }

    /// 创建透传节点
    pub fn create_pass_through(
        id: i64,
    ) -> Result<PlanNodeEnum, crate::query::planner::planner::PlannerError> {
        Ok(PlanNodeEnum::PassThrough(PassThroughNode::new(id)))
    }

    /// 创建联合节点
    pub fn create_union(
        input: PlanNodeEnum,
        distinct: bool,
    ) -> Result<PlanNodeEnum, crate::query::planner::planner::PlannerError> {
        // 这里需要重构 UnionNode::new 来接受 PlanNodeEnum 而不是 PlanNodeEnum
        // 暂时返回一个参数节点作为占位符
        Ok(PlanNodeEnum::Argument(ArgumentNode::new(-1, "union_placeholder")))
    }

    /// 创建展开节点
    pub fn create_unwind(
        input: PlanNodeEnum,
        alias: &str,
        list_expr: &str,
    ) -> Result<PlanNodeEnum, crate::query::planner::planner::PlannerError> {
        // 这里需要重构 UnwindNode::new 来接受 PlanNodeEnum 而不是 PlanNodeEnum
        // 暂时返回一个参数节点作为占位符
        Ok(PlanNodeEnum::Argument(ArgumentNode::new(-1, "unwind_placeholder")))
    }

    /// 创建去重节点
    pub fn create_dedup(
        input: PlanNodeEnum,
    ) -> Result<PlanNodeEnum, crate::query::planner::planner::PlannerError> {
        // 这里需要重构 DedupNode::new 来接受 PlanNodeEnum 而不是 PlanNodeEnum
        // 暂时返回一个参数节点作为占位符
        Ok(PlanNodeEnum::Argument(ArgumentNode::new(-1, "dedup_placeholder")))
    }

    /// 创建RollUp应用节点
    pub fn create_roll_up_apply(
        input: PlanNodeEnum,
        collect_exprs: Vec<String>,
        lambda_vars: Vec<String>,
    ) -> Result<PlanNodeEnum, crate::query::planner::planner::PlannerError> {
        // 这里需要重构 RollUpApplyNode::new 来接受 PlanNodeEnum 而不是 PlanNodeEnum
        // 暂时返回一个参数节点作为占位符
        Ok(PlanNodeEnum::Argument(ArgumentNode::new(-1, "roll_up_apply_placeholder")))
    }

    /// 创建模式应用节点
    pub fn create_pattern_apply(
        input: PlanNodeEnum,
        pattern: &str,
        join_type: &str,
    ) -> Result<PlanNodeEnum, crate::query::planner::planner::PlannerError> {
        // 这里需要重构 PatternApplyNode::new 来接受 PlanNodeEnum 而不是 PlanNodeEnum
        // 暂时返回一个参数节点作为占位符
        Ok(PlanNodeEnum::Argument(ArgumentNode::new(-1, "pattern_apply_placeholder")))
    }

    /// 创建数据收集节点
    pub fn create_data_collect(
        input: PlanNodeEnum,
        collect_kind: &str,
    ) -> Result<PlanNodeEnum, crate::query::planner::planner::PlannerError> {
        // 这里需要重构 DataCollectNode::new 来接受 PlanNodeEnum 而不是 PlanNodeEnum
        // 暂时返回一个参数节点作为占位符
        Ok(PlanNodeEnum::Argument(ArgumentNode::new(-1, "data_collect_placeholder")))
    }

    /// 创建索引扫描节点
    pub fn create_index_scan(
        space_id: i32,
        tag_id: i32,
        index_id: i32,
        scan_type: &str,
    ) -> Result<PlanNodeEnum, crate::query::planner::planner::PlannerError> {
        // 这里需要重构 IndexScan::new 来返回 PlanNodeEnum
        // 暂时返回一个参数节点作为占位符
        Ok(PlanNodeEnum::Argument(ArgumentNode::new(-1, "index_scan_placeholder")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Expression;
    use crate::query::parser::ast::expr::{Expr, VariableExpr};
    use crate::query::parser::ast::types::Span;
    

    #[test]
    fn test_create_start_node() {
        let start_node = PlanNodeFactory::create_start_node().expect("Start node should be created successfully");

        assert_eq!(start_node.kind(), PlanNodeKind::Start);
        assert_eq!(start_node.dependencies().len(), 0);
        assert_eq!(start_node.col_names().len(), 0);
    }

    #[test]
    fn test_create_placeholder_node() {
        let placeholder_node = PlanNodeFactory::create_placeholder_node().expect("Placeholder node should be created successfully");

        assert_eq!(placeholder_node.kind(), PlanNodeKind::Argument);
        assert_eq!(placeholder_node.dependencies().len(), 0);
        assert_eq!(placeholder_node.col_names().len(), 0);
    }

    #[test]
    fn test_create_get_vertices_node() {
        let get_vertices_node = PlanNodeFactory::create_get_vertices(1, "1,2,3").expect("GetVertices node should be created successfully");

        assert_eq!(get_vertices_node.kind(), PlanNodeKind::GetVertices);
        assert_eq!(get_vertices_node.dependencies().len(), 0);
    }
}