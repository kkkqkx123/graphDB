//! 优化带过滤条件的扫描操作的规则
//!
//! 该规则识别带过滤条件的扫描操作，并尝试优化扫描策略。
//!
//! # 适用条件
//!
//! - 节点是 ScanVertices 或 ScanEdges 节点
//! - 子节点包含 Filter 节点

use crate::query::optimizer::plan::{OptContext, OptGroupNode, OptRule, Pattern, TransformResult, Result};
use crate::query::optimizer::rule_patterns::PatternBuilder;
use crate::query::optimizer::rule_traits::BaseOptRule;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

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
        node: &Rc<RefCell<OptGroupNode>>,
    ) -> Result<Option<TransformResult>> {
        let node_ref = node.borrow();
        if !node_ref.plan_node.is_scan_vertices() && !node_ref.plan_node.is_scan_edges() {
            return Ok(None);
        }
        if let Some(matched) = self.match_pattern(ctx, node)? {
            if matched.dependencies.len() >= 1 {
                for dep in &matched.dependencies {
                    if dep.borrow().plan_node.is_filter() {
                        break;
                    }
                }
            }
        }
        Ok(None)
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::with_dependency("ScanVertices", "Filter")
    }
}

impl BaseOptRule for ScanWithFilterOptimizationRule {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::QueryContext;
    use crate::query::optimizer::plan::{OptContext, OptGroupNode};
    use crate::query::planner::plan::PlanNodeEnum;

    fn create_test_context() -> OptContext {
        let query_context = Arc::new(QueryContext::default());
        OptContext::new(query_context)
    }

    #[test]
    fn test_scan_with_filter_optimization_rule() -> Result<()> {
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

        ctx.add_group_node(Rc::new(RefCell::new(filter_opt_node)))?;

        let result = rule
            .apply(&mut ctx, &Rc::new(RefCell::new(opt_node)))
            .expect("Rule should apply successfully");
        // 当前规则实现返回 Ok(None)，因为规则还没有完整实现
        assert!(result.is_none());
        Ok(())
    }
}
