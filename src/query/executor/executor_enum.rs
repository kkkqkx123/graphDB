//! 执行器枚举定义
//!
//! 使用静态分发替代动态分发，所有执行器类型都包含在此枚举中
//! 通过为枚举实现 Executor trait，可以统一处理所有执行器类型

use std::fmt;
use std::fmt::{Debug, Formatter};

use crate::storage::StorageClient;

use super::admin::{
    AlterEdgeExecutor, AlterSpaceExecutor, AlterTagExecutor, AlterUserExecutor, AnalyzeExecutor,
    ChangePasswordExecutor, ClearSpaceExecutor, CreateEdgeExecutor, CreateEdgeIndexExecutor,
    CreateSpaceExecutor, CreateTagExecutor, CreateTagIndexExecutor, CreateUserExecutor,
    DescEdgeExecutor, DescEdgeIndexExecutor, DescSpaceExecutor, DescTagExecutor,
    DescTagIndexExecutor, DropEdgeExecutor, DropEdgeIndexExecutor, DropSpaceExecutor,
    DropTagExecutor, DropTagIndexExecutor, DropUserExecutor, GrantRoleExecutor,
    RebuildEdgeIndexExecutor, RebuildTagIndexExecutor, RevokeRoleExecutor, ShowEdgeIndexesExecutor,
    ShowEdgesExecutor, ShowSpacesExecutor, ShowStatsExecutor, ShowTagIndexesExecutor,
    ShowTagsExecutor, SwitchSpaceExecutor,
};
use super::base::{
    BaseExecutor, DBResult, ExecutionResult, Executor, ExecutorStats, InputExecutor, StartExecutor,
};
use super::data_access::{
    GetEdgesExecutor, GetNeighborsExecutor, GetPropExecutor, GetVerticesExecutor,
    IndexScanExecutor, ScanEdgesExecutor, ScanVerticesExecutor,
};
use super::data_modification::{InsertExecutor, RemoveExecutor};
use super::data_processing::graph_traversal::algorithms::BFSShortestExecutor;
use super::data_processing::graph_traversal::{
    algorithms::MultiShortestPathExecutor, AllPathsExecutor, ExpandAllExecutor, ExpandExecutor,
    ShortestPathExecutor, TraverseExecutor,
};
use super::data_processing::join::{
    CrossJoinExecutor, FullOuterJoinExecutor, HashInnerJoinExecutor, HashLeftJoinExecutor,
    InnerJoinExecutor, LeftJoinExecutor,
};
use super::data_processing::set_operations::{
    IntersectExecutor, MinusExecutor, UnionAllExecutor, UnionExecutor,
};
use super::data_processing::MaterializeExecutor;
use super::logic::{ForLoopExecutor, LoopExecutor, SelectExecutor, WhileLoopExecutor};
use super::pipeline_executors::{ArgumentExecutor, DataCollectExecutor, PassThroughExecutor};
use super::result_processing::transformations::{
    AppendVerticesExecutor, AssignExecutor, PatternApplyExecutor, RollUpApplyExecutor,
    UnwindExecutor,
};
use super::result_processing::{
    AggregateExecutor, DedupExecutor, FilterExecutor, GroupByExecutor, HavingExecutor,
    LimitExecutor, ProjectExecutor, SampleExecutor, SortExecutor, TopNExecutor,
};

/// 执行器枚举
///
/// 包含所有可能的执行器类型，使用静态分发实现多态
pub enum ExecutorEnum<S: StorageClient + Send + 'static> {
    Start(StartExecutor<S>),
    Base(BaseExecutor<S>),
    GetVertices(GetVerticesExecutor<S>),
    GetEdges(GetEdgesExecutor<S>),
    GetNeighbors(GetNeighborsExecutor<S>),
    GetProp(GetPropExecutor<S>),
    AllPaths(AllPathsExecutor<S>),
    Expand(ExpandExecutor<S>),
    ExpandAll(ExpandAllExecutor<S>),
    Traverse(TraverseExecutor<S>),
    ShortestPath(ShortestPathExecutor<S>),
    MultiShortestPath(MultiShortestPathExecutor<S>),
    InnerJoin(InnerJoinExecutor<S>),
    HashInnerJoin(HashInnerJoinExecutor<S>),
    LeftJoin(LeftJoinExecutor<S>),
    HashLeftJoin(HashLeftJoinExecutor<S>),
    FullOuterJoin(FullOuterJoinExecutor<S>),
    CrossJoin(CrossJoinExecutor<S>),
    Union(UnionExecutor<S>),
    UnionAll(UnionAllExecutor<S>),
    Minus(MinusExecutor<S>),
    Intersect(IntersectExecutor<S>),
    Filter(FilterExecutor<S>),
    Project(ProjectExecutor<S>),
    Limit(LimitExecutor<S>),
    Sort(SortExecutor<S>),
    TopN(TopNExecutor<S>),
    Sample(SampleExecutor<S>),
    Aggregate(AggregateExecutor<S>),
    GroupBy(GroupByExecutor<S>),
    Having(HavingExecutor<S>),
    Dedup(DedupExecutor<S>),
    Unwind(UnwindExecutor<S>),
    Assign(AssignExecutor<S>),
    Materialize(MaterializeExecutor<S>),
    AppendVertices(AppendVerticesExecutor<S>),
    RollUpApply(RollUpApplyExecutor<S>),
    PatternApply(PatternApplyExecutor<S>),
    Remove(RemoveExecutor<S>),
    InsertVertices(InsertExecutor<S>),
    InsertEdges(InsertExecutor<S>),
    Loop(LoopExecutor<S>),
    ForLoop(ForLoopExecutor<S>),
    WhileLoop(WhileLoopExecutor<S>),
    Select(SelectExecutor<S>),
    ScanEdges(ScanEdgesExecutor<S>),
    ScanVertices(ScanVerticesExecutor<S>),
    IndexScan(IndexScanExecutor<S>),
    Argument(ArgumentExecutor<S>),
    PassThrough(PassThroughExecutor<S>),
    DataCollect(DataCollectExecutor<S>),
    BFSShortest(BFSShortestExecutor<S>),
    ShowSpaces(ShowSpacesExecutor<S>),
    ShowTags(ShowTagsExecutor<S>),
    ShowEdges(ShowEdgesExecutor<S>),
    CreateTagIndex(CreateTagIndexExecutor<S>),
    DropTagIndex(DropTagIndexExecutor<S>),
    DescTagIndex(DescTagIndexExecutor<S>),
    ShowTagIndexes(ShowTagIndexesExecutor<S>),
    RebuildTagIndex(RebuildTagIndexExecutor<S>),
    CreateEdgeIndex(CreateEdgeIndexExecutor<S>),
    DropEdgeIndex(DropEdgeIndexExecutor<S>),
    DescEdgeIndex(DescEdgeIndexExecutor<S>),
    ShowEdgeIndexes(ShowEdgeIndexesExecutor<S>),
    RebuildEdgeIndex(RebuildEdgeIndexExecutor<S>),
    CreateSpace(CreateSpaceExecutor<S>),
    DropSpace(DropSpaceExecutor<S>),
    DescSpace(DescSpaceExecutor<S>),
    CreateTag(CreateTagExecutor<S>),
    AlterTag(AlterTagExecutor<S>),
    DescTag(DescTagExecutor<S>),
    DropTag(DropTagExecutor<S>),
    CreateEdge(CreateEdgeExecutor<S>),
    AlterEdge(AlterEdgeExecutor<S>),
    DescEdge(DescEdgeExecutor<S>),
    DropEdge(DropEdgeExecutor<S>),
    CreateUser(CreateUserExecutor<S>),
    AlterUser(AlterUserExecutor<S>),
    DropUser(DropUserExecutor<S>),
    ChangePassword(ChangePasswordExecutor<S>),
    GrantRole(GrantRoleExecutor<S>),
    RevokeRole(RevokeRoleExecutor<S>),
    SwitchSpace(SwitchSpaceExecutor<S>),
    AlterSpace(AlterSpaceExecutor<S>),
    ClearSpace(ClearSpaceExecutor<S>),
    ShowStats(ShowStatsExecutor<S>),
    Analyze(AnalyzeExecutor<S>),
}

