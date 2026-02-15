//! PlanNode 访问者模式实现

use super::plan_node_enum::PlanNodeEnum;
use super::space_nodes::{CreateSpaceNode, DescSpaceNode, DropSpaceNode, ShowSpacesNode, SpaceManageInfo};
use super::tag_nodes::{AlterTagNode, CreateTagNode, DescTagNode, DropTagNode, ShowTagsNode};
use super::edge_nodes::{AlterEdgeNode, CreateEdgeNode, DescEdgeNode, DropEdgeNode, ShowEdgesNode};
use super::index_nodes::{
    CreateEdgeIndexNode, CreateTagIndexNode, DescEdgeIndexNode, DescTagIndexNode,
    DropEdgeIndexNode, DropTagIndexNode, RebuildEdgeIndexNode, RebuildTagIndexNode,
    ShowEdgeIndexesNode, ShowTagIndexesNode,
};
use super::user_nodes::{AlterUserNode, ChangePasswordNode, CreateUserNode, DropUserNode};

pub use super::aggregate_node::AggregateNode;
pub use super::control_flow_node::{ArgumentNode, LoopNode, PassThroughNode, SelectNode};
pub use super::data_processing_node::{
    AssignNode, DataCollectNode, DedupNode, PatternApplyNode, RollUpApplyNode, UnionNode,
    UnwindNode,
};
pub use super::filter_node::FilterNode;
pub use super::graph_scan_node::{
    EdgeIndexScanNode, GetEdgesNode, GetNeighborsNode, GetVerticesNode, ScanEdgesNode, ScanVerticesNode,
};
pub use super::set_operations_node::{IntersectNode, MinusNode};
pub use super::join_node::{
    CrossJoinNode, FullOuterJoinNode, HashInnerJoinNode, HashLeftJoinNode, InnerJoinNode, LeftJoinNode,
};
pub use super::project_node::ProjectNode;
pub use super::sample_node::SampleNode;
pub use super::sort_node::{LimitNode, SortNode, TopNNode};
pub use super::start_node::StartNode;
pub use super::traversal_node::{AppendVerticesNode, ExpandAllNode, ExpandNode, TraverseNode};
pub use crate::query::planner::plan::algorithms::{
    AllPaths, BFSShortest, IndexScan, MultiShortestPath, ShortestPath,
};

/// PlanNode 访问者trait - 使用泛型避免动态分发
pub trait PlanNodeVisitor {
    type Result;

    /// 访问Start节点
    fn visit_start(&mut self, node: &StartNode) -> Self::Result;

    /// 访问Project节点
    fn visit_project(&mut self, node: &ProjectNode) -> Self::Result;

    /// 访问Sort节点
    fn visit_sort(&mut self, node: &SortNode) -> Self::Result;

    /// 访问Limit节点
    fn visit_limit(&mut self, node: &LimitNode) -> Self::Result;

    /// 访问TopN节点
    fn visit_topn(&mut self, node: &TopNNode) -> Self::Result;

    /// 访问Sample节点
    fn visit_sample(&mut self, node: &SampleNode) -> Self::Result;

    /// 访问InnerJoin节点
    fn visit_inner_join(&mut self, node: &InnerJoinNode) -> Self::Result;

    /// 访问LeftJoin节点
    fn visit_left_join(&mut self, node: &LeftJoinNode) -> Self::Result;

    /// 访问CrossJoin节点
    fn visit_cross_join(&mut self, node: &CrossJoinNode) -> Self::Result;

    /// 访问HashInnerJoin节点
    fn visit_hash_inner_join(&mut self, node: &HashInnerJoinNode) -> Self::Result;

    /// 访问HashLeftJoin节点
    fn visit_hash_left_join(&mut self, node: &HashLeftJoinNode) -> Self::Result;

    /// 访问FullOuterJoin节点
    fn visit_full_outer_join(&mut self, node: &FullOuterJoinNode) -> Self::Result;

    /// 访问GetVertices节点
    fn visit_get_vertices(&mut self, node: &GetVerticesNode) -> Self::Result;

    /// 访问GetEdges节点
    fn visit_get_edges(&mut self, node: &GetEdgesNode) -> Self::Result;

    /// 访问GetNeighbors节点
    fn visit_get_neighbors(&mut self, node: &GetNeighborsNode) -> Self::Result;

    /// 访问ScanVertices节点
    fn visit_scan_vertices(&mut self, node: &ScanVerticesNode) -> Self::Result;

    /// 访问ScanEdges节点
    fn visit_scan_edges(&mut self, node: &ScanEdgesNode) -> Self::Result;

