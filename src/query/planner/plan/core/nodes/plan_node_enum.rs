//! PlanNode 枚举定义
//!

use super::plan_node_category::PlanNodeCategory;
use super::space_nodes::{CreateSpaceNode, DescSpaceNode, DropSpaceNode, ShowSpacesNode};
use super::tag_nodes::{AlterTagNode, CreateTagNode, DescTagNode, DropTagNode, ShowTagsNode};
use super::edge_nodes::{AlterEdgeNode, CreateEdgeNode, DescEdgeNode, DropEdgeNode, ShowEdgesNode};
use super::index_nodes::{
    CreateEdgeIndexNode, CreateTagIndexNode, DescEdgeIndexNode, DescTagIndexNode,
    DropEdgeIndexNode, DropTagIndexNode, RebuildEdgeIndexNode, RebuildTagIndexNode,
    ShowEdgeIndexesNode, ShowTagIndexesNode,
};
use super::insert_nodes::{InsertEdgesNode, InsertVerticesNode};
use super::user_nodes::{AlterUserNode, ChangePasswordNode, CreateUserNode, DropUserNode};
use crate::query::planner::plan::core::explain::PlanNodeDescription;

// 导入并重新导出所有具体的节点类型
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

/// PlanNode 枚举，包含所有可能的节点类型
///
/// 这个枚举避免了动态分发的性能开销
#[derive(Debug, Clone)]
pub enum PlanNodeEnum {
    /// 起始节点
    Start(StartNode),
    /// 投影节点
    Project(ProjectNode),
    /// 排序节点
    Sort(SortNode),
    /// 限制节点
    Limit(LimitNode),
    /// TopN 节点
    TopN(TopNNode),
    /// 采样节点
    Sample(SampleNode),
    /// 内连接节点
    InnerJoin(InnerJoinNode),
    /// 左连接节点
    LeftJoin(LeftJoinNode),
    /// 交叉连接节点
    CrossJoin(CrossJoinNode),
    /// 获取顶点节点
    GetVertices(GetVerticesNode),
    /// 获取边节点
    GetEdges(GetEdgesNode),
    /// 获取邻居节点
    GetNeighbors(GetNeighborsNode),
    /// 扫描顶点节点
    ScanVertices(ScanVerticesNode),
    /// 扫描边节点
    ScanEdges(ScanEdgesNode),
    /// 边索引扫描节点
    EdgeIndexScan(EdgeIndexScanNode),
    /// 哈希内连接节点
    HashInnerJoin(HashInnerJoinNode),
    /// 哈希左连接节点
    HashLeftJoin(HashLeftJoinNode),
    /// 全外连接节点
    FullOuterJoin(FullOuterJoinNode),
    /// 索引扫描节点
    IndexScan(IndexScan),
    /// 扩展节点
    Expand(ExpandNode),
    /// 全扩展节点
    ExpandAll(ExpandAllNode),
    /// 遍历节点
    Traverse(TraverseNode),
    /// 追加顶点节点
    AppendVertices(AppendVerticesNode),
    /// 过滤节点
    Filter(FilterNode),
    /// 聚合节点
    Aggregate(AggregateNode),
    /// 参数节点
    Argument(ArgumentNode),
    /// 循环节点
    Loop(LoopNode),
    /// 透传节点
    PassThrough(PassThroughNode),
    /// 选择节点
    Select(SelectNode),
    /// 数据收集节点
    DataCollect(DataCollectNode),
    /// 去重节点
    Dedup(DedupNode),
    /// 模式应用节点
    PatternApply(PatternApplyNode),
    /// 滚动应用节点
    RollUpApply(RollUpApplyNode),
    /// 并集节点
    Union(UnionNode),
    /// 差集节点
    Minus(MinusNode),
    /// 交集节点
    Intersect(IntersectNode),
    /// 展开节点
    Unwind(UnwindNode),
    /// 赋值节点
    Assign(AssignNode),
    /// 多源最短路径节点
    MultiShortestPath(MultiShortestPath),
    /// BFS最短路径节点
    BFSShortest(BFSShortest),
    /// 所有路径节点
    AllPaths(AllPaths),
    /// 最短路径节点
    ShortestPath(ShortestPath),

    /// ========== 管理节点 ==========
    /// 创建图空间
    CreateSpace(CreateSpaceNode),
    /// 删除图空间
    DropSpace(DropSpaceNode),
    /// 描述图空间
    DescSpace(DescSpaceNode),
    /// 显示所有图空间
    ShowSpaces(ShowSpacesNode),
    /// 创建标签
    CreateTag(CreateTagNode),
    /// 修改标签
    AlterTag(AlterTagNode),
    /// 描述标签
    DescTag(DescTagNode),
    /// 删除标签
    DropTag(DropTagNode),
    /// 显示所有标签
    ShowTags(ShowTagsNode),
    /// 创建边类型
    CreateEdge(CreateEdgeNode),
    /// 修改边类型
    AlterEdge(AlterEdgeNode),
    /// 描述边类型
    DescEdge(DescEdgeNode),
    /// 删除边类型
    DropEdge(DropEdgeNode),
    /// 显示所有边类型
    ShowEdges(ShowEdgesNode),
    /// 创建标签索引
    CreateTagIndex(CreateTagIndexNode),
    /// 删除标签索引
    DropTagIndex(DropTagIndexNode),
    /// 描述标签索引
    DescTagIndex(DescTagIndexNode),
    /// 显示所有标签索引
    ShowTagIndexes(ShowTagIndexesNode),
    /// 创建边索引
    CreateEdgeIndex(CreateEdgeIndexNode),
    /// 删除边索引
    DropEdgeIndex(DropEdgeIndexNode),
    /// 描述边索引
    DescEdgeIndex(DescEdgeIndexNode),
    /// 显示所有边索引
    ShowEdgeIndexes(ShowEdgeIndexesNode),
    /// 重建标签索引
    RebuildTagIndex(RebuildTagIndexNode),
    /// 重建边索引
    RebuildEdgeIndex(RebuildEdgeIndexNode),
    /// 创建用户
    CreateUser(CreateUserNode),
    /// 修改用户
    AlterUser(AlterUserNode),
    /// 删除用户
    DropUser(DropUserNode),
    /// 修改密码
    ChangePassword(ChangePasswordNode),
    /// 插入顶点
    InsertVertices(InsertVerticesNode),
    /// 插入边
    InsertEdges(InsertEdgesNode),
}

impl Default for PlanNodeEnum {
    fn default() -> Self {
        PlanNodeEnum::Start(StartNode::new())
    }
}

impl PlanNodeEnum {
    /// 零成本类型检查 - 直接使用模式匹配
    pub fn is_start(&self) -> bool {
        matches!(self, PlanNodeEnum::Start(_))
    }

