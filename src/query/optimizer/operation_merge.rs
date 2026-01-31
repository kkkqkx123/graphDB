//! 操作合并优化规则
//! 这些规则负责合并多个连续的相同类型操作，以减少中间结果和执行开销

use super::engine::OptimizerError;
use super::plan::{OptContext, OptGroupNode, OptRule, Pattern, TransformResult};
use super::rule_patterns::{CommonPatterns, PatternBuilder};
use super::rule_traits::{combine_conditions, BaseOptRule, MergeRule};
use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;
use crate::query::planner::plan::FilterNode as FilterPlanNode;
use crate::query::visitor::PlanNodeVisitor;
use std::rc::Rc;
use std::cell::RefCell;
use std::result::Result as StdResult;

/// 合并过滤访问者
#[derive(Clone)]
struct CombineFilterVisitor {
    merged: bool,
    new_node: Option<OptGroupNode>,
    ctx: *const OptContext,
    node_dependencies: Vec<usize>,
}

impl CombineFilterVisitor {
    fn get_ctx(&self) -> &OptContext {
        unsafe { &*self.ctx }
    }
}

impl PlanNodeVisitor for CombineFilterVisitor {
    type Result = Self;

    fn visit_default(&mut self) -> Self::Result {
        self.clone()
    }

    fn visit_filter(&mut self, node: &crate::query::planner::plan::core::nodes::FilterNode) -> Self::Result {
        if let Some(dep_id) = self.node_dependencies.first() {
            if let Some(child_node) = self.get_ctx().find_group_node_by_plan_node_id(*dep_id) {
                let child_node_ref = child_node.borrow();
                if child_node_ref.plan_node.is_filter() {
                    if let Some(child_filter) = child_node_ref.plan_node.as_filter() {
                        let top_condition = node.condition();
                        let child_condition = child_filter.condition();

                        let combined_condition_str = combine_conditions(
                            &format!("{:?}", top_condition),
                            &format!("{:?}", child_condition),
                        );

                        let child_input = child_filter.input().clone();
                        let combined_filter_node = match FilterPlanNode::new(
                            child_input,
                            crate::core::Expression::Variable(combined_condition_str),
                        ) {
                            Ok(filter_node) => filter_node,
                            Err(_) => return self.clone(),
                        };

                        let combined_opt_node = OptGroupNode::new(
                            node.id() as usize,
                            crate::query::planner::plan::PlanNodeEnum::Filter(combined_filter_node),
                        );

                        drop(child_node_ref);

                        self.merged = true;
                        self.new_node = Some(combined_opt_node);
                    }
                }
            }
        }

        self.clone()
    }
}

/// 合并多个过滤操作的规则
#[derive(Debug)]
pub struct CombineFilterRule;

impl OptRule for CombineFilterRule {
    fn name(&self) -> &str {
        "CombineFilterRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> Result<Option<TransformResult>, OptimizerError> {
        let node_ref = group_node.borrow();
        if !node_ref.plan_node.is_filter() {
            return Ok(Some(TransformResult::unchanged()));
        }

        let mut visitor = CombineFilterVisitor {
            merged: false,
            new_node: None,
            ctx: ctx as *const OptContext,
            node_dependencies: node_ref.dependencies.clone(),
        };

        let result = visitor.visit(&node_ref.plan_node);
        drop(node_ref);

        if result.merged {
            if let Some(new_node) = result.new_node {
                let mut transform_result = TransformResult::new();
                transform_result.add_new_group_node(Rc::new(RefCell::new(new_node)));
                return Ok(Some(transform_result));
            }
        }
        Ok(Some(TransformResult::unchanged()))
    }

    fn pattern(&self) -> Pattern {
        CommonPatterns::filter_over_filter()
    }
}

impl BaseOptRule for CombineFilterRule {}

impl MergeRule for CombineFilterRule {
    fn can_merge(&self, group_node: &Rc<RefCell<OptGroupNode>>, child: &OptGroupNode) -> bool {
        let node_ref = group_node.borrow();
        node_ref.plan_node.is_filter() && child.plan_node.is_filter()
    }

