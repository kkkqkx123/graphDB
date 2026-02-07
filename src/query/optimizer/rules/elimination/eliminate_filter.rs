//! 消除冗余过滤操作的规则

use crate::query::optimizer::plan::{OptContext, OptGroupNode, OptRule, Pattern, TransformResult};
use crate::query::optimizer::rule_patterns::PatternBuilder;
use crate::query::optimizer::rule_traits::{is_expression_tautology, BaseOptRule};
use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;
use crate::query::visitor::PlanNodeVisitor;
use std::rc::Rc;
use std::cell::RefCell;

/// 消除冗余过滤操作的规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   Filter(TRUE)
///       |
///   ScanVertices
/// ```
///
/// After:
/// ```text
///   ScanVertices
/// ```
///
/// # 适用条件
///
/// - 过滤条件为永真式（如 TRUE、1=1 等）
#[derive(Debug)]
pub struct EliminateFilterRule;

impl OptRule for EliminateFilterRule {
    fn name(&self) -> &str {
        "EliminateFilterRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> Result<Option<TransformResult>, crate::query::optimizer::engine::OptimizerError> {
        let node_ref = group_node.borrow();
        let mut visitor = EliminateFilterVisitor {
            ctx: &ctx,
            is_eliminated: false,
            eliminated_node: None,
            node_dependencies: node_ref.dependencies.clone(),
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
        PatternBuilder::filter()
    }
}

impl BaseOptRule for EliminateFilterRule {}

/// 消除过滤访问者
///
/// 状态不变量：
/// - `is_eliminated` 为 true 时，`eliminated_node` 必须为 Some
/// - `is_eliminated` 为 false 时，`eliminated_node` 必须为 None
#[derive(Clone)]
struct EliminateFilterVisitor<'a> {
    is_eliminated: bool,
    eliminated_node: Option<OptGroupNode>,
    ctx: &'a OptContext,
    node_dependencies: Vec<usize>,
}

impl<'a> PlanNodeVisitor for EliminateFilterVisitor<'a> {
    type Result = Self;

    fn visit_default(&mut self) -> Self::Result {
        self.clone()
    }

    fn visit_filter(&mut self, node: &crate::query::planner::plan::core::nodes::FilterNode) -> Self::Result {
        if self.is_eliminated {
            return self.clone();
        }

        let condition = node.condition();
        if !is_expression_tautology(condition) {
            return self.clone();
        }

        if let Some(dep_id) = self.node_dependencies.first() {
            if let Some(child_node) = self.ctx.find_group_node_by_id(*dep_id) {
                let new_node = child_node.clone();

                if let Some(_output_var) = node.output_var() {
                    let mut new_node_borrowed = new_node.borrow_mut();
                    new_node_borrowed.plan_node = SingleInputNode::input(node).clone();
                }

                self.is_eliminated = true;
                self.eliminated_node = Some(new_node.borrow().clone());
            }
        }

        self.clone()
    }
}
