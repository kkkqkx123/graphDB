//! 连接器模块
//!
//! 提供计划节点之间的连接功能，包括内连接、左连接和输入添加

use crate::query::QueryContext;
use crate::query::planner::plan::{PlanNodeEnum, SubPlan};
use crate::query::planner::planner::PlannerError;
use std::collections::HashSet;

/// 计划连接器
///
/// 用于连接两个子计划，类似于 C++ 实现中的 SegmentsConnector
pub struct SegmentsConnector;

impl SegmentsConnector {
    /// 创建内连接
    ///
    /// 将两个计划进行内连接，使用指定的连接键
    pub fn inner_join(
        _qctx: &QueryContext,
        left: SubPlan,
        right: SubPlan,
        _inter_aliases: HashSet<&str>,
    ) -> Result<SubPlan, PlannerError> {
        let left_root = match left.root {
            Some(ref r) => r,
            None => return Ok(right),
        };

        let right_root = match right.root {
            Some(ref r) => r,
            None => return Ok(left),
        };

        let _col_names = left_root.col_names().to_vec();
        let join_node = PlanNodeEnum::InnerJoin(
            crate::query::planner::plan::core::nodes::InnerJoinNode::new(
                left_root.clone(),
                right_root.clone(),
                vec![],
                vec![],
            )
            .map_err(|e| PlannerError::JoinFailed(format!("内连接节点创建失败: {}", e)))?,
        );

        Ok(SubPlan {
            root: Some(join_node),
            tail: left.tail.or(right.tail),
        })
    }

    /// 创建左连接
    ///
    /// 将两个计划进行左连接，用于可选 MATCH 等场景
    pub fn left_join(
        _qctx: &QueryContext,
        left: SubPlan,
        right: SubPlan,
        _inter_aliases: HashSet<&str>,
    ) -> Result<SubPlan, PlannerError> {
        let left_root = match left.root {
            Some(ref r) => r,
            None => return Ok(right),
        };

        let right_root = match right.root {
            Some(ref r) => r,
            None => return Ok(left),
        };

        let join_node = PlanNodeEnum::LeftJoin(
            crate::query::planner::plan::core::nodes::LeftJoinNode::new(
                left_root.clone(),
                right_root.clone(),
                vec![],
                vec![],
            )
            .map_err(|e| PlannerError::JoinFailed(format!("左连接节点创建失败: {}", e)))?,
        );

        Ok(SubPlan {
            root: Some(join_node),
            tail: left.tail.or(right.tail),
        })
    }

    /// 添加输入
    ///
    /// 将一个计划作为另一个计划的输入
    pub fn add_input(input_plan: SubPlan, dependent_plan: SubPlan, _is_left: bool) -> SubPlan {
        SubPlan {
            root: dependent_plan.root,
            tail: input_plan.tail,
        }
    }

    /// 创建交叉连接
    ///
    /// 将两个计划进行笛卡尔积连接
    pub fn cross_join(left: SubPlan, right: SubPlan) -> Result<SubPlan, PlannerError> {
        let left_root = match left.root {
            Some(ref r) => r,
            None => return Ok(right),
        };

        let right_root = match right.root {
            Some(ref r) => r,
            None => return Ok(left),
        };

        let join_node = PlanNodeEnum::CrossJoin(
            crate::query::planner::plan::core::nodes::CrossJoinNode::new(
                left_root.clone(),
                right_root.clone(),
            )
            .map_err(|e| PlannerError::JoinFailed(format!("交叉连接节点创建失败: {}", e)))?,
        );

        Ok(SubPlan {
            root: Some(join_node),
            tail: left.tail.or(right.tail),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::session::{RequestContext, RequestParams};
    use std::sync::Arc;

    fn create_test_query_context() -> QueryContext {
        let request_params = RequestParams::new("TEST".to_string());
        let rctx = Arc::new(RequestContext::new(None, request_params));
        QueryContext::new(rctx)
    }

    #[test]
    fn test_inner_join() {
        let left = SubPlan::from_single_node(PlanNodeEnum::Start(
            crate::query::planner::plan::core::nodes::StartNode::new(),
        ));
        let right = SubPlan::from_single_node(PlanNodeEnum::Start(
            crate::query::planner::plan::core::nodes::StartNode::new(),
        ));

        let result = SegmentsConnector::inner_join(&create_test_query_context(), left, right, HashSet::new());
        assert!(result.is_ok());
        assert!(result.expect("Expected planner result to exist").root.is_some());
    }

    #[test]
    fn test_left_join() {
        let left = SubPlan::from_single_node(PlanNodeEnum::Start(
            crate::query::planner::plan::core::nodes::StartNode::new(),
        ));
        let right = SubPlan::from_single_node(PlanNodeEnum::Start(
            crate::query::planner::plan::core::nodes::StartNode::new(),
        ));

        let result = SegmentsConnector::left_join(&create_test_query_context(), left, right, HashSet::new());
        assert!(result.is_ok());
        assert!(result.expect("Expected planner result to exist").root.is_some());
    }

    #[test]
    fn test_cross_join() {
        let left = SubPlan::from_single_node(PlanNodeEnum::Start(
            crate::query::planner::plan::core::nodes::StartNode::new(),
        ));
        let right = SubPlan::from_single_node(PlanNodeEnum::Start(
            crate::query::planner::plan::core::nodes::StartNode::new(),
        ));

        let result = SegmentsConnector::cross_join(left, right);
        assert!(result.is_ok());
        assert!(result.expect("Expected planner result to exist").root.is_some());
    }

    #[test]
    fn test_add_input() {
        let input_plan = SubPlan::from_single_node(PlanNodeEnum::Start(
            crate::query::planner::plan::core::nodes::StartNode::new(),
        ));
        let dependent_plan = SubPlan::from_single_node(PlanNodeEnum::Start(
            crate::query::planner::plan::core::nodes::StartNode::new(),
        ));

        let result = SegmentsConnector::add_input(input_plan, dependent_plan, true);
        assert!(result.root.is_some());
    }

    #[test]
    fn test_inner_join_with_empty_left() {
        let left = SubPlan::new(None, None);
        let right = SubPlan::from_single_node(PlanNodeEnum::Start(
            crate::query::planner::plan::core::nodes::StartNode::new(),
        ));

        let result = SegmentsConnector::inner_join(&create_test_query_context(), left, right, HashSet::new());
        assert!(result.is_ok());
        assert!(result.expect("Expected planner result to exist").root.is_some());
    }

    #[test]
    fn test_cross_join_with_empty_right() {
        let left = SubPlan::from_single_node(PlanNodeEnum::Start(
            crate::query::planner::plan::core::nodes::StartNode::new(),
        ));
        let right = SubPlan::new(None, None);

        let result = SegmentsConnector::cross_join(left, right);
        assert!(result.is_ok());
        assert!(result.expect("Expected planner result to exist").root.is_some());
    }
}
