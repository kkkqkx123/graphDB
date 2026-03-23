//! PlanNode 枚举定义
//!
//! 本文件定义了 PlanNodeEnum 枚举，包含所有可能的计划节点类型
//! 使用宏生成样板代码以减少重复

use crate::query::planner::plan::core::nodes::insert::insert_nodes::{
    InsertEdgesNode, InsertVerticesNode,
};
use crate::query::planner::plan::core::nodes::management::edge_nodes::{
    AlterEdgeNode, CreateEdgeNode, DescEdgeNode, DropEdgeNode, ShowEdgesNode,
};
use crate::query::planner::plan::core::nodes::management::index_nodes::{
    CreateEdgeIndexNode, CreateTagIndexNode, DescEdgeIndexNode, DescTagIndexNode,
    DropEdgeIndexNode, DropTagIndexNode, RebuildEdgeIndexNode, RebuildTagIndexNode,
    ShowEdgeIndexesNode, ShowTagIndexesNode,
};
use crate::query::planner::plan::core::nodes::management::space_nodes::{
    AlterSpaceNode, ClearSpaceNode, CreateSpaceNode, DescSpaceNode, DropSpaceNode, ShowSpacesNode,
    SwitchSpaceNode,
};
use crate::query::planner::plan::core::nodes::management::stats_nodes::ShowStatsNode;
use crate::query::planner::plan::core::nodes::management::tag_nodes::{
    AlterTagNode, CreateTagNode, DescTagNode, DropTagNode, ShowTagsNode,
};
use crate::query::planner::plan::core::nodes::management::user_nodes::{
    AlterUserNode, ChangePasswordNode, CreateUserNode, DropUserNode, GrantRoleNode, RevokeRoleNode,
};

// 导入并重新导出所有具体的节点类型
pub use crate::query::planner::plan::core::nodes::access::graph_scan_node::{
    EdgeIndexScanNode, GetEdgesNode, GetNeighborsNode, GetVerticesNode, ScanEdgesNode,
    ScanVerticesNode,
};
pub use crate::query::planner::plan::core::nodes::access::index_scan::{
    IndexLimit, IndexScanNode, OrderByItem, ScanType,
};
pub use crate::query::planner::plan::core::nodes::control_flow::control_flow_node::{
    ArgumentNode, LoopNode, PassThroughNode, SelectNode,
};
pub use crate::query::planner::plan::core::nodes::control_flow::start_node::StartNode;
pub use crate::query::planner::plan::core::nodes::data_processing::aggregate_node::AggregateNode;
pub use crate::query::planner::plan::core::nodes::data_processing::data_processing_node::{
    AssignNode, DataCollectNode, DedupNode, MaterializeNode, PatternApplyNode, RemoveNode,
    RollUpApplyNode, UnionNode, UnwindNode,
};
pub use crate::query::planner::plan::core::nodes::data_processing::set_operations_node::{
    IntersectNode, MinusNode,
};
pub use crate::query::planner::plan::core::nodes::join::join_node::{
    CrossJoinNode, FullOuterJoinNode, HashInnerJoinNode, HashLeftJoinNode, InnerJoinNode,
    LeftJoinNode,
};
pub use crate::query::planner::plan::core::nodes::operation::filter_node::FilterNode;
pub use crate::query::planner::plan::core::nodes::operation::project_node::ProjectNode;
pub use crate::query::planner::plan::core::nodes::operation::sample_node::SampleNode;
pub use crate::query::planner::plan::core::nodes::operation::sort_node::{
    LimitNode, SortNode, TopNNode,
};
pub use crate::query::planner::plan::core::nodes::traversal::path_algorithms::{
    AllPathsNode, BFSShortestNode, MultiShortestPathNode, ShortestPathNode,
};
pub use crate::query::planner::plan::core::nodes::traversal::traversal_node::{
    AppendVerticesNode, ExpandAllNode, ExpandNode, TraverseNode,
};

/// PlanNode 枚举，包含所有可能的节点类型
///
/// 这个枚举避免了动态分发的性能开销
#[derive(Debug, Clone)]
pub enum PlanNodeEnum {
    // ========== 访问节点 ==========
    Start(StartNode),
    GetVertices(GetVerticesNode),
    GetEdges(GetEdgesNode),
    GetNeighbors(GetNeighborsNode),
    ScanVertices(ScanVerticesNode),
    ScanEdges(ScanEdgesNode),
    EdgeIndexScan(EdgeIndexScanNode),
    IndexScan(IndexScanNode),

    // ========== 操作节点 ==========
    Project(ProjectNode),
    Filter(FilterNode),
    Sort(SortNode),
    Limit(LimitNode),
    TopN(TopNNode),
    Sample(SampleNode),
    Dedup(DedupNode),
    Aggregate(AggregateNode),

    // ========== 连接节点 ==========
    InnerJoin(InnerJoinNode),
    LeftJoin(LeftJoinNode),
    CrossJoin(CrossJoinNode),
    HashInnerJoin(HashInnerJoinNode),
    HashLeftJoin(HashLeftJoinNode),
    FullOuterJoin(FullOuterJoinNode),

    // ========== 遍历节点 ==========
    Expand(ExpandNode),
    ExpandAll(ExpandAllNode),
    Traverse(TraverseNode),
    AppendVertices(AppendVerticesNode),

    // ========== 控制流节点 ==========
    Argument(ArgumentNode),
    Loop(LoopNode),
    PassThrough(PassThroughNode),
    Select(SelectNode),

