//! PlanNode 访问者接口
//! 提供统一的 PlanNode 遍历和转换机制

use crate::query::planner::plan::core::nodes::{
    AggregateNode, ArgumentNode, AssignNode, CreateEdgeNode, CreateEdgeIndexNode,
    CreateSpaceNode, CreateTagNode, CreateTagIndexNode, CrossJoinNode, DataCollectNode,
    DedupNode, DescEdgeNode, DescSpaceNode, DescTagNode, DescTagIndexNode, DropEdgeNode,
    DropSpaceNode, DropTagNode, DropTagIndexNode, DropEdgeIndexNode, EdgeIndexScanNode, ExpandAllNode, ExpandNode,
    FilterNode, GetEdgesNode, GetNeighborsNode, GetVerticesNode, HashInnerJoinNode, HashLeftJoinNode,
    InnerJoinNode, IntersectNode, LeftJoinNode, LimitNode, LoopNode, MinusNode, PassThroughNode, PatternApplyNode,
    ProjectNode, RollUpApplyNode, SampleNode, ScanEdgesNode, ScanVerticesNode, SelectNode,
    ShowEdgesNode, ShowSpacesNode, ShowTagsNode, ShowTagIndexesNode, ShowEdgeIndexesNode,
    SortNode, StartNode, TopNNode, TraverseNode, UnwindNode, UnionNode, AppendVerticesNode,
    RebuildEdgeIndexNode, RebuildTagIndexNode,
    CreateUserNode, AlterUserNode, DropUserNode, ChangePasswordNode,
};
use crate::query::planner::plan::algorithms::{
    AllPaths, BFSShortest, FulltextIndexScan, IndexScan, MultiShortestPath, ShortestPath,
};
use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;

pub trait PlanNodeVisitor {
    type Result;
    
