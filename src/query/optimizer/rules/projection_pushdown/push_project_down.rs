//! 向数据源推送投影操作的规则
//!
//! 该规则将投影操作推向数据源，减少数据传输量。
//!
//! # 转换示例
//!
//! Before:
//! ```text
//! Project(col1)
//!       |
//!   ScanVertices
//! ```
//!
//! After:
//! ```text
//! ScanVertices(col1)
//! ```
//!
//! # 适用条件
//!
//! - Project 节点的子节点是数据访问节点（ScanVertices、ScanEdges、GetVertices）
//! - Project 节点有列定义

use crate::query::optimizer::plan::{OptContext, OptGroupNode, OptRule, Pattern, TransformResult, Result as OptResult};
use crate::query::optimizer::rule_patterns::PatternBuilder;
use crate::query::optimizer::rule_traits::{BaseOptRule, PushDownRule};
use crate::query::planner::plan::core::nodes::PlanNodeEnum;
use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;
use crate::query::visitor::PlanNodeVisitor;
use std::rc::Rc;
use std::cell::RefCell;

/// 投影下推访问者
///
/// 状态不变量：
/// - `is_pushed_down` 为 true 时，`pushed_node` 必须为 Some
/// - `is_pushed_down` 为 false 时，`pushed_node` 必须为 None
#[derive(Clone)]
struct ProjectionPushDownVisitor<'a> {
    is_pushed_down: bool,
    pushed_node: Option<OptGroupNode>,
    ctx: &'a OptContext,
}

impl<'a> ProjectionPushDownVisitor<'a> {
    fn can_push_down_project(node: &crate::query::planner::plan::core::nodes::ProjectNode) -> bool {
        !node.columns().is_empty()
    }
}

impl<'a> PlanNodeVisitor for ProjectionPushDownVisitor<'a> {
    type Result = Self;

    fn visit_default(&mut self) -> Self::Result {
        self.clone()
    }

    fn visit_project(&mut self, node: &crate::query::planner::plan::core::nodes::ProjectNode) -> Self::Result {
        let input = node.input();
        let input_id = input.id() as usize;

        if let Some(child_node) = self.ctx.find_group_node_by_plan_node_id(input_id) {
            let child_node_ref = child_node.borrow();
            let child_name = child_node_ref.plan_node.name();

            let (pushed_down, new_node) = match child_name.as_ref() {
                "ScanVertices" | "ScanEdges" | "GetVertices" | "GetEdges" | "GetNeighbors" => {
                    if Self::can_push_down_project(node) {
                        let mut new_child_node = child_node_ref.clone();
                        new_child_node.plan_node = input.clone();
                        (true, Some(new_child_node))
                    } else {
                        (false, None)
                    }
                }
                _ => (false, None),
            };

            drop(child_node_ref);

            if pushed_down {
                self.is_pushed_down = true;
                self.pushed_node = new_node;
            }
        } else if input.is_start() {
            if Self::can_push_down_project(node) {
                let start_node = input.clone();
                let new_opt_node = OptGroupNode::new(input_id, start_node);
                self.is_pushed_down = true;
                self.pushed_node = Some(new_opt_node);
            }
        }

        self.clone()
    }
}

/// 向数据源推送投影操作的规则
#[derive(Debug)]
pub struct PushProjectDownRule;

impl OptRule for PushProjectDownRule {
    fn name(&self) -> &str {
        "PushProjectDownRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        node: &Rc<RefCell<OptGroupNode>>,
    ) -> OptResult<Option<TransformResult>> {
        let node_ref = node.borrow();
        if !node_ref.plan_node.is_project() {
            return Ok(None);
        }
        let mut visitor = ProjectionPushDownVisitor {
            is_pushed_down: false,
            pushed_node: None,
            ctx: &ctx,
        };
        let result = visitor.visit(&node_ref.plan_node);
        if result.is_pushed_down {
            if let Some(new_node) = result.pushed_node {
                let mut transform_result = TransformResult::new();
                transform_result.add_new_group_node(Rc::new(RefCell::new(new_node)));
                Ok(Some(transform_result))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::project()
    }
}

impl BaseOptRule for PushProjectDownRule {}

impl PushDownRule for PushProjectDownRule {
    fn can_push_down_to(&self, child_kind: &PlanNodeEnum) -> bool {
        // 投影可以下推到数据访问操作
        matches!(
            child_kind,
            PlanNodeEnum::ScanVertices(_)
                | PlanNodeEnum::ScanEdges(_)
                | PlanNodeEnum::GetVertices(_)
        )
    }

    fn create_pushed_down_node(
        &self,
        _ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
        _child: &OptGroupNode,
    ) -> OptResult<Option<TransformResult>> {
        let _node_ref = group_node.borrow();
        let mut result = TransformResult::new();
        result.add_new_group_node(group_node.clone());
        Ok(Some(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::context::execution::QueryContext;
    use crate::query::optimizer::plan::{OptContext, OptGroupNode};

    fn create_test_context() -> OptContext {
        let query_context = QueryContext::new();
        OptContext::new(query_context)
    }

    #[test]
    fn test_push_project_down_rule() {
        let rule = PushProjectDownRule;
        let mut ctx = create_test_context();

        let project_node = PlanNodeEnum::Project(
            crate::query::planner::plan::core::nodes::ProjectNode::new(
                PlanNodeEnum::Start(crate::query::planner::plan::core::nodes::StartNode::new()),
                vec![crate::query::validator::YieldColumn::new(
                    crate::core::Expression::Variable("col1".to_string()),
                    "col1".to_string(),
                )],
            )
            .expect("Node should be created successfully"),
        );
        let opt_node = OptGroupNode::new(1, project_node);

        let result = rule
            .apply(&mut ctx, &Rc::new(RefCell::new(opt_node)))
            .expect("Rule should apply successfully");
        assert!(result.is_some());
    }
}
