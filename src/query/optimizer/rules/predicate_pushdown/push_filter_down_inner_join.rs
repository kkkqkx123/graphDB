//! 将过滤条件下推到内连接操作的规则
//!
//! 该规则识别 Filter -> InnerJoin 模式，
//! 并将过滤条件下推到连接的一侧。

use crate::query::optimizer::plan::{OptContext, OptGroupNode, OptRule, Pattern, TransformResult};
use crate::query::optimizer::rule_patterns::PatternBuilder;
use crate::query::optimizer::rule_traits::BaseOptRule;
use crate::core::Expression;
use crate::query::optimizer::expression_utils::{check_col_name, split_filter};
use crate::query::planner::plan::core::nodes::plan_node_traits::BinaryInputNode;
use crate::query::planner::plan::core::nodes::PlanNodeEnum;
use std::rc::Rc;
use std::cell::RefCell;

/// 将过滤条件下推到内连接操作的规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   Filter(a.col1 > 10)
///           |
///   InnerJoin
///   /          \
/// Left      Right
/// ```
///
/// After:
/// ```text
///   InnerJoin
///   /          \
/// Filter      Right
/// (a.col1>10)
///   |
/// Left
/// ```
///
/// # 适用条件
///
/// - 过滤条件仅涉及连接的一侧
/// - 可以安全地将条件下推
#[derive(Debug)]
pub struct PushFilterDownInnerJoinRule;

impl OptRule for PushFilterDownInnerJoinRule {
    fn name(&self) -> &str {
        "PushFilterDownInnerJoinRule"
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

        if node_ref.dependencies.len() != 1 {
            return Ok(None);
        }

        let child_id = node_ref.dependencies[0];
        let child_node = match ctx.find_group_node_by_id(child_id) {
            Some(node) => node,
            None => return Ok(None),
        };

        let child_ref = child_node.borrow();
        
        if child_ref.plan_node.name() != "InnerJoin" {
            return Ok(None);
        }

        let filter_condition = match node_ref.plan_node.as_filter() {
            Some(filter) => filter.condition().clone(),
            None => return Ok(None),
        };

        let left_col_names = match &child_ref.plan_node {
            PlanNodeEnum::InnerJoin(join) => BinaryInputNode::left_input(join).col_names().to_vec(),
            _ => return Ok(None),
        };

        let picker = |expr: &Expression| -> bool {
            check_col_name(&left_col_names, expr)
        };

        let (filter_picked, filter_unpicked) = split_filter(&filter_condition, picker);

        let filter_picked = match filter_picked {
            Some(expr) => expr,
            None => return Ok(None),
        };

        let new_left_filter = match node_ref.plan_node.as_filter() {
            Some(filter) => {
                let mut new_filter = filter.clone();
                new_filter.set_condition(filter_picked);
                new_filter
            }
            None => return Ok(None),
        };

        let mut new_left_filter_group_node = node_ref.clone();
        new_left_filter_group_node.plan_node = PlanNodeEnum::Filter(new_left_filter);
        new_left_filter_group_node.dependencies = child_ref.dependencies.clone();

        let mut result = TransformResult::new();
        result.erase_curr = true;

        if let Some(unpicked) = filter_unpicked {
            let new_filter_node = match node_ref.plan_node.as_filter() {
                Some(filter) => {
                    let mut new_filter = filter.clone();
                    new_filter.set_condition(unpicked);
                    new_filter
                }
                None => return Ok(None),
            };

            let mut new_filter_group_node = node_ref.clone();
            new_filter_group_node.plan_node = PlanNodeEnum::Filter(new_filter_node);
            new_filter_group_node.dependencies = vec![child_id];

            result.add_new_group_node(Rc::new(RefCell::new(new_filter_group_node)));
        } else {
            result.add_new_group_node(Rc::new(RefCell::new(new_left_filter_group_node)));
        }
        
        Ok(Some(result))
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::with_dependency("Filter", "InnerJoin")
    }
}

impl BaseOptRule for PushFilterDownInnerJoinRule {}