    /// 访问EdgeIndexScan节点
    fn visit_edge_index_scan(&mut self, node: &EdgeIndexScanNode) -> Self::Result;

    /// 访问Expand节点
    fn visit_expand(&mut self, node: &ExpandNode) -> Self::Result;

    /// 访问ExpandAll节点
    fn visit_expand_all(&mut self, node: &ExpandAllNode) -> Self::Result;

    /// 访问Traverse节点
    fn visit_traverse(&mut self, node: &TraverseNode) -> Self::Result;

    /// 访问AppendVertices节点
    fn visit_append_vertices(&mut self, node: &AppendVerticesNode) -> Self::Result;

    /// 访问Filter节点
    fn visit_filter(&mut self, node: &FilterNode) -> Self::Result;

    /// 访问Aggregate节点
    fn visit_aggregate(&mut self, node: &AggregateNode) -> Self::Result;

    /// 访问Argument节点
    fn visit_argument(&mut self, node: &ArgumentNode) -> Self::Result;

    /// 访问Loop节点
    fn visit_loop(&mut self, node: &LoopNode) -> Self::Result;

    /// 访问PassThrough节点
    fn visit_pass_through(&mut self, node: &PassThroughNode) -> Self::Result;

    /// 访问Select节点
    fn visit_select(&mut self, node: &SelectNode) -> Self::Result;

    /// 访问DataCollect节点
    fn visit_data_collect(&mut self, node: &DataCollectNode) -> Self::Result;

    /// 访问Dedup节点
    fn visit_dedup(&mut self, node: &DedupNode) -> Self::Result;

    /// 访问PatternApply节点
    fn visit_pattern_apply(&mut self, node: &PatternApplyNode) -> Self::Result;

    /// 访问RollUpApply节点
    fn visit_roll_up_apply(&mut self, node: &RollUpApplyNode) -> Self::Result;

    /// 访问Union节点
    fn visit_union(&mut self, node: &UnionNode) -> Self::Result;

    /// 访问Minus节点
    fn visit_minus(&mut self, node: &MinusNode) -> Self::Result;

    /// 访问Intersect节点
    fn visit_intersect(&mut self, node: &IntersectNode) -> Self::Result;

    /// 访问Unwind节点
    fn visit_unwind(&mut self, node: &UnwindNode) -> Self::Result;

    /// 访问Assign节点
    fn visit_assign(&mut self, node: &AssignNode) -> Self::Result;

    /// 访问IndexScan节点
    fn visit_index_scan(&mut self, node: &IndexScan) -> Self::Result;

    /// 访问MultiShortestPath节点
    fn visit_multi_shortest_path(&mut self, node: &MultiShortestPath) -> Self::Result;

    /// 访问BFSShortest节点
    fn visit_bfs_shortest(&mut self, node: &BFSShortest) -> Self::Result;

    /// 访问AllPaths节点
    fn visit_all_paths(&mut self, node: &AllPaths) -> Self::Result;

    /// 访问ShortestPath节点
    fn visit_shortest_path(&mut self, node: &ShortestPath) -> Self::Result;

    /// 访问CreateSpace节点
    fn visit_create_space(&mut self, node: &CreateSpaceNode) -> Self::Result;

    /// 访问DropSpace节点
    fn visit_drop_space(&mut self, node: &DropSpaceNode) -> Self::Result;

    /// 访问DescSpace节点
    fn visit_desc_space(&mut self, node: &DescSpaceNode) -> Self::Result;

    /// 访问ShowSpaces节点
    fn visit_show_spaces(&mut self, node: &ShowSpacesNode) -> Self::Result;

    /// 访问CreateTag节点
    fn visit_create_tag(&mut self, node: &CreateTagNode) -> Self::Result;

    /// 访问AlterTag节点
    fn visit_alter_tag(&mut self, node: &AlterTagNode) -> Self::Result;

    /// 访问DescTag节点
    fn visit_desc_tag(&mut self, node: &DescTagNode) -> Self::Result;

    /// 访问DropTag节点
    fn visit_drop_tag(&mut self, node: &DropTagNode) -> Self::Result;

    /// 访问ShowTags节点
    fn visit_show_tags(&mut self, node: &ShowTagsNode) -> Self::Result;

    /// 访问CreateEdge节点
    fn visit_create_edge(&mut self, node: &CreateEdgeNode) -> Self::Result;

    /// 访问AlterEdge节点
    fn visit_alter_edge(&mut self, node: &AlterEdgeNode) -> Self::Result;

    /// 访问DescEdge节点
    fn visit_desc_edge(&mut self, node: &DescEdgeNode) -> Self::Result;

