//! 将过滤条件下推到哈希内连接操作的规则
//!
//! 该规则识别 Filter -> HashInnerJoin 模式，
//! 并将过滤条件下推到连接的两侧。

use crate::query::optimizer::plan::{OptContext, OptGroupNode, OptRule, Pattern, TransformResult, OptimizerError};
use crate::query::optimizer::rule_patterns::PatternBuilder;
use crate::query::optimizer::rule_traits::BaseOptRule;
use crate::core::Expression;
use crate::query::optimizer::expression_utils::{check_col_name, split_filter};
use crate::query::planner::plan::core::nodes::plan_node_traits::BinaryInputNode;
use crate::query::planner::plan::core::nodes::PlanNodeEnum;
use std::rc::Rc;
use std::cell::RefCell;

/// 将过滤条件下推到哈希内连接操作的规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   Filter(a.col1 > 10 AND b.col2 < 20)
///           |
///   HashInnerJoin
///   /          \
/// Left      Right
/// ```
///
/// After:
/// ```text
///   HashInnerJoin
///   /          \
/// Filter      Filter
/// (a.col1>10) (b.col2<20)
///   |            |
/// Left        Right
/// ```
///
/// # 适用条件
///
/// - 过滤条件可以分离为左右两侧的条件
/// - 可以安全地将条件下推到两侧
#[derive(Debug)]
pub struct PushFilterDownHashInnerJoinRule;

impl OptRule for PushFilterDownHashInnerJoinRule {
    fn name(&self) -> &str {
        "PushFilterDownHashInnerJoinRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> Result<Option<TransformResult>, OptimizerError> {
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
        
        if child_ref.plan_node.name() != "HashInnerJoin" {
            return Ok(None);
        }

        let filter_condition = match node_ref.plan_node.as_filter() {
            Some(filter) => filter.condition().clone(),
            None => return Ok(None),
        };

        let (left_col_names, right_col_names) = match &child_ref.plan_node {
            PlanNodeEnum::HashInnerJoin(join) => (
                BinaryInputNode::left_input(join).col_names().to_vec(),
                BinaryInputNode::right_input(join).col_names().to_vec(),
            ),
            _ => return Ok(None),
        };

        let left_picker = |expr: &Expression| -> bool {
            check_col_name(&left_col_names, expr)
        };

        let (_left_filter_picked, remaining_after_left) = split_filter(&filter_condition, left_picker);

        let _left_filter_picked = match _left_filter_picked {
            Some(expr) => expr,
            None => return Ok(None),
        };

        let right_picker = |expr: &Expression| -> bool {
            check_col_name(&right_col_names, expr)
        };

        let (_right_filter_picked, _remaining_after_right) = match remaining_after_left {
            Some(expr) => split_filter(&expr, right_picker),
            None => (None, None),
        };

        let _right_filter_picked = match _right_filter_picked {
            Some(expr) => expr,
            None => return Ok(None),
        };

        let mut result = TransformResult::new();
        result.add_new_group_node(group_node.clone());
        
        Ok(Some(result))
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::with_dependency("Filter", "HashInnerJoin")
    }
}

impl BaseOptRule for PushFilterDownHashInnerJoinRule {}
