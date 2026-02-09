//! PlanNode 访问者 trait
//!
//! 提供统一的 PlanNode 遍历接口，简化优化规则和数据转换的实现。
//! 访问者模式使得可以在不修改节点结构的情况下对节点进行操作。

use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;
use crate::query::planner::plan::core::nodes::aggregate_node::*;
use crate::query::planner::plan::core::nodes::control_flow_node::*;
use crate::query::planner::plan::core::nodes::data_processing_node::*;
use crate::query::planner::plan::core::nodes::edge_nodes::*;
use crate::query::planner::plan::core::nodes::filter_node::*;
use crate::query::planner::plan::core::nodes::graph_scan_node::*;
use crate::query::planner::plan::core::nodes::index_nodes::*;
use crate::query::planner::plan::core::nodes::join_node::*;
use crate::query::planner::plan::core::nodes::project_node::*;
use crate::query::planner::plan::core::nodes::sample_node::*;
use crate::query::planner::plan::core::nodes::set_operations_node::*;
use crate::query::planner::plan::core::nodes::sort_node::*;
use crate::query::planner::plan::core::nodes::space_nodes::*;
use crate::query::planner::plan::core::nodes::start_node::*;
use crate::query::planner::plan::core::nodes::tag_nodes::*;
use crate::query::planner::plan::core::nodes::traversal_node::*;

/// PlanNode 访问者 trait
///
/// 提供统一的 PlanNode 遍历接口，简化优化规则和数据转换的实现。
/// 访问者模式使得可以在不修改节点结构的情况下对节点进行操作。
pub trait PlanNodeVisitor {
    /// 访问结果的类型
    type Result;

