//! 消除重复操作的规则

use crate::query::optimizer::plan::{OptContext, OptGroupNode, OptRule, Pattern, TransformResult};
use crate::query::optimizer::rule_patterns::PatternBuilder;
use crate::query::optimizer::rule_traits::BaseOptRule;
use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;
use crate::query::visitor::PlanNodeVisitor;
use std::rc::Rc;
use std::cell::RefCell;

/// 消除重复操作的规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   Dedup
///       |
///   IndexScan (索引扫描保证唯一性)
/// ```
///
/// After:
/// ```text
///   IndexScan
/// ```
///
/// # 适用条件
///
/// - Dedup 节点的子节点为 IndexScan、GetVertices 或 GetEdges
/// - 这些操作本身就保证结果的唯一性
#[derive(Debug)]
pub struct DedupEliminationRule;

impl OptRule for DedupEliminationRule {
    fn name(&self) -> &str {
        "DedupEliminationRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> Result<Option<TransformResult>, crate::query::optimizer::engine::OptimizerError> {
        let node_ref = group_node.borrow();
        let mut visitor = DedupEliminationVisitor {
            ctx,
            is_eliminated: false,
            eliminated_node: None,
        };

        let result = visitor.visit(&node_ref.plan_node);
        drop(node_ref);

        if result.is_eliminated {
            if let Some(new_node) = result.eliminated_node {
                let mut result = TransformResult::new();
                result.add_new_group_node(Rc::new(RefCell::new(new_node)));
                return Ok(Some(result));
            }
        }
        Ok(None)
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::dedup()
    }
}

impl BaseOptRule for DedupEliminationRule {}

/// 消除去重访问者
///
/// 状态不变量：
/// - `is_eliminated` 为 true 时，`eliminated_node` 必须为 Some
/// - `is_eliminated` 为 false 时，`eliminated_node` 必须为 None
#[derive(Clone)]
struct DedupEliminationVisitor<'a> {
    is_eliminated: bool,
    eliminated_node: Option<OptGroupNode>,
    ctx: &'a OptContext,
}

impl<'a> PlanNodeVisitor for DedupEliminationVisitor<'a> {
    type Result = Self;

    fn visit_default(&mut self) -> Self::Result {
        self.clone()
    }

    fn visit_dedup(&mut self, node: &crate::query::planner::plan::core::nodes::DedupNode) -> Self::Result {
        if self.is_eliminated {
            return self.clone();
        }

        let input = SingleInputNode::input(node);
        let input_id = input.id() as usize;

        if let Some(child_node) = self.ctx.find_group_node_by_plan_node_id(input_id) {
            let child_node_ref = child_node.borrow();
            if child_node_ref.plan_node.is_index_scan()
                || child_node_ref.plan_node.is_get_vertices()
                || child_node_ref.plan_node.is_get_edges()
            {
                let mut new_node = child_node_ref.clone();

                if let Some(_output_var) = node.output_var() {
                    new_node.plan_node = input.clone();
                }

                drop(child_node_ref);

                self.is_eliminated = true;
                self.eliminated_node = Some(new_node);
            } else {
                drop(child_node_ref);
            }
        }

        self.clone()
    }
}
