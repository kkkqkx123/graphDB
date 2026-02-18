//! 移除连接下方的添加顶点操作的规则

use crate::query::optimizer::plan::{OptContext, OptGroupNode, OptRule, Pattern, TransformResult, OptimizerError};
use crate::query::optimizer::rule_patterns::PatternBuilder;
use crate::query::optimizer::rule_traits::BaseOptRule;
use crate::query::planner::plan::core::nodes::plan_node_traits::MultipleInputNode;
use crate::query::visitor::PlanNodeVisitor;
use std::rc::Rc;
use std::cell::RefCell;

/// 移除连接下方的添加顶点操作的规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   AppendVertices
///       |
///   InnerJoin
/// ```
///
/// After:
/// ```text
///   InnerJoin
/// ```
///
/// # 适用条件
///
/// - AppendVertices 节点的子节点为连接操作（InnerJoin、HashInnerJoin、HashLeftJoin）
/// - 连接操作已经包含了所需的顶点信息
#[derive(Debug)]
pub struct RemoveAppendVerticesBelowJoinRule;

impl OptRule for RemoveAppendVerticesBelowJoinRule {
    fn name(&self) -> &str {
        "RemoveAppendVerticesBelowJoinRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> Result<Option<TransformResult>, OptimizerError> {
        let node_ref = group_node.borrow();
        let mut visitor = RemoveAppendVerticesBelowJoinVisitor {
            ctx,
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
        PatternBuilder::with_dependency("AppendVertices", "InnerJoin")
    }
}

impl BaseOptRule for RemoveAppendVerticesBelowJoinRule {}

/// 移除连接下方添加顶点访问者
///
/// 状态不变量：
/// - `is_eliminated` 为 true 时，`eliminated_node` 必须为 Some
/// - `is_eliminated` 为 false 时，`eliminated_node` 必须为 None
#[derive(Clone)]
struct RemoveAppendVerticesBelowJoinVisitor<'a> {
    is_eliminated: bool,
    eliminated_node: Option<OptGroupNode>,
    ctx: &'a OptContext,
    node_dependencies: Vec<usize>,
}

impl<'a> PlanNodeVisitor for RemoveAppendVerticesBelowJoinVisitor<'a> {
    type Result = Self;

    fn visit_default(&mut self) -> Self::Result {
        self.clone()
    }

    fn visit_append_vertices(&mut self, node: &crate::query::planner::plan::core::nodes::AppendVerticesNode) -> Self::Result {
        if self.is_eliminated {
            return self.clone();
        }

        if let Some(dep_id) = self.node_dependencies.first() {
            if let Some(child_node) = self.ctx.find_group_node_by_plan_node_id(*dep_id) {
                let child_node_ref = child_node.borrow();
                if child_node_ref.plan_node.is_inner_join()
                    || child_node_ref.plan_node.is_hash_inner_join()
                    || child_node_ref.plan_node.is_hash_left_join()
                {
                    let mut new_node = child_node_ref.clone();

                    if let Some(_output_var) = node.output_var() {
                        let inputs = node.inputs();
                        if let Some(input) = inputs.first() {
                            new_node.plan_node = *(*input).clone();
                        }
                    }

                    drop(child_node_ref);

                    self.is_eliminated = true;
                    self.eliminated_node = Some(new_node);
                } else {
                    drop(child_node_ref);
                }
            }
        }

        self.clone()
    }
}
