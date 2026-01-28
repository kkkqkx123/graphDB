//! 转换规则
//! 这些规则负责将计划节点转换为等效但更高效的节点

use super::engine::OptimizerError;
use super::plan::{OptContext, OptGroupNode, OptRule, Pattern};
use super::rule_patterns::PatternBuilder;
use super::rule_traits::BaseOptRule;
use crate::query::planner::plan::core::nodes::PlanNodeEnum;
use crate::query::visitor::PlanNodeVisitor;

/// 转换Limit-Sort为TopN的规则
#[derive(Debug)]
pub struct TopNRule;

impl OptRule for TopNRule {
    fn name(&self) -> &str {
        "TopNRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        let mut visitor = TopNRuleVisitor {
            ctx: ctx as *const OptContext,
            converted: false,
            new_node: None,
        };

        let result = visitor.visit(&node.plan_node);
        if result.converted {
            Ok(result.new_node)
        } else {
            Ok(None)
        }
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::with_dependency("Limit", "Sort")
    }
}

impl BaseOptRule for TopNRule {}

/// TopN 规则访问者
struct TopNRuleVisitor {
    converted: bool,
    new_node: Option<OptGroupNode>,
    ctx: *const OptContext,
}

impl TopNRuleVisitor {
    fn get_ctx(&self) -> &OptContext {
        unsafe { &*self.ctx }
    }
}

impl PlanNodeVisitor for TopNRuleVisitor {
    type Result = Self;

    fn visit_default(&mut self) -> Self::Result {
        self.clone()
    }

