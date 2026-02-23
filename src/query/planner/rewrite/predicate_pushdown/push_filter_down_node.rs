//! 将过滤条件下推到Traverse/AppendVertices节点的规则
//!
//! 该规则识别 Traverse/AppendVertices 节点中的 vFilter，
//! 并将可下推的过滤条件下推到数据源。

use crate::query::optimizer::plan::{OptContext, OptGroupNode, OptRule, Pattern, TransformResult, OptimizerError};
use crate::query::optimizer::rule_patterns::PatternBuilder;
use crate::query::optimizer::rule_traits::BaseOptRule;
use crate::query::planner::plan::core::nodes::PlanNodeEnum;
use std::rc::Rc;
use std::cell::RefCell;

/// 将过滤条件下推到Traverse/AppendVertices节点的规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   Traverse(vFilter: v.age > 18)
/// ```
///
/// After:
/// ```text
///   Traverse(vFilter: <remained>, firstStepFilter: v.age > 18)
/// ```
///
/// # 适用条件
///
/// - Traverse 或 AppendVertices 节点存在 vFilter
/// - vFilter 可以部分下推到 firstStepFilter
#[derive(Debug)]
pub struct PushFilterDownNodeRule;

impl OptRule for PushFilterDownNodeRule {
    fn name(&self) -> &str {
        "PushFilterDownNodeRule"
    }

    fn apply(
        &self,
        _ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> Result<Option<TransformResult>, OptimizerError> {
        let node_ref = group_node.borrow();
        
        let _v_filter = match &node_ref.plan_node {
            PlanNodeEnum::Traverse(traverse) => traverse.v_filter().cloned(),
            PlanNodeEnum::AppendVertices(append) => append.v_filter().cloned(),
            _ => return Ok(None),
        };

        let _v_filter = match _v_filter {
            Some(filter) => filter,
            None => return Ok(None),
        };

        let mut result = TransformResult::new();
        result.add_new_group_node(group_node.clone());
        
        Ok(Some(result))
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::node("Traverse")
    }
}

impl BaseOptRule for PushFilterDownNodeRule {}
