//! Definition of the PlanNode enumeration
//!
//! This document defines the PlanNodeEnum enumeration, which includes all possible types of planning nodes.
//! Use macros to generate template code in order to reduce repetition.

use std::collections::HashSet;

use crate::query::planning::plan::core::nodes::base::memory_estimation::MemoryEstimatable;
use crate::query::planning::plan::core::nodes::base::plan_node_traits::SingleInputNode;
use crate::query::planning::plan::core::nodes::insert::insert_nodes::{
    InsertEdgesNode, InsertVerticesNode,
};
use crate::query::planning::plan::core::nodes::management::edge_nodes::{
    AlterEdgeNode, CreateEdgeNode, DescEdgeNode, DropEdgeNode, ShowEdgesNode,
};
use crate::query::planning::plan::core::nodes::management::index_nodes::{
    CreateEdgeIndexNode, CreateTagIndexNode, DescEdgeIndexNode, DescTagIndexNode,
    DropEdgeIndexNode, DropTagIndexNode, RebuildEdgeIndexNode, RebuildTagIndexNode,
    ShowEdgeIndexesNode, ShowTagIndexesNode,
};
use crate::query::planning::plan::core::nodes::management::space_nodes::{
    AlterSpaceNode, ClearSpaceNode, CreateSpaceNode, DescSpaceNode, DropSpaceNode, ShowSpacesNode,
    SwitchSpaceNode,
};
use crate::query::planning::plan::core::nodes::management::stats_nodes::ShowStatsNode;
use crate::query::planning::plan::core::nodes::management::tag_nodes::{
    AlterTagNode, CreateTagNode, DescTagNode, DropTagNode, ShowCreateTagNode, ShowTagsNode,
};
use crate::query::planning::plan::core::nodes::management::user_nodes::{
    AlterUserNode, ChangePasswordNode, CreateUserNode, DropUserNode, GrantRoleNode, RevokeRoleNode,
};

// Import and re-export all specific node types.
pub use crate::query::planning::plan::core::nodes::access::graph_scan_node::{
    EdgeIndexScanNode, GetEdgesNode, GetNeighborsNode, GetVerticesNode, ScanEdgesNode,
    ScanVerticesNode,
};
pub use crate::query::planning::plan::core::nodes::access::index_scan::{
    IndexLimit, IndexScanNode, OrderByItem, ScanType,
};
pub use crate::query::planning::plan::core::nodes::control_flow::control_flow_node::{
    ArgumentNode, LoopNode, PassThroughNode, SelectNode,
};
pub use crate::query::planning::plan::core::nodes::control_flow::start_node::StartNode;
pub use crate::query::planning::plan::core::nodes::data_processing::aggregate_node::AggregateNode;
pub use crate::query::planning::plan::core::nodes::data_processing::data_processing_node::{
    AssignNode, DataCollectNode, DedupNode, MaterializeNode, PatternApplyNode, RemoveNode,
    RollUpApplyNode, UnionNode, UnwindNode,
};
pub use crate::query::planning::plan::core::nodes::data_processing::set_operations_node::{
    IntersectNode, MinusNode,
};
pub use crate::query::planning::plan::core::nodes::join::join_node::{
    CrossJoinNode, FullOuterJoinNode, HashInnerJoinNode, HashLeftJoinNode, InnerJoinNode,
    LeftJoinNode,
};
pub use crate::query::planning::plan::core::nodes::operation::filter_node::FilterNode;
pub use crate::query::planning::plan::core::nodes::operation::project_node::ProjectNode;
pub use crate::query::planning::plan::core::nodes::operation::sample_node::SampleNode;
pub use crate::query::planning::plan::core::nodes::operation::sort_node::{
    LimitNode, SortNode, TopNNode,
};
pub use crate::query::planning::plan::core::nodes::traversal::path_algorithms::{
    AllPathsNode, BFSShortestNode, MultiShortestPathNode, ShortestPathNode,
};
pub use crate::query::planning::plan::core::nodes::traversal::traversal_node::{
    AppendVerticesNode, ExpandAllNode, ExpandNode, TraverseNode,
};

/// The PlanNode enumeration includes all possible node types.
///
/// This enumeration avoids the performance overhead associated with dynamic distribution.
#[derive(Debug, Clone)]
pub enum PlanNodeEnum {
    // Access Node
    Start(StartNode),
    GetVertices(GetVerticesNode),
    GetEdges(GetEdgesNode),
    GetNeighbors(GetNeighborsNode),
    ScanVertices(ScanVerticesNode),
    ScanEdges(ScanEdgesNode),
    EdgeIndexScan(EdgeIndexScanNode),
    IndexScan(IndexScanNode),

    // Operation Node
    Project(ProjectNode),
    Filter(FilterNode),
    Sort(SortNode),
    Limit(LimitNode),
    TopN(TopNNode),
    Sample(SampleNode),
    Dedup(DedupNode),
    Aggregate(AggregateNode),

    // ========== Connecting Nodes ==========
    InnerJoin(InnerJoinNode),
    LeftJoin(LeftJoinNode),
    CrossJoin(CrossJoinNode),
    HashInnerJoin(HashInnerJoinNode),
    HashLeftJoin(HashLeftJoinNode),
    FullOuterJoin(FullOuterJoinNode),

    // Traversal of nodes
    Expand(ExpandNode),
    ExpandAll(ExpandAllNode),
    Traverse(TraverseNode),
    AppendVertices(AppendVerticesNode),

    // ========== Control Flow Nodes ==========
    Argument(ArgumentNode),
    Loop(LoopNode),
    PassThrough(PassThroughNode),
    Select(SelectNode),

    // ========== Data Processing Node ----------
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