impl<S: StorageClient + Send + 'static> Debug for ExecutorEnum<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let (variant_name, exec_name) = match self {
            ExecutorEnum::Start(exec) => ("Start", exec.name()),
            ExecutorEnum::Base(exec) => ("Base", exec.name()),
            ExecutorEnum::GetVertices(exec) => ("GetVertices", exec.name()),
            ExecutorEnum::GetEdges(exec) => ("GetEdges", exec.name()),
            ExecutorEnum::GetNeighbors(exec) => ("GetNeighbors", exec.name()),
            ExecutorEnum::GetProp(exec) => ("GetProp", exec.name()),
            ExecutorEnum::AllPaths(exec) => ("AllPaths", exec.name()),
            ExecutorEnum::Expand(exec) => ("Expand", exec.name()),
            ExecutorEnum::ExpandAll(exec) => ("ExpandAll", exec.name()),
            ExecutorEnum::Traverse(exec) => ("Traverse", exec.name()),
            ExecutorEnum::ShortestPath(exec) => ("ShortestPath", exec.name()),
            ExecutorEnum::MultiShortestPath(exec) => ("MultiShortestPath", exec.name()),
            ExecutorEnum::InnerJoin(exec) => ("InnerJoin", exec.name()),
            ExecutorEnum::HashInnerJoin(exec) => ("HashInnerJoin", exec.name()),
            ExecutorEnum::LeftJoin(exec) => ("LeftJoin", exec.name()),
            ExecutorEnum::HashLeftJoin(exec) => ("HashLeftJoin", exec.name()),
            ExecutorEnum::FullOuterJoin(exec) => ("FullOuterJoin", exec.name()),
            ExecutorEnum::CrossJoin(exec) => ("CrossJoin", exec.name()),
            ExecutorEnum::Union(exec) => ("Union", exec.name()),
            ExecutorEnum::UnionAll(exec) => ("UnionAll", exec.name()),
            ExecutorEnum::Minus(exec) => ("Minus", exec.name()),
            ExecutorEnum::Intersect(exec) => ("Intersect", exec.name()),
            ExecutorEnum::Filter(exec) => ("Filter", exec.name()),
            ExecutorEnum::Project(exec) => ("Project", exec.name()),
            ExecutorEnum::Limit(exec) => ("Limit", exec.name()),
            ExecutorEnum::Sort(exec) => ("Sort", exec.name()),
            ExecutorEnum::TopN(exec) => ("TopN", exec.name()),
            ExecutorEnum::Sample(exec) => ("Sample", exec.name()),
            ExecutorEnum::Aggregate(exec) => ("Aggregate", exec.name()),
            ExecutorEnum::GroupBy(exec) => ("GroupBy", exec.name()),
            ExecutorEnum::Having(exec) => ("Having", exec.name()),
            ExecutorEnum::Dedup(exec) => ("Dedup", exec.name()),
            ExecutorEnum::Unwind(exec) => ("Unwind", exec.name()),
            ExecutorEnum::Assign(exec) => ("Assign", exec.name()),
            ExecutorEnum::Materialize(exec) => ("Materialize", exec.name()),
            ExecutorEnum::AppendVertices(exec) => ("AppendVertices", exec.name()),
            ExecutorEnum::RollUpApply(exec) => ("RollUpApply", exec.name()),
            ExecutorEnum::PatternApply(exec) => ("PatternApply", exec.name()),
            ExecutorEnum::Remove(exec) => ("Remove", exec.name()),
            ExecutorEnum::InsertVertices(exec) => ("InsertVertices", exec.name()),
            ExecutorEnum::InsertEdges(exec) => ("InsertEdges", exec.name()),
            ExecutorEnum::Loop(exec) => ("Loop", exec.name()),
            ExecutorEnum::ForLoop(exec) => ("ForLoop", exec.name()),
            ExecutorEnum::WhileLoop(exec) => ("WhileLoop", exec.name()),
            ExecutorEnum::Select(exec) => ("Select", exec.name()),
            ExecutorEnum::ScanEdges(exec) => ("ScanEdges", exec.name()),
            ExecutorEnum::ScanVertices(exec) => ("ScanVertices", exec.name()),
            ExecutorEnum::IndexScan(exec) => ("IndexScan", exec.name()),
            ExecutorEnum::Argument(exec) => ("Argument", exec.name()),
            ExecutorEnum::PassThrough(exec) => ("PassThrough", exec.name()),
            ExecutorEnum::DataCollect(exec) => ("DataCollect", exec.name()),
            ExecutorEnum::BFSShortest(exec) => ("BFSShortest", exec.name()),
            ExecutorEnum::ShowSpaces(exec) => ("ShowSpaces", exec.name()),
            ExecutorEnum::ShowTags(exec) => ("ShowTags", exec.name()),
            ExecutorEnum::ShowEdges(exec) => ("ShowEdges", exec.name()),
            ExecutorEnum::CreateTagIndex(exec) => ("CreateTagIndex", exec.name()),
            ExecutorEnum::DropTagIndex(exec) => ("DropTagIndex", exec.name()),
            ExecutorEnum::DescTagIndex(exec) => ("DescTagIndex", exec.name()),
            ExecutorEnum::ShowTagIndexes(exec) => ("ShowTagIndexes", exec.name()),
            ExecutorEnum::RebuildTagIndex(exec) => ("RebuildTagIndex", exec.name()),
            ExecutorEnum::CreateEdgeIndex(exec) => ("CreateEdgeIndex", exec.name()),
            ExecutorEnum::DropEdgeIndex(exec) => ("DropEdgeIndex", exec.name()),
            ExecutorEnum::DescEdgeIndex(exec) => ("DescEdgeIndex", exec.name()),
            ExecutorEnum::ShowEdgeIndexes(exec) => ("ShowEdgeIndexes", exec.name()),
            ExecutorEnum::RebuildEdgeIndex(exec) => ("RebuildEdgeIndex", exec.name()),
            ExecutorEnum::CreateSpace(exec) => ("CreateSpace", exec.name()),
            ExecutorEnum::DropSpace(exec) => ("DropSpace", exec.name()),
            ExecutorEnum::DescSpace(exec) => ("DescSpace", exec.name()),
            ExecutorEnum::CreateTag(exec) => ("CreateTag", exec.name()),
            ExecutorEnum::AlterTag(exec) => ("AlterTag", exec.name()),
            ExecutorEnum::DescTag(exec) => ("DescTag", exec.name()),
            ExecutorEnum::DropTag(exec) => ("DropTag", exec.name()),
            ExecutorEnum::CreateEdge(exec) => ("CreateEdge", exec.name()),
            ExecutorEnum::AlterEdge(exec) => ("AlterEdge", exec.name()),
            ExecutorEnum::DescEdge(exec) => ("DescEdge", exec.name()),
            ExecutorEnum::DropEdge(exec) => ("DropEdge", exec.name()),
            ExecutorEnum::CreateUser(exec) => ("CreateUser", exec.name()),
            ExecutorEnum::AlterUser(exec) => ("AlterUser", exec.name()),
            ExecutorEnum::DropUser(exec) => ("DropUser", exec.name()),
            ExecutorEnum::ChangePassword(exec) => ("ChangePassword", exec.name()),
            ExecutorEnum::GrantRole(exec) => ("GrantRole", exec.name()),
            ExecutorEnum::RevokeRole(exec) => ("RevokeRole", exec.name()),
            ExecutorEnum::SwitchSpace(exec) => ("SwitchSpace", exec.name()),
            ExecutorEnum::AlterSpace(exec) => ("AlterSpace", exec.name()),
            ExecutorEnum::ClearSpace(exec) => ("ClearSpace", exec.name()),
            ExecutorEnum::ShowStats(exec) => ("ShowStats", exec.name()),
            ExecutorEnum::Analyze(exec) => ("Analyze", exec.name()),
        };
        f.write_str(&format!("ExecutorEnum::{}({})", variant_name, exec_name))
    }
}