    pub fn is_project(&self) -> bool {
        matches!(self, PlanNodeEnum::Project(_))
    }

    pub fn is_filter(&self) -> bool {
        matches!(self, PlanNodeEnum::Filter(_))
    }

    pub fn is_sort(&self) -> bool {
        matches!(self, PlanNodeEnum::Sort(_))
    }

    pub fn is_limit(&self) -> bool {
        matches!(self, PlanNodeEnum::Limit(_))
    }

    pub fn is_dedup(&self) -> bool {
        matches!(self, PlanNodeEnum::Dedup(_))
    }

    pub fn is_loop(&self) -> bool {
        matches!(self, PlanNodeEnum::Loop(_))
    }

    pub fn is_argument(&self) -> bool {
        matches!(self, PlanNodeEnum::Argument(_))
    }

    pub fn is_pass_through(&self) -> bool {
        matches!(self, PlanNodeEnum::PassThrough(_))
    }

    pub fn is_data_collect(&self) -> bool {
        matches!(self, PlanNodeEnum::DataCollect(_))
    }

    pub fn is_bfs_shortest(&self) -> bool {
        matches!(self, PlanNodeEnum::BFSShortest(_))
    }

    pub fn is_all_paths(&self) -> bool {
        matches!(self, PlanNodeEnum::AllPaths(_))
    }

    pub fn is_multi_shortest_path(&self) -> bool {
        matches!(self, PlanNodeEnum::MultiShortestPath(_))
    }

    pub fn is_shortest_path(&self) -> bool {
        matches!(self, PlanNodeEnum::ShortestPath(_))
    }

    pub fn is_union(&self) -> bool {
        matches!(self, PlanNodeEnum::Union(_))
    }

    pub fn is_unwind(&self) -> bool {
        matches!(self, PlanNodeEnum::Unwind(_))
    }

    pub fn is_assign(&self) -> bool {
        matches!(self, PlanNodeEnum::Assign(_))
    }

    pub fn is_pattern_apply(&self) -> bool {
        matches!(self, PlanNodeEnum::PatternApply(_))
    }

    pub fn is_roll_up_apply(&self) -> bool {
        matches!(self, PlanNodeEnum::RollUpApply(_))
    }

    pub fn is_sample(&self) -> bool {
        matches!(self, PlanNodeEnum::Sample(_))
    }

    pub fn is_topn(&self) -> bool {
        matches!(self, PlanNodeEnum::TopN(_))
    }

    pub fn is_hash_inner_join(&self) -> bool {
        matches!(self, PlanNodeEnum::HashInnerJoin(_))
    }

    pub fn is_hash_left_join(&self) -> bool {
        matches!(self, PlanNodeEnum::HashLeftJoin(_))
    }

    pub fn is_index_scan(&self) -> bool {
        matches!(self, PlanNodeEnum::IndexScan(_))
    }

    pub fn is_expand_all(&self) -> bool {
        matches!(self, PlanNodeEnum::ExpandAll(_))
    }

    pub fn is_select(&self) -> bool {
        matches!(self, PlanNodeEnum::Select(_))
    }

    pub fn is_get_vertices(&self) -> bool {
        matches!(self, PlanNodeEnum::GetVertices(_))
    }

    pub fn is_get_edges(&self) -> bool {
        matches!(self, PlanNodeEnum::GetEdges(_))
    }

    pub fn is_get_neighbors(&self) -> bool {
        matches!(self, PlanNodeEnum::GetNeighbors(_))
    }

    pub fn is_scan_vertices(&self) -> bool {
        matches!(self, PlanNodeEnum::ScanVertices(_))
    }

    pub fn is_scan_edges(&self) -> bool {
        matches!(self, PlanNodeEnum::ScanEdges(_))
    }

    pub fn is_append_vertices(&self) -> bool {
        matches!(self, PlanNodeEnum::AppendVertices(_))
    }

    pub fn is_inner_join(&self) -> bool {
        matches!(self, PlanNodeEnum::InnerJoin(_))
    }

    pub fn is_left_join(&self) -> bool {
        matches!(self, PlanNodeEnum::LeftJoin(_))
    }

    pub fn is_cross_join(&self) -> bool {
        matches!(self, PlanNodeEnum::CrossJoin(_))
    }

    pub fn is_traverse(&self) -> bool {
        matches!(self, PlanNodeEnum::Traverse(_))
    }

    pub fn is_expand(&self) -> bool {
        matches!(self, PlanNodeEnum::Expand(_))
    }

    pub fn is_create_space(&self) -> bool {
        matches!(self, PlanNodeEnum::CreateSpace(_))
    }

    pub fn is_drop_space(&self) -> bool {
        matches!(self, PlanNodeEnum::DropSpace(_))
    }

    pub fn is_desc_space(&self) -> bool {
        matches!(self, PlanNodeEnum::DescSpace(_))
    }

    pub fn is_show_spaces(&self) -> bool {
        matches!(self, PlanNodeEnum::ShowSpaces(_))
    }

    pub fn is_create_tag(&self) -> bool {
        matches!(self, PlanNodeEnum::CreateTag(_))
    }

    pub fn is_alter_tag(&self) -> bool {
        matches!(self, PlanNodeEnum::AlterTag(_))
    }

    pub fn is_desc_tag(&self) -> bool {
        matches!(self, PlanNodeEnum::DescTag(_))
    }

    pub fn is_drop_tag(&self) -> bool {
        matches!(self, PlanNodeEnum::DropTag(_))
    }

    pub fn is_show_tags(&self) -> bool {
        matches!(self, PlanNodeEnum::ShowTags(_))
    }

    pub fn is_create_edge(&self) -> bool {
        matches!(self, PlanNodeEnum::CreateEdge(_))
    }

    pub fn is_alter_edge(&self) -> bool {
        matches!(self, PlanNodeEnum::AlterEdge(_))
    }

    pub fn is_desc_edge(&self) -> bool {
        matches!(self, PlanNodeEnum::DescEdge(_))
    }

    pub fn is_drop_edge(&self) -> bool {
        matches!(self, PlanNodeEnum::DropEdge(_))
    }

    pub fn is_show_edges(&self) -> bool {
        matches!(self, PlanNodeEnum::ShowEdges(_))
    }

    pub fn is_create_user(&self) -> bool {
        matches!(self, PlanNodeEnum::CreateUser(_))
    }

    pub fn is_alter_user(&self) -> bool {
        matches!(self, PlanNodeEnum::AlterUser(_))
    }

    pub fn is_drop_user(&self) -> bool {
        matches!(self, PlanNodeEnum::DropUser(_))
    }

    pub fn is_change_password(&self) -> bool {
        matches!(self, PlanNodeEnum::ChangePassword(_))
    }

    pub fn is_insert_vertices(&self) -> bool {
        matches!(self, PlanNodeEnum::InsertVertices(_))
    }

