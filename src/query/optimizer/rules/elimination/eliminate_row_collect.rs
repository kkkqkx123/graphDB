//! 消除冗余数据收集操作的规则

use crate::query::optimizer::plan::{OptContext, OptGroupNode, OptRule, Pattern, TransformResult};
use crate::query::optimizer::rule_patterns::PatternBuilder;
use crate::query::optimizer::rule_traits::{create_basic_pattern, BaseOptRule};
use crate::query::visitor::PlanNodeVisitor;
use std::rc::Rc;
use std::cell::RefCell;

/// 消除冗余数据收集操作的规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   DataCollect(kind=kRowBasedMove)
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
/// - DataCollect 节点的 kind 为 kRowBasedMove
/// - 子节点可以直接返回结果
#[derive(Debug)]
pub struct EliminateRowCollectRule;

impl OptRule for EliminateRowCollectRule {
    fn name(&self) -> &str {
        "EliminateRowCollectRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> Result<Option<TransformResult>, crate::query::optimizer::engine::OptimizerError> {
        let node_ref = group_node.borrow();
        let mut visitor = EliminateRowCollectVisitor {
            ctx: &ctx,
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
        create_basic_pattern("DataCollect")
    }
}

impl BaseOptRule for EliminateRowCollectRule {}

/// 消除数据收集访问者
///
/// 状态不变量：
/// - `is_eliminated` 为 true 时，`eliminated_node` 必须为 Some
/// - `is_eliminated` 为 false 时，`eliminated_node` 必须为 None
#[derive(Clone)]
struct EliminateRowCollectVisitor<'a> {
    is_eliminated: bool,
    eliminated_node: Option<OptGroupNode>,
    ctx: &'a OptContext,
}

impl<'a> PlanNodeVisitor for EliminateRowCollectVisitor<'a> {
    type Result = Self;

    fn visit_default(&mut self) -> Self::Result {
        self.clone()
    }

    fn visit_data_collect(&mut self, node: &crate::query::planner::plan::core::nodes::DataCollectNode) -> Self::Result {
        if self.is_eliminated {
            return self.clone();
        }

        if node.collect_kind() != "kRowBasedMove" {
            return self.clone();
        }

        let deps = node.dependencies();
        if deps.is_empty() {
            return self.clone();
        }

        let input = deps.first().unwrap();
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
