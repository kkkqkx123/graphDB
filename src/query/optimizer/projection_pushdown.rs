//! 投影下推优化规则
//! 这些规则负责将投影操作下推到计划树的底层，以减少数据传输量

use super::optimizer::OptimizerError;
use super::rule_patterns::PatternBuilder;
use super::rule_traits::{BaseOptRule, PushDownRule};
use crate::query::optimizer::optimizer::{OptContext, OptGroupNode, OptRule, Pattern};
use crate::query::planner::plan::core::nodes::PlanNodeEnum;

/// 投影下推规则
#[derive(Debug)]
pub struct ProjectionPushDownRule;

impl OptRule for ProjectionPushDownRule {
    fn name(&self) -> &str {
        "ProjectionPushDownRule"
    }

    fn apply(
        &self,
        _ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        if node.plan_node.is_project() {
            Ok(Some(node.clone()))
        } else {
            Ok(None)
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
        _ctx: &mut OptContext,
        _node: &OptGroupNode,
        _child: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 在完整实现中，这里会创建下推后的节点
        // 目前简化实现，返回None
        Ok(None)
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
        _ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 在完整实现中，这会将投影操作下推
        // 更接近数据源以减少数据传输
        if node.plan_node.is_project() {
            Ok(Some(node.clone()))
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
        _node: &OptGroupNode,
        _child: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 在完整实现中，这里会创建下推后的节点
        // 目前简化实现，返回None
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::context::execution::QueryContext;
    use crate::query::optimizer::optimizer::{OptContext, OptGroupNode};
    use crate::query::planner::plan::core::nodes::ProjectNode;

    fn create_test_context() -> OptContext {
        let session_info = crate::api::session::session_manager::SessionInfo {
            session_id: 1,
            user_name: "test_user".to_string(),
            space_name: None,
            graph_addr: None,
            create_time: std::time::SystemTime::now(),
            last_access_time: std::time::SystemTime::now(),
            active_queries: 0,
            timezone: None,
        };
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