impl<S: StorageClient + Send + 'static> ExecutorEnum<S> {
    pub fn id(&self) -> i64 {
        self::delegate_to_executor!(self, id)
    }

    pub fn name(&self) -> &str {
        self::delegate_to_executor!(self, name)
    }

    pub fn description(&self) -> &str {
        self.name()
    }

    pub fn stats(&self) -> &ExecutorStats {
        self::delegate_to_executor!(self, stats)
    }

    pub fn stats_mut(&mut self) -> &mut ExecutorStats {
        self::delegate_to_executor_mut!(self, stats_mut)
    }
}

impl<S: StorageClient + Send + 'static> Executor<S> for ExecutorEnum<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        self::delegate_to_executor_mut!(self, execute)
    }

    fn open(&mut self) -> DBResult<()> {
        self::delegate_to_executor_mut!(self, open)
    }

    fn close(&mut self) -> DBResult<()> {
        self::delegate_to_executor_mut!(self, close)
    }

    fn is_open(&self) -> bool {
        self::delegate_to_executor!(self, is_open)
    }

    fn id(&self) -> i64 {
        self.id()
    }

    fn name(&self) -> &str {
        self.name()
    }

    fn description(&self) -> &str {
        self.name()
    }

    fn stats(&self) -> &ExecutorStats {
        self.stats()
    }

    fn stats_mut(&mut self) -> &mut ExecutorStats {
        self.stats_mut()
    }
}

impl<S: StorageClient + Send + 'static> InputExecutor<S> for ExecutorEnum<S> {
    fn set_input(&mut self, input: ExecutorEnum<S>) {
        match self {
            ExecutorEnum::Filter(exec) => exec.set_input(input),
            ExecutorEnum::Project(exec) => exec.set_input(input),
            ExecutorEnum::Limit(exec) => exec.set_input(input),
            ExecutorEnum::Sort(exec) => exec.set_input(input),
            ExecutorEnum::TopN(exec) => exec.set_input(input),
            ExecutorEnum::Sample(exec) => exec.set_input(input),
            ExecutorEnum::Dedup(exec) => exec.set_input(input),
            ExecutorEnum::Expand(exec) => exec.set_input(input),
            ExecutorEnum::ExpandAll(exec) => exec.set_input(input),
            ExecutorEnum::Traverse(exec) => exec.set_input(input),
            ExecutorEnum::ShortestPath(exec) => exec.set_input(input),
            ExecutorEnum::Aggregate(exec) => exec.set_input(input),
            ExecutorEnum::GroupBy(exec) => exec.set_input(input),
            ExecutorEnum::Having(exec) => exec.set_input(input),
            ExecutorEnum::Remove(exec) => exec.set_input(input),
            ExecutorEnum::Materialize(exec) => exec.set_input(input),
            _ => {}
        }
    }

    fn get_input(&self) -> Option<&ExecutorEnum<S>> {
        match self {
            ExecutorEnum::Filter(exec) => exec.get_input(),
            ExecutorEnum::Project(exec) => exec.get_input(),
            ExecutorEnum::Limit(exec) => exec.get_input(),
            ExecutorEnum::Sort(exec) => exec.get_input(),
            ExecutorEnum::TopN(exec) => exec.get_input(),
            ExecutorEnum::Sample(exec) => exec.get_input(),
            ExecutorEnum::Dedup(exec) => exec.get_input(),
            ExecutorEnum::Expand(exec) => exec.get_input(),
            ExecutorEnum::ExpandAll(exec) => exec.get_input(),
            ExecutorEnum::Traverse(exec) => exec.get_input(),
            ExecutorEnum::ShortestPath(exec) => exec.get_input(),
            ExecutorEnum::MultiShortestPath(exec) => exec.get_input(),
            ExecutorEnum::Aggregate(exec) => exec.get_input(),
            ExecutorEnum::GroupBy(exec) => exec.get_input(),
            ExecutorEnum::Having(exec) => exec.get_input(),
            ExecutorEnum::Remove(exec) => exec.get_input(),
            ExecutorEnum::Materialize(exec) => exec.get_input(),
            _ => None,
        }
    }
}

