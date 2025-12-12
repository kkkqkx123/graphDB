//! 索引优化规则
//! 这些规则负责优化索引操作，包括基于过滤条件的索引扫描优化和索引扫描操作本身的优化

use super::optimizer::OptimizerError;
use super::rule_patterns::PatternBuilder;
use super::rule_traits::BaseOptRule;
use crate::query::optimizer::optimizer::{OptContext, OptGroupNode, OptRule, Pattern};
use crate::query::planner::plan::{IndexScan as IndexScanPlanNode, PlanNode, PlanNodeKind};

/// 基于过滤条件优化边索引扫描的规则
#[derive(Debug)]
pub struct OptimizeEdgeIndexScanByFilterRule;

impl OptRule for OptimizeEdgeIndexScanByFilterRule {
    fn name(&self) -> &str {
        "OptimizeEdgeIndexScanByFilterRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 检查是否为索引扫描操作
        if node.plan_node.kind() != PlanNodeKind::IndexScan {
            return Ok(None);
        }

        // 查找依赖中的过滤操作
        if node.dependencies.len() >= 1 {
            for dep_id in &node.dependencies {
                if let Some(dep_node) = ctx.find_group_node_by_plan_node_id(*dep_id) {
                    if dep_node.plan_node.kind() == PlanNodeKind::Filter {
                        // 检查过滤条件是否可以推入到索引扫描中
                        if let Some(filter_node) = dep_node
                            .plan_node
                            .as_any()
                            .downcast_ref::<crate::query::planner::plan::operations::Filter>(
                        ) {
                            // 在完整实现中，我们会将过滤条件合并到索引扫描中
                            // 以减少从索引检索的行数
                            // 这里我们简单地返回当前节点，实际实现中需要修改索引扫描计划
                            return Ok(Some(node.clone()));
                        }
                    }
                }
            }
        }
        Ok(None)
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::index_scan() // 专门用于边索引扫描
    }
}

impl BaseOptRule for OptimizeEdgeIndexScanByFilterRule {}

/// 基于过滤条件优化标签索引扫描的规则
#[derive(Debug)]
pub struct OptimizeTagIndexScanByFilterRule;

impl OptRule for OptimizeTagIndexScanByFilterRule {
    fn name(&self) -> &str {
        "OptimizeTagIndexScanByFilterRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 检查是否为索引扫描操作
        if node.plan_node.kind() != PlanNodeKind::IndexScan {
            return Ok(None);
        }

        // 查找依赖中的过滤操作
        if node.dependencies.len() >= 1 {
            for dep_id in &node.dependencies {
                if let Some(dep_node) = ctx.find_group_node_by_plan_node_id(*dep_id) {
                    if dep_node.plan_node.kind() == PlanNodeKind::Filter {
                        // 检查过滤条件是否可以推入到索引扫描中
                        if let Some(filter_node) = dep_node
                            .plan_node
                            .as_any()
                            .downcast_ref::<crate::query::planner::plan::operations::Filter>(
                        ) {
                            // 在完整实现中，我们会将过滤条件合并到索引扫描中
                            // 以减少从索引检索的行数
                            // 这里我们简单地返回当前节点，实际实现中需要修改索引扫描计划
                            return Ok(Some(node.clone()));
                        }
                    }
                }
            }
        }
        Ok(None)
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::index_scan() // 专门用于标签索引扫描
    }
}

impl BaseOptRule for OptimizeTagIndexScanByFilterRule {}

/// 转换边索引全扫描为更优操作的规则
#[derive(Debug)]
pub struct EdgeIndexFullScanRule;

impl OptRule for EdgeIndexFullScanRule {
    fn name(&self) -> &str {
        "EdgeIndexFullScanRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 检查是否为可能是全扫描的索引扫描操作
        if node.plan_node.kind() != PlanNodeKind::IndexScan {
            return Ok(None);
        }

        // 检查是否没有有效的过滤条件，这可能意味着全扫描
        // 在完整实现中，我们需要检查索引扫描的条件
        // 如果索引扫描是全扫描（没有有效过滤条件），可能转换为其他操作
        if let Some(index_scan_node) = node.plan_node.as_any().downcast_ref::<IndexScanPlanNode>() {
            // 如果索引扫描没有有效的过滤条件，可能是全扫描
            if !index_scan_node.has_effective_filter() {
                // 根据具体情况，我们可能将其转换为更高效的操作
                // 简单起见，目前我们返回原节点
                return Ok(Some(node.clone()));
            }
        }
        Ok(None)
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::index_scan() // 专门用于边索引扫描
    }
}

impl BaseOptRule for EdgeIndexFullScanRule {}

/// 转换标签索引全扫描为更优操作的规则
#[derive(Debug)]
pub struct TagIndexFullScanRule;

impl OptRule for TagIndexFullScanRule {
    fn name(&self) -> &str {
        "TagIndexFullScanRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 检查是否为可能是全扫描的索引扫描操作
        if node.plan_node.kind() != PlanNodeKind::IndexScan {
            return Ok(None);
        }

        // 检查是否没有有效的过滤条件，这可能意味着全扫描
        // 在完整实现中，我们需要检查索引扫描的条件
        // 如果索引扫描是全扫描（没有有效过滤条件），可能转换为其他操作
        if let Some(index_scan_node) = node.plan_node.as_any().downcast_ref::<IndexScanPlanNode>() {
            // 如果索引扫描没有有效的过滤条件，可能是全扫描
            if !index_scan_node.has_effective_filter() {
                // 根据具体情况，我们可能将其转换为更高效的操作
                // 简单起见，目前我们返回原节点
                return Ok(Some(node.clone()));
            }
        }
        Ok(None)
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::index_scan() // 专门用于标签索引扫描
    }
}

impl BaseOptRule for TagIndexFullScanRule {}