    // ========== 数据处理节点 ==========
    DataCollect(DataCollectNode),
    Remove(RemoveNode),
    PatternApply(PatternApplyNode),
    RollUpApply(RollUpApplyNode),
    Union(UnionNode),
    Minus(MinusNode),
    Intersect(IntersectNode),
    Unwind(UnwindNode),
    Materialize(MaterializeNode),
    Assign(AssignNode),

    // ========== 算法节点 ==========
    MultiShortestPath(MultiShortestPathNode),
    BFSShortest(BFSShortestNode),
    AllPaths(AllPathsNode),
    ShortestPath(ShortestPathNode),

    // ========== 管理节点 - 空间 ==========
    CreateSpace(CreateSpaceNode),
    DropSpace(DropSpaceNode),
    DescSpace(DescSpaceNode),
    ShowSpaces(ShowSpacesNode),
    SwitchSpace(SwitchSpaceNode),
    AlterSpace(AlterSpaceNode),
    ClearSpace(ClearSpaceNode),

    // ========== 管理节点 - 标签 ==========
    CreateTag(CreateTagNode),
    AlterTag(AlterTagNode),
    DescTag(DescTagNode),
    DropTag(DropTagNode),
    ShowTags(ShowTagsNode),

    // ========== 管理节点 - 边类型 ==========
    CreateEdge(CreateEdgeNode),
    AlterEdge(AlterEdgeNode),
    DescEdge(DescEdgeNode),
    DropEdge(DropEdgeNode),
    ShowEdges(ShowEdgesNode),

    // ========== 管理节点 - 索引 ==========
    CreateTagIndex(CreateTagIndexNode),
    DropTagIndex(DropTagIndexNode),
    DescTagIndex(DescTagIndexNode),
    ShowTagIndexes(ShowTagIndexesNode),
    RebuildTagIndex(RebuildTagIndexNode),
    CreateEdgeIndex(CreateEdgeIndexNode),
    DropEdgeIndex(DropEdgeIndexNode),
    DescEdgeIndex(DescEdgeIndexNode),
    ShowEdgeIndexes(ShowEdgeIndexesNode),
    RebuildEdgeIndex(RebuildEdgeIndexNode),

    // ========== 管理节点 - 用户 ==========
    CreateUser(CreateUserNode),
    AlterUser(AlterUserNode),
    DropUser(DropUserNode),
    ChangePassword(ChangePasswordNode),
    GrantRole(GrantRoleNode),
    RevokeRole(RevokeRoleNode),

    // ========== 管理节点 - 数据 ==========
    InsertVertices(InsertVerticesNode),
    InsertEdges(InsertEdgesNode),

    // ========== 统计节点 ==========
    ShowStats(ShowStatsNode),
}

impl Default for PlanNodeEnum {
    fn default() -> Self {
        PlanNodeEnum::Start(StartNode::new())
    }
}

// 使用宏生成 is_xxx 方法
crate::define_enum_is_methods! {
    PlanNodeEnum,
    // 访问节点
    (Start, is_start),
    (GetVertices, is_get_vertices),
    (GetEdges, is_get_edges),
    (GetNeighbors, is_get_neighbors),
    (ScanVertices, is_scan_vertices),
    (ScanEdges, is_scan_edges),
    (EdgeIndexScan, is_edge_index_scan),
    (IndexScan, is_index_scan),
    // 操作节点
    (Project, is_project),
    (Filter, is_filter),
    (Sort, is_sort),
    (Limit, is_limit),
    (TopN, is_topn),
    (Sample, is_sample),
    (Dedup, is_dedup),
    (Aggregate, is_aggregate),
    // 连接节点
    (InnerJoin, is_inner_join),
    (LeftJoin, is_left_join),
    (CrossJoin, is_cross_join),
    (HashInnerJoin, is_hash_inner_join),
    (HashLeftJoin, is_hash_left_join),
    (FullOuterJoin, is_full_outer_join),
    // 遍历节点
    (Expand, is_expand),
    (ExpandAll, is_expand_all),
    (Traverse, is_traverse),
    (AppendVertices, is_append_vertices),
    // 控制流节点
    (Argument, is_argument),
    (Loop, is_loop),
    (PassThrough, is_pass_through),
    (Select, is_select),
    // 数据处理节点
    (DataCollect, is_data_collect),
    (Remove, is_remove),
    (PatternApply, is_pattern_apply),
    (RollUpApply, is_roll_up_apply),
    (Union, is_union),
    (Minus, is_minus),
    (Intersect, is_intersect),
    (Unwind, is_unwind),
    (Materialize, is_materialize),
    (Assign, is_assign),
    // 算法节点
    (MultiShortestPath, is_multi_shortest_path),
    (BFSShortest, is_bfs_shortest),
    (AllPaths, is_all_paths),
    (ShortestPath, is_shortest_path),
    // 管理节点 - 空间
    (CreateSpace, is_create_space),
    (DropSpace, is_drop_space),
    (DescSpace, is_desc_space),
    (ShowSpaces, is_show_spaces),
    (SwitchSpace, is_switch_space),
    (AlterSpace, is_alter_space),
    (ClearSpace, is_clear_space),
    // 管理节点 - 标签
    (CreateTag, is_create_tag),
    (AlterTag, is_alter_tag),
    (DescTag, is_desc_tag),
    (DropTag, is_drop_tag),
    (ShowTags, is_show_tags),
    // 管理节点 - 边类型
    (CreateEdge, is_create_edge),
    (AlterEdge, is_alter_edge),
    (DescEdge, is_desc_edge),
    (DropEdge, is_drop_edge),
    (ShowEdges, is_show_edges),
    // 管理节点 - 索引
    (CreateTagIndex, is_create_tag_index),
    (DropTagIndex, is_drop_tag_index),
    (DescTagIndex, is_desc_tag_index),
    (ShowTagIndexes, is_show_tag_indexes),
    (RebuildTagIndex, is_rebuild_tag_index),
    (CreateEdgeIndex, is_create_edge_index),
    (DropEdgeIndex, is_drop_edge_index),
    (DescEdgeIndex, is_desc_edge_index),
    (ShowEdgeIndexes, is_show_edge_indexes),
    (RebuildEdgeIndex, is_rebuild_edge_index),
    // 管理节点 - 用户
    (CreateUser, is_create_user),
    (AlterUser, is_alter_user),
    (DropUser, is_drop_user),
    (ChangePassword, is_change_password),
    (GrantRole, is_grant_role),
    (RevokeRole, is_revoke_role),
    // 管理节点 - 数据
    (InsertVertices, is_insert_vertices),
    (InsertEdges, is_insert_edges),
    // 统计节点
    (ShowStats, is_show_stats),
}

