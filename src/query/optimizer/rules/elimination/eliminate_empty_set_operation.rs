//! 空集操作优化规则
//!
//! 优化集合操作中的空集情况：
//! - Minus: 如果减输入为空，直接返回主输入
//! - Intersect: 如果任一输入为空，返回空集

use crate::query::optimizer::plan::{OptContext, OptGroupNode, OptRule, Pattern, TransformResult, OptimizerError};
use crate::query::optimizer::rule_traits::BaseOptRule;
use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum as Enum;
use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;
use crate::query::visitor::PlanNodeVisitor;
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::Arc;

/// 空集操作优化规则
///
/// 优化集合操作中的空集情况，避免不必要的计算
#[derive(Debug)]
pub struct EliminateEmptySetOperationRule;

impl OptRule for EliminateEmptySetOperationRule {
    fn name(&self) -> &str {
        "EliminateEmptySetOperationRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> Result<Option<TransformResult>, OptimizerError> {
        let node_ref = group_node.borrow();
        let mut visitor = EliminateEmptySetOperationVisitor {
            ctx,
            is_eliminated: false,
            eliminated_node: None,
        };

        let result = visitor.visit(&node_ref.plan_node);
        drop(node_ref);

        if result.is_eliminated {
            if let Some(new_node) = result.eliminated_node {
                let mut transform_result = TransformResult::new();
                transform_result.add_new_group_node(Rc::new(RefCell::new(new_node)));
                return Ok(Some(transform_result));
            }
        }
        Ok(None)
    }

    fn pattern(&self) -> Pattern {
        Pattern::new()
    }
}

impl BaseOptRule for EliminateEmptySetOperationRule {}

/// 空集操作优化访问者
struct EliminateEmptySetOperationVisitor<'a> {
    is_eliminated: bool,
    eliminated_node: Option<OptGroupNode>,
    ctx: &'a mut OptContext,
}

impl<'a> PlanNodeVisitor for EliminateEmptySetOperationVisitor<'a> {
    type Result = EliminateEmptySetOperationResult;

    fn visit_default(&mut self) -> Self::Result {
        EliminateEmptySetOperationResult {
            is_eliminated: self.is_eliminated,
            eliminated_node: self.eliminated_node.take(),
        }
    }

    fn visit_minus(&mut self, node: &crate::query::planner::plan::core::nodes::set_operations_node::MinusNode) -> Self::Result {
        if self.is_eliminated {
            return self.visit_default();
        }

        let minus_input = node.minus_input();
        
        if is_empty_node(minus_input) {
            self.is_eliminated = true;
            self.eliminated_node = Some(OptGroupNode::new(
                self.ctx.allocate_node_id(),
                node.input().clone(),
            ));
            return self.visit_default();
        }
        
        self.visit_default()
    }

    fn visit_intersect(&mut self, node: &crate::query::planner::plan::core::nodes::set_operations_node::IntersectNode) -> Self::Result {
        if self.is_eliminated {
            return self.visit_default();
        }

        let input = node.input();
        let intersect_input = node.intersect_input();
        
        if is_empty_node(input) || is_empty_node(intersect_input) {
            self.is_eliminated = true;
            self.eliminated_node = Some(OptGroupNode::new(
                self.ctx.allocate_node_id(),
                create_empty_node(),
            ));
            return self.visit_default();
        }
        
        self.visit_default()
    }
}

/// 空集操作优化结果
struct EliminateEmptySetOperationResult {
    is_eliminated: bool,
    eliminated_node: Option<OptGroupNode>,
}

/// 判断节点是否为空集节点
fn is_empty_node(node: &Enum) -> bool {
    match node {
        Enum::Start(_) => true,
        Enum::ScanVertices(n) => n.limit().map_or(false, |l| l == 0),
        Enum::ScanEdges(n) => n.limit().map_or(false, |l| l == 0),
        Enum::GetVertices(n) => n.limit().map_or(false, |l| l == 0),
        Enum::GetEdges(n) => n.limit().map_or(false, |l| l == 0),
        Enum::Limit(n) => n.count() == 0,
        Enum::Filter(n) => is_empty_node(n.input()),
        Enum::Project(n) => is_empty_node(n.input()),
        Enum::Dedup(n) => is_empty_node(n.input()),
        _ => false,
    }
}

/// 创建空集节点
fn create_empty_node() -> Enum {
    use crate::query::planner::plan::core::nodes::start_node::StartNode;
    Enum::Start(StartNode::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::context::QueryContext;
    use crate::query::optimizer::plan::{OptContext, OptGroupNode};
    use crate::query::planner::plan::core::nodes::set_operations_node::{MinusNode, IntersectNode};
    use crate::query::planner::plan::core::nodes::start_node::StartNode;
    use std::rc::Rc;
    use std::cell::RefCell;

    fn create_test_context() -> OptContext {
        let query_context = Arc::new(QueryContext::default());
        OptContext::new(query_context)
    }

    #[test]
    fn test_eliminate_empty_minus() {
        let rule = EliminateEmptySetOperationRule;
        let mut ctx = create_test_context();

        let start = Enum::Start(StartNode::new());
        let empty_start = Enum::Start(StartNode::new());
        
        let minus_node = MinusNode::new(start.clone(), empty_start).expect("Failed to create minus node");
        let plan_node = Enum::Minus(minus_node);
        let opt_node = OptGroupNode::new(1, plan_node);

        let result = rule.apply(&mut ctx, &Rc::new(RefCell::new(opt_node)));
        
        assert!(result.is_ok());
        assert!(result.expect("Expected result to exist").is_some());
    }

    #[test]
    fn test_eliminate_empty_intersect() {
        let rule = EliminateEmptySetOperationRule;
        let mut ctx = create_test_context();

        let start = Enum::Start(StartNode::new());
        let empty_start = Enum::Start(StartNode::new());
        
        let intersect_node = IntersectNode::new(start.clone(), empty_start).expect("Failed to create intersect node");
        let plan_node = Enum::Intersect(intersect_node);
        let opt_node = OptGroupNode::new(1, plan_node);

        let result = rule.apply(&mut ctx, &Rc::new(RefCell::new(opt_node)));
        
        assert!(result.is_ok());
        assert!(result.expect("Expected result to exist").is_some());
    }
}
