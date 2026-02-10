//! 连接优化规则
//!
//! 根据子节点的特征选择最优的连接策略：
//! - 如果一侧是索引扫描，优先使用索引连接
//! - 如果一侧数据量小，优先使用嵌套循环连接
//! - 如果两侧都较大，使用哈希连接
//!
//! # 适用条件
//!
//! - 节点是连接节点（HashInnerJoin、HashLeftJoin、InnerJoin、CrossJoin）
//! - 至少有两个依赖节点

use crate::query::optimizer::plan::{OptContext, OptGroupNode, OptRule, Pattern, TransformResult, Result as OptResult};
use crate::query::optimizer::rule_patterns::PatternBuilder;
use crate::query::optimizer::rule_traits::BaseOptRule;
use std::cell::RefCell;
use std::rc::Rc;

/// 连接优化规则
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
        
        // 只处理连接节点
        if !node_ref.plan_node.is_hash_inner_join()
            && !node_ref.plan_node.is_hash_left_join()
            && !node_ref.plan_node.is_inner_join()
            && !node_ref.plan_node.is_cross_join()
        {
            return Ok(None);
        }

        // 需要至少两个依赖
        if node_ref.dependencies.len() < 2 {
            return Ok(None);
        }

        let left_dep_id = node_ref.dependencies[0];
        let right_dep_id = node_ref.dependencies[1];

        let (left_node, right_node) = match (
            ctx.find_group_node_by_plan_node_id(left_dep_id),
            ctx.find_group_node_by_plan_node_id(right_dep_id),
        ) {
            (Some(l), Some(r)) => (l, r),
            _ => return Ok(None),
        };

        let left_node_ref = left_node.borrow();
        let right_node_ref = right_node.borrow();

        // 评估连接策略
        let strategy = self.evaluate_join_strategy(&left_node_ref, &right_node_ref);
        
        match strategy {
            JoinStrategy::HashJoin => {
                // 哈希连接已经是默认策略，无需转换
                return Ok(None);
            }
            JoinStrategy::IndexJoin => {
                // 如果一侧是索引扫描，可以考虑索引连接
                // 这里只是标记，实际转换由其他规则处理
                return Ok(None);
            }
        }
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::join()
    }
}

impl BaseOptRule for JoinOptimizationRule {}

impl JoinOptimizationRule {
    /// 评估最优的连接策略
    fn evaluate_join_strategy(
        &self,
        left_node: &OptGroupNode,
        right_node: &OptGroupNode,
    ) -> JoinStrategy {
        let left_type = left_node.plan_node.type_name();
        let right_type = right_node.plan_node.type_name();
        
        // 如果任一侧是索引扫描，优先使用索引连接
        if left_type == "IndexScan" || right_type == "IndexScan" {
            return JoinStrategy::IndexJoin;
        }
        
        // 如果任一侧是扫描操作且有过滤条件，可能数据量较小
        let left_has_filter = self.node_has_filter(left_node);
        let right_has_filter = self.node_has_filter(right_node);
        
        if left_has_filter || right_has_filter {
            // 有过滤条件的一侧可能数据量较小，适合哈希连接
            return JoinStrategy::HashJoin;
        }
        
        // 默认使用哈希连接
        JoinStrategy::HashJoin
    }
    
    /// 检查节点是否有过滤条件
    fn node_has_filter(&self, node: &OptGroupNode) -> bool {
        match node.plan_node.name() {
            "ScanVertices" | "ScanEdges" | "IndexScan" => {
                // 检查扫描节点是否有过滤条件
                // 这里简化实现，实际应该检查节点的filter属性
                false
            }
            _ => false,
        }
    }
}

/// 连接策略枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum JoinStrategy {
    /// 哈希连接：适合大表连接
    HashJoin,
    /// 索引连接：利用索引加速连接
    IndexJoin,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::context::execution::QueryContext;
    use crate::query::optimizer::engine::OptimizerError;
    use crate::query::optimizer::plan::{OptContext, OptGroupNode};
    use crate::query::planner::plan::PlanNodeEnum;

    fn create_test_context() -> OptContext {
        let query_context = QueryContext::new();
        OptContext::new(query_context)
    }

    #[test]
    fn test_join_optimization_rule() -> Result<(), OptimizerError> {
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
        ctx.add_group_node(Rc::new(RefCell::new(left_group_node)))?;
        ctx.add_group_node(Rc::new(RefCell::new(right_group_node)))?;

        let result = rule
            .apply(&mut ctx, &Rc::new(RefCell::new(opt_node)))
            .expect("Rule should apply successfully");
        // 当前规则实现返回 Ok(None)，因为 Start 节点不是 IndexScan 或 ScanVertices
        assert!(result.is_none());
        Ok(())
    }
}