    fn visit_limit(&mut self, node: &crate::query::planner::plan::core::nodes::LimitNode) -> Self::Result {
        if self.converted {
            return self.clone();
        }

        if node.dependencies().is_empty() {
            return self.clone();
        }

        let child_dep_id = node.dependencies()[0].id() as usize;
        if let Some(child_node) = self.get_ctx().find_group_node_by_plan_node_id(child_dep_id) {
            if child_node.plan_node.is_sort() {
                if let Some(sort_plan_node) = child_node.plan_node.as_sort() {
                    let mut new_node = child_node.clone();
                    let sort_input = sort_plan_node.dependencies()[0].as_ref().clone();

                    let mut topn_node =
                        crate::query::planner::plan::core::nodes::TopNNode::new(
                            sort_input,
                            sort_plan_node.sort_items().to_vec(),
                            node.count(),
                        )
                        .expect("TopN node should be created successfully");

                    if let Some(output_var) = node.output_var() {
                        topn_node.set_output_var(output_var.clone());
                    }

                    new_node.plan_node = PlanNodeEnum::TopN(topn_node);

                    if !child_node.dependencies.is_empty() {
                        new_node.dependencies = child_node.dependencies.clone();
                    } else {
                        new_node.dependencies = vec![];
                    }

                    self.converted = true;
                    self.new_node = Some(new_node);
                }
            }
        }

        self.clone()
    }

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
            _ => self.visit_default(),
        }
    }

    fn visit_start(&mut self, _node: &crate::query::planner::plan::core::nodes::StartNode) -> Self::Result {
        self.clone()
    }

    fn visit_filter(&mut self, _node: &crate::query::planner::plan::core::nodes::FilterNode) -> Self::Result {
        self.clone()
    }

    fn visit_project(&mut self, _node: &crate::query::planner::plan::core::nodes::ProjectNode) -> Self::Result {
        self.clone()
    }

    fn visit_sort(&mut self, _node: &crate::query::planner::plan::core::nodes::SortNode) -> Self::Result {
        self.clone()
    }

    fn visit_topn(&mut self, _node: &crate::query::planner::plan::core::nodes::TopNNode) -> Self::Result {
        self.clone()
    }

    fn visit_sample(&mut self, _node: &crate::query::planner::plan::core::nodes::SampleNode) -> Self::Result {
        self.clone()
    }

    fn visit_dedup(&mut self, _node: &crate::query::planner::plan::core::nodes::DedupNode) -> Self::Result {
        self.clone()
    }

    fn visit_get_vertices(&mut self, _node: &crate::query::planner::plan::core::nodes::GetVerticesNode) -> Self::Result {
        self.clone()
    }

    fn visit_get_edges(&mut self, _node: &crate::query::planner::plan::core::nodes::GetEdgesNode) -> Self::Result {
        self.clone()
    }

    fn visit_get_neighbors(&mut self, _node: &crate::query::planner::plan::core::nodes::GetNeighborsNode) -> Self::Result {
        self.clone()
    }

    fn visit_scan_vertices(&mut self, _node: &crate::query::planner::plan::core::nodes::ScanVerticesNode) -> Self::Result {
        self.clone()
    }

    fn visit_scan_edges(&mut self, _node: &crate::query::planner::plan::core::nodes::ScanEdgesNode) -> Self::Result {
        self.clone()
    }

    fn visit_index_scan(&mut self, _node: &crate::query::planner::plan::algorithms::IndexScan) -> Self::Result {
        self.clone()
    }

    fn visit_fulltext_index_scan(&mut self, _node: &crate::query::planner::plan::algorithms::FulltextIndexScan) -> Self::Result {
        self.clone()
    }

    fn visit_expand(&mut self, _node: &crate::query::planner::plan::core::nodes::ExpandNode) -> Self::Result {
        self.clone()
    }

    fn visit_expand_all(&mut self, _node: &crate::query::planner::plan::core::nodes::ExpandAllNode) -> Self::Result {
        self.clone()
    }

    fn visit_traverse(&mut self, _node: &crate::query::planner::plan::core::nodes::TraverseNode) -> Self::Result {
        self.clone()
    }

    fn visit_append_vertices(&mut self, _node: &crate::query::planner::plan::core::nodes::AppendVerticesNode) -> Self::Result {
        self.clone()
    }

    fn visit_inner_join(&mut self, _node: &crate::query::planner::plan::core::nodes::InnerJoinNode) -> Self::Result {
        self.clone()
    }

    fn visit_left_join(&mut self, _node: &crate::query::planner::plan::core::nodes::LeftJoinNode) -> Self::Result {
        self.clone()
    }

    fn visit_cross_join(&mut self, _node: &crate::query::planner::plan::core::nodes::CrossJoinNode) -> Self::Result {
        self.clone()
    }

    fn visit_hash_inner_join(&mut self, _node: &crate::query::planner::plan::core::nodes::HashInnerJoinNode) -> Self::Result {
        self.clone()
    }

    fn visit_hash_left_join(&mut self, _node: &crate::query::planner::plan::core::nodes::HashLeftJoinNode) -> Self::Result {
        self.clone()
    }

    fn visit_aggregate(&mut self, _node: &crate::query::planner::plan::core::nodes::AggregateNode) -> Self::Result {
        self.clone()
    }

    fn visit_argument(&mut self, _node: &crate::query::planner::plan::core::nodes::ArgumentNode) -> Self::Result {
        self.clone()
    }

    fn visit_loop(&mut self, _node: &crate::query::planner::plan::core::nodes::LoopNode) -> Self::Result {
        self.clone()
    }

    fn visit_pass_through(&mut self, _node: &crate::query::planner::plan::core::nodes::PassThroughNode) -> Self::Result {
        self.clone()
    }

    fn visit_select(&mut self, _node: &crate::query::planner::plan::core::nodes::SelectNode) -> Self::Result {
        self.clone()
    }

    fn visit_data_collect(&mut self, _node: &crate::query::planner::plan::core::nodes::DataCollectNode) -> Self::Result {
        self.clone()
    }

    fn visit_pattern_apply(&mut self, _node: &crate::query::planner::plan::core::nodes::PatternApplyNode) -> Self::Result {
        self.clone()
    }

    fn visit_rollup_apply(&mut self, _node: &crate::query::planner::plan::core::nodes::RollUpApplyNode) -> Self::Result {
        self.clone()
    }

    fn visit_union(&mut self, _node: &crate::query::planner::plan::core::nodes::UnionNode) -> Self::Result {
        self.clone()
    }

    fn visit_unwind(&mut self, _node: &crate::query::planner::plan::core::nodes::UnwindNode) -> Self::Result {
        self.clone()
    }

    fn visit_assign(&mut self, _node: &crate::query::planner::plan::core::nodes::AssignNode) -> Self::Result {
        self.clone()
    }

    fn visit_multi_shortest_path(&mut self, _node: &crate::query::planner::plan::algorithms::MultiShortestPath) -> Self::Result {
        self.clone()
    }

    fn visit_bfs_shortest(&mut self, _node: &crate::query::planner::plan::algorithms::BFSShortest) -> Self::Result {
        self.clone()
    }

    fn visit_all_paths(&mut self, _node: &crate::query::planner::plan::algorithms::AllPaths) -> Self::Result {
        self.clone()
    }

    fn visit_shortest_path(&mut self, _node: &crate::query::planner::plan::algorithms::ShortestPath) -> Self::Result {
        self.clone()
    }

    fn visit_create_space(&mut self, _node: &crate::query::planner::plan::core::nodes::CreateSpaceNode) -> Self::Result {
        self.clone()
    }

    fn visit_drop_space(&mut self, _node: &crate::query::planner::plan::core::nodes::DropSpaceNode) -> Self::Result {
        self.clone()
    }

    fn visit_desc_space(&mut self, _node: &crate::query::planner::plan::core::nodes::DescSpaceNode) -> Self::Result {
        self.clone()
    }

    fn visit_show_spaces(&mut self, _node: &crate::query::planner::plan::core::nodes::ShowSpacesNode) -> Self::Result {
        self.clone()
    }

    fn visit_create_tag(&mut self, _node: &crate::query::planner::plan::core::nodes::CreateTagNode) -> Self::Result {
        self.clone()
    }

    fn visit_alter_tag(&mut self, _node: &crate::query::planner::plan::core::nodes::AlterTagNode) -> Self::Result {
        self.clone()
    }

    fn visit_desc_tag(&mut self, _node: &crate::query::planner::plan::core::nodes::DescTagNode) -> Self::Result {
        self.clone()
    }

    fn visit_drop_tag(&mut self, _node: &crate::query::planner::plan::core::nodes::DropTagNode) -> Self::Result {
        self.clone()
    }

    fn visit_show_tags(&mut self, _node: &crate::query::planner::plan::core::nodes::ShowTagsNode) -> Self::Result {
        self.clone()
    }

    fn visit_create_edge(&mut self, _node: &crate::query::planner::plan::core::nodes::CreateEdgeNode) -> Self::Result {
        self.clone()
    }

    fn visit_alter_edge(&mut self, _node: &crate::query::planner::plan::core::nodes::AlterEdgeNode) -> Self::Result {
        self.clone()
    }

    fn visit_desc_edge(&mut self, _node: &crate::query::planner::plan::core::nodes::DescEdgeNode) -> Self::Result {
        self.clone()
    }

    fn visit_drop_edge(&mut self, _node: &crate::query::planner::plan::core::nodes::DropEdgeNode) -> Self::Result {
        self.clone()
    }

    fn visit_show_edges(&mut self, _node: &crate::query::planner::plan::core::nodes::ShowEdgesNode) -> Self::Result {
        self.clone()
    }

    fn visit_create_tag_index(&mut self, _node: &crate::query::planner::plan::core::nodes::CreateTagIndexNode) -> Self::Result {
        self.clone()
    }

    fn visit_drop_tag_index(&mut self, _node: &crate::query::planner::plan::core::nodes::DropTagIndexNode) -> Self::Result {
        self.clone()
    }

    fn visit_desc_tag_index(&mut self, _node: &crate::query::planner::plan::core::nodes::DescTagIndexNode) -> Self::Result {
        self.clone()
    }

    fn visit_show_tag_indexes(&mut self, _node: &crate::query::planner::plan::core::nodes::ShowTagIndexesNode) -> Self::Result {
        self.clone()
    }

    fn visit_create_edge_index(&mut self, _node: &crate::query::planner::plan::core::nodes::CreateEdgeIndexNode) -> Self::Result {
        self.clone()
    }

    fn visit_drop_edge_index(&mut self, _node: &crate::query::planner::plan::core::nodes::DropEdgeIndexNode) -> Self::Result {
        self.clone()
    }

    fn visit_desc_edge_index(&mut self, _node: &crate::query::planner::plan::core::nodes::DescEdgeIndexNode) -> Self::Result {
        self.clone()
    }

    fn visit_show_edge_indexes(&mut self, _node: &crate::query::planner::plan::core::nodes::ShowEdgeIndexesNode) -> Self::Result {
        self.clone()
    }

    fn visit_rebuild_tag_index(&mut self, _node: &crate::query::planner::plan::core::nodes::RebuildTagIndexNode) -> Self::Result {
        self.clone()
    }

    fn visit_rebuild_edge_index(&mut self, _node: &crate::query::planner::plan::core::nodes::RebuildEdgeIndexNode) -> Self::Result {
        self.clone()
    }
}