// 使用宏生成 as_xxx 方法
crate::define_enum_as_methods! {
    PlanNodeEnum,
    // 访问节点
    (Start, as_start, StartNode),
    (GetVertices, as_get_vertices, GetVerticesNode),
    (GetEdges, as_get_edges, GetEdgesNode),
    (GetNeighbors, as_get_neighbors, GetNeighborsNode),
    (ScanVertices, as_scan_vertices, ScanVerticesNode),
    (ScanEdges, as_scan_edges, ScanEdgesNode),
    (EdgeIndexScan, as_edge_index_scan, EdgeIndexScanNode),
    (IndexScan, as_index_scan, IndexScanNode),
    // 操作节点
    (Project, as_project, ProjectNode),
    (Filter, as_filter, FilterNode),
    (Sort, as_sort, SortNode),
    (Limit, as_limit, LimitNode),
    (TopN, as_topn, TopNNode),
    (Sample, as_sample, SampleNode),
    (Dedup, as_dedup, DedupNode),
    (Aggregate, as_aggregate, AggregateNode),
    // 连接节点
    (InnerJoin, as_inner_join, InnerJoinNode),
    (LeftJoin, as_left_join, LeftJoinNode),
    (CrossJoin, as_cross_join, CrossJoinNode),
    (HashInnerJoin, as_hash_inner_join, HashInnerJoinNode),
    (HashLeftJoin, as_hash_left_join, HashLeftJoinNode),
    (FullOuterJoin, as_full_outer_join, FullOuterJoinNode),
    // 遍历节点
    (Expand, as_expand, ExpandNode),
    (ExpandAll, as_expand_all, ExpandAllNode),
    (Traverse, as_traverse, TraverseNode),
    (AppendVertices, as_append_vertices, AppendVerticesNode),
    // 控制流节点
    (Argument, as_argument, ArgumentNode),
    (Loop, as_loop, LoopNode),
    (PassThrough, as_pass_through, PassThroughNode),
    (Select, as_select, SelectNode),
    // 数据处理节点
    (DataCollect, as_data_collect, DataCollectNode),
    (Remove, as_remove, RemoveNode),
    (PatternApply, as_pattern_apply, PatternApplyNode),
    (RollUpApply, as_roll_up_apply, RollUpApplyNode),
    (Union, as_union, UnionNode),
    (Minus, as_minus, MinusNode),
    (Intersect, as_intersect, IntersectNode),
    (Unwind, as_unwind, UnwindNode),
    (Materialize, as_materialize, MaterializeNode),
    (Assign, as_assign, AssignNode),
    // 算法节点
    (MultiShortestPath, as_multi_shortest_path, MultiShortestPathNode),
    (BFSShortest, as_bfs_shortest, BFSShortestNode),
    (AllPaths, as_all_paths, AllPathsNode),
    (ShortestPath, as_shortest_path, ShortestPathNode),
    // 管理节点 - 空间
    (CreateSpace, as_create_space, CreateSpaceNode),
    (DropSpace, as_drop_space, DropSpaceNode),
    (DescSpace, as_desc_space, DescSpaceNode),
    (ShowSpaces, as_show_spaces, ShowSpacesNode),
    (SwitchSpace, as_switch_space, SwitchSpaceNode),
    (AlterSpace, as_alter_space, AlterSpaceNode),
    (ClearSpace, as_clear_space, ClearSpaceNode),
    // 管理节点 - 标签
    (CreateTag, as_create_tag, CreateTagNode),
    (AlterTag, as_alter_tag, AlterTagNode),
    (DescTag, as_desc_tag, DescTagNode),
    (DropTag, as_drop_tag, DropTagNode),
    (ShowTags, as_show_tags, ShowTagsNode),
    // 管理节点 - 边类型
    (CreateEdge, as_create_edge, CreateEdgeNode),
    (AlterEdge, as_alter_edge, AlterEdgeNode),
    (DescEdge, as_desc_edge, DescEdgeNode),
    (DropEdge, as_drop_edge, DropEdgeNode),
    (ShowEdges, as_show_edges, ShowEdgesNode),
    // 管理节点 - 索引
    (CreateTagIndex, as_create_tag_index, CreateTagIndexNode),
    (DropTagIndex, as_drop_tag_index, DropTagIndexNode),
    (DescTagIndex, as_desc_tag_index, DescTagIndexNode),
    (ShowTagIndexes, as_show_tag_indexes, ShowTagIndexesNode),
    (RebuildTagIndex, as_rebuild_tag_index, RebuildTagIndexNode),
    (CreateEdgeIndex, as_create_edge_index, CreateEdgeIndexNode),
    (DropEdgeIndex, as_drop_edge_index, DropEdgeIndexNode),
    (DescEdgeIndex, as_desc_edge_index, DescEdgeIndexNode),
    (ShowEdgeIndexes, as_show_edge_indexes, ShowEdgeIndexesNode),
    (RebuildEdgeIndex, as_rebuild_edge_index, RebuildEdgeIndexNode),
    // 管理节点 - 用户
    (CreateUser, as_create_user, CreateUserNode),
    (AlterUser, as_alter_user, AlterUserNode),
    (DropUser, as_drop_user, DropUserNode),
    (ChangePassword, as_change_password, ChangePasswordNode),
    (GrantRole, as_grant_role, GrantRoleNode),
    (RevokeRole, as_revoke_role, RevokeRoleNode),
    // 管理节点 - 数据
    (InsertVertices, as_insert_vertices, InsertVerticesNode),
    (InsertEdges, as_insert_edges, InsertEdgesNode),
    // 统计节点
    (ShowStats, as_show_stats, ShowStatsNode),
}

