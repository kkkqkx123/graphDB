//! PlanNode 枚举定义
//!

use super::plan_node_category::PlanNodeCategory;
use crate::query::core::{NodeType, NodeCategory, NodeTypeMapping};
use super::space_nodes::{CreateSpaceNode, DescSpaceNode, DropSpaceNode, ShowSpacesNode, SpaceManageInfo};
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
    CrossJoinNode, HashInnerJoinNode, HashLeftJoinNode, InnerJoinNode, LeftJoinNode,
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
                    desc = desc.with_output_var(var.name.clone());
                }
                desc.add_description("cost", format!("{:.2}", node.cost()));
                desc
            }
            PlanNodeEnum::Project(node) => {
                let mut desc = PlanNodeDescription::new("Project", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.name.clone());
                }
                desc.add_description("cost", format!("{:.2}", node.cost()));
                desc
            }
            PlanNodeEnum::Filter(node) => {
                let mut desc = PlanNodeDescription::new("Filter", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.name.clone());
                }
                desc.add_description("cost", format!("{:.2}", node.cost()));
                desc
            }
            PlanNodeEnum::Sort(node) => {
                let mut desc = PlanNodeDescription::new("Sort", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.name.clone());
                }
                desc.add_description("cost", format!("{:.2}", node.cost()));
                desc
            }
            PlanNodeEnum::Limit(node) => {
                let mut desc = PlanNodeDescription::new("Limit", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.name.clone());
                }
                desc.add_description("cost", format!("{:.2}", node.cost()));
                desc
            }
            PlanNodeEnum::InnerJoin(node) => {
                let mut desc = PlanNodeDescription::new("InnerJoin", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.name.clone());
                }
                desc.add_description("cost", format!("{:.2}", node.cost()));
                desc
            }
            PlanNodeEnum::LeftJoin(node) => {
                let mut desc = PlanNodeDescription::new("LeftJoin", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.name.clone());
                }
                desc.add_description("cost", format!("{:.2}", node.cost()));
                desc
            }
            PlanNodeEnum::CrossJoin(node) => {
                let mut desc = PlanNodeDescription::new("CrossJoin", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.name.clone());
                }
                desc.add_description("cost", format!("{:.2}", node.cost()));
                desc
            }
            PlanNodeEnum::GetVertices(node) => {
                let mut desc = PlanNodeDescription::new("GetVertices", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.name.clone());
                }
                desc.add_description("cost", format!("{:.2}", node.cost()));
                desc
            }
            PlanNodeEnum::GetEdges(node) => {
                let mut desc = PlanNodeDescription::new("GetEdges", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.name.clone());
                }
                desc.add_description("cost", format!("{:.2}", node.cost()));
                desc
            }
            PlanNodeEnum::GetNeighbors(node) => {
                let mut desc = PlanNodeDescription::new("GetNeighbors", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.name.clone());
                }
                desc.add_description("cost", format!("{:.2}", node.cost()));
                desc
            }
            PlanNodeEnum::ScanVertices(node) => {
                let mut desc = PlanNodeDescription::new("ScanVertices", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.name.clone());
                }
                desc.add_description("cost", format!("{:.2}", node.cost()));
                desc
            }
            PlanNodeEnum::ScanEdges(node) => {
                let mut desc = PlanNodeDescription::new("ScanEdges", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.name.clone());
                }
                desc.add_description("cost", format!("{:.2}", node.cost()));
                desc
            }
            PlanNodeEnum::Expand(node) => {
                let mut desc = PlanNodeDescription::new("Expand", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.name.clone());
                }
                desc.add_description("cost", format!("{:.2}", node.cost()));
                desc
            }
            PlanNodeEnum::ExpandAll(node) => {
                let mut desc = PlanNodeDescription::new("ExpandAll", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.name.clone());
                }
                desc.add_description("cost", format!("{:.2}", node.cost()));
                desc
            }
            PlanNodeEnum::Traverse(node) => {
                let mut desc = PlanNodeDescription::new("Traverse", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.name.clone());
                }
                desc.add_description("cost", format!("{:.2}", node.cost()));
                desc
            }
            PlanNodeEnum::AppendVertices(node) => {
                let mut desc = PlanNodeDescription::new("AppendVertices", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.name.clone());
                }
                desc.add_description("cost", format!("{:.2}", node.cost()));
                desc
            }
            PlanNodeEnum::Aggregate(node) => {
                let mut desc = PlanNodeDescription::new("Aggregate", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.name.clone());
                }
                desc.add_description("cost", format!("{:.2}", node.cost()));
                desc
            }
            PlanNodeEnum::Argument(node) => {
                let mut desc = PlanNodeDescription::new("Argument", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.name.clone());
                }
                desc.add_description("cost", format!("{:.2}", node.cost()));
                desc
            }
            PlanNodeEnum::Loop(node) => {
                let mut desc = PlanNodeDescription::new("Loop", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.name.clone());
                }
                desc.add_description("cost", format!("{:.2}", node.cost()));
                desc
            }
            PlanNodeEnum::PassThrough(node) => {
                let mut desc = PlanNodeDescription::new("PassThrough", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.name.clone());
                }
                desc.add_description("cost", format!("{:.2}", node.cost()));
                desc
            }
            PlanNodeEnum::Select(node) => {
                let mut desc = PlanNodeDescription::new("Select", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.name.clone());
                }
                desc.add_description("cost", format!("{:.2}", node.cost()));
                desc
            }
            PlanNodeEnum::DataCollect(node) => {
                let mut desc = PlanNodeDescription::new("DataCollect", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.name.clone());
                }
                desc.add_description("cost", format!("{:.2}", node.cost()));
                desc
            }
            PlanNodeEnum::Dedup(node) => {
                let mut desc = PlanNodeDescription::new("Dedup", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.name.clone());
                }
                desc.add_description("cost", format!("{:.2}", node.cost()));
                desc
            }
            PlanNodeEnum::PatternApply(node) => {
                let mut desc = PlanNodeDescription::new("PatternApply", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.name.clone());
                }
                desc.add_description("cost", format!("{:.2}", node.cost()));
                desc
            }
            PlanNodeEnum::RollUpApply(node) => {
                let mut desc = PlanNodeDescription::new("RollUpApply", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.name.clone());
                }
                desc.add_description("cost", format!("{:.2}", node.cost()));
                desc
            }
            PlanNodeEnum::Union(node) => {
                let mut desc = PlanNodeDescription::new("Union", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.name.clone());
                }
                desc.add_description("cost", format!("{:.2}", node.cost()));
                desc
            }
            PlanNodeEnum::Minus(node) => {
                let mut desc = PlanNodeDescription::new("Minus", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.name.clone());
                }
                desc.add_description("cost", format!("{:.2}", node.cost()));
                desc
            }
            PlanNodeEnum::Intersect(node) => {
                let mut desc = PlanNodeDescription::new("Intersect", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.name.clone());
                }
                desc.add_description("cost", format!("{:.2}", node.cost()));
                desc
            }
            PlanNodeEnum::Unwind(node) => {
                let mut desc = PlanNodeDescription::new("Unwind", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.name.clone());
                }
                desc.add_description("cost", format!("{:.2}", node.cost()));
                desc
            }
            PlanNodeEnum::TopN(node) => {
                let mut desc = PlanNodeDescription::new("TopN", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.name.clone());
                }
                desc.add_description("cost", format!("{:.2}", node.cost()));
                desc
            }
            PlanNodeEnum::Sample(node) => {
                let mut desc = PlanNodeDescription::new("Sample", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.name.clone());
                }
                desc.add_description("count", node.count().to_string());
                desc.add_description("cost", format!("{:.2}", node.cost()));
                desc
            }
            PlanNodeEnum::HashInnerJoin(node) => {
                let mut desc = PlanNodeDescription::new("HashInnerJoin", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.name.clone());
                }
                desc.add_description("cost", format!("{:.2}", node.cost()));
                desc
            }
            PlanNodeEnum::HashLeftJoin(node) => {
                let mut desc = PlanNodeDescription::new("HashLeftJoin", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.name.clone());
                }
                desc.add_description("cost", format!("{:.2}", node.cost()));
                desc
            }
            PlanNodeEnum::IndexScan(node) => {
                let mut desc = PlanNodeDescription::new("IndexScan", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.name.clone());
                }
                desc.add_description("cost", format!("{:.2}", node.cost()));
                desc
            }
            PlanNodeEnum::EdgeIndexScan(node) => {
                let mut desc = PlanNodeDescription::new("EdgeIndexScan", node.id());
                desc.add_description("spaceId", node.space_id().to_string());
                desc.add_description("edgeType", node.edge_type().to_string());
                desc.add_description("indexName", node.index_name().to_string());
                desc.add_description("cost", format!("{:.2}", node.cost()));
                desc
            }
            PlanNodeEnum::MultiShortestPath(node) => {
                let mut desc = PlanNodeDescription::new("MultiShortestPath", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.name.clone());
                }
                desc.add_description("cost", format!("{:.2}", node.cost()));
                desc
            }
            PlanNodeEnum::BFSShortest(node) => {
                let mut desc = PlanNodeDescription::new("BFSShortest", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.name.clone());
                }
                desc.add_description("cost", format!("{:.2}", node.cost()));
                desc
            }
            PlanNodeEnum::AllPaths(node) => {
                let mut desc = PlanNodeDescription::new("AllPaths", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.name.clone());
                }
                desc.add_description("cost", format!("{:.2}", node.cost()));
                desc
            }
            PlanNodeEnum::ShortestPath(node) => {
                let mut desc = PlanNodeDescription::new("ShortestPath", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.name.clone());
                }
                desc.add_description("cost", format!("{:.2}", node.cost()));
                desc
            }
            PlanNodeEnum::Assign(node) => {
                let mut desc = PlanNodeDescription::new("Assign", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.name.clone());
                }
                desc.add_description("cost", format!("{:.2}", node.cost()));
                desc
            }

            // ========== 管理节点 - 详细描述 ==========
            // Space 管理节点
            PlanNodeEnum::CreateSpace(node) => {
                let mut desc = PlanNodeDescription::new("CreateSpace", node.id());
                let info = node.info();
                desc.add_description("spaceName", info.space_name.clone());
                desc.add_description("partitionNum", info.partition_num.to_string());
                desc.add_description("replicaFactor", info.replica_factor.to_string());
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
                desc.add_description("tagName", info.tag_name.clone());
                desc.add_description("properties", format!("[{} properties]", info.prop_names.len()));
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

// 使用操作实现

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

    /// 获取节点的所有子节点
    /// 用于遍历执行计划树
    pub fn children(&self) -> Vec<&PlanNodeEnum> {
        match self {
            // ZeroInputNode: 没有子节点
            PlanNodeEnum::Start(_) => vec![],
            PlanNodeEnum::CreateSpace(_) => vec![],
            PlanNodeEnum::DropSpace(_) => vec![],
            PlanNodeEnum::DescSpace(_) => vec![],
            PlanNodeEnum::ShowSpaces(_) => vec![],
            PlanNodeEnum::CreateTag(_) => vec![],
            PlanNodeEnum::AlterTag(_) => vec![],
            PlanNodeEnum::DescTag(_) => vec![],
            PlanNodeEnum::DropTag(_) => vec![],
            PlanNodeEnum::ShowTags(_) => vec![],
            PlanNodeEnum::CreateEdge(_) => vec![],
            PlanNodeEnum::AlterEdge(_) => vec![],
            PlanNodeEnum::DescEdge(_) => vec![],
            PlanNodeEnum::DropEdge(_) => vec![],
            PlanNodeEnum::ShowEdges(_) => vec![],
            PlanNodeEnum::CreateTagIndex(_) => vec![],
            PlanNodeEnum::DropTagIndex(_) => vec![],
            PlanNodeEnum::DescTagIndex(_) => vec![],
            PlanNodeEnum::ShowTagIndexes(_) => vec![],
            PlanNodeEnum::CreateEdgeIndex(_) => vec![],
            PlanNodeEnum::DropEdgeIndex(_) => vec![],
            PlanNodeEnum::DescEdgeIndex(_) => vec![],
            PlanNodeEnum::ShowEdgeIndexes(_) => vec![],
            PlanNodeEnum::RebuildTagIndex(_) => vec![],
            PlanNodeEnum::RebuildEdgeIndex(_) => vec![],
            PlanNodeEnum::CreateUser(_) => vec![],
            PlanNodeEnum::AlterUser(_) => vec![],
            PlanNodeEnum::DropUser(_) => vec![],
            PlanNodeEnum::ChangePassword(_) => vec![],
            PlanNodeEnum::InsertVertices(_) => vec![],
            PlanNodeEnum::InsertEdges(_) => vec![],
            PlanNodeEnum::IndexScan(_) => vec![],
            PlanNodeEnum::ScanVertices(_) => vec![],
            PlanNodeEnum::ScanEdges(_) => vec![],
            PlanNodeEnum::EdgeIndexScan(_) => vec![],
            PlanNodeEnum::GetVertices(_) => vec![],
            PlanNodeEnum::GetEdges(_) => vec![],
            PlanNodeEnum::GetNeighbors(_) => vec![],
            PlanNodeEnum::ShortestPath(_) => vec![],
            PlanNodeEnum::AllPaths(_) => vec![],
            PlanNodeEnum::BFSShortest(_) => vec![],
            PlanNodeEnum::MultiShortestPath(_) => vec![],

            // SingleInputNode: 有一个子节点
            PlanNodeEnum::Project(node) => vec![super::plan_node_traits::SingleInputNode::input(node)],
            PlanNodeEnum::Filter(node) => vec![super::plan_node_traits::SingleInputNode::input(node)],
            PlanNodeEnum::Sort(node) => vec![super::plan_node_traits::SingleInputNode::input(node)],
            PlanNodeEnum::Limit(node) => vec![super::plan_node_traits::SingleInputNode::input(node)],
            PlanNodeEnum::TopN(node) => vec![super::plan_node_traits::SingleInputNode::input(node)],
            PlanNodeEnum::Sample(node) => vec![super::plan_node_traits::SingleInputNode::input(node)],
            PlanNodeEnum::Dedup(node) => vec![super::plan_node_traits::SingleInputNode::input(node)],
            PlanNodeEnum::DataCollect(node) => vec![super::plan_node_traits::SingleInputNode::input(node)],
            PlanNodeEnum::Aggregate(node) => vec![super::plan_node_traits::SingleInputNode::input(node)],
            PlanNodeEnum::Unwind(node) => vec![super::plan_node_traits::SingleInputNode::input(node)],
            PlanNodeEnum::Assign(node) => vec![super::plan_node_traits::SingleInputNode::input(node)],
            PlanNodeEnum::PatternApply(node) => vec![super::plan_node_traits::SingleInputNode::input(node)],
            PlanNodeEnum::RollUpApply(node) => vec![super::plan_node_traits::SingleInputNode::input(node)],
            PlanNodeEnum::Traverse(node) => vec![super::plan_node_traits::SingleInputNode::input(node)],

            // BinaryInputNode: 有两个子节点
            PlanNodeEnum::InnerJoin(node) => vec![super::plan_node_traits::BinaryInputNode::left_input(node), super::plan_node_traits::BinaryInputNode::right_input(node)],
            PlanNodeEnum::LeftJoin(node) => vec![super::plan_node_traits::BinaryInputNode::left_input(node), super::plan_node_traits::BinaryInputNode::right_input(node)],
            PlanNodeEnum::CrossJoin(node) => vec![super::plan_node_traits::BinaryInputNode::left_input(node), super::plan_node_traits::BinaryInputNode::right_input(node)],
            PlanNodeEnum::HashInnerJoin(node) => vec![super::plan_node_traits::BinaryInputNode::left_input(node), super::plan_node_traits::BinaryInputNode::right_input(node)],
            PlanNodeEnum::HashLeftJoin(node) => vec![super::plan_node_traits::BinaryInputNode::left_input(node), super::plan_node_traits::BinaryInputNode::right_input(node)],

            // MultipleInputNode: 有多个子节点
            PlanNodeEnum::Expand(node) => node.dependencies().iter().map(|b| b.as_ref()).collect(),
            PlanNodeEnum::ExpandAll(node) => node.dependencies().iter().map(|b| b.as_ref()).collect(),
            PlanNodeEnum::AppendVertices(node) => node.dependencies().iter().map(|b| b.as_ref()).collect(),

            // UnionNode: 使用 dependencies() 获取所有子节点
            PlanNodeEnum::Union(node) => node.dependencies().iter().map(|b| b.as_ref()).collect(),
            PlanNodeEnum::Minus(node) => node.dependencies().iter().map(|b| b.as_ref()).collect(),
            PlanNodeEnum::Intersect(node) => node.dependencies().iter().map(|b| b.as_ref()).collect(),

            // ControlFlowNode
            PlanNodeEnum::Argument(_) => vec![],
            PlanNodeEnum::Loop(_) => vec![],
            PlanNodeEnum::PassThrough(_) => vec![],
            PlanNodeEnum::Select(_) => vec![],
        }
    }
}

