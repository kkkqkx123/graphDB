//! 连接优化规则
//! 这些规则负责优化连接操作，专注于连接算法和策略优化

use super::optimizer::OptimizerError;
use super::rule_patterns::PatternBuilder;
use super::rule_traits::BaseOptRule;
use crate::query::optimizer::optimizer::{OptContext, OptGroupNode, OptRule, Pattern};
use crate::query::planner::plan::PlanNodeKind;

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
        if node.plan_node.kind() != PlanNodeKind::InnerJoin
            && node.plan_node.kind() != PlanNodeKind::HashInnerJoin
            && node.plan_node.kind() != PlanNodeKind::HashLeftJoin
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
                match (left_node.plan_node.kind(), right_node.plan_node.kind()) {
                    (PlanNodeKind::IndexScan, _) | (_, PlanNodeKind::IndexScan) => {
                        // 如果任一侧是索引扫描，这可能意味着较小的结果集
                        // 根据具体情况，我们可以优化连接策略
                    }
                    (PlanNodeKind::GetVertices, _) | (_, PlanNodeKind::GetVertices) => {
                        // 类似地，GetVertices可能返回较小结果集
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
            left_node.plan_node.kind(),
            PlanNodeKind::IndexScan | PlanNodeKind::GetVertices | PlanNodeKind::GetEdges
        ) || matches!(
            right_node.plan_node.kind(),
            PlanNodeKind::IndexScan | PlanNodeKind::GetVertices | PlanNodeKind::GetEdges
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::context::QueryContext;
    use crate::query::optimizer::optimizer::{OptContext, OptGroupNode};
    use crate::query::planner::plan::core::nodes::LimitNode;

    fn create_test_context() -> OptContext {
        OptContext::new(QueryContext::default())
    }

    #[test]
    fn test_join_optimization_rule() {
        let rule = JoinOptimizationRule;
        let mut ctx = create_test_context();

        // 创建一个连接节点（使用Limit作为占位符，因为我们没有特定的连接结构）
        let join_node = std::sync::Arc::new(
            LimitNode::new(
                std::sync::Arc::new(crate::query::planner::plan::core::nodes::StartNode::new()),
                10,
                0,
            )
            .expect("Limit node should be created successfully"),
        )
            as std::sync::Arc<dyn crate::query::planner::plan::core::plan_node_traits::PlanNode>;
        let opt_node = OptGroupNode::new(1, join_node);

        let result = rule.apply(&mut ctx, &opt_node).expect("Rule should apply successfully");
        assert!(result.is_some());
    }
}