// 使用宏生成 as_xxx_mut 方法
crate::define_enum_as_mut_methods! {
    PlanNodeEnum,
    // 访问节点
    (Start, as_start_mut, StartNode),
    (GetVertices, as_get_vertices_mut, GetVerticesNode),
    (GetEdges, as_get_edges_mut, GetEdgesNode),
    (GetNeighbors, as_get_neighbors_mut, GetNeighborsNode),
    (ScanVertices, as_scan_vertices_mut, ScanVerticesNode),
    (ScanEdges, as_scan_edges_mut, ScanEdgesNode),
    (EdgeIndexScan, as_edge_index_scan_mut, EdgeIndexScanNode),
    (IndexScan, as_index_scan_mut, IndexScanNode),
    // 操作节点
    (Project, as_project_mut, ProjectNode),
    (Filter, as_filter_mut, FilterNode),
    (Sort, as_sort_mut, SortNode),
    (Limit, as_limit_mut, LimitNode),
    (TopN, as_topn_mut, TopNNode),
    (Sample, as_sample_mut, SampleNode),
    (Dedup, as_dedup_mut, DedupNode),
    (Aggregate, as_aggregate_mut, AggregateNode),
    // 连接节点
    (InnerJoin, as_inner_join_mut, InnerJoinNode),
    (LeftJoin, as_left_join_mut, LeftJoinNode),
    (CrossJoin, as_cross_join_mut, CrossJoinNode),
    (HashInnerJoin, as_hash_inner_join_mut, HashInnerJoinNode),
    (HashLeftJoin, as_hash_left_join_mut, HashLeftJoinNode),
    (FullOuterJoin, as_full_outer_join_mut, FullOuterJoinNode),
    // 遍历节点
    (Expand, as_expand_mut, ExpandNode),
    (ExpandAll, as_expand_all_mut, ExpandAllNode),
    (Traverse, as_traverse_mut, TraverseNode),
    (AppendVertices, as_append_vertices_mut, AppendVerticesNode),
    // 控制流节点
    (Argument, as_argument_mut, ArgumentNode),
    (Loop, as_loop_mut, LoopNode),
    (PassThrough, as_pass_through_mut, PassThroughNode),
    (Select, as_select_mut, SelectNode),
    // 数据处理节点
    (DataCollect, as_data_collect_mut, DataCollectNode),
    (Remove, as_remove_mut, RemoveNode),
    (PatternApply, as_pattern_apply_mut, PatternApplyNode),
    (RollUpApply, as_roll_up_apply_mut, RollUpApplyNode),
    (Union, as_union_mut, UnionNode),
    (Minus, as_minus_mut, MinusNode),
    (Intersect, as_intersect_mut, IntersectNode),
    (Unwind, as_unwind_mut, UnwindNode),
    (Materialize, as_materialize_mut, MaterializeNode),
    (Assign, as_assign_mut, AssignNode),
    // 算法节点
    (MultiShortestPath, as_multi_shortest_path_mut, MultiShortestPathNode),
    (BFSShortest, as_bfs_shortest_mut, BFSShortestNode),
    (AllPaths, as_all_paths_mut, AllPathsNode),
    (ShortestPath, as_shortest_path_mut, ShortestPathNode),
    // 管理节点 - 空间
    (CreateSpace, as_create_space_mut, CreateSpaceNode),
    (DropSpace, as_drop_space_mut, DropSpaceNode),
    (DescSpace, as_desc_space_mut, DescSpaceNode),
    (ShowSpaces, as_show_spaces_mut, ShowSpacesNode),
    (SwitchSpace, as_switch_space_mut, SwitchSpaceNode),
    (AlterSpace, as_alter_space_mut, AlterSpaceNode),
    (ClearSpace, as_clear_space_mut, ClearSpaceNode),
    // 管理节点 - 标签
    (CreateTag, as_create_tag_mut, CreateTagNode),
    (AlterTag, as_alter_tag_mut, AlterTagNode),
    (DescTag, as_desc_tag_mut, DescTagNode),
    (DropTag, as_drop_tag_mut, DropTagNode),
    (ShowTags, as_show_tags_mut, ShowTagsNode),
    // 管理节点 - 边类型
    (CreateEdge, as_create_edge_mut, CreateEdgeNode),
    (AlterEdge, as_alter_edge_mut, AlterEdgeNode),
    (DescEdge, as_desc_edge_mut, DescEdgeNode),
    (DropEdge, as_drop_edge_mut, DropEdgeNode),
    (ShowEdges, as_show_edges_mut, ShowEdgesNode),
    // 管理节点 - 索引
    (CreateTagIndex, as_create_tag_index_mut, CreateTagIndexNode),
    (DropTagIndex, as_drop_tag_index_mut, DropTagIndexNode),
    (DescTagIndex, as_desc_tag_index_mut, DescTagIndexNode),
    (ShowTagIndexes, as_show_tag_indexes_mut, ShowTagIndexesNode),
    (RebuildTagIndex, as_rebuild_tag_index_mut, RebuildTagIndexNode),
    (CreateEdgeIndex, as_create_edge_index_mut, CreateEdgeIndexNode),
    (DropEdgeIndex, as_drop_edge_index_mut, DropEdgeIndexNode),
    (DescEdgeIndex, as_desc_edge_index_mut, DescEdgeIndexNode),
    (ShowEdgeIndexes, as_show_edge_indexes_mut, ShowEdgeIndexesNode),
    (RebuildEdgeIndex, as_rebuild_edge_index_mut, RebuildEdgeIndexNode),
    // 管理节点 - 用户
    (CreateUser, as_create_user_mut, CreateUserNode),
    (AlterUser, as_alter_user_mut, AlterUserNode),
    (DropUser, as_drop_user_mut, DropUserNode),
    (ChangePassword, as_change_password_mut, ChangePasswordNode),
    (GrantRole, as_grant_role_mut, GrantRoleNode),
    (RevokeRole, as_revoke_role_mut, RevokeRoleNode),
    // 管理节点 - 数据
    (InsertVertices, as_insert_vertices_mut, InsertVerticesNode),
    (InsertEdges, as_insert_edges_mut, InsertEdgesNode),
    // 统计节点
    (ShowStats, as_show_stats_mut, ShowStatsNode),
}

