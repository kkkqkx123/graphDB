//! 连接优化规则
//!
//! 根据子节点的特征选择最优的连接策略：
//! - 如果一侧是索引扫描，优先使用索引连接
//! - 如果两侧数据量差异大，将小表作为哈希表构建侧
//! - 基于成本模型选择最优的连接算法
//!
//! # 适用条件
//!
//! - 节点是连接节点（InnerJoin、LeftJoin、CrossJoin）
//! - 至少有两个依赖节点

use crate::query::optimizer::plan::{
    OptContext, OptGroupNode, OptRule, Pattern, TransformResult,
};
use crate::query::optimizer::rule_patterns::PatternBuilder;
use crate::query::optimizer::rule_traits::BaseOptRule;
use crate::query::planner::plan::core::nodes::{
    HashInnerJoinNode, HashLeftJoinNode, PlanNodeEnum,
};
use std::cell::RefCell;
use std::rc::Rc;

/// 连接优化规则
///
/// 该规则负责将非哈希连接（InnerJoin、LeftJoin）转换为哈希连接（HashInnerJoin、HashLeftJoin），
/// 并基于成本模型选择最优的连接策略。
///
/// # 转换示例
///
/// Before:
/// ```text
///   InnerJoin
///   /       \
/// Left    Right
/// ```
///
/// After (当数据量较大时):
/// ```text
///   HashInnerJoin
///   /           \
/// Left        Right
/// ```
#[derive(Debug)]
pub struct JoinOptimizationRule;

impl OptRule for JoinOptimizationRule {
    fn name(&self) -> &str {
        "JoinOptimizationRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> Result<Option<TransformResult>, crate::query::optimizer::engine::OptimizerError> {
        let node_ref = group_node.borrow();

        // 只处理非哈希连接节点
        let (left_input, right_input, hash_keys, probe_keys, is_inner) = match &node_ref.plan_node {
            PlanNodeEnum::InnerJoin(join) => (
                join.left_input().clone(),
                join.right_input().clone(),
                join.hash_keys().to_vec(),
                join.probe_keys().to_vec(),
                true,
            ),
            PlanNodeEnum::LeftJoin(join) => (
                join.left_input().clone(),
                join.right_input().clone(),
                join.hash_keys().to_vec(),
                join.probe_keys().to_vec(),
                false,
            ),
            _ => return Ok(None),
        };

        // 检查是否已经是最优形式
        if self.is_already_optimal(&node_ref.plan_node) {
            return Ok(None);
        }

        // 评估连接策略
        let strategy = self.evaluate_join_strategy(ctx, &left_input, &right_input)?;

        match strategy {
            JoinStrategy::HashJoin => {
                // 将 InnerJoin/LeftJoin 转换为 HashInnerJoin/HashLeftJoin
                let new_join_node = if is_inner {
                    PlanNodeEnum::HashInnerJoin(
                        HashInnerJoinNode::new(
                            left_input,
                            right_input,
                            hash_keys,
                            probe_keys,
                        )
                        .map_err(|e| {
                            crate::query::optimizer::engine::OptimizerError::new(
                                format!("创建HashInnerJoinNode失败: {:?}", e),
                                2001,
                            )
                        })?,
                    )
                } else {
                    PlanNodeEnum::HashLeftJoin(
                        HashLeftJoinNode::new(
                            left_input,
                            right_input,
                            hash_keys,
                            probe_keys,
                        )
                        .map_err(|e| {
                            crate::query::optimizer::engine::OptimizerError::new(
                                format!("创建HashLeftJoinNode失败: {:?}", e),
                                2002,
                            )
                        })?,
                    )
                };

                let mut new_group_node = node_ref.clone();
                new_group_node.plan_node = new_join_node;

                let mut result = TransformResult::new();
                result.erase_curr = true;
                result.add_new_group_node(Rc::new(RefCell::new(new_group_node)));

                Ok(Some(result))
            }
            JoinStrategy::KeepOriginal => {
                // 保持原始连接类型
                Ok(None)
            }
        }
    }

    fn pattern(&self) -> Pattern {
        // 匹配 InnerJoin 或 LeftJoin（非哈希连接）
        PatternBuilder::multi(vec!["InnerJoin", "LeftJoin"])
    }
}

impl BaseOptRule for JoinOptimizationRule {}

impl JoinOptimizationRule {
    /// 检查节点是否已经是优化的形式
    fn is_already_optimal(&self, plan_node: &PlanNodeEnum) -> bool {
        plan_node.is_hash_inner_join() || plan_node.is_hash_left_join()
    }

