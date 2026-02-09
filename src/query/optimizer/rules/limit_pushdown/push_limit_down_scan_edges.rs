//! 将LIMIT下推到扫描边操作的规则
//!
//! 该规则识别 Limit -> ScanEdges 模式，
//! 并将LIMIT值集成到ScanEdges操作中。

use crate::query::optimizer::plan::{OptContext, OptGroupNode, OptRule, Pattern, TransformResult};
use crate::query::optimizer::rule_patterns::PatternBuilder;
use crate::query::optimizer::rule_traits::BaseOptRule;
use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;
use std::rc::Rc;
use std::cell::RefCell;

/// 将LIMIT下推到扫描边操作的规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   Limit(100)
///       |
///   ScanEdges
/// ```
///
/// After:
/// ```text
///   ScanEdges(limit=100)
/// ```
///
/// # 适用条件
///
/// - 当前节点为Limit节点
/// - 子节点为ScanEdges节点
/// - Limit节点只有一个子节点
#[derive(Debug)]
pub struct PushLimitDownScanEdgesRule;

impl OptRule for PushLimitDownScanEdgesRule {
    fn name(&self) -> &str {
        "PushLimitDownScanEdgesRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> Result<Option<TransformResult>, crate::query::optimizer::engine::OptimizerError> {
        let node_ref = group_node.borrow();
        
        if !node_ref.plan_node.is_limit() {
            return Ok(None);
        }

        if node_ref.dependencies.len() != 1 {
            return Ok(None);
        }

        let child_id = node_ref.dependencies[0];
        let child_node = match ctx.find_group_node_by_id(child_id) {
            Some(node) => node,
            None => return Ok(None),
        };

        let child_ref = child_node.borrow();
        
        if !child_ref.plan_node.is_scan_edges() {
            return Ok(None);
        }

        let limit_value = match node_ref.plan_node.as_limit() {
            Some(limit) => limit.count(),
            None => return Ok(None),
        };

        if let Some(scan_edges) = child_ref.plan_node.as_scan_edges() {
            let mut new_scan_edges = scan_edges.clone();
            new_scan_edges.set_limit(limit_value);
            
            if let Some(output_var) = node_ref.plan_node.output_var() {
                new_scan_edges.set_output_var(output_var.clone());
            }

            let mut new_node = child_ref.clone();
            new_node.plan_node = PlanNodeEnum::ScanEdges(new_scan_edges);

            let mut result = TransformResult::new();
            result.add_new_group_node(Rc::new(RefCell::new(new_node)));
            return Ok(Some(result));
        }

        Ok(None)
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::with_dependency("Limit", "ScanEdges")
    }
}

impl BaseOptRule for PushLimitDownScanEdgesRule {}
