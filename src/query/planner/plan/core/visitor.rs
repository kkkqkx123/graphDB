use super::plan_node_traits::PlanNode as BasePlanNode;
use super::nodes::{
    FilterNode, ProjectNode, InnerJoinNode, LeftJoinNode, StartNode, PlaceholderNode,
    AggregateNode, SortNode, LimitNode, TopNNode,
    GetVerticesNode, GetEdgesNode, GetNeighborsNode, ScanVerticesNode, ScanEdgesNode,
    ExpandNode, ExpandAllNode, TraverseNode, AppendVerticesNode,
    ArgumentNode, SelectNode, LoopNode, PassThroughNode,
    UnionNode, UnwindNode, DedupNode, RollUpApplyNode, PatternApplyNode, DataCollectNode
};
use crate::query::planner::plan::algorithms::{FulltextIndexScan, IndexScan};
use crate::query::planner::plan::management::dml::{
    DeleteEdges, DeleteTags, DeleteVertices, InsertEdges, InsertVertices, NewEdge, NewProp, NewTag,
    NewVertex, UpdateEdge, UpdateVertex,
};
use std::fmt;

pub trait PlanNodeVisitor: std::fmt::Debug {
    fn pre_visit(&mut self) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_plan_node(&mut self, _node: &dyn BasePlanNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_get_neighbors(&mut self, _node: &GetNeighborsNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_get_vertices(&mut self, _node: &GetVerticesNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_get_edges(&mut self, _node: &GetEdgesNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_traverse(&mut self, _node: &TraverseNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_append_vertices(&mut self, _node: &AppendVerticesNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_filter(&mut self, _node: &FilterNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_project(&mut self, _node: &ProjectNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_union(&mut self, _node: &UnionNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_unwind(&mut self, _node: &UnwindNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_aggregate(&mut self, _node: &AggregateNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_sort(&mut self, _node: &SortNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_limit(&mut self, _node: &LimitNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_topn(&mut self, _node: &TopNNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_data_collect(&mut self, _node: &DataCollectNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_scan_vertices(&mut self, _node: &ScanVerticesNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_index_scan(&mut self, _node: &IndexScan) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_fulltext_index_scan(
        &mut self,
        _node: &FulltextIndexScan,
    ) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_expand(&mut self, _node: &ExpandNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_expand_all(&mut self, _node: &ExpandAllNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_start(&mut self, _node: &StartNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_start_node(&mut self, _node: &StartNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_argument(&mut self, _node: &ArgumentNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_argument_node(&mut self, _node: &ArgumentNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_placeholder(&mut self, _node: &PlaceholderNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_select(&mut self, _node: &SelectNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_loop(&mut self, _node: &LoopNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_pass_through(&mut self, _node: &PassThroughNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_hash_left_join(&mut self, _node: &InnerJoinNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_left_join(&mut self, _node: &LeftJoinNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_hash_inner_join(&mut self, _node: &InnerJoinNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_roll_up_apply(&mut self, _node: &RollUpApplyNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_pattern_apply(&mut self, _node: &PatternApplyNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_dedup(&mut self, _node: &DedupNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_scan_edges(&mut self, _node: &ScanEdgesNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_scan_edges_node(&mut self, _node: &ScanEdgesNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问InsertVertices节点
    fn visit_insert_vertices(&mut self, _node: &InsertVertices) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问InsertEdges节点
    fn visit_insert_edges(&mut self, _node: &InsertEdges) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问UpdateVertex节点
    fn visit_update_vertex(&mut self, _node: &UpdateVertex) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问UpdateEdge节点
    fn visit_update_edge(&mut self, _node: &UpdateEdge) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问DeleteVertices节点
    fn visit_delete_vertices(&mut self, _node: &DeleteVertices) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问DeleteEdges节点
    fn visit_delete_edges(&mut self, _node: &DeleteEdges) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问DeleteTags节点
    fn visit_delete_tags(&mut self, _node: &DeleteTags) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问NewVertex节点
    fn visit_new_vertex(&mut self, _node: &NewVertex) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问NewTag节点
    fn visit_new_tag(&mut self, _node: &NewTag) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问NewProp节点
    fn visit_new_prop(&mut self, _node: &NewProp) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问NewEdge节点
    fn visit_new_edge(&mut self, _node: &NewEdge) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }


    /// 在访问节点后调用的方法
    fn post_visit(&mut self) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }
}

/// 计划节点访问错误
#[derive(Debug, Clone)]
pub enum PlanNodeVisitError {
    /// 访问错误
    VisitError(String),

    /// 遍历错误
    TraversalError(String),

    /// 验证错误
    ValidationError(String),
}

impl fmt::Display for PlanNodeVisitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlanNodeVisitError::VisitError(msg) => write!(f, "访问错误: {}", msg),
            PlanNodeVisitError::TraversalError(msg) => write!(f, "遍历错误: {}", msg),
            PlanNodeVisitError::ValidationError(msg) => write!(f, "验证错误: {}", msg),
        }
    }
}

impl std::error::Error for PlanNodeVisitError {}

/// 具体的计划节点访问者实现示例
#[derive(Debug)]
pub struct DefaultPlanNodeVisitor;

impl PlanNodeVisitor for DefaultPlanNodeVisitor {
    fn pre_visit(&mut self) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_plan_node(&mut self, _node: &dyn BasePlanNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_get_neighbors(&mut self, _node: &GetNeighborsNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_get_vertices(&mut self, _node: &GetVerticesNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_get_edges(&mut self, _node: &GetEdgesNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_traverse(&mut self, _node: &TraverseNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_append_vertices(&mut self, _node: &AppendVerticesNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_filter(&mut self, _node: &FilterNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_project(&mut self, _node: &ProjectNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_union(&mut self, _node: &UnionNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_unwind(&mut self, _node: &UnwindNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_aggregate(&mut self, _node: &AggregateNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_dedup(&mut self, _node: &DedupNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_scan_edges(&mut self, _node: &ScanEdgesNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_scan_edges_node(&mut self, _node: &ScanEdgesNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_roll_up_apply(&mut self, _node: &RollUpApplyNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_pattern_apply(&mut self, _node: &PatternApplyNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_sort(&mut self, _node: &SortNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_limit(&mut self, _node: &LimitNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_topn(&mut self, _node: &TopNNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_data_collect(&mut self, _node: &DataCollectNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_scan_vertices(&mut self, _node: &ScanVerticesNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_index_scan(&mut self, _node: &IndexScan) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_fulltext_index_scan(
        &mut self,
        _node: &FulltextIndexScan,
    ) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_expand(&mut self, _node: &ExpandNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_expand_all(&mut self, _node: &ExpandAllNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_start(&mut self, _node: &StartNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_start_node(&mut self, _node: &StartNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_argument(&mut self, _node: &ArgumentNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_argument_node(&mut self, _node: &ArgumentNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_placeholder(&mut self, _node: &PlaceholderNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_select(&mut self, _node: &SelectNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_loop(&mut self, _node: &LoopNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_pass_through(&mut self, _node: &PassThroughNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_hash_left_join(&mut self, _node: &InnerJoinNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_left_join(&mut self, _node: &LeftJoinNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_hash_inner_join(&mut self, _node: &InnerJoinNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_insert_vertices(&mut self, _node: &InsertVertices) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_insert_edges(&mut self, _node: &InsertEdges) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_update_vertex(&mut self, _node: &UpdateVertex) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_update_edge(&mut self, _node: &UpdateEdge) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_delete_vertices(&mut self, _node: &DeleteVertices) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_delete_edges(&mut self, _node: &DeleteEdges) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_delete_tags(&mut self, _node: &DeleteTags) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_new_vertex(&mut self, _node: &NewVertex) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_new_tag(&mut self, _node: &NewTag) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_new_prop(&mut self, _node: &NewProp) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_new_edge(&mut self, _node: &NewEdge) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }
    

    fn post_visit(&mut self) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }
}