    fn visit_start(&mut self, node: &StartNode) -> Self::Result;
    fn visit_project(&mut self, node: &ProjectNode) -> Self::Result;
    fn visit_sort(&mut self, node: &SortNode) -> Self::Result;
    fn visit_limit(&mut self, node: &LimitNode) -> Self::Result;
    fn visit_top_n(&mut self, node: &TopNNode) -> Self::Result;
    fn visit_sample(&mut self, node: &SampleNode) -> Self::Result;
    fn visit_inner_join(&mut self, node: &InnerJoinNode) -> Self::Result;
    fn visit_left_join(&mut self, node: &LeftJoinNode) -> Self::Result;
    fn visit_cross_join(&mut self, node: &CrossJoinNode) -> Self::Result;
    fn visit_get_vertices(&mut self, node: &GetVerticesNode) -> Self::Result;
    fn visit_get_edges(&mut self, node: &GetEdgesNode) -> Self::Result;
    fn visit_get_neighbors(&mut self, node: &GetNeighborsNode) -> Self::Result;
    fn visit_scan_vertices(&mut self, node: &ScanVerticesNode) -> Self::Result;
    fn visit_scan_edges(&mut self, node: &ScanEdgesNode) -> Self::Result;
    fn visit_edge_index_scan(&mut self, node: &EdgeIndexScanNode) -> Self::Result;
    fn visit_hash_inner_join(&mut self, node: &HashInnerJoinNode) -> Self::Result;
    fn visit_hash_left_join(&mut self, node: &HashLeftJoinNode) -> Self::Result;
    fn visit_index_scan(&mut self, node: &IndexScan) -> Self::Result;
    fn visit_fulltext_index_scan(&mut self, node: &FulltextIndexScan) -> Self::Result;
    fn visit_expand(&mut self, node: &ExpandNode) -> Self::Result;
    fn visit_expand_all(&mut self, node: &ExpandAllNode) -> Self::Result;
    fn visit_traverse(&mut self, node: &TraverseNode) -> Self::Result;
    fn visit_append_vertices(&mut self, node: &AppendVerticesNode) -> Self::Result;
    fn visit_filter(&mut self, node: &FilterNode) -> Self::Result;
    fn visit_aggregate(&mut self, node: &AggregateNode) -> Self::Result;
    fn visit_argument(&mut self, node: &ArgumentNode) -> Self::Result;
    fn visit_loop(&mut self, node: &LoopNode) -> Self::Result;
    fn visit_pass_through(&mut self, node: &PassThroughNode) -> Self::Result;
    fn visit_select(&mut self, node: &SelectNode) -> Self::Result;
    fn visit_data_collect(&mut self, node: &DataCollectNode) -> Self::Result;
    fn visit_dedup(&mut self, node: &DedupNode) -> Self::Result;
    fn visit_pattern_apply(&mut self, node: &PatternApplyNode) -> Self::Result;
    fn visit_roll_up_apply(&mut self, node: &RollUpApplyNode) -> Self::Result;
    fn visit_union(&mut self, node: &UnionNode) -> Self::Result;
    fn visit_minus(&mut self, node: &MinusNode) -> Self::Result;
    fn visit_intersect(&mut self, node: &IntersectNode) -> Self::Result;
    fn visit_unwind(&mut self, node: &UnwindNode) -> Self::Result;
    fn visit_assign(&mut self, node: &AssignNode) -> Self::Result;
    fn visit_multi_shortest_path(&mut self, node: &MultiShortestPath) -> Self::Result;
    fn visit_bfs_shortest(&mut self, node: &BFSShortest) -> Self::Result;
    fn visit_all_paths(&mut self, node: &AllPaths) -> Self::Result;
    fn visit_shortest_path(&mut self, node: &ShortestPath) -> Self::Result;
    fn visit_create_space(&mut self, node: &CreateSpaceNode) -> Self::Result;
    fn visit_drop_space(&mut self, node: &DropSpaceNode) -> Self::Result;
    fn visit_desc_space(&mut self, node: &DescSpaceNode) -> Self::Result;
    fn visit_show_spaces(&mut self, node: &ShowSpacesNode) -> Self::Result;
    fn visit_create_tag(&mut self, node: &CreateTagNode) -> Self::Result;
    fn visit_alter_tag(&mut self, node: &crate::query::planner::plan::core::nodes::AlterTagNode) -> Self::Result;
    fn visit_desc_tag(&mut self, node: &DescTagNode) -> Self::Result;
    fn visit_drop_tag(&mut self, node: &DropTagNode) -> Self::Result;
    fn visit_show_tags(&mut self, node: &ShowTagsNode) -> Self::Result;
    fn visit_create_edge(&mut self, node: &CreateEdgeNode) -> Self::Result;
    fn visit_alter_edge(&mut self, node: &crate::query::planner::plan::core::nodes::AlterEdgeNode) -> Self::Result;
    fn visit_desc_edge(&mut self, node: &DescEdgeNode) -> Self::Result;
    fn visit_drop_edge(&mut self, node: &DropEdgeNode) -> Self::Result;
    fn visit_show_edges(&mut self, node: &ShowEdgesNode) -> Self::Result;
    fn visit_create_tag_index(&mut self, node: &CreateTagIndexNode) -> Self::Result;
    fn visit_drop_tag_index(&mut self, node: &DropTagIndexNode) -> Self::Result;
    fn visit_desc_tag_index(&mut self, node: &DescTagIndexNode) -> Self::Result;
    fn visit_show_tag_indexes(&mut self, node: &ShowTagIndexesNode) -> Self::Result;
    fn visit_create_edge_index(&mut self, node: &CreateEdgeIndexNode) -> Self::Result;
    fn visit_drop_edge_index(&mut self, node: &DropEdgeIndexNode) -> Self::Result;
    fn visit_desc_edge_index(&mut self, node: &crate::query::planner::plan::core::nodes::DescEdgeIndexNode) -> Self::Result;
    fn visit_show_edge_indexes(&mut self, node: &ShowEdgeIndexesNode) -> Self::Result;
    fn visit_rebuild_tag_index(&mut self, node: &RebuildTagIndexNode) -> Self::Result;
    fn visit_rebuild_edge_index(&mut self, node: &RebuildEdgeIndexNode) -> Self::Result;
    fn visit_create_user(&mut self, node: &CreateUserNode) -> Self::Result;
    fn visit_alter_user(&mut self, node: &AlterUserNode) -> Self::Result;
    fn visit_drop_user(&mut self, node: &DropUserNode) -> Self::Result;
    fn visit_change_password(&mut self, node: &ChangePasswordNode) -> Self::Result;

