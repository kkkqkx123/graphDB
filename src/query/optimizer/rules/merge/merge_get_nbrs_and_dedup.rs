//! 合并获取邻居和去重操作的规则

use crate::query::optimizer::plan::{OptContext, OptGroupNode, OptRule, Pattern, TransformResult};
use crate::query::optimizer::rule_patterns::PatternBuilder;
use crate::query::optimizer::rule_traits::{BaseOptRule, MergeRule};
use std::rc::Rc;
use std::cell::RefCell;

/// 合并获取邻居和去重操作的规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   GetNeighbors
///       |
///   Dedup
///       |
///   ScanVertices
/// ```
///
/// After:
/// ```text
///   GetNeighbors
///       |
///   ScanVertices
/// ```
///
/// # 适用条件
///
/// - 当前节点为GetNeighbors节点
/// - 子节点为Dedup节点
/// - 可以将去重操作合并到GetNeighbors中
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
    ) -> Result<Option<TransformResult>, crate::query::optimizer::engine::OptimizerError> {
        let node_ref = group_node.borrow();
        if !node_ref.plan_node.is_get_neighbors() {
            return Ok(None);
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
        Ok(None)
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
        _ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
        _child: &OptGroupNode,
    ) -> Result<Option<TransformResult>, crate::query::optimizer::engine::OptimizerError> {
        let _node_ref = group_node.borrow();
        let mut result = TransformResult::new();
        result.add_new_group_node(group_node.clone());
        Ok(Some(result))
    }
}
