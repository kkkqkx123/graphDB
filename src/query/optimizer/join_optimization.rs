//! 连接优化规则
//! 这些规则负责优化连接操作，专注于连接算法和策略优化

use super::optimizer::OptimizerError;
use super::rule_patterns::PatternBuilder;
use super::rule_traits::BaseOptRule;
use crate::query::optimizer::optimizer::{OptContext, OptGroupNode, OptRule, Pattern};
use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;

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
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 检查是否为连接操作
        if !node.plan_node.is_hash_inner_join()
            && !node.plan_node.is_hash_left_join()
            && !node.plan_node.is_inner_join()
        {
            return Ok(None);
        }

        // 分析连接并可能基于数据特征将其转换为更高效的连接算法
        if node.dependencies.len() >= 2 {
            // 获取连接的左右子节点
            let left_dep_id = node.dependencies[0];
            let right_dep_id = node.dependencies[1];

            if let (Some(left_node), Some(right_node)) = (
                ctx.find_group_node_by_plan_node_id(left_dep_id),
                ctx.find_group_node_by_plan_node_id(right_dep_id),
            ) {
                // 在实际实现中，我们会评估左右子树的大小
                // 以决定是否需要更改连接算法
                // 例如，如果右表很小，我们可能希望转换为HashJoin
                // 或者如果左表很小，可以考虑SwapJoin并转为HashJoin

                // 简单的启发式：如果右子树有GetVertices或IndexScan等操作，
                // 而且看起来结果集较小，我们可能考虑哈希连接
                match (
                    left_node.plan_node.type_name(),
                    right_node.plan_node.type_name(),
                ) {
                    ("IndexScan", _) | (_, "IndexScan") => {
                        // 如果任一侧是索引扫描，这可能意味着较小的结果集
                        // 根据具体情况，我们可以优化连接策略
                    }
                    ("ScanVertices", _) | (_, "ScanVertices") => {
                        // 类似地，ScanVertices可能返回较小结果集
                    }
                    _ => {
                        // 其他情况，保持原连接计划
                    }
                }

                // 基于子节点类型决定是否优化连接算法
                let should_optimize = self.should_optimize_join(left_node, right_node);

                if should_optimize {
                    // 在实际实现中，我们可能会根据估计的行数选择最合适的连接算法
                    // 例如，如果一侧非常小，使用哈希连接；如果两侧都很大，使用嵌套循环或排序合并连接
                    // 这里我们只是示例，返回当前节点
                    Ok(Some(node.clone()))
                } else {
                    Ok(Some(node.clone())) // 不需要优化，返回原节点
                }
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
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
    use crate::core::context::QueryContext;
    use crate::query::optimizer::optimizer::{OptContext, OptGroupNode};
    use crate::query::planner::plan::core::nodes::LimitNode;

    fn create_test_context() -> OptContext {
        let session_info = crate::core::context::session::SessionInfo::new(
            "test_session",
            "test_user",
            vec!["user".to_string()],
            "127.0.0.1",
            8080,
            "test_client",
            "test_connection",
        );
        let query_context = QueryContext::new(
            "test_query",
            crate::core::context::query::QueryType::DataQuery,
            "TEST QUERY",
            session_info,
        );
        OptContext::new(query_context)
    }

    #[test]
    fn test_join_optimization_rule() {
        let rule = JoinOptimizationRule;
        let mut ctx = create_test_context();

        // 创建一个连接节点（使用HashInnerJoin作为测试）
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
        let opt_node = OptGroupNode::new(1, join_node);

        let result = rule
            .apply(&mut ctx, &opt_node)
            .expect("Rule should apply successfully");
        assert!(result.is_some());
    }
}
