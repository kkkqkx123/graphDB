//! 消除冗余添加顶点操作的规则

use crate::query::optimizer::plan::{OptContext, OptGroupNode, OptRule, Pattern, TransformResult};
use crate::query::optimizer::rule_traits::{create_basic_pattern, BaseOptRule};
use crate::query::planner::plan::core::nodes::plan_node_traits::{MultipleInputNode, PlanNode};
use crate::query::visitor::PlanNodeVisitor;
use std::rc::Rc;
use std::cell::RefCell;

/// 消除冗余添加顶点操作的规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   AppendVertices(vids=[], tag_ids=[])
///       |
///   GetNeighbors
/// ```
///
/// After:
/// ```text
///   GetNeighbors
/// ```
///
/// # 适用条件
///
/// - AppendVertices 节点的 vids 和 tag_ids 都为空
#[derive(Debug)]
pub struct EliminateAppendVerticesRule;

impl OptRule for EliminateAppendVerticesRule {
    fn name(&self) -> &str {
        "EliminateAppendVerticesRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> Result<Option<TransformResult>, crate::query::optimizer::engine::OptimizerError> {
        let node_ref = group_node.borrow();
        let mut visitor = EliminateAppendVerticesVisitor {
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
        create_basic_pattern("AppendVertices")
    }
}

impl BaseOptRule for EliminateAppendVerticesRule {}

/// 消除添加顶点访问者
///
/// 状态不变量：
/// - `is_eliminated` 为 true 时，`eliminated_node` 必须为 Some
/// - `is_eliminated` 为 false 时，`eliminated_node` 必须为 None
#[derive(Clone)]
struct EliminateAppendVerticesVisitor<'a> {
    is_eliminated: bool,
    eliminated_node: Option<OptGroupNode>,
    ctx: &'a OptContext,
    node_dependencies: Vec<usize>,
}

impl<'a> PlanNodeVisitor for EliminateAppendVerticesVisitor<'a> {
    type Result = Self;

    fn visit_default(&mut self) -> Self::Result {
        self.clone()
    }

    fn visit_append_vertices(&mut self, node: &crate::query::planner::plan::core::nodes::AppendVerticesNode) -> Self::Result {
        if self.is_eliminated {
            return self.clone();
        }

        if !node.vids().is_empty() || !node.tag_ids().is_empty() {
            return self.clone();
        }

        if let Some(dep_id) = self.node_dependencies.first() {
            if let Some(child_node) = self.ctx.find_group_node_by_plan_node_id(*dep_id) {
                let new_node = child_node.clone();

                if let Some(_output_var) = node.output_var() {
                    let inputs = node.inputs();
                    if let Some(input) = inputs.first() {
                        let mut new_node_borrowed = new_node.borrow_mut();
                        new_node_borrowed.plan_node = (**input).clone();
                    }
                }

                self.is_eliminated = true;
                self.eliminated_node = Some(new_node.borrow().clone());
            }
        }

        self.clone()
    }
}