    // Algorithm Nodes
    MultiShortestPath(MultiShortestPathNode),
    BFSShortest(BFSShortestNode),
    AllPaths(AllPathsNode),
    ShortestPath(ShortestPathNode),

    // Management Node – Space
    CreateSpace(CreateSpaceNode),
    DropSpace(DropSpaceNode),
    DescSpace(DescSpaceNode),
    ShowSpaces(ShowSpacesNode),
    SwitchSpace(SwitchSpaceNode),
    AlterSpace(AlterSpaceNode),
    ClearSpace(ClearSpaceNode),

    // Management Node – Tags
    CreateTag(CreateTagNode),
    AlterTag(AlterTagNode),
    DescTag(DescTagNode),
    DropTag(DropTagNode),
    ShowTags(ShowTagsNode),
    ShowCreateTag(ShowCreateTagNode),

    // Management Node – Edge Type
    CreateEdge(CreateEdgeNode),
    AlterEdge(AlterEdgeNode),
    DescEdge(DescEdgeNode),
    DropEdge(DropEdgeNode),
    ShowEdges(ShowEdgesNode),

    // Management Node – Index
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

    // Management Node – User
    CreateUser(CreateUserNode),
    AlterUser(AlterUserNode),
    DropUser(DropUserNode),
    ChangePassword(ChangePasswordNode),
    GrantRole(GrantRoleNode),
    RevokeRole(RevokeRoleNode),

    // Management Node – Data
    InsertVertices(InsertVerticesNode),
    InsertEdges(InsertEdgesNode),

    // Statistics Nodes ============
    ShowStats(ShowStatsNode),
}

impl Default for PlanNodeEnum {
    fn default() -> Self {
        PlanNodeEnum::Start(StartNode::new())
    }
}

// Use macros to generate the is_xxx method.
crate::define_enum_is_methods! {
    PlanNodeEnum,
    // Access node
    (Start, is_start),
    (GetVertices, is_get_vertices),
    (GetEdges, is_get_edges),
    (GetNeighbors, is_get_neighbors),
    (ScanVertices, is_scan_vertices),
    (ScanEdges, is_scan_edges),
    (EdgeIndexScan, is_edge_index_scan),
    (IndexScan, is_index_scan),
    // Operation node
    (Project, is_project),
    (Filter, is_filter),
    (Sort, is_sort),
    (Limit, is_limit),
    (TopN, is_topn),
    (Sample, is_sample),
    (Dedup, is_dedup),
    (Aggregate, is_aggregate),
    // Connecting nodes
    (InnerJoin, is_inner_join),
    (LeftJoin, is_left_join),
    (CrossJoin, is_cross_join),
    (HashInnerJoin, is_hash_inner_join),
    (HashLeftJoin, is_hash_left_join),
    (FullOuterJoin, is_full_outer_join),
    // Traverse the nodes
    (Expand, is_expand),
    (ExpandAll, is_expand_all),
    (Traverse, is_traverse),
    (AppendVertices, is_append_vertices),
    // Control flow nodes
    (Argument, is_argument),
    (Loop, is_loop),
    (PassThrough, is_pass_through),
    (Select, is_select),
    // Data processing node
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
    // Algorithm node
    (MultiShortestPath, is_multi_shortest_path),
    (BFSShortest, is_bfs_shortest),
    (AllPaths, is_all_paths),
    (ShortestPath, is_shortest_path),
    // Management Node – Space
    (CreateSpace, is_create_space),
    (DropSpace, is_drop_space),
    (DescSpace, is_desc_space),
    (ShowSpaces, is_show_spaces),
    (SwitchSpace, is_switch_space),
    (AlterSpace, is_alter_space),
    (ClearSpace, is_clear_space),
    // Management Node – Tags
    (CreateTag, is_create_tag),
    (AlterTag, is_alter_tag),
    (DescTag, is_desc_tag),
    (DropTag, is_drop_tag),
    (ShowTags, is_show_tags),
    // Management Node – Edge Type
    (CreateEdge, is_create_edge),
    (AlterEdge, is_alter_edge),
    (DescEdge, is_desc_edge),
    (DropEdge, is_drop_edge),
    (ShowEdges, is_show_edges),
    // Management Node – Index
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
    // Management Node – User
    (CreateUser, is_create_user),
    (AlterUser, is_alter_user),
    (DropUser, is_drop_user),
    (ChangePassword, is_change_password),
    (GrantRole, is_grant_role),
    (RevokeRole, is_revoke_role),
    // Management Node – Data
    (InsertVertices, is_insert_vertices),
    (InsertEdges, is_insert_edges),
    // Statistical nodes
    (ShowStats, is_show_stats),
}

