//! 索引优化规则
//! 这些规则负责优化索引操作，包括基于过滤条件的索引扫描优化和索引扫描操作本身的优化

use super::optimizer::OptimizerError;
use super::rule_traits::{BaseOptRule};
use super::rule_patterns::PatternBuilder;
use crate::query::optimizer::optimizer::{OptContext, OptGroupNode, OptRule, Pattern};
use crate::query::planner::plan::{PlanNodeKind, PlanNode};

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

        // 匹配模式以确定这是否为带有适用过滤器的边索引扫描
        if let Some(matched) = self.match_pattern(ctx, node)? {
            if matched.dependencies.len() >= 1 {
                // 检查是否有可以推入索引扫描的适用过滤器
                for dep in &matched.dependencies {
                    if dep.plan_node().kind() == PlanNodeKind::Filter {
                        // 在完整实现中，我们会将过滤条件合并到索引扫描中
                        // 以减少从索引检索的行数
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

        // 匹配模式以确定这是否为带有适用过滤器的标签索引扫描
        if let Some(matched) = self.match_pattern(ctx, node)? {
            if matched.dependencies.len() >= 1 {
                // 检查是否有可以推入索引扫描的适用过滤器
                for dep in &matched.dependencies {
                    if dep.plan_node().kind() == PlanNodeKind::Filter {
                        // 在完整实现中，我们会将过滤条件合并到索引扫描中
                        // 以减少从索引检索的行数
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

        // 匹配模式以确定这是否为边索引扫描
        if let Some(matched) = self.match_pattern(ctx, node)? {
            // 在完整实现中，我们会检查这是否为全索引扫描
            // （扫描整个索引而不带条件）并可能优化它
            // 例如，如果索引扫描覆盖所有数据而没有益处，我们可能
            // 将其转换为可能更高效的全表扫描
            Ok(Some(node.clone()))
        } else {
            Ok(None)
        }
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

        // 匹配模式以确定这是否为标签索引扫描
        if let Some(matched) = self.match_pattern(ctx, node)? {
            // 在完整实现中，我们会检查这是否为全索引扫描
            // （扫描整个索引而不带条件）并可能优化它
            // 例如，如果索引扫描覆盖所有数据而没有益处，我们可能
            // 将其转换为可能更高效的全表扫描
            Ok(Some(node.clone()))
        } else {
            Ok(None)
        }
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

        // 匹配模式并在可能时优化索引扫描
        if let Some(matched) = self.match_pattern(ctx, node)? {
            // 在完整实现中，我们会基于各种因素优化索引扫描：
            // - 索引选择性
            // - 数据分布
            // - 可用内存
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

        // 匹配模式以识别边索引扫描的UNION ALL
        if let Some(matched) = self.match_pattern(ctx, node)? {
            // 在完整实现中，我们会优化涉及边索引扫描的UNION ALL操作
            // 通过可能合并或重新排序它们
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

        // 匹配模式以识别标签索引扫描的UNION ALL
        if let Some(matched) = self.match_pattern(ctx, node)? {
            // 在完整实现中，我们会优化涉及标签索引扫描的UNION ALL操作
            // 通过可能合并或重新排序它们
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
        let index_scan_node = Box::new(IndexScan::new(1, "edge_type", vec!["prop1".to_string()], None, None));
        let opt_node = OptGroupNode::new(1, index_scan_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_optimize_tag_index_scan_by_filter_rule() {
        let rule = OptimizeTagIndexScanByFilterRule;
        let mut ctx = create_test_context();

        // 创建一个索引扫描节点
        let index_scan_node = Box::new(IndexScan::new(1, "tag1", vec!["prop1".to_string()], None, None));
        let opt_node = OptGroupNode::new(1, index_scan_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_edge_index_full_scan_rule() {
        let rule = EdgeIndexFullScanRule;
        let mut ctx = create_test_context();

        // 创建一个索引扫描节点
        let index_scan_node = Box::new(IndexScan::new(1, "edge_type", vec!["prop1".to_string()], None, None));
        let opt_node = OptGroupNode::new(1, index_scan_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_tag_index_full_scan_rule() {
        let rule = TagIndexFullScanRule;
        let mut ctx = create_test_context();

        // 创建一个索引扫描节点
        let index_scan_node = Box::new(IndexScan::new(1, "tag1", vec!["prop1".to_string()], None, None));
        let opt_node = OptGroupNode::new(1, index_scan_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_index_scan_rule() {
        let rule = IndexScanRule;
        let mut ctx = create_test_context();

        // 创建一个索引扫描节点
        let index_scan_node = Box::new(IndexScan::new(1, "tag1", vec!["prop1".to_string()], None, None));
        let opt_node = OptGroupNode::new(1, index_scan_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_union_all_edge_index_scan_rule() {
        let rule = UnionAllEdgeIndexScanRule;
        let mut ctx = create_test_context();

        // 创建一个索引扫描节点
        let index_scan_node = Box::new(IndexScan::new(1, "edge_type", vec!["prop1".to_string()], None, None));
        let opt_node = OptGroupNode::new(1, index_scan_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_union_all_tag_index_scan_rule() {
        let rule = UnionAllTagIndexScanRule;
        let mut ctx = create_test_context();

        // 创建一个索引扫描节点
        let index_scan_node = Box::new(IndexScan::new(1, "tag1", vec!["prop1".to_string()], None, None));
        let opt_node = OptGroupNode::new(1, index_scan_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        assert!(result.is_some());
    }
}