    fn visit(&mut self, node: &PlanNodeEnum) -> Self::Result {
        match node {
            PlanNodeEnum::Start(n) => self.visit_start(n),
            PlanNodeEnum::Project(n) => self.visit_project(n),
            PlanNodeEnum::Sort(n) => self.visit_sort(n),
            PlanNodeEnum::Limit(n) => self.visit_limit(n),
            PlanNodeEnum::TopN(n) => self.visit_top_n(n),
            PlanNodeEnum::Sample(n) => self.visit_sample(n),
            PlanNodeEnum::InnerJoin(n) => self.visit_inner_join(n),
            PlanNodeEnum::LeftJoin(n) => self.visit_left_join(n),
            PlanNodeEnum::CrossJoin(n) => self.visit_cross_join(n),
            PlanNodeEnum::GetVertices(n) => self.visit_get_vertices(n),
            PlanNodeEnum::GetEdges(n) => self.visit_get_edges(n),
            PlanNodeEnum::GetNeighbors(n) => self.visit_get_neighbors(n),
            PlanNodeEnum::ScanVertices(n) => self.visit_scan_vertices(n),
            PlanNodeEnum::ScanEdges(n) => self.visit_scan_edges(n),
            PlanNodeEnum::EdgeIndexScan(n) => self.visit_edge_index_scan(n),
            PlanNodeEnum::HashInnerJoin(n) => self.visit_hash_inner_join(n),
            PlanNodeEnum::HashLeftJoin(n) => self.visit_hash_left_join(n),
            PlanNodeEnum::IndexScan(n) => self.visit_index_scan(n),
            PlanNodeEnum::FulltextIndexScan(n) => self.visit_fulltext_index_scan(n),
            PlanNodeEnum::Expand(n) => self.visit_expand(n),
            PlanNodeEnum::ExpandAll(n) => self.visit_expand_all(n),
            PlanNodeEnum::Traverse(n) => self.visit_traverse(n),
            PlanNodeEnum::AppendVertices(n) => self.visit_append_vertices(n),
            PlanNodeEnum::Filter(n) => self.visit_filter(n),
            PlanNodeEnum::Aggregate(n) => self.visit_aggregate(n),
            PlanNodeEnum::Argument(n) => self.visit_argument(n),
            PlanNodeEnum::Loop(n) => self.visit_loop(n),
            PlanNodeEnum::PassThrough(n) => self.visit_pass_through(n),
            PlanNodeEnum::Select(n) => self.visit_select(n),
            PlanNodeEnum::DataCollect(n) => self.visit_data_collect(n),
            PlanNodeEnum::Dedup(n) => self.visit_dedup(n),
            PlanNodeEnum::PatternApply(n) => self.visit_pattern_apply(n),
            PlanNodeEnum::RollUpApply(n) => self.visit_roll_up_apply(n),
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
            PlanNodeEnum::DescTagIndex(n) => self.visit_desc_tag_index(n),
            PlanNodeEnum::ShowTagIndexes(n) => self.visit_show_tag_indexes(n),
            PlanNodeEnum::CreateEdgeIndex(n) => self.visit_create_edge_index(n),
            PlanNodeEnum::DropEdgeIndex(n) => self.visit_drop_edge_index(n),
            PlanNodeEnum::DescEdgeIndex(n) => self.visit_desc_edge_index(n),
            PlanNodeEnum::ShowEdgeIndexes(n) => self.visit_show_edge_indexes(n),
            PlanNodeEnum::RebuildTagIndex(n) => self.visit_rebuild_tag_index(n),
            PlanNodeEnum::RebuildEdgeIndex(n) => self.visit_rebuild_edge_index(n),
            PlanNodeEnum::CreateUser(n) => self.visit_create_user(n),
            PlanNodeEnum::AlterUser(n) => self.visit_alter_user(n),
            PlanNodeEnum::DropUser(n) => self.visit_drop_user(n),
            PlanNodeEnum::ChangePassword(n) => self.visit_change_password(n),
        }
    }
}

pub trait PlanNodeVisitable {
    fn accept<V: PlanNodeVisitor>(&self, visitor: &mut V) -> V::Result;
}