    /// 评估最优的连接策略
    fn evaluate_join_strategy(
        &self,
        ctx: &OptContext,
        left_node: &PlanNodeEnum,
        right_node: &PlanNodeEnum,
    ) -> Result<JoinStrategy, crate::query::optimizer::engine::OptimizerError> {
        // 估算行数
        let left_rows = self.estimate_row_count(ctx, left_node)?;
        let right_rows = self.estimate_row_count(ctx, right_node)?;

        // 决策逻辑：
        // 1. 如果任一侧数据量很小（小于阈值），使用哈希连接
        // 2. 如果两侧数据量都很大，使用哈希连接
        // 3. 如果一侧是索引扫描且另一侧很小，保持原样（可能使用索引连接）

        const SMALL_TABLE_THRESHOLD: u64 = 1000;
        const LARGE_TABLE_THRESHOLD: u64 = 10000;

        let left_is_small = left_rows < SMALL_TABLE_THRESHOLD;
        let right_is_small = right_rows < SMALL_TABLE_THRESHOLD;
        let left_is_large = left_rows > LARGE_TABLE_THRESHOLD;
        let right_is_large = right_rows > LARGE_TABLE_THRESHOLD;

        // 如果一侧是索引扫描，另一侧很小，保持原样
        if (left_node.is_index_scan() && right_is_small)
            || (right_node.is_index_scan() && left_is_small)
        {
            return Ok(JoinStrategy::KeepOriginal);
        }

        // 如果任一侧数据量很小，使用哈希连接（小表作为构建侧）
        if left_is_small || right_is_small {
            return Ok(JoinStrategy::HashJoin);
        }

        // 如果两侧数据量都很大，使用哈希连接
        if left_is_large && right_is_large {
            return Ok(JoinStrategy::HashJoin);
        }

        // 默认使用哈希连接（通常比嵌套循环连接更高效）
        Ok(JoinStrategy::HashJoin)
    }

    /// 估算子树的代价
    /// 此方法当前未使用，但保留作为基于代价优化（CBO）的基础设施
    #[allow(dead_code)]
    fn estimate_subtree_cost(
        &self,
        _ctx: &OptContext,
        node: &PlanNodeEnum,
    ) -> Result<f64, crate::query::optimizer::engine::OptimizerError> {
        // 基于节点类型估算代价
        let cost = match node {
            PlanNodeEnum::ScanVertices(_) => 100.0,
            PlanNodeEnum::ScanEdges(_) => 100.0,
            PlanNodeEnum::IndexScan(_) => 50.0,
            PlanNodeEnum::EdgeIndexScan(_) => 50.0,
            PlanNodeEnum::GetVertices(_) => 80.0,
            PlanNodeEnum::GetEdges(_) => 80.0,
            PlanNodeEnum::GetNeighbors(_) => 80.0,
            PlanNodeEnum::Filter(filter) => {
                // Filter的代价取决于其子节点
                let input_cost = filter
                    .dependencies()
                    .first()
                    .map(|d| self.estimate_subtree_cost(_ctx, d).unwrap_or(100.0))
                    .unwrap_or(100.0);
                input_cost * 1.1 // Filter增加10%的代价
            }
            PlanNodeEnum::Project(project) => {
                let input_cost = project
                    .dependencies()
                    .first()
                    .map(|d| self.estimate_subtree_cost(_ctx, d).unwrap_or(100.0))
                    .unwrap_or(100.0);
                input_cost * 1.05 // Project增加5%的代价
            }
            PlanNodeEnum::Limit(limit) => {
                let input_cost = limit
                    .dependencies()
                    .first()
                    .map(|d| self.estimate_subtree_cost(_ctx, d).unwrap_or(100.0))
                    .unwrap_or(100.0);
                input_cost * 0.5 // Limit减少50%的代价
            }
            PlanNodeEnum::InnerJoin(join) => {
                let left_cost = self.estimate_subtree_cost(_ctx, join.left_input()).unwrap_or(100.0);
                let right_cost =
                    self.estimate_subtree_cost(_ctx, join.right_input()).unwrap_or(100.0);
                left_cost + right_cost + 50.0 // 连接操作的基础代价
            }
            PlanNodeEnum::LeftJoin(join) => {
                let left_cost = self.estimate_subtree_cost(_ctx, join.left_input()).unwrap_or(100.0);
                let right_cost =
                    self.estimate_subtree_cost(_ctx, join.right_input()).unwrap_or(100.0);
                left_cost + right_cost + 50.0
            }
            PlanNodeEnum::HashInnerJoin(join) => {
                let left_cost = self.estimate_subtree_cost(_ctx, join.left_input()).unwrap_or(100.0);
                let right_cost =
                    self.estimate_subtree_cost(_ctx, join.right_input()).unwrap_or(100.0);
                left_cost + right_cost + 30.0 // 哈希连接代价较低
            }
            PlanNodeEnum::HashLeftJoin(join) => {
                let left_cost = self.estimate_subtree_cost(_ctx, join.left_input()).unwrap_or(100.0);
                let right_cost =
                    self.estimate_subtree_cost(_ctx, join.right_input()).unwrap_or(100.0);
                left_cost + right_cost + 30.0
            }
            PlanNodeEnum::CrossJoin(join) => {
                let left_cost = self.estimate_subtree_cost(_ctx, join.left_input()).unwrap_or(100.0);
                let right_cost =
                    self.estimate_subtree_cost(_ctx, join.right_input()).unwrap_or(100.0);
                left_cost * right_cost * 0.1 // 交叉连接代价很高
            }
            _ => 100.0,
        };

        Ok(cost)
    }

