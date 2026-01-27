//! 谓词重排序优化规则
//! 重新排列谓词顺序以优化查询性能

use super::engine::OptimizerError;
use super::plan::{OptContext, OptGroupNode, OptRule, Pattern};
use super::rule_patterns::PatternBuilder;
use super::rule_traits::BaseOptRule;
use crate::query::planner::plan::core::nodes::PlanNodeEnum;
use crate::query::visitor::PlanNodeVisitor;

/// 谓词重排序规则
///
/// 将过滤条件按照选择性从高到低排序，尽早过滤掉更多数据。
#[derive(Debug)]
pub struct PredicateReorderRule;

impl OptRule for PredicateReorderRule {
    fn name(&self) -> &str {
        "PredicateReorderRule"
    }

    fn apply(
        &self,
        _ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        let mut visitor = PredicateReorderVisitor {
            reordered: false,
            new_node: None,
        };

        let result = visitor.visit(&node.plan_node);
        if result.reordered {
            Ok(result.new_node)
        } else {
            Ok(None)
        }
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::filter()
    }
}

impl BaseOptRule for PredicateReorderRule {}

/// 谓词重排序访问者
struct PredicateReorderVisitor {
    reordered: bool,
    new_node: Option<OptGroupNode>,
}

impl PlanNodeVisitor for PredicateReorderVisitor {
    type Result = Self;

    fn visit_filter(&mut self, node: &crate::query::planner::plan::core::nodes::FilterNode) -> Self::Result {
        use crate::core::Expression;

        let condition = node.condition();

        if let Expression::Binary { op: crate::core::types::operators::BinaryOperator::And, left, right } = condition {
            let left_selectivity = Self::estimate_selectivity(left);
            let right_selectivity = Self::estimate_selectivity(right);

            if left_selectivity < right_selectivity {
                let reordered_condition = Expression::Binary {
                    op: crate::core::types::operators::BinaryOperator::And,
                    left: right.clone(),
                    right: left.clone(),
                };

                let mut new_node = node.clone();
                new_node.set_condition(reordered_condition);

                let mut opt_node = OptGroupNode::new(node.id() as usize, PlanNodeEnum::Filter(new_node));
                opt_node.dependencies = node.dependencies().iter().map(|d| d.id() as usize).collect();

                self.reordered = true;
                self.new_node = Some(opt_node);
            }
        }

        self.clone()
    }

    fn visit(&mut self, node: &PlanNodeEnum) -> Self::Result {
        PlanNodeVisitor::visit(self, node)
    }

    fn visit_start(&mut self, _node: &crate::query::planner::plan::core::nodes::StartNode) -> Self::Result {
        self.clone()
    }

    fn visit_project(&mut self, _node: &crate::query::planner::plan::core::nodes::ProjectNode) -> Self::Result {
        self.clone()
    }

    fn visit_sort(&mut self, _node: &crate::query::planner::plan::core::nodes::SortNode) -> Self::Result {
        self.clone()
    }

    fn visit_limit(&mut self, _node: &crate::query::planner::plan::core::nodes::LimitNode) -> Self::Result {
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

impl Clone for PredicateReorderVisitor {
    fn clone(&self) -> Self {
        Self {
            reordered: self.reordered,
            new_node: self.new_node.clone(),
        }
    }
}

impl PredicateReorderVisitor {
    fn estimate_selectivity(expr: &crate::core::Expression) -> f64 {
        use crate::core::{Expression, types::operators::BinaryOperator};

        match expr {
            Expression::Binary { op, left, right } => {
                match op {
                    BinaryOperator::Equal => {
                        let left_is_literal = matches!(left.as_ref(), Expression::Literal(_));
                        let right_is_literal = matches!(right.as_ref(), Expression::Literal(_));
                        if left_is_literal || right_is_literal {
                            0.01
                        } else {
                            0.1
                        }
                    }
                    BinaryOperator::NotEqual => 0.9,
                    BinaryOperator::LessThan | BinaryOperator::LessThanOrEqual | 
                    BinaryOperator::GreaterThan | BinaryOperator::GreaterThanOrEqual => 0.33,
                    BinaryOperator::And => {
                        Self::estimate_selectivity(left) * Self::estimate_selectivity(right)
                    }
                    BinaryOperator::Or => {
                        1.0 - (1.0 - Self::estimate_selectivity(left)) * (1.0 - Self::estimate_selectivity(right))
                    }
                    _ => 0.5,
                }
            }
            Expression::Function { name, .. } => {
                match name.to_lowercase().as_str() {
                    "id" => 0.01,
                    "exists" => 0.5,
                    _ => 0.1,
                }
            }
            _ => 0.5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::context::execution::QueryContext;
    use crate::query::optimizer::plan::{OptContext, OptGroupNode};
    use crate::query::planner::plan::core::nodes::{FilterNode, StartNode};
    use crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode;

    fn create_test_context() -> OptContext {
        let query_context = QueryContext::new();
        OptContext::new(query_context)
    }

    #[test]
    fn test_predicate_reorder_rule() {
        let rule = PredicateReorderRule;
        let mut ctx = create_test_context();

        let filter_node = FilterNode::new(
            PlanNodeEnum::Start(StartNode::new()),
            crate::core::Expression::Binary {
                op: crate::core::BinaryOperator::And,
                left: Box::new(crate::core::Expression::Literal(crate::core::Value::String("value1".to_string()))),
                right: Box::new(crate::core::Expression::Literal(crate::core::Value::String("value2".to_string()))),
            },
        )
        .expect("Filter node should be created successfully");
        let opt_node = OptGroupNode::new(1, filter_node.into_enum());

        let result = rule
            .apply(&mut ctx, &opt_node)
            .expect("Rule should apply successfully");
        assert!(result.is_some());
    }
}