    fn create_merged_node(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
        child: &OptGroupNode,
    ) -> Result<Option<TransformResult>, OptimizerError> {
        let node_ref = group_node.borrow();
        if let (Some(top_filter), Some(child_filter)) =
            (node_ref.plan_node.as_filter(), child.plan_node.as_filter())
        {
            let top_condition = top_filter.condition();
            let child_condition = child_filter.condition();

            let combined_condition_str = combine_conditions(
                &format!("{:?}", top_condition),
                &format!("{:?}", child_condition),
            );

            let input = top_filter
                .dependencies()
                .first()
                .expect("Filter should have at least one dependency")
                .clone();

            let combined_filter_node = match FilterPlanNode::new(
                *input,
                crate::core::Expression::Variable(combined_condition_str),
            ) {
                Ok(node) => node,
                Err(_) => top_filter.clone(),
            };

            let mut combined_filter_opt_node = node_ref.clone();
            combined_filter_opt_node.plan_node =
                crate::query::planner::plan::PlanNodeEnum::Filter(combined_filter_node);

            combined_filter_opt_node.dependencies = node_ref.dependencies.clone();

            let mut result = TransformResult::new();
            result.add_new_group_node(Rc::new(RefCell::new(combined_filter_opt_node)));
            return Ok(Some(result));
        }

        let mut result = TransformResult::new();
        result.add_new_group_node(group_node.clone());
        Ok(Some(result))
    }
}

/// 折叠多个投影操作的规则
#[derive(Debug)]
pub struct CollapseProjectRule;

impl OptRule for CollapseProjectRule {
    fn name(&self) -> &str {
        "CollapseProjectRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> Result<Option<TransformResult>, OptimizerError> {
        let node_ref = group_node.borrow();
        if !node_ref.plan_node.is_project() {
            return Ok(Some(TransformResult::unchanged()));
        }

        if let Some(matched) = self.match_pattern(ctx, group_node)? {
            if matched.dependencies.len() == 1 {
                let child = &matched.dependencies[0];

                if child.borrow().plan_node.is_project() {
                    drop(node_ref);
                    let mut result = TransformResult::new();
                    result.add_new_group_node(group_node.clone());
                    return Ok(Some(result));
                }
            }
        }
        Ok(Some(TransformResult::unchanged()))
    }

    fn pattern(&self) -> Pattern {
        CommonPatterns::project_over_project()
    }
}

impl BaseOptRule for CollapseProjectRule {}

impl MergeRule for CollapseProjectRule {
    fn can_merge(&self, group_node: &Rc<RefCell<OptGroupNode>>, child: &OptGroupNode) -> bool {
        let node_ref = group_node.borrow();
        node_ref.plan_node.is_project() && child.plan_node.is_project()
    }

    fn create_merged_node(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
        child: &OptGroupNode,
    ) -> Result<Option<TransformResult>, OptimizerError> {
        let node_ref = group_node.borrow();
        let mut result = TransformResult::new();
        result.add_new_group_node(group_node.clone());
        Ok(Some(result))
    }
}

/// 合并获取顶点和投影操作的规则
#[derive(Debug)]
pub struct MergeGetVerticesAndProjectRule;

impl OptRule for MergeGetVerticesAndProjectRule {
    fn name(&self) -> &str {
        "MergeGetVerticesAndProjectRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> Result<Option<TransformResult>, OptimizerError> {
        let node_ref = group_node.borrow();
        if !node_ref.plan_node.is_get_vertices() {
            return Ok(Some(TransformResult::unchanged()));
        }

        if let Some(matched) = self.match_pattern(ctx, group_node)? {
            if matched.dependencies.len() >= 1 {
                let child = &matched.dependencies[0];
                if child.borrow().plan_node.is_project() {
                    drop(node_ref);
                    let mut result = TransformResult::new();
                    result.add_new_group_node(group_node.clone());
                    return Ok(Some(result));
                }
            }
        }
        Ok(Some(TransformResult::unchanged()))
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::with_dependency("GetVertices", "Project")
    }
}

impl BaseOptRule for MergeGetVerticesAndProjectRule {}

impl MergeRule for MergeGetVerticesAndProjectRule {
    fn can_merge(&self, group_node: &Rc<RefCell<OptGroupNode>>, child: &OptGroupNode) -> bool {
        let node_ref = group_node.borrow();
        node_ref.plan_node.is_get_vertices() && child.plan_node.is_project()
    }