    /// 估算子树的行数
    fn estimate_row_count(
        &self,
        _ctx: &OptContext,
        node: &PlanNodeEnum,
    ) -> Result<u64, crate::query::optimizer::engine::OptimizerError> {
        // 基于节点类型估算行数
        let rows = match node {
            PlanNodeEnum::ScanVertices(_) => 10000,
            PlanNodeEnum::ScanEdges(_) => 10000,
            PlanNodeEnum::IndexScan(_) => 100,
            PlanNodeEnum::EdgeIndexScan(_) => 100,
            PlanNodeEnum::GetVertices(_) => 1000,
            PlanNodeEnum::GetEdges(_) => 1000,
            PlanNodeEnum::GetNeighbors(_) => 1000,
            PlanNodeEnum::Filter(_) => 500,      // 过滤后行数减少
            PlanNodeEnum::Project(_) => 1000,    // 投影不改变行数
            PlanNodeEnum::Limit(limit) => {
                // Limit限制行数
                let input_rows = limit
                    .dependencies()
                    .first()
                    .map(|d| self.estimate_row_count(_ctx, d).unwrap_or(1000))
                    .unwrap_or(1000);
                input_rows.min(100) // 假设Limit通常限制在100行以内
            }
            PlanNodeEnum::Dedup(_) => 500,       // 去重后行数减少
            PlanNodeEnum::InnerJoin(join) => {
                let left_rows = self.estimate_row_count(_ctx, join.left_input()).unwrap_or(1000);
                let right_rows = self.estimate_row_count(_ctx, join.right_input()).unwrap_or(1000);
                left_rows.min(right_rows) // 内连接结果通常小于较小的输入
            }
            PlanNodeEnum::LeftJoin(join) => {
                let left_rows = self.estimate_row_count(_ctx, join.left_input()).unwrap_or(1000);
                left_rows // 左连接结果行数等于左表行数
            }
            PlanNodeEnum::HashInnerJoin(join) => {
                let left_rows = self.estimate_row_count(_ctx, join.left_input()).unwrap_or(1000);
                let right_rows = self.estimate_row_count(_ctx, join.right_input()).unwrap_or(1000);
                left_rows.min(right_rows)
            }
            PlanNodeEnum::HashLeftJoin(join) => {
                let left_rows = self.estimate_row_count(_ctx, join.left_input()).unwrap_or(1000);
                left_rows
            }
            PlanNodeEnum::CrossJoin(join) => {
                let left_rows = self.estimate_row_count(_ctx, join.left_input()).unwrap_or(1000);
                let right_rows = self.estimate_row_count(_ctx, join.right_input()).unwrap_or(1000);
                left_rows * right_rows // 交叉连接是笛卡尔积
            }
            _ => 1000,
        };

        Ok(rows)
    }
}

