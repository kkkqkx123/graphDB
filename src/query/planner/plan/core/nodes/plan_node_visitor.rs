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

/// 生成 PlanNode 访问者默认方法的宏
macro_rules! impl_visitor_methods {
    ($($name:ident, $node_type:ty, $visit_method:ident);* $(;)?) => {
        $(
            fn $visit_method(&mut self, node: &$node_type) -> Self::Result {
                let _ = node;
                self.visit_default()
            }
        )*
    };
}

/// PlanNode 访问者trait - 使用泛型避免动态分发
pub trait PlanNodeVisitor {
    type Result;

    /// 默认访问方法
    fn visit_default(&mut self) -> Self::Result;

    impl_visitor_methods!(
        Start, StartNode, visit_start;
        Project, ProjectNode, visit_project;
        Sort, SortNode, visit_sort;
        Limit, LimitNode, visit_limit;
        TopN, TopNNode, visit_topn;
        Sample, SampleNode, visit_sample;
    );

    impl_visitor_methods!(
        InnerJoin, InnerJoinNode, visit_inner_join;
        LeftJoin, LeftJoinNode, visit_left_join;
        CrossJoin, CrossJoinNode, visit_cross_join;
        HashInnerJoin, HashInnerJoinNode, visit_hash_inner_join;
        HashLeftJoin, HashLeftJoinNode, visit_hash_left_join;
        FullOuterJoin, FullOuterJoinNode, visit_full_outer_join;
    );

    impl_visitor_methods!(
        GetVertices, GetVerticesNode, visit_get_vertices;
        GetEdges, GetEdgesNode, visit_get_edges;
        GetNeighbors, GetNeighborsNode, visit_get_neighbors;
        ScanVertices, ScanVerticesNode, visit_scan_vertices;
        ScanEdges, ScanEdgesNode, visit_scan_edges;
        EdgeIndexScan, EdgeIndexScanNode, visit_edge_index_scan;
    );

    impl_visitor_methods!(
        Expand, ExpandNode, visit_expand;
        ExpandAll, ExpandAllNode, visit_expand_all;
        Traverse, TraverseNode, visit_traverse;
        AppendVertices, AppendVerticesNode, visit_append_vertices;
    );

    impl_visitor_methods!(
        Filter, FilterNode, visit_filter;
        Aggregate, AggregateNode, visit_aggregate;
        Dedup, DedupNode, visit_dedup;
    );

    impl_visitor_methods!(
        Argument, ArgumentNode, visit_argument;
        Loop, LoopNode, visit_loop;
        PassThrough, PassThroughNode, visit_pass_through;
        Select, SelectNode, visit_select;
        DataCollect, DataCollectNode, visit_data_collect;
    );

    impl_visitor_methods!(
        PatternApply, PatternApplyNode, visit_pattern_apply;
        RollUpApply, RollUpApplyNode, visit_roll_up_apply;
    );

    impl_visitor_methods!(
        Union, UnionNode, visit_union;
        Minus, MinusNode, visit_minus;
        Intersect, IntersectNode, visit_intersect;
        Unwind, UnwindNode, visit_unwind;
        Assign, AssignNode, visit_assign;
    );

    impl_visitor_methods!(
        IndexScan, IndexScan, visit_index_scan;
        MultiShortestPath, MultiShortestPath, visit_multi_shortest_path;
        BFSShortest, BFSShortest, visit_bfs_shortest;
        AllPaths, AllPaths, visit_all_paths;
        ShortestPath, ShortestPath, visit_shortest_path;
    );

    impl_visitor_methods!(
        CreateSpace, CreateSpaceNode, visit_create_space;
        DropSpace, DropSpaceNode, visit_drop_space;
        DescSpace, DescSpaceNode, visit_desc_space;
        ShowSpaces, ShowSpacesNode, visit_show_spaces;
    );

    impl_visitor_methods!(
        CreateTag, CreateTagNode, visit_create_tag;
        AlterTag, AlterTagNode, visit_alter_tag;
        DescTag, DescTagNode, visit_desc_tag;
        DropTag, DropTagNode, visit_drop_tag;
        ShowTags, ShowTagsNode, visit_show_tags;
    );

    impl_visitor_methods!(
        CreateEdge, CreateEdgeNode, visit_create_edge;
        AlterEdge, AlterEdgeNode, visit_alter_edge;
        DescEdge, DescEdgeNode, visit_desc_edge;
        DropEdge, DropEdgeNode, visit_drop_edge;
        ShowEdges, ShowEdgesNode, visit_show_edges;
    );

    impl_visitor_methods!(
        CreateTagIndex, CreateTagIndexNode, visit_create_tag_index;
        DropTagIndex, DropTagIndexNode, visit_drop_tag_index;
        DescTagIndex, DescTagIndexNode, visit_desc_tag_index;
        ShowTagIndexes, ShowTagIndexesNode, visit_show_tag_indexes;
    );

    impl_visitor_methods!(
        CreateEdgeIndex, CreateEdgeIndexNode, visit_create_edge_index;
        DropEdgeIndex, DropEdgeIndexNode, visit_drop_edge_index;
        DescEdgeIndex, DescEdgeIndexNode, visit_desc_edge_index;
        ShowEdgeIndexes, ShowEdgeIndexesNode, visit_show_edge_indexes;
    );

    impl_visitor_methods!(
        RebuildTagIndex, RebuildTagIndexNode, visit_rebuild_tag_index;
        RebuildEdgeIndex, RebuildEdgeIndexNode, visit_rebuild_edge_index;
    );

    impl_visitor_methods!(
        CreateUser, CreateUserNode, visit_create_user;
        AlterUser, AlterUserNode, visit_alter_user;
        DropUser, DropUserNode, visit_drop_user;
        ChangePassword, ChangePasswordNode, visit_change_password;
    );
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