impl Clone for TopNRuleVisitor {
    fn clone(&self) -> Self {
        Self {
            converted: self.converted,
            new_node: self.new_node.clone(),
            ctx: self.ctx,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::context::execution::QueryContext;
    use crate::query::optimizer::plan::{OptContext, OptGroupNode};
    use crate::query::planner::plan::core::nodes::SortNode;

    fn create_test_context() -> OptContext {
        let _session_info = crate::api::session::session_manager::SessionInfo {
            session_id: 1,
            user_name: "test_user".to_string(),
            space_name: None,
            graph_addr: None,
            create_time: std::time::SystemTime::now(),
            last_access_time: std::time::SystemTime::now(),
            active_queries: 0,
            timezone: None,
        };
        let query_context = QueryContext::new();
        OptContext::new(query_context)
    }

    #[test]
    fn test_top_n_rule() {
        let rule = TopNRule;
        let mut ctx = create_test_context();

        // 创建一个Sort节点
        let sort_node = PlanNodeEnum::Sort(
            SortNode::new(
                PlanNodeEnum::Start(crate::query::planner::plan::core::nodes::StartNode::new()),
                vec![],
            )
            .expect("Sort node should be created successfully"),
        );
        let opt_node = OptGroupNode::new(1, sort_node);

        let result = rule
            .apply(&mut ctx, &opt_node)
            .expect("Rule should apply successfully");
        assert!(result.is_none());
    }
}