/// 连接策略枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum JoinStrategy {
    /// 使用哈希连接
    HashJoin,
    /// 保持原始连接类型
    KeepOriginal,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::context::execution::QueryContext;
    use crate::query::optimizer::engine::OptimizerError;
    use crate::query::optimizer::plan::{OptContext, OptGroupNode};
    use crate::query::planner::plan::core::nodes::{
        InnerJoinNode, ScanVerticesNode, StartNode,
    };

    fn create_test_context() -> OptContext {
        let query_context = QueryContext::new();
        OptContext::new(query_context)
    }

    fn create_test_start_node() -> PlanNodeEnum {
        PlanNodeEnum::Start(StartNode::new())
    }

    fn create_test_scan_vertices_node() -> PlanNodeEnum {
        PlanNodeEnum::ScanVertices(ScanVerticesNode::new(1))
    }

    #[test]
    fn test_join_optimization_rule_with_scan() -> Result<(), OptimizerError> {
        let rule = JoinOptimizationRule;
        let mut ctx = create_test_context();

        let left_node = create_test_scan_vertices_node();
        let right_node = create_test_scan_vertices_node();
        let hash_keys = vec![];
        let probe_keys = vec![];

        let inner_join =
            InnerJoinNode::new(left_node, right_node, hash_keys, probe_keys).expect("创建内连接节点失败");

        let join_node = PlanNodeEnum::InnerJoin(inner_join);
        let opt_node = OptGroupNode::new(1, join_node);

        let result = rule
            .apply(&mut ctx, &Rc::new(RefCell::new(opt_node)))
            .expect("规则应用应该成功");

        // 扫描节点数据量较大，应该转换为哈希连接
        assert!(
            result.is_some(),
            "应该将InnerJoin转换为HashInnerJoin"
        );

        let transform_result = result.unwrap();
        assert!(transform_result.erase_curr);
        assert_eq!(transform_result.new_group_nodes.len(), 1);

        let new_node = transform_result.new_group_nodes[0].borrow();
        assert!(
            new_node.plan_node.is_hash_inner_join(),
            "新节点应该是HashInnerJoin"
        );

        Ok(())
    }

    #[test]
    fn test_join_optimization_rule_with_small_tables() -> Result<(), OptimizerError> {
        let rule = JoinOptimizationRule;
        let mut ctx = create_test_context();

        // 使用Start节点（模拟小表）
        let left_node = create_test_start_node();
        let right_node = create_test_start_node();
        let hash_keys = vec![];
        let probe_keys = vec![];

        let inner_join =
            InnerJoinNode::new(left_node, right_node, hash_keys, probe_keys).expect("创建内连接节点失败");

        let join_node = PlanNodeEnum::InnerJoin(inner_join);
        let opt_node = OptGroupNode::new(1, join_node);

        let result = rule
            .apply(&mut ctx, &Rc::new(RefCell::new(opt_node)))
            .expect("规则应用应该成功");

        // Start节点数据量小，应该转换为哈希连接
        assert!(
            result.is_some(),
            "应该将InnerJoin转换为HashInnerJoin"
        );

        Ok(())
    }

    #[test]
    fn test_join_optimization_rule_hash_join_not_applied() -> Result<(), OptimizerError> {
        let rule = JoinOptimizationRule;
        let mut ctx = create_test_context();

        // 测试已经是哈希连接的情况
        let left_node = create_test_scan_vertices_node();
        let right_node = create_test_scan_vertices_node();
        let hash_keys = vec![];
        let probe_keys = vec![];

        let hash_inner_join =
            HashInnerJoinNode::new(left_node, right_node, hash_keys, probe_keys)
                .expect("创建哈希内连接节点失败");

        let join_node = PlanNodeEnum::HashInnerJoin(hash_inner_join);
        let opt_node = OptGroupNode::new(1, join_node);

        let result = rule
            .apply(&mut ctx, &Rc::new(RefCell::new(opt_node)))
            .expect("规则应用应该成功");

        // 已经是哈希连接，不应该再转换
        assert!(
            result.is_none(),
            "已经是HashInnerJoin，不应该再转换"
        );

        Ok(())
    }

    #[test]
    fn test_estimate_subtree_cost() {
        let rule = JoinOptimizationRule;
        let ctx = create_test_context();

        let scan_node = create_test_scan_vertices_node();
        let cost = rule
            .estimate_subtree_cost(&ctx, &scan_node)
            .expect("估算代价应该成功");

        assert_eq!(cost, 100.0, "扫描节点的代价应该是100.0");
    }

    #[test]
    fn test_estimate_row_count() {
        let rule = JoinOptimizationRule;
        let ctx = create_test_context();

        let scan_node = create_test_scan_vertices_node();
        let rows = rule
            .estimate_row_count(&ctx, &scan_node)
            .expect("估算行数应该成功");

        assert_eq!(rows, 10000, "扫描节点的行数应该是10000");
    }
}