pub trait ChainableExecutor<S: StorageClient + Send + 'static>:
    super::base::Executor<S> + InputExecutor<S>
{
    fn into_executor_enum(self) -> ExecutorEnum<S>
    where
        Self: Sized + 'static;
}

impl<S: StorageClient + Send + 'static> ChainableExecutor<S> for ExecutorEnum<S> {
    fn into_executor_enum(self) -> ExecutorEnum<S> {
        self
    }
}

use crate::query::core::{NodeCategory, NodeType};

impl<S: StorageClient + Send + 'static> NodeType for ExecutorEnum<S> {
    fn node_type_id(&self) -> &'static str {
        match self {
            ExecutorEnum::Start(_) => "start",
            ExecutorEnum::Base(_) => "base",
            ExecutorEnum::GetVertices(_) => "get_vertices",
            ExecutorEnum::GetEdges(_) => "get_edges",
            ExecutorEnum::GetNeighbors(_) => "get_neighbors",
            ExecutorEnum::GetProp(_) => "get_prop",
            ExecutorEnum::AllPaths(_) => "all_paths",
            ExecutorEnum::Expand(_) => "expand",
            ExecutorEnum::ExpandAll(_) => "expand_all",
            ExecutorEnum::Traverse(_) => "traverse",
            ExecutorEnum::ShortestPath(_) => "shortest_path",
            ExecutorEnum::MultiShortestPath(_) => "multi_shortest_path",
            ExecutorEnum::InnerJoin(_) => "inner_join",
            ExecutorEnum::HashInnerJoin(_) => "hash_inner_join",
            ExecutorEnum::LeftJoin(_) => "left_join",
            ExecutorEnum::HashLeftJoin(_) => "hash_left_join",
            ExecutorEnum::FullOuterJoin(_) => "full_outer_join",
            ExecutorEnum::CrossJoin(_) => "cross_join",
            ExecutorEnum::Union(_) => "union",
            ExecutorEnum::UnionAll(_) => "union_all",
            ExecutorEnum::Minus(_) => "minus",
            ExecutorEnum::Intersect(_) => "intersect",
            ExecutorEnum::Filter(_) => "filter",
            ExecutorEnum::Project(_) => "project",
            ExecutorEnum::Limit(_) => "limit",
            ExecutorEnum::Sort(_) => "sort",
            ExecutorEnum::TopN(_) => "topn",
            ExecutorEnum::Sample(_) => "sample",
            ExecutorEnum::Aggregate(_) => "aggregate",
            ExecutorEnum::GroupBy(_) => "group_by",
            ExecutorEnum::Having(_) => "having",
            ExecutorEnum::Dedup(_) => "dedup",
            ExecutorEnum::Unwind(_) => "unwind",
            ExecutorEnum::Assign(_) => "assign",
            ExecutorEnum::Materialize(_) => "materialize",
            ExecutorEnum::AppendVertices(_) => "append_vertices",
            ExecutorEnum::RollUpApply(_) => "rollup_apply",
            ExecutorEnum::PatternApply(_) => "pattern_apply",
            ExecutorEnum::Remove(_) => "remove",
            ExecutorEnum::InsertVertices(_) => "insert_vertices",
            ExecutorEnum::InsertEdges(_) => "insert_edges",
            ExecutorEnum::Loop(_) => "loop",
            ExecutorEnum::ForLoop(_) => "for_loop",
            ExecutorEnum::WhileLoop(_) => "while_loop",
            ExecutorEnum::Select(_) => "select",
            ExecutorEnum::ScanEdges(_) => "scan_edges",
            ExecutorEnum::ScanVertices(_) => "scan_vertices",
            ExecutorEnum::IndexScan(_) => "index_scan",
            ExecutorEnum::Argument(_) => "argument",
            ExecutorEnum::PassThrough(_) => "pass_through",
            ExecutorEnum::DataCollect(_) => "data_collect",
            ExecutorEnum::BFSShortest(_) => "bfs_shortest",
            ExecutorEnum::ShowSpaces(_) => "show_spaces",
            ExecutorEnum::ShowTags(_) => "show_tags",
            ExecutorEnum::ShowEdges(_) => "show_edges",
            ExecutorEnum::CreateTagIndex(_) => "create_tag_index",
            ExecutorEnum::DropTagIndex(_) => "drop_tag_index",
            ExecutorEnum::DescTagIndex(_) => "desc_tag_index",
            ExecutorEnum::ShowTagIndexes(_) => "show_tag_indexes",
            ExecutorEnum::RebuildTagIndex(_) => "rebuild_tag_index",
            ExecutorEnum::CreateEdgeIndex(_) => "create_edge_index",
            ExecutorEnum::DropEdgeIndex(_) => "drop_edge_index",
            ExecutorEnum::DescEdgeIndex(_) => "desc_edge_index",
            ExecutorEnum::ShowEdgeIndexes(_) => "show_edge_indexes",
            ExecutorEnum::RebuildEdgeIndex(_) => "rebuild_edge_index",
            ExecutorEnum::CreateSpace(_) => "create_space",
            ExecutorEnum::DropSpace(_) => "drop_space",
            ExecutorEnum::DescSpace(_) => "desc_space",
            ExecutorEnum::CreateTag(_) => "create_tag",
            ExecutorEnum::AlterTag(_) => "alter_tag",
            ExecutorEnum::DescTag(_) => "desc_tag",
            ExecutorEnum::DropTag(_) => "drop_tag",
            ExecutorEnum::CreateEdge(_) => "create_edge",
            ExecutorEnum::AlterEdge(_) => "alter_edge",
            ExecutorEnum::DescEdge(_) => "desc_edge",
            ExecutorEnum::DropEdge(_) => "drop_edge",
            ExecutorEnum::CreateUser(_) => "create_user",
            ExecutorEnum::AlterUser(_) => "alter_user",
            ExecutorEnum::DropUser(_) => "drop_user",
            ExecutorEnum::ChangePassword(_) => "change_password",
            ExecutorEnum::GrantRole(_) => "grant_role",
            ExecutorEnum::RevokeRole(_) => "revoke_role",
            ExecutorEnum::SwitchSpace(_) => "switch_space",
            ExecutorEnum::AlterSpace(_) => "alter_space",
            ExecutorEnum::ClearSpace(_) => "clear_space",
            ExecutorEnum::ShowStats(_) => "show_stats",
            ExecutorEnum::Analyze(_) => "analyze",
        }
    }

    fn node_type_name(&self) -> &'static str {
        match self {
            ExecutorEnum::Start(_) => "Start",
            ExecutorEnum::Base(_) => "Base",
            ExecutorEnum::GetVertices(_) => "Get Vertices",
            ExecutorEnum::GetEdges(_) => "Get Edges",
            ExecutorEnum::GetNeighbors(_) => "Get Neighbors",
            ExecutorEnum::GetProp(_) => "Get Properties",
            ExecutorEnum::AllPaths(_) => "All Paths",
            ExecutorEnum::Expand(_) => "Expand",
            ExecutorEnum::ExpandAll(_) => "Expand All",
            ExecutorEnum::Traverse(_) => "Traverse",
            ExecutorEnum::ShortestPath(_) => "Shortest Path",
            ExecutorEnum::MultiShortestPath(_) => "Multi Shortest Path",
            ExecutorEnum::InnerJoin(_) => "Inner Join",
            ExecutorEnum::HashInnerJoin(_) => "Hash Inner Join",
            ExecutorEnum::LeftJoin(_) => "Left Join",
            ExecutorEnum::HashLeftJoin(_) => "Hash Left Join",
            ExecutorEnum::FullOuterJoin(_) => "Full Outer Join",
            ExecutorEnum::CrossJoin(_) => "Cross Join",
            ExecutorEnum::Union(_) => "Union",
            ExecutorEnum::UnionAll(_) => "Union All",
            ExecutorEnum::Minus(_) => "Minus",
            ExecutorEnum::Intersect(_) => "Intersect",
            ExecutorEnum::Filter(_) => "Filter",
            ExecutorEnum::Project(_) => "Project",
            ExecutorEnum::Limit(_) => "Limit",
            ExecutorEnum::Sort(_) => "Sort",
            ExecutorEnum::TopN(_) => "Top N",
            ExecutorEnum::Sample(_) => "Sample",
            ExecutorEnum::Aggregate(_) => "Aggregate",
            ExecutorEnum::GroupBy(_) => "Group By",
            ExecutorEnum::Having(_) => "Having",
            ExecutorEnum::Dedup(_) => "Dedup",
            ExecutorEnum::Unwind(_) => "Unwind",
            ExecutorEnum::Assign(_) => "Assign",
            ExecutorEnum::Materialize(_) => "Materialize",
            ExecutorEnum::AppendVertices(_) => "Append Vertices",
            ExecutorEnum::RollUpApply(_) => "RollUp Apply",
            ExecutorEnum::PatternApply(_) => "Pattern Apply",
            ExecutorEnum::Remove(_) => "Remove",
            ExecutorEnum::InsertVertices(_) => "Insert Vertices",
            ExecutorEnum::InsertEdges(_) => "Insert Edges",
            ExecutorEnum::Loop(_) => "Loop",
            ExecutorEnum::ForLoop(_) => "For Loop",
            ExecutorEnum::WhileLoop(_) => "While Loop",
            ExecutorEnum::Select(_) => "Select",
            ExecutorEnum::ScanEdges(_) => "Scan Edges",
            ExecutorEnum::ScanVertices(_) => "Scan Vertices",
            ExecutorEnum::IndexScan(_) => "Index Scan",
            ExecutorEnum::Argument(_) => "Argument",
            ExecutorEnum::PassThrough(_) => "Pass Through",
            ExecutorEnum::DataCollect(_) => "Data Collect",
            ExecutorEnum::BFSShortest(_) => "BFS Shortest",
            ExecutorEnum::ShowSpaces(_) => "Show Spaces",
            ExecutorEnum::ShowTags(_) => "Show Tags",
            ExecutorEnum::ShowEdges(_) => "Show Edges",
            ExecutorEnum::CreateTagIndex(_) => "Create Tag Index",
            ExecutorEnum::DropTagIndex(_) => "Drop Tag Index",
            ExecutorEnum::DescTagIndex(_) => "Desc Tag Index",
            ExecutorEnum::ShowTagIndexes(_) => "Show Tag Indexes",
            ExecutorEnum::RebuildTagIndex(_) => "Rebuild Tag Index",
            ExecutorEnum::CreateEdgeIndex(_) => "Create Edge Index",
            ExecutorEnum::DropEdgeIndex(_) => "Drop Edge Index",
            ExecutorEnum::DescEdgeIndex(_) => "Desc Edge Index",
            ExecutorEnum::ShowEdgeIndexes(_) => "Show Edge Indexes",
            ExecutorEnum::RebuildEdgeIndex(_) => "Rebuild Edge Index",
            ExecutorEnum::CreateSpace(_) => "Create Space",
            ExecutorEnum::DropSpace(_) => "Drop Space",
            ExecutorEnum::DescSpace(_) => "Desc Space",
            ExecutorEnum::CreateTag(_) => "Create Tag",
            ExecutorEnum::AlterTag(_) => "Alter Tag",
            ExecutorEnum::DescTag(_) => "Desc Tag",
            ExecutorEnum::DropTag(_) => "Drop Tag",
            ExecutorEnum::CreateEdge(_) => "Create Edge",
            ExecutorEnum::AlterEdge(_) => "Alter Edge",
            ExecutorEnum::DescEdge(_) => "Desc Edge",
            ExecutorEnum::DropEdge(_) => "Drop Edge",
            ExecutorEnum::CreateUser(_) => "Create User",
            ExecutorEnum::AlterUser(_) => "Alter User",
            ExecutorEnum::DropUser(_) => "Drop User",
            ExecutorEnum::ChangePassword(_) => "Change Password",
            ExecutorEnum::GrantRole(_) => "Grant Role",
            ExecutorEnum::RevokeRole(_) => "Revoke Role",
            ExecutorEnum::SwitchSpace(_) => "Switch Space",
            ExecutorEnum::AlterSpace(_) => "Alter Space",
            ExecutorEnum::ClearSpace(_) => "Clear Space",
            ExecutorEnum::ShowStats(_) => "Show Stats",
            ExecutorEnum::Analyze(_) => "Analyze",
        }
    }

    fn category(&self) -> NodeCategory {
        match self {
            ExecutorEnum::Start(_) => NodeCategory::Other,
            ExecutorEnum::Base(_) => NodeCategory::Other,
            ExecutorEnum::GetVertices(_) => NodeCategory::Scan,
            ExecutorEnum::GetEdges(_) => NodeCategory::Scan,
            ExecutorEnum::GetNeighbors(_) => NodeCategory::Scan,
            ExecutorEnum::GetProp(_) => NodeCategory::Scan,
            ExecutorEnum::AllPaths(_) => NodeCategory::Path,
            ExecutorEnum::Expand(_) => NodeCategory::Traversal,
            ExecutorEnum::ExpandAll(_) => NodeCategory::Traversal,
            ExecutorEnum::Traverse(_) => NodeCategory::Traversal,
            ExecutorEnum::ShortestPath(_) => NodeCategory::Path,
            ExecutorEnum::MultiShortestPath(_) => NodeCategory::Path,
            ExecutorEnum::InnerJoin(_) => NodeCategory::Join,
            ExecutorEnum::HashInnerJoin(_) => NodeCategory::Join,
            ExecutorEnum::LeftJoin(_) => NodeCategory::Join,
            ExecutorEnum::HashLeftJoin(_) => NodeCategory::Join,
            ExecutorEnum::FullOuterJoin(_) => NodeCategory::Join,
            ExecutorEnum::CrossJoin(_) => NodeCategory::Join,
            ExecutorEnum::Union(_) => NodeCategory::SetOp,
            ExecutorEnum::UnionAll(_) => NodeCategory::SetOp,
            ExecutorEnum::Minus(_) => NodeCategory::SetOp,
            ExecutorEnum::Intersect(_) => NodeCategory::SetOp,
            ExecutorEnum::Filter(_) => NodeCategory::Filter,
            ExecutorEnum::Project(_) => NodeCategory::Project,
            ExecutorEnum::Limit(_) => NodeCategory::Other,
            ExecutorEnum::Sort(_) => NodeCategory::Sort,
            ExecutorEnum::TopN(_) => NodeCategory::Other,
            ExecutorEnum::Sample(_) => NodeCategory::Other,
            ExecutorEnum::Aggregate(_) => NodeCategory::Aggregate,
            ExecutorEnum::GroupBy(_) => NodeCategory::Aggregate,
            ExecutorEnum::Having(_) => NodeCategory::Filter,
            ExecutorEnum::Dedup(_) => NodeCategory::Other,
            ExecutorEnum::Unwind(_) => NodeCategory::Other,
            ExecutorEnum::Assign(_) => NodeCategory::Other,
            ExecutorEnum::Materialize(_) => NodeCategory::Other,
            ExecutorEnum::AppendVertices(_) => NodeCategory::Traversal,
            ExecutorEnum::RollUpApply(_) => NodeCategory::Other,
            ExecutorEnum::PatternApply(_) => NodeCategory::Other,
            ExecutorEnum::Remove(_) => NodeCategory::Other,
            ExecutorEnum::InsertVertices(_) => NodeCategory::Other,
            ExecutorEnum::InsertEdges(_) => NodeCategory::Other,
            ExecutorEnum::Loop(_) => NodeCategory::Control,
            ExecutorEnum::ForLoop(_) => NodeCategory::Control,
            ExecutorEnum::WhileLoop(_) => NodeCategory::Control,
            ExecutorEnum::Select(_) => NodeCategory::Control,
            ExecutorEnum::ScanEdges(_) => NodeCategory::Scan,
            ExecutorEnum::ScanVertices(_) => NodeCategory::Scan,
            ExecutorEnum::IndexScan(_) => NodeCategory::Scan,
            ExecutorEnum::Argument(_) => NodeCategory::Other,
            ExecutorEnum::PassThrough(_) => NodeCategory::Other,
            ExecutorEnum::DataCollect(_) => NodeCategory::DataCollect,
            ExecutorEnum::BFSShortest(_) => NodeCategory::Path,
            ExecutorEnum::ShowSpaces(_) => NodeCategory::Admin,
            ExecutorEnum::ShowTags(_) => NodeCategory::Admin,
            ExecutorEnum::ShowEdges(_) => NodeCategory::Admin,
            ExecutorEnum::CreateTagIndex(_) => NodeCategory::Admin,
            ExecutorEnum::DropTagIndex(_) => NodeCategory::Admin,
            ExecutorEnum::DescTagIndex(_) => NodeCategory::Admin,
            ExecutorEnum::ShowTagIndexes(_) => NodeCategory::Admin,
            ExecutorEnum::RebuildTagIndex(_) => NodeCategory::Admin,
            ExecutorEnum::CreateEdgeIndex(_) => NodeCategory::Admin,
            ExecutorEnum::DropEdgeIndex(_) => NodeCategory::Admin,
            ExecutorEnum::DescEdgeIndex(_) => NodeCategory::Admin,
            ExecutorEnum::ShowEdgeIndexes(_) => NodeCategory::Admin,
            ExecutorEnum::RebuildEdgeIndex(_) => NodeCategory::Admin,
            ExecutorEnum::CreateSpace(_) => NodeCategory::Admin,
            ExecutorEnum::DropSpace(_) => NodeCategory::Admin,
            ExecutorEnum::DescSpace(_) => NodeCategory::Admin,
            ExecutorEnum::CreateTag(_) => NodeCategory::Admin,
            ExecutorEnum::AlterTag(_) => NodeCategory::Admin,
            ExecutorEnum::DescTag(_) => NodeCategory::Admin,
            ExecutorEnum::DropTag(_) => NodeCategory::Admin,
            ExecutorEnum::CreateEdge(_) => NodeCategory::Admin,
            ExecutorEnum::AlterEdge(_) => NodeCategory::Admin,
            ExecutorEnum::DescEdge(_) => NodeCategory::Admin,
            ExecutorEnum::DropEdge(_) => NodeCategory::Admin,
            ExecutorEnum::CreateUser(_) => NodeCategory::Admin,
            ExecutorEnum::AlterUser(_) => NodeCategory::Admin,
            ExecutorEnum::DropUser(_) => NodeCategory::Admin,
            ExecutorEnum::ChangePassword(_) => NodeCategory::Admin,
            ExecutorEnum::GrantRole(_) => NodeCategory::Admin,
            ExecutorEnum::RevokeRole(_) => NodeCategory::Admin,
            ExecutorEnum::SwitchSpace(_) => NodeCategory::Admin,
            ExecutorEnum::AlterSpace(_) => NodeCategory::Admin,
            ExecutorEnum::ClearSpace(_) => NodeCategory::Admin,
            ExecutorEnum::ShowStats(_) => NodeCategory::Admin,
            ExecutorEnum::Analyze(_) => NodeCategory::Admin,
        }
    }
}

