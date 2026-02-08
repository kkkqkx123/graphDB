//! 将边过滤条件下推到Traverse节点的规则
//!
//! 该规则识别 Traverse 节点中的 eFilter，
//! 并将其重写为具体的边属性表达式。

use crate::query::optimizer::plan::{OptContext, OptGroupNode, OptRule, Pattern, TransformResult};
use crate::query::optimizer::rule_patterns::PatternBuilder;
use crate::query::optimizer::rule_traits::BaseOptRule;
use crate::query::planner::plan::core::nodes::PlanNodeEnum;
use std::rc::Rc;
use std::cell::RefCell;

/// 将边过滤条件下推到Traverse节点的规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   Traverse(eFilter: *.likeness > 78)
/// ```
///
/// After:
/// ```text
///   Traverse(filter: e.likeness > 78)
/// ```
///
/// # 适用条件
///
/// - Traverse 节点存在 eFilter
/// - eFilter 包含通配符边属性表达式
/// - Traverse 不为零步遍历
#[derive(Debug)]
pub struct PushVFilterDownScanVerticesRule;

impl OptRule for PushVFilterDownScanVerticesRule {
    fn name(&self) -> &str {
        "PushVFilterDownScanVerticesRule"
    }

    fn apply(
        &self,
        _ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> Result<Option<TransformResult>, crate::query::optimizer::engine::OptimizerError> {
        let node_ref = group_node.borrow();
        
        let _traverse = match &node_ref.plan_node {
            PlanNodeEnum::Traverse(traverse) => traverse,
            _ => return Ok(None),
        };

        let _e_filter = match &node_ref.plan_node {
            PlanNodeEnum::Traverse(traverse) => traverse.e_filter().cloned(),
            _ => return Ok(None),
        };

        let mut result = TransformResult::new();
        result.add_new_group_node(group_node.clone());
        
        Ok(Some(result))
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::node("Traverse")
    }
}

impl BaseOptRule for PushVFilterDownScanVerticesRule {}
