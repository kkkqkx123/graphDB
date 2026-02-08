//! 合并多个过滤操作的规则

use crate::query::optimizer::plan::{OptContext, OptGroupNode, OptRule, Pattern, TransformResult};
use crate::query::optimizer::rule_patterns::CommonPatterns;
use crate::query::optimizer::rule_traits::{combine_conditions, BaseOptRule, MergeRule};
use crate::query::planner::plan::FilterNode as FilterPlanNode;
use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;
use crate::query::visitor::PlanNodeVisitor;
use std::rc::Rc;
use std::cell::RefCell;

/// 合并多个过滤操作的规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   Filter(col2 > 200)
///       |
///   Filter(col1 > 100)
///       |
///   ScanVertices
/// ```
///
/// After:
/// ```text
///   Filter(col1 > 100 AND col2 > 200)
///       |
///   ScanVertices
/// ```
///
/// # 适用条件
///
/// - 当前节点为Filter节点
/// - 子节点也为Filter节点
/// - 可以合并两个过滤条件
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
    ) -> Result<Option<TransformResult>, crate::query::optimizer::engine::OptimizerError> {
        let node_ref = group_node.borrow();
        if !node_ref.plan_node.is_filter() {
            return Ok(None);
        }

        let mut visitor = CombineFilterVisitor {
            is_merged: false,
            merged_node: None,
            ctx: &ctx,
            node_dependencies: node_ref.dependencies.clone(),
        };

        let result = visitor.visit(&node_ref.plan_node);
        drop(node_ref);

        if result.is_merged {
            if let Some(new_node) = result.merged_node {
                let mut transform_result = TransformResult::new();
                transform_result.add_new_group_node(Rc::new(RefCell::new(new_node)));
                return Ok(Some(transform_result));
            }
        }
        Ok(None)
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
        _ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
        child: &OptGroupNode,
    ) -> Result<Option<TransformResult>, crate::query::optimizer::engine::OptimizerError> {
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

/// 合并过滤访问者
///
/// 状态不变量：
/// - `is_merged` 为 true 时，`merged_node` 必须为 Some
/// - `is_merged` 为 false 时，`merged_node` 必须为 None
#[derive(Clone)]
struct CombineFilterVisitor<'a> {
    is_merged: bool,
    merged_node: Option<OptGroupNode>,
    ctx: &'a OptContext,
    node_dependencies: Vec<usize>,
}

impl<'a> PlanNodeVisitor for CombineFilterVisitor<'a> {
    type Result = Self;

    fn visit_default(&mut self) -> Self::Result {
        self.clone()
    }

    fn visit_filter(&mut self, node: &crate::query::planner::plan::core::nodes::FilterNode) -> Self::Result {
        if let Some(dep_id) = self.node_dependencies.first() {
            if let Some(child_node) = self.ctx.find_group_node_by_id(*dep_id) {
                let child_node_ref = child_node.borrow();
                if child_node_ref.plan_node.is_filter() {
                    if let Some(child_filter) = child_node_ref.plan_node.as_filter() {
                        let top_condition = node.condition();
                        let child_condition = child_filter.condition();

                        let combined_condition_str = combine_conditions(
                            &format!("{:?}", top_condition),
                            &format!("{:?}", child_condition),
                        );

                        let child_input = (*child_filter.input()).clone();
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

                        self.is_merged = true;
                        self.merged_node = Some(combined_opt_node);
                    }
                }
            }
        }

        self.clone()
    }
}
