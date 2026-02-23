//! 将过滤条件下推到GetNeighbors操作的规则
//!
//! 该规则识别 Filter -> GetNeighbors 模式，
//! 并将过滤条件下推到 GetNeighbors 节点中。

use crate::query::optimizer::plan::{OptContext, OptGroupNode, OptRule, Pattern, TransformResult, OptimizerError};
use crate::query::optimizer::rule_patterns::PatternBuilder;
use crate::query::optimizer::rule_traits::BaseOptRule;
use std::rc::Rc;
use std::cell::RefCell;

/// 将过滤条件下推到GetNeighbors操作的规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   Filter(e.likeness > 78)
///           |
///   GetNeighbors
/// ```
///
/// After:
/// ```text
///   GetNeighbors(filter: e.likeness > 78)
/// ```
///
/// # 适用条件
///
/// - GetNeighbors 节点获取边属性
/// - 过滤条件可以下推到存储层
#[derive(Debug)]
pub struct PushFilterDownGetNbrsRule;

impl OptRule for PushFilterDownGetNbrsRule {
    fn name(&self) -> &str {
        "PushFilterDownGetNbrsRule"
    }

    fn apply(
        &self,
        _ctx: &mut OptContext,
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
        let child_node = match _ctx.find_group_node_by_id(child_id) {
            Some(node) => node,
            None => return Ok(None),
        };

        let child_ref = child_node.borrow();
        
        if child_ref.plan_node.name() != "GetNeighbors" {
            return Ok(None);
        }

        let mut result = TransformResult::new();
        result.add_new_group_node(group_node.clone());
        
        Ok(Some(result))
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::with_dependency("Filter", "GetNeighbors")
    }
}

impl BaseOptRule for PushFilterDownGetNbrsRule {}