// 使用宏生成 type_name 方法
crate::define_enum_type_name! {
    PlanNodeEnum,
    // 访问节点
    (Start, "Start"),
    (GetVertices, "GetVertices"),
    (GetEdges, "GetEdges"),
    (GetNeighbors, "GetNeighbors"),
    (ScanVertices, "ScanVertices"),
    (ScanEdges, "ScanEdges"),
    (EdgeIndexScan, "EdgeIndexScan"),
    (IndexScan, "IndexScan"),
    // 操作节点
    (Project, "Project"),
    (Filter, "Filter"),
    (Sort, "Sort"),
    (Limit, "Limit"),
    (TopN, "TopN"),
    (Sample, "Sample"),
    (Dedup, "Dedup"),
    (Aggregate, "Aggregate"),
    // 连接节点
    (InnerJoin, "InnerJoin"),
    (LeftJoin, "LeftJoin"),
    (CrossJoin, "CrossJoin"),
    (HashInnerJoin, "HashInnerJoin"),
    (HashLeftJoin, "HashLeftJoin"),
    (FullOuterJoin, "FullOuterJoin"),
    // 遍历节点
    (Expand, "Expand"),
    (ExpandAll, "ExpandAll"),
    (Traverse, "Traverse"),
    (AppendVertices, "AppendVertices"),
    // 控制流节点
    (Argument, "Argument"),
    (Loop, "Loop"),
    (PassThrough, "PassThrough"),
    (Select, "Select"),
    // 数据处理节点
    (DataCollect, "DataCollect"),
    (Remove, "Remove"),
    (PatternApply, "PatternApply"),
    (RollUpApply, "RollUpApply"),
    (Union, "Union"),
    (Minus, "Minus"),
    (Intersect, "Intersect"),
    (Unwind, "Unwind"),
    (Materialize, "Materialize"),
    (Assign, "Assign"),
    // 算法节点
    (MultiShortestPath, "MultiShortestPath"),
    (BFSShortest, "BFSShortest"),
    (AllPaths, "AllPaths"),
    (ShortestPath, "ShortestPath"),
    // 管理节点 - 空间
    (CreateSpace, "CreateSpace"),
    (DropSpace, "DropSpace"),
    (DescSpace, "DescSpace"),
    (ShowSpaces, "ShowSpaces"),
    (SwitchSpace, "SwitchSpace"),
    (AlterSpace, "AlterSpace"),
    (ClearSpace, "ClearSpace"),
    // 管理节点 - 标签
    (CreateTag, "CreateTag"),
    (AlterTag, "AlterTag"),
    (DescTag, "DescTag"),
    (DropTag, "DropTag"),
    (ShowTags, "ShowTags"),
    // 管理节点 - 边类型
    (CreateEdge, "CreateEdge"),
    (AlterEdge, "AlterEdge"),
    (DescEdge, "DescEdge"),
    (DropEdge, "DropEdge"),
    (ShowEdges, "ShowEdges"),
    // 管理节点 - 索引
    (CreateTagIndex, "CreateTagIndex"),
    (DropTagIndex, "DropTagIndex"),
    (DescTagIndex, "DescTagIndex"),
    (ShowTagIndexes, "ShowTagIndexes"),
    (RebuildTagIndex, "RebuildTagIndex"),
    (CreateEdgeIndex, "CreateEdgeIndex"),
    (DropEdgeIndex, "DropEdgeIndex"),
    (DescEdgeIndex, "DescEdgeIndex"),
    (ShowEdgeIndexes, "ShowEdgeIndexes"),
    (RebuildEdgeIndex, "RebuildEdgeIndex"),
    // 管理节点 - 用户
    (CreateUser, "CreateUser"),
    (AlterUser, "AlterUser"),
    (DropUser, "DropUser"),
    (ChangePassword, "ChangePassword"),
    (GrantRole, "GrantRole"),
    (RevokeRole, "RevokeRole"),
    // 管理节点 - 数据
    (InsertVertices, "InsertVertices"),
    (InsertEdges, "InsertEdges"),
    // 统计节点
    (ShowStats, "ShowStats"),
}