// Use macros to generate the as_xxx method.
crate::define_enum_as_methods! {
    PlanNodeEnum,
    // Access node
    (Start, as_start, StartNode),
    (GetVertices, as_get_vertices, GetVerticesNode),
    (GetEdges, as_get_edges, GetEdgesNode),
    (GetNeighbors, as_get_neighbors, GetNeighborsNode),
    (ScanVertices, as_scan_vertices, ScanVerticesNode),
    (ScanEdges, as_scan_edges, ScanEdgesNode),
    (EdgeIndexScan, as_edge_index_scan, EdgeIndexScanNode),
    (IndexScan, as_index_scan, IndexScanNode),
    // Operation node
    (Project, as_project, ProjectNode),
    (Filter, as_filter, FilterNode),
    (Sort, as_sort, SortNode),
    (Limit, as_limit, LimitNode),
    (TopN, as_topn, TopNNode),
    (Sample, as_sample, SampleNode),
    (Dedup, as_dedup, DedupNode),
    (Aggregate, as_aggregate, AggregateNode),
    // Connecting nodes
    (InnerJoin, as_inner_join, InnerJoinNode),
    (LeftJoin, as_left_join, LeftJoinNode),
    (CrossJoin, as_cross_join, CrossJoinNode),
    (HashInnerJoin, as_hash_inner_join, HashInnerJoinNode),
    (HashLeftJoin, as_hash_left_join, HashLeftJoinNode),
    (FullOuterJoin, as_full_outer_join, FullOuterJoinNode),
    // Traverse the nodes
    (Expand, as_expand, ExpandNode),
    (ExpandAll, as_expand_all, ExpandAllNode),
    (Traverse, as_traverse, TraverseNode),
    (AppendVertices, as_append_vertices, AppendVerticesNode),
    // Control flow nodes
    (Argument, as_argument, ArgumentNode),
    (Loop, as_loop, LoopNode),
    (PassThrough, as_pass_through, PassThroughNode),
    (Select, as_select, SelectNode),
    // Data processing node
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
    // Algorithm node
    (MultiShortestPath, as_multi_shortest_path, MultiShortestPathNode),
    (BFSShortest, as_bfs_shortest, BFSShortestNode),
    (AllPaths, as_all_paths, AllPathsNode),
    (ShortestPath, as_shortest_path, ShortestPathNode),
    // Management Node – Space
    (CreateSpace, as_create_space, CreateSpaceNode),
    (DropSpace, as_drop_space, DropSpaceNode),
    (DescSpace, as_desc_space, DescSpaceNode),
    (ShowSpaces, as_show_spaces, ShowSpacesNode),
    (SwitchSpace, as_switch_space, SwitchSpaceNode),
    (AlterSpace, as_alter_space, AlterSpaceNode),
    (ClearSpace, as_clear_space, ClearSpaceNode),
    // Management Node – Tags
    (CreateTag, as_create_tag, CreateTagNode),
    (AlterTag, as_alter_tag, AlterTagNode),
    (DescTag, as_desc_tag, DescTagNode),
    (DropTag, as_drop_tag, DropTagNode),
    (ShowTags, as_show_tags, ShowTagsNode),
    // Management Node – Edge Type
    (CreateEdge, as_create_edge, CreateEdgeNode),
    (AlterEdge, as_alter_edge, AlterEdgeNode),
    (DescEdge, as_desc_edge, DescEdgeNode),
    (DropEdge, as_drop_edge, DropEdgeNode),
    (ShowEdges, as_show_edges, ShowEdgesNode),
    // Management Node – Index
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
    // Management Node – User
    (CreateUser, as_create_user, CreateUserNode),
    (AlterUser, as_alter_user, AlterUserNode),
    (DropUser, as_drop_user, DropUserNode),
    (ChangePassword, as_change_password, ChangePasswordNode),
    (GrantRole, as_grant_role, GrantRoleNode),
    (RevokeRole, as_revoke_role, RevokeRoleNode),
    // Management Node – Data
    (InsertVertices, as_insert_vertices, InsertVerticesNode),
    (InsertEdges, as_insert_edges, InsertEdgesNode),
    // Statistical node
    (ShowStats, as_show_stats, ShowStatsNode),
}

// Use macros to generate the _xxx_mut method.
crate::define_enum_as_mut_methods! {
    PlanNodeEnum,
    // Access node
    (Start, as_start_mut, StartNode),
    (GetVertices, as_get_vertices_mut, GetVerticesNode),
    (GetEdges, as_get_edges_mut, GetEdgesNode),
    (GetNeighbors, as_get_neighbors_mut, GetNeighborsNode),
    (ScanVertices, as_scan_vertices_mut, ScanVerticesNode),
    (ScanEdges, as_scan_edges_mut, ScanEdgesNode),
    (EdgeIndexScan, as_edge_index_scan_mut, EdgeIndexScanNode),
    (IndexScan, as_index_scan_mut, IndexScanNode),
    // Operation node
    (Project, as_project_mut, ProjectNode),
    (Filter, as_filter_mut, FilterNode),
    (Sort, as_sort_mut, SortNode),
    (Limit, as_limit_mut, LimitNode),
    (TopN, as_topn_mut, TopNNode),
    (Sample, as_sample_mut, SampleNode),
    (Dedup, as_dedup_mut, DedupNode),
    (Aggregate, as_aggregate_mut, AggregateNode),
    // Connecting nodes
    (InnerJoin, as_inner_join_mut, InnerJoinNode),
    (LeftJoin, as_left_join_mut, LeftJoinNode),
    (CrossJoin, as_cross_join_mut, CrossJoinNode),
    (HashInnerJoin, as_hash_inner_join_mut, HashInnerJoinNode),
    (HashLeftJoin, as_hash_left_join_mut, HashLeftJoinNode),
    (FullOuterJoin, as_full_outer_join_mut, FullOuterJoinNode),
    // Traverse the nodes
    (Expand, as_expand_mut, ExpandNode),
    (ExpandAll, as_expand_all_mut, ExpandAllNode),
    (Traverse, as_traverse_mut, TraverseNode),
    (AppendVertices, as_append_vertices_mut, AppendVerticesNode),
    // Control flow nodes
    (Argument, as_argument_mut, ArgumentNode),
    (Loop, as_loop_mut, LoopNode),
    (PassThrough, as_pass_through_mut, PassThroughNode),
    (Select, as_select_mut, SelectNode),
    // Data processing node
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
    // Algorithm node
    (MultiShortestPath, as_multi_shortest_path_mut, MultiShortestPathNode),
    (BFSShortest, as_bfs_shortest_mut, BFSShortestNode),
    (AllPaths, as_all_paths_mut, AllPathsNode),
    (ShortestPath, as_shortest_path_mut, ShortestPathNode),
    // Management Node – Space
    (CreateSpace, as_create_space_mut, CreateSpaceNode),
    (DropSpace, as_drop_space_mut, DropSpaceNode),
    (DescSpace, as_desc_space_mut, DescSpaceNode),
    (ShowSpaces, as_show_spaces_mut, ShowSpacesNode),
    (SwitchSpace, as_switch_space_mut, SwitchSpaceNode),
    (AlterSpace, as_alter_space_mut, AlterSpaceNode),
    (ClearSpace, as_clear_space_mut, ClearSpaceNode),
    // Management Node – Tags
    (CreateTag, as_create_tag_mut, CreateTagNode),
    (AlterTag, as_alter_tag_mut, AlterTagNode),
    (DescTag, as_desc_tag_mut, DescTagNode),
    (DropTag, as_drop_tag_mut, DropTagNode),
    (ShowTags, as_show_tags_mut, ShowTagsNode),
    // Management Node – Edge Type
    (CreateEdge, as_create_edge_mut, CreateEdgeNode),
    (AlterEdge, as_alter_edge_mut, AlterEdgeNode),
    (DescEdge, as_desc_edge_mut, DescEdgeNode),
    (DropEdge, as_drop_edge_mut, DropEdgeNode),
    (ShowEdges, as_show_edges_mut, ShowEdgesNode),
    // Management Node – Index
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
    // Management Node – User
    (CreateUser, as_create_user_mut, CreateUserNode),
    (AlterUser, as_alter_user_mut, AlterUserNode),
    (DropUser, as_drop_user_mut, DropUserNode),
    (ChangePassword, as_change_password_mut, ChangePasswordNode),
    (GrantRole, as_grant_role_mut, GrantRoleNode),
    (RevokeRole, as_revoke_role_mut, RevokeRoleNode),
    // Management Node – Data
    (InsertVertices, as_insert_vertices_mut, InsertVerticesNode),
    (InsertEdges, as_insert_edges_mut, InsertEdgesNode),
    // Statistical nodes
    (ShowStats, as_show_stats_mut, ShowStatsNode),
}

