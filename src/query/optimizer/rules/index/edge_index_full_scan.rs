//! 转换边索引全扫描为更优操作的规则
//!
//! 该规则识别 EdgeIndexFullScan 节点，
//! 并根据索引和过滤条件转换为更高效的索引扫描操作。

use crate::query::optimizer::plan::{OptContext, OptGroupNode, OptRule, Pattern, TransformResult};
use crate::query::optimizer::rule_patterns::PatternBuilder;
use crate::query::optimizer::rule_traits::BaseOptRule;
use crate::query::planner::plan::algorithms::IndexScan;
use crate::query::planner::plan::core::nodes::PlanNodeEnum;
use std::rc::Rc;
use std::cell::RefCell;

/// 转换边索引全扫描为更优操作的规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   EdgeIndexFullScan
/// ```
///
/// After:
/// ```text
///   EdgeIndexRangeScan (如果有范围条件)
///   或
///   EdgeIndexPrefixScan (如果有前缀条件)
/// ```
///
/// # 适用条件
///
/// - 索引扫描节点为边索引扫描
/// - 存在有效的过滤条件或索引限制
#[derive(Debug)]
pub struct EdgeIndexFullScanRule;

impl OptRule for EdgeIndexFullScanRule {
    fn name(&self) -> &str {
        "EdgeIndexFullScanRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> Result<Option<TransformResult>, crate::query::optimizer::engine::OptimizerError> {
        let node_ref = group_node.borrow();
        
        if !node_ref.plan_node.is_index_scan() {
            return Ok(None);
        }

        let index_scan = match node_ref.plan_node.as_index_scan() {
            Some(scan) => scan,
            None => return Ok(None),
        };

        if !index_scan.is_edge_scan() {
            return Ok(None);
        }

        optimize_edge_index_scan(index_scan, ctx, group_node)
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::single("IndexScan")
    }
}

impl BaseOptRule for EdgeIndexFullScanRule {}

/// 优化边索引扫描
fn optimize_edge_index_scan(
    index_scan: &IndexScan,
    _ctx: &mut OptContext,
    group_node: &Rc<RefCell<OptGroupNode>>,
) -> Result<Option<TransformResult>, crate::query::optimizer::engine::OptimizerError> {
    if index_scan.scan_limits.is_empty() {
        return Ok(None);
    }

    let mut new_index_scan = index_scan.clone();
    new_index_scan.scan_type = "RANGE".to_string();

    let mut new_index_scan_group_node = group_node.borrow().clone();
    new_index_scan_group_node.plan_node = PlanNodeEnum::IndexScan(new_index_scan);

    let mut result = TransformResult::new();
    result.add_new_group_node(Rc::new(RefCell::new(new_index_scan_group_node)));
    
    Ok(Some(result))
}