    /// 访问DropEdge节点
    fn visit_drop_edge(&mut self, node: &DropEdgeNode) -> Self::Result;

    /// 访问ShowEdges节点
    fn visit_show_edges(&mut self, node: &ShowEdgesNode) -> Self::Result;

    /// 访问CreateTagIndex节点
    fn visit_create_tag_index(&mut self, node: &CreateTagIndexNode) -> Self::Result;

    /// 访问DropTagIndex节点
    fn visit_drop_tag_index(&mut self, node: &DropTagIndexNode) -> Self::Result;

    /// 访问DescTagIndex节点
    fn visit_desc_tag_index(&mut self, node: &DescTagIndexNode) -> Self::Result;

    /// 访问ShowTagIndexes节点
    fn visit_show_tag_indexes(&mut self, node: &ShowTagIndexesNode) -> Self::Result;

    /// 访问CreateEdgeIndex节点
    fn visit_create_edge_index(&mut self, node: &CreateEdgeIndexNode) -> Self::Result;

    /// 访问DropEdgeIndex节点
    fn visit_drop_edge_index(&mut self, node: &DropEdgeIndexNode) -> Self::Result;

    /// 访问DescEdgeIndex节点
    fn visit_desc_edge_index(&mut self, node: &DescEdgeIndexNode) -> Self::Result;

    /// 访问ShowEdgeIndexes节点
    fn visit_show_edge_indexes(&mut self, node: &ShowEdgeIndexesNode) -> Self::Result;

    /// 访问RebuildTagIndex节点
    fn visit_rebuild_tag_index(&mut self, node: &RebuildTagIndexNode) -> Self::Result;

    /// 访问RebuildEdgeIndex节点
    fn visit_rebuild_edge_index(&mut self, node: &RebuildEdgeIndexNode) -> Self::Result;

    /// 访问CreateUser节点
    fn visit_create_user(&mut self, node: &CreateUserNode) -> Self::Result;

    /// 访问AlterUser节点
    fn visit_alter_user(&mut self, node: &AlterUserNode) -> Self::Result;

    /// 访问DropUser节点
    fn visit_drop_user(&mut self, node: &DropUserNode) -> Self::Result;

    /// 访问ChangePassword节点
    fn visit_change_password(&mut self, node: &ChangePasswordNode) -> Self::Result;
}

