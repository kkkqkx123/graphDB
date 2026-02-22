//! 消除冗余过滤操作的规则

use crate::query::optimizer::plan::{OptContext, OptGroupNode};
use crate::query::optimizer::rule_patterns::PatternBuilder;
use crate::query::optimizer::rule_traits::is_expression_tautology;
use crate::query::optimizer::PlanNodeVisitor;

crate::define_elimination_rule! {
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
    pub struct EliminateFilterRule {
        target: Filter,
        target_check: is_filter,
        pattern: PatternBuilder::filter()
    }
    visitor: EliminateFilterVisitor
}

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

        let deps = node.dependencies();
        if deps.is_empty() {
            return self.clone();
        }

        let input = match deps.first() {
            Some(node) => node,
            None => return self.clone(),
        };
        let input_id = input.id() as usize;

        if let Some(child_node) = self.ctx.find_group_node_by_plan_node_id(input_id) {
            let new_node = child_node.clone();

            if let Some(_output_var) = node.output_var() {
                let mut new_node_borrowed = new_node.borrow_mut();
                new_node_borrowed.plan_node = (**input).clone();
            }

            self.is_eliminated = true;
            self.eliminated_node = Some(new_node.borrow().clone());
        }

        self.clone()
    }
}