/// 通用索引扫描操作的规则
#[derive(Debug)]
pub struct IndexScanRule;

impl OptRule for IndexScanRule {
    fn name(&self) -> &str {
        "IndexScanRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 检查是否为索引扫描操作
        if node.plan_node.kind() != PlanNodeKind::IndexScan {
            return Ok(None);
        }

        // 在完整实现中，我们会基于各种因素优化索引扫描：
        // - 索引选择性
        // - 数据分布
        // - 可用内存
        // 这里，我们基于NebulaGraph的IndexScanRule实现，检查索引扫描的查询上下文
        if let Some(index_scan_node) = node.plan_node.as_any().downcast_ref::<IndexScanPlanNode>() {
            // 实际优化逻辑可能会根据索引条件创建更优化的索引扫描计划
            // 暂时返回当前节点
            Ok(Some(node.clone()))
        } else {
            Ok(None)
        }
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::index_scan()
    }
}

impl BaseOptRule for IndexScanRule {}

/// 边索引扫描的UNION ALL规则
#[derive(Debug)]
pub struct UnionAllEdgeIndexScanRule;

impl OptRule for UnionAllEdgeIndexScanRule {
    fn name(&self) -> &str {
        "UnionAllEdgeIndexScanRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 检查是否为作为UNION一部分的索引扫描操作
        if node.plan_node.kind() != PlanNodeKind::IndexScan {
            return Ok(None);
        }

        // 在完整实现中，我们会优化涉及边索引扫描的UNION ALL操作
        // 通过可能合并或重新排序它们
        // 这里我们检查节点是否有多个依赖（表示UNION操作）
        if node.dependencies.len() > 1 {
            // 这可能是一个UNION ALL操作，我们可以尝试优化
            // 暂时返回当前节点
            Ok(Some(node.clone()))
        } else {
            Ok(None)
        }
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::index_scan() // 用于边索引扫描的UNION ALL
    }
}

impl BaseOptRule for UnionAllEdgeIndexScanRule {}

/// 标签索引扫描的UNION ALL规则
#[derive(Debug)]
pub struct UnionAllTagIndexScanRule;

impl OptRule for UnionAllTagIndexScanRule {
    fn name(&self) -> &str {
        "UnionAllTagIndexScanRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 检查是否为作为UNION一部分的索引扫描操作
        if node.plan_node.kind() != PlanNodeKind::IndexScan {
            return Ok(None);
        }

        // 在完整实现中，我们会优化涉及标签索引扫描的UNION ALL操作
        // 通过可能合并或重新排序它们
        // 这里我们检查节点是否有多个依赖（表示UNION操作）
        if node.dependencies.len() > 1 {
            // 这可能是一个UNION ALL操作，我们可以尝试优化
            // 暂时返回当前节点
            Ok(Some(node.clone()))
        } else {
            Ok(None)
        }
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::index_scan() // 用于标签索引扫描的UNION ALL
    }
}

impl BaseOptRule for UnionAllTagIndexScanRule {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::context::QueryContext;
    use crate::query::optimizer::optimizer::{OptContext, OptGroupNode};
    use crate::query::planner::plan::{IndexScan, Limit};
    use crate::query::planner::plan::{PlanNode, PlanNodeKind};

    fn create_test_context() -> OptContext {
        OptContext::new(QueryContext::default())
    }

    #[test]
    fn test_optimize_edge_index_scan_by_filter_rule() {
        let rule = OptimizeEdgeIndexScanByFilterRule;
        let mut ctx = create_test_context();

        // 创建一个索引扫描节点
        let index_scan_node = Box::new(IndexScan::new(1, 1, 2, 3, "RANGE"));
        let opt_node = OptGroupNode::new(1, index_scan_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_optimize_tag_index_scan_by_filter_rule() {
        let rule = OptimizeTagIndexScanByFilterRule;
        let mut ctx = create_test_context();

        // 创建一个索引扫描节点
        let index_scan_node = Box::new(IndexScan::new(1, 1, 2, 3, "RANGE"));
        let opt_node = OptGroupNode::new(1, index_scan_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_edge_index_full_scan_rule() {
        let rule = EdgeIndexFullScanRule;
        let mut ctx = create_test_context();

        // 创建一个索引扫描节点
        let index_scan_node = Box::new(IndexScan::new(1, 1, 2, 3, "RANGE"));
        let opt_node = OptGroupNode::new(1, index_scan_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_tag_index_full_scan_rule() {
        let rule = TagIndexFullScanRule;
        let mut ctx = create_test_context();

        // 创建一个索引扫描节点
        let index_scan_node = Box::new(IndexScan::new(1, 1, 2, 3, "RANGE"));
        let opt_node = OptGroupNode::new(1, index_scan_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_index_scan_rule() {
        let rule = IndexScanRule;
        let mut ctx = create_test_context();

        // 创建一个索引扫描节点
        let index_scan_node = Box::new(IndexScan::new(1, 1, 2, 3, "RANGE"));
        let opt_node = OptGroupNode::new(1, index_scan_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_union_all_edge_index_scan_rule() {
        let rule = UnionAllEdgeIndexScanRule;
        let mut ctx = create_test_context();

        // 创建一个索引扫描节点
        let index_scan_node = Box::new(IndexScan::new(1, 1, 2, 3, "RANGE"));
        let opt_node = OptGroupNode::new(1, index_scan_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_union_all_tag_index_scan_rule() {
        let rule = UnionAllTagIndexScanRule;
        let mut ctx = create_test_context();

        // 创建一个索引扫描节点
        let index_scan_node = Box::new(IndexScan::new(1, 1, 2, 3, "RANGE"));
        let opt_node = OptGroupNode::new(1, index_scan_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        assert!(result.is_some());
    }
}