/// PlanNodeEnum 的 NodeType trait 实现
///
/// 为每个变体提供统一的类型标识和分类
impl NodeType for PlanNodeEnum {
    fn node_type_id(&self) -> &'static str {
        match self {
            PlanNodeEnum::Start(_) => "start",
            PlanNodeEnum::Project(_) => "project",
            PlanNodeEnum::Sort(_) => "sort",
            PlanNodeEnum::Limit(_) => "limit",
            PlanNodeEnum::TopN(_) => "topn",
            PlanNodeEnum::Sample(_) => "sample",
            PlanNodeEnum::InnerJoin(_) => "inner_join",
            PlanNodeEnum::LeftJoin(_) => "left_join",
            PlanNodeEnum::CrossJoin(_) => "cross_join",
            PlanNodeEnum::GetVertices(_) => "get_vertices",
            PlanNodeEnum::GetEdges(_) => "get_edges",
            PlanNodeEnum::GetNeighbors(_) => "get_neighbors",
            PlanNodeEnum::ScanVertices(_) => "scan_vertices",
            PlanNodeEnum::ScanEdges(_) => "scan_edges",
            PlanNodeEnum::EdgeIndexScan(_) => "edge_index_scan",
            PlanNodeEnum::HashInnerJoin(_) => "hash_inner_join",
            PlanNodeEnum::HashLeftJoin(_) => "hash_left_join",
            PlanNodeEnum::IndexScan(_) => "index_scan",
            PlanNodeEnum::Expand(_) => "expand",
            PlanNodeEnum::ExpandAll(_) => "expand_all",
            PlanNodeEnum::Traverse(_) => "traverse",
            PlanNodeEnum::AppendVertices(_) => "append_vertices",
            PlanNodeEnum::Filter(_) => "filter",
            PlanNodeEnum::Aggregate(_) => "aggregate",
            PlanNodeEnum::Argument(_) => "argument",
            PlanNodeEnum::Loop(_) => "loop",
            PlanNodeEnum::PassThrough(_) => "pass_through",
            PlanNodeEnum::Select(_) => "select",
            PlanNodeEnum::DataCollect(_) => "data_collect",
            PlanNodeEnum::Dedup(_) => "dedup",
            PlanNodeEnum::PatternApply(_) => "pattern_apply",
            PlanNodeEnum::RollUpApply(_) => "rollup_apply",
            PlanNodeEnum::Union(_) => "union",
            PlanNodeEnum::Minus(_) => "minus",
            PlanNodeEnum::Intersect(_) => "intersect",
            PlanNodeEnum::Unwind(_) => "unwind",
            PlanNodeEnum::Assign(_) => "assign",
            PlanNodeEnum::MultiShortestPath(_) => "multi_shortest_path",
            PlanNodeEnum::BFSShortest(_) => "bfs_shortest",
            PlanNodeEnum::AllPaths(_) => "all_paths",
            PlanNodeEnum::ShortestPath(_) => "shortest_path",
            PlanNodeEnum::CreateSpace(_) => "create_space",
            PlanNodeEnum::DropSpace(_) => "drop_space",
            PlanNodeEnum::DescSpace(_) => "desc_space",
            PlanNodeEnum::ShowSpaces(_) => "show_spaces",
            PlanNodeEnum::CreateTag(_) => "create_tag",
            PlanNodeEnum::AlterTag(_) => "alter_tag",
            PlanNodeEnum::DescTag(_) => "desc_tag",
            PlanNodeEnum::DropTag(_) => "drop_tag",
            PlanNodeEnum::ShowTags(_) => "show_tags",
            PlanNodeEnum::CreateEdge(_) => "create_edge",
            PlanNodeEnum::AlterEdge(_) => "alter_edge",
            PlanNodeEnum::DescEdge(_) => "desc_edge",
            PlanNodeEnum::DropEdge(_) => "drop_edge",
            PlanNodeEnum::ShowEdges(_) => "show_edges",
            PlanNodeEnum::CreateTagIndex(_) => "create_tag_index",
            PlanNodeEnum::DropTagIndex(_) => "drop_tag_index",
            PlanNodeEnum::DescTagIndex(_) => "desc_tag_index",
            PlanNodeEnum::ShowTagIndexes(_) => "show_tag_indexes",
            PlanNodeEnum::CreateEdgeIndex(_) => "create_edge_index",
            PlanNodeEnum::DropEdgeIndex(_) => "drop_edge_index",
            PlanNodeEnum::DescEdgeIndex(_) => "desc_edge_index",
            PlanNodeEnum::ShowEdgeIndexes(_) => "show_edge_indexes",
            PlanNodeEnum::RebuildTagIndex(_) => "rebuild_tag_index",
            PlanNodeEnum::RebuildEdgeIndex(_) => "rebuild_edge_index",
            PlanNodeEnum::CreateUser(_) => "create_user",
            PlanNodeEnum::AlterUser(_) => "alter_user",
            PlanNodeEnum::DropUser(_) => "drop_user",
            PlanNodeEnum::ChangePassword(_) => "change_password",
            PlanNodeEnum::InsertVertices(_) => "insert_vertices",
            PlanNodeEnum::InsertEdges(_) => "insert_edges",
        }
    }

    fn node_type_name(&self) -> &'static str {
        match self {
            PlanNodeEnum::Start(_) => "Start",
            PlanNodeEnum::Project(_) => "Project",
            PlanNodeEnum::Sort(_) => "Sort",
            PlanNodeEnum::Limit(_) => "Limit",
            PlanNodeEnum::TopN(_) => "TopN",
            PlanNodeEnum::Sample(_) => "Sample",
            PlanNodeEnum::InnerJoin(_) => "Inner Join",
            PlanNodeEnum::LeftJoin(_) => "Left Join",
            PlanNodeEnum::CrossJoin(_) => "Cross Join",
            PlanNodeEnum::GetVertices(_) => "Get Vertices",
            PlanNodeEnum::GetEdges(_) => "Get Edges",
            PlanNodeEnum::GetNeighbors(_) => "Get Neighbors",
            PlanNodeEnum::ScanVertices(_) => "Scan Vertices",
            PlanNodeEnum::ScanEdges(_) => "Scan Edges",
            PlanNodeEnum::EdgeIndexScan(_) => "Edge Index Scan",
            PlanNodeEnum::HashInnerJoin(_) => "Hash Inner Join",
            PlanNodeEnum::HashLeftJoin(_) => "Hash Left Join",
            PlanNodeEnum::IndexScan(_) => "Index Scan",
            PlanNodeEnum::Expand(_) => "Expand",
            PlanNodeEnum::ExpandAll(_) => "Expand All",
            PlanNodeEnum::Traverse(_) => "Traverse",
            PlanNodeEnum::AppendVertices(_) => "Append Vertices",
            PlanNodeEnum::Filter(_) => "Filter",
            PlanNodeEnum::Aggregate(_) => "Aggregate",
            PlanNodeEnum::Argument(_) => "Argument",
            PlanNodeEnum::Loop(_) => "Loop",
            PlanNodeEnum::PassThrough(_) => "Pass Through",
            PlanNodeEnum::Select(_) => "Select",
            PlanNodeEnum::DataCollect(_) => "Data Collect",
            PlanNodeEnum::Dedup(_) => "Dedup",
            PlanNodeEnum::PatternApply(_) => "Pattern Apply",
            PlanNodeEnum::RollUpApply(_) => "RollUp Apply",
            PlanNodeEnum::Union(_) => "Union",
            PlanNodeEnum::Minus(_) => "Minus",
            PlanNodeEnum::Intersect(_) => "Intersect",
            PlanNodeEnum::Unwind(_) => "Unwind",
            PlanNodeEnum::Assign(_) => "Assign",
            PlanNodeEnum::MultiShortestPath(_) => "Multi Shortest Path",
            PlanNodeEnum::BFSShortest(_) => "BFS Shortest",
            PlanNodeEnum::AllPaths(_) => "All Paths",
            PlanNodeEnum::ShortestPath(_) => "Shortest Path",
            PlanNodeEnum::CreateSpace(_) => "Create Space",
            PlanNodeEnum::DropSpace(_) => "Drop Space",
            PlanNodeEnum::DescSpace(_) => "Describe Space",
            PlanNodeEnum::ShowSpaces(_) => "Show Spaces",
            PlanNodeEnum::CreateTag(_) => "Create Tag",
            PlanNodeEnum::AlterTag(_) => "Alter Tag",
            PlanNodeEnum::DescTag(_) => "Describe Tag",
            PlanNodeEnum::DropTag(_) => "Drop Tag",
            PlanNodeEnum::ShowTags(_) => "Show Tags",
            PlanNodeEnum::CreateEdge(_) => "Create Edge",
            PlanNodeEnum::AlterEdge(_) => "Alter Edge",
            PlanNodeEnum::DescEdge(_) => "Describe Edge",
            PlanNodeEnum::DropEdge(_) => "Drop Edge",
            PlanNodeEnum::ShowEdges(_) => "Show Edges",
            PlanNodeEnum::CreateTagIndex(_) => "Create Tag Index",
            PlanNodeEnum::DropTagIndex(_) => "Drop Tag Index",
            PlanNodeEnum::DescTagIndex(_) => "Describe Tag Index",
            PlanNodeEnum::ShowTagIndexes(_) => "Show Tag Indexes",
            PlanNodeEnum::CreateEdgeIndex(_) => "Create Edge Index",
            PlanNodeEnum::DropEdgeIndex(_) => "Drop Edge Index",
            PlanNodeEnum::DescEdgeIndex(_) => "Describe Edge Index",
            PlanNodeEnum::ShowEdgeIndexes(_) => "Show Edge Indexes",
            PlanNodeEnum::RebuildTagIndex(_) => "Rebuild Tag Index",
            PlanNodeEnum::RebuildEdgeIndex(_) => "Rebuild Edge Index",
            PlanNodeEnum::CreateUser(_) => "Create User",
            PlanNodeEnum::AlterUser(_) => "Alter User",
            PlanNodeEnum::DropUser(_) => "Drop User",
            PlanNodeEnum::ChangePassword(_) => "Change Password",
            PlanNodeEnum::InsertVertices(_) => "Insert Vertices",
            PlanNodeEnum::InsertEdges(_) => "Insert Edges",
        }
    }

    fn category(&self) -> NodeCategory {
        match self {
            PlanNodeEnum::Start(_) => NodeCategory::Scan,
            PlanNodeEnum::Project(_) => NodeCategory::Project,
            PlanNodeEnum::Sort(_) => NodeCategory::Sort,
            PlanNodeEnum::Limit(_) => NodeCategory::Sort,
            PlanNodeEnum::TopN(_) => NodeCategory::Sort,
            PlanNodeEnum::Sample(_) => NodeCategory::Other,
            PlanNodeEnum::InnerJoin(_) => NodeCategory::Join,
            PlanNodeEnum::LeftJoin(_) => NodeCategory::Join,
            PlanNodeEnum::CrossJoin(_) => NodeCategory::Join,
            PlanNodeEnum::GetVertices(_) => NodeCategory::Scan,
            PlanNodeEnum::GetEdges(_) => NodeCategory::Scan,
            PlanNodeEnum::GetNeighbors(_) => NodeCategory::Scan,
            PlanNodeEnum::ScanVertices(_) => NodeCategory::Scan,
            PlanNodeEnum::ScanEdges(_) => NodeCategory::Scan,
            PlanNodeEnum::EdgeIndexScan(_) => NodeCategory::Scan,
            PlanNodeEnum::HashInnerJoin(_) => NodeCategory::Join,
            PlanNodeEnum::HashLeftJoin(_) => NodeCategory::Join,
            PlanNodeEnum::IndexScan(_) => NodeCategory::Scan,
            PlanNodeEnum::Expand(_) => NodeCategory::Traversal,
            PlanNodeEnum::ExpandAll(_) => NodeCategory::Traversal,
            PlanNodeEnum::Traverse(_) => NodeCategory::Traversal,
            PlanNodeEnum::AppendVertices(_) => NodeCategory::Traversal,
            PlanNodeEnum::Filter(_) => NodeCategory::Filter,
            PlanNodeEnum::Aggregate(_) => NodeCategory::Aggregate,
            PlanNodeEnum::Argument(_) => NodeCategory::Control,
            PlanNodeEnum::Loop(_) => NodeCategory::Control,
            PlanNodeEnum::PassThrough(_) => NodeCategory::Control,
            PlanNodeEnum::Select(_) => NodeCategory::Control,
            PlanNodeEnum::DataCollect(_) => NodeCategory::DataCollect,
            PlanNodeEnum::Dedup(_) => NodeCategory::Aggregate,
            PlanNodeEnum::PatternApply(_) => NodeCategory::DataCollect,
            PlanNodeEnum::RollUpApply(_) => NodeCategory::DataCollect,
            PlanNodeEnum::Union(_) => NodeCategory::SetOp,
            PlanNodeEnum::Minus(_) => NodeCategory::SetOp,
            PlanNodeEnum::Intersect(_) => NodeCategory::SetOp,
            PlanNodeEnum::Unwind(_) => NodeCategory::DataCollect,
            PlanNodeEnum::Assign(_) => NodeCategory::DataCollect,
            PlanNodeEnum::MultiShortestPath(_) => NodeCategory::Path,
            PlanNodeEnum::BFSShortest(_) => NodeCategory::Path,
            PlanNodeEnum::AllPaths(_) => NodeCategory::Path,
            PlanNodeEnum::ShortestPath(_) => NodeCategory::Path,
            PlanNodeEnum::CreateSpace(_) => NodeCategory::Admin,
            PlanNodeEnum::DropSpace(_) => NodeCategory::Admin,
            PlanNodeEnum::DescSpace(_) => NodeCategory::Admin,
            PlanNodeEnum::ShowSpaces(_) => NodeCategory::Admin,
            PlanNodeEnum::CreateTag(_) => NodeCategory::Admin,
            PlanNodeEnum::AlterTag(_) => NodeCategory::Admin,
            PlanNodeEnum::DescTag(_) => NodeCategory::Admin,
            PlanNodeEnum::DropTag(_) => NodeCategory::Admin,
            PlanNodeEnum::ShowTags(_) => NodeCategory::Admin,
            PlanNodeEnum::CreateEdge(_) => NodeCategory::Admin,
            PlanNodeEnum::AlterEdge(_) => NodeCategory::Admin,
            PlanNodeEnum::DescEdge(_) => NodeCategory::Admin,
            PlanNodeEnum::DropEdge(_) => NodeCategory::Admin,
            PlanNodeEnum::ShowEdges(_) => NodeCategory::Admin,
            PlanNodeEnum::CreateTagIndex(_) => NodeCategory::Admin,
            PlanNodeEnum::DropTagIndex(_) => NodeCategory::Admin,
            PlanNodeEnum::DescTagIndex(_) => NodeCategory::Admin,
            PlanNodeEnum::ShowTagIndexes(_) => NodeCategory::Admin,
            PlanNodeEnum::CreateEdgeIndex(_) => NodeCategory::Admin,
            PlanNodeEnum::DropEdgeIndex(_) => NodeCategory::Admin,
            PlanNodeEnum::DescEdgeIndex(_) => NodeCategory::Admin,
            PlanNodeEnum::ShowEdgeIndexes(_) => NodeCategory::Admin,
            PlanNodeEnum::RebuildTagIndex(_) => NodeCategory::Admin,
            PlanNodeEnum::RebuildEdgeIndex(_) => NodeCategory::Admin,
            PlanNodeEnum::CreateUser(_) => NodeCategory::Admin,
            PlanNodeEnum::AlterUser(_) => NodeCategory::Admin,
            PlanNodeEnum::DropUser(_) => NodeCategory::Admin,
            PlanNodeEnum::ChangePassword(_) => NodeCategory::Admin,
            PlanNodeEnum::InsertVertices(_) => NodeCategory::Admin,
            PlanNodeEnum::InsertEdges(_) => NodeCategory::Admin,
        }
    }
}