    pub fn is_insert_edges(&self) -> bool {
        matches!(self, PlanNodeEnum::InsertEdges(_))
    }

    pub fn is_create_tag_index(&self) -> bool {
        matches!(self, PlanNodeEnum::CreateTagIndex(_))
    }

    pub fn is_drop_tag_index(&self) -> bool {
        matches!(self, PlanNodeEnum::DropTagIndex(_))
    }

    pub fn is_desc_tag_index(&self) -> bool {
        matches!(self, PlanNodeEnum::DescTagIndex(_))
    }

    pub fn is_show_tag_indexes(&self) -> bool {
        matches!(self, PlanNodeEnum::ShowTagIndexes(_))
    }

    pub fn is_create_edge_index(&self) -> bool {
        matches!(self, PlanNodeEnum::CreateEdgeIndex(_))
    }

    pub fn is_drop_edge_index(&self) -> bool {
        matches!(self, PlanNodeEnum::DropEdgeIndex(_))
    }

    pub fn is_desc_edge_index(&self) -> bool {
        matches!(self, PlanNodeEnum::DescEdgeIndex(_))
    }

    pub fn is_show_edge_indexes(&self) -> bool {
        matches!(self, PlanNodeEnum::ShowEdgeIndexes(_))
    }

    pub fn is_rebuild_tag_index(&self) -> bool {
        matches!(self, PlanNodeEnum::RebuildTagIndex(_))
    }

    pub fn is_rebuild_edge_index(&self) -> bool {
        matches!(self, PlanNodeEnum::RebuildEdgeIndex(_))
    }

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

    pub fn is_join(&self) -> bool {
        matches!(
            self,
            PlanNodeEnum::InnerJoin(_)
                | PlanNodeEnum::LeftJoin(_)
                | PlanNodeEnum::CrossJoin(_)
                | PlanNodeEnum::HashInnerJoin(_)
                | PlanNodeEnum::HashLeftJoin(_)
        )
    }

    pub fn is_traversal(&self) -> bool {
        matches!(
            self,
            PlanNodeEnum::Expand(_)
                | PlanNodeEnum::ExpandAll(_)
                | PlanNodeEnum::Traverse(_)
                | PlanNodeEnum::AppendVertices(_)
        )
    }

    pub fn is_control_flow(&self) -> bool {
        matches!(
            self,
            PlanNodeEnum::Argument(_)
                | PlanNodeEnum::Loop(_)
                | PlanNodeEnum::PassThrough(_)
                | PlanNodeEnum::Select(_)
        )
    }

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

    pub fn is_algorithm(&self) -> bool {
        matches!(
            self,
            PlanNodeEnum::ShortestPath(_)
                | PlanNodeEnum::AllPaths(_)
                | PlanNodeEnum::MultiShortestPath(_)
                | PlanNodeEnum::BFSShortest(_)
        )
    }

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