impl PlanNodeVisitable for PlanNodeEnum {
    fn accept<V: PlanNodeVisitor>(&self, visitor: &mut V) -> V::Result {
        visitor.visit(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    struct CountingVisitor {
        count: usize,
    }
    
    impl PlanNodeVisitor for CountingVisitor {
        type Result = usize;
        
        fn visit_start(&mut self, _node: &StartNode) -> Self::Result {
            self.count += 1;
            self.count
        }
        
        fn visit_project(&mut self, _node: &ProjectNode) -> Self::Result {
            self.count += 1;
            self.count
        }
        
        fn visit_sort(&mut self, _node: &SortNode) -> Self::Result { self.count += 1; self.count }
        fn visit_limit(&mut self, _node: &LimitNode) -> Self::Result { self.count += 1; self.count }
        fn visit_top_n(&mut self, _node: &TopNNode) -> Self::Result { self.count += 1; self.count }
        fn visit_sample(&mut self, _node: &SampleNode) -> Self::Result { self.count += 1; self.count }
        fn visit_inner_join(&mut self, _node: &InnerJoinNode) -> Self::Result { self.count += 1; self.count }
        fn visit_left_join(&mut self, _node: &LeftJoinNode) -> Self::Result { self.count += 1; self.count }
        fn visit_cross_join(&mut self, _node: &CrossJoinNode) -> Self::Result { self.count += 1; self.count }
        fn visit_get_vertices(&mut self, _node: &GetVerticesNode) -> Self::Result { self.count += 1; self.count }
        fn visit_get_edges(&mut self, _node: &GetEdgesNode) -> Self::Result { self.count += 1; self.count }
        fn visit_get_neighbors(&mut self, _node: &GetNeighborsNode) -> Self::Result { self.count += 1; self.count }
        fn visit_scan_vertices(&mut self, _node: &ScanVerticesNode) -> Self::Result { self.count += 1; self.count }
        fn visit_scan_edges(&mut self, _node: &ScanEdgesNode) -> Self::Result { self.count += 1; self.count }
        fn visit_hash_inner_join(&mut self, _node: &HashInnerJoinNode) -> Self::Result { self.count += 1; self.count }
        fn visit_hash_left_join(&mut self, _node: &HashLeftJoinNode) -> Self::Result { self.count += 1; self.count }
        fn visit_index_scan(&mut self, _node: &IndexScan) -> Self::Result { self.count += 1; self.count }
        fn visit_fulltext_index_scan(&mut self, _node: &FulltextIndexScan) -> Self::Result { self.count += 1; self.count }
        fn visit_expand(&mut self, _node: &ExpandNode) -> Self::Result { self.count += 1; self.count }
        fn visit_expand_all(&mut self, _node: &ExpandAllNode) -> Self::Result { self.count += 1; self.count }
        fn visit_traverse(&mut self, _node: &TraverseNode) -> Self::Result { self.count += 1; self.count }
        fn visit_append_vertices(&mut self, _node: &AppendVerticesNode) -> Self::Result { self.count += 1; self.count }
        fn visit_filter(&mut self, _node: &FilterNode) -> Self::Result { self.count += 1; self.count }
        fn visit_aggregate(&mut self, _node: &AggregateNode) -> Self::Result { self.count += 1; self.count }
        fn visit_argument(&mut self, _node: &ArgumentNode) -> Self::Result { self.count += 1; self.count }
        fn visit_loop(&mut self, _node: &LoopNode) -> Self::Result { self.count += 1; self.count }
        fn visit_pass_through(&mut self, _node: &PassThroughNode) -> Self::Result { self.count += 1; self.count }
        fn visit_select(&mut self, _node: &SelectNode) -> Self::Result { self.count += 1; self.count }
        fn visit_data_collect(&mut self, _node: &DataCollectNode) -> Self::Result { self.count += 1; self.count }
        fn visit_dedup(&mut self, _node: &DedupNode) -> Self::Result { self.count += 1; self.count }
        fn visit_pattern_apply(&mut self, _node: &PatternApplyNode) -> Self::Result { self.count += 1; self.count }
        fn visit_roll_up_apply(&mut self, _node: &RollUpApplyNode) -> Self::Result { self.count += 1; self.count }
        fn visit_union(&mut self, _node: &UnionNode) -> Self::Result { self.count += 1; self.count }
        fn visit_unwind(&mut self, _node: &UnwindNode) -> Self::Result { self.count += 1; self.count }
        fn visit_assign(&mut self, _node: &AssignNode) -> Self::Result { self.count += 1; self.count }
        fn visit_multi_shortest_path(&mut self, _node: &MultiShortestPath) -> Self::Result { self.count += 1; self.count }
        fn visit_bfs_shortest(&mut self, _node: &BFSShortest) -> Self::Result { self.count += 1; self.count }
        fn visit_all_paths(&mut self, _node: &AllPaths) -> Self::Result { self.count += 1; self.count }
        fn visit_shortest_path(&mut self, _node: &ShortestPath) -> Self::Result { self.count += 1; self.count }
        fn visit_create_space(&mut self, _node: &CreateSpaceNode) -> Self::Result { self.count += 1; self.count }
        fn visit_drop_space(&mut self, _node: &DropSpaceNode) -> Self::Result { self.count += 1; self.count }
        fn visit_desc_space(&mut self, _node: &DescSpaceNode) -> Self::Result { self.count += 1; self.count }
        fn visit_show_spaces(&mut self, _node: &ShowSpacesNode) -> Self::Result { self.count += 1; self.count }
        fn visit_create_tag(&mut self, _node: &CreateTagNode) -> Self::Result { self.count += 1; self.count }
        fn visit_alter_tag(&mut self, _node: &crate::query::planner::plan::core::nodes::AlterTagNode) -> Self::Result { self.count += 1; self.count }
        fn visit_desc_tag(&mut self, _node: &DescTagNode) -> Self::Result { self.count += 1; self.count }
        fn visit_drop_tag(&mut self, _node: &DropTagNode) -> Self::Result { self.count += 1; self.count }
        fn visit_show_tags(&mut self, _node: &ShowTagsNode) -> Self::Result { self.count += 1; self.count }
        fn visit_create_edge(&mut self, _node: &CreateEdgeNode) -> Self::Result { self.count += 1; self.count }
        fn visit_alter_edge(&mut self, _node: &crate::query::planner::plan::core::nodes::AlterEdgeNode) -> Self::Result { self.count += 1; self.count }
        fn visit_desc_edge(&mut self, _node: &DescEdgeNode) -> Self::Result { self.count += 1; self.count }
        fn visit_drop_edge(&mut self, _node: &DropEdgeNode) -> Self::Result { self.count += 1; self.count }
        fn visit_show_edges(&mut self, _node: &ShowEdgesNode) -> Self::Result { self.count += 1; self.count }
        fn visit_create_tag_index(&mut self, _node: &CreateTagIndexNode) -> Self::Result { self.count += 1; self.count }
        fn visit_drop_tag_index(&mut self, _node: &DropTagIndexNode) -> Self::Result { self.count += 1; self.count }
        fn visit_desc_tag_index(&mut self, _node: &DescTagIndexNode) -> Self::Result { self.count += 1; self.count }
        fn visit_show_tag_indexes(&mut self, _node: &ShowTagIndexesNode) -> Self::Result { self.count += 1; self.count }
        fn visit_create_edge_index(&mut self, _node: &CreateEdgeIndexNode) -> Self::Result { self.count += 1; self.count }
        fn visit_drop_edge_index(&mut self, _node: &DropEdgeIndexNode) -> Self::Result { self.count += 1; self.count }
        fn visit_desc_edge_index(&mut self, _node: &crate::query::planner::plan::core::nodes::DescEdgeIndexNode) -> Self::Result { self.count += 1; self.count }
        fn visit_show_edge_indexes(&mut self, _node: &ShowEdgeIndexesNode) -> Self::Result { self.count += 1; self.count }
        fn visit_rebuild_tag_index(&mut self, _node: &RebuildTagIndexNode) -> Self::Result { self.count += 1; self.count }
        fn visit_rebuild_edge_index(&mut self, _node: &RebuildEdgeIndexNode) -> Self::Result { self.count += 1; self.count }
        fn visit_create_user(&mut self, _node: &CreateUserNode) -> Self::Result { self.count += 1; self.count }
        fn visit_alter_user(&mut self, _node: &AlterUserNode) -> Self::Result { self.count += 1; self.count }
        fn visit_drop_user(&mut self, _node: &DropUserNode) -> Self::Result { self.count += 1; self.count }
        fn visit_change_password(&mut self, _node: &ChangePasswordNode) -> Self::Result { self.count += 1; self.count }
    }
    
    #[test]
    fn test_plan_node_visitor() {
        let mut visitor = CountingVisitor { count: 0 };
        let _result = visitor.visit(&PlanNodeEnum::Start(StartNode::new()));
        assert_eq!(visitor.count, 1);
    }
}