/// 内部宏模块 - 用于简化 ExecutorEnum 的方法委托
mod macros {
    /// 委托给内部执行器的不可变方法
    macro_rules! delegate_to_executor {
        ($self:expr, $method:ident) => {
            match $self {
                ExecutorEnum::Start(exec) => exec.$method(),
                ExecutorEnum::Base(exec) => exec.$method(),
                ExecutorEnum::GetVertices(exec) => exec.$method(),
                ExecutorEnum::GetEdges(exec) => exec.$method(),
                ExecutorEnum::GetNeighbors(exec) => exec.$method(),
                ExecutorEnum::GetProp(exec) => exec.$method(),
                ExecutorEnum::AllPaths(exec) => exec.$method(),
                ExecutorEnum::Expand(exec) => exec.$method(),
                ExecutorEnum::ExpandAll(exec) => exec.$method(),
                ExecutorEnum::Traverse(exec) => exec.$method(),
                ExecutorEnum::ShortestPath(exec) => exec.$method(),
                ExecutorEnum::MultiShortestPath(exec) => exec.$method(),
                ExecutorEnum::InnerJoin(exec) => exec.$method(),
                ExecutorEnum::HashInnerJoin(exec) => exec.$method(),
                ExecutorEnum::LeftJoin(exec) => exec.$method(),
                ExecutorEnum::HashLeftJoin(exec) => exec.$method(),
                ExecutorEnum::FullOuterJoin(exec) => exec.$method(),
                ExecutorEnum::CrossJoin(exec) => exec.$method(),
                ExecutorEnum::Union(exec) => exec.$method(),
                ExecutorEnum::UnionAll(exec) => exec.$method(),
                ExecutorEnum::Minus(exec) => exec.$method(),
                ExecutorEnum::Intersect(exec) => exec.$method(),
                ExecutorEnum::Filter(exec) => exec.$method(),
                ExecutorEnum::Project(exec) => exec.$method(),
                ExecutorEnum::Limit(exec) => exec.$method(),
                ExecutorEnum::Sort(exec) => exec.$method(),
                ExecutorEnum::TopN(exec) => exec.$method(),
                ExecutorEnum::Sample(exec) => exec.$method(),
                ExecutorEnum::Aggregate(exec) => exec.$method(),
                ExecutorEnum::GroupBy(exec) => exec.$method(),
                ExecutorEnum::Having(exec) => exec.$method(),
                ExecutorEnum::Dedup(exec) => exec.$method(),
                ExecutorEnum::Unwind(exec) => exec.$method(),
                ExecutorEnum::Assign(exec) => exec.$method(),
                ExecutorEnum::Materialize(exec) => exec.$method(),
                ExecutorEnum::AppendVertices(exec) => exec.$method(),
                ExecutorEnum::RollUpApply(exec) => exec.$method(),
                ExecutorEnum::PatternApply(exec) => exec.$method(),
                ExecutorEnum::Remove(exec) => exec.$method(),
                ExecutorEnum::InsertVertices(exec) => exec.$method(),
                ExecutorEnum::InsertEdges(exec) => exec.$method(),
                ExecutorEnum::Loop(exec) => exec.$method(),
                ExecutorEnum::ForLoop(exec) => exec.$method(),
                ExecutorEnum::WhileLoop(exec) => exec.$method(),
                ExecutorEnum::Select(exec) => exec.$method(),
                ExecutorEnum::ScanEdges(exec) => exec.$method(),
                ExecutorEnum::ScanVertices(exec) => exec.$method(),
                ExecutorEnum::IndexScan(exec) => exec.$method(),
                ExecutorEnum::Argument(exec) => exec.$method(),
                ExecutorEnum::PassThrough(exec) => exec.$method(),
                ExecutorEnum::DataCollect(exec) => exec.$method(),
                ExecutorEnum::BFSShortest(exec) => exec.$method(),
                ExecutorEnum::ShowSpaces(exec) => exec.$method(),
                ExecutorEnum::ShowTags(exec) => exec.$method(),
                ExecutorEnum::ShowEdges(exec) => exec.$method(),
                ExecutorEnum::CreateTagIndex(exec) => exec.$method(),
                ExecutorEnum::DropTagIndex(exec) => exec.$method(),
                ExecutorEnum::DescTagIndex(exec) => exec.$method(),
                ExecutorEnum::ShowTagIndexes(exec) => exec.$method(),
                ExecutorEnum::RebuildTagIndex(exec) => exec.$method(),
                ExecutorEnum::CreateEdgeIndex(exec) => exec.$method(),
                ExecutorEnum::DropEdgeIndex(exec) => exec.$method(),
                ExecutorEnum::DescEdgeIndex(exec) => exec.$method(),
                ExecutorEnum::ShowEdgeIndexes(exec) => exec.$method(),
                ExecutorEnum::RebuildEdgeIndex(exec) => exec.$method(),
                ExecutorEnum::CreateSpace(exec) => exec.$method(),
                ExecutorEnum::DropSpace(exec) => exec.$method(),
                ExecutorEnum::DescSpace(exec) => exec.$method(),
                ExecutorEnum::CreateTag(exec) => exec.$method(),
                ExecutorEnum::AlterTag(exec) => exec.$method(),
                ExecutorEnum::DescTag(exec) => exec.$method(),
                ExecutorEnum::DropTag(exec) => exec.$method(),
                ExecutorEnum::CreateEdge(exec) => exec.$method(),
                ExecutorEnum::AlterEdge(exec) => exec.$method(),
                ExecutorEnum::DescEdge(exec) => exec.$method(),
                ExecutorEnum::DropEdge(exec) => exec.$method(),
                ExecutorEnum::CreateUser(exec) => exec.$method(),
                ExecutorEnum::AlterUser(exec) => exec.$method(),
                ExecutorEnum::DropUser(exec) => exec.$method(),
                ExecutorEnum::ChangePassword(exec) => exec.$method(),
                ExecutorEnum::GrantRole(exec) => exec.$method(),
                ExecutorEnum::RevokeRole(exec) => exec.$method(),
                ExecutorEnum::SwitchSpace(exec) => exec.$method(),
                ExecutorEnum::AlterSpace(exec) => exec.$method(),
                ExecutorEnum::ClearSpace(exec) => exec.$method(),
                ExecutorEnum::ShowStats(exec) => exec.$method(),
                ExecutorEnum::Analyze(exec) => exec.$method(),
            }
        };
    }

