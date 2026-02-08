//! 折叠多个投影操作的规则

use crate::query::optimizer::plan::{OptContext, OptGroupNode, OptRule, Pattern, TransformResult};
use crate::query::optimizer::rule_patterns::CommonPatterns;
use crate::query::optimizer::rule_traits::{BaseOptRule, MergeRule};
use std::rc::Rc;
use std::cell::RefCell;

/// 折叠多个投影操作的规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   Project(col2)
///       |
///   Project(col1)
///       |
///   ScanVertices
/// ```
///
/// After:
/// ```text
///   Project(col2)
///       |
///   ScanVertices
/// ```
///
/// # 适用条件
///
/// - 当前节点为Project节点
/// - 子节点也为Project节点
/// - 可以折叠两个投影操作
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
    ) -> Result<Option<TransformResult>, crate::query::optimizer::engine::OptimizerError> {
        let node_ref = group_node.borrow();
        if !node_ref.plan_node.is_project() {
            return Ok(None);
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
        Ok(None)
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
