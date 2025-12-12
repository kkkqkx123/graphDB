//! 连接优化规则
//! 这些规则负责优化连接操作，专注于连接算法和策略优化

use super::optimizer::OptimizerError;
use super::rule_traits::{BaseOptRule};
use super::rule_patterns::PatternBuilder;
use crate::query::optimizer::optimizer::{OptContext, OptGroupNode, OptRule, Pattern};
use crate::query::planner::plan::{PlanNodeKind, PlanNode};

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
        if node.plan_node.kind() != PlanNodeKind::InnerJoin &&
           node.plan_node.kind() != PlanNodeKind::HashInnerJoin &&
           node.plan_node.kind() != PlanNodeKind::HashLeftJoin {
            return Ok(None);
        }

        // 在完整实现中，这会分析连接并可能
        // 基于数据特征将其转换为更高效的连接算法
        if let Some(matched) = self.match_pattern(ctx, node)? {
            if matched.dependencies.len() >= 2 {
                // 分析连接输入的大小以确定最佳连接策略
                // 例如，如果一侧小得多，我们可能选择哈希连接
                // 在完整实现中，我们会检查输入的估计行数
                // 目前，我们只返回原始节点
                Ok(Some(node.clone()))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::context::QueryContext;
    use crate::query::optimizer::optimizer::{OptContext, OptGroupNode};
    use crate::query::planner::plan::{Limit};
    use crate::query::planner::plan::{PlanNode, PlanNodeKind};

    fn create_test_context() -> OptContext {
        OptContext::new(QueryContext::default())
    }

    #[test]
    fn test_join_optimization_rule() {
        let rule = JoinOptimizationRule;
        let mut ctx = create_test_context();

        // 创建一个连接节点（使用Limit作为占位符，因为我们没有特定的连接结构）
        let join_node = Box::new(Limit::new(1, 10, 0));
        let opt_node = OptGroupNode::new(1, join_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        assert!(result.is_some());
    }
}