// Use macros to generate the type_name method.
crate::define_enum_type_name! {
    PlanNodeEnum,
    // Access node
    (Start, "Start"),
    (GetVertices, "GetVertices"),
    (GetEdges, "GetEdges"),
    (GetNeighbors, "GetNeighbors"),
    (ScanVertices, "ScanVertices"),
    (ScanEdges, "ScanEdges"),
    (EdgeIndexScan, "EdgeIndexScan"),
    (IndexScan, "IndexScan"),
    // Operation node
    (Project, "Project"),
    (Filter, "Filter"),
    (Sort, "Sort"),
    (Limit, "Limit"),
    (TopN, "TopN"),
    (Sample, "Sample"),
    (Dedup, "Dedup"),
    (Aggregate, "Aggregate"),
    // Connecting nodes
    (InnerJoin, "InnerJoin"),
    (LeftJoin, "LeftJoin"),
    (CrossJoin, "CrossJoin"),
    (HashInnerJoin, "HashInnerJoin"),
    (HashLeftJoin, "HashLeftJoin"),
    (FullOuterJoin, "FullOuterJoin"),
    // Traverse the nodes
    (Expand, "Expand"),
    (ExpandAll, "ExpandAll"),
    (Traverse, "Traverse"),
    (AppendVertices, "AppendVertices"),
    // Control flow nodes
    (Argument, "Argument"),
    (Loop, "Loop"),
    (PassThrough, "PassThrough"),
    (Select, "Select"),
    // Data processing node
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
    // Algorithm node
    (MultiShortestPath, "MultiShortestPath"),
    (BFSShortest, "BFSShortest"),
    (AllPaths, "AllPaths"),
    (ShortestPath, "ShortestPath"),
    // Management Node – Space
    (CreateSpace, "CreateSpace"),
    (DropSpace, "DropSpace"),
    (DescSpace, "DescSpace"),
    (ShowSpaces, "ShowSpaces"),
    (SwitchSpace, "SwitchSpace"),
    (AlterSpace, "AlterSpace"),
    (ClearSpace, "ClearSpace"),
    // Management Node - Tags
    (CreateTag, "CreateTag"),
    (AlterTag, "AlterTag"),
    (DescTag, "DescTag"),
    (DropTag, "DropTag"),
    (ShowTags, "ShowTags"),
    // Management Node – Edge Type
    (CreateEdge, "CreateEdge"),
    (AlterEdge, "AlterEdge"),
    (DescEdge, "DescEdge"),
    (DropEdge, "DropEdge"),
    (ShowEdges, "ShowEdges"),
    // Management Node – Index
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
    // Management Node – User
    (CreateUser, "CreateUser"),
    (AlterUser, "AlterUser"),
    (DropUser, "DropUser"),
    (ChangePassword, "ChangePassword"),
    (GrantRole, "GrantRole"),
    (RevokeRole, "RevokeRole"),
    // Management Node – Data
    (InsertVertices, "InsertVertices"),
    (InsertEdges, "InsertEdges"),
    // Statistical node
    (ShowStats, "ShowStats"),
    // Show Create Tag node
    (ShowCreateTag, "ShowCreateTag"),
}