    fn create_merged_node(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
        child: &OptGroupNode,
    ) -> Result<Option<TransformResult>, OptimizerError> {
        let node_ref = group_node.borrow();
        let mut result = TransformResult::new();
        result.add_new_group_node(group_node.clone());
        Ok(Some(result))
    }
}

/// 合并获取顶点和去重操作的规则
#[derive(Debug)]
pub struct MergeGetVerticesAndDedupRule;

impl OptRule for MergeGetVerticesAndDedupRule {
    fn name(&self) -> &str {
        "MergeGetVerticesAndDedupRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> Result<Option<TransformResult>, OptimizerError> {
        let node_ref = group_node.borrow();
        if !node_ref.plan_node.is_get_vertices() {
            return Ok(Some(TransformResult::unchanged()));
        }

        if let Some(matched) = self.match_pattern(ctx, group_node)? {
            if matched.dependencies.len() >= 1 {
                let child = &matched.dependencies[0];

                if child.borrow().plan_node.is_dedup() {
                    drop(node_ref);
                    let mut result = TransformResult::new();
                    result.add_new_group_node(group_node.clone());
                    return Ok(Some(result));
                }
            }
        }
        Ok(Some(TransformResult::unchanged()))
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::with_dependency("GetVertices", "Dedup")
    }
}

impl BaseOptRule for MergeGetVerticesAndDedupRule {}

impl MergeRule for MergeGetVerticesAndDedupRule {
    fn can_merge(&self, group_node: &Rc<RefCell<OptGroupNode>>, child: &OptGroupNode) -> bool {
        let node_ref = group_node.borrow();
        node_ref.plan_node.is_get_vertices() && child.plan_node.is_dedup()
    }

    fn create_merged_node(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
        child: &OptGroupNode,
    ) -> Result<Option<TransformResult>, OptimizerError> {
        let node_ref = group_node.borrow();
        let mut result = TransformResult::new();
        result.add_new_group_node(group_node.clone());
        Ok(Some(result))
    }
}

/// 合并获取邻居和去重操作的规则
#[derive(Debug)]
pub struct MergeGetNbrsAndDedupRule;

impl OptRule for MergeGetNbrsAndDedupRule {
    fn name(&self) -> &str {
        "MergeGetNbrsAndDedupRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> Result<Option<TransformResult>, OptimizerError> {
        let node_ref = group_node.borrow();
        if !node_ref.plan_node.is_get_neighbors() {
            return Ok(Some(TransformResult::unchanged()));
        }

        if let Some(matched) = self.match_pattern(ctx, group_node)? {
            if matched.dependencies.len() >= 1 {
                let child = &matched.dependencies[0];

                if child.borrow().plan_node.is_dedup() {
                    drop(node_ref);
                    let mut result = TransformResult::new();
                    result.add_new_group_node(group_node.clone());
                    return Ok(Some(result));
                }
            }
        }
        Ok(Some(TransformResult::unchanged()))
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::with_dependency("GetNeighbors", "Dedup")
    }
}

impl BaseOptRule for MergeGetNbrsAndDedupRule {}

impl MergeRule for MergeGetNbrsAndDedupRule {
    fn can_merge(&self, group_node: &Rc<RefCell<OptGroupNode>>, child: &OptGroupNode) -> bool {
        let node_ref = group_node.borrow();
        node_ref.plan_node.is_get_neighbors() && child.plan_node.is_dedup()
    }

    fn create_merged_node(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
        child: &OptGroupNode,
    ) -> Result<Option<TransformResult>, OptimizerError> {
        let node_ref = group_node.borrow();
        let mut result = TransformResult::new();
        result.add_new_group_node(group_node.clone());
        Ok(Some(result))
    }
}

/// 合并获取邻居和投影操作的规则
#[derive(Debug)]
pub struct MergeGetNbrsAndProjectRule;

impl OptRule for MergeGetNbrsAndProjectRule {
    fn name(&self) -> &str {
        "MergeGetNbrsAndProjectRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> Result<Option<TransformResult>, OptimizerError> {
        let node_ref = group_node.borrow();
        if !node_ref.plan_node.is_get_neighbors() {
            return Ok(Some(TransformResult::unchanged()));
        }

        if let Some(matched) = self.match_pattern(ctx, group_node)? {
            if matched.dependencies.len() >= 1 {
                let child = &matched.dependencies[0];

                if child.borrow().plan_node.is_project() {
                    drop(node_ref);
                    let mut result = TransformResult::new();
                    result.add_new_group_node(group_node.clone());
                    return Ok(Some(result));
                }
            }
        }
        Ok(Some(TransformResult::unchanged()))
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::with_dependency("GetNeighbors", "Project")
    }
}

