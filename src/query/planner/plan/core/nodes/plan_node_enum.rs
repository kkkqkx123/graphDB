//! PlanNode 枚举定义
//!
//!

use super::plan_node_traits::PlanNode;
use super::plan_node_category::PlanNodeCategory;
use super::admin_node::{
    CreateSpaceNode, DropSpaceNode, DescSpaceNode, ShowSpacesNode,
    CreateTagNode, AlterTagNode, DescTagNode, DropTagNode, ShowTagsNode,
    CreateEdgeNode, AlterEdgeNode, DescEdgeNode, DropEdgeNode, ShowEdgesNode,
    CreateTagIndexNode, DropTagIndexNode, DescTagIndexNode, ShowTagIndexesNode,
    CreateEdgeIndexNode, DropEdgeIndexNode, DescEdgeIndexNode, ShowEdgeIndexesNode,
    RebuildTagIndexNode, RebuildEdgeIndexNode,
};
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
    GetEdgesNode, GetNeighborsNode, GetVerticesNode, ScanEdgesNode, ScanVerticesNode,
};
pub use super::join_node::{
    CrossJoinNode, HashInnerJoinNode, HashLeftJoinNode, InnerJoinNode, LeftJoinNode,
};
pub use super::project_node::ProjectNode;
pub use super::sample_node::SampleNode;
pub use super::sort_node::{LimitNode, SortNode, TopNNode};
pub use super::start_node::StartNode;
pub use super::traversal_node::{AppendVerticesNode, ExpandAllNode, ExpandNode, TraverseNode};
pub use crate::query::planner::plan::algorithms::{
    AllPaths, BFSShortest, FulltextIndexScan, IndexScan, MultiShortestPath, ShortestPath,
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
    /// 哈希内连接节点
    HashInnerJoin(HashInnerJoinNode),
    /// 哈希左连接节点
    HashLeftJoin(HashLeftJoinNode),
    /// 笛卡尔积节点
    CartesianProduct(CrossJoinNode),
    /// 索引扫描节点
    IndexScan(IndexScan),
    /// 全文索引扫描节点
    FulltextIndexScan(FulltextIndexScan),
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

    pub fn is_fulltext_index_scan(&self) -> bool {
        matches!(self, PlanNodeEnum::FulltextIndexScan(_))
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

    pub fn is_cartesian_product(&self) -> bool {
        matches!(self, PlanNodeEnum::CartesianProduct(_))
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
                | PlanNodeEnum::GetVertices(_)
                | PlanNodeEnum::GetEdges(_)
                | PlanNodeEnum::GetNeighbors(_)
                | PlanNodeEnum::IndexScan(_)
                | PlanNodeEnum::FulltextIndexScan(_)
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
                | PlanNodeEnum::CartesianProduct(_)
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
            PlanNodeEnum::HashInnerJoin(_) => "HashInnerJoin",
            PlanNodeEnum::HashLeftJoin(_) => "HashLeftJoin",
            PlanNodeEnum::CartesianProduct(_) => "CartesianProduct",
            PlanNodeEnum::IndexScan(_) => "IndexScan",
            PlanNodeEnum::FulltextIndexScan(_) => "FulltextIndexScan",
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
            PlanNodeEnum::HashInnerJoin(_) => PlanNodeCategory::Join,
            PlanNodeEnum::HashLeftJoin(_) => PlanNodeCategory::Join,
            PlanNodeEnum::CartesianProduct(_) => PlanNodeCategory::Join,
            PlanNodeEnum::IndexScan(_) => PlanNodeCategory::Access,
            PlanNodeEnum::FulltextIndexScan(_) => PlanNodeCategory::Access,
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

    pub fn as_fulltext_index_scan(&self) -> Option<&FulltextIndexScan> {
        match self {
            PlanNodeEnum::FulltextIndexScan(node) => Some(node),
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

    pub fn as_cartesian_product(&self) -> Option<&CrossJoinNode> {
        match self {
            PlanNodeEnum::CartesianProduct(node) => Some(node),
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
            PlanNodeEnum::CartesianProduct(node) => {
                let mut desc = PlanNodeDescription::new("CartesianProduct", node.id());
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
            PlanNodeEnum::FulltextIndexScan(node) => {
                let mut desc = PlanNodeDescription::new("FulltextIndexScan", node.id());
                if let Some(var) = node.output_var() {
                    desc = desc.with_output_var(var.name.clone());
                }
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

            // 管理节点 - 简化描述
            _ => {
                PlanNodeDescription::new(self.name(), self.id())
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

    /// 访问Start节点 - 编译时分发
    fn visit_start(&mut self, node: &StartNode) -> Self::Result;

    /// 访问Project节点 - 编译时分发
    fn visit_project(&mut self, node: &ProjectNode) -> Self::Result;

    /// 访问Sort节点 - 编译时分发
    fn visit_sort(&mut self, node: &SortNode) -> Self::Result;

    /// 访问Limit节点 - 编译时分发
    fn visit_limit(&mut self, node: &LimitNode) -> Self::Result;

    /// 访问TopN节点 - 编译时分发
    fn visit_topn(&mut self, node: &TopNNode) -> Self::Result;

    /// 访问InnerJoin节点 - 编译时分发
    fn visit_inner_join(&mut self, node: &InnerJoinNode) -> Self::Result;

    /// 访问LeftJoin节点 - 编译时分发
    fn visit_left_join(&mut self, node: &LeftJoinNode) -> Self::Result;

    /// 访问CrossJoin节点 - 编译时分发
    fn visit_cross_join(&mut self, node: &CrossJoinNode) -> Self::Result;

    /// 访问GetVertices节点 - 编译时分发
    fn visit_get_vertices(&mut self, node: &GetVerticesNode) -> Self::Result;

    /// 访问GetEdges节点 - 编译时分发
    fn visit_get_edges(&mut self, node: &GetEdgesNode) -> Self::Result;

    /// 访问GetNeighbors节点 - 编译时分发
    fn visit_get_neighbors(&mut self, node: &GetNeighborsNode) -> Self::Result;

    /// 访问ScanVertices节点 - 编译时分发
    fn visit_scan_vertices(&mut self, node: &ScanVerticesNode) -> Self::Result;

    /// 访问ScanEdges节点 - 编译时分发
    fn visit_scan_edges(&mut self, node: &ScanEdgesNode) -> Self::Result;

    /// 访问Expand节点 - 编译时分发
    fn visit_expand(&mut self, node: &ExpandNode) -> Self::Result;

    /// 访问ExpandAll节点 - 编译时分发
    fn visit_expand_all(&mut self, node: &ExpandAllNode) -> Self::Result;

    /// 访问Traverse节点 - 编译时分发
    fn visit_traverse(&mut self, node: &TraverseNode) -> Self::Result;

    /// 访问AppendVertices节点 - 编译时分发
    fn visit_append_vertices(&mut self, node: &AppendVerticesNode) -> Self::Result;

    /// 访问Filter节点 - 编译时分发
    fn visit_filter(&mut self, node: &FilterNode) -> Self::Result;

    /// 访问Aggregate节点 - 编译时分发
    fn visit_aggregate(&mut self, node: &AggregateNode) -> Self::Result;

    /// 访问Argument节点 - 编译时分发
    fn visit_argument(&mut self, node: &ArgumentNode) -> Self::Result;

    /// 访问Loop节点 - 编译时分发
    fn visit_loop(&mut self, node: &LoopNode) -> Self::Result;

    /// 访问PassThrough节点 - 编译时分发
    fn visit_pass_through(&mut self, node: &PassThroughNode) -> Self::Result;

    /// 访问Select节点 - 编译时分发
    fn visit_select(&mut self, node: &SelectNode) -> Self::Result;

    /// 访问DataCollect节点 - 编译时分发
    fn visit_data_collect(&mut self, node: &DataCollectNode) -> Self::Result;

    /// 访问Dedup节点 - 编译时分发
    fn visit_dedup(&mut self, node: &DedupNode) -> Self::Result;

    /// 访问PatternApply节点 - 编译时分发
    fn visit_pattern_apply(&mut self, node: &PatternApplyNode) -> Self::Result;

    /// 访问RollUpApply节点 - 编译时分发
    fn visit_roll_up_apply(&mut self, node: &RollUpApplyNode) -> Self::Result;

    /// 访问Union节点 - 编译时分发
    fn visit_union(&mut self, node: &UnionNode) -> Self::Result;

    /// 访问Unwind节点 - 编译时分发
    fn visit_unwind(&mut self, node: &UnwindNode) -> Self::Result;

    /// 访问Assign节点 - 编译时分发
    fn visit_assign(&mut self, node: &AssignNode) -> Self::Result;

    /// 访问IndexScan节点 - 编译时分发
    fn visit_index_scan(&mut self, node: &IndexScan) -> Self::Result;

    /// 访问FulltextIndexScan节点 - 编译时分发
    fn visit_fulltext_index_scan(&mut self, node: &FulltextIndexScan) -> Self::Result;

    /// 访问MultiShortestPath节点 - 编译时分发
    fn visit_multi_shortest_path(&mut self, node: &MultiShortestPath) -> Self::Result;

    /// 访问BFSShortest节点 - 编译时分发
    fn visit_bfs_shortest(&mut self, node: &BFSShortest) -> Self::Result;

    /// 访问AllPaths节点 - 编译时分发
    fn visit_all_paths(&mut self, node: &AllPaths) -> Self::Result;

    /// 访问ShortestPath节点 - 编译时分发
    fn visit_shortest_path(&mut self, node: &ShortestPath) -> Self::Result;
}

impl PlanNodeEnum {
    /// 零成本访问者模式 - 编译时分发
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
            PlanNodeEnum::InnerJoin(node) => visitor.visit_inner_join(node),
            PlanNodeEnum::LeftJoin(node) => visitor.visit_left_join(node),
            PlanNodeEnum::CrossJoin(node) => visitor.visit_cross_join(node),
            PlanNodeEnum::GetVertices(node) => visitor.visit_get_vertices(node),
            PlanNodeEnum::GetEdges(node) => visitor.visit_get_edges(node),
            PlanNodeEnum::GetNeighbors(node) => visitor.visit_get_neighbors(node),
            PlanNodeEnum::ScanVertices(node) => visitor.visit_scan_vertices(node),
            PlanNodeEnum::ScanEdges(node) => visitor.visit_scan_edges(node),
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
            PlanNodeEnum::Unwind(node) => visitor.visit_unwind(node),
            PlanNodeEnum::Assign(node) => visitor.visit_assign(node),
            PlanNodeEnum::IndexScan(node) => visitor.visit_index_scan(node),
            PlanNodeEnum::FulltextIndexScan(node) => visitor.visit_fulltext_index_scan(node),
            PlanNodeEnum::MultiShortestPath(node) => visitor.visit_multi_shortest_path(node),
            PlanNodeEnum::BFSShortest(node) => visitor.visit_bfs_shortest(node),
            PlanNodeEnum::AllPaths(node) => visitor.visit_all_paths(node),
            PlanNodeEnum::ShortestPath(node) => visitor.visit_shortest_path(node),

            // 管理节点类型 - 暂时使用默认处理
            _ => unimplemented!("管理节点的访问者模式尚未实现"),
        }
    }
}
