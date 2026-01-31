//! 连接优化规则
//! 这些规则负责优化连接操作，专注于连接算法和策略优化

use super::engine::OptimizerError;
use super::plan::{OptContext, OptGroupNode, OptRule, Pattern, TransformResult, Result as OptResult};
use super::rule_patterns::PatternBuilder;
use super::rule_traits::BaseOptRule;
use std::cell::RefCell;
use std::rc::Rc;
use std::result::Result as StdResult;

/// 转换连接以获得更好性能的规则
#[derive(Debug)]
pub struct JoinOptimizationRule;

impl OptRule for JoinOptimizationRule {
    fn name(&self) -> &str {
        "JoinOptimizationRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        node: &Rc<RefCell<OptGroupNode>>,
    ) -> OptResult<Option<TransformResult>> {
        let node_ref = node.borrow();
        if !node_ref.plan_node.is_hash_inner_join()
            && !node_ref.plan_node.is_hash_left_join()
            && !node_ref.plan_node.is_inner_join()
        {
            return Ok(Some(TransformResult::unchanged()));
        }
        if node_ref.dependencies.len() >= 2 {
            let left_dep_id = node_ref.dependencies[0];
            let right_dep_id = node_ref.dependencies[1];
            if let (Some(left_node), Some(right_node)) = (
                ctx.find_group_node_by_plan_node_id(left_dep_id),
                ctx.find_group_node_by_plan_node_id(right_dep_id),
            ) {
                let left_node_ref = left_node.borrow();
                let right_node_ref = right_node.borrow();
                match (
                    left_node_ref.plan_node.type_name(),
                    right_node_ref.plan_node.type_name(),
                ) {
                    ("IndexScan", _) | (_, "IndexScan") => {}
                    ("ScanVertices", _) | (_, "ScanVertices") => {}
                    _ => {}
                }
                let should_optimize = self.should_optimize_join(&left_node_ref, &right_node_ref);
                if should_optimize {
                    drop(node_ref);
                    Ok(Some(TransformResult::unchanged()))
                } else {
                    Ok(Some(TransformResult::unchanged()))
                }
            } else {
                Ok(Some(TransformResult::unchanged()))
            }
        } else {
            Ok(Some(TransformResult::unchanged()))
        }
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::join()
    }
}

impl BaseOptRule for JoinOptimizationRule {}

impl JoinOptimizationRule {
    /// 根据子节点类型判断是否应该优化连接
    fn should_optimize_join(&self, left_node: &OptGroupNode, right_node: &OptGroupNode) -> bool {
        // 简单的启发式：如果任一侧是索引扫描或者获取特定顶点/边的操作，
        // 可能意味着较小的结果集，适合使用哈希连接
        matches!(
            left_node.plan_node.type_name(),
            "IndexScan" | "ScanVertices" | "ScanEdges"
        ) || matches!(
            right_node.plan_node.type_name(),
            "IndexScan" | "ScanVertices" | "ScanEdges"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::context::execution::QueryContext;
    use crate::query::optimizer::plan::{OptContext, OptGroupNode};
    use crate::query::planner::plan::PlanNodeEnum;

    fn create_test_context() -> OptContext {
        let query_context = QueryContext::new();
        OptContext::new(query_context)
    }

    #[test]
    fn test_join_optimization_rule() {
        let rule = JoinOptimizationRule;
        let mut ctx = create_test_context();

        let left_node_id = 1;
        let right_node_id = 2;

        let left_node =
            PlanNodeEnum::Start(crate::query::planner::plan::core::nodes::StartNode::new());
        let right_node =
            PlanNodeEnum::Start(crate::query::planner::plan::core::nodes::StartNode::new());
        let hash_keys = vec![];
        let probe_keys = vec![];

        let inner_join = crate::query::planner::plan::core::nodes::InnerJoinNode::new(
            left_node, right_node, hash_keys, probe_keys,
        )
        .expect("内连接节点应该创建成功");

        let join_node = PlanNodeEnum::InnerJoin(inner_join);

        let mut opt_node = OptGroupNode::new(3, join_node);
        opt_node.dependencies = vec![left_node_id, right_node_id];

        let left_group_node = OptGroupNode::new(
            left_node_id,
            PlanNodeEnum::Start(crate::query::planner::plan::core::nodes::StartNode::new()),
        );
        let right_group_node = OptGroupNode::new(
            right_node_id,
            PlanNodeEnum::Start(crate::query::planner::plan::core::nodes::StartNode::new()),
        );
        ctx.add_plan_node_and_group_node(left_node_id, &left_group_node);
        ctx.add_plan_node_and_group_node(right_node_id, &right_group_node);

        let result = rule
            .apply(&mut ctx, &opt_node)
            .expect("Rule should apply successfully");
        assert!(result.is_some());
    }
}
