//! 转换索引扫描为更优操作的规则
//!
//! 该规则识别 IndexScan 节点，
//! 并根据索引和过滤条件转换为更高效的索引扫描操作。

use crate::query::optimizer::plan::{OptContext, OptGroupNode, OptRule, Pattern, TransformResult};
use crate::query::optimizer::rule_patterns::PatternBuilder;
use crate::query::optimizer::rule_traits::BaseOptRule;
use crate::query::planner::plan::algorithms::IndexScan;
use crate::query::planner::plan::core::nodes::PlanNodeEnum;
use std::rc::Rc;
use std::cell::RefCell;

/// 转换索引扫描为更优操作的规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   IndexScan
/// ```
///
/// After:
/// ```text
///   IndexRangeScan (如果有范围条件)
///   或
///   IndexPrefixScan (如果有前缀条件)
/// ```
///
/// # 适用条件
///
/// - 索引扫描节点为索引扫描
/// - 存在有效的过滤条件或索引限制
#[derive(Debug)]
pub struct IndexScanRule;

impl OptRule for IndexScanRule {
    fn name(&self) -> &str {
        "IndexScanRule"
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

        if !index_scan.has_effective_filter() {
            return Ok(None);
        }

        let mut new_index_scan = index_scan.clone();
        optimize_index_scan(&mut new_index_scan);

        let mut new_index_scan_group_node = node_ref.clone();
        new_index_scan_group_node.plan_node = PlanNodeEnum::IndexScan(new_index_scan);

        let mut result = TransformResult::new();
        result.add_new_group_node(Rc::new(RefCell::new(new_index_scan_group_node)));
        
        Ok(Some(result))
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::index_scan()
    }
}

impl BaseOptRule for IndexScanRule {}

/// 优化索引扫描
fn optimize_index_scan(index_scan: &mut IndexScan) {
    if index_scan.scan_limits.is_empty() {
        return;
    }

    let has_range_condition = index_scan.scan_limits.iter().any(|limit| {
        limit.begin_value.is_some() || limit.end_value.is_some()
    });

    if has_range_condition {
        index_scan.scan_type = "RANGE".to_string();
    }
}
