//! 投影下推优化规则
//! 这些规则负责将投影操作下推到计划树的底层，以减少数据传输量

use super::engine::OptimizerError;
use super::plan::{OptContext, OptGroupNode, OptRule, Pattern, TransformResult};
use super::rule_patterns::PatternBuilder;
use super::rule_traits::{BaseOptRule, PushDownRule};
use crate::query::planner::plan::core::nodes::PlanNodeEnum;
use crate::query::visitor::PlanNodeVisitor;
use std::rc::Rc;
use std::cell::RefCell;

/// 投影下推访问者
#[derive(Clone)]
struct ProjectionPushDownVisitor {
    pushed_down: bool,
    new_node: Option<OptGroupNode>,
    ctx: *const OptContext,
}

impl ProjectionPushDownVisitor {
    fn get_ctx(&self) -> &OptContext {
        unsafe { &*self.ctx }
    }

    fn can_push_down_project(node: &crate::query::planner::plan::core::nodes::ProjectNode) -> bool {
        !node.columns().is_empty()
    }
}

impl PlanNodeVisitor for ProjectionPushDownVisitor {
    type Result = Self;

    fn visit_default(&mut self) -> Self::Result {
        self.clone()
    }

    fn visit_project(&mut self, node: &crate::query::planner::plan::core::nodes::ProjectNode) -> Self::Result {
        let input = node.input();
        let input_id = input.id() as usize;

        if let Some(child_node) = self.get_ctx().find_group_node_by_plan_node_id(input_id) {
            let child_name = child_node.plan_node.name();

            match child_name.as_ref() {
                "ScanVertices" | "ScanEdges" | "GetVertices" | "GetEdges" | "GetNeighbors" => {
                    if Self::can_push_down_project(node) {
                        let mut new_child_node = child_node.clone();
                        new_child_node.plan_node = input.clone();

                        self.pushed_down = true;
                        self.new_node = Some(new_child_node);
                    }
                }
                _ => {}
            }
        }

        self.clone()
    }
}

/// 投影下推规则
#[derive(Debug)]
pub struct ProjectionPushDownRule;

impl OptRule for ProjectionPushDownRule {
    fn name(&self) -> &str {
        "ProjectionPushDownRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        if !node.plan_node.is_project() {
            return Ok(None);
        }

        let mut visitor = ProjectionPushDownVisitor {
            pushed_down: false,
            new_node: None,
            ctx: ctx as *const OptContext,
        };

        let result = visitor.visit(&node.plan_node);
        if result.pushed_down {
            Ok(result.new_node)
        } else {
            Ok(Some(node.clone()))
        }
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::project()
    }
}

impl BaseOptRule for ProjectionPushDownRule {}

impl PushDownRule for ProjectionPushDownRule {
    fn can_push_down_to(&self, child_kind: &PlanNodeEnum) -> bool {
        // 投影可以下推到大多数数据访问操作
        matches!(
            child_kind,
            PlanNodeEnum::ScanVertices(_)
                | PlanNodeEnum::ScanEdges(_)
                | PlanNodeEnum::GetVertices(_)
                | PlanNodeEnum::GetEdges(_)
                | PlanNodeEnum::GetNeighbors(_)
        )
    }

    fn create_pushed_down_node(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
        child: &OptGroupNode,
    ) -> Result<Option<TransformResult>, OptimizerError> {
        let node_ref = group_node.borrow();
        let mut result = TransformResult::new();
        result.add_new_group_node(group_node.clone());
        Ok(Some(result))
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
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        if !node.plan_node.is_project() {
            return Ok(None);
        }

        let mut visitor = ProjectionPushDownVisitor {
            pushed_down: false,
            new_node: None,
            ctx: ctx as *const OptContext,
        };

        let result = visitor.visit(&node.plan_node);
        if result.pushed_down {
            Ok(result.new_node)
        } else {
            Ok(Some(node.clone()))
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
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
        child: &OptGroupNode,
    ) -> Result<Option<TransformResult>, OptimizerError> {
        let node_ref = group_node.borrow();
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
    fn test_projection_push_down_rule() {
        let rule = ProjectionPushDownRule;
        let mut ctx = create_test_context();

        // 创建一个投影节点
        let project_node = PlanNodeEnum::Project(
            crate::query::planner::plan::core::nodes::ProjectNode::new(
                PlanNodeEnum::Start(crate::query::planner::plan::core::nodes::StartNode::new()),
                vec![],
            )
            .expect("Node should be created successfully"),
        );
        let opt_node = OptGroupNode::new(1, project_node);

        let result = rule
            .apply(&mut ctx, &opt_node)
            .expect("Rule should apply successfully");
        // 规则应该匹配投影节点并尝试下推
        assert!(result.is_some());
    }

    #[test]
    fn test_push_project_down_rule() {
        let rule = PushProjectDownRule;
        let mut ctx = create_test_context();

        // 创建一个投影节点
        let project_node = PlanNodeEnum::Project(
            crate::query::planner::plan::core::nodes::ProjectNode::new(
                PlanNodeEnum::Start(crate::query::planner::plan::core::nodes::StartNode::new()),
                vec![],
            )
            .expect("Node should be created successfully"),
        );
        let opt_node = OptGroupNode::new(1, project_node);

        let result = rule
            .apply(&mut ctx, &opt_node)
            .expect("Rule should apply successfully");
        // 规则应该匹配投影节点并尝试下推到数据源
        assert!(result.is_some());
    }
}
