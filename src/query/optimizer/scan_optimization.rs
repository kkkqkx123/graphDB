//! 扫描优化规则
//! 这些规则负责优化扫描操作，如带过滤条件的扫描和索引全扫描优化

use super::optimizer::OptimizerError;
use super::rule_patterns::PatternBuilder;
use super::rule_traits::BaseOptRule;
use crate::query::optimizer::optimizer::{OptContext, OptGroupNode, OptRule, Pattern};
use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;

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
                    if dep.plan_node().is_filter() {
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
    use crate::core::context::QueryContext;
    use crate::query::optimizer::optimizer::{OptContext, OptGroupNode};
    use crate::query::planner::plan::core::nodes::ScanVerticesNode;

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
    fn test_index_full_scan_rule() {
        let rule = IndexFullScanRule;
        let mut ctx = create_test_context();

        // 创建一个扫描节点（作为索引扫描的占位符）
        let scan_node = PlanNodeEnum::ScanVertices(
            crate::query::planner::plan::core::nodes::ScanVerticesNode::new(1),
        );
        let opt_node = OptGroupNode::new(1, scan_node);

        let result = rule
            .apply(&mut ctx, &opt_node)
            .expect("Rule should apply successfully");
        // 规则应该匹配扫描节点并尝试优化
        assert!(result.is_some());
    }

    #[test]
    fn test_scan_with_filter_optimization_rule() {
        let rule = ScanWithFilterOptimizationRule;
        let mut ctx = create_test_context();

        // 创建一个扫描顶点节点
        let scan_node = PlanNodeEnum::ScanVertices(
            crate::query::planner::plan::core::nodes::ScanVerticesNode::new(1),
        );
        let opt_node = OptGroupNode::new(1, scan_node);

        let result = rule
            .apply(&mut ctx, &opt_node)
            .expect("Rule should apply successfully");
        // 规则应该匹配扫描节点并尝试优化带过滤条件的扫描
        assert!(result.is_some());
    }
}