// 使用宏生成 category 方法
crate::define_enum_category! {
    PlanNodeEnum,
    // 访问节点
    (Start, PlanNodeCategory::Access),
    (GetVertices, PlanNodeCategory::Access),
    (GetEdges, PlanNodeCategory::Access),
    (GetNeighbors, PlanNodeCategory::Access),
    (ScanVertices, PlanNodeCategory::Access),
    (ScanEdges, PlanNodeCategory::Access),
    (EdgeIndexScan, PlanNodeCategory::Access),
    (IndexScan, PlanNodeCategory::Access),
    // 操作节点
    (Project, PlanNodeCategory::Operation),
    (Filter, PlanNodeCategory::Operation),
    (Sort, PlanNodeCategory::Operation),
    (Limit, PlanNodeCategory::Operation),
    (TopN, PlanNodeCategory::Operation),
    (Sample, PlanNodeCategory::Operation),
    (Dedup, PlanNodeCategory::Operation),
    (Aggregate, PlanNodeCategory::Operation),
    // 连接节点
    (InnerJoin, PlanNodeCategory::Join),
    (LeftJoin, PlanNodeCategory::Join),
    (CrossJoin, PlanNodeCategory::Join),
    (HashInnerJoin, PlanNodeCategory::Join),
    (HashLeftJoin, PlanNodeCategory::Join),
    (FullOuterJoin, PlanNodeCategory::Join),
    // 遍历节点
    (Expand, PlanNodeCategory::Traversal),
    (ExpandAll, PlanNodeCategory::Traversal),
    (Traverse, PlanNodeCategory::Traversal),
    (AppendVertices, PlanNodeCategory::Traversal),
    // 控制流节点
    (Argument, PlanNodeCategory::ControlFlow),
    (Loop, PlanNodeCategory::ControlFlow),
    (PassThrough, PlanNodeCategory::ControlFlow),
    (Select, PlanNodeCategory::ControlFlow),
    // 数据处理节点
    (DataCollect, PlanNodeCategory::DataProcessing),
    (Remove, PlanNodeCategory::DataProcessing),
    (PatternApply, PlanNodeCategory::DataProcessing),
    (RollUpApply, PlanNodeCategory::DataProcessing),
    (Union, PlanNodeCategory::DataProcessing),
    (Minus, PlanNodeCategory::DataProcessing),
    (Intersect, PlanNodeCategory::DataProcessing),
    (Unwind, PlanNodeCategory::DataProcessing),
    (Materialize, PlanNodeCategory::DataProcessing),
    (Assign, PlanNodeCategory::DataProcessing),
    // 算法节点
    (MultiShortestPath, PlanNodeCategory::Algorithm),
    (BFSShortest, PlanNodeCategory::Algorithm),
    (AllPaths, PlanNodeCategory::Algorithm),
    (ShortestPath, PlanNodeCategory::Algorithm),
    // 管理节点
    (CreateSpace, PlanNodeCategory::Management),
    (DropSpace, PlanNodeCategory::Management),
    (DescSpace, PlanNodeCategory::Management),
    (ShowSpaces, PlanNodeCategory::Management),
    (SwitchSpace, PlanNodeCategory::Management),
    (AlterSpace, PlanNodeCategory::Management),
    (ClearSpace, PlanNodeCategory::Management),
    (CreateTag, PlanNodeCategory::Management),
    (AlterTag, PlanNodeCategory::Management),
    (DescTag, PlanNodeCategory::Management),
    (DropTag, PlanNodeCategory::Management),
    (ShowTags, PlanNodeCategory::Management),
    (CreateEdge, PlanNodeCategory::Management),
    (AlterEdge, PlanNodeCategory::Management),
    (DescEdge, PlanNodeCategory::Management),
    (DropEdge, PlanNodeCategory::Management),
    (ShowEdges, PlanNodeCategory::Management),
    (CreateTagIndex, PlanNodeCategory::Management),
    (DropTagIndex, PlanNodeCategory::Management),
    (DescTagIndex, PlanNodeCategory::Management),
    (ShowTagIndexes, PlanNodeCategory::Management),
    (RebuildTagIndex, PlanNodeCategory::Management),
    (CreateEdgeIndex, PlanNodeCategory::Management),
    (DropEdgeIndex, PlanNodeCategory::Management),
    (DescEdgeIndex, PlanNodeCategory::Management),
    (ShowEdgeIndexes, PlanNodeCategory::Management),
    (RebuildEdgeIndex, PlanNodeCategory::Management),
    (CreateUser, PlanNodeCategory::Management),
    (AlterUser, PlanNodeCategory::Management),
    (DropUser, PlanNodeCategory::Management),
    (ChangePassword, PlanNodeCategory::Management),
    (GrantRole, PlanNodeCategory::Management),
    (RevokeRole, PlanNodeCategory::Management),
    (InsertVertices, PlanNodeCategory::Management),
    (InsertEdges, PlanNodeCategory::Management),
    (ShowStats, PlanNodeCategory::Management),
}