// Use macros to generate the `category` method.
crate::define_enum_category! {
    PlanNodeEnum,
    // Access node
    (Start, PlanNodeCategory::Access),
    (GetVertices, PlanNodeCategory::Access),
    (GetEdges, PlanNodeCategory::Access),
    (GetNeighbors, PlanNodeCategory::Access),
    (ScanVertices, PlanNodeCategory::Access),
    (ScanEdges, PlanNodeCategory::Access),
    (EdgeIndexScan, PlanNodeCategory::Access),
    (IndexScan, PlanNodeCategory::Access),
    // Operation node
    (Project, PlanNodeCategory::Operation),
    (Filter, PlanNodeCategory::Operation),
    (Sort, PlanNodeCategory::Operation),
    (Limit, PlanNodeCategory::Operation),
    (TopN, PlanNodeCategory::Operation),
    (Sample, PlanNodeCategory::Operation),
    (Dedup, PlanNodeCategory::Operation),
    (Aggregate, PlanNodeCategory::Operation),
    // Connecting nodes
    (InnerJoin, PlanNodeCategory::Join),
    (LeftJoin, PlanNodeCategory::Join),
    (CrossJoin, PlanNodeCategory::Join),
    (HashInnerJoin, PlanNodeCategory::Join),
    (HashLeftJoin, PlanNodeCategory::Join),
    (FullOuterJoin, PlanNodeCategory::Join),
    // Traverse the nodes
    (Expand, PlanNodeCategory::Traversal),
    (ExpandAll, PlanNodeCategory::Traversal),
    (Traverse, PlanNodeCategory::Traversal),
    (AppendVertices, PlanNodeCategory::Traversal),
    // Control flow nodes
    (Argument, PlanNodeCategory::ControlFlow),
    (Loop, PlanNodeCategory::ControlFlow),
    (PassThrough, PlanNodeCategory::ControlFlow),
    (Select, PlanNodeCategory::ControlFlow),
    // Data processing node
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
    // Algorithm node
    (MultiShortestPath, PlanNodeCategory::Algorithm),
    (BFSShortest, PlanNodeCategory::Algorithm),
    (AllPaths, PlanNodeCategory::Algorithm),
    (ShortestPath, PlanNodeCategory::Algorithm),
    // Management Node
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
    (ShowCreateTag, PlanNodeCategory::Management),
}

// Use macros to generate the describe method.
crate::define_enum_describe! {
    PlanNodeEnum,
    // Access node
    (Start, "Start"),
    (GetVertices, "GetVertices"),
    (GetEdges, "GetEdges"),
    (GetNeighbors, "GetNeighbors"),
    (ScanVertices, "ScanVertices"),
    (ScanEdges, "ScanEdges"),
    (EdgeIndexScan, "EdgeIndexScan"),
    (IndexScan, "IndexScan"),
    // Operation node
    (Project, "Project"),
    (Filter, "Filter"),
    (Sort, "Sort"),
    (Limit, "Limit"),
    (TopN, "TopN"),
    (Sample, "Sample"),
    (Dedup, "Dedup"),
    (Aggregate, "Aggregate"),
    // Connecting nodes
    (InnerJoin, "InnerJoin"),
    (LeftJoin, "LeftJoin"),
    (CrossJoin, "CrossJoin"),
    (HashInnerJoin, "HashInnerJoin"),
    (HashLeftJoin, "HashLeftJoin"),
    (FullOuterJoin, "FullOuterJoin"),
    // Traverse the nodes
    (Expand, "Expand"),
    (ExpandAll, "ExpandAll"),
    (Traverse, "Traverse"),
    (AppendVertices, "AppendVertices"),
    // Control flow nodes
    (Argument, "Argument"),
    (Loop, "Loop"),
    (PassThrough, "PassThrough"),
    (Select, "Select"),
    // Data processing node
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
    // Algorithm node
    (MultiShortestPath, "MultiShortestPath"),
    (BFSShortest, "BFSShortest"),
    (AllPaths, "AllPaths"),
    (ShortestPath, "ShortestPath"),
    // Management Node – Space
    (CreateSpace, "CreateSpace"),
    (DropSpace, "DropSpace"),
    (DescSpace, "DescSpace"),
    (ShowSpaces, "ShowSpaces"),
    (SwitchSpace, "SwitchSpace"),
    (AlterSpace, "AlterSpace"),
    (ClearSpace, "ClearSpace"),
    // Management Node – Tags
    (CreateTag, "CreateTag"),
    (AlterTag, "AlterTag"),
    (DescTag, "DescTag"),
    (DropTag, "DropTag"),
    (ShowTags, "ShowTags"),
    // Management Node – Edge Type
    (CreateEdge, "CreateEdge"),
    (AlterEdge, "AlterEdge"),
    (DescEdge, "DescEdge"),
    (DropEdge, "DropEdge"),
    (ShowEdges, "ShowEdges"),
    // Management Node – Index
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
    // Management Node – User
    (CreateUser, "CreateUser"),
    (AlterUser, "AlterUser"),
    (DropUser, "DropUser"),
    (ChangePassword, "ChangePassword"),
    (GrantRole, "GrantRole"),
    (RevokeRole, "RevokeRole"),
    // Management Node – Data
    (InsertVertices, "InsertVertices"),
    (InsertEdges, "InsertEdges"),
    // Statistical node
    (ShowStats, "ShowStats"),
    // Show Create Tag node
    (ShowCreateTag, "ShowCreateTag"),
}

impl PlanNodeEnum {
    /// Node cloning
    pub fn clone_plan_node(&self) -> PlanNodeEnum {
        self.clone()
    }

    /// Determine whether a node is an access node.
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

    /// Determine whether a node is an operation node.
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

    /// Determine whether a node is a connecting node.
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

    /// Determine whether a node is a traversable node.
    pub fn is_traversal(&self) -> bool {
        matches!(
            self,
            PlanNodeEnum::Expand(_)
                | PlanNodeEnum::ExpandAll(_)
                | PlanNodeEnum::Traverse(_)
                | PlanNodeEnum::AppendVertices(_)
        )
    }

    /// Determine whether a node is a control flow node.
    pub fn is_control_flow(&self) -> bool {
        matches!(
            self,
            PlanNodeEnum::Argument(_)
                | PlanNodeEnum::Loop(_)
                | PlanNodeEnum::PassThrough(_)
                | PlanNodeEnum::Select(_)
        )
    }