    pub fn type_name(&self) -> &'static str {
        match self {
            PlanNodeEnum::Start(_) => "Start",
            PlanNodeEnum::Project(_) => "Project",
            PlanNodeEnum::Sort(_) => "Sort",
            PlanNodeEnum::Limit(_) => "Limit",
            PlanNodeEnum::TopN(_) => "TopN",
            PlanNodeEnum::Sample(_) => "Sample",
            PlanNodeEnum::InnerJoin(_) => "InnerJoin",
            PlanNodeEnum::LeftJoin(_) => "LeftJoin",
            PlanNodeEnum::CrossJoin(_) => "CrossJoin",
            PlanNodeEnum::GetVertices(_) => "GetVertices",
            PlanNodeEnum::GetEdges(_) => "GetEdges",
            PlanNodeEnum::GetNeighbors(_) => "GetNeighbors",
            PlanNodeEnum::ScanVertices(_) => "ScanVertices",
            PlanNodeEnum::ScanEdges(_) => "ScanEdges",
            PlanNodeEnum::EdgeIndexScan(_) => "EdgeIndexScan",
            PlanNodeEnum::HashInnerJoin(_) => "HashInnerJoin",
            PlanNodeEnum::HashLeftJoin(_) => "HashLeftJoin",
            PlanNodeEnum::FullOuterJoin(_) => "FullOuterJoin",
            PlanNodeEnum::IndexScan(_) => "IndexScan",
            PlanNodeEnum::Expand(_) => "Expand",
            PlanNodeEnum::ExpandAll(_) => "ExpandAll",
            PlanNodeEnum::Traverse(_) => "Traverse",
            PlanNodeEnum::AppendVertices(_) => "AppendVertices",
            PlanNodeEnum::Filter(_) => "Filter",
            PlanNodeEnum::Aggregate(_) => "Aggregate",
            PlanNodeEnum::Argument(_) => "Argument",
            PlanNodeEnum::Loop(_) => "Loop",
            PlanNodeEnum::PassThrough(_) => "PassThrough",
            PlanNodeEnum::Select(_) => "Select",
            PlanNodeEnum::DataCollect(_) => "DataCollect",
            PlanNodeEnum::Dedup(_) => "Dedup",
            PlanNodeEnum::PatternApply(_) => "PatternApply",
            PlanNodeEnum::RollUpApply(_) => "RollUpApply",
            PlanNodeEnum::Union(_) => "Union",
            PlanNodeEnum::Minus(_) => "Minus",
            PlanNodeEnum::Intersect(_) => "Intersect",
            PlanNodeEnum::Unwind(_) => "Unwind",
            PlanNodeEnum::Assign(_) => "Assign",
            PlanNodeEnum::MultiShortestPath(_) => "MultiShortestPath",
            PlanNodeEnum::BFSShortest(_) => "BFSShortest",
            PlanNodeEnum::AllPaths(_) => "AllPaths",
            PlanNodeEnum::ShortestPath(_) => "ShortestPath",
            PlanNodeEnum::CreateSpace(_) => "CreateSpace",
            PlanNodeEnum::DropSpace(_) => "DropSpace",
            PlanNodeEnum::DescSpace(_) => "DescSpace",
            PlanNodeEnum::ShowSpaces(_) => "ShowSpaces",
            PlanNodeEnum::CreateTag(_) => "CreateTag",
            PlanNodeEnum::AlterTag(_) => "AlterTag",
            PlanNodeEnum::DescTag(_) => "DescTag",
            PlanNodeEnum::DropTag(_) => "DropTag",
            PlanNodeEnum::ShowTags(_) => "ShowTags",
            PlanNodeEnum::CreateEdge(_) => "CreateEdge",
            PlanNodeEnum::AlterEdge(_) => "AlterEdge",
            PlanNodeEnum::DescEdge(_) => "DescEdge",
            PlanNodeEnum::DropEdge(_) => "DropEdge",
            PlanNodeEnum::ShowEdges(_) => "ShowEdges",
            PlanNodeEnum::CreateTagIndex(_) => "CreateTagIndex",
            PlanNodeEnum::DropTagIndex(_) => "DropTagIndex",
            PlanNodeEnum::DescTagIndex(_) => "DescTagIndex",
            PlanNodeEnum::ShowTagIndexes(_) => "ShowTagIndexes",
            PlanNodeEnum::CreateEdgeIndex(_) => "CreateEdgeIndex",
            PlanNodeEnum::DropEdgeIndex(_) => "DropEdgeIndex",
            PlanNodeEnum::DescEdgeIndex(_) => "DescEdgeIndex",
            PlanNodeEnum::ShowEdgeIndexes(_) => "ShowEdgeIndexes",
            PlanNodeEnum::RebuildTagIndex(_) => "RebuildTagIndex",
            PlanNodeEnum::RebuildEdgeIndex(_) => "RebuildEdgeIndex",
            PlanNodeEnum::CreateUser(_) => "CreateUser",
            PlanNodeEnum::AlterUser(_) => "AlterUser",
            PlanNodeEnum::DropUser(_) => "DropUser",
            PlanNodeEnum::ChangePassword(_) => "ChangePassword",
            PlanNodeEnum::InsertVertices(_) => "InsertVertices",
            PlanNodeEnum::InsertEdges(_) => "InsertEdges",
        }
    }

    /// 获取节点所属分类
    pub fn category(&self) -> PlanNodeCategory {
        match self {
            PlanNodeEnum::Start(_) => PlanNodeCategory::Access,
            PlanNodeEnum::Project(_) => PlanNodeCategory::Operation,
            PlanNodeEnum::Sort(_) => PlanNodeCategory::Operation,
            PlanNodeEnum::Limit(_) => PlanNodeCategory::Operation,
            PlanNodeEnum::TopN(_) => PlanNodeCategory::Operation,
            PlanNodeEnum::Sample(_) => PlanNodeCategory::Operation,
            PlanNodeEnum::InnerJoin(_) => PlanNodeCategory::Join,
            PlanNodeEnum::LeftJoin(_) => PlanNodeCategory::Join,
            PlanNodeEnum::CrossJoin(_) => PlanNodeCategory::Join,
            PlanNodeEnum::GetVertices(_) => PlanNodeCategory::Access,
            PlanNodeEnum::GetEdges(_) => PlanNodeCategory::Access,
            PlanNodeEnum::GetNeighbors(_) => PlanNodeCategory::Access,
            PlanNodeEnum::ScanVertices(_) => PlanNodeCategory::Access,
            PlanNodeEnum::ScanEdges(_) => PlanNodeCategory::Access,
            PlanNodeEnum::EdgeIndexScan(_) => PlanNodeCategory::Access,
            PlanNodeEnum::HashInnerJoin(_) => PlanNodeCategory::Join,
            PlanNodeEnum::HashLeftJoin(_) => PlanNodeCategory::Join,
            PlanNodeEnum::FullOuterJoin(_) => PlanNodeCategory::Join,
            PlanNodeEnum::IndexScan(_) => PlanNodeCategory::Access,
            PlanNodeEnum::Expand(_) => PlanNodeCategory::Traversal,
            PlanNodeEnum::ExpandAll(_) => PlanNodeCategory::Traversal,
            PlanNodeEnum::Traverse(_) => PlanNodeCategory::Traversal,
            PlanNodeEnum::AppendVertices(_) => PlanNodeCategory::Traversal,
            PlanNodeEnum::Filter(_) => PlanNodeCategory::Operation,
            PlanNodeEnum::Aggregate(_) => PlanNodeCategory::Operation,
            PlanNodeEnum::Argument(_) => PlanNodeCategory::ControlFlow,
            PlanNodeEnum::Loop(_) => PlanNodeCategory::ControlFlow,
            PlanNodeEnum::PassThrough(_) => PlanNodeCategory::ControlFlow,
            PlanNodeEnum::Select(_) => PlanNodeCategory::ControlFlow,
            PlanNodeEnum::DataCollect(_) => PlanNodeCategory::DataProcessing,
            PlanNodeEnum::Dedup(_) => PlanNodeCategory::Operation,
            PlanNodeEnum::PatternApply(_) => PlanNodeCategory::DataProcessing,
            PlanNodeEnum::RollUpApply(_) => PlanNodeCategory::DataProcessing,
            PlanNodeEnum::Union(_) => PlanNodeCategory::DataProcessing,
            PlanNodeEnum::Minus(_) => PlanNodeCategory::DataProcessing,
            PlanNodeEnum::Intersect(_) => PlanNodeCategory::DataProcessing,
            PlanNodeEnum::Unwind(_) => PlanNodeCategory::DataProcessing,
            PlanNodeEnum::Assign(_) => PlanNodeCategory::DataProcessing,
            PlanNodeEnum::MultiShortestPath(_) => PlanNodeCategory::Algorithm,
            PlanNodeEnum::BFSShortest(_) => PlanNodeCategory::Algorithm,
            PlanNodeEnum::AllPaths(_) => PlanNodeCategory::Algorithm,
            PlanNodeEnum::ShortestPath(_) => PlanNodeCategory::Algorithm,
            PlanNodeEnum::CreateSpace(_) => PlanNodeCategory::Management,
            PlanNodeEnum::DropSpace(_) => PlanNodeCategory::Management,
            PlanNodeEnum::DescSpace(_) => PlanNodeCategory::Management,
            PlanNodeEnum::ShowSpaces(_) => PlanNodeCategory::Management,
            PlanNodeEnum::CreateTag(_) => PlanNodeCategory::Management,
            PlanNodeEnum::AlterTag(_) => PlanNodeCategory::Management,
            PlanNodeEnum::DescTag(_) => PlanNodeCategory::Management,
            PlanNodeEnum::DropTag(_) => PlanNodeCategory::Management,
            PlanNodeEnum::ShowTags(_) => PlanNodeCategory::Management,
            PlanNodeEnum::CreateEdge(_) => PlanNodeCategory::Management,
            PlanNodeEnum::AlterEdge(_) => PlanNodeCategory::Management,
            PlanNodeEnum::DescEdge(_) => PlanNodeCategory::Management,
            PlanNodeEnum::DropEdge(_) => PlanNodeCategory::Management,
            PlanNodeEnum::ShowEdges(_) => PlanNodeCategory::Management,
            PlanNodeEnum::CreateTagIndex(_) => PlanNodeCategory::Management,
            PlanNodeEnum::DropTagIndex(_) => PlanNodeCategory::Management,
            PlanNodeEnum::DescTagIndex(_) => PlanNodeCategory::Management,
            PlanNodeEnum::ShowTagIndexes(_) => PlanNodeCategory::Management,
            PlanNodeEnum::CreateEdgeIndex(_) => PlanNodeCategory::Management,
            PlanNodeEnum::DropEdgeIndex(_) => PlanNodeCategory::Management,
            PlanNodeEnum::DescEdgeIndex(_) => PlanNodeCategory::Management,
            PlanNodeEnum::ShowEdgeIndexes(_) => PlanNodeCategory::Management,
            PlanNodeEnum::RebuildTagIndex(_) => PlanNodeCategory::Management,
            PlanNodeEnum::RebuildEdgeIndex(_) => PlanNodeCategory::Management,
            PlanNodeEnum::CreateUser(_) => PlanNodeCategory::Management,
            PlanNodeEnum::AlterUser(_) => PlanNodeCategory::Management,
            PlanNodeEnum::DropUser(_) => PlanNodeCategory::Management,
            PlanNodeEnum::ChangePassword(_) => PlanNodeCategory::Management,
            PlanNodeEnum::InsertVertices(_) => PlanNodeCategory::Management,
            PlanNodeEnum::InsertEdges(_) => PlanNodeCategory::Management,
        }
    }

    /// 零成本类型转换 - 直接使用模式匹配
    pub fn as_start(&self) -> Option<&StartNode> {
        match self {
            PlanNodeEnum::Start(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_project(&self) -> Option<&ProjectNode> {
        match self {
            PlanNodeEnum::Project(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_filter(&self) -> Option<&FilterNode> {
        match self {
            PlanNodeEnum::Filter(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_sort(&self) -> Option<&SortNode> {
        match self {
            PlanNodeEnum::Sort(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_limit(&self) -> Option<&LimitNode> {
        match self {
            PlanNodeEnum::Limit(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_sample(&self) -> Option<&SampleNode> {
        match self {
            PlanNodeEnum::Sample(node) => Some(node),
            _ => None,
        }
    }

    /// 零成本类型转换（可变） - 直接使用模式匹配
    pub fn as_start_mut(&mut self) -> Option<&mut StartNode> {
        match self {
            PlanNodeEnum::Start(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_project_mut(&mut self) -> Option<&mut ProjectNode> {
        match self {
            PlanNodeEnum::Project(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_filter_mut(&mut self) -> Option<&mut FilterNode> {
        match self {
            PlanNodeEnum::Filter(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_sort_mut(&mut self) -> Option<&mut SortNode> {
        match self {
            PlanNodeEnum::Sort(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_limit_mut(&mut self) -> Option<&mut LimitNode> {
        match self {
            PlanNodeEnum::Limit(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_sample_mut(&mut self) -> Option<&mut SampleNode> {
        match self {
            PlanNodeEnum::Sample(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_create_user_mut(&mut self) -> Option<&mut CreateUserNode> {
        match self {
            PlanNodeEnum::CreateUser(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_alter_user_mut(&mut self) -> Option<&mut AlterUserNode> {
        match self {
            PlanNodeEnum::AlterUser(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_drop_user_mut(&mut self) -> Option<&mut DropUserNode> {
        match self {
            PlanNodeEnum::DropUser(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_change_password_mut(&mut self) -> Option<&mut ChangePasswordNode> {
        match self {
            PlanNodeEnum::ChangePassword(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_get_vertices(&self) -> Option<&GetVerticesNode> {
        match self {
            PlanNodeEnum::GetVertices(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_get_edges(&self) -> Option<&GetEdgesNode> {
        match self {
            PlanNodeEnum::GetEdges(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_get_neighbors(&self) -> Option<&GetNeighborsNode> {
        match self {
            PlanNodeEnum::GetNeighbors(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_scan_vertices(&self) -> Option<&ScanVerticesNode> {
        match self {
            PlanNodeEnum::ScanVertices(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_scan_edges(&self) -> Option<&ScanEdgesNode> {
        match self {
            PlanNodeEnum::ScanEdges(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_edge_index_scan(&self) -> Option<&EdgeIndexScanNode> {
        match self {
            PlanNodeEnum::EdgeIndexScan(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_index_scan(&self) -> Option<&crate::query::planner::plan::algorithms::IndexScan> {
        match self {
            PlanNodeEnum::IndexScan(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_expand_all(&self) -> Option<&ExpandAllNode> {
        match self {
            PlanNodeEnum::ExpandAll(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_traverse(&self) -> Option<&TraverseNode> {
        match self {
            PlanNodeEnum::Traverse(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_expand(&self) -> Option<&ExpandNode> {
        match self {
            PlanNodeEnum::Expand(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_aggregate(&self) -> Option<&AggregateNode> {
        match self {
            PlanNodeEnum::Aggregate(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_create_space(&self) -> Option<&CreateSpaceNode> {
        match self {
            PlanNodeEnum::CreateSpace(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_drop_space(&self) -> Option<&DropSpaceNode> {
        match self {
            PlanNodeEnum::DropSpace(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_desc_space(&self) -> Option<&DescSpaceNode> {
        match self {
            PlanNodeEnum::DescSpace(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_show_spaces(&self) -> Option<&ShowSpacesNode> {
        match self {
            PlanNodeEnum::ShowSpaces(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_create_tag(&self) -> Option<&CreateTagNode> {
        match self {
            PlanNodeEnum::CreateTag(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_alter_tag(&self) -> Option<&AlterTagNode> {
        match self {
            PlanNodeEnum::AlterTag(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_desc_tag(&self) -> Option<&DescTagNode> {
        match self {
            PlanNodeEnum::DescTag(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_drop_tag(&self) -> Option<&DropTagNode> {
        match self {
            PlanNodeEnum::DropTag(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_show_tags(&self) -> Option<&ShowTagsNode> {
        match self {
            PlanNodeEnum::ShowTags(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_create_edge(&self) -> Option<&CreateEdgeNode> {
        match self {
            PlanNodeEnum::CreateEdge(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_alter_edge(&self) -> Option<&AlterEdgeNode> {
        match self {
            PlanNodeEnum::AlterEdge(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_desc_edge(&self) -> Option<&DescEdgeNode> {
        match self {
            PlanNodeEnum::DescEdge(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_drop_edge(&self) -> Option<&DropEdgeNode> {
        match self {
            PlanNodeEnum::DropEdge(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_show_edges(&self) -> Option<&ShowEdgesNode> {
        match self {
            PlanNodeEnum::ShowEdges(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_create_user(&self) -> Option<&CreateUserNode> {
        match self {
            PlanNodeEnum::CreateUser(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_alter_user(&self) -> Option<&AlterUserNode> {
        match self {
            PlanNodeEnum::AlterUser(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_drop_user(&self) -> Option<&DropUserNode> {
        match self {
            PlanNodeEnum::DropUser(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_change_password(&self) -> Option<&ChangePasswordNode> {
        match self {
            PlanNodeEnum::ChangePassword(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_create_tag_index(&self) -> Option<&CreateTagIndexNode> {
        match self {
            PlanNodeEnum::CreateTagIndex(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_drop_tag_index(&self) -> Option<&DropTagIndexNode> {
        match self {
            PlanNodeEnum::DropTagIndex(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_desc_tag_index(&self) -> Option<&DescTagIndexNode> {
        match self {
            PlanNodeEnum::DescTagIndex(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_show_tag_indexes(&self) -> Option<&ShowTagIndexesNode> {
        match self {
            PlanNodeEnum::ShowTagIndexes(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_create_edge_index(&self) -> Option<&CreateEdgeIndexNode> {
        match self {
            PlanNodeEnum::CreateEdgeIndex(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_drop_edge_index(&self) -> Option<&DropEdgeIndexNode> {
        match self {
            PlanNodeEnum::DropEdgeIndex(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_desc_edge_index(&self) -> Option<&DescEdgeIndexNode> {
        match self {
            PlanNodeEnum::DescEdgeIndex(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_show_edge_indexes(&self) -> Option<&ShowEdgeIndexesNode> {
        match self {
            PlanNodeEnum::ShowEdgeIndexes(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_rebuild_tag_index(&self) -> Option<&RebuildTagIndexNode> {
        match self {
            PlanNodeEnum::RebuildTagIndex(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_rebuild_edge_index(&self) -> Option<&RebuildEdgeIndexNode> {
        match self {
            PlanNodeEnum::RebuildEdgeIndex(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_loop(&self) -> Option<&LoopNode> {
        match self {
            PlanNodeEnum::Loop(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_argument(&self) -> Option<&ArgumentNode> {
        match self {
            PlanNodeEnum::Argument(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_pass_through(&self) -> Option<&PassThroughNode> {
        match self {
            PlanNodeEnum::PassThrough(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_data_collect(&self) -> Option<&DataCollectNode> {
        match self {
            PlanNodeEnum::DataCollect(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_bfs_shortest(&self) -> Option<&BFSShortest> {
        match self {
            PlanNodeEnum::BFSShortest(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_all_paths(&self) -> Option<&AllPaths> {
        match self {
            PlanNodeEnum::AllPaths(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_all_paths_mut(&mut self) -> Option<&mut AllPaths> {
        match self {
            PlanNodeEnum::AllPaths(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_multi_shortest_path(&self) -> Option<&MultiShortestPath> {
        match self {
            PlanNodeEnum::MultiShortestPath(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_shortest_path(&self) -> Option<&ShortestPath> {
        match self {
            PlanNodeEnum::ShortestPath(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_union(&self) -> Option<&UnionNode> {
        match self {
            PlanNodeEnum::Union(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_minus(&self) -> Option<&MinusNode> {
        match self {
            PlanNodeEnum::Minus(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_intersect(&self) -> Option<&IntersectNode> {
        match self {
            PlanNodeEnum::Intersect(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_unwind(&self) -> Option<&UnwindNode> {
        match self {
            PlanNodeEnum::Unwind(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_assign(&self) -> Option<&AssignNode> {
        match self {
            PlanNodeEnum::Assign(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_pattern_apply(&self) -> Option<&PatternApplyNode> {
        match self {
            PlanNodeEnum::PatternApply(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_roll_up_apply(&self) -> Option<&RollUpApplyNode> {
        match self {
            PlanNodeEnum::RollUpApply(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_topn(&self) -> Option<&TopNNode> {
        match self {
            PlanNodeEnum::TopN(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_hash_inner_join(&self) -> Option<&HashInnerJoinNode> {
        match self {
            PlanNodeEnum::HashInnerJoin(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_hash_left_join(&self) -> Option<&HashLeftJoinNode> {
        match self {
            PlanNodeEnum::HashLeftJoin(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_full_outer_join(&self) -> Option<&FullOuterJoinNode> {
        match self {
            PlanNodeEnum::FullOuterJoin(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_select(&self) -> Option<&SelectNode> {
        match self {
            PlanNodeEnum::Select(node) => Some(node),
            _ => None,
        }
    }

    /// 克隆节点
    pub fn clone_plan_node(&self) -> PlanNodeEnum {
        self.clone()
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

    /// 零成本描述生成 - 编译时优化
    pub fn describe(&self) -> PlanNodeDescription {
        match self {
            PlanNodeEnum::Start(node) => {
                let mut desc = PlanNodeDescription::new("Start", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.to_string());
                }
                desc
            }
            PlanNodeEnum::Project(node) => {
                let mut desc = PlanNodeDescription::new("Project", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.to_string());
                }
                desc
            }
            PlanNodeEnum::Filter(node) => {
                let mut desc = PlanNodeDescription::new("Filter", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.to_string());
                }
                desc
            }
            PlanNodeEnum::Sort(node) => {
                let mut desc = PlanNodeDescription::new("Sort", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.to_string());
                }
                desc
            }
            PlanNodeEnum::Limit(node) => {
                let mut desc = PlanNodeDescription::new("Limit", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.to_string());
                }
                desc
            }
            PlanNodeEnum::InnerJoin(node) => {
                let mut desc = PlanNodeDescription::new("InnerJoin", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.to_string());
                }
                desc
            }
            PlanNodeEnum::LeftJoin(node) => {
                let mut desc = PlanNodeDescription::new("LeftJoin", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.to_string());
                }
                desc
            }
            PlanNodeEnum::CrossJoin(node) => {
                let mut desc = PlanNodeDescription::new("CrossJoin", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.to_string());
                }
                desc
            }
            PlanNodeEnum::GetVertices(node) => {
                let mut desc = PlanNodeDescription::new("GetVertices", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.to_string());
                }
                desc
            }
            PlanNodeEnum::GetEdges(node) => {
                let mut desc = PlanNodeDescription::new("GetEdges", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.to_string());
                }
                desc
            }
            PlanNodeEnum::GetNeighbors(node) => {
                let mut desc = PlanNodeDescription::new("GetNeighbors", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.to_string());
                }
                desc
            }
            PlanNodeEnum::ScanVertices(node) => {
                let mut desc = PlanNodeDescription::new("ScanVertices", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.to_string());
                }
                desc
            }
            PlanNodeEnum::ScanEdges(node) => {
                let mut desc = PlanNodeDescription::new("ScanEdges", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.to_string());
                }
                desc
            }
            PlanNodeEnum::Expand(node) => {
                let mut desc = PlanNodeDescription::new("Expand", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.to_string());
                }
                desc
            }
            PlanNodeEnum::ExpandAll(node) => {
                let mut desc = PlanNodeDescription::new("ExpandAll", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.to_string());
                }
                desc
            }
            PlanNodeEnum::Traverse(node) => {
                let mut desc = PlanNodeDescription::new("Traverse", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.to_string());
                }
                desc
            }
            PlanNodeEnum::AppendVertices(node) => {
                let mut desc = PlanNodeDescription::new("AppendVertices", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.to_string());
                }
                desc
            }
            PlanNodeEnum::Aggregate(node) => {
                let mut desc = PlanNodeDescription::new("Aggregate", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.to_string());
                }
                desc
            }
            PlanNodeEnum::Argument(node) => {
                let mut desc = PlanNodeDescription::new("Argument", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.to_string());
                }
                desc
            }
            PlanNodeEnum::Loop(node) => {
                let mut desc = PlanNodeDescription::new("Loop", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.to_string());
                }
                desc
            }
            PlanNodeEnum::PassThrough(node) => {
                let mut desc = PlanNodeDescription::new("PassThrough", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.to_string());
                }
                desc
            }
            PlanNodeEnum::Select(node) => {
                let mut desc = PlanNodeDescription::new("Select", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.to_string());
                }
                desc
            }
            PlanNodeEnum::DataCollect(node) => {
                let mut desc = PlanNodeDescription::new("DataCollect", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.to_string());
                }
                desc
            }
            PlanNodeEnum::Dedup(node) => {
                let mut desc = PlanNodeDescription::new("Dedup", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.to_string());
                }
                desc
            }
            PlanNodeEnum::PatternApply(node) => {
                let mut desc = PlanNodeDescription::new("PatternApply", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.to_string());
                }
                desc
            }
            PlanNodeEnum::RollUpApply(node) => {
                let mut desc = PlanNodeDescription::new("RollUpApply", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.to_string());
                }
                desc
            }
            PlanNodeEnum::Union(node) => {
                let mut desc = PlanNodeDescription::new("Union", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.to_string());
                }
                desc
            }
            PlanNodeEnum::Minus(node) => {
                let mut desc = PlanNodeDescription::new("Minus", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.to_string());
                }
                desc
            }
            PlanNodeEnum::Intersect(node) => {
                let mut desc = PlanNodeDescription::new("Intersect", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.to_string());
                }
                desc
            }
            PlanNodeEnum::Unwind(node) => {
                let mut desc = PlanNodeDescription::new("Unwind", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.to_string());
                }
                desc
            }
            PlanNodeEnum::TopN(node) => {
                let mut desc = PlanNodeDescription::new("TopN", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.to_string());
                }
                desc
            }
            PlanNodeEnum::Sample(node) => {
                let mut desc = PlanNodeDescription::new("Sample", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.to_string());
                }
                desc.add_description("count", node.count().to_string());
                desc
            }
            PlanNodeEnum::HashInnerJoin(node) => {
                let mut desc = PlanNodeDescription::new("HashInnerJoin", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.to_string());
                }
                desc
            }
            PlanNodeEnum::HashLeftJoin(node) => {
                let mut desc = PlanNodeDescription::new("HashLeftJoin", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.to_string());
                }
                desc
            }
            PlanNodeEnum::FullOuterJoin(node) => {
                let mut desc = PlanNodeDescription::new("FullOuterJoin", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.to_string());
                }
                desc
            }
            PlanNodeEnum::IndexScan(node) => {
                let mut desc = PlanNodeDescription::new("IndexScan", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.to_string());
                }
                desc
            }
            PlanNodeEnum::EdgeIndexScan(node) => {
                let mut desc = PlanNodeDescription::new("EdgeIndexScan", node.id());
                desc.add_description("spaceId", node.space_id().to_string());
                desc.add_description("edgeType", node.edge_type().to_string());
                desc.add_description("indexName", node.index_name().to_string());
                desc
            }
            PlanNodeEnum::MultiShortestPath(node) => {
                let mut desc = PlanNodeDescription::new("MultiShortestPath", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.to_string());
                }
                desc
            }
            PlanNodeEnum::BFSShortest(node) => {
                let mut desc = PlanNodeDescription::new("BFSShortest", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.to_string());
                }
                desc
            }
            PlanNodeEnum::AllPaths(node) => {
                let mut desc = PlanNodeDescription::new("AllPaths", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.to_string());
                }
                desc
            }
            PlanNodeEnum::ShortestPath(node) => {
                let mut desc = PlanNodeDescription::new("ShortestPath", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.to_string());
                }
                desc
            }
            PlanNodeEnum::Assign(node) => {
                let mut desc = PlanNodeDescription::new("Assign", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.to_string());
                }
                desc
            }

            // ========== 管理节点 - 详细描述 ==========
            // Space 管理节点
            PlanNodeEnum::CreateSpace(node) => {
                let mut desc = PlanNodeDescription::new("CreateSpace", node.id());
                let info = node.info();
                desc.add_description("spaceName", info.space_name.clone());
                desc.add_description("vidType", info.vid_type.clone());
                desc
            }
            PlanNodeEnum::DropSpace(node) => {
                let mut desc = PlanNodeDescription::new("DropSpace", node.id());
                desc.add_description("spaceName", node.space_name().to_string());
                desc
            }
            PlanNodeEnum::DescSpace(node) => {
                let mut desc = PlanNodeDescription::new("DescSpace", node.id());
                desc.add_description("spaceName", node.space_name().to_string());
                desc
            }
            PlanNodeEnum::ShowSpaces(_) => {
                PlanNodeDescription::new("ShowSpaces", self.id())
            }

            // Tag 管理节点
            PlanNodeEnum::CreateTag(node) => {
                let mut desc = PlanNodeDescription::new("CreateTag", node.id());
                let info = node.info();
                desc.add_description("spaceName", info.space_name.clone());
                desc.add_description("tagName", info.tag_name.clone());
                desc.add_description("properties", format!("[{} properties]", info.properties.len()));
                desc
            }
            PlanNodeEnum::AlterTag(node) => {
                let mut desc = PlanNodeDescription::new("AlterTag", node.id());
                let info = node.info();
                desc.add_description("spaceName", info.space_name.clone());
                desc.add_description("tagName", info.tag_name.clone());
                desc.add_description("additions", format!("[{} additions]", info.additions.len()));
                desc.add_description("deletions", format!("[{} deletions]", info.deletions.len()));
                desc
            }
            PlanNodeEnum::DescTag(node) => {
                let mut desc = PlanNodeDescription::new("DescTag", node.id());
                desc.add_description("spaceName", node.space_name().to_string());
                desc.add_description("tagName", node.tag_name().to_string());
                desc
            }
            PlanNodeEnum::DropTag(node) => {
                let mut desc = PlanNodeDescription::new("DropTag", node.id());
                desc.add_description("spaceName", node.space_name().to_string());
                desc.add_description("tagName", node.tag_name().to_string());
                desc
            }
            PlanNodeEnum::ShowTags(_) => {
                PlanNodeDescription::new("ShowTags", self.id())
            }

            // Edge 管理节点
            PlanNodeEnum::CreateEdge(node) => {
                let mut desc = PlanNodeDescription::new("CreateEdge", node.id());
                let info = node.info();
                desc.add_description("spaceName", info.space_name.clone());
                desc.add_description("edgeName", info.edge_name.clone());
                desc.add_description("properties", format!("[{} properties]", info.properties.len()));
                desc
            }
            PlanNodeEnum::AlterEdge(node) => {
                let mut desc = PlanNodeDescription::new("AlterEdge", node.id());
                let info = node.info();
                desc.add_description("spaceName", info.space_name.clone());
                desc.add_description("edgeName", info.edge_name.clone());
                desc.add_description("additions", format!("[{} additions]", info.additions.len()));
                desc.add_description("deletions", format!("[{} deletions]", info.deletions.len()));
                desc
            }
            PlanNodeEnum::DescEdge(node) => {
                let mut desc = PlanNodeDescription::new("DescEdge", node.id());
                desc.add_description("spaceName", node.space_name().to_string());
                desc.add_description("edgeName", node.edge_name().to_string());
                desc
            }
            PlanNodeEnum::DropEdge(node) => {
                let mut desc = PlanNodeDescription::new("DropEdge", node.id());
                desc.add_description("spaceName", node.space_name().to_string());
                desc.add_description("edgeName", node.edge_name().to_string());
                desc
            }
            PlanNodeEnum::ShowEdges(_) => {
                PlanNodeDescription::new("ShowEdges", self.id())
            }

            // Tag 索引管理节点
            PlanNodeEnum::CreateTagIndex(node) => {
                let mut desc = PlanNodeDescription::new("CreateTagIndex", node.id());
                let info = node.info();
                desc.add_description("spaceName", info.space_name.clone());
                desc.add_description("indexName", info.index_name.clone());
                desc.add_description("targetName", info.target_name.clone());
                desc.add_description("properties", format!("[{} properties]", info.properties.len()));
                desc
            }
            PlanNodeEnum::DropTagIndex(node) => {
                let mut desc = PlanNodeDescription::new("DropTagIndex", node.id());
                desc.add_description("spaceName", node.space_name().to_string());
                desc.add_description("indexName", node.index_name().to_string());
                desc
            }
            PlanNodeEnum::DescTagIndex(node) => {
                let mut desc = PlanNodeDescription::new("DescTagIndex", node.id());
                desc.add_description("spaceName", node.space_name().to_string());
                desc.add_description("indexName", node.index_name().to_string());
                desc
            }
            PlanNodeEnum::ShowTagIndexes(_) => {
                PlanNodeDescription::new("ShowTagIndexes", self.id())
            }
            PlanNodeEnum::RebuildTagIndex(node) => {
                let mut desc = PlanNodeDescription::new("RebuildTagIndex", node.id());
                desc.add_description("spaceName", node.space_name().to_string());
                desc.add_description("indexName", node.index_name().to_string());
                desc
            }

            // Edge 索引管理节点
            PlanNodeEnum::CreateEdgeIndex(node) => {
                let mut desc = PlanNodeDescription::new("CreateEdgeIndex", node.id());
                let info = node.info();
                desc.add_description("spaceName", info.space_name.clone());
                desc.add_description("indexName", info.index_name.clone());
                desc.add_description("targetName", info.target_name.clone());
                desc.add_description("properties", format!("[{} properties]", info.properties.len()));
                desc
            }
            PlanNodeEnum::DropEdgeIndex(node) => {
                let mut desc = PlanNodeDescription::new("DropEdgeIndex", node.id());
                desc.add_description("spaceName", node.space_name().to_string());
                desc.add_description("indexName", node.index_name().to_string());
                desc
            }
            PlanNodeEnum::DescEdgeIndex(node) => {
                let mut desc = PlanNodeDescription::new("DescEdgeIndex", node.id());
                desc.add_description("spaceName", node.space_name().to_string());
                desc.add_description("indexName", node.index_name().to_string());
                desc
            }
            PlanNodeEnum::ShowEdgeIndexes(_) => {
                PlanNodeDescription::new("ShowEdgeIndexes", self.id())
            }
            PlanNodeEnum::RebuildEdgeIndex(node) => {
                let mut desc = PlanNodeDescription::new("RebuildEdgeIndex", node.id());
                desc.add_description("spaceName", node.space_name().to_string());
                desc.add_description("indexName", node.index_name().to_string());
                desc
            }

            // User 管理节点
            PlanNodeEnum::CreateUser(node) => {
                let mut desc = PlanNodeDescription::new("CreateUser", node.id());
                desc.add_description("username", node.username().to_string());
                desc.add_description("password", "******");
                desc.add_description("role", node.role().to_string());
                desc
            }
            PlanNodeEnum::AlterUser(node) => {
                let mut desc = PlanNodeDescription::new("AlterUser", node.id());
                desc.add_description("username", node.username().to_string());
                if let Some(role) = node.new_role() {
                    desc.add_description("newRole", role.clone());
                }
                if let Some(locked) = node.is_locked() {
                    desc.add_description("isLocked", locked.to_string());
                }
                desc
            }
            PlanNodeEnum::DropUser(node) => {
                let mut desc = PlanNodeDescription::new("DropUser", node.id());
                desc.add_description("username", node.username().to_string());
                desc
            }
            PlanNodeEnum::ChangePassword(node) => {
                let mut desc = PlanNodeDescription::new("ChangePassword", node.id());
                let info = node.password_info();
                let username_str = info.username.clone().unwrap_or_else(|| "current_user".to_string());
                desc.add_description("username", username_str);
                desc.add_description("password", "******");
                desc.add_description("newPassword", "******");
                desc
            }
            PlanNodeEnum::InsertVertices(node) => {
                let mut desc = PlanNodeDescription::new("InsertVertices", node.id());
                let info = node.info();
                desc.add_description("spaceName", info.space_name.clone());
                let tag_names: Vec<String> = info.tags.iter().map(|t| t.tag_name.clone()).collect();
                desc.add_description("tags", format!("[{}]", tag_names.join(", ")));
                let total_props: usize = info.tags.iter().map(|t| t.prop_names.len()).sum();
                desc.add_description("properties", format!("[{} properties]", total_props));
                desc.add_description("values", format!("[{} values]", info.values.len()));
                desc
            }
            PlanNodeEnum::InsertEdges(node) => {
                let mut desc = PlanNodeDescription::new("InsertEdges", node.id());
                let info = node.info();
                desc.add_description("spaceName", info.space_name.clone());
                desc.add_description("edgeName", info.edge_name.clone());
                desc.add_description("properties", format!("[{} properties]", info.prop_names.len()));
                desc.add_description("edges", format!("[{} edges]", info.edges.len()));
                desc
            }
        }
    }
}

impl std::fmt::Display for PlanNodeEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}()", self)
    }
}