// 使用宏生成 describe 方法
crate::define_enum_describe! {
    PlanNodeEnum,
    // 访问节点
    (Start, "Start"),
    (GetVertices, "GetVertices"),
    (GetEdges, "GetEdges"),
    (GetNeighbors, "GetNeighbors"),
    (ScanVertices, "ScanVertices"),
    (ScanEdges, "ScanEdges"),
    (EdgeIndexScan, "EdgeIndexScan"),
    (IndexScan, "IndexScan"),
    // 操作节点
    (Project, "Project"),
    (Filter, "Filter"),
    (Sort, "Sort"),
    (Limit, "Limit"),
    (TopN, "TopN"),
    (Sample, "Sample"),
    (Dedup, "Dedup"),
    (Aggregate, "Aggregate"),
    // 连接节点
    (InnerJoin, "InnerJoin"),
    (LeftJoin, "LeftJoin"),
    (CrossJoin, "CrossJoin"),
    (HashInnerJoin, "HashInnerJoin"),
    (HashLeftJoin, "HashLeftJoin"),
    (FullOuterJoin, "FullOuterJoin"),
    // 遍历节点
    (Expand, "Expand"),
    (ExpandAll, "ExpandAll"),
    (Traverse, "Traverse"),
    (AppendVertices, "AppendVertices"),
    // 控制流节点
    (Argument, "Argument"),
    (Loop, "Loop"),
    (PassThrough, "PassThrough"),
    (Select, "Select"),
    // 数据处理节点
    (DataCollect, "DataCollect"),
    (Remove, "Remove"),
    (PatternApply, "PatternApply"),
    (RollUpApply, "RollUpApply"),
    (Union, "Union"),
    (Minus, "Minus"),
    (Intersect, "Intersect"),
    (Unwind, "Unwind"),
    (Materialize, "Materialize"),
    (Assign, "Assign"),
    // 算法节点
    (MultiShortestPath, "MultiShortestPath"),
    (BFSShortest, "BFSShortest"),
    (AllPaths, "AllPaths"),
    (ShortestPath, "ShortestPath"),
    // 管理节点 - 空间
    (CreateSpace, "CreateSpace"),
    (DropSpace, "DropSpace"),
    (DescSpace, "DescSpace"),
    (ShowSpaces, "ShowSpaces"),
    (SwitchSpace, "SwitchSpace"),
    (AlterSpace, "AlterSpace"),
    (ClearSpace, "ClearSpace"),
    // 管理节点 - 标签
    (CreateTag, "CreateTag"),
    (AlterTag, "AlterTag"),
    (DescTag, "DescTag"),
    (DropTag, "DropTag"),
    (ShowTags, "ShowTags"),
    // 管理节点 - 边类型
    (CreateEdge, "CreateEdge"),
    (AlterEdge, "AlterEdge"),
    (DescEdge, "DescEdge"),
    (DropEdge, "DropEdge"),
    (ShowEdges, "ShowEdges"),
    // 管理节点 - 索引
    (CreateTagIndex, "CreateTagIndex"),
    (DropTagIndex, "DropTagIndex"),
    (DescTagIndex, "DescTagIndex"),
    (ShowTagIndexes, "ShowTagIndexes"),
    (RebuildTagIndex, "RebuildTagIndex"),
    (CreateEdgeIndex, "CreateEdgeIndex"),
    (DropEdgeIndex, "DropEdgeIndex"),
    (DescEdgeIndex, "DescEdgeIndex"),
    (ShowEdgeIndexes, "ShowEdgeIndexes"),
    (RebuildEdgeIndex, "RebuildEdgeIndex"),
    // 管理节点 - 用户
    (CreateUser, "CreateUser"),
    (AlterUser, "AlterUser"),
    (DropUser, "DropUser"),
    (ChangePassword, "ChangePassword"),
    (GrantRole, "GrantRole"),
    (RevokeRole, "RevokeRole"),
    // 管理节点 - 数据
    (InsertVertices, "InsertVertices"),
    (InsertEdges, "InsertEdges"),
    // 统计节点
    (ShowStats, "ShowStats"),
}

impl PlanNodeEnum {
    /// 节点克隆
    pub fn clone_plan_node(&self) -> PlanNodeEnum {
        self.clone()
    }

    /// 判断节点是否是访问节点
    pub fn is_access(&self) -> bool {
        matches!(
            self,
            PlanNodeEnum::Start(_)
                | PlanNodeEnum::ScanVertices(_)
                | PlanNodeEnum::ScanEdges(_)
                | PlanNodeEnum::EdgeIndexScan(_)
                | PlanNodeEnum::GetVertices(_)
                | PlanNodeEnum::GetEdges(_)
                | PlanNodeEnum::GetNeighbors(_)
                | PlanNodeEnum::IndexScan(_)
        )
    }

    /// 判断节点是否是操作节点
    pub fn is_operation(&self) -> bool {
        matches!(
            self,
            PlanNodeEnum::Filter(_)
                | PlanNodeEnum::Project(_)
                | PlanNodeEnum::Aggregate(_)
                | PlanNodeEnum::Sort(_)
                | PlanNodeEnum::Limit(_)
                | PlanNodeEnum::TopN(_)
                | PlanNodeEnum::Sample(_)
                | PlanNodeEnum::Dedup(_)
        )
    }