/// NodeTypeMapping trait 实现
///
/// 提供 PlanNodeEnum 到 ExecutorEnum 的映射
impl NodeTypeMapping for PlanNodeEnum {
    fn corresponding_executor_type(&self) -> Option<&'static str> {
        match self {
            PlanNodeEnum::Start(_) => Some("start"),
            PlanNodeEnum::Project(_) => Some("project"),
            PlanNodeEnum::Sort(_) => Some("sort"),
            PlanNodeEnum::Limit(_) => Some("limit"),
            PlanNodeEnum::TopN(_) => Some("topn"),
            PlanNodeEnum::Sample(_) => Some("sample"),
            PlanNodeEnum::InnerJoin(_) => Some("inner_join"),
            PlanNodeEnum::LeftJoin(_) => Some("left_join"),
            PlanNodeEnum::CrossJoin(_) => Some("cross_join"),
            PlanNodeEnum::GetVertices(_) => Some("get_vertices"),
            PlanNodeEnum::GetEdges(_) => Some("get_edges"),
            PlanNodeEnum::GetNeighbors(_) => Some("get_neighbors"),
            PlanNodeEnum::ScanVertices(_) => Some("scan_vertices"),
            PlanNodeEnum::ScanEdges(_) => Some("scan_edges"),
            PlanNodeEnum::EdgeIndexScan(_) => Some("edge_index_scan"),
            PlanNodeEnum::HashInnerJoin(_) => Some("hash_inner_join"),
            PlanNodeEnum::HashLeftJoin(_) => Some("hash_left_join"),
            PlanNodeEnum::IndexScan(_) => Some("index_scan"),
            PlanNodeEnum::Expand(_) => Some("expand"),
            PlanNodeEnum::ExpandAll(_) => Some("expand_all"),
            PlanNodeEnum::Traverse(_) => Some("traverse"),
            PlanNodeEnum::AppendVertices(_) => Some("append_vertices"),
            PlanNodeEnum::Filter(_) => Some("filter"),
            PlanNodeEnum::Aggregate(_) => Some("aggregate"),
            PlanNodeEnum::Argument(_) => Some("argument"),
            PlanNodeEnum::Loop(_) => Some("loop"),
            PlanNodeEnum::PassThrough(_) => Some("pass_through"),
            PlanNodeEnum::Select(_) => Some("select"),
            PlanNodeEnum::DataCollect(_) => Some("data_collect"),
            PlanNodeEnum::Dedup(_) => Some("dedup"),
            PlanNodeEnum::PatternApply(_) => Some("pattern_apply"),
            PlanNodeEnum::RollUpApply(_) => Some("rollup_apply"),
            PlanNodeEnum::Union(_) => Some("union"),
            PlanNodeEnum::Minus(_) => Some("minus"),
            PlanNodeEnum::Intersect(_) => Some("intersect"),
            PlanNodeEnum::Unwind(_) => Some("unwind"),
            PlanNodeEnum::Assign(_) => Some("assign"),
            PlanNodeEnum::MultiShortestPath(_) => Some("multi_shortest_path"),
            PlanNodeEnum::BFSShortest(_) => Some("bfs_shortest"),
            PlanNodeEnum::AllPaths(_) => Some("all_paths"),
            PlanNodeEnum::ShortestPath(_) => Some("shortest_path"),
            PlanNodeEnum::CreateSpace(_) => Some("create_space"),
            PlanNodeEnum::DropSpace(_) => Some("drop_space"),
            PlanNodeEnum::DescSpace(_) => Some("desc_space"),
            PlanNodeEnum::ShowSpaces(_) => Some("show_spaces"),
            PlanNodeEnum::CreateTag(_) => Some("create_tag"),
            PlanNodeEnum::AlterTag(_) => Some("alter_tag"),
            PlanNodeEnum::DescTag(_) => Some("desc_tag"),
            PlanNodeEnum::DropTag(_) => Some("drop_tag"),
            PlanNodeEnum::ShowTags(_) => Some("show_tags"),
            PlanNodeEnum::CreateEdge(_) => Some("create_edge"),
            PlanNodeEnum::AlterEdge(_) => Some("alter_edge"),
            PlanNodeEnum::DescEdge(_) => Some("desc_edge"),
            PlanNodeEnum::DropEdge(_) => Some("drop_edge"),
            PlanNodeEnum::ShowEdges(_) => Some("show_edges"),
            PlanNodeEnum::CreateTagIndex(_) => Some("create_tag_index"),
            PlanNodeEnum::DropTagIndex(_) => Some("drop_tag_index"),
            PlanNodeEnum::DescTagIndex(_) => Some("desc_tag_index"),
            PlanNodeEnum::ShowTagIndexes(_) => Some("show_tag_indexes"),
            PlanNodeEnum::CreateEdgeIndex(_) => Some("create_edge_index"),
            PlanNodeEnum::DropEdgeIndex(_) => Some("drop_edge_index"),
            PlanNodeEnum::DescEdgeIndex(_) => Some("desc_edge_index"),
            PlanNodeEnum::ShowEdgeIndexes(_) => Some("show_edge_indexes"),
            PlanNodeEnum::RebuildTagIndex(_) => Some("rebuild_tag_index"),
            PlanNodeEnum::RebuildEdgeIndex(_) => Some("rebuild_edge_index"),
            PlanNodeEnum::CreateUser(_) => Some("create_user"),
            PlanNodeEnum::AlterUser(_) => Some("alter_user"),
            PlanNodeEnum::DropUser(_) => Some("drop_user"),
            PlanNodeEnum::ChangePassword(_) => Some("change_password"),
            PlanNodeEnum::InsertVertices(_) => Some("insert_vertices"),
            PlanNodeEnum::InsertEdges(_) => Some("insert_edges"),
        }
    }
}