    /// 委托给内部执行器的可变方法
    macro_rules! delegate_to_executor_mut {
        ($self:expr, $method:ident) => {
            match $self {
                ExecutorEnum::Start(exec) => exec.$method(),
                ExecutorEnum::Base(exec) => exec.$method(),
                ExecutorEnum::GetVertices(exec) => exec.$method(),
                ExecutorEnum::GetEdges(exec) => exec.$method(),
                ExecutorEnum::GetNeighbors(exec) => exec.$method(),
                ExecutorEnum::GetProp(exec) => exec.$method(),
                ExecutorEnum::AllPaths(exec) => exec.$method(),
                ExecutorEnum::Expand(exec) => exec.$method(),
                ExecutorEnum::ExpandAll(exec) => exec.$method(),
                ExecutorEnum::Traverse(exec) => exec.$method(),
                ExecutorEnum::ShortestPath(exec) => exec.$method(),
                ExecutorEnum::MultiShortestPath(exec) => exec.$method(),
                ExecutorEnum::InnerJoin(exec) => exec.$method(),
                ExecutorEnum::HashInnerJoin(exec) => exec.$method(),
                ExecutorEnum::LeftJoin(exec) => exec.$method(),
                ExecutorEnum::HashLeftJoin(exec) => exec.$method(),
                ExecutorEnum::FullOuterJoin(exec) => exec.$method(),
                ExecutorEnum::CrossJoin(exec) => exec.$method(),
                ExecutorEnum::Union(exec) => exec.$method(),
                ExecutorEnum::UnionAll(exec) => exec.$method(),
                ExecutorEnum::Minus(exec) => exec.$method(),
                ExecutorEnum::Intersect(exec) => exec.$method(),
                ExecutorEnum::Filter(exec) => exec.$method(),
                ExecutorEnum::Project(exec) => exec.$method(),
                ExecutorEnum::Limit(exec) => exec.$method(),
                ExecutorEnum::Sort(exec) => exec.$method(),
                ExecutorEnum::TopN(exec) => exec.$method(),
                ExecutorEnum::Sample(exec) => exec.$method(),
                ExecutorEnum::Aggregate(exec) => exec.$method(),
                ExecutorEnum::GroupBy(exec) => exec.$method(),
                ExecutorEnum::Having(exec) => exec.$method(),
                ExecutorEnum::Dedup(exec) => exec.$method(),
                ExecutorEnum::Unwind(exec) => exec.$method(),
                ExecutorEnum::Assign(exec) => exec.$method(),
                ExecutorEnum::Materialize(exec) => exec.$method(),
                ExecutorEnum::AppendVertices(exec) => exec.$method(),
                ExecutorEnum::RollUpApply(exec) => exec.$method(),
                ExecutorEnum::PatternApply(exec) => exec.$method(),
                ExecutorEnum::Remove(exec) => exec.$method(),
                ExecutorEnum::InsertVertices(exec) => exec.$method(),
                ExecutorEnum::InsertEdges(exec) => exec.$method(),
                ExecutorEnum::Loop(exec) => exec.$method(),
                ExecutorEnum::ForLoop(exec) => exec.$method(),
                ExecutorEnum::WhileLoop(exec) => exec.$method(),
                ExecutorEnum::Select(exec) => exec.$method(),
                ExecutorEnum::ScanEdges(exec) => exec.$method(),
                ExecutorEnum::ScanVertices(exec) => exec.$method(),
                ExecutorEnum::IndexScan(exec) => exec.$method(),
                ExecutorEnum::Argument(exec) => exec.$method(),
                ExecutorEnum::PassThrough(exec) => exec.$method(),
                ExecutorEnum::DataCollect(exec) => exec.$method(),
                ExecutorEnum::BFSShortest(exec) => exec.$method(),
                ExecutorEnum::ShowSpaces(exec) => exec.$method(),
                ExecutorEnum::ShowTags(exec) => exec.$method(),
                ExecutorEnum::ShowEdges(exec) => exec.$method(),
                ExecutorEnum::CreateTagIndex(exec) => exec.$method(),
                ExecutorEnum::DropTagIndex(exec) => exec.$method(),
                ExecutorEnum::DescTagIndex(exec) => exec.$method(),
                ExecutorEnum::ShowTagIndexes(exec) => exec.$method(),
                ExecutorEnum::RebuildTagIndex(exec) => exec.$method(),
                ExecutorEnum::CreateEdgeIndex(exec) => exec.$method(),
                ExecutorEnum::DropEdgeIndex(exec) => exec.$method(),
                ExecutorEnum::DescEdgeIndex(exec) => exec.$method(),
                ExecutorEnum::ShowEdgeIndexes(exec) => exec.$method(),
                ExecutorEnum::RebuildEdgeIndex(exec) => exec.$method(),
                ExecutorEnum::CreateSpace(exec) => exec.$method(),
                ExecutorEnum::DropSpace(exec) => exec.$method(),
                ExecutorEnum::DescSpace(exec) => exec.$method(),
                ExecutorEnum::CreateTag(exec) => exec.$method(),
                ExecutorEnum::AlterTag(exec) => exec.$method(),
                ExecutorEnum::DescTag(exec) => exec.$method(),
                ExecutorEnum::DropTag(exec) => exec.$method(),
                ExecutorEnum::CreateEdge(exec) => exec.$method(),
                ExecutorEnum::AlterEdge(exec) => exec.$method(),
                ExecutorEnum::DescEdge(exec) => exec.$method(),
                ExecutorEnum::DropEdge(exec) => exec.$method(),
                ExecutorEnum::CreateUser(exec) => exec.$method(),
                ExecutorEnum::AlterUser(exec) => exec.$method(),
                ExecutorEnum::DropUser(exec) => exec.$method(),
                ExecutorEnum::ChangePassword(exec) => exec.$method(),
                ExecutorEnum::GrantRole(exec) => exec.$method(),
                ExecutorEnum::RevokeRole(exec) => exec.$method(),
                ExecutorEnum::SwitchSpace(exec) => exec.$method(),
                ExecutorEnum::AlterSpace(exec) => exec.$method(),
                ExecutorEnum::ClearSpace(exec) => exec.$method(),
                ExecutorEnum::ShowStats(exec) => exec.$method(),
                ExecutorEnum::Analyze(exec) => exec.$method(),
            }
        };
    }

    pub(crate) use delegate_to_executor;
    pub(crate) use delegate_to_executor_mut;
}

use macros::{delegate_to_executor, delegate_to_executor_mut};
