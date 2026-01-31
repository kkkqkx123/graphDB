//! 扫描优化规则
//! 这些规则负责优化扫描操作，如带过滤条件的扫描和索引全扫描优化

use super::engine::OptimizerError;
use super::plan::{OptContext, OptGroupNode, OptRule, Pattern};
use super::rule_patterns::PatternBuilder;
use super::rule_traits::BaseOptRule;

/// 优化索引全扫描为更高效的全表扫描的规则
#[derive(Debug)]
pub struct IndexFullScanRule;

impl OptRule for IndexFullScanRule {
    fn name(&self) -> &str {
        "IndexFullScanRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 检查是否为索引扫描操作
        if !node.plan_node.is_index_scan() {
            return Ok(None);
        }

        // 在完整实现中，这会确定何时从索引扫描切换到全扫描
        // 基于估计的选择性、数据分布等
        if let Some(_matched) = self.match_pattern(ctx, node)? {
            // 从索引扫描切换到全扫描的决策将基于：
            // - 索引条件的选择性
            // - 表的大小
            // - 索引查找的成本与全扫描的成本
            // 目前，我们只返回原始节点
            Ok(Some(node.clone()))
        } else {
            Ok(None)
        }
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::index_scan()
    }
}

impl BaseOptRule for IndexFullScanRule {}

/// 优化带过滤条件的扫描操作的规则
#[derive(Debug)]
pub struct ScanWithFilterOptimizationRule;

impl OptRule for ScanWithFilterOptimizationRule {
    fn name(&self) -> &str {
        "ScanWithFilterOptimizationRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 检查是否为扫描操作
        if !node.plan_node.is_scan_vertices() && !node.plan_node.is_scan_edges() {
            return Ok(None);
        }

        // 匹配模式以检查我们是否有扫描上方的过滤
        if let Some(matched) = self.match_pattern(ctx, node)? {
            if matched.dependencies.len() >= 1 {
                // 在依赖中查找可以推入扫描的过滤操作
                for dep in &matched.dependencies {
                    if dep.borrow().plan_node.is_filter() {
                        // 在完整实现中，我们会将过滤条件合并到扫描中
                        // 以减少处理的行数
                        break; // 只检查是否有过滤
                    }
                }
                Ok(Some(node.clone()))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::with_dependency("ScanVertices", "Filter")
    }
}

impl BaseOptRule for ScanWithFilterOptimizationRule {}

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
    fn test_index_full_scan_rule() {
        let rule = IndexFullScanRule;
        let mut ctx = create_test_context();

        let index_scan_node =
            crate::query::planner::plan::algorithms::IndexScan::new(1, 1, 1, 1, "RANGE");
        let index_scan_enum = PlanNodeEnum::IndexScan(index_scan_node);

        let opt_node = OptGroupNode::new(1, index_scan_enum);

        let result = rule
            .apply(&mut ctx, &opt_node)
            .expect("Rule should apply successfully");
        assert!(result.is_some());
    }

    #[test]
    fn test_scan_with_filter_optimization_rule() {
        let rule = ScanWithFilterOptimizationRule;
        let mut ctx = create_test_context();

        let start_node = PlanNodeEnum::Start(
            crate::query::planner::plan::core::nodes::StartNode::new(),
        );
        let filter_node = crate::query::planner::plan::core::nodes::FilterNode::new(
            start_node,
            crate::core::Expression::Variable("col1 > 100".to_string()),
        )
        .expect("Filter node should be created successfully");
        let filter_opt_node = OptGroupNode::new(2, PlanNodeEnum::Filter(filter_node));

        let scan_node = PlanNodeEnum::ScanVertices(
            crate::query::planner::plan::core::nodes::ScanVerticesNode::new(1),
        );
        let mut opt_node = OptGroupNode::new(1, scan_node);
        opt_node.dependencies = vec![2];

        ctx.add_plan_node_and_group_node(2, &filter_opt_node);

        let result = rule
            .apply(&mut ctx, &opt_node)
            .expect("Rule should apply successfully");
        assert!(result.is_some());
    }
}