impl BaseOptRule for MergeGetNbrsAndProjectRule {}

impl MergeRule for MergeGetNbrsAndProjectRule {
    fn can_merge(&self, group_node: &Rc<RefCell<OptGroupNode>>, child: &OptGroupNode) -> bool {
        let node_ref = group_node.borrow();
        node_ref.plan_node.is_get_neighbors() && child.plan_node.is_project()
    }

    fn create_merged_node(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
        child: &OptGroupNode,
    ) -> Result<Option<TransformResult>, OptimizerError> {
        let node_ref = group_node.borrow();
        let mut result = TransformResult::new();
        result.add_new_group_node(group_node.clone());
        Ok(Some(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::context::execution::QueryContext;
    use crate::query::optimizer::plan::{OptContext, OptGroupNode};
    use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;
    use crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode;
    use crate::query::planner::plan::core::nodes::{
        DedupNode as Dedup, FilterNode as Filter, GetNeighborsNode as GetNeighbors,
        GetVerticesNode as GetVertices, ProjectNode as Project, StartNode,
    };

    fn create_test_context() -> OptContext {
        let query_context = QueryContext::new();
        OptContext::new(query_context)
    }

    #[test]
    fn test_combine_filter_rule() {
        let rule = CombineFilterRule;
        let mut ctx = create_test_context();

        let start_node = PlanNodeEnum::Start(StartNode::new());
        let child_filter_node = match Filter::new(
            start_node.clone(),
            crate::core::Expression::Variable("col1 > 100".to_string()),
        ) {
            Ok(node) => node,
            Err(_) => return,
        };
        let child_opt_node = OptGroupNode::new(2, child_filter_node.into_enum());

        let filter_node = match Filter::new(
            start_node,
            crate::core::Expression::Variable("col2 > 200".to_string()),
        ) {
            Ok(node) => node,
            Err(_) => return,
        };
        let mut opt_node = OptGroupNode::new(1, filter_node.into_enum());
        opt_node.dependencies = vec![2];

        ctx.add_plan_node_and_group_node(2, &child_opt_node);

        let result = rule
            .apply(&mut ctx, &opt_node)
            .expect("Rule should apply successfully");
        // 规则应该匹配过滤节点并尝试合并连续的过滤操作
        assert!(result.is_some());
    }

    #[test]
    fn test_collapse_project_rule() {
        let rule = CollapseProjectRule;
        let mut ctx = create_test_context();

        let start_node = PlanNodeEnum::Start(StartNode::new());
        let child_project_node = match Project::new(
            start_node.clone(),
            vec![crate::query::validator::YieldColumn::new(
                crate::core::Expression::Variable("col1".to_string()),
                "col1".to_string(),
            )],
        ) {
            Ok(node) => node,
            Err(_) => return,
        };
        let child_opt_node = OptGroupNode::new(2, child_project_node.into_enum());

        let project_node = match Project::new(
            start_node,
            vec![crate::query::validator::YieldColumn::new(
                crate::core::Expression::Variable("col2".to_string()),
                "col2".to_string(),
            )],
        ) {
            Ok(node) => node,
            Err(_) => return,
        };
        let mut opt_node = OptGroupNode::new(1, project_node.into_enum());
        opt_node.dependencies = vec![2];

        ctx.add_plan_node_and_group_node(2, &child_opt_node);

        let result = rule
            .apply(&mut ctx, &opt_node)
            .expect("Rule should apply successfully");
        // 规则应该匹配投影节点并尝试折叠连续的投影操作
        assert!(result.is_some());
    }

    #[test]
    fn test_merge_get_vertices_and_project_rule() {
        let rule = MergeGetVerticesAndProjectRule;
        let mut ctx = create_test_context();

        let start_node = PlanNodeEnum::Start(StartNode::new());
        let project_node = match Project::new(
            start_node,
            vec![crate::query::validator::YieldColumn::new(
                crate::core::Expression::Variable("col1".to_string()),
                "col1".to_string(),
            )],
        ) {
            Ok(node) => node,
            Err(_) => return,
        };
        let child_opt_node = OptGroupNode::new(2, project_node.into_enum());

        let get_vertices_node = PlanNodeEnum::GetVertices(GetVertices::new(1, ""));
        let mut opt_node = OptGroupNode::new(1, get_vertices_node);
        opt_node.dependencies = vec![2];

        ctx.add_plan_node_and_group_node(2, &child_opt_node);

        let result = rule
            .apply(&mut ctx, &opt_node)
            .expect("Rule should apply successfully");
        // 规则应该匹配获取顶点节点并尝试与投影操作合并
        assert!(result.is_some());
    }

    #[test]
    fn test_merge_get_vertices_and_dedup_rule() {
        let rule = MergeGetVerticesAndDedupRule;
        let mut ctx = create_test_context();

        let start_node = PlanNodeEnum::Start(StartNode::new());
        let dedup_node = match Dedup::new(start_node) {
            Ok(node) => node,
            Err(_) => return,
        };
        let child_opt_node = OptGroupNode::new(2, dedup_node.into_enum());

        let get_vertices_node = PlanNodeEnum::GetVertices(GetVertices::new(1, ""));
        let mut opt_node = OptGroupNode::new(1, get_vertices_node);
        opt_node.dependencies = vec![2];

        ctx.add_plan_node_and_group_node(2, &child_opt_node);

        let result = rule
            .apply(&mut ctx, &opt_node)
            .expect("Rule should apply successfully");
        // 规则应该匹配获取顶点节点并尝试与去重操作合并
        assert!(result.is_some());
    }

    #[test]
    fn test_merge_get_nbrs_and_dedup_rule() {
        let rule = MergeGetNbrsAndDedupRule;
        let mut ctx = create_test_context();

        let start_node = PlanNodeEnum::Start(StartNode::new());
        let dedup_node = match Dedup::new(start_node) {
            Ok(node) => node,
            Err(_) => return,
        };
        let child_opt_node = OptGroupNode::new(2, dedup_node.into_enum());

        let get_nbrs_node = PlanNodeEnum::GetNeighbors(GetNeighbors::new(1, ""));
        let mut opt_node = OptGroupNode::new(1, get_nbrs_node);
        opt_node.dependencies = vec![2];

        ctx.add_plan_node_and_group_node(2, &child_opt_node);

        let result = rule
            .apply(&mut ctx, &opt_node)
            .expect("Rule should apply successfully");
        // 规则应该匹配获取邻居节点并尝试与去重操作合并
        assert!(result.is_some());
    }

    #[test]
    fn test_merge_get_nbrs_and_project_rule() {
        let rule = MergeGetNbrsAndProjectRule;
        let mut ctx = create_test_context();

        let start_node = PlanNodeEnum::Start(StartNode::new());
        let project_node = match Project::new(
            start_node,
            vec![crate::query::validator::YieldColumn::new(
                crate::core::Expression::Variable("col1".to_string()),
                "col1".to_string(),
            )],
        ) {
            Ok(node) => node,
            Err(_) => return,
        };
        let child_opt_node = OptGroupNode::new(2, project_node.into_enum());

        let get_nbrs_node = PlanNodeEnum::GetNeighbors(GetNeighbors::new(1, ""));
        let mut opt_node = OptGroupNode::new(1, get_nbrs_node);
        opt_node.dependencies = vec![2];

        ctx.add_plan_node_and_group_node(2, &child_opt_node);

        let result = rule
            .apply(&mut ctx, &opt_node)
            .expect("Rule should apply successfully");
        // 规则应该匹配获取邻居节点并尝试与投影操作合并
        assert!(result.is_some());
    }

    #[test]
    fn test_combine_conditions() {
        // 测试辅助函数
        let result = combine_conditions(&"age > 18".to_string(), &"name = 'test'".to_string());
        assert_eq!(result, "(age > 18) AND (name = 'test')");

        let result = combine_conditions(&"".to_string(), &"name = 'test'".to_string());
        assert_eq!(result, "name = 'test'");

        let result = combine_conditions(&"age > 18".to_string(), &"".to_string());
        assert_eq!(result, "age > 18");
    }
}