    /// 访问开始节点
    fn visit_start(&mut self, _node: &StartNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问项目节点
    fn visit_project(&mut self, _node: &ProjectNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问过滤节点
    fn visit_filter(&mut self, _node: &FilterNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问排序节点
    fn visit_sort(&mut self, _node: &SortNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问限制节点
    fn visit_limit(&mut self, _node: &LimitNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问 TopN 节点
    fn visit_topn(&mut self, _node: &TopNNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问采样节点
    fn visit_sample(&mut self, _node: &SampleNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问去重节点
    fn visit_dedup(&mut self, _node: &DedupNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问获取顶点节点
    fn visit_get_vertices(&mut self, _node: &GetVerticesNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问获取边节点
    fn visit_get_edges(&mut self, _node: &GetEdgesNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问获取邻居节点
    fn visit_get_neighbors(&mut self, _node: &GetNeighborsNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问扫描顶点节点
    fn visit_scan_vertices(&mut self, _node: &ScanVerticesNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问扫描边节点
    fn visit_scan_edges(&mut self, _node: &ScanEdgesNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问边索引扫描节点
    fn visit_edge_index_scan(&mut self, _node: &EdgeIndexScanNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问索引扫描节点
    fn visit_index_scan(&mut self, _node: &crate::query::planner::plan::algorithms::IndexScan) -> Self::Result {
        self.visit_default()
    }

    /// 访问全文索引扫描节点
    fn visit_fulltext_index_scan(&mut self, _node: &crate::query::planner::plan::algorithms::FulltextIndexScan) -> Self::Result {
        self.visit_default()
    }

    /// 访问扩展节点
    fn visit_expand(&mut self, _node: &ExpandNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问全扩展节点
    fn visit_expand_all(&mut self, _node: &ExpandAllNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问遍历节点
    fn visit_traverse(&mut self, _node: &TraverseNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问追加顶点节点
    fn visit_append_vertices(&mut self, _node: &AppendVerticesNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问内连接节点
    fn visit_inner_join(&mut self, _node: &InnerJoinNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问左连接节点
    fn visit_left_join(&mut self, _node: &LeftJoinNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问交叉连接节点
    fn visit_cross_join(&mut self, _node: &CrossJoinNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问哈希内连接节点
    fn visit_hash_inner_join(&mut self, _node: &HashInnerJoinNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问哈希左连接节点
    fn visit_hash_left_join(&mut self, _node: &HashLeftJoinNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问聚合节点
    fn visit_aggregate(&mut self, _node: &AggregateNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问参数节点
    fn visit_argument(&mut self, _node: &ArgumentNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问循环节点
    fn visit_loop(&mut self, _node: &LoopNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问透传节点
    fn visit_pass_through(&mut self, _node: &PassThroughNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问选择节点
    fn visit_select(&mut self, _node: &SelectNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问数据收集节点
    fn visit_data_collect(&mut self, _node: &DataCollectNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问模式应用节点
    fn visit_pattern_apply(&mut self, _node: &PatternApplyNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问卷起应用节点
    fn visit_rollup_apply(&mut self, _node: &RollUpApplyNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问并集节点
    fn visit_union(&mut self, _node: &UnionNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问差集节点
    fn visit_minus(&mut self, _node: &MinusNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问交集节点
    fn visit_intersect(&mut self, _node: &IntersectNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问展开节点
    fn visit_unwind(&mut self, _node: &UnwindNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问赋值节点
    fn visit_assign(&mut self, _node: &AssignNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问多源最短路径节点
    fn visit_multi_shortest_path(&mut self, _node: &crate::query::planner::plan::algorithms::MultiShortestPath) -> Self::Result {
        self.visit_default()
    }

    /// 访问 BFS 最短路径节点
    fn visit_bfs_shortest(&mut self, _node: &crate::query::planner::plan::algorithms::BFSShortest) -> Self::Result {
        self.visit_default()
    }

    /// 访问所有路径节点
    fn visit_all_paths(&mut self, _node: &crate::query::planner::plan::algorithms::AllPaths) -> Self::Result {
        self.visit_default()
    }

    /// 访问最短路径节点
    fn visit_shortest_path(&mut self, _node: &crate::query::planner::plan::algorithms::ShortestPath) -> Self::Result {
        self.visit_default()
    }

    /// 访问创建空间节点
    fn visit_create_space(&mut self, _node: &CreateSpaceNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问删除空间节点
    fn visit_drop_space(&mut self, _node: &DropSpaceNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问描述空间节点
    fn visit_desc_space(&mut self, _node: &DescSpaceNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问显示所有图空间节点
    fn visit_show_spaces(&mut self, _node: &ShowSpacesNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问创建标签节点
    fn visit_create_tag(&mut self, _node: &CreateTagNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问修改标签节点
    fn visit_alter_tag(&mut self, _node: &AlterTagNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问描述标签节点
    fn visit_desc_tag(&mut self, _node: &DescTagNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问删除标签节点
    fn visit_drop_tag(&mut self, _node: &DropTagNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问显示所有标签节点
    fn visit_show_tags(&mut self, _node: &ShowTagsNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问创建边类型节点
    fn visit_create_edge(&mut self, _node: &CreateEdgeNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问修改边类型节点
    fn visit_alter_edge(&mut self, _node: &AlterEdgeNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问描述边类型节点
    fn visit_desc_edge(&mut self, _node: &DescEdgeNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问删除边类型节点
    fn visit_drop_edge(&mut self, _node: &DropEdgeNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问显示所有边类型节点
    fn visit_show_edges(&mut self, _node: &ShowEdgesNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问创建标签索引节点
    fn visit_create_tag_index(&mut self, _node: &CreateTagIndexNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问删除标签索引节点
    fn visit_drop_tag_index(&mut self, _node: &DropTagIndexNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问描述标签索引节点
    fn visit_desc_tag_index(&mut self, _node: &DescTagIndexNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问显示所有标签索引节点
    fn visit_show_tag_indexes(&mut self, _node: &ShowTagIndexesNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问创建边索引节点
    fn visit_create_edge_index(&mut self, _node: &CreateEdgeIndexNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问删除边索引节点
    fn visit_drop_edge_index(&mut self, _node: &DropEdgeIndexNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问描述边索引节点
    fn visit_desc_edge_index(&mut self, _node: &DescEdgeIndexNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问显示所有边索引节点
    fn visit_show_edge_indexes(&mut self, _node: &ShowEdgeIndexesNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问重建标签索引节点
    fn visit_rebuild_tag_index(&mut self, _node: &RebuildTagIndexNode) -> Self::Result {
        self.visit_default()
    }

    /// 访问重建边索引节点
    fn visit_rebuild_edge_index(&mut self, _node: &RebuildEdgeIndexNode) -> Self::Result {
        self.visit_default()
    }

    /// 默认访问实现
    fn visit_default(&mut self) -> Self::Result;

    /// 统一的访问入口
    fn visit(&mut self, node: &PlanNodeEnum) -> Self::Result {
        match node {
            PlanNodeEnum::Start(n) => self.visit_start(n),
            PlanNodeEnum::Project(n) => self.visit_project(n),
            PlanNodeEnum::Filter(n) => self.visit_filter(n),
            PlanNodeEnum::Sort(n) => self.visit_sort(n),
            PlanNodeEnum::Limit(n) => self.visit_limit(n),
            PlanNodeEnum::TopN(n) => self.visit_topn(n),
            PlanNodeEnum::Sample(n) => self.visit_sample(n),
            PlanNodeEnum::Dedup(n) => self.visit_dedup(n),
            PlanNodeEnum::GetVertices(n) => self.visit_get_vertices(n),
            PlanNodeEnum::GetEdges(n) => self.visit_get_edges(n),
            PlanNodeEnum::GetNeighbors(n) => self.visit_get_neighbors(n),
            PlanNodeEnum::ScanVertices(n) => self.visit_scan_vertices(n),
            PlanNodeEnum::ScanEdges(n) => self.visit_scan_edges(n),
            PlanNodeEnum::EdgeIndexScan(n) => self.visit_edge_index_scan(n),
            PlanNodeEnum::IndexScan(n) => self.visit_index_scan(n),
            PlanNodeEnum::FulltextIndexScan(n) => self.visit_fulltext_index_scan(n),
            PlanNodeEnum::Expand(n) => self.visit_expand(n),
            PlanNodeEnum::ExpandAll(n) => self.visit_expand_all(n),
            PlanNodeEnum::Traverse(n) => self.visit_traverse(n),
            PlanNodeEnum::AppendVertices(n) => self.visit_append_vertices(n),
            PlanNodeEnum::InnerJoin(n) => self.visit_inner_join(n),
            PlanNodeEnum::LeftJoin(n) => self.visit_left_join(n),
            PlanNodeEnum::CrossJoin(n) => self.visit_cross_join(n),
            PlanNodeEnum::HashInnerJoin(n) => self.visit_hash_inner_join(n),
            PlanNodeEnum::HashLeftJoin(n) => self.visit_hash_left_join(n),
            PlanNodeEnum::Aggregate(n) => self.visit_aggregate(n),
            PlanNodeEnum::Argument(n) => self.visit_argument(n),
            PlanNodeEnum::Loop(n) => self.visit_loop(n),
            PlanNodeEnum::PassThrough(n) => self.visit_pass_through(n),
            PlanNodeEnum::Select(n) => self.visit_select(n),
            PlanNodeEnum::DataCollect(n) => self.visit_data_collect(n),
            PlanNodeEnum::PatternApply(n) => self.visit_pattern_apply(n),
            PlanNodeEnum::RollUpApply(n) => self.visit_rollup_apply(n),
            PlanNodeEnum::Union(n) => self.visit_union(n),
            PlanNodeEnum::Minus(n) => self.visit_minus(n),
            PlanNodeEnum::Intersect(n) => self.visit_intersect(n),
            PlanNodeEnum::Unwind(n) => self.visit_unwind(n),
            PlanNodeEnum::Assign(n) => self.visit_assign(n),
            PlanNodeEnum::MultiShortestPath(n) => self.visit_multi_shortest_path(n),
            PlanNodeEnum::BFSShortest(n) => self.visit_bfs_shortest(n),
            PlanNodeEnum::AllPaths(n) => self.visit_all_paths(n),
            PlanNodeEnum::ShortestPath(n) => self.visit_shortest_path(n),
            PlanNodeEnum::CreateSpace(n) => self.visit_create_space(n),
            PlanNodeEnum::DropSpace(n) => self.visit_drop_space(n),
            PlanNodeEnum::DescSpace(n) => self.visit_desc_space(n),
            PlanNodeEnum::ShowSpaces(n) => self.visit_show_spaces(n),
            PlanNodeEnum::CreateTag(n) => self.visit_create_tag(n),
            PlanNodeEnum::AlterTag(n) => self.visit_alter_tag(n),
            PlanNodeEnum::DescTag(n) => self.visit_desc_tag(n),
            PlanNodeEnum::DropTag(n) => self.visit_drop_tag(n),
            PlanNodeEnum::ShowTags(n) => self.visit_show_tags(n),
            PlanNodeEnum::CreateEdge(n) => self.visit_create_edge(n),
            PlanNodeEnum::AlterEdge(n) => self.visit_alter_edge(n),
            PlanNodeEnum::DescEdge(n) => self.visit_desc_edge(n),
            PlanNodeEnum::DropEdge(n) => self.visit_drop_edge(n),
            PlanNodeEnum::ShowEdges(n) => self.visit_show_edges(n),
            PlanNodeEnum::CreateTagIndex(n) => self.visit_create_tag_index(n),
            PlanNodeEnum::DropTagIndex(n) => self.visit_drop_tag_index(n),
            PlanNodeEnum::DescTagIndex(_n) => self.visit_desc_tag_index(_n),
            PlanNodeEnum::ShowTagIndexes(_n) => self.visit_show_tag_indexes(_n),
            PlanNodeEnum::CreateEdgeIndex(_n) => self.visit_create_edge_index(_n),
            PlanNodeEnum::DropEdgeIndex(_n) => self.visit_drop_edge_index(_n),
            PlanNodeEnum::DescEdgeIndex(_n) => self.visit_desc_edge_index(_n),
            PlanNodeEnum::ShowEdgeIndexes(_n) => self.visit_show_edge_indexes(_n),
            PlanNodeEnum::RebuildTagIndex(_n) => self.visit_rebuild_tag_index(_n),
            PlanNodeEnum::RebuildEdgeIndex(_n) => self.visit_rebuild_edge_index(_n),
            PlanNodeEnum::CreateUser(_n) => self.visit_default(),
            PlanNodeEnum::AlterUser(_n) => self.visit_default(),
            PlanNodeEnum::DropUser(_n) => self.visit_default(),
            PlanNodeEnum::ChangePassword(_n) => self.visit_default(),
        }
    }
}

/// 默认的 PlanNode 访问者实现
///
/// 提供空操作的默认实现，用于只需要访问部分节点类型的场景。
pub struct DefaultPlanNodeVisitor;

impl PlanNodeVisitor for DefaultPlanNodeVisitor {
    type Result = ();

    fn visit_default(&mut self) {}
}
