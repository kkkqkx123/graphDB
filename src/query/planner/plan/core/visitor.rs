//! PlanNode访问者模式的定义
//! 用于遍历和处理计划树

use super::plan_node_traits::PlanNode as BasePlanNode;
use crate::query::planner::plan::algorithms::{FulltextIndexScan, IndexScan};
use crate::query::planner::plan::management::dml::{
    DeleteEdges, DeleteTags, DeleteVertices, InsertEdges, InsertVertices, NewEdge, NewProp, NewTag,
    NewVertex, UpdateEdge, UpdateVertex,
};
use crate::query::planner::plan::operations::{
    Aggregate, AppendVertices, Argument, ArgumentNode, CrossJoin, DataCollect, Dedup, Expand,
    ExpandAll, Filter, GetEdges, GetNeighbors, GetVertices, HashInnerJoin, HashJoin, HashLeftJoin,
    Limit, PatternApply, Project, RollUpApply, Sample, ScanEdges, ScanVertices, Sort, Start,
    StartNode, TopN, Traverse, Union, Unwind,
};
use std::fmt;

/// 计划节点访问者特征
/// 用于实现访问者模式，遍历和处理计划树
pub trait PlanNodeVisitor: std::fmt::Debug {
    /// 在访问节点前调用的方法
    fn pre_visit(&mut self) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问一般计划节点
    fn visit_plan_node(&mut self, _node: &dyn BasePlanNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问GetNeighbors节点
    fn visit_get_neighbors(&mut self, _node: &GetNeighbors) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问GetVertices节点
    fn visit_get_vertices(&mut self, _node: &GetVertices) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问GetEdges节点
    fn visit_get_edges(&mut self, _node: &GetEdges) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问Traverse节点
    fn visit_traverse(&mut self, _node: &Traverse) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问AppendVertices节点
    fn visit_append_vertices(&mut self, _node: &AppendVertices) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问Filter节点
    fn visit_filter(&mut self, _node: &Filter) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问Project节点
    fn visit_project(&mut self, _node: &Project) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问Union节点
    fn visit_union(&mut self, _node: &Union) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问Unwind节点
    fn visit_unwind(&mut self, _node: &Unwind) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问Aggregate节点
    fn visit_aggregate(&mut self, _node: &Aggregate) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问Sort节点
    fn visit_sort(&mut self, _node: &Sort) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问Limit节点
    fn visit_limit(&mut self, _node: &Limit) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问TopN节点
    fn visit_top_n(&mut self, _node: &TopN) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问Sample节点
    fn visit_sample(&mut self, _node: &Sample) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问DataCollect节点
    fn visit_data_collect(&mut self, _node: &DataCollect) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问ScanVertices节点
    fn visit_scan_vertices(&mut self, _node: &ScanVertices) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问IndexScan节点
    fn visit_index_scan(&mut self, _node: &IndexScan) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问FulltextIndexScan节点
    fn visit_fulltext_index_scan(
        &mut self,
        _node: &FulltextIndexScan,
    ) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问Expand节点
    fn visit_expand(&mut self, _node: &Expand) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问ExpandAll节点
    fn visit_expand_all(&mut self, _node: &ExpandAll) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问Start节点
    fn visit_start(&mut self, _node: &Start) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问StartNode节点
    fn visit_start_node(&mut self, _node: &StartNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问Argument节点
    fn visit_argument(&mut self, _node: &Argument) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问ArgumentNode节点
    fn visit_argument_node(&mut self, _node: &ArgumentNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问HashLeftJoin节点
    fn visit_hash_left_join(&mut self, _node: &HashLeftJoin) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问HashInnerJoin节点
    fn visit_hash_inner_join(&mut self, _node: &HashInnerJoin) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问CrossJoin节点
    fn visit_cross_join(&mut self, _node: &CrossJoin) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问RollUpApply节点
    fn visit_roll_up_apply(&mut self, _node: &RollUpApply) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问PatternApply节点
    fn visit_pattern_apply(&mut self, _node: &PatternApply) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问Dedup节点
    fn visit_dedup(&mut self, _node: &Dedup) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问ScanEdges节点
    fn visit_scan_edges(&mut self, _node: &ScanEdges) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问ScanEdges节点（graph_scan_ops模块）
    fn visit_scan_edges_node(&mut self, _node: &ScanEdges) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问HashJoin节点
    fn visit_hash_join(&mut self, _node: &HashJoin) -> Result<(), PlanNodeVisitError> {
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

    fn visit_get_neighbors(&mut self, _node: &GetNeighbors) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_get_vertices(&mut self, _node: &GetVertices) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_get_edges(&mut self, _node: &GetEdges) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_traverse(&mut self, _node: &Traverse) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_append_vertices(&mut self, _node: &AppendVertices) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_filter(&mut self, _node: &Filter) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_project(&mut self, _node: &Project) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_union(&mut self, _node: &Union) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_unwind(&mut self, _node: &Unwind) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_aggregate(&mut self, _node: &Aggregate) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_dedup(&mut self, _node: &Dedup) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_scan_edges(&mut self, _node: &ScanEdges) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_scan_edges_node(&mut self, _node: &ScanEdges) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_hash_join(&mut self, _node: &HashJoin) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_cross_join(&mut self, _node: &CrossJoin) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_roll_up_apply(&mut self, _node: &RollUpApply) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_pattern_apply(&mut self, _node: &PatternApply) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_sort(&mut self, _node: &Sort) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_limit(&mut self, _node: &Limit) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_top_n(&mut self, _node: &TopN) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_sample(&mut self, _node: &Sample) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_data_collect(&mut self, _node: &DataCollect) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_scan_vertices(&mut self, _node: &ScanVertices) -> Result<(), PlanNodeVisitError> {
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

    fn visit_expand(&mut self, _node: &Expand) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_expand_all(&mut self, _node: &ExpandAll) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_start(&mut self, _node: &Start) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_start_node(&mut self, _node: &StartNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_argument(&mut self, _node: &Argument) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_argument_node(&mut self, _node: &ArgumentNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_hash_left_join(&mut self, _node: &HashLeftJoin) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_hash_inner_join(&mut self, _node: &HashInnerJoin) -> Result<(), PlanNodeVisitError> {
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