    /// 判断节点是否是连接节点
    pub fn is_join(&self) -> bool {
        matches!(
            self,
            PlanNodeEnum::InnerJoin(_)
                | PlanNodeEnum::LeftJoin(_)
                | PlanNodeEnum::CrossJoin(_)
                | PlanNodeEnum::HashInnerJoin(_)
                | PlanNodeEnum::HashLeftJoin(_)
                | PlanNodeEnum::FullOuterJoin(_)
        )
    }

    /// 判断节点是否是遍历节点
    pub fn is_traversal(&self) -> bool {
        matches!(
            self,
            PlanNodeEnum::Expand(_)
                | PlanNodeEnum::ExpandAll(_)
                | PlanNodeEnum::Traverse(_)
                | PlanNodeEnum::AppendVertices(_)
        )
    }

    /// 判断节点是否是控制流节点
    pub fn is_control_flow(&self) -> bool {
        matches!(
            self,
            PlanNodeEnum::Argument(_)
                | PlanNodeEnum::Loop(_)
                | PlanNodeEnum::PassThrough(_)
                | PlanNodeEnum::Select(_)
        )
    }

    /// 判断节点是否是数据处理节点
    pub fn is_data_processing(&self) -> bool {
        matches!(
            self,
            PlanNodeEnum::DataCollect(_)
                | PlanNodeEnum::Union(_)
                | PlanNodeEnum::Minus(_)
                | PlanNodeEnum::Intersect(_)
                | PlanNodeEnum::Unwind(_)
                | PlanNodeEnum::Assign(_)
                | PlanNodeEnum::PatternApply(_)
                | PlanNodeEnum::RollUpApply(_)
        )
    }

    /// 判断节点是否是算法节点
    pub fn is_algorithm(&self) -> bool {
        matches!(
            self,
            PlanNodeEnum::ShortestPath(_)
                | PlanNodeEnum::AllPaths(_)
                | PlanNodeEnum::MultiShortestPath(_)
                | PlanNodeEnum::BFSShortest(_)
        )
    }

    /// 判断节点是否是管理节点
    pub fn is_management(&self) -> bool {
        matches!(
            self,
            PlanNodeEnum::CreateSpace(_)
                | PlanNodeEnum::DropSpace(_)
                | PlanNodeEnum::DescSpace(_)
                | PlanNodeEnum::ShowSpaces(_)
                | PlanNodeEnum::CreateTag(_)
                | PlanNodeEnum::AlterTag(_)
                | PlanNodeEnum::DescTag(_)
                | PlanNodeEnum::DropTag(_)
                | PlanNodeEnum::ShowTags(_)
                | PlanNodeEnum::CreateEdge(_)
                | PlanNodeEnum::AlterEdge(_)
                | PlanNodeEnum::DescEdge(_)
                | PlanNodeEnum::DropEdge(_)
                | PlanNodeEnum::ShowEdges(_)
                | PlanNodeEnum::CreateTagIndex(_)
                | PlanNodeEnum::DropTagIndex(_)
                | PlanNodeEnum::DescTagIndex(_)
                | PlanNodeEnum::ShowTagIndexes(_)
                | PlanNodeEnum::CreateEdgeIndex(_)
                | PlanNodeEnum::DropEdgeIndex(_)
                | PlanNodeEnum::DescEdgeIndex(_)
                | PlanNodeEnum::ShowEdgeIndexes(_)
                | PlanNodeEnum::RebuildTagIndex(_)
                | PlanNodeEnum::RebuildEdgeIndex(_)
                | PlanNodeEnum::CreateUser(_)
                | PlanNodeEnum::AlterUser(_)
                | PlanNodeEnum::DropUser(_)
                | PlanNodeEnum::ChangePassword(_)
                | PlanNodeEnum::InsertVertices(_)
                | PlanNodeEnum::InsertEdges(_)
        )
    }

    /// 判断节点是否是查询节点
    pub fn is_query_node(&self) -> bool {
        matches!(
            self,
            PlanNodeEnum::GetVertices(_)
                | PlanNodeEnum::GetEdges(_)
                | PlanNodeEnum::GetNeighbors(_)
                | PlanNodeEnum::Expand(_)
                | PlanNodeEnum::ExpandAll(_)
                | PlanNodeEnum::Traverse(_)
                | PlanNodeEnum::AppendVertices(_)
                | PlanNodeEnum::ScanVertices(_)
                | PlanNodeEnum::ScanEdges(_)
        )
    }

    /// 判断节点是否是数据处理节点
    pub fn is_data_processing_node(&self) -> bool {
        matches!(
            self,
            PlanNodeEnum::Filter(_)
                | PlanNodeEnum::Union(_)
                | PlanNodeEnum::Project(_)
                | PlanNodeEnum::Unwind(_)
                | PlanNodeEnum::Sort(_)
                | PlanNodeEnum::TopN(_)
                | PlanNodeEnum::Limit(_)
                | PlanNodeEnum::Aggregate(_)
                | PlanNodeEnum::Dedup(_)
                | PlanNodeEnum::DataCollect(_)
                | PlanNodeEnum::InnerJoin(_)
                | PlanNodeEnum::LeftJoin(_)
                | PlanNodeEnum::CrossJoin(_)
                | PlanNodeEnum::RollUpApply(_)
                | PlanNodeEnum::PatternApply(_)
                | PlanNodeEnum::Argument(_)
        )
    }

    /// 判断节点是否是控制流节点
    pub fn is_control_flow_node(&self) -> bool {
        matches!(
            self,
            PlanNodeEnum::Select(_)
                | PlanNodeEnum::Loop(_)
                | PlanNodeEnum::PassThrough(_)
                | PlanNodeEnum::Start(_)
        )
    }
}