    /// Determine whether a node is a data processing node.
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

    /// Determine whether a node is an algorithm node.
    pub fn is_algorithm(&self) -> bool {
        matches!(
            self,
            PlanNodeEnum::ShortestPath(_)
                | PlanNodeEnum::AllPaths(_)
                | PlanNodeEnum::MultiShortestPath(_)
                | PlanNodeEnum::BFSShortest(_)
        )
    }

    /// Determine whether a node is a management node.
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
                | PlanNodeEnum::ShowCreateTag(_)
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

    /// Determine whether a node is a query node.
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

impl PlanNodeEnum {
    /// Estimate memory usage for this node (in bytes)
    /// This is a recursive estimation that includes all child nodes
    /// Note: This method may count shared subtrees multiple times.
    /// For deduplicated estimation, use `estimate_memory_dedup`.
    pub fn estimate_memory(&self) -> usize {
        let base_size = std::mem::size_of::<PlanNodeEnum>();

        match self {
            // ZeroInputNode: Only the node structure itself
            PlanNodeEnum::Start(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::CreateSpace(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::DropSpace(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::DescSpace(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::ShowSpaces(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::CreateTag(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::AlterTag(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::DescTag(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::DropTag(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::ShowTags(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::ShowCreateTag(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::CreateEdge(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::AlterEdge(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::DescEdge(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::DropEdge(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::ShowEdges(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::CreateTagIndex(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::DropTagIndex(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::DescTagIndex(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::ShowTagIndexes(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::CreateEdgeIndex(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::DropEdgeIndex(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::DescEdgeIndex(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::ShowEdgeIndexes(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::RebuildTagIndex(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::RebuildEdgeIndex(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::CreateUser(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::AlterUser(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::DropUser(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::ChangePassword(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::GrantRole(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::RevokeRole(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::SwitchSpace(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::AlterSpace(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::ClearSpace(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::ShowStats(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::InsertVertices(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::InsertEdges(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::IndexScan(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::ScanVertices(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::ScanEdges(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::EdgeIndexScan(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::GetVertices(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::GetEdges(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::GetNeighbors(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::ShortestPath(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::AllPaths(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::BFSShortest(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::MultiShortestPath(node) => base_size + estimate_node_memory(node),

            // SingleInputNode: Node structure + child node
            PlanNodeEnum::Project(node) => {
                base_size + estimate_node_memory(node) + Self::estimate_input_memory(node.input())
            }
            PlanNodeEnum::Filter(node) => {
                base_size + estimate_node_memory(node) + Self::estimate_input_memory(node.input())
            }
            PlanNodeEnum::Sort(node) => {
                base_size + estimate_node_memory(node) + Self::estimate_input_memory(node.input())
            }
            PlanNodeEnum::Limit(node) => {
                base_size + estimate_node_memory(node) + Self::estimate_input_memory(node.input())
            }
            PlanNodeEnum::TopN(node) => {
                base_size + estimate_node_memory(node) + Self::estimate_input_memory(node.input())
            }
            PlanNodeEnum::Sample(node) => {
                base_size + estimate_node_memory(node) + Self::estimate_input_memory(node.input())
            }
            PlanNodeEnum::Dedup(node) => {
                base_size + estimate_node_memory(node) + Self::estimate_input_memory(node.input())
            }
            PlanNodeEnum::DataCollect(node) => {
                base_size + estimate_node_memory(node) + Self::estimate_input_memory(node.input())
            }
            PlanNodeEnum::Aggregate(node) => {
                base_size + estimate_node_memory(node) + Self::estimate_input_memory(node.input())
            }
            PlanNodeEnum::Unwind(node) => {
                base_size + estimate_node_memory(node) + Self::estimate_input_memory(node.input())
            }
            PlanNodeEnum::Assign(node) => {
                base_size + estimate_node_memory(node) + Self::estimate_input_memory(node.input())
            }
            PlanNodeEnum::PatternApply(node) => {
                base_size + estimate_node_memory(node) + Self::estimate_input_memory(node.input())
            }
            PlanNodeEnum::RollUpApply(node) => {
                base_size + estimate_node_memory(node) + Self::estimate_input_memory(node.input())
            }
            PlanNodeEnum::Remove(node) => {
                base_size + estimate_node_memory(node) + Self::estimate_input_memory(node.input())
            }
            PlanNodeEnum::Materialize(node) => {
                base_size + estimate_node_memory(node) + Self::estimate_input_memory(node.input())
            }
            PlanNodeEnum::Traverse(node) => {
                base_size + estimate_node_memory(node) + Self::estimate_input_memory(node.input())
            }

            // BinaryInputNode: Node structure + two child nodes
            PlanNodeEnum::InnerJoin(node) => {
                base_size
                    + estimate_node_memory(node)
                    + Self::estimate_input_memory(node.left_input())
                    + Self::estimate_input_memory(node.right_input())
            }
            PlanNodeEnum::LeftJoin(node) => {
                base_size
                    + estimate_node_memory(node)
                    + Self::estimate_input_memory(node.left_input())
                    + Self::estimate_input_memory(node.right_input())
            }
            PlanNodeEnum::CrossJoin(node) => {
                base_size
                    + estimate_node_memory(node)
                    + Self::estimate_input_memory(node.left_input())
                    + Self::estimate_input_memory(node.right_input())
            }
            PlanNodeEnum::HashInnerJoin(node) => {
                base_size
                    + estimate_node_memory(node)
                    + Self::estimate_input_memory(node.left_input())
                    + Self::estimate_input_memory(node.right_input())
            }
            PlanNodeEnum::HashLeftJoin(node) => {
                base_size
                    + estimate_node_memory(node)
                    + Self::estimate_input_memory(node.left_input())
                    + Self::estimate_input_memory(node.right_input())
            }
            PlanNodeEnum::FullOuterJoin(node) => {
                base_size
                    + estimate_node_memory(node)
                    + Self::estimate_input_memory(node.left_input())
                    + Self::estimate_input_memory(node.right_input())
            }

            // MultipleInputNode: Node structure + multiple child nodes
            PlanNodeEnum::Expand(node) => {
                base_size
                    + estimate_node_memory(node)
                    + node
                        .dependencies()
                        .iter()
                        .map(Self::estimate_input_memory)
                        .sum::<usize>()
            }
            PlanNodeEnum::ExpandAll(node) => {
                base_size
                    + estimate_node_memory(node)
                    + node
                        .dependencies()
                        .iter()
                        .map(Self::estimate_input_memory)
                        .sum::<usize>()
            }
            PlanNodeEnum::AppendVertices(node) => {
                base_size
                    + estimate_node_memory(node)
                    + node
                        .dependencies()
                        .iter()
                        .map(Self::estimate_input_memory)
                        .sum::<usize>()
            }
            PlanNodeEnum::Union(node) => {
                base_size
                    + estimate_node_memory(node)
                    + node
                        .dependencies()
                        .iter()
                        .map(Self::estimate_input_memory)
                        .sum::<usize>()
            }
            PlanNodeEnum::Minus(node) => {
                base_size
                    + estimate_node_memory(node)
                    + node
                        .dependencies()
                        .iter()
                        .map(Self::estimate_input_memory)
                        .sum::<usize>()
            }
            PlanNodeEnum::Intersect(node) => {
                base_size
                    + estimate_node_memory(node)
                    + node
                        .dependencies()
                        .iter()
                        .map(Self::estimate_input_memory)
                        .sum::<usize>()
            }

            // ControlFlowNode
            PlanNodeEnum::Argument(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::Loop(node) => {
                let mut total = base_size + estimate_node_memory(node);
                if let Some(body) = node.body() {
                    total += Self::estimate_input_memory(body.as_ref());
                }
                total
            }
            PlanNodeEnum::PassThrough(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::Select(node) => {
                let mut total = base_size + estimate_node_memory(node);
                if let Some(if_branch) = node.if_branch() {
                    total += Self::estimate_input_memory(if_branch.as_ref());
                }
                if let Some(else_branch) = node.else_branch() {
                    total += Self::estimate_input_memory(else_branch.as_ref());
                }
                total
            }
        }
    }

    /// Estimate memory with deduplication for shared subtrees
    /// This method uses a HashSet to track visited node IDs, ensuring each node
    /// is only counted once even if referenced multiple times via Arc.
    pub fn estimate_memory_dedup(&self) -> usize {
        let mut visited = HashSet::new();
        self.estimate_memory_internal(&mut visited)
    }

    /// Internal method for recursive estimation with deduplication
    fn estimate_memory_internal(&self, visited: &mut HashSet<i64>) -> usize {
        // Check if we've already visited this node
        if !visited.insert(self.id()) {
            return 0; // Already counted, only Arc overhead would be counted by caller
        }

        let base_size = std::mem::size_of::<PlanNodeEnum>();

        match self {
            // ZeroInputNode: Only the node structure itself
            PlanNodeEnum::Start(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::CreateSpace(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::DropSpace(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::DescSpace(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::ShowSpaces(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::CreateTag(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::AlterTag(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::DescTag(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::DropTag(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::ShowTags(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::ShowCreateTag(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::CreateEdge(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::AlterEdge(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::DescEdge(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::DropEdge(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::ShowEdges(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::CreateTagIndex(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::DropTagIndex(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::DescTagIndex(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::ShowTagIndexes(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::CreateEdgeIndex(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::DropEdgeIndex(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::DescEdgeIndex(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::ShowEdgeIndexes(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::RebuildTagIndex(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::RebuildEdgeIndex(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::CreateUser(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::AlterUser(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::DropUser(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::ChangePassword(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::GrantRole(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::RevokeRole(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::SwitchSpace(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::AlterSpace(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::ClearSpace(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::ShowStats(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::InsertVertices(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::InsertEdges(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::IndexScan(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::ScanVertices(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::ScanEdges(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::EdgeIndexScan(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::GetVertices(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::GetEdges(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::GetNeighbors(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::ShortestPath(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::AllPaths(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::BFSShortest(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::MultiShortestPath(node) => base_size + estimate_node_memory(node),

            // SingleInputNode: Node structure + child node
            PlanNodeEnum::Project(node) => {
                base_size
                    + estimate_node_memory(node)
                    + node.input().estimate_memory_internal(visited)
            }
            PlanNodeEnum::Filter(node) => {
                base_size
                    + estimate_node_memory(node)
                    + node.input().estimate_memory_internal(visited)
            }
            PlanNodeEnum::Sort(node) => {
                base_size
                    + estimate_node_memory(node)
                    + node.input().estimate_memory_internal(visited)
            }
            PlanNodeEnum::Limit(node) => {
                base_size
                    + estimate_node_memory(node)
                    + node.input().estimate_memory_internal(visited)
            }
            PlanNodeEnum::TopN(node) => {
                base_size
                    + estimate_node_memory(node)
                    + node.input().estimate_memory_internal(visited)
            }
            PlanNodeEnum::Sample(node) => {
                base_size
                    + estimate_node_memory(node)
                    + node.input().estimate_memory_internal(visited)
            }
            PlanNodeEnum::Dedup(node) => {
                base_size
                    + estimate_node_memory(node)
                    + node.input().estimate_memory_internal(visited)
            }
            PlanNodeEnum::DataCollect(node) => {
                base_size
                    + estimate_node_memory(node)
                    + node.input().estimate_memory_internal(visited)
            }
            PlanNodeEnum::Aggregate(node) => {
                base_size
                    + estimate_node_memory(node)
                    + node.input().estimate_memory_internal(visited)
            }
            PlanNodeEnum::Unwind(node) => {
                base_size
                    + estimate_node_memory(node)
                    + node.input().estimate_memory_internal(visited)
            }
            PlanNodeEnum::Assign(node) => {
                base_size
                    + estimate_node_memory(node)
                    + node.input().estimate_memory_internal(visited)
            }
            PlanNodeEnum::PatternApply(node) => {
                base_size
                    + estimate_node_memory(node)
                    + node.input().estimate_memory_internal(visited)
            }
            PlanNodeEnum::RollUpApply(node) => {
                base_size
                    + estimate_node_memory(node)
                    + node.input().estimate_memory_internal(visited)
            }
            PlanNodeEnum::Remove(node) => {
                base_size
                    + estimate_node_memory(node)
                    + node.input().estimate_memory_internal(visited)
            }
            PlanNodeEnum::Materialize(node) => {
                base_size
                    + estimate_node_memory(node)
                    + node.input().estimate_memory_internal(visited)
            }
            PlanNodeEnum::Traverse(node) => {
                base_size
                    + estimate_node_memory(node)
                    + node.input().estimate_memory_internal(visited)
            }

            // BinaryInputNode: Node structure + two child nodes
            PlanNodeEnum::InnerJoin(node) => {
                base_size
                    + estimate_node_memory(node)
                    + node.left_input().estimate_memory_internal(visited)
                    + node.right_input().estimate_memory_internal(visited)
            }
            PlanNodeEnum::LeftJoin(node) => {
                base_size
                    + estimate_node_memory(node)
                    + node.left_input().estimate_memory_internal(visited)
                    + node.right_input().estimate_memory_internal(visited)
            }
            PlanNodeEnum::CrossJoin(node) => {
                base_size
                    + estimate_node_memory(node)
                    + node.left_input().estimate_memory_internal(visited)
                    + node.right_input().estimate_memory_internal(visited)
            }
            PlanNodeEnum::HashInnerJoin(node) => {
                base_size
                    + estimate_node_memory(node)
                    + node.left_input().estimate_memory_internal(visited)
                    + node.right_input().estimate_memory_internal(visited)
            }
            PlanNodeEnum::HashLeftJoin(node) => {
                base_size
                    + estimate_node_memory(node)
                    + node.left_input().estimate_memory_internal(visited)
                    + node.right_input().estimate_memory_internal(visited)
            }
            PlanNodeEnum::FullOuterJoin(node) => {
                base_size
                    + estimate_node_memory(node)
                    + node.left_input().estimate_memory_internal(visited)
                    + node.right_input().estimate_memory_internal(visited)
            }

            // MultipleInputNode: Node structure + multiple child nodes
            PlanNodeEnum::Expand(node) => {
                base_size
                    + estimate_node_memory(node)
                    + node
                        .dependencies()
                        .iter()
                        .map(|dep| dep.estimate_memory_internal(visited))
                        .sum::<usize>()
            }
            PlanNodeEnum::ExpandAll(node) => {
                base_size
                    + estimate_node_memory(node)
                    + node
                        .dependencies()
                        .iter()
                        .map(|dep| dep.estimate_memory_internal(visited))
                        .sum::<usize>()
            }
            PlanNodeEnum::AppendVertices(node) => {
                base_size
                    + estimate_node_memory(node)
                    + node
                        .dependencies()
                        .iter()
                        .map(|dep| dep.estimate_memory_internal(visited))
                        .sum::<usize>()
            }
            PlanNodeEnum::Union(node) => {
                base_size
                    + estimate_node_memory(node)
                    + node
                        .dependencies()
                        .iter()
                        .map(|dep| dep.estimate_memory_internal(visited))
                        .sum::<usize>()
            }
            PlanNodeEnum::Minus(node) => {
                base_size
                    + estimate_node_memory(node)
                    + node
                        .dependencies()
                        .iter()
                        .map(|dep| dep.estimate_memory_internal(visited))
                        .sum::<usize>()
            }
            PlanNodeEnum::Intersect(node) => {
                base_size
                    + estimate_node_memory(node)
                    + node
                        .dependencies()
                        .iter()
                        .map(|dep| dep.estimate_memory_internal(visited))
                        .sum::<usize>()
            }

            // ControlFlowNode
            PlanNodeEnum::Argument(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::Loop(node) => {
                let mut total = base_size + estimate_node_memory(node);
                if let Some(body) = node.body() {
                    total += body.as_ref().estimate_memory_internal(visited);
                }
                total
            }
            PlanNodeEnum::PassThrough(node) => base_size + estimate_node_memory(node),
            PlanNodeEnum::Select(node) => {
                let mut total = base_size + estimate_node_memory(node);
                if let Some(if_branch) = node.if_branch() {
                    total += if_branch.as_ref().estimate_memory_internal(visited);
                }
                if let Some(else_branch) = node.else_branch() {
                    total += else_branch.as_ref().estimate_memory_internal(visited);
                }
                total
            }
        }
    }

    /// Estimate memory for an input reference (Arc<PlanNodeEnum>)
    fn estimate_input_memory(input: &PlanNodeEnum) -> usize {
        let arc_overhead = std::mem::size_of::<std::sync::Arc<PlanNodeEnum>>();
        let node_memory = input.estimate_memory();
        arc_overhead + node_memory
    }
}

/// Estimate memory for a node that implements MemoryEstimatable
fn estimate_node_memory<T: MemoryEstimatable>(node: &T) -> usize {
    node.estimate_memory()
}