impl PlanNodeEnum {
    /// 零成本访问者模式
    pub fn accept<V>(&self, visitor: &mut V) -> V::Result
    where
        V: PlanNodeVisitor,
    {
        match self {
            PlanNodeEnum::Start(node) => visitor.visit_start(node),
            PlanNodeEnum::Project(node) => visitor.visit_project(node),
            PlanNodeEnum::Sort(node) => visitor.visit_sort(node),
            PlanNodeEnum::Limit(node) => visitor.visit_limit(node),
            PlanNodeEnum::TopN(node) => visitor.visit_topn(node),
            PlanNodeEnum::Sample(node) => visitor.visit_sample(node),
            PlanNodeEnum::InnerJoin(node) => visitor.visit_inner_join(node),
            PlanNodeEnum::LeftJoin(node) => visitor.visit_left_join(node),
            PlanNodeEnum::CrossJoin(node) => visitor.visit_cross_join(node),
            PlanNodeEnum::GetVertices(node) => visitor.visit_get_vertices(node),
            PlanNodeEnum::GetEdges(node) => visitor.visit_get_edges(node),
            PlanNodeEnum::GetNeighbors(node) => visitor.visit_get_neighbors(node),
            PlanNodeEnum::ScanVertices(node) => visitor.visit_scan_vertices(node),
            PlanNodeEnum::ScanEdges(node) => visitor.visit_scan_edges(node),
            PlanNodeEnum::EdgeIndexScan(node) => visitor.visit_edge_index_scan(node),
            PlanNodeEnum::HashInnerJoin(node) => visitor.visit_hash_inner_join(node),
            PlanNodeEnum::HashLeftJoin(node) => visitor.visit_hash_left_join(node),
            PlanNodeEnum::FullOuterJoin(node) => visitor.visit_full_outer_join(node),
            PlanNodeEnum::Expand(node) => visitor.visit_expand(node),
            PlanNodeEnum::ExpandAll(node) => visitor.visit_expand_all(node),
            PlanNodeEnum::Traverse(node) => visitor.visit_traverse(node),
            PlanNodeEnum::AppendVertices(node) => visitor.visit_append_vertices(node),
            PlanNodeEnum::Filter(node) => visitor.visit_filter(node),
            PlanNodeEnum::Aggregate(node) => visitor.visit_aggregate(node),
            PlanNodeEnum::Argument(node) => visitor.visit_argument(node),
            PlanNodeEnum::Loop(node) => visitor.visit_loop(node),
            PlanNodeEnum::PassThrough(node) => visitor.visit_pass_through(node),
            PlanNodeEnum::Select(node) => visitor.visit_select(node),
            PlanNodeEnum::DataCollect(node) => visitor.visit_data_collect(node),
            PlanNodeEnum::Dedup(node) => visitor.visit_dedup(node),
            PlanNodeEnum::PatternApply(node) => visitor.visit_pattern_apply(node),
            PlanNodeEnum::RollUpApply(node) => visitor.visit_roll_up_apply(node),
            PlanNodeEnum::Union(node) => visitor.visit_union(node),
            PlanNodeEnum::Minus(node) => visitor.visit_minus(node),
            PlanNodeEnum::Intersect(node) => visitor.visit_intersect(node),
            PlanNodeEnum::Unwind(node) => visitor.visit_unwind(node),
            PlanNodeEnum::Assign(node) => visitor.visit_assign(node),
            PlanNodeEnum::IndexScan(node) => visitor.visit_index_scan(node),
            PlanNodeEnum::MultiShortestPath(node) => visitor.visit_multi_shortest_path(node),
            PlanNodeEnum::BFSShortest(node) => visitor.visit_bfs_shortest(node),
            PlanNodeEnum::AllPaths(node) => visitor.visit_all_paths(node),
            PlanNodeEnum::ShortestPath(node) => visitor.visit_shortest_path(node),

            PlanNodeEnum::CreateSpace(node) => visitor.visit_create_space(node),
            PlanNodeEnum::DropSpace(node) => visitor.visit_drop_space(node),
            PlanNodeEnum::DescSpace(node) => visitor.visit_desc_space(node),
            PlanNodeEnum::ShowSpaces(node) => visitor.visit_show_spaces(node),
            PlanNodeEnum::CreateTag(node) => visitor.visit_create_tag(node),
            PlanNodeEnum::AlterTag(node) => visitor.visit_alter_tag(node),
            PlanNodeEnum::DescTag(node) => visitor.visit_desc_tag(node),
            PlanNodeEnum::DropTag(node) => visitor.visit_drop_tag(node),
            PlanNodeEnum::ShowTags(node) => visitor.visit_show_tags(node),
            PlanNodeEnum::CreateEdge(node) => visitor.visit_create_edge(node),
            PlanNodeEnum::AlterEdge(node) => visitor.visit_alter_edge(node),
            PlanNodeEnum::DescEdge(node) => visitor.visit_desc_edge(node),
            PlanNodeEnum::DropEdge(node) => visitor.visit_drop_edge(node),
            PlanNodeEnum::ShowEdges(node) => visitor.visit_show_edges(node),
            PlanNodeEnum::CreateTagIndex(node) => visitor.visit_create_tag_index(node),
            PlanNodeEnum::DropTagIndex(node) => visitor.visit_drop_tag_index(node),
            PlanNodeEnum::DescTagIndex(node) => visitor.visit_desc_tag_index(node),
            PlanNodeEnum::ShowTagIndexes(node) => visitor.visit_show_tag_indexes(node),
            PlanNodeEnum::CreateEdgeIndex(node) => visitor.visit_create_edge_index(node),
            PlanNodeEnum::DropEdgeIndex(node) => visitor.visit_drop_edge_index(node),
            PlanNodeEnum::DescEdgeIndex(node) => visitor.visit_desc_edge_index(node),
            PlanNodeEnum::ShowEdgeIndexes(node) => visitor.visit_show_edge_indexes(node),
            PlanNodeEnum::RebuildTagIndex(node) => visitor.visit_rebuild_tag_index(node),
            PlanNodeEnum::RebuildEdgeIndex(node) => visitor.visit_rebuild_edge_index(node),
            PlanNodeEnum::CreateUser(node) => visitor.visit_create_user(node),
            PlanNodeEnum::AlterUser(node) => visitor.visit_alter_user(node),
            PlanNodeEnum::DropUser(node) => visitor.visit_drop_user(node),
            PlanNodeEnum::ChangePassword(node) => visitor.visit_change_password(node),
            PlanNodeEnum::InsertVertices(_node) => visitor.visit_create_space(&CreateSpaceNode::new(-1, SpaceManageInfo::new("".to_string()))),
            PlanNodeEnum::InsertEdges(_node) => visitor.visit_create_space(&CreateSpaceNode::new(-1, SpaceManageInfo::new("".to_string()))),
        }
    }
}
